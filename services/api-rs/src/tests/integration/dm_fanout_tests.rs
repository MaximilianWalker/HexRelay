use super::*;
use axum::{extract::State as AxumState, http::HeaderMap, routing::post, Router};
use communication_core::{
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
    NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RelayPolicy,
    StaticPeerRegistry, StoragePolicy,
};
use std::{sync::LazyLock, time::Duration as StdDuration};

use crate::{
    domain::{
        dm::{
            forwarding::{
                forward_signature_payload, NodeForwardDmEnvelopeRequest, NODE_FORWARD_PATH,
            },
            outbound_forwarding::{retry_due_dm_outbound_forwards, DmOutboundForwardRetryConfig},
            outbound_forwarding::{
                spawn_dm_outbound_forward_retry_worker, DmOutboundForwardRetryWorkerConfig,
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

fn unique_message_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

const TEST_RETRY_STALE_ATTEMPT_SECONDS: i64 = 60;
static OUTBOUND_FORWARD_RETRY_TEST_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

struct SignedNodeDescriptor {
    descriptor: NodeDescriptor,
    private_key_pkcs8: Vec<u8>,
}

struct CapturedNodeForward {
    headers: HeaderMap,
    body: Vec<u8>,
}

struct NodeForwardCaptureState {
    sender: tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<CapturedNodeForward>>>,
    status: StatusCode,
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

fn api_node_state(
    local: &SignedNodeDescriptor,
    static_peers: Vec<NodeDescriptor>,
    pool: sqlx::PgPool,
) -> AppState {
    AppState::new(
        local.descriptor.node_id.clone(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "primary".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        None,
        "http://127.0.0.1:8081".to_string(),
        BTreeMap::from([(
            "primary".to_string(),
            "hexrelay-dev-signing-key-change-me".to_string(),
        )]),
        None,
        false,
        "Lax".to_string(),
        ApiRateLimitConfig {
            auth_challenge_per_window: 30,
            auth_verify_per_window: 30,
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            dm_dispatch_per_window: 120,
            dm_catch_up_per_window: 120,
            dm_ack_per_window: 600,
            dm_internal_forward_per_window: 240,
            window_seconds: 60,
        },
        false,
    )
    .with_public_identity_registration(true)
    .with_db_pool(pool)
    .with_local_node_identity(Some(LocalNodeIdentity {
        descriptor: local.descriptor.clone(),
        private_key_pkcs8: local.private_key_pkcs8.clone(),
    }))
    .with_static_peer_registry(
        StaticPeerRegistry::try_new(static_peers).expect("API node static peer registry"),
    )
}

async fn start_node_forward_capture(
) -> (String, tokio::sync::oneshot::Receiver<CapturedNodeForward>) {
    start_node_forward_capture_with_status(StatusCode::ACCEPTED).await
}

async fn bind_api_node() -> (String, tokio::net::TcpListener) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind API node");
    let addr = listener.local_addr().expect("API node address");

    (format!("http://{}", addr), listener)
}

fn spawn_api_node(
    listener: tokio::net::TcpListener,
    app: axum::Router,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}

async fn start_node_forward_capture_with_status(
    status: StatusCode,
) -> (String, tokio::sync::oneshot::Receiver<CapturedNodeForward>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind node forward capture server");
    let addr = listener.local_addr().expect("node forward capture address");
    let (tx, rx) = tokio::sync::oneshot::channel::<CapturedNodeForward>();
    let state = std::sync::Arc::new(NodeForwardCaptureState {
        sender: tokio::sync::Mutex::new(Some(tx)),
        status,
    });
    let app = Router::new()
        .route(NODE_FORWARD_PATH, post(capture_node_forward))
        .with_state(state);

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    (format!("http://{}", addr), rx)
}

async fn capture_node_forward(
    AxumState(state): AxumState<std::sync::Arc<NodeForwardCaptureState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> axum::http::StatusCode {
    if let Some(sender) = state.sender.lock().await.take() {
        let _ = sender.send(CapturedNodeForward {
            headers,
            body: body.to_vec(),
        });
    }

    state.status
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

async fn seed_due_failed_outbound_forward(
    pool: &sqlx::PgPool,
    sender: &str,
    recipient: &str,
    destination_node_id: &str,
    message_id: &str,
) {
    ensure_db_identity_key(pool, sender).await;
    dm_repo::record_dm_outbound_forward_queued(
        pool,
        &dm_repo::DmOutboundForwardWrite {
            sender_identity_id: sender,
            destination_node_id,
            message_id,
            thread_id: "thread-seeded-outbound-forward",
            recipient_identity_id: recipient,
            ciphertext: "enc:seeded-outbound-forward",
            source_device_id: Some("desktop-main"),
            delivery_cursor: 1,
        },
    )
    .await
    .expect("seed queued outbound forward");
    dm_repo::mark_dm_outbound_forward_failed(
        pool,
        sender,
        destination_node_id,
        message_id,
        "seeded retryable failure",
        Some(Utc::now() - Duration::seconds(1)),
    )
    .await
    .expect("seed failed outbound forward");
}

async fn delete_outbound_forward_record(
    pool: &sqlx::PgPool,
    sender: &str,
    destination_node_id: &str,
    message_id: &str,
) {
    sqlx::query(
        "
        DELETE FROM dm_outbound_forwarding_log
        WHERE sender_identity_id = $1
          AND destination_node_id = $2
          AND message_id = $3
        ",
    )
    .bind(sender)
    .bind(destination_node_id)
    .bind(message_id)
    .execute(pool)
    .await
    .expect("delete test outbound forward record");
}

async fn delete_retry_test_outbound_forward_records(pool: &sqlx::PgPool) {
    sqlx::query(
        "
        DELETE FROM dm_outbound_forwarding_log
        WHERE sender_identity_id LIKE 'usr-origin-retry-%'
           OR message_id LIKE 'msg-static-peer-retry%'
        ",
    )
    .execute(pool)
    .await
    .expect("delete prior retry test outbound forward records");
}

async fn wait_for_outbound_forward_state(
    pool: &sqlx::PgPool,
    sender: &str,
    destination_node_id: &str,
    message_id: &str,
    expected_state: &str,
) -> crate::models::DmOutboundForwardRecord {
    tokio::time::timeout(StdDuration::from_secs(5), async {
        loop {
            if let Some(record) = dm_repo::get_dm_outbound_forward_record(
                pool,
                sender,
                destination_node_id,
                message_id,
            )
            .await
            .expect("load outbound forward record while waiting")
            {
                if record.forwarding_state == expected_state {
                    return record;
                }
            }

            tokio::time::sleep(StdDuration::from_millis(25)).await;
        }
    })
    .await
    .expect("outbound forward record should reach expected state")
}

#[tokio::test]
async fn dm_delivery_metadata_retention_purges_only_expired_delivery_metadata() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let sender = unique_identity("usr-retention-sender");
    let recipient = unique_identity("usr-retention-recipient");
    let thread_id = dm_history_repo::direct_dm_thread_id(&sender, &recipient);
    let message_id = unique_message_id("msg-retention");
    let forwarded_message_id = unique_message_id("msg-retention-forwarded");
    let failed_message_id = unique_message_id("msg-retention-failed");
    let queued_message_id = unique_message_id("msg-retention-queued");

    ensure_db_identity_key(&pool, &sender).await;
    ensure_db_identity_key(&pool, &recipient).await;
    dm_history_repo::insert_dm_thread(
        &pool,
        dm_history_repo::DmThreadInsertParams {
            thread_id: &thread_id,
            kind: "dm",
            title: "Retention Test",
        },
    )
    .await
    .expect("insert retention dm thread");
    for identity_id in [&sender, &recipient] {
        dm_history_repo::insert_dm_thread_participant(
            &pool,
            dm_history_repo::DmThreadParticipantInsertParams {
                thread_id: &thread_id,
                identity_id,
                last_read_seq: 0,
            },
        )
        .await
        .expect("insert retention dm participant");
    }
    dm_history_repo::insert_dm_message(
        &pool,
        dm_history_repo::DmMessageInsertParams {
            message_id: &message_id,
            thread_id: &thread_id,
            author_id: &sender,
            seq: 1,
            ciphertext: "enc:retention-message",
            created_at: &Utc::now().to_rfc3339(),
            edited_at: None,
        },
    )
    .await
    .expect("insert retention dm message");
    dm_repo::upsert_dm_profile_device(
        &pool,
        &recipient,
        &DmProfileDeviceRecord {
            device_id: "phone-main".to_string(),
            device_secret_hash: "hash".to_string(),
            active: true,
            last_seen_epoch: Utc::now().timestamp(),
        },
    )
    .await
    .expect("insert retention profile device");

    let retention_cutoff = Utc::now();
    let expired_delivery_at = retention_cutoff - Duration::seconds(1);
    let expired_outbound_at = retention_cutoff - Duration::seconds(1);
    sqlx::query(
        "
        INSERT INTO dm_fanout_stream_heads (identity_id, latest_cursor, updated_at)
        VALUES ($1, 1, NOW())
        ON CONFLICT (identity_id) DO UPDATE SET latest_cursor = 1, updated_at = NOW()
        ",
    )
    .bind(&recipient)
    .execute(&pool)
    .await
    .expect("insert retention stream head");
    sqlx::query(
        "
        INSERT INTO dm_fanout_device_cursors (identity_id, device_id, cursor, updated_at)
        VALUES ($1, 'phone-main', 1, NOW())
        ON CONFLICT (identity_id, device_id) DO UPDATE SET cursor = 1, updated_at = NOW()
        ",
    )
    .bind(&recipient)
    .execute(&pool)
    .await
    .expect("insert retention device cursor");
    sqlx::query(
        "
        INSERT INTO dm_fanout_delivery_log (
            identity_id,
            cursor,
            thread_id,
            message_id,
            sender_identity_id,
            ciphertext,
            source_device_id,
            delivery_state,
            reachability_state,
            delivered_device_ids,
            created_at
        )
        VALUES ($1, 1, $2, $3, $4, 'enc:retention-message', 'desktop-main', 'acked', 'reachable', '[\"phone-main\"]'::jsonb, $5)
        ",
    )
    .bind(&recipient)
    .bind(&thread_id)
    .bind(&message_id)
    .bind(&sender)
    .bind(expired_delivery_at)
    .execute(&pool)
    .await
    .expect("insert retention delivery metadata");

    for (message_id, state, next_attempt_at) in [
        (forwarded_message_id.as_str(), "forwarded", None),
        (failed_message_id.as_str(), "failed", None),
        (
            queued_message_id.as_str(),
            "queued",
            Some(Utc::now() + Duration::minutes(5)),
        ),
    ] {
        sqlx::query(
            "
            INSERT INTO dm_outbound_forwarding_log (
                sender_identity_id,
                destination_node_id,
                message_id,
                thread_id,
                recipient_identity_id,
                ciphertext,
                source_device_id,
                delivery_cursor,
                forwarding_state,
                attempt_count,
                last_error,
                last_attempt_at,
                next_attempt_at,
                forwarded_at,
                created_at,
                updated_at
            )
            VALUES ($1, 'node-retention-peer', $2, $3, $4, 'enc:retention-outbound', 'desktop-main', 1, $5, 1, NULL, $6, $7, NULL, $6, $6)
            ",
        )
        .bind(&sender)
        .bind(message_id)
        .bind(&thread_id)
        .bind(&recipient)
        .bind(state)
        .bind(expired_outbound_at)
        .bind(next_attempt_at)
        .execute(&pool)
        .await
        .expect("insert retention outbound metadata");
    }

    let summary =
        dm_repo::purge_expired_dm_delivery_metadata(&pool, retention_cutoff, retention_cutoff)
            .await
            .expect("purge expired dm delivery metadata");
    assert!(summary.fanout_delivery_records_deleted >= 1);
    assert!(summary.outbound_forward_records_deleted >= 2);

    let delivery_rows = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM dm_fanout_delivery_log WHERE identity_id = $1 AND message_id = $2",
    )
    .bind(&recipient)
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("count purged delivery metadata");
    assert_eq!(delivery_rows, 0);

    let message_rows = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM dm_messages WHERE message_id = $1 AND ciphertext = 'enc:retention-message'",
    )
    .bind(&message_id)
    .fetch_one(&pool)
    .await
    .expect("count retained ciphertext history");
    assert_eq!(message_rows, 1);

    for removed_message_id in [&forwarded_message_id, &failed_message_id] {
        let rows = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dm_outbound_forwarding_log WHERE sender_identity_id = $1 AND message_id = $2",
        )
        .bind(&sender)
        .bind(removed_message_id)
        .fetch_one(&pool)
        .await
        .expect("count removed outbound metadata");
        assert_eq!(rows, 0);
    }

    let queued_rows = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM dm_outbound_forwarding_log WHERE sender_identity_id = $1 AND message_id = $2",
    )
    .bind(&sender)
    .bind(&queued_message_id)
    .fetch_one(&pool)
    .await
    .expect("count retained queued outbound metadata");
    assert_eq!(queued_rows, 1);

    delete_outbound_forward_record(&pool, &sender, "node-retention-peer", &queued_message_id).await;
}

#[tokio::test]
async fn fanout_dispatch_rate_limits_per_sender_identity() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let sender = unique_identity("usr-rate-sender");
    let recipient = unique_identity("usr-rate-recipient");
    let mut state = test_state_with_public_identity_registration().with_db_pool(pool.clone());
    state.rate_limits.dm_dispatch_per_window = 1;

    let sender_cookie = issue_db_session_cookie(&pool, &state, &sender).await;
    ensure_db_identity_key(&pool, &recipient).await;
    let app = build_app(state);

    for (index, expected_status) in [(0, StatusCode::OK), (1, StatusCode::TOO_MANY_REQUESTS)] {
        let request = Request::builder()
            .method("POST")
            .uri("/dm/fanout/dispatch")
            .header("content-type", "application/json")
            .header(
                "cookie",
                format!("hexrelay_session={sender_cookie}; hexrelay_csrf=test-csrf"),
            )
            .header("x-csrf-token", "test-csrf")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:rate-limited"}}"#,
                recipient,
                unique_message_id(&format!("msg-rate-{index}"))
            )))
            .expect("build rate-limited fanout request");

        let response = app
            .clone()
            .oneshot(request)
            .await
            .expect("rate-limited fanout response");
        assert_eq!(response.status(), expected_status);
    }
}

