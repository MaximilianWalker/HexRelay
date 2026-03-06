use super::*;

#[tokio::test]
async fn validates_and_revokes_db_backed_session() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) = register_identity(app, "db-user-verify", &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"db-user-verify"}"#))
        .expect("build challenge request");

    let challenge_response = app
        .clone()
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_bytes = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_bytes).expect("decode challenge response");

    let signature = signing_key.sign(challenge.nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"db-user-verify","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id, signature_hex
        )))
        .expect("build verify request");

    let verify_response = app
        .clone()
        .oneshot(verify_request)
        .await
        .expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::OK);

    let session_cookie =
        extract_cookie_from_set_cookie_headers(&verify_response, "hexrelay_session")
            .expect("verify response includes session cookie");

    let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
        .await
        .expect("read verify response body");
    let verify: AuthVerifyResponse =
        serde_json::from_slice(&verify_bytes).expect("decode verify response");

    let validate_request = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build validate request");

    let validate_response = app
        .clone()
        .oneshot(validate_request)
        .await
        .expect("validate response");
    assert_eq!(validate_response.status(), StatusCode::OK);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/sessions/revoke")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"session_id":"{}"}}"#,
            verify.session_id
        )))
        .expect("build revoke request");

    let revoke_response = app
        .clone()
        .oneshot(revoke_request)
        .await
        .expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);

    let validate_after_revoke = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build post-revoke validate request");

    let validate_after_revoke_response = app
        .oneshot(validate_after_revoke)
        .await
        .expect("post-revoke validate response");
    assert_eq!(
        validate_after_revoke_response.status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn redeems_db_invite_after_app_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let (session_cookie, app) = authenticate_identity(app, "db-user-invite").await;

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/invites")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"mode":"one_time"}"#))
        .expect("build create invite request");

    let create_response = app
        .oneshot(create_request)
        .await
        .expect("create invite response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create invite body");
    let created: InviteCreateResponse =
        serde_json::from_slice(&create_bytes).expect("decode invite create response");

    let Some(app_after_restart) = app_with_database().await else {
        return;
    };

    let redeem_request = Request::builder()
        .method("POST")
        .uri("/v1/invites/redeem")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
            created.token, TEST_NODE_FINGERPRINT
        )))
        .expect("build redeem invite request");

    let redeem_response = app_after_restart
        .oneshot(redeem_request)
        .await
        .expect("redeem invite response");
    assert_eq!(redeem_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn verifies_db_challenge_after_app_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) = register_identity(app, "db-user-restart", &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"db-user-restart"}"#))
        .expect("build challenge request");

    let challenge_response = app
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_bytes = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_bytes).expect("decode challenge response");

    let signature = signing_key.sign(challenge.nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let Some(app_after_restart) = app_with_database().await else {
        return;
    };

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"db-user-restart","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id, signature_hex
        )))
        .expect("build verify request");

    let verify_response = app_after_restart
        .oneshot(verify_request)
        .await
        .expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::OK);
}
