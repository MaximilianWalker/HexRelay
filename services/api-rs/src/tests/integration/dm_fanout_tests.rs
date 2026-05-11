use super::*;
use axum::{extract::State as AxumState, http::HeaderMap, routing::post, Router};
use communication_core::{
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
    NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RelayPolicy,
    StaticPeerRegistry, StoragePolicy,
};

use crate::{
    domain::{
        dm::{
            forwarding::{
                forward_signature_payload, NodeForwardDmEnvelopeRequest, NODE_FORWARD_PATH,
            },
            validation::DM_OFFLINE_DELIVERY_MODE,
        },
        node_identity::LocalNodeIdentity,
    },
    infra::db::repos::dm_repo,
    models::{DmPolicy, DmProfileDeviceRecord},
};

fn device_secret(device_id: &str) -> String {
    format!("secret-{device_id}")
}

struct SignedNodeDescriptor {
    descriptor: NodeDescriptor,
    private_key_pkcs8: Vec<u8>,
}

struct CapturedNodeForward {
    headers: HeaderMap,
    body: Vec<u8>,
}

fn signed_node_descriptor(
    node_id: &str,
    descriptor_id: &str,
    address: &str,
) -> SignedNodeDescriptor {
    let pkcs8 =
        Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate node descriptor key");
    let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive node public key");
    let now = Utc::now().timestamp();
    let mut descriptor = NodeDescriptor {
        node_id: node_id.to_string(),
        node_public_key: public_key,
        descriptor_id: descriptor_id.to_string(),
        issued_at_epoch_seconds: now - 1,
        expires_at_epoch_seconds: now + 300,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy: PeeringPolicy::StaticAllowlist,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec![address.to_string()],
        supported_protocols: vec!["hexrelay-node-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: None,
        signature: NodeSignature {
            algorithm: NodeSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    };
    descriptor.signature.value =
        sign_descriptor_ed25519_pkcs8(&descriptor, pkcs8.as_ref()).expect("sign descriptor");

    SignedNodeDescriptor {
        descriptor,
        private_key_pkcs8: pkcs8.as_ref().to_vec(),
    }
}

fn signed_node_forward_body(
    origin: &SignedNodeDescriptor,
    destination_node_id: &str,
    sender_identity_id: &str,
    recipient_identity_id: &str,
    message_id: &str,
) -> (Vec<u8>, String, String, String) {
    let request = NodeForwardDmEnvelopeRequest {
        route_kind: "static_peer_direct".to_string(),
        origin_node_descriptor: origin.descriptor.clone(),
        destination_node_id: destination_node_id.to_string(),
        relay_node_id: None,
        message_id: message_id.to_string(),
        thread_id: "thread-origin-forward".to_string(),
        sender_identity_id: sender_identity_id.to_string(),
        recipient_identity_id: recipient_identity_id.to_string(),
        ciphertext: "enc:node-forwarded-ciphertext".to_string(),
        source_device_id: Some("desktop-main".to_string()),
        accepted_at: Utc::now().to_rfc3339(),
        delivery_cursor: 1,
        target_device_ids: vec!["phone-main".to_string()],
    };
    let body = serde_json::to_vec(&request).expect("encode node forward request");
    let timestamp = Utc::now().timestamp().to_string();
    let nonce = format!("nonce-{}", Uuid::new_v4().simple());
    let key_pair =
        Ed25519KeyPair::from_pkcs8(&origin.private_key_pkcs8).expect("decode origin node key");
    let signature = hex::encode(key_pair.sign(&forward_signature_payload(
        "POST",
        NODE_FORWARD_PATH,
        &timestamp,
        &nonce,
        &body,
    )));

    (body, timestamp, nonce, signature)
}

async fn start_node_forward_capture(
) -> (String, tokio::sync::oneshot::Receiver<CapturedNodeForward>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind node forward capture server");
    let addr = listener.local_addr().expect("node forward capture address");
    let (tx, rx) = tokio::sync::oneshot::channel::<CapturedNodeForward>();
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));
    let app = Router::new()
        .route(NODE_FORWARD_PATH, post(capture_node_forward))
        .with_state(state);

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    (format!("http://{}", addr), rx)
}