#[tokio::test]
async fn fanout_dispatch_forwards_between_two_local_api_nodes_over_http() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let sender = unique_identity("usr-two-node-sender");
    let recipient = unique_identity("usr-two-node-recipient");
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

    let (node_a_base_url, node_a_listener) = bind_api_node().await;
    let (node_b_base_url, node_b_listener) = bind_api_node().await;
    let node_a = signed_node_descriptor("node-a", "descriptor-node-a", &node_a_base_url);
    let node_b = signed_node_descriptor("node-b", "descriptor-node-b", &node_b_base_url);
    let node_a_state = api_node_state(&node_a, vec![node_b.descriptor.clone()], pool.clone());
    let sender_token = issue_db_session_cookie(&pool, &node_a_state, &sender).await;
    let node_b_state = api_node_state(&node_b, vec![node_a.descriptor.clone()], pool.clone());
    let node_b_handle = spawn_api_node(node_b_listener, build_app(node_b_state));
    let node_a_handle = spawn_api_node(node_a_listener, build_app(node_a_state));

    let message_id = unique_message_id("msg-two-node-http");
    let response = reqwest::Client::new()
        .post(format!("{node_a_base_url}/dm/fanout/dispatch"))
        .header("authorization", format!("Bearer {sender_token}"))
        .json(&serde_json::json!({
            "recipient_identity_id": recipient.as_str(),
            "message_id": message_id.as_str(),
            "ciphertext": "enc:two-node-http-smoke",
            "source_device_id": "desktop-main",
            "destination_node_id": node_b.descriptor.node_id.as_str(),
        }))
        .send()
        .await
        .expect("two-node fanout response");
    let status = response.status();
    let body = response
        .text()
        .await
        .expect("read two-node fanout response");
    assert_eq!(status.as_u16(), StatusCode::OK.as_u16(), "{body}");
    let payload: serde_json::Value =
        serde_json::from_str(&body).expect("decode two-node fanout response");
    assert_eq!(payload["status"], "accepted");
    assert_eq!(payload["reason_code"], "fanout_forwarded_to_static_peer");
    assert_eq!(payload["delivery_state"], "forwarded");

    let outbound = dm_repo::get_dm_outbound_forward_record(
        &pool,
        &sender,
        &node_b.descriptor.node_id,
        &message_id,
    )
    .await
    .expect("load two-node outbound forward")
    .expect("two-node outbound forward");
    assert_eq!(outbound.forwarding_state, "forwarded");
    assert_eq!(outbound.attempt_count, 1);
    assert!(outbound.last_error.is_none());

    let records = dm_repo::list_dm_fanout_delivery_records(&pool, &recipient)
        .await
        .expect("load node B delivery records");
    let accepted = records
        .iter()
        .find(|record| record.message_id == message_id)
        .expect("node B accepted forwarded envelope");
    assert_eq!(accepted.sender_identity_id, sender);
    assert_eq!(accepted.ciphertext, "enc:two-node-http-smoke");
    assert_eq!(accepted.source_device_id.as_deref(), Some("desktop-main"));
    assert_eq!(accepted.delivery_state, "pending_delivery");
    assert_eq!(accepted.reachability_state, "unknown");

    delete_outbound_forward_record(&pool, &sender, &node_b.descriptor.node_id, &message_id).await;
    node_a_handle.abort();
    node_b_handle.abort();
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
    let message_id = unique_message_id("msg-node-forward");
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
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let state = test_state_with_public_identity_registration().with_db_pool(pool.clone());
    let token = issue_db_session_cookie(&pool, &state, &sender).await;
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
    let message_id = unique_message_id("msg-static-peer-forward");

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{recipient}","message_id":"{message_id}","ciphertext":"enc:static-peer","source_device_id":"desktop-main","destination_node_id":"{}"}}"#,
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

    let record = dm_repo::get_dm_outbound_forward_record(
        &pool,
        &sender,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await
    .expect("load outbound forward record")
    .expect("outbound forward record");
    assert_eq!(record.sender_identity_id, sender);
    assert_eq!(record.destination_node_id, destination.descriptor.node_id);
    assert_eq!(record.message_id, message_id);
    assert_eq!(record.recipient_identity_id, recipient);
    assert_eq!(record.ciphertext, "enc:static-peer");
    assert_eq!(record.source_device_id.as_deref(), Some("desktop-main"));
    assert!(record.delivery_cursor > 0);
    assert_eq!(record.forwarding_state, "forwarded");
    assert_eq!(record.attempt_count, 1);
    assert!(record.last_error.is_none());

    delete_outbound_forward_record(&pool, &sender, &destination.descriptor.node_id, &message_id)
        .await;
}

