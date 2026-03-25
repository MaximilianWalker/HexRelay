use super::*;

struct DmFixtureIdentities {
    nora_id: String,
    jules_id: String,
    mina_id: String,
    alex_id: String,
}

impl DmFixtureIdentities {
    fn unique() -> Self {
        Self {
            nora_id: unique_identity("usr-nora-k"),
            jules_id: unique_identity("usr-jules-p"),
            mina_id: unique_identity("usr-mina-s"),
            alex_id: unique_identity("usr-alex-r"),
        }
    }
}

struct SeededDmHistory {
    nora_jules_thread_id: String,
    nora_alex_thread_id: String,
}

async fn seed_default_dm_history(
    pool: &sqlx::PgPool,
    identities: &DmFixtureIdentities,
) -> SeededDmHistory {
    let suffix = Uuid::new_v4().simple().to_string();
    let nora_jules_thread_id = format!("dm-thread-nora-jules-{suffix}");
    let atlas_thread_id = format!("gdm-thread-atlas-{suffix}");
    let nora_alex_thread_id = format!("dm-thread-nora-alex-{suffix}");

    seed_dm_thread(
        pool,
        &nora_jules_thread_id,
        "dm",
        "Nora K + Jules P",
        &[
            (identities.nora_id.as_str(), 401),
            (identities.jules_id.as_str(), 401),
        ],
        &[
            (
                &format!("msg-401-{suffix}"),
                identities.nora_id.as_str(),
                401,
                "enc:88f0ab",
                "2026-03-12T09:05:08Z",
                None,
            ),
            (
                &format!("msg-402-{suffix}"),
                identities.jules_id.as_str(),
                402,
                "enc:5c8e73",
                "2026-03-12T09:12:00Z",
                Some("2026-03-12T09:12:39Z"),
            ),
            (
                &format!("msg-403-{suffix}"),
                identities.nora_id.as_str(),
                403,
                "enc:4bf120",
                "2026-03-12T09:19:24Z",
                None,
            ),
            (
                &format!("msg-404-{suffix}"),
                identities.jules_id.as_str(),
                404,
                "enc:95a0f4",
                "2026-03-12T09:21:11Z",
                None,
            ),
        ],
    )
    .await;

    seed_dm_thread(
        pool,
        &atlas_thread_id,
        "group_dm",
        "Atlas Draft Squad",
        &[
            (identities.nora_id.as_str(), 144),
            (identities.mina_id.as_str(), 145),
            (identities.alex_id.as_str(), 145),
        ],
        &[
            (
                &format!("msg-144-{suffix}"),
                identities.nora_id.as_str(),
                144,
                "enc:bada55",
                "2026-03-12T08:03:19Z",
                None,
            ),
            (
                &format!("msg-145-{suffix}"),
                identities.mina_id.as_str(),
                145,
                "enc:10beef",
                "2026-03-12T08:10:00Z",
                None,
            ),
        ],
    )
    .await;

    seed_dm_thread(
        pool,
        &nora_alex_thread_id,
        "dm",
        "Nora K + Alex R",
        &[
            (identities.nora_id.as_str(), 220),
            (identities.alex_id.as_str(), 220),
        ],
        &[(
            &format!("msg-220-{suffix}"),
            identities.alex_id.as_str(),
            220,
            "enc:deed01",
            "2026-03-11T21:45:30Z",
            None,
        )],
    )
    .await;

    SeededDmHistory {
        nora_jules_thread_id,
        nora_alex_thread_id,
    }
}

