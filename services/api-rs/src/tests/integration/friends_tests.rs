use super::*;

// ── Bootstrap endpoint tests ──────────────────────────────────────────

#[tokio::test]
async fn bootstrap_returns_peer_identity_after_acceptance() {
    let (app, tokens) = app_with_sessions(&["usr-p", "usr-q"]);

    // Seed identity keys for both parties so the bootstrap handler can resolve them.
    // Keys must be valid 32-byte ed25519 public keys in hex (64 hex chars).
    let key_p = "aa".repeat(32);
    let key_q = "bb".repeat(32);
    let app = register_identity_expect_success(app, "usr-p", &key_p).await;
    let app = register_identity_expect_success(app, "usr-q", &key_q).await;

    // Create a friend request: usr-p → usr-q
    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-p"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-p","target_identity_id":"usr-q"}"#,
        ))
        .expect("build create request");
    let create_resp = app.clone().oneshot(create_req).await.expect("create resp");
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord = serde_json::from_slice(&create_body).expect("decode create");

    // Accept the friend request (target accepts).
    let accept_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-q"]))
        .body(Body::empty())
        .expect("build accept request");
    let accept_resp = app.clone().oneshot(accept_req).await.expect("accept resp");
    assert_eq!(accept_resp.status(), StatusCode::OK);

    // Requester fetches bootstrap → should get target's identity material.
    let bootstrap_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-p"]))
        .body(Body::empty())
        .expect("build bootstrap request (requester)");
    let bootstrap_resp = app
        .clone()
        .oneshot(bootstrap_req)
        .await
        .expect("bootstrap resp (requester)");
    assert_eq!(bootstrap_resp.status(), StatusCode::OK);
    let bootstrap_body = to_bytes(bootstrap_resp.into_body(), usize::MAX)
        .await
        .expect("read bootstrap body");
    let bundle: IdentityBootstrapBundle =
        serde_json::from_slice(&bootstrap_body).expect("decode bootstrap bundle");
    assert_eq!(bundle.identity_id, "usr-q");
    assert_eq!(bundle.public_key, "bb".repeat(32));
    assert_eq!(bundle.algorithm, "ed25519");

    // Target fetches bootstrap → should get requester's identity material.
    let bootstrap_req2 = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-q"]))
        .body(Body::empty())
        .expect("build bootstrap request (target)");
    let bootstrap_resp2 = app
        .clone()
        .oneshot(bootstrap_req2)
        .await
        .expect("bootstrap resp (target)");
    assert_eq!(bootstrap_resp2.status(), StatusCode::OK);
    let bootstrap_body2 = to_bytes(bootstrap_resp2.into_body(), usize::MAX)
        .await
        .expect("read bootstrap body (target)");
    let bundle2: IdentityBootstrapBundle =
        serde_json::from_slice(&bootstrap_body2).expect("decode bootstrap bundle (target)");
    assert_eq!(bundle2.identity_id, "usr-p");
    assert_eq!(bundle2.public_key, "aa".repeat(32));
    assert_eq!(bundle2.algorithm, "ed25519");
}

