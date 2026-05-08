use super::*;

#[tokio::test]
async fn preflight_blocks_when_pairing_material_is_missing() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"pairing_envelope_present":false}"#))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "pairing_missing");
}

#[tokio::test]
async fn preflight_rejects_cookie_auth_without_csrf_token() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
        )
        .header("content-type", "application/json")
        .body(Body::from(r#"{"pairing_envelope_present":false}"#))
        .expect("build cookie preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["code"], "csrf_invalid");
}

#[tokio::test]
async fn preflight_blocks_when_policy_requires_friend_and_peer_not_friend() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p"}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "policy_blocked");
}

#[tokio::test]
async fn preflight_blocks_with_firewall_remediation_when_local_bind_is_denied() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"local_bind_allowed":false}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "port_unavailable");
    assert!(payload["remediation"][0]
        .as_str()
        .expect("first remediation step")
        .contains("firewall"));
}

#[tokio::test]
async fn preflight_blocks_with_peer_remediation_when_reachability_hint_fails() {
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

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":false}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "peer_unreachable");
    assert!(payload["remediation"][0]
        .as_str()
        .expect("first remediation step")
        .contains("online"));
}

#[tokio::test]
async fn preflight_ready_when_policy_allows_and_direct_hints_present() {
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
    assert_eq!(payload["reason_code"], "preflight_ok");
    assert_eq!(payload["transport_profile"], "direct_only");
}

#[tokio::test]
async fn preflight_prefers_lan_reason_when_peer_has_fresh_lan_presence() {
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
            r#"{"endpoint_hints":["tcp://192.168.1.20:4040"]}"#,
        ))
        .expect("build lan announce request");

    let announce_response = app
        .clone()
        .oneshot(announce_request)
        .await
        .expect("lan announce response");
    assert_eq!(announce_response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "preflight_ok_lan");
    assert_eq!(payload["transport_profile"], "direct_only");
}

#[tokio::test]
async fn preflight_blocks_when_policy_is_same_server_even_if_client_claims_context() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"same_server"}"#))
        .expect("build dm policy update request");

    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-jules-p","local_bind_allowed":true,"peer_reachable_hint":true,"same_server_context":true}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "policy_blocked");
}

#[tokio::test]
async fn preflight_allows_when_policy_is_same_server_and_membership_is_shared() {
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

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"same_server"}"#))
        .expect("build dm policy update request");

    let policy_response = app
        .clone()
        .oneshot(policy_update)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
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
            r#"{"pairing_envelope_present":true,"peer_identity_id":"usr-nora-k","local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");

    let response = app.oneshot(request).await.expect("preflight response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode preflight response");
    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "preflight_ok");
}

#[tokio::test]
async fn rejects_invalid_preflight_request() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"pairing_envelope_present":true,"peer_identity_id":"   "}"#,
        ))
        .expect("build invalid preflight request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid preflight response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid preflight body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid preflight body");
    assert_eq!(payload["code"], "preflight_invalid");
    assert_eq!(
        payload["message"],
        "peer_identity_id must be 3-64 chars using letters, numbers, _ or -"
    );
}
