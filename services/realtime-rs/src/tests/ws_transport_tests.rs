use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, Message as WsMessage},
};

use crate::app::{build_app, AppState};

use crate::transport::ws::handlers::gateway::{is_session_valid, route_inbound_event};

const TEST_ALLOWED_ORIGIN: &str = "http://localhost:3002";

fn test_allowed_origins() -> Vec<String> {
    vec![TEST_ALLOWED_ORIGIN.to_string()]
}

fn set_allowed_origin(request: &mut tokio_tungstenite::tungstenite::http::Request<()>) {
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static(TEST_ALLOWED_ORIGIN));
}

#[derive(Serialize)]
struct ValidatePayload {
    session_id: String,
    identity_id: String,
    expires_at: String,
}

async fn start_validate_server(accept_authorization: bool) -> String {
    async fn validate_endpoint(
        State(accept_authorization): State<bool>,
        headers: HeaderMap,
    ) -> (StatusCode, Json<ValidatePayload>) {
        let authorized = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == "Bearer test-token")
            .unwrap_or(false)
            || headers
                .get("cookie")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.contains("hexrelay_session=test-token"))
                .unwrap_or(false);

        if !authorized {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ValidatePayload {
                    session_id: String::new(),
                    identity_id: String::new(),
                    expires_at: String::new(),
                }),
            );
        }

        let payload = ValidatePayload {
            session_id: "sess-1".to_string(),
            identity_id: "usr-1".to_string(),
            expires_at: "2030-01-01T00:00:00Z".to_string(),
        };

        if accept_authorization {
            (StatusCode::OK, Json(payload))
        } else {
            (StatusCode::UNAUTHORIZED, Json(payload))
        }
    }

    let app = Router::new()
        .route("/v1/auth/sessions/validate", get(validate_endpoint))
        .with_state(accept_authorization);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("read listener address");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test API");
    });

    format!("http://{}", address)
}

#[tokio::test]
async fn rejects_missing_authorization_header() {
    let state = AppState::new(
        "http://127.0.0.1:1".to_string(),
        test_allowed_origins(),
        60,
        60,
        16384,
        120,
        60,
        3,
    );
    let headers = HeaderMap::new();

    assert!(!is_session_valid(&state, &headers).await);
}

#[tokio::test]
async fn accepts_valid_authorization_with_successful_validation() {
    let api_base = start_validate_server(true).await;
    let state = AppState::new(api_base, test_allowed_origins(), 60, 60, 16384, 120, 60, 3);
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    assert!(is_session_valid(&state, &headers).await);
}

#[tokio::test]
async fn rejects_authorization_when_validation_endpoint_denies() {
    let api_base = start_validate_server(false).await;
    let state = AppState::new(api_base, test_allowed_origins(), 60, 60, 16384, 120, 60, 3);
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    assert!(!is_session_valid(&state, &headers).await);
}

async fn start_ws_server(api_base_url: String, ws_connect_rate_limit: usize) -> String {
    start_ws_server_with_limits(api_base_url, ws_connect_rate_limit, 16384, 120, 60, 3).await
}

async fn start_ws_server_with_limits(
    api_base_url: String,
    ws_connect_rate_limit: usize,
    ws_max_inbound_message_bytes: usize,
    ws_message_rate_limit: usize,
    ws_message_rate_window_seconds: u64,
    ws_max_connections_per_identity: usize,
) -> String {
    let app = build_app(AppState::new(
        api_base_url,
        test_allowed_origins(),
        ws_connect_rate_limit,
        60,
        ws_max_inbound_message_bytes,
        ws_message_rate_limit,
        ws_message_rate_window_seconds,
        ws_max_connections_per_identity,
    ));
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind websocket listener");
    let address = listener
        .local_addr()
        .expect("read websocket listener address");

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve websocket app");
    });

    format!("ws://{}/ws", address)
}

#[tokio::test]
async fn websocket_upgrade_rejects_missing_authorization() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;

    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_disallowed_origin() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    request
        .headers_mut()
        .insert("origin", HeaderValue::from_static("https://evil.example"));

    let result = connect_async(request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("403") || message.contains("Forbidden"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_missing_origin() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );

    let result = connect_async(request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("403") || message.contains("Forbidden"));
}

#[tokio::test]
async fn websocket_upgrade_accepts_valid_cookie() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "cookie",
        HeaderValue::from_static("hexrelay_session=test-token"),
    );
    set_allowed_origin(&mut request);

    let connection = connect_async(request)
        .await
        .expect("websocket connect response");

    assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_upgrade_accepts_valid_authorization() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let connection = connect_async(request)
        .await
        .expect("websocket connect response");

    assert_eq!(connection.1.status(), StatusCode::SWITCHING_PROTOCOLS);
}