async fn capture_node_forward(
    AxumState(sender): AxumState<
        std::sync::Arc<
            tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<CapturedNodeForward>>>,
        >,
    >,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> axum::http::StatusCode {
    if let Some(sender) = sender.lock().await.take() {
        let _ = sender.send(CapturedNodeForward {
            headers,
            body: body.to_vec(),
        });
    }

    axum::http::StatusCode::ACCEPTED
}

async fn set_dm_policy_anyone(app: axum::Router, token: &str) -> axum::Router {
    let request = Request::builder()
        .method("POST")
        .uri("/dm/privacy-policy")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("dm policy update response");
    assert_eq!(response.status(), StatusCode::OK);
    app
}

#[tokio::test]
async fn node_forward_endpoint_accepts_authenticated_static_peer_envelope() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let sender = unique_identity("usr-node-forward-sender");
    let recipient = unique_identity("usr-node-forward-recipient");
    ensure_db_identity_key(&pool, &sender).await;
    ensure_db_identity_key(&pool, &recipient).await;
    dm_repo::upsert_dm_policy(
        &pool,
        &recipient,
        &DmPolicy {
            inbound_policy: "anyone".to_string(),
            offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
        },
    )
    .await
    .expect("set recipient DM policy");
    dm_repo::upsert_dm_profile_device(
        &pool,
        &recipient,
        &DmProfileDeviceRecord {
            device_id: "phone-main".to_string(),
            device_secret_hash: "test-secret-hash".to_string(),
            active: true,
            last_seen_epoch: Utc::now().timestamp(),
        },
    )
    .await
    .expect("upsert recipient profile device");

    let local = signed_node_descriptor(
        TEST_NODE_FINGERPRINT,
        "descriptor-local",
        "https://local.example",
    );
    let origin =
        signed_node_descriptor("node-origin", "descriptor-origin", "https://origin.example");
    let state = test_state_with_public_identity_registration()
        .with_db_pool(pool.clone())
        .with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }))
        .with_static_peer_registry(
            StaticPeerRegistry::try_new(vec![origin.descriptor.clone()]).expect("registry"),
        );
    let app = build_app(state);
    let message_id = format!("msg-node-forward-{}", Uuid::new_v4().simple());
    let (body, timestamp, nonce, signature) = signed_node_forward_body(
        &origin,
        &local.descriptor.node_id,
        &sender,
        &recipient,
        &message_id,
    );

    let request = Request::builder()
        .method("POST")
        .uri(NODE_FORWARD_PATH)
        .header("content-type", "application/json")
        .header("x-hexrelay-node-id", origin.descriptor.node_id.as_str())
        .header(
            "x-hexrelay-node-descriptor-id",
            origin.descriptor.descriptor_id.as_str(),
        )
        .header("x-hexrelay-node-signature-algorithm", "ed25519")
        .header("x-hexrelay-node-signature-timestamp", timestamp)
        .header("x-hexrelay-node-signature-nonce", nonce)
        .header("x-hexrelay-node-signature", signature)
        .body(Body::from(body))
        .expect("build node forward request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("node forward response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read node forward body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["reason_code"], "fanout_pending_delivery");

    let records = dm_repo::list_dm_fanout_delivery_records(&pool, &recipient)
        .await
        .expect("load delivery records");
    let record = records
        .iter()
        .find(|record| record.message_id == message_id)
        .expect("forwarded delivery record");
    assert_eq!(record.sender_identity_id, sender);
    assert_eq!(record.ciphertext, "enc:node-forwarded-ciphertext");
    assert_eq!(record.source_device_id.as_deref(), Some("desktop-main"));
}

