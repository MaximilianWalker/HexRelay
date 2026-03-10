use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use futures::stream::StreamExt;
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use tracing::warn;

use crate::state::AppState;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    if !is_allowed_origin(&state, &headers) {
        warn!("rejected websocket upgrade due to disallowed origin");
        return ws_rejection(
            StatusCode::FORBIDDEN,
            "origin_disallowed",
            "websocket origin is not allowed",
        );
    }

    let rate_key = websocket_rate_limit_key(&state, &headers, Some(peer_addr));
    let allowed = state.rate_limiter.allow(
        "ws_connect",
        &rate_key,
        state.ws_connect_rate_limit,
        state.ws_rate_limit_window_seconds,
    );
    if !allowed {
        warn!(rate_key = %rate_key, "rejected websocket upgrade due to connect rate limit");
        return ws_rejection(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            "too many websocket upgrade attempts",
        );
    }

    let session = match validate_session(&state, &headers).await {
        Some(value) => value,
        None => {
            warn!("rejected websocket upgrade due to failed session validation");
            return ws_rejection(
                StatusCode::UNAUTHORIZED,
                "session_invalid",
                "session validation failed",
            );
        }
    };

    if !try_acquire_connection_slot(&state, &session.identity_id).await {
        warn!(
            identity_id = %session.identity_id,
            "rejected websocket upgrade due to identity connection cap"
        );
        return ws_rejection(
            StatusCode::TOO_MANY_REQUESTS,
            "connection_cap_reached",
            "too many active websocket sessions for identity",
        );
    }

    ws.on_upgrade(move |socket| handle_socket(state, socket, session.identity_id))
}

async fn handle_socket(state: AppState, mut socket: WebSocket, session_identity_id: String) {
    let _ = socket.send(Message::Text(connection_ready_banner())).await;

    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if text.len() > state.ws_max_inbound_message_bytes {
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to oversized text payload"
                    );
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
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to message rate limit"
                    );
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
                let allowed = state.rate_limiter.allow(
                    "ws_message",
                    &session_identity_id,
                    state.ws_message_rate_limit,
                    state.ws_message_rate_window_seconds,
                );
                if !allowed {
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to message rate limit"
                    );
                    let _ = socket
                        .send(Message::Text(build_error_event(
                            "event_rate_limited",
                            "too many websocket messages",
                        )))
                        .await;
                    break;
                }

                if bytes.len() > state.ws_max_inbound_message_bytes {
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to oversized binary payload"
                    );
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

pub(crate) fn route_inbound_event(raw: &str, session_identity_id: &str) -> String {
    crate::domain::events::service::route_inbound_event(raw, session_identity_id)
}

fn connection_ready_banner() -> String {
    crate::domain::events::service::connection_ready_banner()
}

fn build_error_event(code: &str, message: &str) -> String {
    crate::domain::events::service::build_error_event(code, message)
}

#[cfg(test)]
pub(crate) async fn is_session_valid(state: &AppState, headers: &HeaderMap) -> bool {
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
        Err(error) => {
            warn!(error = %error, "session validation upstream request failed");
            return None;
        }
    };

    if !response.status().is_success() {
        warn!(
            status = %response.status(),
            "session validation upstream returned non-success status"
        );
        return None;
    }

    let payload = match response.json::<SessionValidateResponse>().await {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "session validation upstream payload decode failed");
            return None;
        }
    };

    Some(ValidatedSession {
        identity_id: payload.identity_id,
    })
}

fn ws_rejection(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "code": code,
            "message": message,
        })),
    )
        .into_response()
}

fn websocket_rate_limit_key(
    state: &AppState,
    headers: &HeaderMap,
    peer_addr: Option<SocketAddr>,
) -> String {
    if let Some(source) = request_source_fingerprint(state, headers) {
        return format!("src:{:016x}", stable_hash(&source));
    }

    if let Some(peer_addr) = peer_addr {
        return format!("peer:{:016x}", stable_hash(&peer_addr.ip().to_string()));
    }

    "src:unknown".to_string()
}

fn request_source_fingerprint(state: &AppState, headers: &HeaderMap) -> Option<String> {
    if !state.trust_proxy_headers {
        return None;
    }

    if let Some(value) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(format!("xff:{}", value));
    }

    if let Some(value) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(format!("xri:{}", value));
    }

    None
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
