use super::*;

#[tokio::test]
async fn returns_default_dm_policy_for_new_identity() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("GET")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .body(Body::empty())
        .expect("build dm policy get request");

    let response = app.oneshot(request).await.expect("dm policy get response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read dm policy get body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode dm policy get response");

    assert_eq!(payload["inbound_policy"], "friends_only");
    assert_eq!(payload["offline_delivery_mode"], "best_effort_online");
}

#[tokio::test]
async fn updates_dm_policy_and_persists_for_identity() {
    let app = build_app(AppState::default());
    let (session_cookie, app) = authenticate_identity(app, "usr-dm-policy").await;

    let update_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");

    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("dm policy update response");
    assert_eq!(update_response.status(), StatusCode::OK);

    let read_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/privacy-policy")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build dm policy read request");

    let read_response = app
        .oneshot(read_request)
        .await
        .expect("dm policy read response");
    assert_eq!(read_response.status(), StatusCode::OK);

    let read_body = to_bytes(read_response.into_body(), usize::MAX)
        .await
        .expect("read dm policy response body");
    let read_payload: serde_json::Value =
        serde_json::from_slice(&read_body).expect("decode dm policy response");
    assert_eq!(read_payload["inbound_policy"], "anyone");
}

#[tokio::test]
async fn rejects_invalid_dm_policy_update() {
    let app = build_app(AppState::default());
    let (session_cookie, app) = authenticate_identity(app, "usr-dm-invalid").await;

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"invalid"}"#))
        .expect("build invalid dm policy update request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid dm policy update response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid dm policy body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid dm policy body");
    assert_eq!(payload["code"], "dm_policy_invalid");
}

#[tokio::test]
async fn persists_dm_policy_across_db_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let identity_id = unique_identity("db-dm-policy");
    let (session_cookie, app) = authenticate_identity(app, &identity_id).await;

    let update_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");

    let update_response = app
        .clone()
        .oneshot(update_request)
        .await
        .expect("dm policy update response");
    assert_eq!(update_response.status(), StatusCode::OK);

    let Some(restarted_app) = app_with_database().await else {
        return;
    };
    let read_request = Request::builder()
        .method("GET")
        .uri("/v1/dm/privacy-policy")
        .header("cookie", format!("hexrelay_session={session_cookie}"))
        .body(Body::empty())
        .expect("build dm policy read request");

    let read_response = restarted_app
        .oneshot(read_request)
        .await
        .expect("dm policy read response");
    assert_eq!(read_response.status(), StatusCode::OK);

    let read_body = to_bytes(read_response.into_body(), usize::MAX)
        .await
        .expect("read dm policy response body");
    let read_payload: serde_json::Value =
        serde_json::from_slice(&read_body).expect("decode dm policy response");
    assert_eq!(read_payload["inbound_policy"], "anyone");
    assert_eq!(read_payload["offline_delivery_mode"], "best_effort_online");
}

#[tokio::test]
async fn endpoint_cards_and_profile_devices_persist_across_db_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let identity_id = unique_identity("db-dm-device-state");
    let (session_cookie, app) = authenticate_identity(app, &identity_id).await;

    let register_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/endpoint-cards")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(
            r#"{"cards":[{"endpoint_id":"lan-a","endpoint_hint":"udp://192.168.1.20:4040","estimated_rtt_ms":9,"priority":2}]}"#,
        ))
        .expect("build endpoint-card register request");

    let register_response = app
        .clone()
        .oneshot(register_request)
        .await
        .expect("endpoint-card register response");
    assert_eq!(register_response.status(), StatusCode::OK);

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");

    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy update response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let heartbeat_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"desktop-main","active":true}"#))
        .expect("build device heartbeat request");

    let heartbeat_response = app
        .oneshot(heartbeat_request)
        .await
        .expect("device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let Some(restarted_app) = app_with_database().await else {
        return;
    };

    let dial_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/connectivity/parallel-dial")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"peer_identity_id":"{}"}}"#,
            identity_id
        )))
        .expect("build restart parallel dial request");

    let dial_response = restarted_app
        .clone()
        .oneshot(dial_request)
        .await
        .expect("parallel dial response after restart");
    assert_eq!(dial_response.status(), StatusCode::OK);
    let dial_body = to_bytes(dial_response.into_body(), usize::MAX)
        .await
        .expect("read parallel dial response body");
    let dial_payload: serde_json::Value =
        serde_json::from_slice(&dial_body).expect("decode parallel dial response");
    assert_eq!(dial_payload["status"], "ready");
    assert_eq!(dial_payload["winner_endpoint_id"], "lan-a");

    let catch_up_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header(
            "cookie",
            format!("hexrelay_session={session_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main"}"#))
        .expect("build restart catch-up request");

    let catch_up_response = restarted_app
        .oneshot(catch_up_request)
        .await
        .expect("catch-up response after restart");
    assert_eq!(catch_up_response.status(), StatusCode::OK);
    let catch_up_body = to_bytes(catch_up_response.into_body(), usize::MAX)
        .await
        .expect("read catch-up response body");
    let catch_up_payload: serde_json::Value =
        serde_json::from_slice(&catch_up_body).expect("decode catch-up response");
    assert_eq!(catch_up_payload["status"], "ready");
    assert_eq!(catch_up_payload["reason_code"], "fanout_catch_up_no_missed");
}