#[tokio::test]
async fn lists_dm_threads_with_unread_filter_and_cursor() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.nora_id.as_str()]).await
    else {
        return;
    };
    seed_default_dm_history(&pool, &identities).await;

    let first_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads?unread_only=true&limit=1")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build first dm thread list request");

    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first dm thread list response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let first_body = to_bytes(first_response.into_body(), usize::MAX)
        .await
        .expect("read first dm thread list body");
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first_body).expect("decode first dm thread list body");

    let next_cursor = first_payload["next_cursor"]
        .as_str()
        .expect("next cursor from first page");
    assert_eq!(first_payload["items"].as_array().map(Vec::len), Some(1));

    let second_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads?unread_only=true&limit=1&cursor={next_cursor}"
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build second dm thread list request");

    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second dm thread list response");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second dm thread list body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second dm thread list body");

    assert_eq!(second_payload["items"].as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn lists_dm_thread_messages_with_seq_cursor_pagination() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.nora_id.as_str()]).await
    else {
        return;
    };
    let history = seed_default_dm_history(&pool, &identities).await;

    let first_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/{}/messages?limit=2",
            history.nora_jules_thread_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build first dm messages request");

    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first dm messages response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let first_body = to_bytes(first_response.into_body(), usize::MAX)
        .await
        .expect("read first dm messages body");
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first_body).expect("decode first dm messages body");

    let first_items = first_payload["items"]
        .as_array()
        .expect("first page items array");
    assert_eq!(first_items.len(), 2);
    assert_eq!(first_items[0]["seq"], 404);
    assert_eq!(first_items[1]["seq"], 403);
    let next_cursor = first_payload["next_cursor"]
        .as_str()
        .expect("next cursor from first dm message page");
    assert_eq!(next_cursor, "403");

    let second_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/{}/messages?limit=2&cursor={next_cursor}",
            history.nora_jules_thread_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build second dm messages request");

    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second dm messages response");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second dm messages body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second dm messages body");

    let second_items = second_payload["items"]
        .as_array()
        .expect("second page items array");
    assert_eq!(second_items.len(), 2);
    assert_eq!(second_items[0]["seq"], 402);
    assert_eq!(second_items[1]["seq"], 401);
    assert!(second_payload["next_cursor"].is_null());
}

#[tokio::test]
async fn dm_thread_messages_cursor_is_strictly_exclusive() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.nora_id.as_str()]).await
    else {
        return;
    };
    let history = seed_default_dm_history(&pool, &identities).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/{}/messages?limit=2&cursor=404",
            history.nora_jules_thread_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build exclusive cursor request");

    let response = app
        .oneshot(request)
        .await
        .expect("exclusive cursor response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read exclusive cursor body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode exclusive cursor body");
    let items = payload["items"]
        .as_array()
        .expect("exclusive cursor items array");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["seq"], 403);
    assert_eq!(items[1]["seq"], 402);
    assert_eq!(payload["next_cursor"], "402");
}

#[tokio::test]
async fn dm_thread_messages_returns_empty_page_for_member_without_messages() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };

    let thread_id = format!("dm-thread-empty-{}", Uuid::new_v4().simple());
    seed_dm_thread(
        &pool,
        &thread_id,
        "dm",
        "Nora K + Quiet Contact",
        &[("usr-nora-k", 0), ("usr-quiet-contact", 0)],
        &[],
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/v1/dm/threads/{thread_id}/messages?limit=10"))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
        )
        .body(Body::empty())
        .expect("build empty thread request");

    let response = app.oneshot(request).await.expect("empty thread response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read empty thread body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode empty thread body");

    assert_eq!(payload["items"].as_array().map(Vec::len), Some(0));
    assert!(payload["next_cursor"].is_null());
}

#[tokio::test]
async fn dm_thread_messages_accept_cursor_larger_than_storage_range() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.nora_id.as_str()]).await
    else {
        return;
    };
    let history = seed_default_dm_history(&pool, &identities).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/{}/messages?limit=2&cursor={}",
            history.nora_jules_thread_id,
            u64::MAX
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build large cursor request");

    let response = app.oneshot(request).await.expect("large cursor response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read large cursor body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode large cursor body");
    let items = payload["items"]
        .as_array()
        .expect("large cursor items array");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["seq"], 404);
    assert_eq!(items[1]["seq"], 403);
    assert_eq!(payload["next_cursor"], "403");
}

