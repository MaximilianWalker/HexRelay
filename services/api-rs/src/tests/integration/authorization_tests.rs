use super::*;

#[tokio::test]
async fn gets_server_detail_for_authenticated_member_only() {
    let member_id = unique_identity("usr-auth-member");
    let outsider_id = unique_identity("usr-auth-outsider");
    let server_id = format!("srv-authz-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, &server_id, "Authz", &member_id, true, false, 3).await;

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&outsider_id]))
        .body(Body::empty())
        .expect("build outsider request");

    let member_response = app
        .clone()
        .oneshot(member_request)
        .await
        .expect("member response");
    let outsider_response = app
        .clone()
        .oneshot(outsider_request)
        .await
        .expect("outsider response");

    assert_eq!(member_response.status(), StatusCode::OK);
    assert_eq!(outsider_response.status(), StatusCode::FORBIDDEN);

    let member_body = to_bytes(member_response.into_body(), usize::MAX)
        .await
        .expect("read member response body");
    let member_payload: serde_json::Value =
        serde_json::from_slice(&member_body).expect("decode member response");
    assert_eq!(member_payload["item"]["id"], server_id);
    assert_eq!(member_payload["item"]["favorite"], true);

    let outsider_body = to_bytes(outsider_response.into_body(), usize::MAX)
        .await
        .expect("read outsider response body");
    let outsider_payload: serde_json::Value =
        serde_json::from_slice(&outsider_body).expect("decode outsider response");
    assert_eq!(outsider_payload["code"], "server_access_denied");
}

#[tokio::test]
async fn lists_server_channels_for_authenticated_member_only() {
    let member_id = unique_identity("usr-channel-auth-member");
    let outsider_id = unique_identity("usr-channel-auth-outsider");
    let server_id = format!("srv-channel-authz-{}", uuid::Uuid::new_v4().simple());
    let channel_a = format!("chn-authz-a-{}", uuid::Uuid::new_v4().simple());
    let channel_b = format!("chn-authz-b-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        &server_id,
        "Authz Channels",
        &member_id,
        false,
        false,
        0,
    )
    .await;
    server_channels_repo::insert_server_channel(
        &pool,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: &channel_a,
            server_id: &server_id,
            name: "general",
            kind: "text",
        },
    )
    .await
    .expect("insert first channel");
    server_channels_repo::insert_server_channel(
        &pool,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: &channel_b,
            server_id: &server_id,
            name: "random",
            kind: "text",
        },
    )
    .await
    .expect("insert second channel");

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&outsider_id]))
        .body(Body::empty())
        .expect("build outsider request");

    let member_response = app
        .clone()
        .oneshot(member_request)
        .await
        .expect("member response");
    let outsider_response = app
        .clone()
        .oneshot(outsider_request)
        .await
        .expect("outsider response");

    assert_eq!(member_response.status(), StatusCode::OK);
    assert_eq!(outsider_response.status(), StatusCode::FORBIDDEN);

    let member_body = to_bytes(member_response.into_body(), usize::MAX)
        .await
        .expect("read member response body");
    let member_payload: serde_json::Value =
        serde_json::from_slice(&member_body).expect("decode member response");
    let ids = member_payload["items"]
        .as_array()
        .expect("channels array")
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert!(ids.contains(channel_a.as_str()));
    assert!(ids.contains(channel_b.as_str()));

    let outsider_body = to_bytes(outsider_response.into_body(), usize::MAX)
        .await
        .expect("read outsider response body");
    let outsider_payload: serde_json::Value =
        serde_json::from_slice(&outsider_body).expect("decode outsider response");
    assert_eq!(outsider_payload["code"], "server_access_denied");
}

#[tokio::test]
async fn lists_empty_server_channel_collection_for_member() {
    let member_id = unique_identity("usr-empty-channel-member");
    let server_id = format!("srv-empty-channels-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(
        &pool,
        &server_id,
        "Empty Channels",
        &member_id,
        false,
        false,
        0,
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["items"], serde_json::json!([]));
}

