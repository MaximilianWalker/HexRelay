use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

use crate::state::AppState;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ws_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if !is_allowed_origin(&state, &headers) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let rate_key = websocket_rate_limit_key(&headers);
    let allowed = state.rate_limiter.allow(
        "ws_connect",
        &rate_key,
        state.ws_connect_rate_limit,
        state.ws_rate_limit_window_seconds,
    );
    if !allowed {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    let session = match validate_session(&state, &headers).await {
        Some(value) => value,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    if !try_acquire_connection_slot(&state, &session.identity_id).await {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(state, socket, session.identity_id))
}

async fn handle_socket(state: AppState, mut socket: WebSocket, session_identity_id: String) {
    let _ = socket.send(Message::Text(connection_ready_banner())).await;

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if text.len() > state.ws_max_inbound_message_bytes {
                    let _ = socket
                        .send(Message::Text(build_error_event(
                            "event_too_large",
                            "inbound message exceeds max size",
                        )))
                        .await;
                    break;
                }

                let allowed = state.rate_limiter.allow(
                    "ws_message",
                    &session_identity_id,
                    state.ws_message_rate_limit,
                    state.ws_message_rate_window_seconds,
                );
                if !allowed {
                    let _ = socket
                        .send(Message::Text(build_error_event(
                            "event_rate_limited",
                            "too many websocket messages",
                        )))
                        .await;
                    break;
                }

                let response = route_inbound_event(&text, &session_identity_id);
                let _ = socket.send(Message::Text(response)).await;
            }
            Ok(Message::Binary(bytes)) => {
                if bytes.len() > state.ws_max_inbound_message_bytes {
                    let _ = socket
                        .send(Message::Text(build_error_event(
                            "event_too_large",
                            "inbound message exceeds max size",
                        )))
                        .await;
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    release_connection_slot(&state, &session_identity_id).await;
}

#[derive(Deserialize)]
struct SessionValidateResponse {
    #[serde(rename = "session_id")]
    _session_id: String,
    #[serde(rename = "identity_id")]
    identity_id: String,
    #[serde(rename = "expires_at")]
    _expires_at: String,
}

#[derive(Clone)]
struct ValidatedSession {
    identity_id: String,
}

#[derive(Deserialize)]
struct RealtimeInboundEnvelope {
    event_type: String,
    event_version: u8,
    #[serde(default)]
    correlation_id: Option<String>,
    data: Value,
}

#[derive(Deserialize, Serialize)]
struct CallSignalOfferData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    sdp_offer: String,
}

#[derive(Deserialize, Serialize)]
struct CallSignalAnswerData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    sdp_answer: String,
}

#[derive(Deserialize, Serialize)]
struct CallSignalIceCandidateData {
    call_id: String,
    from_user_id: String,
    to_user_id: String,
    candidate: String,
    #[serde(default)]
    sdp_mid: Option<String>,
    #[serde(default)]
    sdp_mline_index: Option<i32>,
}

#[derive(Serialize)]
struct RealtimeOutboundEnvelope<T: Serialize> {
    event_id: String,
    event_type: String,
    event_version: u8,
    occurred_at: String,
    correlation_id: String,
    producer: String,
    data: T,
}

#[derive(Serialize)]
struct RealtimeErrorData {
    code: String,
    message: String,
}

fn connection_ready_banner() -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "realtime.connected".to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: Uuid::new_v4().to_string(),
        producer: "realtime-gateway".to_string(),
        data: serde_json::json!({ "status": "ok" }),
    };

    serde_json::to_string(&envelope)
        .unwrap_or_else(|_| "{\"event_type\":\"realtime.connected\"}".to_string())
}

