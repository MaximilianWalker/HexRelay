use super::*;

#[tokio::test]
async fn dev_testing_profiles_are_disabled_by_default() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("GET")
        .uri("/v1/dev/testing/profiles")
        .body(Body::empty())
        .expect("build dev testing profiles request");

    let response = app.oneshot(request).await.expect("dev testing response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn creates_real_db_session_for_seeded_testing_profile() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    ensure_fixture_identity_key(
        &pool,
        "usr-test-alice",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    .await;

    let state = AppState::default()
        .with_dev_testing(true)
        .with_db_pool(pool.clone());
    let app = build_app(state);
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dev/testing/sessions")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"profile_id":"alice.primary"}"#))
        .expect("build dev testing session request");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("dev testing session response");
    assert_eq!(response.status(), StatusCode::OK);
    let session_cookie = extract_cookie_from_set_cookie_headers(&response, "hexrelay_session")
        .expect("testing session response includes session cookie");

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read dev testing session body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode dev testing session body");
    assert_eq!(payload["profile_id"], "alice.primary");
    assert_eq!(payload["identity_id"], "usr-test-alice");
    assert_eq!(payload["session_id"], "sess-test-alice-primary");

    let validate_request = Request::builder()
        .method("GET")
        .uri("/v1/auth/sessions/validate")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build validate request");
    let validate_response = app
        .oneshot(validate_request)
        .await
        .expect("validate testing session response");
    assert_eq!(validate_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_testing_profile_with_mismatched_fixture_key() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    ensure_db_identity_key(&pool, "usr-test-bob").await;

    let state = AppState::default()
        .with_dev_testing(true)
        .with_db_pool(pool.clone());
    let app = build_app(state);
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dev/testing/sessions")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"profile_id":"bob.primary"}"#))
        .expect("build dev testing session request");

    let response = app
        .oneshot(request)
        .await
        .expect("dev testing mismatch response");
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

async fn ensure_fixture_identity_key(pool: &sqlx::PgPool, identity_id: &str, public_key: &str) {
    sqlx::query(
        "
        INSERT INTO identity_keys (identity_id, public_key, algorithm)
        VALUES ($1, $2, 'ed25519')
        ON CONFLICT (identity_id) DO UPDATE
        SET public_key = EXCLUDED.public_key,
            algorithm = EXCLUDED.algorithm
        ",
    )
    .bind(identity_id)
    .bind(public_key)
    .execute(pool)
    .await
    .expect("ensure fixture identity key");
}