#[tokio::test]
async fn fanout_dispatch_records_failed_outbound_destination_forward() {
    let (destination_base_url, capture_rx) =
        start_node_forward_capture_with_status(StatusCode::INTERNAL_SERVER_ERROR).await;
    let sender = unique_identity("usr-origin-sender");
    let recipient = unique_identity("usr-remote-recipient");
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let state = test_state_with_public_identity_registration().with_db_pool(pool.clone());
    let token = issue_db_session_cookie(&pool, &state, &sender).await;
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
    let message_id = unique_message_id("msg-static-peer-failed");

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{recipient}","message_id":"{message_id}","ciphertext":"enc:static-peer-failed","source_device_id":"desktop-main","destination_node_id":"{}"}}"#,
            destination.descriptor.node_id
        )))
        .expect("build destination-node fanout request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("destination-node fanout response");
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let response_body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read failed destination-node fanout body");
    let response_payload: serde_json::Value =
        serde_json::from_slice(&response_body).expect("decode failed fanout response");
    assert_eq!(response_payload["code"], "fanout_forwarding_failed");

    let captured = capture_rx
        .await
        .expect("capture failed node-forwarded request");
    let forwarded: NodeForwardDmEnvelopeRequest =
        serde_json::from_slice(&captured.body).expect("decode node forward body");
    assert_eq!(forwarded.sender_identity_id, sender);
    assert_eq!(forwarded.recipient_identity_id, recipient);
    assert_eq!(forwarded.ciphertext, "enc:static-peer-failed");

    let record = dm_repo::get_dm_outbound_forward_record(
        &pool,
        &sender,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await
    .expect("load failed outbound forward record")
    .expect("failed outbound forward record");
    assert_eq!(record.forwarding_state, "failed");
    assert_eq!(record.attempt_count, 1);
    assert!(record
        .last_error
        .as_deref()
        .is_some_and(|value| value.contains("500 Internal Server Error")));

    delete_outbound_forward_record(&pool, &sender, &destination.descriptor.node_id, &message_id)
        .await;
}

#[tokio::test]
async fn outbound_forward_retry_forwards_due_failed_static_peer_record() {
    let (destination_base_url, capture_rx) = start_node_forward_capture().await;
    let sender = unique_identity("usr-origin-retry-sender");
    let recipient = unique_identity("usr-remote-retry-recipient");
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let _retry_guard = OUTBOUND_FORWARD_RETRY_TEST_LOCK.lock().await;
    delete_retry_test_outbound_forward_records(&pool).await;
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
    let message_id = unique_message_id("msg-static-peer-retry");
    seed_due_failed_outbound_forward(
        &pool,
        &sender,
        &recipient,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await;
    let state = test_state_with_public_identity_registration()
        .with_db_pool(pool.clone())
        .with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }))
        .with_static_peer_registry(
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry"),
        );

    let summary = retry_due_dm_outbound_forwards(
        &state,
        DmOutboundForwardRetryConfig {
            limit: 10,
            max_attempts: 5,
            stale_attempt_seconds: TEST_RETRY_STALE_ATTEMPT_SECONDS,
        },
    )
    .await
    .expect("retry due outbound forwards");

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.attempted, 1);
    assert_eq!(summary.forwarded, 1);
    assert_eq!(summary.retryable_failed, 0);
    assert_eq!(summary.terminal_failed, 0);
    let captured = capture_rx
        .await
        .expect("capture retried node-forwarded request");
    let forwarded: NodeForwardDmEnvelopeRequest =
        serde_json::from_slice(&captured.body).expect("decode retried node forward body");
    assert_eq!(forwarded.sender_identity_id, sender);
    assert_eq!(forwarded.recipient_identity_id, recipient);
    assert_eq!(forwarded.ciphertext, "enc:seeded-outbound-forward");

    let record = dm_repo::get_dm_outbound_forward_record(
        &pool,
        &sender,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await
    .expect("load retried outbound forward record")
    .expect("retried outbound forward record");
    assert_eq!(record.forwarding_state, "forwarded");
    assert_eq!(record.attempt_count, 2);
    assert!(record.next_attempt_at.is_none());

    delete_outbound_forward_record(&pool, &sender, &destination.descriptor.node_id, &message_id)
        .await;
}

