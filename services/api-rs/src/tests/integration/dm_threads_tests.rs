use super::*;

async fn seed_default_dm_history(pool: &sqlx::PgPool) {
    seed_dm_thread(
        pool,
        "dm-thread-nora-jules",
        "dm",
        "Nora K + Jules P",
        &[("usr-nora-k", 401), ("usr-jules-p", 401)],
        &[
            (
                "msg-401",
                "usr-nora-k",
                401,
                "enc:88f0ab",
                "2026-03-12T09:05:08Z",
                None,
            ),
            (
                "msg-402",
                "usr-jules-p",
                402,
                "enc:5c8e73",
                "2026-03-12T09:12:00Z",
                Some("2026-03-12T09:12:39Z"),
            ),
            (
                "msg-403",
                "usr-nora-k",
                403,
                "enc:4bf120",
                "2026-03-12T09:19:24Z",
                None,
            ),
            (
                "msg-404",
                "usr-jules-p",
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
        "gdm-thread-atlas",
        "group_dm",
        "Atlas Draft Squad",
        &[
            ("usr-nora-k", 144),
            ("usr-mina-s", 145),
            ("usr-alex-r", 145),
        ],
        &[
            (
                "msg-144",
                "usr-nora-k",
                144,
                "enc:bada55",
                "2026-03-12T08:03:19Z",
                None,
            ),
            (
                "msg-145",
                "usr-mina-s",
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
        "dm-thread-nora-alex",
        "dm",
        "Nora K + Alex R",
        &[("usr-nora-k", 220), ("usr-alex-r", 220)],
        &[(
            "msg-220",
            "usr-alex-r",
            220,
            "enc:deed01",
            "2026-03-11T21:45:30Z",
            None,
        )],
    )
    .await;
}

#[tokio::test]
async fn lists_dm_threads_with_unread_filter_and_cursor() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };
    seed_default_dm_history(&pool).await;

    let first_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads?unread_only=true&limit=1")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
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
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
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
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-nora-k"]).await else {
        return;
    };
    seed_default_dm_history(&pool).await;

    let first_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads/dm-thread-nora-jules/messages?limit=2")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
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

    assert_eq!(first_payload["items"].as_array().map(Vec::len), Some(2));
    let next_cursor = first_payload["next_cursor"]
        .as_str()
        .expect("next cursor from first dm message page");

    let second_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/dm/threads/dm-thread-nora-jules/messages?limit=2&cursor={next_cursor}"
        ))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-nora-k"]),
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

    assert_eq!(second_payload["items"].as_array().map(Vec::len), Some(2));
    assert!(second_payload["next_cursor"].is_null());
}

#[tokio::test]
async fn dm_thread_listing_is_scoped_to_authenticated_identity_membership() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-jules-p"]).await else {
        return;
    };
    seed_default_dm_history(&pool).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads?limit=10")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-jules-p"]),
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
    assert_eq!(items[0]["thread_id"], "dm-thread-nora-jules");
}

#[tokio::test]
async fn dm_thread_messages_return_not_found_for_non_members() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-jules-p"]).await else {
        return;
    };
    seed_default_dm_history(&pool).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/dm/threads/dm-thread-nora-alex/messages?limit=10")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-jules-p"]),
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
