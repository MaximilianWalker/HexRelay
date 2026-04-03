use super::*;

#[tokio::test]
async fn presence_watchers_require_internal_token() {
    let app = build_app(AppState::default());
    let request = Request::builder()
        .method("GET")
        .uri("/v1/internal/presence/watchers/usr-main")
        .body(Body::empty())
        .expect("build watcher request");

    let response = app.oneshot(request).await.expect("watcher response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn presence_watchers_include_self_and_accepted_unblocked_friends_only() {
    let state = AppState::default();
    state
        .friend_requests
        .write()
        .expect("friend request write lock")
        .extend([
            (
                "req-accepted".to_string(),
                crate::models::FriendRequestRecord {
                    request_id: "req-accepted".to_string(),
                    requester_identity_id: "usr-main".to_string(),
                    target_identity_id: "usr-friend".to_string(),
                    status: "accepted".to_string(),
                    created_at: Utc::now().to_rfc3339(),
                },
            ),
            (
                "req-pending".to_string(),
                crate::models::FriendRequestRecord {
                    request_id: "req-pending".to_string(),
                    requester_identity_id: "usr-main".to_string(),
                    target_identity_id: "usr-pending".to_string(),
                    status: "pending".to_string(),
                    created_at: Utc::now().to_rfc3339(),
                },
            ),
            (
                "req-blocked".to_string(),
                crate::models::FriendRequestRecord {
                    request_id: "req-blocked".to_string(),
                    requester_identity_id: "usr-blocked".to_string(),
                    target_identity_id: "usr-main".to_string(),
                    status: "accepted".to_string(),
                    created_at: Utc::now().to_rfc3339(),
                },
            ),
        ]);
    state
        .blocked_users
        .write()
        .expect("blocked users write lock")
        .insert(
            "usr-main".to_string(),
            HashMap::from([("usr-blocked".to_string(), Utc::now().timestamp())]),
        );

    let app = build_app(state.clone());
    let request = Request::builder()
        .method("GET")
        .uri("/v1/internal/presence/watchers/usr-main")
        .header(
            "x-hexrelay-internal-token",
            state.presence_watcher_internal_token.as_str(),
        )
        .body(Body::empty())
        .expect("build watcher request");

    let response = app.oneshot(request).await.expect("watcher response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read watcher response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode watcher payload");
    let watchers = payload["watchers"]
        .as_array()
        .expect("watchers array")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();

    assert!(watchers.contains(&"usr-main"));
    assert!(watchers.contains(&"usr-friend"));
    assert!(!watchers.contains(&"usr-pending"));
    assert!(!watchers.contains(&"usr-blocked"));
}

#[tokio::test]
async fn presence_watchers_reject_channel_dispatch_token() {
    let state = AppState::default();
    let app = build_app(state);
    let request = Request::builder()
        .method("GET")
        .uri("/v1/internal/presence/watchers/usr-main")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .body(Body::empty())
        .expect("build watcher request");

    let response = app.oneshot(request).await.expect("watcher response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