#[tokio::test]
async fn bootstrap_returns_403_on_pending_request() {
    let (app, tokens) = app_with_sessions(&["usr-r", "usr-s"]);

    let app = register_identity_expect_success(app, "usr-r", &"cc".repeat(32)).await;
    let app = register_identity_expect_success(app, "usr-s", &"dd".repeat(32)).await;

    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-r"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-r","target_identity_id":"usr-s"}"#,
        ))
        .expect("build create request");
    let create_resp = app.clone().oneshot(create_req).await.expect("create resp");
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord = serde_json::from_slice(&create_body).expect("decode create");

    // Bootstrap while still pending → 403
    let bootstrap_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-r"]))
        .body(Body::empty())
        .expect("build bootstrap request");
    let bootstrap_resp = app.oneshot(bootstrap_req).await.expect("bootstrap resp");
    assert_eq!(bootstrap_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn bootstrap_returns_403_on_declined_request() {
    let (app, tokens) = app_with_sessions(&["usr-t", "usr-u"]);

    let app = register_identity_expect_success(app, "usr-t", &"ee".repeat(32)).await;
    let app = register_identity_expect_success(app, "usr-u", &"ff".repeat(32)).await;

    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-t"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-t","target_identity_id":"usr-u"}"#,
        ))
        .expect("build create request");
    let create_resp = app.clone().oneshot(create_req).await.expect("create resp");
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord = serde_json::from_slice(&create_body).expect("decode create");

    // Decline it
    let decline_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/decline",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-u"]))
        .body(Body::empty())
        .expect("build decline request");
    let decline_resp = app
        .clone()
        .oneshot(decline_req)
        .await
        .expect("decline resp");
    assert_eq!(decline_resp.status(), StatusCode::NO_CONTENT);

    // Bootstrap after decline → 403
    let bootstrap_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-t"]))
        .body(Body::empty())
        .expect("build bootstrap request");
    let bootstrap_resp = app.oneshot(bootstrap_req).await.expect("bootstrap resp");
    assert_eq!(bootstrap_resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn bootstrap_returns_401_for_third_party() {
    let (app, tokens) = app_with_sessions(&["usr-v", "usr-w", "usr-x"]);

    let app = register_identity_expect_success(app, "usr-v", &"11".repeat(32)).await;
    let app = register_identity_expect_success(app, "usr-w", &"22".repeat(32)).await;

    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-v"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-v","target_identity_id":"usr-w"}"#,
        ))
        .expect("build create request");
    let create_resp = app.clone().oneshot(create_req).await.expect("create resp");
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord = serde_json::from_slice(&create_body).expect("decode create");

    // Accept so status is accepted
    let accept_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-w"]))
        .body(Body::empty())
        .expect("build accept request");
    let accept_resp = app.clone().oneshot(accept_req).await.expect("accept resp");
    assert_eq!(accept_resp.status(), StatusCode::OK);

    // Third party (usr-x) tries to fetch bootstrap → 401
    let bootstrap_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-x"]))
        .body(Body::empty())
        .expect("build bootstrap request (third party)");
    let bootstrap_resp = app
        .oneshot(bootstrap_req)
        .await
        .expect("bootstrap resp (third party)");
    assert_eq!(bootstrap_resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn bootstrap_returns_400_for_nonexistent_request() {
    let (app, tokens) = app_with_sessions(&["usr-y"]);

    let bootstrap_req = Request::builder()
        .method("GET")
        .uri("/v1/friends/requests/nonexistent-id/bootstrap")
        .header("authorization", format!("Bearer {}", tokens["usr-y"]))
        .body(Body::empty())
        .expect("build bootstrap request");
    let bootstrap_resp = app.oneshot(bootstrap_req).await.expect("bootstrap resp");
    assert_eq!(bootstrap_resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn bootstrap_returns_403_when_peer_is_blocked_after_acceptance() {
    let (app, tokens) = app_with_sessions(&["usr-p", "usr-q"]);

    let app = register_identity_expect_success(app, "usr-p", &"aa".repeat(32)).await;
    let app = register_identity_expect_success(app, "usr-q", &"bb".repeat(32)).await;

    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-p"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-p","target_identity_id":"usr-q"}"#,
        ))
        .expect("build create request");
    let create_resp = app.clone().oneshot(create_req).await.expect("create resp");
    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let create_body = to_bytes(create_resp.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord = serde_json::from_slice(&create_body).expect("decode create");

    let accept_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-q"]))
        .body(Body::empty())
        .expect("build accept request");
    let accept_resp = app.clone().oneshot(accept_req).await.expect("accept resp");
    assert_eq!(accept_resp.status(), StatusCode::OK);

    let block_req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-q"]))
        .body(Body::from(r#"{"target_identity_id":"usr-p"}"#))
        .expect("build block request");
    let block_resp = app.clone().oneshot(block_req).await.expect("block resp");
    assert_eq!(block_resp.status(), StatusCode::CREATED);

    let bootstrap_req = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/friends/requests/{}/bootstrap",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-p"]))
        .body(Body::empty())
        .expect("build bootstrap request");
    let bootstrap_resp = app.oneshot(bootstrap_req).await.expect("bootstrap resp");
    assert_eq!(bootstrap_resp.status(), StatusCode::FORBIDDEN);
}

// ── Existing friend request tests ─────────────────────────────────────

#[tokio::test]
async fn creates_and_lists_friend_requests() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-nora-k","target_identity_id":"usr-jules-p"}"#,
        ))
        .expect("build friend request create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create friend request response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let list_request = Request::builder()
        .method("GET")
        .uri("/v1/friends/requests?identity_id=usr-jules-p&direction=inbound")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .body(Body::empty())
        .expect("build friend request list request");

    let list_response = app
        .oneshot(list_request)
        .await
        .expect("list friend request response");
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read friend request list body");
    let list_payload: FriendRequestPage =
        serde_json::from_slice(&list_body).expect("decode friend request page");
    assert_eq!(list_payload.items.len(), 1);
}

