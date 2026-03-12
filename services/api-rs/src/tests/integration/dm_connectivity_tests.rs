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
async fn preflight_ready_when_policy_allows_and_direct_hints_present() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let policy_update = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
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
