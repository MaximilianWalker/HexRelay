use super::*;
use communication_core::{
    lan_discovery_signing_payload, LanDiscoveryAdvertisement, LAN_DISCOVERY_SCOPE,
    LAN_DISCOVERY_TTL_SECONDS,
};

#[tokio::test]
async fn announces_lan_presence_and_lists_policy_eligible_peer() {
    let (app, tokens, state) = app_with_sessions_and_state(&["usr-nora-k", "usr-jules-p"]);
    seed_accepted_friendship(&state, "usr-nora-k", "usr-jules-p");

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let announce_body = to_bytes(announce_response.into_body(), usize::MAX)
        .await
        .expect("read lan announce body");
    let announce_payload: serde_json::Value =
        serde_json::from_slice(&announce_body).expect("decode lan announce body");
    assert_eq!(announce_payload["scope"], "lan_subnet");
    assert_eq!(announce_payload["ttl_seconds"], 120);
    assert!(announce_payload["expires_at"].as_str().is_some());

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .clone()
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["identity_id"], "usr-jules-p");
    assert_eq!(items[0]["endpoint_hints"][0], "udp://192.168.1.12:4040");
    assert!(items[0]["expires_at"].as_str().is_some());
}

#[tokio::test]
async fn hides_lan_peer_when_peer_policy_disallows_sender() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert!(items.is_empty());
}

#[tokio::test]
async fn preflight_prefers_lan_fast_path_when_peer_is_discovered() {
    let (app, tokens, state) = app_with_sessions_and_state(&["usr-nora-k", "usr-jules-p"]);
    seed_accepted_friendship(&state, "usr-nora-k", "usr-jules-p");

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");
    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let preflight_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");

    let preflight_response = app
        .oneshot(preflight_request)
        .await
        .expect("preflight response");
    assert_eq!(preflight_response.status(), StatusCode::OK);

    let body = to_bytes(preflight_response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "preflight_ok_lan");
}

#[tokio::test]
async fn rejects_invalid_lan_discovery_announce_request() {
    let (app, tokens) = app_with_sessions(&["usr-jules-p"]);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"endpoint_hints":[]}"#))
        .expect("build invalid lan announce request");

    let response = app
        .oneshot(announce_request)
        .await
        .expect("invalid lan announce response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid lan announce body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid lan announce body");
    assert_eq!(payload["code"], "lan_discovery_invalid");
}

#[tokio::test]
async fn rejects_non_local_lan_discovery_endpoint_hints() {
    let (app, tokens) = app_with_sessions(&["usr-jules-p"]);

    for body in [
        r#"{"endpoint_hints":["udp://8.8.8.8:4040"]}"#,
        r#"{"endpoint_hints":["udp://peer.local:4040"]}"#,
        r#"{"endpoint_hints":["udp://192.168.1.12:0"]}"#,
    ] {
        let announce_request = Request::builder()
            .method("POST")
            .uri("/v1/dm/connectivity/lan-discovery/announce")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .header("content-type", "application/json")
            .body(Body::from(body))
            .expect("build invalid lan announce request");

        let response = app
            .clone()
            .oneshot(announce_request)
            .await
            .expect("invalid lan announce response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read invalid lan announce body");
        let payload: serde_json::Value =
            serde_json::from_slice(&body).expect("decode invalid lan announce body");
        assert_eq!(payload["code"], "lan_discovery_invalid");
    }
}

#[tokio::test]
async fn prunes_expired_lan_presence_and_preflight_uses_non_lan_ready_path() {
    let (app, tokens, state) = app_with_sessions_and_state(&["usr-nora-k", "usr-jules-p"]);
    seed_accepted_friendship(&state, "usr-nora-k", "usr-jules-p");

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    state
        .dm_lan_presence
        .write()
        .expect("acquire dm lan presence write lock")
        .insert(
            "usr-jules-p".to_string(),
            crate::models::DmLanPresenceRecord {
                identity_id: "usr-jules-p".to_string(),
                endpoint_hints: vec!["udp://192.168.1.12:4040".to_string()],
                last_seen_epoch: Utc::now().timestamp() - 10_000,
                expires_at_epoch: Utc::now().timestamp() - 9_880,
            },
        );

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");
    let peers_response = app
        .clone()
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);
    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");
    assert!(peers_payload["items"]
        .as_array()
        .expect("peers items array")
        .is_empty());

    let preflight_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");
    let preflight_response = app
        .oneshot(preflight_request)
        .await
        .expect("preflight response");
    assert_eq!(preflight_response.status(), StatusCode::OK);
    let preflight_body = to_bytes(preflight_response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let preflight_payload: serde_json::Value =
        serde_json::from_slice(&preflight_body).expect("decode preflight body");
    assert_eq!(preflight_payload["status"], "ready");
    assert_eq!(preflight_payload["reason_code"], "preflight_ok");
}

