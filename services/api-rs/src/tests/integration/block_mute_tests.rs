use super::*;

// ── Deserialization helpers ───────────────────────────────────────────

#[derive(Deserialize)]
struct BlockListResponse {
    items: Vec<BlockRecordResp>,
}

#[derive(Deserialize)]
struct BlockRecordResp {
    blocker_identity_id: String,
    blocked_identity_id: String,
}

#[derive(Deserialize)]
struct MuteListResponse {
    items: Vec<MuteRecordResp>,
}

#[derive(Deserialize)]
struct MuteRecordResp {
    muter_identity_id: String,
    muted_identity_id: String,
}

#[derive(Deserialize)]
struct DmFanoutDispatchResp {
    status: String,
    reason_code: String,
}

#[derive(Deserialize)]
struct DmPreflightResp {
    status: String,
    reason_code: String,
}

// ── Block CRUD ────────────────────────────────────────────────────────

#[tokio::test]
async fn block_user_and_list_returns_blocked_entry() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // Block usr-b
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List blocked users
    let req = Request::builder()
        .method("GET")
        .uri("/v1/users/blocked")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::empty())
        .expect("build list blocked request");
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .expect("list blocked response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let list: BlockListResponse = serde_json::from_slice(&body).expect("decode block list");
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].blocker_identity_id, "usr-a");
    assert_eq!(list.items[0].blocked_identity_id, "usr-b");
}

#[tokio::test]
async fn unblock_user_removes_from_list() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // Block then unblock
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/unblock")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build unblock request");
    let resp = app.clone().oneshot(req).await.expect("unblock response");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // List should be empty
    let req = Request::builder()
        .method("GET")
        .uri("/v1/users/blocked")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::empty())
        .expect("build list blocked request");
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .expect("list blocked response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let list: BlockListResponse = serde_json::from_slice(&body).expect("decode block list");
    assert_eq!(list.items.len(), 0);
}

#[tokio::test]
async fn double_block_returns_409_conflict() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    let build_block = || {
        Request::builder()
            .method("POST")
            .uri("/v1/users/block")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-a"]))
            .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
            .expect("build block request")
    };

    let resp = app
        .clone()
        .oneshot(build_block())
        .await
        .expect("first block");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .clone()
        .oneshot(build_block())
        .await
        .expect("second block");
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn self_block_returns_400() {
    let (app, tokens) = app_with_sessions(&["usr-a"]);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-a"}"#))
        .expect("build self-block request");
    let resp = app.clone().oneshot(req).await.expect("self-block response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Mute CRUD ─────────────────────────────────────────────────────────

#[tokio::test]
async fn mute_user_and_list_returns_muted_entry() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/mute")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build mute request");
    let resp = app.clone().oneshot(req).await.expect("mute response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = Request::builder()
        .method("GET")
        .uri("/v1/users/muted")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::empty())
        .expect("build list muted request");
    let resp = app.clone().oneshot(req).await.expect("list muted response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let list: MuteListResponse = serde_json::from_slice(&body).expect("decode mute list");
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].muter_identity_id, "usr-a");
    assert_eq!(list.items[0].muted_identity_id, "usr-b");
}

#[tokio::test]
async fn unmute_user_removes_from_list() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/mute")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build mute request");
    let resp = app.clone().oneshot(req).await.expect("mute response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/unmute")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build unmute request");
    let resp = app.clone().oneshot(req).await.expect("unmute response");
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    let req = Request::builder()
        .method("GET")
        .uri("/v1/users/muted")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::empty())
        .expect("build list muted request");
    let resp = app.clone().oneshot(req).await.expect("list muted response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body");
    let list: MuteListResponse = serde_json::from_slice(&body).expect("decode mute list");
    assert_eq!(list.items.len(), 0);
}

#[tokio::test]
async fn self_mute_returns_400() {
    let (app, tokens) = app_with_sessions(&["usr-a"]);

    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/mute")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-a"}"#))
        .expect("build self-mute request");
    let resp = app.clone().oneshot(req).await.expect("self-mute response");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Fanout filter: block prevents DM fanout ───────────────────────────

#[tokio::test]
async fn block_prevents_dm_fanout_dispatch() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // Block usr-b
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Attempt DM fanout to blocked user
    let req = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-b","ciphertext":"dGVzdA==","message_id":"msg-001"}"#,
        ))
        .expect("build fanout request");
    let resp = app.clone().oneshot(req).await.expect("fanout response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let fanout: DmFanoutDispatchResp = serde_json::from_slice(&body).expect("decode fanout");
    assert_eq!(fanout.status, "blocked");
    assert_eq!(fanout.reason_code, "fanout_blocked_user");
}

#[tokio::test]
async fn reverse_block_prevents_dm_fanout_dispatch() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // usr-b blocks usr-a (reverse direction)
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-b"]))
        .body(Body::from(r#"{"target_identity_id":"usr-a"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // usr-a tries DM fanout to usr-b (who blocked them)
    let req = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-b","ciphertext":"dGVzdA==","message_id":"msg-002"}"#,
        ))
        .expect("build fanout request");
    let resp = app.clone().oneshot(req).await.expect("fanout response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let fanout: DmFanoutDispatchResp = serde_json::from_slice(&body).expect("decode fanout");
    assert_eq!(fanout.status, "blocked");
    assert_eq!(fanout.reason_code, "fanout_blocked_user");
}

// ── Fanout filter: block prevents friend request ──────────────────────

#[tokio::test]
async fn block_prevents_friend_request_creation() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // Block usr-b
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Attempt friend request to blocked user
    let req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-a","target_identity_id":"usr-b"}"#,
        ))
        .expect("build friend request");
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .expect("friend request response");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn reverse_block_prevents_friend_request_creation() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // usr-b blocks usr-a
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-b"]))
        .body(Body::from(r#"{"target_identity_id":"usr-a"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // usr-a tries to send friend request to usr-b (who blocked them)
    let req = Request::builder()
        .method("POST")
        .uri("/v1/friends/requests")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"requester_identity_id":"usr-a","target_identity_id":"usr-b"}"#,
        ))
        .expect("build friend request");
    let resp = app
        .clone()
        .oneshot(req)
        .await
        .expect("friend request response");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Preflight block check ─────────────────────────────────────────────

#[tokio::test]
async fn block_prevents_dm_connectivity_preflight() {
    let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

    // Block usr-b
    let req = Request::builder()
        .method("POST")
        .uri("/v1/users/block")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(r#"{"target_identity_id":"usr-b"}"#))
        .expect("build block request");
    let resp = app.clone().oneshot(req).await.expect("block response");
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Attempt connectivity preflight to blocked user
    let req = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/preflight")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", tokens["usr-a"]))
        .body(Body::from(
            r#"{"peer_identity_id":"usr-b","pairing_envelope_present":true,"local_bind_allowed":true,"peer_reachable_hint":true}"#,
        ))
        .expect("build preflight request");
    let resp = app.clone().oneshot(req).await.expect("preflight response");
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read preflight body");
    let preflight: DmPreflightResp = serde_json::from_slice(&body).expect("decode preflight");
    assert_eq!(preflight.status, "blocked");
    assert_eq!(preflight.reason_code, "preflight_blocked_user");
}
