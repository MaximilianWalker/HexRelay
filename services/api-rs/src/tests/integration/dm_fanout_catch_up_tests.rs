use super::*;

use futures::{SinkExt, StreamExt};
use realtime_rs::app::{build_app as build_realtime_app, AppState as RealtimeAppState};
use tokio::net::TcpListener;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, Message as WsMessage},
};

use crate::infra::db::repos::dm_repo;

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

fn device_secret(device_id: &str) -> String {
    format!("secret-{device_id}")
}

fn unique_message_id(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

async fn set_dm_policy_anyone(app: axum::Router, token: &str) -> axum::Router {
    let request = Request::builder()
        .method("POST")
        .uri("/dm/privacy-policy")
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

async fn heartbeat_device(app: &axum::Router, token: &str, device_id: &str, active: bool) {
    let heartbeat = Request::builder()
        .method("POST")
        .uri("/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"{device_id}","device_secret":"{}","active":{active}}}"#,
            device_secret(device_id)
        )))
        .expect("build profile device heartbeat request");
    let response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(response.status(), StatusCode::OK);
}

async fn dispatch_dm(
    app: &axum::Router,
    token: &str,
    recipient_identity_id: &str,
    message_id: &str,
    ciphertext: &str,
) {
    let dispatch = Request::builder()
        .method("POST")
        .uri("/dm/fanout/dispatch")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{recipient_identity_id}","message_id":"{message_id}","ciphertext":"{ciphertext}"}}"#
        )))
        .expect("build fanout dispatch request");
    let response = app
        .clone()
        .oneshot(dispatch)
        .await
        .expect("fanout dispatch response");
    assert_eq!(response.status(), StatusCode::OK);
}

async fn catch_up(app: &axum::Router, token: &str, device_id: &str) -> serde_json::Value {
    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"{device_id}","device_secret":"{}"}}"#,
            device_secret(device_id)
        )))
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
    serde_json::from_slice(&body).expect("decode catch-up body")
}

async fn catch_up_page(
    app: &axum::Router,
    token: &str,
    device_id: &str,
    cursor: Option<&str>,
    limit: u32,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "device_id": device_id,
        "device_secret": device_secret(device_id),
        "limit": limit,
    });
    if let Some(cursor) = cursor {
        payload["cursor"] = serde_json::json!(cursor);
    }

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("build paged fanout catch-up request");
    let response = app
        .clone()
        .oneshot(catch_up)
        .await
        .expect("paged fanout catch-up response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read paged catch-up body");
    serde_json::from_slice(&body).expect("decode paged catch-up body")
}

async fn ack_dm_envelope(
    app: &axum::Router,
    envelope_id: &str,
    message_id: &str,
    thread_id: &str,
    recipient_identity_id: &str,
    device_id: &str,
    delivery_cursor: &str,
) -> (StatusCode, serde_json::Value) {
    let ack = Request::builder()
        .method("POST")
        .uri("/internal/dm/envelopes/ack")
        .header(
            "x-hexrelay-internal-token",
            "hexrelay-dev-channel-dispatch-token-change-me",
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"envelope_id":"{envelope_id}","message_id":"{message_id}","thread_id":"{thread_id}","recipient_identity_id":"{recipient_identity_id}","device_id":"{device_id}","delivery_cursor":"{delivery_cursor}","ack_status":"received","received_at":"2026-03-26T00:00:01Z"}}"#
        )))
        .expect("build ack request");
    let response = app.clone().oneshot(ack).await.expect("ack response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read ack body");
    let payload = if body.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&body).expect("decode ack body")
    };
    (status, payload)
}