#[tokio::test]
async fn fanout_cursor_metadata_persists_across_db_restart() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let sender_identity = unique_identity("db-fanout-sender");
    let recipient_identity = unique_identity("db-fanout-recipient");
    let (sender_cookie, app) = authenticate_identity(app, &sender_identity).await;
    let (recipient_cookie, app) = authenticate_identity(app, &recipient_identity).await;

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat_request = Request::builder()
            .method("POST")
            .uri("/v1/dm/profile-devices/heartbeat")
            .header("content-type", "application/json")
            .header(
                "cookie",
                format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
            )
            .header("x-csrf-token", "test-csrf")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","active":{active}}}"#
            )))
            .expect("build heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat_request)
            .await
            .expect("heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    let dispatch_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={sender_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-restart","ciphertext":"enc:restart"}}"#,
            recipient_identity
        )))
        .expect("build fanout dispatch request");
    let dispatch_response = app
        .clone()
        .oneshot(dispatch_request)
        .await
        .expect("fanout dispatch response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);

    let Some(restarted_app) = app_with_database().await else {
        return;
    };

    let activate_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"phone-main","active":true}"#))
        .expect("build activation request");
    let activate_response = restarted_app
        .clone()
        .oneshot(activate_request)
        .await
        .expect("activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let thread_id = crate::infra::db::repos::dm_history_repo::direct_dm_thread_id(
        &sender_identity,
        &recipient_identity,
    );

    let messages_request = Request::builder()
        .method("GET")
        .uri(format!("/v1/dm/threads/{thread_id}/messages?limit=10"))
        .header("cookie", format!("hexrelay_session={recipient_cookie}"))
        .body(Body::empty())
        .expect("build thread messages request after restart");
    let messages_response = restarted_app
        .clone()
        .oneshot(messages_request)
        .await
        .expect("thread messages response after restart");
    assert_eq!(messages_response.status(), StatusCode::OK);

    let messages_body = to_bytes(messages_response.into_body(), usize::MAX)
        .await
        .expect("read thread messages body after restart");
    let messages_payload: serde_json::Value = serde_json::from_slice(&messages_body)
        .expect("decode thread messages payload after restart");
    assert_eq!(messages_payload["items"][0]["message_id"], "msg-restart");

    let catch_up_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build catch-up request");
    let catch_up_response = restarted_app
        .clone()
        .oneshot(catch_up_request)
        .await
        .expect("catch-up response");
    assert_eq!(catch_up_response.status(), StatusCode::OK);

    let first_body = to_bytes(catch_up_response.into_body(), usize::MAX)
        .await
        .expect("read first catch-up body");
    let first_payload: serde_json::Value =
        serde_json::from_slice(&first_body).expect("decode first catch-up body");
    assert_eq!(first_payload["replay_count"], 1);
    assert_eq!(first_payload["next_cursor"], "1");

    let second_catch_up_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build second catch-up request");
    let second_catch_up_response = restarted_app
        .oneshot(second_catch_up_request)
        .await
        .expect("second catch-up response");
    assert_eq!(second_catch_up_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_catch_up_response.into_body(), usize::MAX)
        .await
        .expect("read second catch-up body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second catch-up body");
    assert_eq!(second_payload["replay_count"], 0);
    assert_eq!(second_payload["next_cursor"], "1");
    assert_eq!(second_payload["reason_code"], "fanout_catch_up_no_missed");
}