#[tokio::test]
async fn fanout_dispatch_forwards_to_explicit_destination_node() {
    let (destination_base_url, capture_rx) = start_node_forward_capture().await;
    let sender = unique_identity("usr-origin-sender");
    let recipient = unique_identity("usr-remote-recipient");
    let (_app, tokens, state) = app_with_sessions_and_state(&[sender.as_str()]);
    let local = signed_node_descriptor(
        TEST_NODE_FINGERPRINT,
        "descriptor-local",
        "https://local.example",
    );
    let destination = signed_node_descriptor(
        "node-destination",
        "descriptor-destination",
        &destination_base_url,
    );
    let state = state
        .with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }))
        .with_static_peer_registry(
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry"),
        );
    let app = build_app(state);

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{recipient}","message_id":"msg-static-peer-forward","ciphertext":"enc:static-peer","source_device_id":"desktop-main","destination_node_id":"{}"}}"#,
            destination.descriptor.node_id
        )))
        .expect("build destination-node fanout request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("destination-node fanout response");
    assert_eq!(response.status(), StatusCode::OK);
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read destination-node fanout body");
    let response_payload: serde_json::Value =
        serde_json::from_slice(&response_body).expect("decode fanout response");
    assert_eq!(response_payload["status"], "accepted");
    assert_eq!(
        response_payload["reason_code"],
        "fanout_forwarded_to_static_peer"
    );
    assert_eq!(response_payload["delivery_state"], "forwarded");

    let captured = capture_rx.await.expect("capture node-forwarded request");
    assert_eq!(
        captured
            .headers
            .get("x-hexrelay-node-id")
            .and_then(|value| value.to_str().ok()),
        Some(local.descriptor.node_id.as_str())
    );
    let forwarded: NodeForwardDmEnvelopeRequest =
        serde_json::from_slice(&captured.body).expect("decode node forward body");
    assert_eq!(
        forwarded.destination_node_id,
        destination.descriptor.node_id
    );
    assert_eq!(forwarded.sender_identity_id, sender);
    assert_eq!(forwarded.recipient_identity_id, recipient);
    assert_eq!(forwarded.ciphertext, "enc:static-peer");
    assert_eq!(forwarded.source_device_id.as_deref(), Some("desktop-main"));
}

#[tokio::test]
async fn fanout_dispatch_accepts_for_catch_up_without_claiming_active_delivery() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    for (device_id, active) in [
        ("desktop-main", true),
        ("phone-main", true),
        ("tablet-idle", false),
    ] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/dm/profile-devices/heartbeat")
            .header(
                "authorization",
                format!("Bearer {}", tokens[recipient.as_str()]),
            )
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","device_secret":"{}","active":{active}}}"#,
                device_secret(device_id)
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-1001","ciphertext":"enc:abcd1234"}}"#,
            recipient
        )))
        .expect("build fanout request");
    let fanout_response = app
        .clone()
        .oneshot(fanout_request)
        .await
        .expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["delivery_state"], "pending_delivery");
    assert_eq!(payload["reachability_state"], "unknown");
    assert_eq!(payload["reason_code"], "fanout_pending_delivery");
    assert_eq!(payload["fanout_count"], 0);

    let delivered = payload["delivered_device_ids"]
        .as_array()
        .expect("delivered array");
    assert!(delivered.is_empty());
    assert_eq!(
        payload["skipped_device_ids"],
        serde_json::json!(["tablet-idle"])
    );
}

