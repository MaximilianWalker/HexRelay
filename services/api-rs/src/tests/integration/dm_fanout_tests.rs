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
async fn fanout_dispatch_delivers_to_all_active_profile_devices() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    for (device_id, active) in [
        ("desktop-main", true),
        ("phone-main", true),
        ("tablet-idle", false),
    ] {
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

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-1001","ciphertext":"enc:abcd1234"}"#,
        ))
        .expect("build fanout request");
    let fanout_response = app.oneshot(fanout_request).await.expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["reason_code"], "fanout_ok");
    assert_eq!(payload["fanout_count"], 2);

    let delivered = payload["delivered_device_ids"]
        .as_array()
        .expect("delivered array");
    assert_eq!(delivered.len(), 2);
    assert!(delivered.contains(&serde_json::Value::String("desktop-main".to_string())));
    assert!(delivered.contains(&serde_json::Value::String("phone-main".to_string())));
}

#[tokio::test]
async fn fanout_dispatch_skips_source_device_when_present() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    for device_id in ["desktop-main", "phone-main"] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/v1/dm/profile-devices/heartbeat")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","active":true}}"#
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-1002","ciphertext":"enc:abcd9999","source_device_id":"desktop-main"}"#,
        ))
        .expect("build fanout request");
    let fanout_response = app.oneshot(fanout_request).await.expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "ready");
    assert_eq!(payload["fanout_count"], 1);
    assert_eq!(payload["delivered_device_ids"][0], "phone-main");
    assert!(payload["skipped_device_ids"]
        .as_array()
        .expect("skipped array")
        .contains(&serde_json::Value::String("desktop-main".to_string())));
}

#[tokio::test]
async fn fanout_dispatch_blocks_when_no_active_devices_registered() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    let fanout_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-1003","ciphertext":"enc:abcd5555"}"#,
        ))
        .expect("build fanout request");
    let fanout_response = app.oneshot(fanout_request).await.expect("fanout response");
    assert_eq!(fanout_response.status(), StatusCode::OK);

    let body = to_bytes(fanout_response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");

    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "fanout_no_active_devices");
    assert_eq!(payload["fanout_count"], 0);
}

#[tokio::test]
async fn fanout_dispatch_blocks_when_backlog_reaches_capacity() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);
    let app = set_dm_policy_anyone(app, &tokens["usr-jules-p"]).await;

    let heartbeat = Request::builder()
        .method("POST")
        .uri("/v1/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"device_id":"desktop-main","active":true}"#))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    for index in 1..=1024 {
        let fanout_request = Request::builder()
            .method("POST")
            .uri("/v1/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"usr-jules-p","message_id":"msg-{index}","ciphertext":"enc:{index}"}}"#
            )))
            .expect("build fanout request");
        let fanout_response = app
            .clone()
            .oneshot(fanout_request)
            .await
            .expect("fanout response while filling backlog");
        assert_eq!(fanout_response.status(), StatusCode::OK);
    }

    let blocked_request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-overflow","ciphertext":"enc:overflow"}"#,
        ))
        .expect("build blocked fanout request");
    let blocked_response = app
        .oneshot(blocked_request)
        .await
        .expect("blocked fanout response");
    assert_eq!(blocked_response.status(), StatusCode::OK);

    let body = to_bytes(blocked_response.into_body(), usize::MAX)
        .await
        .expect("read blocked fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode blocked payload");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "fanout_backlog_full");
    assert_eq!(payload["fanout_count"], 0);
}

#[tokio::test]
async fn fanout_dispatch_blocks_when_recipient_policy_disallows_sender() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"recipient_identity_id":"usr-jules-p","message_id":"msg-policy-blocked","ciphertext":"enc:block"}"#,
        ))
        .expect("build fanout request");
    let response = app.oneshot(request).await.expect("fanout response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read fanout body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode fanout body");
    assert_eq!(payload["status"], "blocked");
    assert_eq!(payload["reason_code"], "fanout_policy_blocked");
}