#[tokio::test]
async fn accepts_and_declines_friend_requests() {
    let (app, tokens) = app_with_sessions(&["usr-mina-s", "usr-alex-r", "usr-nora-k"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-mina-s"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-mina-s","target_identity_id":"usr-alex-r"}"#,
        ))
        .expect("build create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create response");

    let accept_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-alex-r"]))
        .body(Body::empty())
        .expect("build accept request");
    let accept_response = app
        .clone()
        .oneshot(accept_request)
        .await
        .expect("accept response");
    assert_eq!(accept_response.status(), StatusCode::OK);

    let create_decline_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-nora-k","target_identity_id":"usr-alex-r"}"#,
        ))
        .expect("build create decline request");

    let create_decline_response = app
        .clone()
        .oneshot(create_decline_request)
        .await
        .expect("create decline response");
    assert_eq!(create_decline_response.status(), StatusCode::CREATED);

    let decline_body = to_bytes(create_decline_response.into_body(), usize::MAX)
        .await
        .expect("read decline create body");
    let decline_created: FriendRequestRecord =
        serde_json::from_slice(&decline_body).expect("decode decline create");

    let decline_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/decline",
            decline_created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-alex-r"]))
        .body(Body::empty())
        .expect("build decline request");
    let decline_response = app
        .oneshot(decline_request)
        .await
        .expect("decline response");
    assert_eq!(decline_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rejects_conflicting_transition_when_not_pending() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-a","target_identity_id":"usr-b"}"#,
        ))
        .expect("build create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create response");

    let first_accept = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-b"]))
        .body(Body::empty())
        .expect("build first accept");
    let first_accept_response = app
        .clone()
        .oneshot(first_accept)
        .await
        .expect("first accept response");
    assert_eq!(first_accept_response.status(), StatusCode::OK);

    let idempotent_accept = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-b"]))
        .body(Body::empty())
        .expect("build idempotent accept");
    let idempotent_accept_response = app
        .clone()
        .oneshot(idempotent_accept)
        .await
        .expect("idempotent accept response");
    assert_eq!(idempotent_accept_response.status(), StatusCode::OK);

    let conflicting_decline = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/decline",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-b"]))
        .body(Body::empty())
        .expect("build conflicting decline");
    let conflicting_decline_response = app
        .oneshot(conflicting_decline)
        .await
        .expect("conflicting decline response");
    assert_eq!(conflicting_decline_response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn requester_can_cancel_pending_friend_request() {
    let (app, tokens) = app_with_sessions(&["usr-c", "usr-d"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-c"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-c","target_identity_id":"usr-d"}"#,
        ))
        .expect("build create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create response");

    let cancel_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/cancel",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-c"]))
        .body(Body::empty())
        .expect("build cancel request");

    let cancel_response = app.oneshot(cancel_request).await.expect("cancel response");
    assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn rejects_transition_by_wrong_actor() {
    let (app, tokens) = app_with_sessions(&["usr-e", "usr-f", "usr-g"]);

    let create_request = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-e"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-e","target_identity_id":"usr-f"}"#,
        ))
        .expect("build create request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create response");
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body = to_bytes(create_response.into_body(), usize::MAX)
        .await
        .expect("read create body");
    let created: FriendRequestRecord =
        serde_json::from_slice(&create_body).expect("decode create response");

    let wrong_actor_accept = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/friends/requests/{}/accept",
            created.request_id
        ))
        .header("authorization", format!("Bearer {}", tokens["usr-g"]))
        .body(Body::empty())
        .expect("build wrong actor accept request");

    let wrong_actor_response = app
        .oneshot(wrong_actor_accept)
        .await
        .expect("wrong actor response");
    assert_eq!(wrong_actor_response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn returns_bad_request_for_non_actionable_friend_request_transitions() {
    let (app, tokens) = app_with_sessions(&["usr-h", "usr-i"]);

    let missing_accept = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests/missing-request/accept")
        .header("authorization", format!("Bearer {}", tokens["usr-i"]))
        .body(Body::empty())
        .expect("build missing accept request");
    let missing_accept_response = app
        .clone()
        .oneshot(missing_accept)
        .await
        .expect("missing accept response");
    assert_eq!(missing_accept_response.status(), StatusCode::BAD_REQUEST);

    let missing_decline = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests/missing-request/decline")
        .header("authorization", format!("Bearer {}", tokens["usr-i"]))
        .body(Body::empty())
        .expect("build missing decline request");
    let missing_decline_response = app
        .clone()
        .oneshot(missing_decline)
        .await
        .expect("missing decline response");
    assert_eq!(missing_decline_response.status(), StatusCode::BAD_REQUEST);

    let missing_cancel = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests/missing-request/cancel")
        .header("authorization", format!("Bearer {}", tokens["usr-h"]))
        .body(Body::empty())
        .expect("build missing cancel request");
    let missing_cancel_response = app
        .oneshot(missing_cancel)
        .await
        .expect("missing cancel response");
    assert_eq!(missing_cancel_response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_friend_request_without_authorization_header() {
    let app = build_app(AppState::default());

    let request = Request::builder()
        .method("GET")
        .uri("/v1/friends/requests?identity_id=usr-a")
        .body(Body::empty())
        .expect("build friend request list request");

    let response = app
        .oneshot(request)
        .await
        .expect("friend request list response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
