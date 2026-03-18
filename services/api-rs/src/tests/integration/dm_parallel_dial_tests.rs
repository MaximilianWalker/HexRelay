use super::*;

async fn set_dm_policy_anyone(app: axum::Router, token: &str) -> axum::Router {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
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
async fn parallel_dial_selects_fastest_reachable_endpoint_and_cancels_others() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"wan-a","endpoint_hint":"udp://203.0.113.20:4040","estimated_rtt_ms":70,"priority":1},{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":9,"priority":2},{"endpoint_id":"wan-b","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":40,"priority":1}]}"#,
        ))
        .expect("build endpoint register request");

    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","max_parallel_attempts":3}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "parallel_dial_connected");
    assert_eq!(payload["winner_endpoint_id"], "lan-a");

    let canceled = payload["canceled_endpoint_ids"]
        .as_array()
        .expect("canceled endpoints array");
    assert_eq!(canceled.len(), 2);
}

#[tokio::test]
async fn parallel_dial_ignores_revoked_endpoint_cards() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10},{"endpoint_id":"wan-a","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":30}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards/revoke")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"endpoint_ids":["lan-a"]}"#))
        .expect("build endpoint revoke request");
    let revoke_response = app
        .clone()
        .oneshot(revoke_request)
        .await
        .expect("endpoint revoke response");
    assert_eq!(revoke_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","max_parallel_attempts":2}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["winner_endpoint_id"], "wan-a");
}

#[tokio::test]
async fn revoke_endpoint_cards_returns_newly_revoked_ids_in_request_order() {
    let sender_identity = unique_identity("db-bulk-revoke-sender");
    let recipient_identity = unique_identity("db-bulk-revoke-recipient");
    let endpoint_wan_a = format!("wan-a-{}", uuid::Uuid::new_v4().simple());
    let endpoint_lan_a = format!("lan-a-{}", uuid::Uuid::new_v4().simple());
    let endpoint_wan_b = format!("wan-b-{}", uuid::Uuid::new_v4().simple());

    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender_identity.as_str(), recipient_identity.as_str()])
            .await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient_identity.as_str()]).await;

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-jules-p"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"cards":[{{"endpoint_id":"{}","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":30}},{{"endpoint_id":"{}","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10}},{{"endpoint_id":"{}","endpoint_hint":"udp://203.0.113.20:4040","estimated_rtt_ms":40}}]}}"#,
            endpoint_wan_a, endpoint_lan_a, endpoint_wan_b,
        )))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards/revoke")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[recipient_identity.as_str()]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"endpoint_ids":["{}","{}","missing-id"]}}"#,
            endpoint_wan_b, endpoint_lan_a,
        )))
        .expect("build endpoint revoke request");
    let revoke_response = app
        .clone()
        .oneshot(revoke_request)
        .await
        .expect("endpoint revoke response");
    assert_eq!(revoke_response.status(), StatusCode::OK);

    let revoke_body = to_bytes(revoke_response.into_body(), usize::MAX)
        .await
        .expect("read endpoint revoke body");
    let revoke_payload: serde_json::Value =
        serde_json::from_slice(&revoke_body).expect("decode endpoint revoke body");

    assert_eq!(
        revoke_payload["revoked_endpoint_ids"],
        serde_json::json!([endpoint_wan_b, endpoint_lan_a])
    );

    let idempotent_revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards/revoke")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[recipient_identity.as_str()]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"endpoint_ids":["{}","{}"]}}"#,
            endpoint_wan_b, endpoint_lan_a,
        )))
        .expect("build idempotent endpoint revoke request");
    let idempotent_revoke_response = app
        .oneshot(idempotent_revoke_request)
        .await
        .expect("idempotent endpoint revoke response");
    assert_eq!(idempotent_revoke_response.status(), StatusCode::OK);

    let idempotent_revoke_body = to_bytes(idempotent_revoke_response.into_body(), usize::MAX)
        .await
        .expect("read idempotent endpoint revoke body");
    let idempotent_revoke_payload: serde_json::Value =
        serde_json::from_slice(&idempotent_revoke_body)
            .expect("decode idempotent endpoint revoke body");

    assert_eq!(
        idempotent_revoke_payload["revoked_endpoint_ids"],
        serde_json::json!([])
    );
}

#[tokio::test]
async fn parallel_dial_returns_blocked_when_all_attempts_fail() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"wan-a","endpoint_hint":"udp://203.0.113.20:4040","estimated_rtt_ms":70},{"endpoint_id":"wan-b","endpoint_hint":"tcp://198.51.100.20:4041","estimated_rtt_ms":40}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"peer_identity_id":"usr-jules-p","unreachable_endpoint_ids":["wan-a","wan-b"]}"#,
        ))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "parallel_dial_exhausted");
}

#[tokio::test]
async fn parallel_dial_blocks_when_peer_policy_disallows_sender() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"peer_identity_id":"usr-jules-p"}"#))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "parallel_dial_policy_blocked");
}

#[tokio::test]
async fn parallel_dial_allows_when_peer_policy_is_same_server_and_membership_is_shared() {
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

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("cookie", format!("hexrelay_session={}; hexrelay_csrf=test-csrf", tokens["usr-jules-p"]))
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"peer_identity_id":"usr-jules-p"}"#))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::OK);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "parallel_dial_connected");
}

#[tokio::test]
async fn parallel_dial_rejects_cookie_auth_without_matching_csrf_token() {
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

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
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
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":10}]}"#,
        ))
        .expect("build endpoint register request");
    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "wrong-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"peer_identity_id":"usr-jules-p"}"#))
        .expect("build parallel dial request");

    let dial_response = app
        .oneshot(dial_request)
        .await
        .expect("parallel dial response");
    assert_eq!(dial_response.status(), StatusCode::UNAUTHORIZED);

    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial body");
    let payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial body");

    assert_eq!(payload["code"], "csrf_invalid");
}
