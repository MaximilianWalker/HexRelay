use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use futures::{stream::StreamExt, SinkExt};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;
use uuid::Uuid;

use crate::state::{AppState, ConnectionSenderEntry};

const MAX_DEVICE_ID_LEN: usize = 64;

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

    let device_id = websocket_device_id(&headers);

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

    let identity_id = session.identity_id;
    let connection_id = Uuid::new_v4().to_string();
    let state_for_upgrade = state.clone();
    let state_for_failed_upgrade = state.clone();
    let identity_for_failed_upgrade = identity_id.clone();

    ws.on_failed_upgrade(move |error| {
        let state = state_for_failed_upgrade.clone();
        let identity_id = identity_for_failed_upgrade.clone();
        warn!(
            identity_id = %identity_id,
            error = %error,
            "failed websocket upgrade after acquiring connection slot"
        );
        tokio::spawn(async move {
            release_connection_slot_after_failed_upgrade(state, identity_id).await;
        });
    })
    .on_upgrade(move |socket| {
        handle_socket(
            state_for_upgrade,
            socket,
            identity_id,
            connection_id,
            device_id,
        )
    })
}

async fn handle_socket(
    state: AppState,
    socket: WebSocket,
    session_identity_id: String,
    connection_id: String,
    device_id: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (outbound_tx, mut outbound_rx) = mpsc::channel::<String>(64);

    register_connection_sender(
        &state,
        &session_identity_id,
        &connection_id,
        outbound_tx.clone(),
        device_id.clone(),
    )
    .await;

    let writer = tokio::spawn(async move {
        while let Some(payload) = outbound_rx.recv().await {
            if sender.send(Message::Text(payload)).await.is_err() {
                break;
            }
        }
    });

    let _ = outbound_tx.try_send(connection_ready_banner());
    crate::domain::presence::hydrate_presence_backlog_if_needed(
        &state,
        &session_identity_id,
        device_id.as_deref(),
        &outbound_tx,
    )
    .await;
    crate::domain::presence::publish_online_if_needed(&state, &session_identity_id).await;

    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                if text.len() > state.ws_max_inbound_message_bytes {
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to oversized text payload"
                    );
                    let _ = outbound_tx.try_send(build_error_event(
                        "event_too_large",
                        "inbound message exceeds max size",
                    ));
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
                    let _ = outbound_tx.try_send(build_error_event(
                        "event_rate_limited",
                        "too many websocket messages",
                    ));
                    break;
                }

                let response = route_inbound_event(&text, &session_identity_id);
                let _ = outbound_tx.try_send(response);
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
                    let _ = outbound_tx.try_send(build_error_event(
                        "event_rate_limited",
                        "too many websocket messages",
                    ));
                    break;
                }

                if bytes.len() > state.ws_max_inbound_message_bytes {
                    warn!(
                        identity_id = %session_identity_id,
                        "closed websocket due to oversized binary payload"
                    );
                    let _ = outbound_tx.try_send(build_error_event(
                        "event_too_large",
                        "inbound message exceeds max size",
                    ));
                    break;
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    unregister_connection_sender(&state, &session_identity_id, &connection_id).await;
    crate::domain::presence::publish_offline_if_needed(&state, &session_identity_id).await;
    release_connection_slot(&state, &session_identity_id).await;
    drop(outbound_tx);
    let _ = writer.await;
}

#[derive(Deserialize)]
struct SessionValidateResponse {
    #[serde(rename = "session_id")]
    _session_id: String,
    #[serde(rename = "identity_id")]
    identity_id: String,
    #[serde(rename = "expires_at")]
    expires_at: String,
}

#[derive(Clone)]
struct ValidatedSession {
    identity_id: String,
    expires_at: DateTime<Utc>,
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

fn websocket_device_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-hexrelay-device-id")
        .and_then(|value| value.to_str().ok())
        .and_then(validate_device_id)
}

fn validate_device_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_DEVICE_ID_LEN {
        return None;
    }

    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return None;
    }

    Some(trimmed.to_owned())
}

#[cfg(test)]
pub(crate) async fn is_session_valid(state: &AppState, headers: &HeaderMap) -> bool {
    validate_session(state, headers).await.is_some()
}

async fn validate_session(state: &AppState, headers: &HeaderMap) -> Option<ValidatedSession> {
    let cache_key = session_cache_key(headers);

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

    match validate_session_upstream(state, cookie_header, authorization_header).await {
        UpstreamSessionValidation::Authorized(session) => {
            if state.ws_auth_grace_seconds > 0 {
                if let Some(cache_key) = cache_key {
                    cache_validated_session(
                        state,
                        cache_key,
                        session.identity_id.clone(),
                        session.expires_at,
                    )
                    .await;
                }
            }
            Some(session)
        }
        UpstreamSessionValidation::Denied => {
            if let Some(cache_key) = cache_key {
                remove_cached_session(state, &cache_key).await;
            }
            None
        }
        UpstreamSessionValidation::Unavailable => {
            let cache_key = cache_key?;
            load_cached_session(state, &cache_key).await
        }
    }
}

enum UpstreamSessionValidation {
    Authorized(ValidatedSession),
    Denied,
    Unavailable,
}