#[tokio::test]
async fn fanout_dispatch_does_not_skip_recipient_device_matching_source_device_id() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    for device_id in ["desktop-main", "phone-main"] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/dm/profile-devices/heartbeat")
            .header(
                "authorization",
                format!("Bearer {}", tokens[recipient.as_str()]),
            )
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","device_secret":"{}","active":true}}"#,
                device_secret(device_id)
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-1002","ciphertext":"enc:abcd9999","source_device_id":"desktop-main"}}"#,
            recipient
        )))
        .expect("build fanout request");
    let fanout_response = app
        .clone()
        .oneshot(fanout_request)
        .await
        .expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["delivery_state"], "pending_delivery");
    assert_eq!(payload["reachability_state"], "unknown");
    assert_eq!(payload["fanout_count"], 0);
    assert!(payload["delivered_device_ids"]
        .as_array()
        .expect("delivered array")
        .is_empty());
    assert!(payload["skipped_device_ids"]
        .as_array()
        .expect("skipped array")
        .is_empty());

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}"}}"#,
            device_secret("desktop-main")
        )))
        .expect("build fanout catch-up request");
    let catch_up_response = app.oneshot(catch_up).await.expect("catch-up response");
    assert_eq!(catch_up_response.status(), StatusCode::OK);

    let catch_up_body = to_bytes(catch_up_response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let catch_up_payload: serde_json::Value =
        serde_json::from_slice(&catch_up_body).expect("decode catch-up payload");
    assert_eq!(catch_up_payload["replay_count"], 1);
    assert_eq!(catch_up_payload["items"][0]["message_id"], "msg-1002");
}

#[tokio::test]
async fn fanout_dispatch_blocks_when_no_active_devices_registered() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"recipient_identity_id":"{}","message_id":"msg-1003","ciphertext":"enc:abcd5555"}}"#, recipient)))
        .expect("build fanout request");
    let fanout_response = app.oneshot(fanout_request).await.expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["reason_code"], "fanout_pending_delivery");
    assert_eq!(payload["delivery_state"], "pending_delivery");
    assert_eq!(payload["reachability_state"], "unreachable");
    assert_eq!(payload["fanout_count"], 0);
}

#[tokio::test]
async fn dm_message_seq_allocation_is_serialized_per_thread() {
    let sender = unique_identity("usr-seq-sender");
    let recipient = unique_identity("usr-seq-recipient");
    let Some((_app, _tokens, pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };

    let mut setup_tx = pool.begin().await.expect("begin seed transaction");
    let thread_id =
        dm_history_repo::ensure_direct_dm_thread_in_tx(&mut setup_tx, &sender, &recipient)
            .await
            .expect("ensure dm thread");
    dm_history_repo::insert_dm_message_in_tx(
        &mut setup_tx,
        dm_history_repo::DmMessageInsertParams {
            message_id: "msg-seq-seed",
            thread_id: &thread_id,
            author_id: &sender,
            seq: 1,
            ciphertext: "enc:seed",
            created_at: "2026-01-01T00:00:00Z",
            edited_at: None,
        },
    )
    .await
    .expect("seed first dm message");
    setup_tx.commit().await.expect("commit seed transaction");

    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let thread_a = thread_id.clone();
    let thread_b = thread_id.clone();
    let sender_a = sender.clone();
    let sender_b = sender.clone();

    let (first, second) = tokio::join!(
        async move {
            let mut tx = pool_a.begin().await.expect("begin first tx");
            let seq = dm_history_repo::next_dm_message_seq_in_tx(&mut tx, &thread_a)
                .await
                .expect("allocate first seq");
            dm_history_repo::insert_dm_message_in_tx(
                &mut tx,
                dm_history_repo::DmMessageInsertParams {
                    message_id: "msg-seq-a",
                    thread_id: &thread_a,
                    author_id: &sender_a,
                    seq,
                    ciphertext: "enc:a",
                    created_at: "2026-01-01T00:00:01Z",
                    edited_at: None,
                },
            )
            .await
            .expect("insert first concurrent message");
            tx.commit().await.expect("commit first tx");
            seq
        },
        async move {
            let mut tx = pool_b.begin().await.expect("begin second tx");
            let seq = dm_history_repo::next_dm_message_seq_in_tx(&mut tx, &thread_b)
                .await
                .expect("allocate second seq");
            dm_history_repo::insert_dm_message_in_tx(
                &mut tx,
                dm_history_repo::DmMessageInsertParams {
                    message_id: "msg-seq-b",
                    thread_id: &thread_b,
                    author_id: &sender_b,
                    seq,
                    ciphertext: "enc:b",
                    created_at: "2026-01-01T00:00:02Z",
                    edited_at: None,
                },
            )
            .await
            .expect("insert second concurrent message");
            tx.commit().await.expect("commit second tx");
            seq
        }
    );

    let mut seqs = vec![first, second];
    seqs.sort_unstable();
    assert_eq!(seqs, vec![2, 3]);
}

#[tokio::test]
async fn fanout_dispatch_requires_durable_storage() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-no-db","ciphertext":"enc:no-db"}"#,
        ))
        .expect("build no-db fanout request");
    let response = app.oneshot(request).await.expect("no-db fanout response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read no-db fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode no-db payload");
    assert_eq!(payload["code"], "storage_unavailable");
}