#[tokio::test]
async fn outbound_forward_retry_worker_drives_due_failed_static_peer_record() {
    let (destination_base_url, capture_rx) = start_node_forward_capture().await;
    let sender = unique_identity("usr-origin-retry-worker-sender");
    let recipient = unique_identity("usr-remote-retry-worker-recipient");
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let _retry_guard = OUTBOUND_FORWARD_RETRY_TEST_LOCK.lock().await;
    delete_retry_test_outbound_forward_records(&pool).await;
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
    let message_id = unique_message_id("msg-static-peer-retry-worker");
    seed_due_failed_outbound_forward(
        &pool,
        &sender,
        &recipient,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await;
    let state = test_state_with_public_identity_registration()
        .with_db_pool(pool.clone())
        .with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }))
        .with_static_peer_registry(
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry"),
        );

    let worker = spawn_dm_outbound_forward_retry_worker(
        state,
        DmOutboundForwardRetryWorkerConfig {
            retry: DmOutboundForwardRetryConfig {
                limit: 10,
                max_attempts: 5,
                stale_attempt_seconds: TEST_RETRY_STALE_ATTEMPT_SECONDS,
            },
            interval: StdDuration::from_millis(10),
        },
    );

    let captured = tokio::time::timeout(StdDuration::from_secs(5), capture_rx)
        .await
        .expect("worker should forward due outbound record")
        .expect("capture worker node-forwarded request");
    let forwarded: NodeForwardDmEnvelopeRequest =
        serde_json::from_slice(&captured.body).expect("decode worker retried node forward body");
    assert_eq!(forwarded.sender_identity_id, sender);
    assert_eq!(forwarded.recipient_identity_id, recipient);
    assert_eq!(forwarded.ciphertext, "enc:seeded-outbound-forward");

    let record = wait_for_outbound_forward_state(
        &pool,
        &sender,
        &destination.descriptor.node_id,
        &message_id,
        "forwarded",
    )
    .await;
    assert_eq!(record.attempt_count, 2);
    assert!(record.next_attempt_at.is_none());

    worker.abort();
    let _ = worker.await;
    delete_outbound_forward_record(&pool, &sender, &destination.descriptor.node_id, &message_id)
        .await;
}