async fn validate_session_upstream(
    state: &AppState,
    cookie_header: Option<&str>,
    authorization_header: Option<&str>,
) -> UpstreamSessionValidation {
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
            return UpstreamSessionValidation::Unavailable;
        }
    };

    if response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN {
        warn!(
            status = %response.status(),
            "session validation upstream returned non-success status"
        );
        return UpstreamSessionValidation::Denied;
    }

    if response.status().is_server_error() {
        warn!(
            status = %response.status(),
            "session validation upstream returned unavailable status"
        );
        return UpstreamSessionValidation::Unavailable;
    }

    if !response.status().is_success() {
        warn!(
            status = %response.status(),
            "session validation upstream returned non-success status"
        );
        return UpstreamSessionValidation::Denied;
    }

    let payload = match response.json::<SessionValidateResponse>().await {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "session validation upstream payload decode failed");
            return UpstreamSessionValidation::Denied;
        }
    };

    let expires_at = match DateTime::parse_from_rfc3339(&payload.expires_at) {
        Ok(value) => value.with_timezone(&Utc),
        Err(error) => {
            warn!(error = %error, "session validation upstream expires_at decode failed");
            return UpstreamSessionValidation::Denied;
        }
    };

    if Utc::now() > expires_at {
        warn!(expires_at = %expires_at, "session validation upstream returned expired session");
        return UpstreamSessionValidation::Denied;
    }

    UpstreamSessionValidation::Authorized(ValidatedSession {
        identity_id: payload.identity_id,
        expires_at,
    })
}

async fn cache_validated_session(
    state: &AppState,
    key: String,
    identity_id: String,
    expires_at: DateTime<Utc>,
) {
    let mut guard = state.validated_session_cache.lock().await;

    let is_new_key = !guard.contains_key(&key);
    if is_new_key && guard.len() >= state.ws_auth_cache_max_entries {
        if let Some((oldest_key, _)) = guard
            .iter()
            .min_by_key(|(_, value)| value.validated_at)
            .map(|(cache_key, value)| (cache_key.clone(), value.validated_at))
        {
            guard.remove(&oldest_key);
        }
    }

    guard.insert(
        key,
        crate::state::CachedSession {
            identity_id,
            expires_at,
            validated_at: tokio::time::Instant::now(),
        },
    );
}

async fn remove_cached_session(state: &AppState, key: &str) {
    state.validated_session_cache.lock().await.remove(key);
}

async fn load_cached_session(state: &AppState, key: &str) -> Option<ValidatedSession> {
    if state.ws_auth_grace_seconds == 0 {
        return None;
    }

    let max_age = Duration::from_secs(state.ws_auth_grace_seconds);
    let mut guard = state.validated_session_cache.lock().await;
    let cached = guard.get(key).cloned()?;

    if cached.validated_at.elapsed() > max_age || Utc::now() > cached.expires_at {
        guard.remove(key);
        return None;
    }

    drop(guard);

    Some(ValidatedSession {
        identity_id: cached.identity_id,
        expires_at: cached.expires_at,
    })
}

fn session_cache_key(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(format!("auth:{:016x}", stable_hash(value)));
    }

    let session_cookie = headers
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(extract_session_cookie)
        .map(str::to_string)?;
    Some(format!("cookie:{:016x}", stable_hash(&session_cookie)))
}

fn extract_session_cookie(raw_cookie_header: &str) -> Option<&str> {
    raw_cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("hexrelay_session="))
        .map(str::trim)
        .filter(|value| !value.is_empty())
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
    let digest = Sha256::digest(value.as_bytes());
    let mut first_eight = [0_u8; 8];
    first_eight.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(first_eight)
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

async fn register_connection_sender(
    state: &AppState,
    identity_id: &str,
    connection_id: &str,
    sender: mpsc::Sender<String>,
    device_id: Option<String>,
) {
    let mut guard = state.connection_senders.lock().await;
    guard.entry(identity_id.to_string()).or_default().insert(
        connection_id.to_string(),
        ConnectionSenderEntry { sender, device_id },
    );
}

async fn unregister_connection_sender(state: &AppState, identity_id: &str, connection_id: &str) {
    let mut guard = state.connection_senders.lock().await;
    if let Some(connections) = guard.get_mut(identity_id) {
        connections.remove(connection_id);
        if connections.is_empty() {
            guard.remove(identity_id);
        }
    }
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

async fn release_connection_slot_after_failed_upgrade(state: AppState, identity_id: String) {
    release_connection_slot(&state, &identity_id).await;
}

#[cfg(test)]
mod tests {
    use super::{
        cache_validated_session, release_connection_slot_after_failed_upgrade, stable_hash,
    };
    use crate::state::AppState;
    use chrono::{Duration as ChronoDuration, Utc};
    use tokio::time::{sleep, Duration};

    #[test]
    fn stable_hash_is_deterministic_across_processes() {
        assert_eq!(stable_hash("test-value"), 6_562_878_253_510_288_723);
    }

    #[tokio::test]
    async fn cache_eviction_respects_max_entries_bound() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
            None,
            false,
            60,
            60,
            16384,
            120,
            60,
            3,
            30,
            1,
        )
        .expect("build state");

        cache_validated_session(
            &state,
            "k1".to_string(),
            "u1".to_string(),
            Utc::now() + ChronoDuration::minutes(5),
        )
        .await;
        sleep(Duration::from_millis(1)).await;
        cache_validated_session(
            &state,
            "k2".to_string(),
            "u2".to_string(),
            Utc::now() + ChronoDuration::minutes(5),
        )
        .await;

        let cache = state.validated_session_cache.lock().await;
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains_key("k1"));
        assert!(cache.contains_key("k2"));
    }

    #[tokio::test]
    async fn failed_upgrade_release_removes_connection_slot() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
            None,
            false,
            60,
            60,
            16384,
            120,
            60,
            1,
            30,
            10000,
        )
        .expect("build state");

        state
            .active_connections
            .lock()
            .await
            .insert("usr-1".to_string(), 1);

        release_connection_slot_after_failed_upgrade(state.clone(), "usr-1".to_string()).await;

        assert!(state.active_connections.lock().await.get("usr-1").is_none());
    }
}
