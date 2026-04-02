use super::*;

use futures::StreamExt;
use realtime_rs::{
    app::{build_app as build_realtime_app, AppState as RealtimeAppState},
    domain::{channels::spawn_channel_subscriber, presence::spawn_presence_subscriber},
};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message as WsMessage},
};

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
async fn rejects_server_channel_message_routes_without_authentication() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, _, _)) = app_with_database_and_sessions(&[&fixture.member_id]).await else {
        return;
    };

    let list_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages?limit=2",
            fixture.server_id, fixture.channel_id
        ))
        .body(Body::empty())
        .expect("build unauthenticated list request");
    let create_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "hello" }).to_string(),
        ))
        .expect("build unauthenticated create request");
    let edit_request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "edited" }).to_string(),
        ))
        .expect("build unauthenticated edit request");
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .body(Body::empty())
        .expect("build unauthenticated delete request");

    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("list response");
    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("create response");
    let edit_response = app
        .clone()
        .oneshot(edit_request)
        .await
        .expect("edit response");
    let delete_response = app.oneshot(delete_request).await.expect("delete response");

    for response in [
        list_response,
        create_response,
        edit_response,
        delete_response,
    ] {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read unauthenticated response body");
        let payload: serde_json::Value =
            serde_json::from_slice(&body).expect("decode unauthenticated response body");
        assert_eq!(payload["code"], "session_invalid");
    }
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
async fn rejects_server_channel_message_create_when_channel_is_not_in_requested_server() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let member_id = unique_identity("usr-create-cross-server");
    let server_a = format!("srv-create-cross-a-{}", Uuid::new_v4().simple());
    let server_b = format!("srv-create-cross-b-{}", Uuid::new_v4().simple());
    let channel_b = format!("chn-create-cross-b-{}", Uuid::new_v4().simple());

    seed_server_membership(&pool, &server_a, "Server A", &member_id, false, false, 0).await;
    seed_server_membership(&pool, &server_b, "Server B", &member_id, false, false, 0).await;
    server_channels_repo::insert_server_channel(
        &pool,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: &channel_b,
            server_id: &server_b,
            name: "server-b-only",
            kind: "text",
        },
    )
    .await
    .expect("insert server B channel");

    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{server_a}/channels/{channel_b}/messages"
        ))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "hello" }).to_string(),
        ))
        .expect("build cross-server create request");

    let response = app
        .oneshot(request)
        .await
        .expect("cross-server create response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read create response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode create body");
    assert_eq!(payload["code"], "server_access_denied");
}

#[tokio::test]
async fn rejects_server_channel_message_patch_and_delete_when_channel_is_not_in_requested_server() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let member_id = unique_identity("usr-mutate-cross-server");
    let server_a = format!("srv-mutate-cross-a-{}", Uuid::new_v4().simple());
    let server_b = format!("srv-mutate-cross-b-{}", Uuid::new_v4().simple());
    let channel_b = format!("chn-mutate-cross-b-{}", Uuid::new_v4().simple());
    let message_b = format!("scm-mutate-cross-b-{}", Uuid::new_v4().simple());

    seed_server_membership(&pool, &server_a, "Server A", &member_id, false, false, 0).await;
    seed_server_membership(&pool, &server_b, "Server B", &member_id, false, false, 0).await;
    seed_server_channel(
        &pool,
        &server_b,
        "Server B",
        &channel_b,
        "server-b-only",
        &[&member_id],
        &[(
            &message_b,
            &member_id,
            1,
            "server b message",
            None,
            &[],
            "2026-03-26T03:00:00Z",
            None,
            None,
        )],
    )
    .await;

    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    let patch_request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{server_a}/channels/{channel_b}/messages/{message_b}"
        ))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "edited" }).to_string(),
        ))
        .expect("build cross-server patch request");
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{server_a}/channels/{channel_b}/messages/{message_b}"
        ))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build cross-server delete request");

    let patch_response = app
        .clone()
        .oneshot(patch_request)
        .await
        .expect("patch response");
    let delete_response = app.oneshot(delete_request).await.expect("delete response");

    assert_eq!(patch_response.status(), StatusCode::FORBIDDEN);
    assert_eq!(delete_response.status(), StatusCode::FORBIDDEN);

    let patch_body = to_bytes(patch_response.into_body(), usize::MAX)
        .await
        .expect("read patch body");
    let delete_body = to_bytes(delete_response.into_body(), usize::MAX)
        .await
        .expect("read delete body");
    let patch_payload: serde_json::Value =
        serde_json::from_slice(&patch_body).expect("decode patch body");
    let delete_payload: serde_json::Value =
        serde_json::from_slice(&delete_body).expect("decode delete body");
    assert_eq!(patch_payload["code"], "server_access_denied");
    assert_eq!(delete_payload["code"], "server_access_denied");
}

