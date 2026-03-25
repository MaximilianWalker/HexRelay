use super::*;

#[tokio::test]
async fn gets_server_detail_for_authenticated_member_only() {
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&["usr-member", "usr-outsider"]).await
    else {
        return;
    };

    seed_server_membership(&pool, "srv-authz", "Authz", "usr-member", true, false, 3).await;

    let member_request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-authz")
        .header("authorization", format!("Bearer {}", tokens["usr-member"]))
        .body(Body::empty())
        .expect("build member request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-authz")
        .header(
            "authorization",
            format!("Bearer {}", tokens["usr-outsider"]),
        )
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
    assert_eq!(member_payload["item"]["id"], "srv-authz");
    assert_eq!(member_payload["item"]["favorite"], true);

    let outsider_body = to_bytes(outsider_response.into_body(), usize::MAX)
        .await
        .expect("read outsider response body");
    let outsider_payload: serde_json::Value =
        serde_json::from_slice(&outsider_body).expect("decode outsider response");
    assert_eq!(outsider_payload["code"], "server_access_denied");
}

#[tokio::test]
async fn gets_server_detail_for_cookie_authenticated_member_only() {
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&["usr-member", "usr-outsider"]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-cookie",
        "Cookie Auth",
        "usr-member",
        false,
        false,
        2,
    )
    .await;

    let member_request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-cookie")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-member"]),
        )
        .body(Body::empty())
        .expect("build member cookie request");
    let outsider_request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-cookie")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-outsider"]),
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
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&["usr-member", "usr-outsider"]).await
    else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-member-only",
        "Member Only",
        "usr-member",
        false,
        false,
        1,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-outsider-only",
        "Outsider Only",
        "usr-outsider",
        false,
        false,
        0,
    )
    .await;

    let bypass_request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-member-only")
        .header(
            "authorization",
            format!("Bearer {}", tokens["usr-outsider"]),
        )
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
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-member"]).await else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-authz-reuse",
        "Authz Reuse",
        "usr-member",
        false,
        true,
        4,
    )
    .await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-authz-reuse")
        .header("authorization", format!("Bearer {}", tokens["usr-member"]))
        .body(Body::empty())
        .expect("build request");

    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read response body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode response");
    assert_eq!(payload["item"]["id"], "srv-authz-reuse");
    assert_eq!(payload["item"]["muted"], true);
}

#[tokio::test]
async fn lists_servers_with_same_identity_scope_for_cookie_and_bearer_auth() {
    let Some((app, tokens, pool)) = app_with_database_and_sessions(&["usr-member"]).await else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-scope-a",
        "Scope A",
        "usr-member",
        true,
        false,
        1,
    )
    .await;
    seed_server_membership(
        &pool,
        "srv-scope-b",
        "Scope B",
        "usr-member",
        false,
        true,
        0,
    )
    .await;

    let bearer_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header("authorization", format!("Bearer {}", tokens["usr-member"]))
        .body(Body::empty())
        .expect("build bearer request");
    let cookie_request = Request::builder()
        .method("GET")
        .uri("/v1/servers")
        .header(
            "cookie",
            format!("hexrelay_session={}", tokens["usr-member"]),
        )
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
    assert_eq!(bearer_payload["items"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn rejects_server_list_without_authentication() {
    let Some((app, _, pool)) = app_with_database_and_sessions(&["usr-member"]).await else {
        return;
    };

    seed_server_membership(
        &pool,
        "srv-list-authz",
        "List Authz",
        "usr-member",
        false,
        false,
        0,
    )
    .await;

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
    let Some((app, _, pool)) = app_with_database_and_sessions(&["usr-member"]).await else {
        return;
    };

    seed_server_membership(&pool, "srv-authz", "Authz", "usr-member", false, false, 0).await;

    let request = Request::builder()
        .method("GET")
        .uri("/v1/servers/srv-authz")
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