fn expected_envelope_id(
    message_id: &str,
    recipient_identity_id: &str,
    device_id: &str,
    delivery_cursor: &str,
) -> String {
    let material = format!("{message_id}:{recipient_identity_id}:{device_id}:{delivery_cursor}");
    let digest = digest(&SHA256, material.as_bytes());
    format!("dm-env-{}", hex::encode(digest.as_ref()))
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

async fn connect_ws_with_token_and_device(ws_url: &str, token: &str, device_id: &str) -> WsStream {
    connect_ws_with_token_device_secret(ws_url, token, device_id, &device_secret(device_id)).await
}

async fn connect_ws_with_token_device_secret(
    ws_url: &str,
    token: &str,
    device_id: &str,
    device_secret_value: &str,
) -> WsStream {
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
    request.headers_mut().insert(
        "x-hexrelay-device-secret",
        HeaderValue::from_str(device_secret_value).expect("device secret header"),
    );

    let (socket, _) = connect_async(request)
        .await
        .expect("connect websocket with device");
    socket
}

async fn recv_ws_event(
    socket: &mut WsStream,
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
    .expect("websocket event timeout")
}

async fn recv_ws_event_type(socket: &mut WsStream, expected_event_type: &str) -> serde_json::Value {
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
            if payload["event_type"] == expected_event_type {
                break payload;
            }
        }
    })
    .await
    .expect("websocket event timeout")
}

async fn recv_ws_event_type_timeout(
    socket: &mut WsStream,
    expected_event_type: &str,
    timeout: std::time::Duration,
) -> Option<serde_json::Value> {
    tokio::time::timeout(timeout, async {
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
            if payload["event_type"] == expected_event_type {
                break payload;
            }
        }
    })
    .await
    .ok()
}

