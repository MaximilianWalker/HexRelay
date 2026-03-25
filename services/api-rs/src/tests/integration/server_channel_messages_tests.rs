use super::*;

struct ServerChannelFixture {
    server_id: String,
    channel_id: String,
    member_id: String,
    outsider_id: String,
    teammate_id: String,
    first_message_id: String,
    second_message_id: String,
}

async fn seed_server_channel_fixture(pool: &sqlx::PgPool) -> ServerChannelFixture {
    let server_id = format!("srv-channel-{}", Uuid::new_v4().simple());
    let channel_id = format!("chn-general-{}", Uuid::new_v4().simple());
    let member_id = unique_identity("usr-channel-member");
    let outsider_id = unique_identity("usr-channel-outsider");
    let teammate_id = unique_identity("usr-channel-teammate");
    let first_message_id = format!("scm-first-{}", Uuid::new_v4().simple());
    let second_message_id = format!("scm-second-{}", Uuid::new_v4().simple());

    seed_server_channel(
        pool,
        &server_id,
        "Channels",
        &channel_id,
        "general",
        &[&member_id, &teammate_id],
        &[
            (
                &first_message_id,
                &member_id,
                1,
                "hello server",
                None,
                &[],
                "2026-03-25T20:40:00Z",
                None,
                None,
            ),
            (
                &second_message_id,
                &teammate_id,
                2,
                "welcome aboard",
                Some(first_message_id.as_str()),
                &[member_id.as_str()],
                "2026-03-25T20:41:00Z",
                None,
                None,
            ),
        ],
    )
    .await;

    ensure_db_identity_key(pool, &outsider_id).await;

    ServerChannelFixture {
        server_id,
        channel_id,
        member_id,
        outsider_id,
        teammate_id,
        first_message_id,
        second_message_id,
    }
}

#[tokio::test]
async fn lists_server_channel_messages_for_members_only() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=2",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build member list request");

    let response = app.oneshot(request).await.expect("member list response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read member list body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode list payload");
    let items = payload["items"].as_array().expect("items array");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["message_id"], fixture.second_message_id);
    assert_eq!(items[0]["reply_to_message_id"], fixture.first_message_id);
    assert_eq!(items[0]["mentions"], serde_json::json!([fixture.member_id]));
    assert_eq!(items[1]["message_id"], fixture.first_message_id);
    assert!(payload["next_cursor"].is_null());
}

#[tokio::test]
async fn rejects_server_channel_listing_for_outsiders() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.outsider_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=2",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.outsider_id]),
        )
        .body(Body::empty())
        .expect("build outsider request");

    let response = app.oneshot(request).await.expect("outsider response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read outsider body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode outsider body");
    assert_eq!(payload["code"], "server_access_denied");
}

#[tokio::test]
async fn returns_not_found_for_unknown_server_channel() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let missing_channel_id = format!("chn-missing-{}", Uuid::new_v4().simple());
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=2",
            fixture.server_id, missing_channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build missing channel request");

    let response = app
        .oneshot(request)
        .await
        .expect("missing channel response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read missing channel body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode missing body");
    assert_eq!(payload["code"], "channel_not_found");
}

#[tokio::test]
async fn paginates_server_channel_messages_by_channel_seq_cursor() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=1",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build first page request");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("first page response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read first page body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode first page");
    assert_eq!(payload["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["items"][0]["message_id"], fixture.second_message_id);
    assert_eq!(payload["next_cursor"], "2");

    let second_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=1&cursor=2",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build second page request");

    let second_response = app
        .oneshot(second_request)
        .await
        .expect("second page response");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second page body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second page");
    assert_eq!(second_payload["items"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        second_payload["items"][0]["message_id"],
        fixture.first_message_id
    );
    assert!(second_payload["next_cursor"].is_null());
}

#[tokio::test]
async fn rejects_server_channel_message_cursor_outside_storage_range() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=1&cursor={}",
            fixture.server_id,
            fixture.channel_id,
            u64::MAX
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build out-of-range cursor request");

    let response = app
        .oneshot(request)
        .await
        .expect("out-of-range cursor response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read out-of-range cursor body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode cursor body");
    assert_eq!(payload["code"], "cursor_out_of_range");
}

#[tokio::test]
async fn creates_server_channel_message_with_reply_and_mentions() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) =
        app_with_database_and_sessions(&[&fixture.member_id, &fixture.teammate_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[&fixture.member_id]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"content":"ship it","reply_to_message_id":"{}","mention_identity_ids":["{}"]}}"#,
            fixture.second_message_id, fixture.teammate_id
        )))
        .expect("build create message request");

    let response = app.oneshot(request).await.expect("create message response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read create message body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode create payload");

    assert_eq!(payload["channel_id"], fixture.channel_id);
    assert_eq!(payload["author_id"], fixture.member_id);
    assert_eq!(payload["reply_to_message_id"], fixture.second_message_id);
    assert_eq!(
        payload["mentions"],
        serde_json::json!([fixture.teammate_id])
    );
    assert_eq!(payload["channel_seq"], 3);
}

#[tokio::test]
async fn rejects_server_channel_message_with_cross_channel_reply_target() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let other_channel_id = format!("chn-other-{}", Uuid::new_v4().simple());
    let other_message_id = format!("scm-other-{}", Uuid::new_v4().simple());
    seed_server_channel(
        &pool,
        &fixture.server_id,
        "Channels",
        &other_channel_id,
        "random",
        &[&fixture.member_id, &fixture.teammate_id],
        &[(
            &other_message_id,
            &fixture.teammate_id,
            1,
            "other channel",
            None,
            &[],
            "2026-03-25T20:42:00Z",
            None,
            None,
        )],
    )
    .await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"content":"bad reply","reply_to_message_id":"{}","mention_identity_ids":[]}}"#,
            other_message_id
        )))
        .expect("build cross-channel reply request");

    let response = app
        .oneshot(request)
        .await
        .expect("cross-channel reply response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read cross-channel reply body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode reply body");
    assert_eq!(payload["code"], "reply_target_invalid");
}