#[tokio::test]
async fn outbound_forward_retry_reschedules_retryable_transport_failure() {
    let (destination_base_url, capture_rx) =
        start_node_forward_capture_with_status(StatusCode::INTERNAL_SERVER_ERROR).await;
    let sender = unique_identity("usr-origin-retry-sender");
    let recipient = unique_identity("usr-remote-retry-recipient");
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let _retry_guard = OUTBOUND_FORWARD_RETRY_TEST_LOCK.lock().await;
    delete_retry_test_outbound_forward_records(&pool).await;
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
    let message_id = unique_message_id("msg-static-peer-retry-failed");
    seed_due_failed_outbound_forward(
        &pool,
        &sender,
        &recipient,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await;
    let before_retry = Utc::now();
    let state = test_state_with_public_identity_registration()
        .with_db_pool(pool.clone())
        .with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }))
        .with_static_peer_registry(
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry"),
        );

    let summary = retry_due_dm_outbound_forwards(
        &state,
        DmOutboundForwardRetryConfig {
            limit: 10,
            max_attempts: 5,
            stale_attempt_seconds: TEST_RETRY_STALE_ATTEMPT_SECONDS,
        },
    )
    .await
    .expect("retry due outbound forwards");

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.attempted, 1);
    assert_eq!(summary.forwarded, 0);
    assert_eq!(summary.retryable_failed, 1);
    assert_eq!(summary.terminal_failed, 0);
    let captured = capture_rx
        .await
        .expect("capture failed retried node-forwarded request");
    let forwarded: NodeForwardDmEnvelopeRequest =
        serde_json::from_slice(&captured.body).expect("decode retried node forward body");
    assert_eq!(forwarded.ciphertext, "enc:seeded-outbound-forward");

    let record = dm_repo::get_dm_outbound_forward_record(
        &pool,
        &sender,
        &destination.descriptor.node_id,
        &message_id,
    )
    .await
    .expect("load failed retried outbound forward record")
    .expect("failed retried outbound forward record");
    assert_eq!(record.forwarding_state, "failed");
    assert_eq!(record.attempt_count, 2);
    assert!(record
        .last_error
        .as_deref()
        .is_some_and(|value| value.contains("500 Internal Server Error")));
    assert!(record
        .next_attempt_at
        .is_some_and(|value| value > before_retry));

    delete_outbound_forward_record(&pool, &sender, &destination.descriptor.node_id, &message_id)
        .await;
}