#[tokio::test]
async fn rejects_server_channel_message_mutation_routes_for_outsiders() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let fixture = seed_server_channel_fixture(&pool).await;
    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&fixture.outsider_id]).await
    else {
        return;
    };

    let create_request = Request::builder()
        .method("POST")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages",
            fixture.server_id, fixture.channel_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.outsider_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "outsider create" }).to_string(),
        ))
        .expect("build outsider create request");
    let edit_request = Request::builder()
        .method("PATCH")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.outsider_id]),
        )
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({ "content": "outsider edit" }).to_string(),
        ))
        .expect("build outsider edit request");
    let delete_request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/v1/servers/{}/channels/{}/messages/{}",
            fixture.server_id, fixture.channel_id, fixture.first_message_id
        ))
        .header(
            "authorization",
            format!("Bearer {}", tokens[&fixture.outsider_id]),
        )
        .body(Body::empty())
        .expect("build outsider delete request");

    let create_response = app
        .clone()
        .oneshot(create_request)
        .await
        .expect("outsider create response");
    let edit_response = app
        .clone()
        .oneshot(edit_request)
        .await
        .expect("outsider edit response");
    let delete_response = app
        .oneshot(delete_request)
        .await
        .expect("outsider delete response");

    for response in [create_response, edit_response, delete_response] {
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read outsider response body");
        let payload: serde_json::Value =
            serde_json::from_slice(&body).expect("decode outsider response body");
        assert_eq!(payload["code"], "server_access_denied");
    }
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

async fn start_api_http_server(state: AppState) -> String {
    let app = build_app(state);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind API listener");
    let address = listener.local_addr().expect("read API listener address");
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve API app");
    });
    format!("http://{address}")
}

async fn connect_ws_with_token_and_device(
    ws_url: &str,
    token: &str,
    device_id: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("authorization header"),
    );
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static("http://localhost:3002"));
    request.headers_mut().insert(
        "x-hexrelay-device-id",
        HeaderValue::from_str(device_id).expect("device header"),
    );

    let (socket, _) = connect_async(request)
        .await
        .expect("connect websocket with device");
    socket
}

async fn connect_ws_with_token(
    ws_url: &str,
    token: &str,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>> {
    let mut request = ws_url
        .into_client_request()
        .expect("build websocket request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("authorization header"),
    );
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static("http://localhost:3002"));

    let (socket, _) = connect_async(request).await.expect("connect websocket");
    socket
}

async fn recv_channel_event(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected_event_type: &str,
    expected_message_id: &str,
) -> serde_json::Value {
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        loop {
            let message = socket
                .next()
                .await
                .expect("socket message")
                .expect("websocket frame");
            let text = match message {
                WsMessage::Text(value) => value,
                _ => continue,
            };
            let payload: serde_json::Value =
                serde_json::from_str(&text).expect("decode websocket payload");
            if payload["event_type"] == expected_event_type
                && payload["data"]["message_id"] == expected_message_id
            {
                break payload;
            }
        }
    })
    .await
    .expect("channel event timeout")
}

async fn assert_no_channel_event(
    socket: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    expected_event_type: &str,
    expected_message_id: &str,
    timeout: std::time::Duration,
) {
    let wait_result = tokio::time::timeout(timeout, async {
        while let Some(message) = socket.next().await {
            let message = match message {
                Ok(value) => value,
                Err(_) => return,
            };
            let text = match message {
                WsMessage::Text(value) => value,
                _ => continue,
            };
            let payload: serde_json::Value = match serde_json::from_str(&text) {
                Ok(value) => value,
                Err(_) => continue,
            };
            if payload["event_type"] == expected_event_type
                && payload["data"]["message_id"] == expected_message_id
            {
                panic!(
                    "unexpected channel event for event_type={expected_event_type} message_id={expected_message_id}: {text}"
                );
            }
        }
    })
    .await;

    if let Ok(()) = wait_result {
        panic!("socket closed before channel absence assertion completed");
    }
}