#[tokio::test]
async fn dm_thread_listing_is_scoped_to_authenticated_identity_membership() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.jules_id.as_str()]).await
    else {
        return;
    };
    let history = seed_default_dm_history(&pool, &identities).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads?limit=10")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.jules_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build dm thread list request");

    let response = app.oneshot(request).await.expect("dm thread list response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read dm thread list body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode dm thread list body");
    let items = payload["items"].as_array().expect("thread items array");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["thread_id"], history.nora_jules_thread_id);
}

#[tokio::test]
async fn dm_thread_messages_return_not_found_for_non_members() {
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.jules_id.as_str()]).await
    else {
        return;
    };
    let history = seed_default_dm_history(&pool, &identities).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/{}/messages?limit=10",
            history.nora_alex_thread_id
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.jules_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build unauthorized dm messages request");

    let response = app
        .oneshot(request)
        .await
        .expect("unauthorized dm messages response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read unauthorized dm messages body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode unauthorized dm messages body");
    assert_eq!(payload["code"], "thread_not_found");
}

#[tokio::test]
async fn unread_only_cursor_restarts_when_cursor_thread_becomes_fully_read() {
    // Documents edge-case: when unread_only=true and the cursor thread's unread
    // count drops to 0 between page requests, the cursor thread is excluded from
    // the filtered CTE. The COALESCE fallback resets pagination to the beginning.
    // This is graceful degradation (restart from page 1), not an error.
    let identities = DmFixtureIdentities::unique();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[identities.nora_id.as_str()]).await
    else {
        return;
    };
    seed_default_dm_history(&pool, &identities).await;

    // Page 1: get first thread with unread_only=true, limit=1
    let first_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads?unread_only=true&limit=1")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build first unread page request");

    let first_response = app
        .clone()
        .oneshot(first_request)
        .await
        .expect("first unread page response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let first_body = to_bytes(first_response.into_body(), usize::MAX)
        .await
        .expect("read first unread page body");
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first_body).expect("decode first unread page body");

    let cursor = first_payload["next_cursor"]
        .as_str()
        .expect("cursor from first unread page");

    // Mark the cursor thread as fully read so its unread drops to 0
    sqlx::query(
        "UPDATE dm_thread_participants SET last_read_seq = (SELECT COALESCE(MAX(seq), 0) FROM dm_messages WHERE thread_id = $1) WHERE thread_id = $1 AND identity_id = $2",
    )
    .bind(cursor)
    .bind(identities.nora_id.as_str())
    .execute(&pool)
    .await
    .expect("mark cursor thread fully read");

    // Page 2: cursor thread now has unread=0, filtered CTE excludes it,
    // pagination restarts from the beginning (graceful degradation).
    let second_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads?unread_only=true&limit=10&cursor={cursor}"
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[identities.nora_id.as_str()]),
        )
        .body(Body::empty())
        .expect("build second unread page request after mark-read");

    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second unread page response after mark-read");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second unread page body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second unread page body");

    // The response succeeds (no error) — items restart from the beginning of
    // the filtered set rather than continuing past the now-excluded cursor.
    let items = second_payload["items"]
        .as_array()
        .expect("items array from restart page");
    assert!(
        !items.is_empty(),
        "restart page should contain remaining unread threads"
    );
}

