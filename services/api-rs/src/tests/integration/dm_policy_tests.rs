use super::*;

#[tokio::test]
async fn returns_default_dm_policy_for_new_identity() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build dm policy get request");

    let response = app.oneshot(request).await.expect("dm policy get response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read dm policy get body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode dm policy get response");

    assert_eq!(payload["inbound_policy"], "friends_only");
    assert_eq!(payload["offline_delivery_mode"], "best_effort_online");
}

#[tokio::test]
async fn updates_dm_policy_and_persists_for_identity() {
    let app = build_app(AppState::default());
    let (session_cookie, app) = authenticate_identity(app, "usr-dm-policy").await;

    let update_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");

    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("dm policy update response");
    assert_eq!(update_response.status(), StatusCode::OK);

    let read_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/privacy-policy")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build dm policy read request");

    let read_response = app
        .oneshot(read_request)
        .await
        .expect("dm policy read response");
    assert_eq!(read_response.status(), StatusCode::OK);

    let read_body = to_bytes(read_response.into_body(), usize::MAX)
        .await
        .expect("read dm policy response body");
    let read_payload: serde_json::Value =
        serde_json::from_slice(&read_body).expect("decode dm policy response");
    assert_eq!(read_payload["inbound_policy"], "anyone");
}

#[tokio::test]
async fn rejects_invalid_dm_policy_update() {
    let app = build_app(AppState::default());
    let (session_cookie, app) = authenticate_identity(app, "usr-dm-invalid").await;

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"invalid"}"#))
        .expect("build invalid dm policy update request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid dm policy update response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
