use super::*;

#[tokio::test]
async fn registers_identity_key_with_hex_key() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
        ))
        .expect("build request");

    let response = app.oneshot(request).await.expect("get response");
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn rejects_duplicate_identity_registration() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-dup","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
        ))
        .expect("build first request");
    let first_response = app.clone().oneshot(request).await.expect("first response");
    assert_eq!(first_response.status(), StatusCode::CREATED);

    let duplicate_request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-dup","public_key":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","algorithm":"ed25519"}"#,
        ))
        .expect("build duplicate request");
    let duplicate_response = app
        .oneshot(duplicate_request)
        .await
        .expect("duplicate response");
    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn rejects_invalid_algorithm() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"rsa"}"#,
        ))
        .expect("build request");

    let response = app.oneshot(request).await.expect("get response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_invalid_public_key_format() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-1","public_key":"not-a-real-key","algorithm":"ed25519"}"#,
        ))
        .expect("build request");

    let response = app.oneshot(request).await.expect("get response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_identity_id_with_surrounding_whitespace() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":" user-1 ","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
        ))
        .expect("build request");

    let response = app.oneshot(request).await.expect("get response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn issues_auth_challenge_for_registered_identity() {
    let app = build_app(AppState::default());
    let (register_status, app) = register_identity(
        app,
        "user-1",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-1"}"#))
        .expect("build challenge request");

    let challenge_response = app
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn issues_indistinguishable_challenge_for_unknown_identity() {
    let app = build_app(AppState::default());
    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"missing-user"}"#))
        .expect("build challenge request");

    let challenge_response = app
        .clone()
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_body = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_body).expect("decode challenge response");

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"missing-user","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id,
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        )))
        .expect("build verify request");

    let verify_response = app.oneshot(verify_request).await.expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rate_limits_auth_challenge_requests() {
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
            auth_challenge_per_window: 1,
            auth_verify_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        false,
    );

    state
        .identity_keys
        .write()
        .expect("acquire identity key write lock")
        .insert(
            "user-rate-limit".to_string(),
            RegisteredIdentityKey {
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                algorithm: "ed25519".to_string(),
            },
        );

    let app = build_app(state);

    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-rate-limit"}"#))
        .expect("build first challenge request");
    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first challenge response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-rate-limit"}"#))
        .expect("build second challenge request");
    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second challenge response");
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limits_auth_challenge_by_x_forwarded_for_when_proxy_headers_trusted() {
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
            auth_challenge_per_window: 1,
            auth_verify_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        true,
    );

    state
        .identity_keys
        .write()
        .expect("acquire identity key write lock")
        .insert(
            "user-xff".to_string(),
            RegisteredIdentityKey {
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                algorithm: "ed25519".to_string(),
            },
        );

    let app = build_app(state);

    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "198.51.100.1")
        .body(Body::from(r#"{"identity_id":"user-xff"}"#))
        .expect("build first challenge request");
    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first challenge response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "198.51.100.1")
        .body(Body::from(r#"{"identity_id":"user-xff"}"#))
        .expect("build second challenge request");
    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second challenge response");
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limits_auth_challenge_by_x_real_ip_when_proxy_headers_trusted() {
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
            auth_challenge_per_window: 1,
            auth_verify_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        true,
    );

    state
        .identity_keys
        .write()
        .expect("acquire identity key write lock")
        .insert(
            "user-xri".to_string(),
            RegisteredIdentityKey {
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                algorithm: "ed25519".to_string(),
            },
        );

    let app = build_app(state);

    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-real-ip", "198.51.100.2")
        .body(Body::from(r#"{"identity_id":"user-xri"}"#))
        .expect("build first challenge request");
    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first challenge response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-real-ip", "198.51.100.2")
        .body(Body::from(r#"{"identity_id":"user-xri"}"#))
        .expect("build second challenge request");
    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second challenge response");
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn rate_limits_auth_challenge_source_even_when_identity_changes() {
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
            auth_challenge_per_window: 1,
            auth_verify_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        true,
    );

    let app = build_app(state);

    let first_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "198.51.100.20")
        .body(Body::from(r#"{"identity_id":"spray-user-a"}"#))
        .expect("build first challenge request");
    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first challenge response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let second_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "198.51.100.20")
        .body(Body::from(r#"{"identity_id":"spray-user-b"}"#))
        .expect("build second challenge request");
    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second challenge response");
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn verifies_auth_challenge_and_revokes_session() {
    let app = build_app(AppState::default());
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) = register_identity(app, "user-verify", &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-verify"}"#))
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
            r#"{{"identity_id":"user-verify","challenge_id":"{}","signature":"{}"}}"#,
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
    assert!(!verify.session_id.is_empty());

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

    let revoke_response = app.oneshot(revoke_request).await.expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rejects_session_revoke_with_missing_csrf_header() {
    let (app, tokens) = app_with_sessions(&["usr-csrf-missing"]);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/sessions/revoke")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-csrf-missing"]
            ),
        )
        .body(Body::from(r#"{"session_id":"sess-usr-csrf-missing"}"#))
        .expect("build revoke request");

    let revoke_response = app.oneshot(revoke_request).await.expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_session_revoke_with_mismatched_csrf_header() {
    let (app, tokens) = app_with_sessions(&["usr-csrf-mismatch"]);

    let revoke_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/sessions/revoke")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-csrf-mismatch"]
            ),
        )
        .header("x-csrf-token", "wrong-csrf")
        .body(Body::from(r#"{"session_id":"sess-usr-csrf-mismatch"}"#))
        .expect("build revoke request");

    let revoke_response = app.oneshot(revoke_request).await.expect("revoke response");
    assert_eq!(revoke_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_invalid_signature_on_verify() {
    let app = build_app(AppState::default());
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) =
        register_identity(app, "user-invalid-signature", &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-invalid-signature"}"#))
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

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"user-invalid-signature","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id,
            hex::encode([0_u8; 64])
        )))
        .expect("build verify request");

    let verify_response = app.oneshot(verify_request).await.expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn validates_session_with_header() {
    let (app, tokens) = app_with_sessions(&["usr-session"]);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("authorization", format!("Bearer {}", tokens["usr-session"]))
        .body(Body::empty())
        .expect("build session validate request");

    let response = app
        .oneshot(request)
        .await
        .expect("session validate response");
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_replayed_challenge_verification() {
    let app = build_app(AppState::default());
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) = register_identity(app, "user-replay", &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"identity_id":"user-replay"}"#))
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

    let verify_body = format!(
        r#"{{"identity_id":"user-replay","challenge_id":"{}","signature":"{}"}}"#,
        challenge.challenge_id, signature_hex
    );

    let request_a = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(verify_body.clone()))
        .expect("build verify request a");

    let request_b = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(verify_body))
        .expect("build verify request b");

    let (response_a, response_b) = tokio::join!(
        app.clone().oneshot(request_a),
        app.clone().oneshot(request_b)
    );

    let status_a = response_a.expect("verify response a").status();
    let status_b = response_b.expect("verify response b").status();

    let success_count =
        usize::from(status_a == StatusCode::OK) + usize::from(status_b == StatusCode::OK);
    let unauthorized_count = usize::from(status_a == StatusCode::UNAUTHORIZED)
        + usize::from(status_b == StatusCode::UNAUTHORIZED);

    assert_eq!(success_count, 1);
    assert_eq!(unauthorized_count, 1);
}

#[tokio::test]
async fn rejects_expired_challenge_verification() {
    let state = AppState::default();

    state
        .identity_keys
        .write()
        .expect("acquire identity key write lock")
        .insert(
            "user-expired".to_string(),
            RegisteredIdentityKey {
                public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                    .to_string(),
                algorithm: "ed25519".to_string(),
            },
        );

    state
        .auth_challenges
        .write()
        .expect("acquire challenge write lock")
        .insert(
            "challenge-expired".to_string(),
            AuthChallengeRecord {
                identity_id: "user-expired".to_string(),
                nonce: "deadbeef".to_string(),
                expires_at: Utc::now() - Duration::seconds(1),
            },
        );

    let app = build_app(state);

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"identity_id":"user-expired","challenge_id":"challenge-expired","signature":"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}"#,
        ))
        .expect("build verify request");

    let response = app.oneshot(verify_request).await.expect("verify response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