#[tokio::test]
async fn api_server_channel_mutations_fan_out_over_realtime_websocket() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };
    let Some(redis_client) = prepared_presence_redis_client().await else {
        return;
    };

    let server_id = format!("srv-fanout-{}", Uuid::new_v4().simple());
    let channel_id = format!("chn-fanout-{}", Uuid::new_v4().simple());
    let member_id = unique_identity("usr-fanout-member");
    let teammate_id = unique_identity("usr-fanout-teammate");
    let outsider_id = unique_identity("usr-fanout-outsider");

    seed_server_channel(
        &pool,
        &server_id,
        "Fanout",
        &channel_id,
        "general",
        &[&member_id, &teammate_id],
        &[],
    )
    .await;
    ensure_db_identity_key(&pool, &outsider_id).await;

    let realtime_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind realtime listener");
    let realtime_address = realtime_listener
        .local_addr()
        .expect("read realtime listener address");
    let realtime_base_url = format!("http://{realtime_address}");

    let mut api_state = AppState::default().with_db_pool(pool.clone());
    api_state.presence_redis_client = Some(redis_client.clone());
    api_state.realtime_base_url = realtime_base_url;

    let member_token = issue_db_session_cookie(&pool, &api_state, &member_id).await;
    let teammate_token = issue_db_session_cookie(&pool, &api_state, &teammate_id).await;
    let outsider_token = issue_db_session_cookie(&pool, &api_state, &outsider_id).await;

    let api_base_url = start_api_http_server(api_state.clone()).await;

    let realtime_state = RealtimeAppState::new(
        api_base_url.clone(),
        vec!["http://localhost:3002".to_string()],
        api_state.channel_dispatch_internal_token.clone(),
        api_state.presence_watcher_internal_token.clone(),
        Some(redis_client.clone()),
        false,
        60,
        60,
        16384,
        120,
        60,
        3,
        0,
        10000,
    )
    .expect("build realtime state");
    spawn_presence_subscriber(realtime_state.clone());
    spawn_channel_subscriber(realtime_state.clone());
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let realtime_app = build_realtime_app(realtime_state);
    tokio::spawn(async move {
        axum::serve(
            realtime_listener,
            realtime_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve realtime app");
    });

    let ws_url = format!("ws://{realtime_address}/ws");
    let mut member_socket =
        connect_ws_with_token_and_device(&ws_url, &member_token, "device-primary").await;
    let mut teammate_socket = connect_ws_with_token(&ws_url, &teammate_token).await;
    let mut outsider_socket = connect_ws_with_token(&ws_url, &outsider_token).await;
    let _ = member_socket.next().await;
    let _ = teammate_socket.next().await;
    let _ = outsider_socket.next().await;

    let client = reqwest::Client::new();
    let create_response = client
        .post(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages"
        ))
        .bearer_auth(&member_token)
        .json(&serde_json::json!({
            "content": "hello realtime",
            "mention_identity_ids": []
        }))
        .send()
        .await
        .expect("send create request");
    assert_eq!(create_response.status(), reqwest::StatusCode::CREATED);
    let created_message: serde_json::Value = create_response.json().await.expect("decode create");
    let message_id = created_message["message_id"]
        .as_str()
        .expect("created message id")
        .to_string();

    let member_created =
        recv_channel_event(&mut member_socket, "channel.message.created", &message_id).await;
    let teammate_created =
        recv_channel_event(&mut teammate_socket, "channel.message.created", &message_id).await;
    assert_eq!(member_created["data"]["channel_id"], channel_id);
    assert_eq!(teammate_created["data"]["channel_seq"], 1);
    assert_no_channel_event(
        &mut outsider_socket,
        "channel.message.created",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;

    let edit_response = client
        .patch(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        ))
        .bearer_auth(&member_token)
        .json(&serde_json::json!({
            "content": "hello realtime edited",
            "mention_identity_ids": []
        }))
        .send()
        .await
        .expect("send edit request");
    assert_eq!(edit_response.status(), reqwest::StatusCode::OK);

    let member_updated =
        recv_channel_event(&mut member_socket, "channel.message.updated", &message_id).await;
    let teammate_updated =
        recv_channel_event(&mut teammate_socket, "channel.message.updated", &message_id).await;
    assert_eq!(member_updated["data"]["channel_seq"], 1);
    assert_eq!(teammate_updated["data"]["channel_id"], channel_id);
    assert_no_channel_event(
        &mut outsider_socket,
        "channel.message.updated",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;

    let no_op_edit_response = client
        .patch(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        ))
        .bearer_auth(&member_token)
        .json(&serde_json::json!({
            "content": "hello realtime edited",
            "mention_identity_ids": []
        }))
        .send()
        .await
        .expect("send no-op edit request");
    assert_eq!(no_op_edit_response.status(), reqwest::StatusCode::OK);
    assert_no_channel_event(
        &mut member_socket,
        "channel.message.updated",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;
    assert_no_channel_event(
        &mut teammate_socket,
        "channel.message.updated",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;

    let delete_response = client
        .delete(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        ))
        .bearer_auth(&member_token)
        .send()
        .await
        .expect("send delete request");
    assert_eq!(delete_response.status(), reqwest::StatusCode::OK);

    let member_deleted =
        recv_channel_event(&mut member_socket, "channel.message.deleted", &message_id).await;
    let teammate_deleted =
        recv_channel_event(&mut teammate_socket, "channel.message.deleted", &message_id).await;
    assert_eq!(member_deleted["data"]["channel_seq"], 1);
    assert_eq!(teammate_deleted["data"]["channel_id"], channel_id);
    assert_no_channel_event(
        &mut outsider_socket,
        "channel.message.deleted",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;

    let repeated_delete_response = client
        .delete(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages/{message_id}"
        ))
        .bearer_auth(&member_token)
        .send()
        .await
        .expect("send repeated delete request");
    assert_eq!(repeated_delete_response.status(), reqwest::StatusCode::OK);
    assert_no_channel_event(
        &mut member_socket,
        "channel.message.deleted",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;
    assert_no_channel_event(
        &mut teammate_socket,
        "channel.message.deleted",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;

    let mut late_device =
        connect_ws_with_token_and_device(&ws_url, &member_token, "device-late").await;
    let _ = late_device.next().await;
    let late_created =
        recv_channel_event(&mut late_device, "channel.message.created", &message_id).await;
    let late_updated =
        recv_channel_event(&mut late_device, "channel.message.updated", &message_id).await;
    let late_deleted =
        recv_channel_event(&mut late_device, "channel.message.deleted", &message_id).await;
    assert_eq!(late_created["data"]["channel_id"], channel_id);
    assert_eq!(late_updated["data"]["channel_id"], channel_id);
    assert_eq!(late_deleted["data"]["channel_id"], channel_id);

    late_device.close(None).await.expect("close late device");
    let mut late_device_reconnect =
        connect_ws_with_token_and_device(&ws_url, &member_token, "device-late").await;
    let _ = late_device_reconnect.next().await;
    assert_no_channel_event(
        &mut late_device_reconnect,
        "channel.message.created",
        &message_id,
        std::time::Duration::from_millis(500),
    )
    .await;
}

#[tokio::test]
async fn server_channel_create_succeeds_when_realtime_dispatch_is_unreachable() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let server_id = format!("srv-dispatch-down-{}", Uuid::new_v4().simple());
    let channel_id = format!("chn-dispatch-down-{}", Uuid::new_v4().simple());
    let member_id = unique_identity("usr-dispatch-down-member");

    seed_server_channel(
        &pool,
        &server_id,
        "Dispatch Down",
        &channel_id,
        "general",
        &[&member_id],
        &[],
    )
    .await;

    let Some((app, tokens, _)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    let mut state = AppState::default().with_db_pool(pool.clone());
    state.realtime_base_url = "http://127.0.0.1:1".to_string();
    let api_base_url = start_api_http_server(state).await;

    let create_response = reqwest::Client::new()
        .post(format!(
            "{api_base_url}/v1/servers/{server_id}/channels/{channel_id}/messages"
        ))
        .bearer_auth(&tokens[&member_id])
        .json(&serde_json::json!({
            "content": "persist despite realtime outage",
            "mention_identity_ids": []
        }))
        .send()
        .await
        .expect("send create request");

    assert_eq!(create_response.status(), reqwest::StatusCode::CREATED);
    let created_message: serde_json::Value = create_response.json().await.expect("decode create");
    let created_message_id = created_message["message_id"]
        .as_str()
        .expect("created message id")
        .to_string();

    let list_request = Request::builder()
        .method("GET")
        .uri(format!(
            "/v1/servers/{server_id}/channels/{channel_id}/messages?limit=10"
        ))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build list request");

    let list_response = app.oneshot(list_request).await.expect("list response");
    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = to_bytes(list_response.into_body(), usize::MAX)
        .await
        .expect("read list body");
    let page: serde_json::Value = serde_json::from_slice(&list_body).expect("decode page");
    let items = page["items"].as_array().expect("items array");
    assert!(items
        .iter()
        .any(|item| item["message_id"] == created_message_id));
}