fn route_inbound_event(raw: &str, session_identity_id: &str) -> String {
    let parsed = match serde_json::from_str::<RealtimeInboundEnvelope>(raw) {
        Ok(value) => value,
        Err(_) => {
            return build_error_event("event_invalid", "invalid event envelope payload");
        }
    };

    if parsed.event_version != 1 {
        return build_error_event("event_version_unsupported", "event_version must be 1");
    }

    match parsed.event_type.as_str() {
        "call.signal.offer" => match serde_json::from_value::<CallSignalOfferData>(parsed.data) {
            Ok(data) => {
                if data.from_user_id != session_identity_id {
                    return build_error_event(
                        "event_identity_mismatch",
                        "from_user_id does not match authenticated session",
                    );
                }

                build_event("call.signal.offer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.offer payload"),
        },
        "call.signal.answer" => match serde_json::from_value::<CallSignalAnswerData>(parsed.data) {
            Ok(data) => {
                if data.from_user_id != session_identity_id {
                    return build_error_event(
                        "event_identity_mismatch",
                        "from_user_id does not match authenticated session",
                    );
                }

                build_event("call.signal.answer", parsed.correlation_id, data)
            }
            Err(_) => build_error_event("event_invalid", "invalid call.signal.answer payload"),
        },
        "call.signal.ice_candidate" => {
            match serde_json::from_value::<CallSignalIceCandidateData>(parsed.data) {
                Ok(data) => {
                    if data.from_user_id != session_identity_id {
                        return build_error_event(
                            "event_identity_mismatch",
                            "from_user_id does not match authenticated session",
                        );
                    }

                    build_event("call.signal.ice_candidate", parsed.correlation_id, data)
                }
                Err(_) => {
                    build_error_event("event_invalid", "invalid call.signal.ice_candidate payload")
                }
            }
        }
        _ => build_error_event("event_unsupported", "unsupported realtime event_type"),
    }
}

fn build_event<T: Serialize>(event_type: &str, correlation_id: Option<String>, data: T) -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: event_type.to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: correlation_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        producer: "realtime-gateway".to_string(),
        data,
    };

    serde_json::to_string(&envelope).unwrap_or_else(|_| {
        build_error_event("event_serialize_failed", "failed to serialize event")
    })
}

fn build_error_event(code: &str, message: &str) -> String {
    let envelope = RealtimeOutboundEnvelope {
        event_id: Uuid::new_v4().to_string(),
        event_type: "error".to_string(),
        event_version: 1,
        occurred_at: Utc::now().to_rfc3339(),
        correlation_id: Uuid::new_v4().to_string(),
        producer: "realtime-gateway".to_string(),
        data: RealtimeErrorData {
            code: code.to_string(),
            message: message.to_string(),
        },
    };

    serde_json::to_string(&envelope).unwrap_or_else(|_| "{\"event_type\":\"error\"}".to_string())
}

#[cfg(test)]
async fn is_session_valid(state: &AppState, headers: &HeaderMap) -> bool {
    validate_session(state, headers).await.is_some()
}

async fn validate_session(state: &AppState, headers: &HeaderMap) -> Option<ValidatedSession> {
    let cookie_header = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty());
    let authorization_header = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty());
    if cookie_header.is_none() && authorization_header.is_none() {
        return None;
    }

    let url = format!("{}/v1/auth/sessions/validate", state.api_base_url);
    let mut request = state.http_client.get(url);
    if let Some(cookie_header) = cookie_header {
        request = request.header("cookie", cookie_header);
    }
    if let Some(authorization_header) = authorization_header {
        request = request.header("authorization", authorization_header);
    }

    let response = match request.send().await {
        Ok(value) => value,
        Err(_) => return None,
    };

    if !response.status().is_success() {
        return None;
    }

    let payload = match response.json::<SessionValidateResponse>().await {
        Ok(value) => value,
        Err(_) => return None,
    };

    Some(ValidatedSession {
        identity_id: payload.identity_id,
    })
}

fn websocket_rate_limit_key(headers: &HeaderMap) -> String {
    let cookie = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(cookie) = cookie {
        return format!("cookie:{:016x}", stable_hash(&cookie));
    }

    let auth = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());

    if let Some(auth) = auth {
        format!("auth:{:016x}", stable_hash(&auth))
    } else {
        "auth-missing".to_string()
    }
}

fn is_allowed_origin(state: &AppState, headers: &HeaderMap) -> bool {
    let Some(origin) = headers
        .get("origin")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return false;
    };

    state
        .allowed_origins
        .iter()
        .any(|allowed| allowed == origin)
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

async fn try_acquire_connection_slot(state: &AppState, identity_id: &str) -> bool {
    let mut guard = state.active_connections.lock().await;
    let current = guard.get(identity_id).copied().unwrap_or(0);
    if current >= state.ws_max_connections_per_identity {
        return false;
    }

    guard.insert(identity_id.to_string(), current + 1);
    true
}

async fn release_connection_slot(state: &AppState, identity_id: &str) {
    let mut guard = state.active_connections.lock().await;
    if let Some(current) = guard.get_mut(identity_id) {
        if *current <= 1 {
            guard.remove(identity_id);
        } else {
            *current -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
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

    use crate::{app::build_app, state::AppState};

    use super::{is_session_valid, route_inbound_event};

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
}