#[tokio::test]
async fn fanout_dispatch_blocks_when_recipient_policy_disallows_sender() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"recipient_identity_id":"{}","message_id":"msg-policy-blocked","ciphertext":"enc:block"}}"#, recipient)))
        .expect("build fanout request");
    let response = app.oneshot(request).await.expect("fanout response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "fanout_policy_blocked");
    assert_eq!(payload["delivery_state"], "rejected");
    assert_eq!(payload["reachability_state"], "blocked");
}

#[tokio::test]
async fn fanout_dispatch_allows_when_recipient_policy_is_same_server_and_membership_is_shared() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        sender.as_str(),
        false,
        false,
        0,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        recipient.as_str(),
        false,
        false,
        0,
    )
    .await;

    let policy_request = Request::builder()
        .method("POST")
        .uri("/dm/privacy-policy")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[recipient.as_str()]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"same_server"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let heartbeat = Request::builder()
        .method("POST")
        .uri("/dm/profile-devices/heartbeat")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[recipient.as_str()]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}","active":true}}"#,
            device_secret("desktop-main")
        )))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[sender.as_str()]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-shared-server","ciphertext":"enc:shared"}}"#,
            recipient
        )))
        .expect("build fanout request");
    let response = app.oneshot(request).await.expect("fanout response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");
    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["reason_code"], "fanout_pending_delivery");
    assert_eq!(payload["delivery_state"], "pending_delivery");
    assert_eq!(payload["reachability_state"], "unknown");
    assert_eq!(payload["fanout_count"], 0);
}

#[tokio::test]
async fn fanout_dispatch_rejects_invalid_payload() {
    let sender = unique_identity("usr-nora-k");
    let Some((app, tokens, _pool)) = app_with_database_and_sessions(&[sender.as_str()]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"bad id","message_id":"msg-invalid","ciphertext":"enc:test"}"#,
        ))
        .expect("build invalid fanout dispatch request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid fanout dispatch response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid fanout dispatch body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid fanout dispatch body");
    assert_eq!(payload["code"], "fanout_invalid");
}

#[tokio::test]
async fn fanout_dispatch_rejects_unknown_recipient_identity() {
    let sender = unique_identity("usr-nora-k");
    let Some((app, tokens, _pool)) = app_with_database_and_sessions(&[sender.as_str()]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-missing-recipient","message_id":"msg-unknown","ciphertext":"enc:test"}"#,
        ))
        .expect("build unknown-recipient fanout dispatch request");

    let response = app
        .oneshot(request)
        .await
        .expect("unknown-recipient fanout dispatch response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read unknown-recipient fanout dispatch body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode unknown-recipient fanout dispatch body");
    assert_eq!(payload["code"], "fanout_invalid");
    assert_eq!(
        payload["message"],
        "recipient_identity_id must reference a registered identity"
    );
}