#[tokio::test]
async fn accepted_dm_without_active_devices_survives_restart_and_catches_up_later() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let sender_identity = unique_identity("db-pending-sender");
    let recipient_identity = unique_identity("db-pending-recipient");
    let (sender_cookie, app) = authenticate_identity(app, &sender_identity).await;
    let (recipient_cookie, app) = authenticate_identity(app, &recipient_identity).await;

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let dispatch_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={sender_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-pending","ciphertext":"enc:pending"}}"#,
            recipient_identity
        )))
        .expect("build pending fanout dispatch request");
    let dispatch_response = app
        .clone()
        .oneshot(dispatch_request)
        .await
        .expect("pending fanout dispatch response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);

    let dispatch_body = to_bytes(dispatch_response.into_body(), usize::MAX)
        .await
        .expect("read pending dispatch body");
    let dispatch_payload: serde_json::Value =
        serde_json::from_slice(&dispatch_body).expect("decode pending dispatch payload");
    assert_eq!(dispatch_payload["status"], "accepted");
    assert_eq!(dispatch_payload["delivery_state"], "pending_delivery");
    assert_eq!(dispatch_payload["reachability_state"], "unreachable");

    let Some(restarted_app) = app_with_database().await else {
        return;
    };

    let thread_id = crate::infra::db::repos::dm_history_repo::direct_dm_thread_id(
        &sender_identity,
        &recipient_identity,
    );

    let pool = prepared_database_pool()
        .await
        .expect("prepared DB pool after restart");
    let persisted_messages =
        crate::infra::db::repos::dm_history_repo::list_dm_thread_messages_for_identity(
            &pool,
            &recipient_identity,
            &thread_id,
            None,
            10,
        )
        .await
        .expect("load persisted dm messages after restart")
        .expect("recipient thread still exists after restart");
    assert_eq!(persisted_messages[0].message_id, "msg-pending");

    let activate_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"phone-main","active":true}"#))
        .expect("build activation request");
    let activate_response = restarted_app
        .clone()
        .oneshot(activate_request)
        .await
        .expect("activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build catch-up request");
    let catch_up_response = restarted_app
        .oneshot(catch_up_request)
        .await
        .expect("catch-up response");
    assert_eq!(catch_up_response.status(), StatusCode::OK);

    let catch_up_body = to_bytes(catch_up_response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let catch_up_payload: serde_json::Value =
        serde_json::from_slice(&catch_up_body).expect("decode catch-up payload");
    assert_eq!(catch_up_payload["replay_count"], 1);
    assert_eq!(catch_up_payload["items"][0]["message_id"], "msg-pending");
}

#[tokio::test]
async fn duplicate_dm_message_id_returns_conflict_and_preserves_original_ciphertext() {
    let Some(app) = app_with_database().await else {
        return;
    };

    let sender_identity = unique_identity("db-dup-sender");
    let recipient_identity = unique_identity("db-dup-recipient");
    let (sender_cookie, app) = authenticate_identity(app, &sender_identity).await;
    let (recipient_cookie, app) = authenticate_identity(app, &recipient_identity).await;

    let policy_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={recipient_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy request");
    let policy_response = app
        .clone()
        .oneshot(policy_request)
        .await
        .expect("dm policy response");
    assert_eq!(policy_response.status(), StatusCode::OK);

    let first_dispatch = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={sender_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-dup","ciphertext":"enc:first"}}"#,
            recipient_identity
        )))
        .expect("build first dispatch request");
    let first_response = app
        .clone()
        .oneshot(first_dispatch)
        .await
        .expect("first dispatch response");
    assert_eq!(first_response.status(), StatusCode::OK);

    let duplicate_dispatch = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("content-type", "application/json")
        .header(
            "cookie",
            format!("hexrelay_session={sender_cookie}; hexrelay_csrf=test-csrf"),
        )
        .header("x-csrf-token", "test-csrf")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"msg-dup","ciphertext":"enc:rewritten"}}"#,
            recipient_identity
        )))
        .expect("build duplicate dispatch request");
    let duplicate_response = app
        .clone()
        .oneshot(duplicate_dispatch)
        .await
        .expect("duplicate dispatch response");
    assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);

    let duplicate_body = to_bytes(duplicate_response.into_body(), usize::MAX)
        .await
        .expect("read duplicate response body");
    let duplicate_payload: serde_json::Value =
        serde_json::from_slice(&duplicate_body).expect("decode duplicate response body");
    assert_eq!(duplicate_payload["code"], "fanout_message_id_conflict");

    let pool = prepared_database_pool().await.expect("prepared DB pool");
    let thread_id = crate::infra::db::repos::dm_history_repo::direct_dm_thread_id(
        &sender_identity,
        &recipient_identity,
    );
    let persisted_messages =
        crate::infra::db::repos::dm_history_repo::list_dm_thread_messages_for_identity(
            &pool,
            &recipient_identity,
            &thread_id,
            None,
            10,
        )
        .await
        .expect("load persisted dm messages")
        .expect("recipient thread exists");
    assert_eq!(persisted_messages.len(), 1);
    assert_eq!(persisted_messages[0].message_id, "msg-dup");
    assert_eq!(persisted_messages[0].ciphertext, "enc:first");
}

#[tokio::test]
async fn rejects_invalid_profile_device_heartbeat() {
    let (app, tokens) = app_with_sessions(&["usr-device-invalid"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header(
            "authorization",
            format!("Bearer {}", tokens["usr-device-invalid"]),
        )
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"   ","active":true}"#))
        .expect("build invalid device heartbeat request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid device heartbeat response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid device heartbeat body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid device heartbeat body");
    assert_eq!(payload["code"], "profile_device_invalid");
}
