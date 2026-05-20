use super::*;

#[tokio::test]
async fn gets_server_detail_for_authenticated_member_only() {
    let member_id = unique_identity("usr-auth-member");
    let outsider_id = unique_identity("usr-auth-outsider");
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, "Authz", &member_id, true, false, 3).await;

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
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
    let server_id = TEST_SERVER_ID.to_string();
    let channel_a = format!("chn-authz-a-{}", uuid::Uuid::new_v4().simple());
    let channel_b = format!("chn-authz-b-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, "Authz Channels", &member_id, false, false, 0).await;
    server_channels_repo::insert_server_channel(
        &pool,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: &channel_a,
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
            name: "random",
            kind: "text",
        },
    )
    .await
    .expect("insert second channel");

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}/channels"))
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
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Empty Channels", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}/channels"))
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
async fn channel_listing_honors_configured_role_read_permissions() {
    let member_id = unique_identity("usr-channel-role-reader");
    let server_id = TEST_SERVER_ID.to_string();
    let readable_channel_id = format!("chn-readable-{}", uuid::Uuid::new_v4().simple());
    let hidden_channel_id = format!("chn-hidden-{}", uuid::Uuid::new_v4().simple());
    let role_id = format!("role-reader-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Role Filter", &member_id, false, false, 0).await;
    for (channel_id, name) in [
        (&readable_channel_id, "visible"),
        (&hidden_channel_id, "hidden"),
    ] {
        server_channels_repo::insert_server_channel(
            &pool,
            server_channels_repo::ServerChannelInsertParams {
                channel_id,
                name,
                kind: "text",
            },
        )
        .await
        .expect("insert channel");
    }
    server_channels_repo::insert_server_role(
        &pool,
        server_channels_repo::ServerRoleInsertParams {
            role_id: &role_id,
            name: "reader",
            rank: 1,
        },
    )
    .await
    .expect("insert role");
    server_channels_repo::assign_server_membership_role(
        &pool,
        server_channels_repo::ServerMembershipRoleInsertParams {
            identity_id: &member_id,
            role_id: &role_id,
        },
    )
    .await
    .expect("assign role");
    server_channels_repo::upsert_server_channel_role_permissions(
        &pool,
        server_channels_repo::ServerChannelRolePermissionParams {
            channel_id: &readable_channel_id,
            role_id: &role_id,
            can_read: true,
            can_send: false,
            can_manage: false,
        },
    )
    .await
    .expect("insert readable channel permission");
    server_channels_repo::upsert_server_channel_role_permissions(
        &pool,
        server_channels_repo::ServerChannelRolePermissionParams {
            channel_id: &hidden_channel_id,
            role_id: &role_id,
            can_read: false,
            can_send: false,
            can_manage: false,
        },
    )
    .await
    .expect("insert hidden channel permission");

    let request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}/channels"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build channel list request");

    let response = app.oneshot(request).await.expect("channel list response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read channel list body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    let ids = payload["items"]
        .as_array()
        .expect("channels array")
        .iter()
        .filter_map(|item| item["id"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(readable_channel_id.as_str()));
    assert!(!ids.contains(hidden_channel_id.as_str()));
}

#[tokio::test]
async fn gets_server_detail_for_cookie_authenticated_member_only() {
    let member_id = unique_identity("usr-cookie-member");
    let outsider_id = unique_identity("usr-cookie-outsider");
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, "Cookie Auth", &member_id, false, false, 2).await;

    let member_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
        .header("cookie", format!("hexrelay_session={}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build member cookie request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
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
    let member_server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[&member_id, &outsider_id]).await
    else {
        return;
    };

    seed_server_membership(&pool, "Member Only", &member_id, false, false, 1).await;

    let bypass_request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{member_server_id}"))
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
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Authz Reuse", &member_id, false, true, 4).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
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
    let server_a = TEST_SERVER_ID.to_string();
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Scope A", &member_id, true, false, 1).await;

    let bearer_request = Request::builder()
        .method("GET")
        .uri("/servers")
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build bearer request");
    let cookie_request = Request::builder()
        .method("GET")
        .uri("/servers")
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
    assert_eq!(ids.len(), 1);
    assert!(ids.contains(server_a.as_str()));
}

#[tokio::test]
async fn forbids_member_access_to_non_local_server_id() {
    let member_id = unique_identity("usr-non-local-member");
    let non_local_server_id = format!("srv-non-local-{}", uuid::Uuid::new_v4().simple());
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Non Local", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{non_local_server_id}"))
        .header("authorization", format!("Bearer {}", tokens[&member_id]))
        .body(Body::empty())
        .expect("build non-local server request");

    let response = app.oneshot(request).await.expect("non-local response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read non-local body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode non-local body");
    assert_eq!(payload["code"], "server_access_denied");
}

#[tokio::test]
async fn rejects_server_list_without_authentication() {
    let member_id = unique_identity("usr-list-unauth");
    let Some((app, _, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "List Authz", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri("/servers")
        .body(Body::empty())
        .expect("build unauthenticated list request");

    let response = app.oneshot(request).await.expect("list response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rejects_server_detail_without_authentication() {
    let member_id = unique_identity("usr-detail-unauth");
    let server_id = TEST_SERVER_ID.to_string();
    let Some((app, _, pool)) = app_with_database_and_sessions(&[&member_id]).await else {
        return;
    };

    seed_server_membership(&pool, "Authz", &member_id, false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri(format!("/servers/{server_id}"))
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