#[tokio::test]
async fn outbound_forward_retry_stops_when_destination_policy_is_not_configured() {
    let sender = unique_identity("usr-origin-retry-sender");
    let recipient = unique_identity("usr-remote-retry-recipient");
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let _retry_guard = OUTBOUND_FORWARD_RETRY_TEST_LOCK.lock().await;
    delete_retry_test_outbound_forward_records(&pool).await;
    let destination_node_id = "node-missing";
    let message_id = unique_message_id("msg-static-peer-retry-terminal");
    seed_due_failed_outbound_forward(&pool, &sender, &recipient, destination_node_id, &message_id)
        .await;
    let state = test_state_with_public_identity_registration().with_db_pool(pool.clone());

    let summary = retry_due_dm_outbound_forwards(
        &state,
        DmOutboundForwardRetryConfig {
            limit: 10,
            max_attempts: 5,
            stale_attempt_seconds: TEST_RETRY_STALE_ATTEMPT_SECONDS,
        },
    )
    .await
    .expect("retry due outbound forwards");

    assert_eq!(summary.scanned, 1);
    assert_eq!(summary.attempted, 1);
    assert_eq!(summary.forwarded, 0);
    assert_eq!(summary.retryable_failed, 0);
    assert_eq!(summary.terminal_failed, 1);

    let record =
        dm_repo::get_dm_outbound_forward_record(&pool, &sender, destination_node_id, &message_id)
            .await
            .expect("load terminal outbound forward record")
            .expect("terminal outbound forward record");
    assert_eq!(record.forwarding_state, "failed");
    assert_eq!(record.attempt_count, 2);
    assert!(record
        .last_error
        .as_deref()
        .is_some_and(|value| value.contains("static peer route unavailable")));
    assert!(record.next_attempt_at.is_none());

    let due = dm_repo::list_due_dm_outbound_forward_records(&pool, 10, 5, 0)
        .await
        .expect("list due outbound forwards after terminal failure");
    assert!(due.iter().all(|record| record.message_id != message_id));

    delete_outbound_forward_record(&pool, &sender, destination_node_id, &message_id).await;
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
    let message_id = unique_message_id("msg-catch-up");

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
        .header(
            "authorization",
            format!("Bearer {}", tokens[sender.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:abcd1234"}}"#,
            recipient, message_id
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
    let message_id = unique_message_id("msg-source-device");

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
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:abcd9999","source_device_id":"desktop-main"}}"#,
            recipient, message_id
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
    assert_eq!(catch_up_payload["items"][0]["message_id"], message_id);
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
    let message_id = unique_message_id("msg-no-active-device");

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header(
            "authorization",
            format!("Bearer {}", tokens[sender.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:abcd5555"}}"#,
            recipient, message_id
        )))
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
    let seed_message_id = unique_message_id("msg-seq-seed");
    let first_message_id = unique_message_id("msg-seq-a");
    let second_message_id = unique_message_id("msg-seq-b");
    dm_history_repo::insert_dm_message_in_tx(
        &mut setup_tx,
        dm_history_repo::DmMessageInsertParams {
            message_id: &seed_message_id,
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
    let first_message_id_for_task = first_message_id.clone();
    let second_message_id_for_task = second_message_id.clone();

    let (first, second) = tokio::join!(
        async move {
            let mut tx = pool_a.begin().await.expect("begin first tx");
            let seq = dm_history_repo::next_dm_message_seq_in_tx(&mut tx, &thread_a)
                .await
                .expect("allocate first seq");
            dm_history_repo::insert_dm_message_in_tx(
                &mut tx,
                dm_history_repo::DmMessageInsertParams {
                    message_id: &first_message_id_for_task,
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
                    message_id: &second_message_id_for_task,
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
    let message_id = unique_message_id("msg-policy-blocked");

    let request = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header(
            "authorization",
            format!("Bearer {}", tokens[sender.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:block"}}"#,
            recipient, message_id
        )))
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
    let message_id = unique_message_id("msg-shared-server");

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
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:shared"}}"#,
            recipient, message_id
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