#[tokio::test]
async fn dm_envelope_dispatch_ack_persists_through_realtime_websocket() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let sender = unique_identity("usr-realtime-sender");
    let recipient = unique_identity("usr-realtime-recipient");
    let device_id = "desktop-main";
    let message_id = format!("msg-realtime-{}", Uuid::new_v4().simple());

    let realtime_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind realtime listener");
    let realtime_address = realtime_listener
        .local_addr()
        .expect("read realtime listener address");
    let realtime_base_url = format!("http://{realtime_address}");

    let mut api_state = AppState::default().with_db_pool(pool.clone());
    api_state.realtime_base_url = realtime_base_url;
    let sender_token = issue_db_session_cookie(&pool, &api_state, &sender).await;
    let recipient_token = issue_db_session_cookie(&pool, &api_state, &recipient).await;
    let api_base_url = start_api_http_server(api_state.clone()).await;

    let realtime_state = RealtimeAppState::new(
        api_base_url.clone(),
        vec!["http://localhost:3002".to_string()],
        api_state.channel_dispatch_internal_token.clone(),
        api_state.presence_watcher_internal_token.clone(),
        None,
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
    let realtime_app = build_realtime_app(realtime_state);
    tokio::spawn(async move {
        axum::serve(
            realtime_listener,
            realtime_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve realtime app");
    });

    let client = reqwest::Client::new();
    let policy_response = client
        .post(format!("{api_base_url}/dm/privacy-policy"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({ "inbound_policy": "anyone" }))
        .send()
        .await
        .expect("set recipient DM policy");
    assert_eq!(policy_response.status(), reqwest::StatusCode::OK);

    let heartbeat_response = client
        .post(format!("{api_base_url}/dm/profile-devices/heartbeat"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({
            "device_id": device_id,
            "device_secret": device_secret(device_id),
            "active": true,
        }))
        .send()
        .await
        .expect("heartbeat recipient device");
    assert_eq!(heartbeat_response.status(), reqwest::StatusCode::OK);

    let ws_url = format!("ws://{realtime_address}/ws");
    let mut recipient_socket =
        connect_ws_with_token_and_device(&ws_url, &recipient_token, device_id).await;
    let connected = recv_ws_event_type(&mut recipient_socket, "realtime.connected").await;
    assert_eq!(connected["data"]["status"], "ok");
    let verified = recv_ws_event_type(&mut recipient_socket, "dm.device.verified").await;
    assert_eq!(verified["data"]["device_id"], device_id);

    let fanout_response = client
        .post(format!("{api_base_url}/dm/fanout/dispatch"))
        .bearer_auth(&sender_token)
        .json(&serde_json::json!({
            "recipient_identity_id": &recipient,
            "message_id": &message_id,
            "ciphertext": "enc:realtime-dispatch-ack",
        }))
        .send()
        .await
        .expect("dispatch DM fanout");
    assert_eq!(fanout_response.status(), reqwest::StatusCode::OK);
    let fanout_payload: serde_json::Value = fanout_response.json().await.expect("decode fanout");
    assert_eq!(fanout_payload["status"], "accepted");
    assert_eq!(
        fanout_payload["delivered_device_ids"],
        serde_json::json!([])
    );

    let dispatched =
        recv_ws_event(&mut recipient_socket, "dm.envelope.dispatched", &message_id).await;
    assert_eq!(dispatched["data"]["recipient_identity_id"], recipient);
    assert_eq!(dispatched["data"]["target_device_id"], device_id);
    assert_eq!(dispatched["data"]["delivery_cursor"], "1");

    let ack_payload = serde_json::json!({
        "event_type": "dm.envelope.ack",

        "correlation_id": "corr-dm-ack-1",
        "data": {
            "envelope_id": dispatched["data"]["envelope_id"],
            "message_id": dispatched["data"]["message_id"],
            "thread_id": dispatched["data"]["thread_id"],
            "recipient_identity_id": dispatched["data"]["recipient_identity_id"],
            "device_id": dispatched["data"]["target_device_id"],
            "delivery_cursor": dispatched["data"]["delivery_cursor"],
            "ack_status": "received",
            "received_at": "2026-03-26T00:00:01Z",
        }
    });
    recipient_socket
        .send(WsMessage::Text(ack_payload.to_string()))
        .await
        .expect("send DM ack");
    let ack_echo = recv_ws_event(&mut recipient_socket, "dm.envelope.ack", &message_id).await;
    assert_eq!(ack_echo["correlation_id"], "corr-dm-ack-1");
    assert_eq!(ack_echo["data"]["ack_status"], "received");

    let catch_up_response = client
        .post(format!("{api_base_url}/dm/fanout/catch-up"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({ "device_id": device_id, "device_secret": device_secret(device_id) }))
        .send()
        .await
        .expect("catch up recipient device");
    assert_eq!(catch_up_response.status(), reqwest::StatusCode::OK);
    let catch_up_payload: serde_json::Value =
        catch_up_response.json().await.expect("decode catch-up");
    assert_eq!(catch_up_payload["reason_code"], "fanout_catch_up_no_missed");
    assert_eq!(catch_up_payload["replay_count"], 0);
    assert_eq!(catch_up_payload["next_cursor"], "1");
}

#[tokio::test]
async fn dm_realtime_dispatch_requires_verified_device_secret() {
    let Some(pool) = prepared_database_pool().await else {
        return;
    };

    let sender = unique_identity("usr-realtime-spoof-sender");
    let recipient = unique_identity("usr-realtime-spoof-recipient");
    let device_id = "desktop-main";
    let message_id = format!("msg-realtime-spoof-{}", Uuid::new_v4().simple());

    let realtime_listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind realtime listener");
    let realtime_address = realtime_listener
        .local_addr()
        .expect("read realtime listener address");
    let realtime_base_url = format!("http://{realtime_address}");

    let mut api_state = AppState::default().with_db_pool(pool.clone());
    api_state.realtime_base_url = realtime_base_url;
    let sender_token = issue_db_session_cookie(&pool, &api_state, &sender).await;
    let recipient_token = issue_db_session_cookie(&pool, &api_state, &recipient).await;
    let api_base_url = start_api_http_server(api_state.clone()).await;

    let realtime_state = RealtimeAppState::new(
        api_base_url.clone(),
        vec!["http://localhost:3002".to_string()],
        api_state.channel_dispatch_internal_token.clone(),
        api_state.presence_watcher_internal_token.clone(),
        None,
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
    let realtime_app = build_realtime_app(realtime_state);
    tokio::spawn(async move {
        axum::serve(
            realtime_listener,
            realtime_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("serve realtime app");
    });

    let client = reqwest::Client::new();
    let policy_response = client
        .post(format!("{api_base_url}/dm/privacy-policy"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({ "inbound_policy": "anyone" }))
        .send()
        .await
        .expect("set recipient DM policy");
    assert_eq!(policy_response.status(), reqwest::StatusCode::OK);

    let heartbeat_response = client
        .post(format!("{api_base_url}/dm/profile-devices/heartbeat"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({
            "device_id": device_id,
            "device_secret": device_secret(device_id),
            "active": true,
        }))
        .send()
        .await
        .expect("heartbeat recipient device");
    assert_eq!(heartbeat_response.status(), reqwest::StatusCode::OK);

    let ws_url = format!("ws://{realtime_address}/ws");
    let mut recipient_socket = connect_ws_with_token_device_secret(
        &ws_url,
        &recipient_token,
        device_id,
        "secret-wrong-device",
    )
    .await;
    let connected = recv_ws_event_type(&mut recipient_socket, "realtime.connected").await;
    assert_eq!(connected["data"]["status"], "ok");
    let proof_error = recv_ws_event_type(&mut recipient_socket, "error").await;
    assert_eq!(proof_error["data"]["code"], "event_device_mismatch");

    let fanout_response = client
        .post(format!("{api_base_url}/dm/fanout/dispatch"))
        .bearer_auth(&sender_token)
        .json(&serde_json::json!({
            "recipient_identity_id": &recipient,
            "message_id": &message_id,
            "ciphertext": "enc:realtime-wrong-secret",
        }))
        .send()
        .await
        .expect("dispatch DM fanout");
    assert_eq!(fanout_response.status(), reqwest::StatusCode::OK);

    let unexpected_dispatch = recv_ws_event_type_timeout(
        &mut recipient_socket,
        "dm.envelope.dispatched",
        std::time::Duration::from_secs(1),
    )
    .await;
    assert!(unexpected_dispatch.is_none());

    let catch_up_response = client
        .post(format!("{api_base_url}/dm/fanout/catch-up"))
        .bearer_auth(&recipient_token)
        .json(&serde_json::json!({ "device_id": device_id, "device_secret": device_secret(device_id) }))
        .send()
        .await
        .expect("catch up recipient device");
    assert_eq!(catch_up_response.status(), reqwest::StatusCode::OK);
    let catch_up_payload: serde_json::Value =
        catch_up_response.json().await.expect("decode catch-up");
    assert_eq!(catch_up_payload["replay_count"], 1);
}

#[tokio::test]
async fn fanout_catch_up_paginates_from_request_cursor() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let message_ids = [
        unique_message_id("msg-page-a"),
        unique_message_id("msg-page-b"),
        unique_message_id("msg-page-c"),
    ];
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;
    heartbeat_device(&app, &tokens[recipient.as_str()], "desktop-main", true).await;

    for message_id in &message_ids {
        dispatch_dm(
            &app,
            &tokens[sender.as_str()],
            &recipient,
            message_id,
            "enc:paged-catch-up",
        )
        .await;
    }

    let first_page =
        catch_up_page(&app, &tokens[recipient.as_str()], "desktop-main", None, 1).await;
    assert_eq!(first_page["replay_count"], 1);
    assert_eq!(first_page["next_cursor"], "1");
    assert_eq!(first_page["items"][0]["cursor"], "1");
    assert_eq!(
        first_page["items"][0]["message_id"],
        message_ids[0].as_str()
    );

    let second_page = catch_up_page(
        &app,
        &tokens[recipient.as_str()],
        "desktop-main",
        Some("1"),
        1,
    )
    .await;
    assert_eq!(second_page["replay_count"], 1);
    assert_eq!(second_page["next_cursor"], "2");
    assert_eq!(second_page["items"][0]["cursor"], "2");
    assert_eq!(
        second_page["items"][0]["message_id"],
        message_ids[1].as_str()
    );
}

#[tokio::test]
async fn fanout_catch_up_next_cursor_advances_past_deduped_rows() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let message_id = unique_message_id("msg-page-dup");
    let Some((app, tokens, pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;
    heartbeat_device(&app, &tokens[recipient.as_str()], "desktop-main", true).await;

    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &message_id,
        "enc:paged-duplicate",
    )
    .await;

    let records = dm_repo::list_dm_fanout_delivery_records_page(&pool, &recipient, 0, 10)
        .await
        .expect("list delivery records");
    assert_eq!(records.len(), 1);

    let mut tx = pool.begin().await.expect("begin duplicate delivery tx");
    let duplicate_cursor = dm_repo::advance_dm_fanout_stream_head_in_tx(&mut tx, &recipient)
        .await
        .expect("advance duplicate delivery cursor");
    let duplicate = crate::models::DmFanoutDeliveryRecord {
        cursor: duplicate_cursor,
        ..records[0].clone()
    };
    dm_repo::append_dm_fanout_delivery_record_in_tx(
        &mut tx,
        &recipient,
        &records[0].thread_id,
        &duplicate,
    )
    .await
    .expect("append duplicate delivery record");
    tx.commit().await.expect("commit duplicate delivery tx");

    let first_page =
        catch_up_page(&app, &tokens[recipient.as_str()], "desktop-main", None, 2).await;
    assert_eq!(first_page["replay_count"], 1);
    assert_eq!(first_page["items"][0]["cursor"], "1");
    assert_eq!(first_page["next_cursor"], duplicate_cursor.to_string());
    assert_eq!(
        first_page["deduped_message_ids"]
            .as_array()
            .expect("deduped ids")
            .len(),
        1
    );

    let next_page = catch_up_page(
        &app,
        &tokens[recipient.as_str()],
        "desktop-main",
        Some(first_page["next_cursor"].as_str().expect("next cursor")),
        2,
    )
    .await;
    assert_eq!(next_page["reason_code"], "fanout_catch_up_no_missed");
    assert_eq!(next_page["replay_count"], 0);
    assert_eq!(next_page["next_cursor"], duplicate_cursor.to_string());
}

#[tokio::test]
async fn fanout_catch_up_replays_messages_for_late_activated_device() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let message_id = unique_message_id("msg-2001");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/dm/profile-devices/heartbeat")
            .header(
                "authorization",
                format!("Bearer {}", tokens[recipient.as_str()]),
            )
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","device_secret":"{}","active":{active}}}"#,
                device_secret(device_id)
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
        .uri("/dm/fanout/dispatch")
        .header(
            "authorization",
            format!("Bearer {}", tokens[sender.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:late-2001"}}"#,
            recipient, message_id
        )))
        .expect("build fanout dispatch request");
    let dispatch_response = app
        .clone()
        .oneshot(dispatch)
        .await
        .expect("fanout dispatch response");
    assert_eq!(dispatch_response.status(), StatusCode::OK);

    let activate_phone = Request::builder()
        .method("POST")
        .uri("/dm/profile-devices/heartbeat")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}","active":true}}"#,
            device_secret("phone-main")
        )))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
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
    assert_eq!(payload["items"][0]["message_id"], message_id.as_str());
}

#[tokio::test]
async fn fanout_catch_up_replays_until_ack_advances_cursor() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let message_ids = [
        unique_message_id("msg-dup-a"),
        unique_message_id("msg-dup-b"),
    ];
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/dm/profile-devices/heartbeat")
            .header(
                "authorization",
                format!("Bearer {}", tokens[recipient.as_str()]),
            )
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","device_secret":"{}","active":{active}}}"#,
                device_secret(device_id)
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    for message_id in &message_ids {
        let dispatch = Request::builder()
            .method("POST")
            .uri("/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"{}","message_id":"{}","ciphertext":"enc:dup-same","source_device_id":"sender-main"}}"#,
                recipient, message_id
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
        .uri("/dm/profile-devices/heartbeat")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}","active":true}}"#,
            device_secret("phone-main")
        )))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
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
    assert_eq!(payload["replay_count"], 2);
    assert_eq!(payload["next_cursor"], "2");
    assert_eq!(payload["items"][0]["message_id"], message_ids[0].as_str());
    assert_eq!(payload["items"][1]["message_id"], message_ids[1].as_str());
    assert!(payload["items"][0]["envelope_id"]
        .as_str()
        .expect("envelope id")
        .starts_with("dm-env-"));
    assert!(payload["items"][0]["thread_id"]
        .as_str()
        .expect("thread id")
        .starts_with("dm-"));
    assert_eq!(
        payload["deduped_message_ids"]
            .as_array()
            .expect("deduped ids array")
            .len(),
        0
    );

    let second_catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
        .expect("build second fanout catch-up request");
    let second_response = app
        .clone()
        .oneshot(second_catch_up)
        .await
        .expect("second fanout catch-up response");
    assert_eq!(second_response.status(), StatusCode::OK);

    let second_body = to_bytes(second_response.into_body(), usize::MAX)
        .await
        .expect("read second catch-up body");
    let second_payload: serde_json::Value =
        serde_json::from_slice(&second_body).expect("decode second catch-up body");

    assert_eq!(second_payload["reason_code"], "fanout_catch_up_ok");
    assert_eq!(second_payload["replay_count"], 2);
    assert_eq!(second_payload["next_cursor"], "2");

    for item in payload["items"].as_array().expect("catch-up items") {
        let attempts = if item["cursor"] == "1" { 2 } else { 1 };
        for _ in 0..attempts {
            let ack = Request::builder()
                .method("POST")
                .uri("/internal/dm/envelopes/ack")
                .header(
                    "x-hexrelay-internal-token",
                    "hexrelay-dev-channel-dispatch-token-change-me",
                )
                .header("content-type", "application/json")
                .body(Body::from(format!(
                    r#"{{"envelope_id":"{}","message_id":"{}","thread_id":"{}","recipient_identity_id":"{}","device_id":"phone-main","delivery_cursor":"{}","ack_status":"received","received_at":"2026-03-26T00:00:01Z"}}"#,
                    item["envelope_id"].as_str().expect("envelope id"),
                    item["message_id"].as_str().expect("message id"),
                    item["thread_id"].as_str().expect("thread id"),
                    recipient,
                    item["cursor"].as_str().expect("cursor"),
                )))
                .expect("build ack request");
            let ack_response = app.clone().oneshot(ack).await.expect("ack response");
            assert_eq!(ack_response.status(), StatusCode::ACCEPTED);
        }
    }

    let post_ack_catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
        .expect("build post-ack fanout catch-up request");
    let post_ack_response = app
        .oneshot(post_ack_catch_up)
        .await
        .expect("post-ack fanout catch-up response");
    assert_eq!(post_ack_response.status(), StatusCode::OK);

    let post_ack_body = to_bytes(post_ack_response.into_body(), usize::MAX)
        .await
        .expect("read post-ack catch-up body");
    let post_ack_payload: serde_json::Value =
        serde_json::from_slice(&post_ack_body).expect("decode post-ack catch-up body");
    assert_eq!(post_ack_payload["reason_code"], "fanout_catch_up_no_missed");
    assert_eq!(post_ack_payload["replay_count"], 0);
    assert_eq!(post_ack_payload["next_cursor"], "2");
}

