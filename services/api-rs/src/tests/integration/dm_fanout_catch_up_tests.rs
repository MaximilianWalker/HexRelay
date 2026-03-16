use super::*;

#[tokio::test]
async fn fanout_catch_up_replays_messages_for_late_activated_device() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

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
        .body(Body::from(r#"{"device_id":"phone-main","active":true}"#))
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
async fn fanout_catch_up_dedupes_duplicate_message_ids_and_advances_cursor() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

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

    for ciphertext in ["enc:dup-1", "enc:dup-2"] {
        let dispatch = Request::builder()
            .method("POST")
            .uri("/v1/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"usr-jules-p","message_id":"msg-dup","ciphertext":"{ciphertext}"}}"#
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
        .body(Body::from(r#"{"device_id":"phone-main","active":true}"#))
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
