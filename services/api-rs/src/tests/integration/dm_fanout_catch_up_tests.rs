use super::*;

async fn set_dm_policy_anyone(app: axum::Router, token: &str) -> axum::Router {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/privacy-policy")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"inbound_policy":"anyone"}"#))
        .expect("build dm policy update request");
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("dm policy update response");
    assert_eq!(response.status(), StatusCode::OK);
    app
}

#[tokio::test]
async fn fanout_catch_up_replays_messages_for_late_activated_device() {
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&["usr-nora-k", "usr-jules-p"]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/v1/dm/profile-devices/heartbeat")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","active":{active}}}"#
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    let dispatch = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-2001","ciphertext":"enc:late-2001"}"#,
        ))
        .expect("build fanout dispatch request");
    let dispatch_response = app
        .clone()
        .oneshot(dispatch)
        .await
        .expect("fanout dispatch response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);

    let activate_phone = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from("{\"device_id\":\"phone-main\",\"active\":true}"))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build fanout catch-up request");
    let response = app
        .oneshot(catch_up)
        .await
        .expect("fanout catch-up response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode catch-up body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "fanout_catch_up_ok");
    assert_eq!(payload["replay_count"], 1);
    assert_eq!(payload["next_cursor"], "1");
    assert_eq!(payload["items"][0]["message_id"], "msg-2001");
}

#[tokio::test]
async fn fanout_catch_up_dedupes_identical_replay_entries_and_advances_cursor() {
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&["usr-nora-k", "usr-jules-p"]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/v1/dm/profile-devices/heartbeat")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","active":{active}}}"#
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    for _ in [0, 1] {
        let dispatch = Request::builder()
            .method("POST")
            .uri("/v1/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-dup","ciphertext":"enc:dup-same","source_device_id":"sender-main"}"#,
            ))
            .expect("build fanout dispatch request");
        let dispatch_response = app
            .clone()
            .oneshot(dispatch)
            .await
            .expect("fanout dispatch response");
        assert_eq!(dispatch_response.status(), StatusCode::OK);
    }

    let activate_phone = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from("{\"device_id\":\"phone-main\",\"active\":true}"))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build fanout catch-up request");
    let response = app
        .clone()
        .oneshot(catch_up)
        .await
        .expect("fanout catch-up response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode catch-up body");

    assert_eq!(payload["reason_code"], "fanout_catch_up_ok");
    assert_eq!(payload["replay_count"], 1);
    assert_eq!(payload["next_cursor"], "2");
    assert_eq!(payload["items"][0]["message_id"], "msg-dup");
    assert!(payload["deduped_message_ids"]
        .as_array()
        .expect("deduped ids array")
        .contains(&serde_json::Value::String("msg-dup".to_string())));

    let second_catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build second fanout catch-up request");
    let second_response = app
        .oneshot(second_catch_up)
        .await
        .expect("second fanout catch-up response");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second catch-up body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second catch-up body");

    assert_eq!(second_payload["reason_code"], "fanout_catch_up_no_missed");
    assert_eq!(second_payload["replay_count"], 0);
    assert_eq!(second_payload["next_cursor"], "2");
}

#[tokio::test]
async fn fanout_catch_up_keeps_distinct_payload_variants_with_same_message_id() {
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&["usr-nora-k", "usr-jules-p"]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/v1/dm/profile-devices/heartbeat")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","active":{active}}}"#
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    for (ciphertext, source_device_id) in [
        ("enc:variant-1", "sender-main"),
        ("enc:variant-2", "tablet-main"),
    ] {
        let dispatch = Request::builder()
            .method("POST")
            .uri("/v1/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"usr-jules-p","message_id":"msg-variant","ciphertext":"{ciphertext}","source_device_id":"{source_device_id}"}}"#
            )))
            .expect("build fanout dispatch request");
        let dispatch_response = app
            .clone()
            .oneshot(dispatch)
            .await
            .expect("fanout dispatch response");
        assert_eq!(dispatch_response.status(), StatusCode::OK);
    }

    let activate_phone = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from("{\"device_id\":\"phone-main\",\"active\":true}"))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"phone-main"}"#))
        .expect("build fanout catch-up request");
    let response = app
        .oneshot(catch_up)
        .await
        .expect("fanout catch-up response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode catch-up body");

    assert_eq!(payload["reason_code"], "fanout_catch_up_ok");
    assert_eq!(payload["replay_count"], 2);
    assert_eq!(payload["next_cursor"], "2");
    assert_eq!(
        payload["deduped_message_ids"]
            .as_array()
            .expect("deduped ids array")
            .len(),
        0
    );
}

#[tokio::test]
async fn fanout_catch_up_blocks_for_inactive_device() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let heartbeat = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main","active":false}"#))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main"}"#))
        .expect("build fanout catch-up request");
    let response = app
        .oneshot(catch_up)
        .await
        .expect("fanout catch-up response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode catch-up body");

    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "fanout_device_inactive");
}

#[tokio::test]
async fn fanout_catch_up_rejects_cursor_beyond_delivery_tail() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let heartbeat = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main","active":true}"#))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main","cursor":"99"}"#))
        .expect("build fanout catch-up request");
    let response = app
        .oneshot(catch_up)
        .await
        .expect("fanout catch-up response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read catch-up body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode catch-up body");
    assert_eq!(payload["code"], "cursor_out_of_range");
}

#[tokio::test]
async fn fanout_catch_up_rejects_invalid_payload() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"   "}"#))
        .expect("build invalid fanout catch-up request");

    let response = app
        .oneshot(request)
        .await
        .expect("invalid fanout catch-up response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read invalid fanout catch-up body");
    let payload: serde_json::Value =
        serde_json::from_slice(&body).expect("decode invalid fanout catch-up body");
    assert_eq!(payload["code"], "fanout_invalid");
}