#[tokio::test]
async fn mark_dm_thread_read_advances_last_read_seq_and_returns_unread() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };

    // Isolated thread to avoid parallel test interference on shared rows.
    seed_dm_thread(
        &pool,
        "dm-mark-advance",
        "dm",
        "Mark Advance Test",
        &[("usr-nora-k", 401), ("usr-jules-p", 401)],
        &[
            (
                "msg-ma-401",
                "usr-nora-k",
                401,
                "enc:aa01",
                "2026-03-12T09:05:08Z",
                None,
            ),
            (
                "msg-ma-402",
                "usr-jules-p",
                402,
                "enc:aa02",
                "2026-03-12T09:12:00Z",
                None,
            ),
            (
                "msg-ma-403",
                "usr-nora-k",
                403,
                "enc:aa03",
                "2026-03-12T09:19:24Z",
                None,
            ),
            (
                "msg-ma-404",
                "usr-jules-p",
                404,
                "enc:aa04",
                "2026-03-12T09:21:11Z",
                None,
            ),
        ],
    )
    .await;

    // Initially last_read_seq=401 for usr-nora-k (4 messages: 401-404).
    // Mark read up to seq 403, expect unread=1 (seq 404 remains unread).
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/threads/dm-mark-advance/read")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"last_read_seq": 403}"#))
        .expect("build mark-read request");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("mark-read response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read mark-read body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode mark-read body");

    assert_eq!(payload["thread_id"], "dm-mark-advance");
    assert_eq!(payload["last_read_seq"], 403);
    assert_eq!(payload["unread"], 1);
}

#[tokio::test]
async fn mark_dm_thread_read_is_monotonic() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };

    // Isolated thread to avoid parallel test interference on shared rows.
    seed_dm_thread(
        &pool,
        "dm-mark-mono",
        "dm",
        "Mark Mono Test",
        &[("usr-nora-k", 401), ("usr-jules-p", 401)],
        &[
            (
                "msg-mm-401",
                "usr-nora-k",
                401,
                "enc:bb01",
                "2026-03-12T09:05:08Z",
                None,
            ),
            (
                "msg-mm-402",
                "usr-jules-p",
                402,
                "enc:bb02",
                "2026-03-12T09:12:00Z",
                None,
            ),
            (
                "msg-mm-403",
                "usr-nora-k",
                403,
                "enc:bb03",
                "2026-03-12T09:19:24Z",
                None,
            ),
            (
                "msg-mm-404",
                "usr-jules-p",
                404,
                "enc:bb04",
                "2026-03-12T09:21:11Z",
                None,
            ),
        ],
    )
    .await;

    // First advance to 403
    let advance_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/threads/dm-mark-mono/read")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"last_read_seq": 403}"#))
        .expect("build advance request");

    let advance_response = app
        .clone()
        .oneshot(advance_request)
        .await
        .expect("advance response");
    assert_eq!(advance_response.status(), StatusCode::OK);

    // Try to regress to 401 — should be a no-op, seq stays at 403
    let regress_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/threads/dm-mark-mono/read")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-nora-k"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"last_read_seq": 401}"#))
        .expect("build regress request");

    let regress_response = app
        .oneshot(regress_request)
        .await
        .expect("regress response");
    assert_eq!(regress_response.status(), StatusCode::OK);

    let body = to_bytes(regress_response.into_body(), usize::MAX)
        .await
        .expect("read regress body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode regress body");

    assert_eq!(payload["last_read_seq"], 403);
    assert_eq!(payload["unread"], 1);
}

#[tokio::test]
async fn mark_dm_thread_read_returns_not_found_for_non_member() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-jules-p"]).await else {
        return;
    };

    // Isolated thread — usr-jules-p is NOT a participant.
    seed_dm_thread(
        &pool,
        "dm-mark-nomember",
        "dm",
        "Mark Non-Member Test",
        &[("usr-nora-k", 100), ("usr-alex-r", 100)],
        &[(
            "msg-mn-100",
            "usr-nora-k",
            100,
            "enc:cc01",
            "2026-03-11T21:45:30Z",
            None,
        )],
    )
    .await;

    // usr-jules-p is not a participant in dm-mark-nomember
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/threads/dm-mark-nomember/read")
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens["usr-jules-p"]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"last_read_seq": 100}"#))
        .expect("build non-member mark-read request");

    let response = app
        .oneshot(request)
        .await
        .expect("non-member mark-read response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read non-member mark-read body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode non-member mark-read body");
    assert_eq!(payload["code"], "thread_not_found");
}