#[tokio::test]
async fn websocket_replies_with_valid_event_envelope_for_call_signal_offer() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-123","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-b","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send offer event");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");
    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };

    let payload: Value = serde_json::from_str(&text).expect("decode response envelope");
    assert_eq!(payload["event_type"], "call.signal.offer");
    assert_eq!(payload["event_version"], 1);
    assert_eq!(payload["producer"], "realtime-gateway");
    assert_eq!(payload["correlation_id"], "corr-123");
    assert_eq!(payload["data"]["call_id"], "call-1");
}

#[test]
fn returns_error_for_invalid_event_payload() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":1,"data":{"call_id":"x"}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_invalid");
}

#[test]
fn returns_error_for_unsupported_version() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":2,"data":{}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_version_unsupported");
}

#[test]
fn returns_error_for_unsupported_event_type() {
    let response = route_inbound_event(
        r#"{"event_type":"presence.updated","event_version":1,"data":{}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_unsupported");
}

#[test]
fn returns_error_for_identity_mismatch() {
    let response = route_inbound_event(
        r#"{"event_type":"call.signal.offer","event_version":1,"data":{"call_id":"call-1","from_user_id":"usr-b","to_user_id":"usr-a","sdp_offer":"v=0\r\n"}}"#,
        "usr-a",
    );

    let payload: Value = serde_json::from_str(&response).expect("decode error envelope");
    assert_eq!(payload["event_type"], "error");
    assert_eq!(payload["data"]["code"], "event_identity_mismatch");
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_api_is_unreachable() {
    let ws_url = start_ws_server("http://127.0.0.1:1".to_string(), 60).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let result = connect_async(request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("401") || message.contains("Unauthorized"));
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_rate_limited() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 1).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);

    let _ = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());

    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_upgrade_rate_limit_cannot_be_bypassed_by_rotating_auth_header() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server(api_base, 1).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);

    let _ = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer totally-different-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_closes_with_rate_limited_event_when_message_limit_exceeded() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 16384, 1, 60, 3).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-1","data":{"call_id":"call-1","from_user_id":"usr-1","to_user_id":"usr-b","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send first message");
    let _ = socket.next().await;

    socket
        .send(WsMessage::Text(
            r#"{"event_type":"call.signal.offer","event_version":1,"correlation_id":"corr-2","data":{"call_id":"call-2","from_user_id":"usr-1","to_user_id":"usr-c","sdp_offer":"v=0\r\n"}}"#
                .to_string(),
        ))
        .await
        .expect("send second message");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");
    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };

    let payload: Value = serde_json::from_str(&text).expect("decode error envelope");
    assert_eq!(payload["data"]["code"], "event_rate_limited");
}

#[tokio::test]
async fn websocket_upgrade_rejects_when_identity_connection_cap_exceeded() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 16384, 120, 60, 1).await;

    let mut first_request = ws_url
        .clone()
        .into_client_request()
        .expect("build first websocket request");
    first_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut first_request);
    let (_first_socket, _) = connect_async(first_request)
        .await
        .expect("first websocket should connect");

    let mut second_request = ws_url
        .into_client_request()
        .expect("build second websocket request");
    second_request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut second_request);

    let result = connect_async(second_request).await;
    assert!(result.is_err());
    let message = result
        .err()
        .map(|value| value.to_string())
        .unwrap_or_default();
    assert!(message.contains("429") || message.contains("Too Many Requests"));
}

#[tokio::test]
async fn websocket_rejects_text_payload_above_limit() {
    let api_base = start_validate_server(true).await;
    let ws_url = start_ws_server_with_limits(api_base, 60, 64, 120, 60, 3).await;

    let mut request = ws_url
        .into_client_request()
        .expect("build websocket client request");
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_static("Bearer test-token"),
    );
    set_allowed_origin(&mut request);

    let (mut socket, _) = connect_async(request)
        .await
        .expect("websocket connect response");

    let _ = socket.next().await;

    socket
        .send(WsMessage::Text("x".repeat(1024)))
        .await
        .expect("send oversized payload");

    let message = socket
        .next()
        .await
        .expect("socket message")
        .expect("ws frame");

    let text = match message {
        WsMessage::Text(value) => value,
        _ => panic!("expected text frame"),
    };
    let payload: Value = serde_json::from_str(&text).expect("decode error envelope");
    assert_eq!(payload["data"]["code"], "event_too_large");
}
