use super::*;

#[tokio::test]
async fn creates_and_redeems_multi_use_invite() {
    let (app, tokens) = app_with_sessions(&["usr-invite"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
        .body(Body::from(r#"{"mode":"multi_use","max_uses":2}"#))
        .expect("build create invite request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create response body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_bytes).expect("decode invite create response");

    let redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build redeem invite request");

    let redeem_response = app
        .clone()
        .oneshot(redeem_request)
        .await
        .expect("redeem invite response");
    assert_eq!(redeem_response.status(), StatusCode::OK);

    let second_redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build second redeem invite request");

    let second_redeem_response = app
        .oneshot(second_redeem_request)
        .await
        .expect("second redeem invite response");
    assert_eq!(second_redeem_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_exhausted_one_time_invite() {
    let (app, tokens) = app_with_sessions(&["usr-invite"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
        .body(Body::from(r#"{"mode":"one_time"}"#))
        .expect("build create invite request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create response body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_bytes).expect("decode invite create response");

    let first_redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build first redeem request");
    let first_redeem_response = app
        .clone()
        .oneshot(first_redeem_request)
        .await
        .expect("first redeem response");
    assert_eq!(first_redeem_response.status(), StatusCode::OK);

    let second_redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build second redeem request");
    let second_redeem_response = app
        .oneshot(second_redeem_request)
        .await
        .expect("second redeem response");
    assert_eq!(second_redeem_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_expired_invite() {
    let (app, tokens) = app_with_sessions(&["usr-invite"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
        .body(Body::from(
            r#"{"mode":"multi_use","expires_at":"2000-01-01T00:00:00Z"}"#,
        ))
        .expect("build create invite request");

    let create_response = app
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_fingerprint_mismatch_on_redeem() {
    let (app, tokens) = app_with_sessions(&["usr-invite"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
        .body(Body::from(r#"{"mode":"multi_use","max_uses":2}"#))
        .expect("build create invite request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create response body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_bytes).expect("decode invite create response");

    let redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"mismatch-node"}}"#,
            created.token
        )))
        .expect("build redeem request");

    let redeem_response = app.oneshot(redeem_request).await.expect("redeem response");
    assert_eq!(redeem_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rate_limits_invite_redeem_requests() {
    let state = AppState::new(
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        BTreeMap::from([(
            "v1".to_string(),
            "hexrelay-dev-signing-key-change-me".to_string(),
        )]),
        None,
        false,
        "Lax".to_string(),
        ApiRateLimitConfig {
            auth_challenge_per_window: 30,
            auth_verify_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 1,
            window_seconds: 60,
        },
    );

    let token = "invite-rate-limit-token";
    let token_hash = hex::encode(digest(&SHA256, token.as_bytes()).as_ref());
    state
        .invites
        .write()
        .expect("acquire invite write lock")
        .insert(
            token_hash,
            crate::models::InviteRecord {
                mode: "multi_use".to_string(),
                node_fingerprint: TEST_NODE_FINGERPRINT.to_string(),
                expires_at: None,
                max_uses: None,
                uses: 0,
            },
        );

    let app = build_app(state);

    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            token, TEST_NODE_FINGERPRINT
        )))
        .expect("build first redeem request");
    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first redeem response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            token, TEST_NODE_FINGERPRINT
        )))
        .expect("build second redeem request");
    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second redeem response");
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}