#[tokio::test]
async fn policy_allowed_untrusted_peer_does_not_enable_lan_fast_path() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");
    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");
    let peers_response = app
        .clone()
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);
    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");
    assert!(peers_payload["items"]
        .as_array()
        .expect("peers items array")
        .is_empty());

    let preflight_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");
    let preflight_response = app
        .oneshot(preflight_request)
        .await
        .expect("preflight response");
    assert_eq!(preflight_response.status(), StatusCode::OK);
    let preflight_body = to_bytes(preflight_response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let preflight_payload: serde_json::Value =
        serde_json::from_slice(&preflight_body).expect("decode preflight body");
    assert_eq!(preflight_payload["status"], "ready");
    assert_eq!(preflight_payload["reason_code"], "preflight_ok");
}

#[tokio::test]
async fn lists_lan_peer_when_same_server_policy_has_trusted_shared_membership() {
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&["usr-nora-k", "usr-jules-p"]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        "usr-nora-k",
        false,
        false,
        0,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-shared-lab",
        "Shared Lab",
        "usr-jules-p",
        false,
        false,
        0,
    )
    .await;

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-jules-p"]
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

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-jules-p"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
        )
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["identity_id"], "usr-jules-p");
}

#[tokio::test]
async fn hides_lan_peer_when_block_relationship_exists() {
    let (app, tokens, state) = app_with_sessions_and_state(&["usr-nora-k", "usr-jules-p"]);
    seed_accepted_friendship(&state, "usr-nora-k", "usr-jules-p");

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let block_request = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"target_identity_id":"usr-nora-k"}"#))
        .expect("build block request");
    let block_response = app
        .clone()
        .oneshot(block_request)
        .await
        .expect("block response");
    assert_eq!(block_response.status(), StatusCode::CREATED);

    let announce_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/lan-discovery/announce")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"endpoint_hints":["udp://192.168.1.12:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build lan peers request");

    let peers_response = app
        .oneshot(peers_request)
        .await
        .expect("lan peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);

    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read lan peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode lan peers body");

    let items = peers_payload["items"]
        .as_array()
        .expect("peers items array");
    assert!(items.is_empty());
}

#[tokio::test]
async fn internal_ingest_accepts_signed_lan_discovery_advertisement() {
    let (app, tokens, state) = app_with_sessions_and_state(&["usr-nora-k", "usr-jules-p"]);
    seed_accepted_friendship(&state, "usr-nora-k", "usr-jules-p");
    let signing_key = register_lan_identity_key(&state, "usr-jules-p");
    let now = Utc::now().timestamp();
    let advertisement = signed_lan_advertisement(
        "usr-jules-p",
        &signing_key,
        now,
        now + LAN_DISCOVERY_TTL_SECONDS,
    );

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("x-hexrelay-observed-source-ip", "192.168.1.12")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let ingest_response = app
        .clone()
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(ingest_response.status(), StatusCode::ACCEPTED);

    let peers_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/connectivity/lan-discovery/peers")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build LAN peers request");
    let peers_response = app
        .oneshot(peers_request)
        .await
        .expect("LAN peers response");
    assert_eq!(peers_response.status(), StatusCode::OK);
    let peers_body = to_bytes(peers_response.into_body(), usize::MAX)
        .await
        .expect("read LAN peers body");
    let peers_payload: serde_json::Value =
        serde_json::from_slice(&peers_body).expect("decode LAN peers body");
    let items = peers_payload["items"].as_array().expect("items array");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["identity_id"], "usr-jules-p");
    assert_eq!(items[0]["endpoint_hints"][0], "udp://192.168.1.12:4040");
}

#[tokio::test]
async fn internal_ingest_rejects_lan_discovery_source_mismatch() {
    let (app, _tokens, state) = app_with_sessions_and_state(&["usr-jules-p"]);
    let signing_key = register_lan_identity_key(&state, "usr-jules-p");
    let now = Utc::now().timestamp();
    let advertisement = signed_lan_advertisement(
        "usr-jules-p",
        &signing_key,
        now,
        now + LAN_DISCOVERY_TTL_SECONDS,
    );

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("x-hexrelay-observed-source-ip", "192.168.1.99")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let response = app
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read LAN ingest body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode error body");
    assert_eq!(payload["code"], "lan_discovery_invalid");
    assert!(state
        .dm_lan_presence
        .read()
        .expect("acquire LAN presence read lock")
        .is_empty());
}

