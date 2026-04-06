use super::*;
use crate::models as api_models;

#[derive(Deserialize)]
struct DiscoveryUserListResponse {
    items: Vec<DiscoveryUserSummary>,
}

#[derive(Deserialize)]
struct DiscoveryUserSummary {
    identity_id: String,
    display_name: String,
    relationship_state: String,
    shared_server_count: u32,
    can_send_friend_request: bool,
    has_pending_inbound_request: bool,
    has_pending_outbound_request: bool,
}

#[tokio::test]
async fn discovery_lists_global_known_users_and_relationship_states() {
    let state = AppState::default();
    let expires_at = Utc::now() + Duration::hours(1);
    state.sessions.write().expect("session write lock").insert(
        "sess-a".to_string(),
        SessionRecord {
            identity_id: "usr-a".to_string(),
            expires_at,
        },
    );

    state
        .identity_keys
        .write()
        .expect("identity key write lock")
        .extend([
            (
                "usr-a".to_string(),
                RegisteredIdentityKey {
                    public_key: "aa".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-b".to_string(),
                RegisteredIdentityKey {
                    public_key: "bb".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-c".to_string(),
                RegisteredIdentityKey {
                    public_key: "cc".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
        ]);

    {
        let mut requests = state
            .friend_requests
            .write()
            .expect("friend request write lock");
        requests.insert(
            "req-accepted".to_string(),
            api_models::FriendRequestRecord {
                request_id: "req-accepted".to_string(),
                requester_identity_id: "usr-a".to_string(),
                target_identity_id: "usr-b".to_string(),
                status: "accepted".to_string(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
        requests.insert(
            "req-pending".to_string(),
            api_models::FriendRequestRecord {
                request_id: "req-pending".to_string(),
                requester_identity_id: "usr-c".to_string(),
                target_identity_id: "usr-a".to_string(),
                status: "pending".to_string(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
    }

    let token = issue_session_token(
        "sess-a",
        "usr-a",
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert_eq!(payload.items.len(), 2);
    assert!(payload.items.iter().any(|item| {
        item.identity_id == "usr-b"
            && item.display_name == "usr-b"
            && item.relationship_state == "accepted"
            && !item.can_send_friend_request
    }));
    assert!(payload.items.iter().any(|item| {
        item.identity_id == "usr-c"
            && item.relationship_state == "pending"
            && item.has_pending_inbound_request
            && !item.has_pending_outbound_request
    }));
}

#[tokio::test]
async fn discovery_excludes_blocked_users_bidirectionally() {
    let state = AppState::default();
    let expires_at = Utc::now() + Duration::hours(1);
    state.sessions.write().expect("session write lock").insert(
        "sess-a".to_string(),
        SessionRecord {
            identity_id: "usr-a".to_string(),
            expires_at,
        },
    );

    state
        .identity_keys
        .write()
        .expect("identity key write lock")
        .extend([
            (
                "usr-a".to_string(),
                RegisteredIdentityKey {
                    public_key: "aa".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-b".to_string(),
                RegisteredIdentityKey {
                    public_key: "bb".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-c".to_string(),
                RegisteredIdentityKey {
                    public_key: "cc".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
        ]);

    {
        let mut blocked = state
            .blocked_users
            .write()
            .expect("blocked users write lock");
        blocked.insert(
            "usr-a".to_string(),
            HashMap::from([("usr-b".to_string(), Utc::now().timestamp())]),
        );
        blocked.insert(
            "usr-c".to_string(),
            HashMap::from([("usr-a".to_string(), Utc::now().timestamp())]),
        );
    }

    let token = issue_session_token(
        "sess-a",
        "usr-a",
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert!(payload.items.is_empty());
}

#[tokio::test]
async fn discovery_rate_limits_queries() {
    let state = AppState::new(
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        None,
        "http://127.0.0.1:8081".to_string(),
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
            discovery_query_per_window: 1,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        false,
    );

    {
        let mut sessions = state.sessions.write().expect("session write lock");
        sessions.insert(
            "sess-rate".to_string(),
            SessionRecord {
                identity_id: "usr-rate".to_string(),
                expires_at: Utc::now() + Duration::hours(1),
            },
        );
    }

    let token = issue_session_token(
        "sess-rate",
        "usr-rate",
        (Utc::now() + Duration::hours(1)).timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let first = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build first request");
    let second = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build second request");

    let first_response = app.clone().oneshot(first).await.expect("first response");
    let second_response = app.oneshot(second).await.expect("second response");
    assert_eq!(first_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn discovery_rejects_invalid_scope() {
    let state = AppState::default();
    let expires_at = Utc::now() + Duration::hours(1);
    state.sessions.write().expect("session write lock").insert(
        "sess-invalid-scope".to_string(),
        SessionRecord {
            identity_id: "usr-scope".to_string(),
            expires_at,
        },
    );

    let token = issue_session_token(
        "sess-invalid-scope",
        "usr-scope",
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=planetary")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build invalid discovery request");

    let response = app.oneshot(request).await.expect("invalid discovery response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid discovery body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid discovery payload");
    assert_eq!(payload["code"], "scope_invalid");
}

#[tokio::test]
async fn discovery_ignores_blank_query_and_clamps_large_limit() {
    let actor = unique_identity("usr-discovery-query-actor");
    let allowed_a = unique_identity("usr-discovery-query-alpha");
    let allowed_b = unique_identity("usr-discovery-query-beta");

    let Some((_app, _tokens, pool)) =
        app_with_database_and_sessions(&[&actor, &allowed_a, &allowed_b]).await
    else {
        return;
    };

    let state = AppState::new(
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        Vec::new(),
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        None,
        "http://127.0.0.1:8081".to_string(),
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
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        false,
    )
    .with_db_pool(pool.clone());

    ensure_db_identity_key(&pool, &actor).await;
    ensure_db_identity_key(&pool, &allowed_a).await;
    ensure_db_identity_key(&pool, &allowed_b).await;

    let token = issue_db_session_cookie(&pool, &state, &actor).await;
    let app = build_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global&query=%20%20%20&limit=999")
        .header("cookie", format!("hexrelay_session={token}"))
        .body(Body::empty())
        .expect("build blank-query discovery request");

    let response = app.oneshot(request).await.expect("blank-query discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read blank-query discovery body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode blank-query discovery payload");

    assert_eq!(payload.items.len(), 2);
}

#[tokio::test]
async fn discovery_shared_server_scope_uses_persisted_memberships() {
    let actor = unique_identity("usr-discovery-shared-actor");
    let shared_peer = unique_identity("usr-discovery-shared-peer");
    let other_user = unique_identity("usr-discovery-shared-other");

    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&actor, &shared_peer, &other_user]).await
    else {
        return;
    };

    sqlx::query(
        "INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status) VALUES ($1, $2, $3, 'accepted')",
    )
    .bind(unique_identity("req-discovery-shared"))
    .bind(&actor)
    .bind(&other_user)
    .execute(&pool)
    .await
    .expect("insert persisted relationship");

    seed_server_membership(&pool, "srv-shared", "Shared", &actor, false, false, 0).await;
    seed_server_membership(&pool, "srv-shared", "Shared", &shared_peer, false, false, 0).await;
    seed_server_membership(&pool, "srv-other", "Other", &other_user, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=shared_server")
        .header("cookie", format!("hexrelay_session={}", tokens[&actor]))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].identity_id, shared_peer);
    assert_eq!(payload.items[0].shared_server_count, 1);
}

#[tokio::test]
async fn discovery_excludes_configured_denylist() {
    let state = AppState::new(
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        vec!["usr-denied".to_string()],
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        None,
        "http://127.0.0.1:8081".to_string(),
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
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        false,
    );

    let expires_at = Utc::now() + Duration::hours(1);
    state.sessions.write().expect("session write lock").insert(
        "sess-actor".to_string(),
        SessionRecord {
            identity_id: "usr-actor".to_string(),
            expires_at,
        },
    );

    state
        .identity_keys
        .write()
        .expect("identity key write lock")
        .extend([
            (
                "usr-actor".to_string(),
                RegisteredIdentityKey {
                    public_key: "aa".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-denied".to_string(),
                RegisteredIdentityKey {
                    public_key: "bb".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-allowed".to_string(),
                RegisteredIdentityKey {
                    public_key: "cc".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
        ]);

    let token = issue_session_token(
        "sess-actor",
        "usr-actor",
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].identity_id, "usr-allowed");
}

#[tokio::test]
async fn discovery_global_db_includes_identity_keys_and_honors_limit_after_exclusions() {
    let actor = unique_identity("usr-discovery-actor");
    let blocked = unique_identity("usr-discovery-blocked");
    let denied = unique_identity("usr-discovery-denied");
    let allowed = unique_identity("usr-discovery-allowed");

    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let state = AppState::new(
        TEST_NODE_FINGERPRINT.to_string(),
        vec![TEST_ALLOWED_ORIGIN.to_string()],
        "v1".to_string(),
        vec![denied.clone()],
        "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
        "hexrelay-dev-presence-watcher-token-change-me".to_string(),
        None,
        "http://127.0.0.1:8081".to_string(),
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
            discovery_query_per_window: 30,
            invite_create_per_window: 20,
            invite_redeem_per_window: 40,
            window_seconds: 60,
        },
        false,
    )
    .with_db_pool(pool.clone());

    ensure_db_identity_key(&pool, &actor).await;
    ensure_db_identity_key(&pool, &blocked).await;
    ensure_db_identity_key(&pool, &denied).await;
    ensure_db_identity_key(&pool, &allowed).await;

    state
        .blocked_users
        .write()
        .expect("blocked users write lock")
        .insert(
            actor.clone(),
            HashMap::from([(blocked.clone(), Utc::now().timestamp())]),
        );

    let token = issue_db_session_cookie(&pool, &state, &actor).await;
    let app = build_app(state);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global&query=usr-discovery-&limit=1")
        .header("cookie", format!("hexrelay_session={token}"))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].identity_id, allowed);
}

#[tokio::test]
async fn discovery_prefers_accepted_relationship_over_newer_terminal_state() {
    let state = AppState::default();
    let expires_at = Utc::now() + Duration::hours(1);
    state.sessions.write().expect("session write lock").insert(
        "sess-main".to_string(),
        SessionRecord {
            identity_id: "usr-main".to_string(),
            expires_at,
        },
    );

    state
        .identity_keys
        .write()
        .expect("identity key write lock")
        .extend([
            (
                "usr-main".to_string(),
                RegisteredIdentityKey {
                    public_key: "aa".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
            (
                "usr-peer".to_string(),
                RegisteredIdentityKey {
                    public_key: "bb".repeat(32),
                    algorithm: "ed25519".to_string(),
                },
            ),
        ]);

    {
        let mut requests = state
            .friend_requests
            .write()
            .expect("friend request write lock");
        requests.insert(
            "req-accepted".to_string(),
            api_models::FriendRequestRecord {
                request_id: "req-accepted".to_string(),
                requester_identity_id: "usr-main".to_string(),
                target_identity_id: "usr-peer".to_string(),
                status: "accepted".to_string(),
                created_at: (Utc::now() - Duration::minutes(5)).to_rfc3339(),
            },
        );
        requests.insert(
            "req-cancelled".to_string(),
            api_models::FriendRequestRecord {
                request_id: "req-cancelled".to_string(),
                requester_identity_id: "usr-main".to_string(),
                target_identity_id: "usr-peer".to_string(),
                status: "cancelled".to_string(),
                created_at: Utc::now().to_rfc3339(),
            },
        );
    }

    let token = issue_session_token(
        "sess-main",
        "usr-main",
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key"),
    );

    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/discovery/users?scope=global")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .expect("build discovery request");

    let response = app.oneshot(request).await.expect("discovery response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read discovery response body");
    let payload: DiscoveryUserListResponse =
        serde_json::from_slice(&body).expect("decode discovery payload");

    assert_eq!(payload.items.len(), 1);
    assert_eq!(payload.items[0].identity_id, "usr-peer");
    assert_eq!(payload.items[0].relationship_state, "accepted");
    assert!(!payload.items[0].can_send_friend_request);
}