#[tokio::test]
async fn gets_server_detail_for_cookie_authenticated_member_only() {
    let member_id = unique_identity("usr-cookie-member");
    let outsider_id = unique_identity("usr-cookie-outsider");
    let server_id = format!("srv-cookie-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        &server_id,
        "Cookie Auth",
        &member_id,
        false,
        false,
        2,
    )
    .await;

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .header("cookie", format!("hexrelay_session={}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member cookie request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens[&outsider_id]),
        )
        .body(Body::empty())
        .expect("build outsider cookie request");

    let member_response = app
        .clone()
        .oneshot(member_request)
        .await
        .expect("member cookie response");
    let outsider_response = app
        .clone()
        .oneshot(outsider_request)
        .await
        .expect("outsider cookie response");

    assert_eq!(member_response.status(), StatusCode::OK);
    assert_eq!(outsider_response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn forbids_server_detail_bypass_via_path_switch() {
    let member_id = unique_identity("usr-bypass-member");
    let outsider_id = unique_identity("usr-bypass-outsider");
    let member_server_id = format!("srv-member-only-{}", uuid::Uuid::new_v4().simple());
    let outsider_server_id = format!("srv-outsider-only-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        &member_server_id,
        "Member Only",
        &member_id,
        false,
        false,
        1,
    )
    .await;
    seed_server_membership(
        &pool,
        &outsider_server_id,
        "Outsider Only",
        &outsider_id,
        false,
        false,
        0,
    )
    .await;

    let bypass_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{member_server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&outsider_id]))
        .body(Body::empty())
        .expect("build bypass request");

    let response = app.oneshot(bypass_request).await.expect("bypass response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read bypass response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["code"], "server_access_denied");
}

#[tokio::test]
async fn server_detail_authorization_survives_auth_reuse_in_handler() {
    let member_id = unique_identity("usr-authz-reuse");
    let server_id = format!("srv-authz-reuse-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, &server_id, "Authz Reuse", &member_id, false, true, 4).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["item"]["id"], server_id);
    assert_eq!(payload["item"]["muted"], true);
}

#[tokio::test]
async fn lists_servers_with_same_identity_scope_for_cookie_and_bearer_auth() {
    let member_id = unique_identity("usr-list-scope");
    let server_a = format!("srv-scope-a-{}", uuid::Uuid::new_v4().simple());
    let server_b = format!("srv-scope-b-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, &server_a, "Scope A", &member_id, true, false, 1).await;
    seed_server_membership(&pool, &server_b, "Scope B", &member_id, false, true, 0).await;

    let bearer_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build bearer request");
    let cookie_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header("cookie", format!("hexrelay_session={}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build cookie request");

    let bearer_response = app
        .clone()
        .oneshot(bearer_request)
        .await
        .expect("bearer response");
    let cookie_response = app
        .clone()
        .oneshot(cookie_request)
        .await
        .expect("cookie response");

    assert_eq!(bearer_response.status(), StatusCode::OK);
    assert_eq!(cookie_response.status(), StatusCode::OK);

    let bearer_body = to_bytes(bearer_response.into_body(), usize::MAX)
        .await
        .expect("read bearer body");
    let cookie_body = to_bytes(cookie_response.into_body(), usize::MAX)
        .await
        .expect("read cookie body");
    let bearer_payload: serde_json::Value =
        serde_json::from_slice(&bearer_body).expect("decode bearer payload");
    let cookie_payload: serde_json::Value =
        serde_json::from_slice(&cookie_body).expect("decode cookie payload");

    assert_eq!(bearer_payload, cookie_payload);
    let ids = bearer_payload["items"]
        .as_array()
        .expect("server items array")
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(server_a.as_str()));
    assert!(ids.contains(server_b.as_str()));
}

#[tokio::test]
async fn rejects_server_list_without_authentication() {
    let member_id = unique_identity("usr-list-unauth");
    let server_id = format!("srv-list-authz-{}", uuid::Uuid::new_v4().simple());
    let Some((app, _, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, &server_id, "List Authz", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .body(Body::empty())
        .expect("build unauthenticated list request");

    let response = app.oneshot(request).await.expect("list response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_server_detail_without_authentication() {
    let member_id = unique_identity("usr-detail-unauth");
    let server_id = format!("srv-authz-{}", uuid::Uuid::new_v4().simple());
    let Some((app, _, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, &server_id, "Authz", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/v1/servers/{server_id}"))
        .body(Body::empty())
        .expect("build unauthenticated request");

    let response = app
        .oneshot(request)
        .await
        .expect("unauthenticated response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read unauthenticated response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["code"], "session_invalid");
}