#[tokio::test]
async fn internal_ingest_rejects_invalid_lan_discovery_token() {
    let (app, _tokens, state) = app_with_sessions_and_state(&["usr-jules-p"]);
    let signing_key = register_lan_identity_key(&state, "usr-jules-p");
    let now = Utc::now().timestamp();
    let advertisement = signed_lan_advertisement(
        "usr-jules-p",
        &signing_key,
        now,
        now + LAN_DISCOVERY_TTL_SECONDS,
    );

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header("x-hexrelay-internal-token", "wrong-token")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let response = app
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read LAN ingest body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode error body");
    assert_eq!(payload["code"], "internal_token_invalid");
}

#[tokio::test]
async fn internal_ingest_rejects_tampered_lan_discovery_signature() {
    let (app, _tokens, state) = app_with_sessions_and_state(&["usr-jules-p"]);
    let signing_key = register_lan_identity_key(&state, "usr-jules-p");
    let now = Utc::now().timestamp();
    let mut advertisement = signed_lan_advertisement(
        "usr-jules-p",
        &signing_key,
        now,
        now + LAN_DISCOVERY_TTL_SECONDS,
    );
    advertisement.signature = "00".repeat(64);

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("x-hexrelay-observed-source-ip", "192.168.1.12")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let response = app
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read LAN ingest body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode error body");
    assert_eq!(payload["code"], "lan_discovery_invalid");
    assert!(state
        .dm_lan_presence
        .read()
        .expect("acquire LAN presence read lock")
        .is_empty());
}

#[tokio::test]
async fn internal_ingest_rejects_unknown_lan_discovery_identity_key() {
    let (app, _tokens, state) = app_with_sessions_and_state(&["usr-jules-p"]);
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let now = Utc::now().timestamp();
    let advertisement = signed_lan_advertisement(
        "usr-jules-p",
        &signing_key,
        now,
        now + LAN_DISCOVERY_TTL_SECONDS,
    );

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("x-hexrelay-observed-source-ip", "192.168.1.12")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let response = app
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read LAN ingest body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode error body");
    assert_eq!(payload["code"], "lan_discovery_invalid");
    assert!(state
        .dm_lan_presence
        .read()
        .expect("acquire LAN presence read lock")
        .is_empty());
}

#[tokio::test]
async fn internal_ingest_rejects_expired_lan_discovery_advertisement() {
    let (app, _tokens, state) = app_with_sessions_and_state(&["usr-jules-p"]);
    let signing_key = register_lan_identity_key(&state, "usr-jules-p");
    let now = Utc::now().timestamp();
    let advertisement = signed_lan_advertisement("usr-jules-p", &signing_key, now - 300, now - 180);

    let ingest_request = Request::builder()
        .method("POST")
        .uri("/v1/internal/dm/connectivity/lan-discovery/ingest")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("x-hexrelay-observed-source-ip", "192.168.1.12")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&advertisement).expect("encode advertisement"),
        ))
        .expect("build LAN ingest request");

    let response = app
        .oneshot(ingest_request)
        .await
        .expect("LAN ingest response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read LAN ingest body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode error body");
    assert_eq!(payload["code"], "lan_discovery_invalid");
    assert!(state
        .dm_lan_presence
        .read()
        .expect("acquire LAN presence read lock")
        .is_empty());
}

fn register_lan_identity_key(state: &AppState, identity_id: &str) -> Ed25519KeyPair {
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    state
        .identity_keys
        .write()
        .expect("acquire identity key write lock")
        .insert(
            identity_id.to_string(),
            RegisteredIdentityKey {
                public_key: hex::encode(signing_key.public_key().as_ref()),
                algorithm: "ed25519".to_string(),
            },
        );

    signing_key
}

fn signed_lan_advertisement(
    identity_id: &str,
    signing_key: &Ed25519KeyPair,
    issued_at_epoch: i64,
    expires_at_epoch: i64,
) -> LanDiscoveryAdvertisement {
    let endpoint_hints = vec!["udp://192.168.1.12:4040".to_string()];
    let nonce = format!("nonce-{identity_id}-{issued_at_epoch}");
    let signing_payload = lan_discovery_signing_payload(
        identity_id,
        &endpoint_hints,
        issued_at_epoch,
        expires_at_epoch,
        &nonce,
    );
    let signature = signing_key.sign(&signing_payload);

    LanDiscoveryAdvertisement {
        version: 1,
        identity_id: identity_id.to_string(),
        endpoint_hints,
        scope: LAN_DISCOVERY_SCOPE.to_string(),
        issued_at_epoch,
        expires_at_epoch,
        nonce,
        signature: hex::encode(signature.as_ref()),
    }
}