#[tokio::test]
async fn rejects_server_channel_message_with_non_member_mention() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"content":"bad mention","mention_identity_ids":["{}"]}}"#,
            fixture.outsider_id
        )))
        .expect("build invalid mention request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid mention response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid mention body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode mention body");
    assert_eq!(payload["code"], "mention_invalid");
}

#[tokio::test]
async fn rejects_server_channel_message_with_invalid_mention_identity_format() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"content":"bad mention format","mention_identity_ids":[" bad-id "]}"#,
        ))
        .expect("build invalid mention format request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid mention format response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid mention format body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid mention format body");
    assert_eq!(payload["code"], "mention_invalid");
}

#[tokio::test]
async fn normalizes_blank_reply_target_to_none_on_server_channel_create() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"content":"blank reply target","reply_to_message_id":"   "}"#,
        ))
        .expect("build blank reply target request");

    let response = app
        .oneshot(request)
        .await
        .expect("blank reply target response");
    assert_eq!(response.status(), StatusCode::CREATED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read blank reply target body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode blank reply target body");
    assert!(payload["reply_to_message_id"].is_null());
}

#[tokio::test]
async fn author_can_edit_server_channel_message_and_replace_mentions() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) =
        app_with_database_and_sessions(&[&fixture.member_id, &fixture.teammate_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "cookie",
            format!(
                "hexrelay_session={}; hexrelay_csrf=test-csrf",
                tokens[&fixture.member_id]
            ),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"content":"edited content","mention_identity_ids":["{}"]}}"#,
            fixture.teammate_id
        )))
        .expect("build edit request");

    let response = app.oneshot(request).await.expect("edit response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read edit body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode edit body");
    assert_eq!(payload["content"], "edited content");
    assert_eq!(
        payload["mentions"],
        serde_json::json!([fixture.teammate_id])
    );
    assert_eq!(payload["reply_to_message_id"], serde_json::Value::Null);
    assert_eq!(payload["channel_seq"], 1);
    assert!(payload["edited_at"].is_string());
}

#[tokio::test]
async fn no_op_server_channel_edit_preserves_null_edited_at() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) =
        app_with_database_and_sessions(&[&fixture.teammate_id, &fixture.member_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.second_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.teammate_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"content":"welcome aboard","mention_identity_ids":["{}"]}}"#,
            fixture.member_id
        )))
        .expect("build no-op edit request");

    let response = app.oneshot(request).await.expect("no-op edit response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read no-op edit body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode no-op edit");
    assert!(payload["edited_at"].is_null());
}

#[tokio::test]
async fn rejects_server_channel_edit_for_non_author_member() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) =
        app_with_database_and_sessions(&[&fixture.member_id, &fixture.teammate_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.second_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"content":"not allowed","mention_identity_ids":[]}"#,
        ))
        .expect("build forbidden edit request");

    let response = app.oneshot(request).await.expect("forbidden edit response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read forbidden edit body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode forbidden edit body");
    assert_eq!(payload["code"], "message_edit_forbidden");
}

#[tokio::test]
async fn rejects_server_channel_edit_for_deleted_message() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build delete request");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let edit_request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"content":"cannot revive","mention_identity_ids":[]}"#,
        ))
        .expect("build edit deleted request");

    let response = app
        .oneshot(edit_request)
        .await
        .expect("edit deleted response");
    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read edit deleted body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode edit deleted body");
    assert_eq!(payload["code"], "message_deleted");
}

#[tokio::test]
async fn soft_delete_server_channel_message_returns_tombstone_and_is_idempotent() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build delete request");

    let response = app.clone().oneshot(request).await.expect("delete response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read delete body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode delete body");
    assert_eq!(payload["content"], "");
    assert_eq!(payload["mentions"], serde_json::json!([]));
    assert!(payload["deleted_at"].is_string());

    let repeat_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build repeat delete request");

    let repeat_response = app
        .oneshot(repeat_request)
        .await
        .expect("repeat delete response");
    assert_eq!(repeat_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn rejects_server_channel_delete_for_non_author_member() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) =
        app_with_database_and_sessions(&[&fixture.member_id, &fixture.teammate_id]).await
    else {
        return;
    };

    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.second_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build forbidden delete request");

    let response = app
        .oneshot(request)
        .await
        .expect("forbidden delete response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read forbidden delete body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode forbidden delete body");
    assert_eq!(payload["code"], "message_delete_forbidden");
}

#[tokio::test]
async fn deleted_server_channel_messages_remain_visible_as_tombstones() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build delete request");
    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete response");
    assert_eq!(delete_response.status(), StatusCode::OK);

    let list_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=2",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.member_id]),
        )
        .body(Body::empty())
        .expect("build list request");

    let list_response = app.oneshot(list_request).await.expect("list response");
    assert_eq!(list_response.status(), StatusCode::OK);

    let body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read list body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode list body");
    let items = payload["items"].as_array().expect("items array");
    assert_eq!(items[1]["message_id"], fixture.first_message_id);
    assert_eq!(items[1]["content"], "");
    assert_eq!(items[1]["mentions"], serde_json::json!([]));
    assert!(items[1]["deleted_at"].is_string());
}