#[tokio::test]
async fn fanout_ack_does_not_skip_prior_unacked_envelopes() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let first_message_id = unique_message_id("msg-order-a");
    let second_message_id = unique_message_id("msg-order-b");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;
    heartbeat_device(&app, &tokens[recipient.as_str()], "desktop-main", true).await;

    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &first_message_id,
        "enc:order-a",
    )
    .await;
    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &second_message_id,
        "enc:order-b",
    )
    .await;

    let initial = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    assert_eq!(initial["replay_count"], 2);
    let items = initial["items"].as_array().expect("catch-up items");
    assert_eq!(items[0]["cursor"], "1");
    assert_eq!(items[1]["cursor"], "2");

    let second = &items[1];
    let (status, payload) = ack_dm_envelope(
        &app,
        second["envelope_id"].as_str().expect("envelope id"),
        second["message_id"].as_str().expect("message id"),
        second["thread_id"].as_str().expect("thread id"),
        &recipient,
        "desktop-main",
        second["cursor"].as_str().expect("cursor"),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(payload.is_null());

    let after_out_of_order_ack = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    assert_eq!(after_out_of_order_ack["replay_count"], 1);
    assert_eq!(after_out_of_order_ack["next_cursor"], "1");
    assert_eq!(after_out_of_order_ack["items"][0]["cursor"], "1");
    assert_eq!(
        after_out_of_order_ack["items"][0]["message_id"],
        first_message_id.as_str()
    );

    let first = &items[0];
    let (status, _) = ack_dm_envelope(
        &app,
        first["envelope_id"].as_str().expect("envelope id"),
        first["message_id"].as_str().expect("message id"),
        first["thread_id"].as_str().expect("thread id"),
        &recipient,
        "desktop-main",
        first["cursor"].as_str().expect("cursor"),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);

    let post_contiguous_ack = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    assert_eq!(
        post_contiguous_ack["reason_code"],
        "fanout_catch_up_no_missed"
    );
    assert_eq!(post_contiguous_ack["replay_count"], 0);
    assert_eq!(post_contiguous_ack["next_cursor"], "2");
}

