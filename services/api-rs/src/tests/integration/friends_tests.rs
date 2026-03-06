use super::*;

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
