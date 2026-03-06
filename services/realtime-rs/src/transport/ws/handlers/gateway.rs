use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use futures::stream::StreamExt;
use serde::Deserialize;
use std::hash::{Hash, Hasher};

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