#[tokio::test]
async fn fanout_ack_concurrent_out_of_order_advances_cursor() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let first_message_id = unique_message_id("msg-concurrent-a");
    let second_message_id = unique_message_id("msg-concurrent-b");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;
    heartbeat_device(&app, &tokens[recipient.as_str()], "desktop-main", true).await;

    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &first_message_id,
        "enc:concurrent-a",
    )
    .await;
    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &second_message_id,
        "enc:concurrent-b",
    )
    .await;

    let initial = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    assert_eq!(initial["replay_count"], 2);
    let items = initial["items"].as_array().expect("catch-up items");
    let first = &items[0];
    let second = &items[1];
    let first_envelope_id = first["envelope_id"]
        .as_str()
        .expect("envelope id")
        .to_string();
    let first_message_id = first["message_id"]
        .as_str()
        .expect("message id")
        .to_string();
    let first_thread_id = first["thread_id"].as_str().expect("thread id").to_string();
    let first_cursor = first["cursor"].as_str().expect("cursor").to_string();
    let second_envelope_id = second["envelope_id"]
        .as_str()
        .expect("envelope id")
        .to_string();
    let second_message_id = second["message_id"]
        .as_str()
        .expect("message id")
        .to_string();
    let second_thread_id = second["thread_id"].as_str().expect("thread id").to_string();
    let second_cursor = second["cursor"].as_str().expect("cursor").to_string();

    let second_ack = ack_dm_envelope(
        &app,
        &second_envelope_id,
        &second_message_id,
        &second_thread_id,
        &recipient,
        "desktop-main",
        &second_cursor,
    );
    let first_ack = ack_dm_envelope(
        &app,
        &first_envelope_id,
        &first_message_id,
        &first_thread_id,
        &recipient,
        "desktop-main",
        &first_cursor,
    );
    let ((second_status, _), (first_status, _)) = tokio::join!(second_ack, first_ack);
    assert_eq!(second_status, StatusCode::ACCEPTED);
    assert_eq!(first_status, StatusCode::ACCEPTED);

    let post_ack = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    assert_eq!(post_ack["reason_code"], "fanout_catch_up_no_missed");
    assert_eq!(post_ack["replay_count"], 0);
    assert_eq!(post_ack["next_cursor"], "2");
}

#[tokio::test]
async fn fanout_ack_rejects_mismatched_envelope_id_and_unknown_device() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let message_id = unique_message_id("msg-ack-validate");
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;
    heartbeat_device(&app, &tokens[recipient.as_str()], "desktop-main", true).await;

    dispatch_dm(
        &app,
        &tokens[sender.as_str()],
        &recipient,
        &message_id,
        "enc:ack-validate",
    )
    .await;

    let catch_up_payload = catch_up(&app, &tokens[recipient.as_str()], "desktop-main").await;
    let item = &catch_up_payload["items"][0];

    let (status, payload) = ack_dm_envelope(
        &app,
        "dm-env-wrong-but-shaped",
        item["message_id"].as_str().expect("message id"),
        item["thread_id"].as_str().expect("thread id"),
        &recipient,
        "desktop-main",
        item["cursor"].as_str().expect("cursor"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(payload["code"], "dm_ack_invalid");

    let unknown_device_id = "console-main";
    let unknown_device_envelope_id = expected_envelope_id(
        item["message_id"].as_str().expect("message id"),
        &recipient,
        unknown_device_id,
        item["cursor"].as_str().expect("cursor"),
    );
    let (status, payload) = ack_dm_envelope(
        &app,
        &unknown_device_envelope_id,
        item["message_id"].as_str().expect("message id"),
        item["thread_id"].as_str().expect("thread id"),
        &recipient,
        unknown_device_id,
        item["cursor"].as_str().expect("cursor"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(payload["code"], "dm_ack_unknown");
}

#[tokio::test]
async fn fanout_catch_up_keeps_distinct_payload_variants_with_same_message_id() {
    let sender = unique_identity("usr-nora-k");
    let recipient = unique_identity("usr-jules-p");
    let variants = [
        (
            unique_message_id("msg-variant-1"),
            "enc:variant-1",
            "sender-main",
        ),
        (
            unique_message_id("msg-variant-2"),
            "enc:variant-2",
            "tablet-main",
        ),
    ];
    let Some((app, tokens, _pool)) =
        app_with_database_and_sessions(&[sender.as_str(), recipient.as_str()]).await
    else {
        return;
    };
    let app = set_dm_policy_anyone(app, &tokens[recipient.as_str()]).await;

    for (device_id, active) in [("desktop-main", true), ("phone-main", false)] {
        let heartbeat = Request::builder()
            .method("POST")
            .uri("/dm/profile-devices/heartbeat")
            .header(
                "authorization",
                format!("Bearer {}", tokens[recipient.as_str()]),
            )
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"device_id":"{device_id}","device_secret":"{}","active":{active}}}"#,
                device_secret(device_id)
            )))
            .expect("build profile device heartbeat request");
        let heartbeat_response = app
            .clone()
            .oneshot(heartbeat)
            .await
            .expect("profile device heartbeat response");
        assert_eq!(heartbeat_response.status(), StatusCode::OK);
    }

    for (message_id, ciphertext, source_device_id) in &variants {
        let dispatch = Request::builder()
            .method("POST")
            .uri("/dm/fanout/dispatch")
            .header("authorization", format!("Bearer {}", tokens[sender.as_str()]))
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"recipient_identity_id":"{recipient}","message_id":"{message_id}","ciphertext":"{ciphertext}","source_device_id":"{source_device_id}"}}"#
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
        .uri("/dm/profile-devices/heartbeat")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}","active":true}}"#,
            device_secret("phone-main")
        )))
        .expect("build profile device activation request");
    let activate_response = app
        .clone()
        .oneshot(activate_phone)
        .await
        .expect("profile device activation response");
    assert_eq!(activate_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header(
            "authorization",
            format!("Bearer {}", tokens[recipient.as_str()]),
        )
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
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
        .uri("/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}","active":false}}"#,
            device_secret("desktop-main")
        )))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}"}}"#,
            device_secret("desktop-main")
        )))
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
async fn fanout_catch_up_rate_limits_per_identity_across_devices() {
    let mut state = test_state_with_public_identity_registration();
    state.rate_limits.dm_catch_up_per_window = 1;

    let identity_id = "usr-catch-up-rate";
    let expires_at = Utc::now() + Duration::hours(1);
    let session_id = format!("sess-{identity_id}");
    let token = issue_session_token(
        &session_id,
        identity_id,
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key for tests"),
    );

    state
        .sessions
        .write()
        .expect("acquire session write lock")
        .insert(
            session_id,
            SessionRecord {
                identity_id: identity_id.to_string(),
                expires_at,
            },
        );

    let app = build_app(state);
    heartbeat_device(&app, &token, "desktop-main", true).await;
    heartbeat_device(&app, &token, "phone-main", true).await;

    let first = catch_up(&app, &token, "desktop-main").await;
    assert_eq!(first["status"], "ready");

    let second = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"phone-main","device_secret":"{}"}}"#,
            device_secret("phone-main")
        )))
        .expect("build rate-limited catch-up request");

    let response = app
        .oneshot(second)
        .await
        .expect("rate-limited catch-up response");
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read rate-limit body");
    let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode rate-limit body");
    assert_eq!(payload["code"], "rate_limited");
}

#[tokio::test]
async fn fanout_catch_up_rejects_cursor_beyond_delivery_tail() {
    let (app, tokens) = app_with_sessions(&["usr-nora-k"]);

    let heartbeat = Request::builder()
        .method("POST")
        .uri("/dm/profile-devices/heartbeat")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}","active":true}}"#,
            device_secret("desktop-main")
        )))
        .expect("build profile device heartbeat request");
    let heartbeat_response = app
        .clone()
        .oneshot(heartbeat)
        .await
        .expect("profile device heartbeat response");
    assert_eq!(heartbeat_response.status(), StatusCode::OK);

    let catch_up = Request::builder()
        .method("POST")
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"device_id":"desktop-main","device_secret":"{}","cursor":"99"}}"#,
            device_secret("desktop-main")
        )))
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
        .uri("/dm/fanout/catch-up")
        .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"device_id":"   ","device_secret":"secret-desktop-main"}"#,
        ))
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
