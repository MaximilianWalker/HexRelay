use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::ConnectInfo,
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
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

use crate::state::{AppState, ConnectionSenderEntry, DevFaultConfig, DevFaultState};

const MAX_DEVICE_ID_LEN: usize = 64;
const MIN_DEVICE_SECRET_LEN: usize = 16;
const MAX_DEVICE_SECRET_LEN: usize = 128;
const DROP_DEBT_EPSILON: f64 = 1.0e-12;

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ws_handler(
    State(state): State<AppState>,
    ConnectInfo(peer_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    uri: Uri,
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

    if apply_dev_fault(&state).await {
        warn!("rejected websocket upgrade due to dev fault drop");
        return ws_rejection(
            StatusCode::SERVICE_UNAVAILABLE,
            "dev_fault_drop",
            "dev fault injection dropped websocket upgrade",
        );
    }

    let auth_context = SessionAuthContext::from_headers(&headers);
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

    let device_id = websocket_device_id(&headers, uri.query());
    let device_secret = websocket_device_secret(&headers);

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
            device_secret,
            auth_context,
        )
    })
}

async fn handle_socket(
    state: AppState,
    socket: WebSocket,
    session_identity_id: String,
    connection_id: String,
    device_id: Option<String>,
    initial_device_secret: Option<String>,
    auth_context: SessionAuthContext,
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
    if let (Some(device_id), Some(device_secret)) =
        (device_id.as_deref(), initial_device_secret.as_deref())
    {
        let (response, verified) = crate::app::dms::verify_dm_device_binding(
            &state,
            &session_identity_id,
            device_id,
            device_secret,
            None,
        )
        .await;
        if verified {
            mark_connection_dm_device_verified(&state, &session_identity_id, &connection_id).await;
        }
        let _ = outbound_tx.try_send(response);
    }
    crate::app::presence::hydrate_presence_backlog_if_needed(
        &state,
        &session_identity_id,
        device_id.as_deref(),
        &outbound_tx,
    )
    .await;
    crate::app::channels::hydrate_channel_backlog_if_needed(
        &state,
        &session_identity_id,
        device_id.as_deref(),
        &outbound_tx,
    )
    .await;
    crate::app::presence::publish_online_if_needed(&state, &session_identity_id).await;

    let connected_at = tokio::time::Instant::now();
    let mut dev_fault_tick = tokio::time::interval(Duration::from_millis(250));
    dev_fault_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        if state.enable_dev_faults {
            tokio::select! {
                _ = dev_fault_tick.tick() => {
                    if !dev_fault_disconnect_due(&state, connected_at).await {
                        continue;
                    }

                    warn!(identity_id = %session_identity_id, "closed websocket due to dev fault disconnect timer");
                    let _ = outbound_tx.try_send(build_error_event(
                        "dev_fault_disconnect",
                        "dev fault injection closed websocket",
                    ));
                    break;
                }
                message = receiver.next() => {
                    let Some(message) = message else { break; };
                    if !handle_inbound_message(
                        &state,
                        &session_identity_id,
                        &connection_id,
                        device_id.as_deref(),
                        &auth_context,
                        &outbound_tx,
                        message,
                    )
                    .await
                    {
                        break;
                    }
                }
            }
        } else {
            let Some(message) = receiver.next().await else {
                break;
            };
            if !handle_inbound_message(
                &state,
                &session_identity_id,
                &connection_id,
                device_id.as_deref(),
                &auth_context,
                &outbound_tx,
                message,
            )
            .await
            {
                break;
            }
        }
    }

    unregister_connection_sender(&state, &session_identity_id, &connection_id).await;
    crate::app::presence::publish_offline_if_needed(&state, &session_identity_id).await;
    release_connection_slot(&state, &session_identity_id).await;
    drop(outbound_tx);
    let _ = writer.await;
}

async fn dev_fault_disconnect_due(state: &AppState, connected_at: tokio::time::Instant) -> bool {
    current_dev_fault_config(state)
        .await
        .disconnect_after_seconds
        .map(|seconds| connected_at.elapsed() >= Duration::from_secs(seconds))
        .unwrap_or(false)
}

async fn handle_inbound_message(
    state: &AppState,
    session_identity_id: &str,
    connection_id: &str,
    device_id: Option<&str>,
    auth_context: &SessionAuthContext,
    outbound_tx: &mpsc::Sender<String>,
    message: Result<Message, axum::Error>,
) -> bool {
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
                return false;
            }

            if !message_rate_allowed(state, session_identity_id, outbound_tx) {
                return false;
            }

            if apply_dev_fault(state).await {
                warn!(identity_id = %session_identity_id, "dropped websocket text message due to dev fault");
                return true;
            }

            if crate::app::dms::is_dm_device_proof_event(&text) {
                let (response, verified) = crate::app::dms::handle_dm_device_proof(
                    state,
                    session_identity_id,
                    device_id,
                    &text,
                )
                .await;
                if verified {
                    mark_connection_dm_device_verified(state, session_identity_id, connection_id)
                        .await;
                }
                let _ = outbound_tx.try_send(response);
            } else if crate::app::dms::is_dm_envelope_ack_event(&text) {
                let dm_device_verified = connection_dm_device_verified(
                    state,
                    session_identity_id,
                    connection_id,
                    device_id,
                )
                .await;
                let response = crate::app::dms::handle_dm_envelope_ack(
                    state,
                    session_identity_id,
                    device_id,
                    dm_device_verified,
                    &text,
                )
                .await;
                let _ = outbound_tx.try_send(response);
            } else {
                let routed =
                    crate::domain::events::service::route_inbound_event(&text, session_identity_id);
                if let Some(delivery) = routed.recipient_delivery.as_ref() {
                    if !signal_target_rate_allowed(
                        state,
                        session_identity_id,
                        &delivery.target_identity_id,
                        outbound_tx,
                    ) {
                        return true;
                    }
                    if !signaling_recipient_authorized(
                        state,
                        auth_context,
                        &delivery.target_identity_id,
                    )
                    .await
                    {
                        let _ = outbound_tx.try_send(build_error_event(
                            "event_forbidden",
                            "signaling recipient is not an accepted contact",
                        ));
                        return true;
                    }
                    dispatch_recipient_signal(
                        state,
                        &delivery.target_identity_id,
                        &delivery.payload,
                    )
                    .await;
                }
                let _ = outbound_tx.try_send(routed.sender_response);
            }
            true
        }
        Ok(Message::Binary(bytes)) => {
            if !message_rate_allowed(state, session_identity_id, outbound_tx) {
                return false;
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
                return false;
            }

            if apply_dev_fault(state).await {
                warn!(identity_id = %session_identity_id, "dropped websocket binary message due to dev fault");
            }
            true
        }
        Ok(Message::Close(_)) => false,
        Ok(_) => true,
        Err(_) => false,
    }
}

fn message_rate_allowed(
    state: &AppState,
    session_identity_id: &str,
    outbound_tx: &mpsc::Sender<String>,
) -> bool {
    let allowed = state.rate_limiter.allow(
        "ws_message",
        session_identity_id,
        state.ws_message_rate_limit,
        state.ws_message_rate_window_seconds,
    );
    if allowed {
        return true;
    }

    warn!(
        identity_id = %session_identity_id,
        "closed websocket due to message rate limit"
    );
    let _ = outbound_tx.try_send(build_error_event(
        "event_rate_limited",
        "too many websocket messages",
    ));
    false
}

fn signal_target_rate_allowed(
    state: &AppState,
    session_identity_id: &str,
    target_identity_id: &str,
    outbound_tx: &mpsc::Sender<String>,
) -> bool {
    let rate_key = format!("{session_identity_id}->{target_identity_id}");
    let allowed = state.rate_limiter.allow(
        "ws_signal_target",
        &rate_key,
        state.ws_message_rate_limit,
        state.ws_message_rate_window_seconds,
    );
    if allowed {
        return true;
    }

    warn!(
        identity_id = %session_identity_id,
        target_identity_id = %target_identity_id,
        "rejected recipient-targeted signaling due to sender-recipient rate limit"
    );
    let _ = outbound_tx.try_send(build_error_event(
        "event_rate_limited",
        "too many websocket signaling messages to recipient",
    ));
    false
}

async fn current_dev_fault_config(state: &AppState) -> DevFaultConfig {
    if !state.enable_dev_faults {
        return DevFaultConfig::default();
    }

    state.dev_faults.lock().await.config.clone()
}

async fn apply_dev_fault(state: &AppState) -> bool {
    if !state.enable_dev_faults {
        return false;
    }

    let config = current_dev_fault_config(state).await;
    if config.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(config.delay_ms)).await;
    }

    let mut faults = state.dev_faults.lock().await;
    should_drop_dev_fault(&mut faults)
}

fn should_drop_dev_fault(faults: &mut DevFaultState) -> bool {
    let drop_rate = faults.config.drop_rate;
    if drop_rate <= 0.0 {
        faults.drop_debt = 0.0;
        return false;
    }
    if drop_rate >= 1.0 {
        faults.drop_debt = 0.0;
        return true;
    }

    faults.drop_debt += drop_rate;
    if faults.drop_debt + DROP_DEBT_EPSILON < 1.0 {
        return false;
    }

    faults.drop_debt = (faults.drop_debt - 1.0).max(0.0);
    true
}

#[derive(Clone, Default)]
struct SessionAuthContext {
    cookie_header: Option<String>,
    authorization_header: Option<String>,
}

impl SessionAuthContext {
    fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            cookie_header: headers
                .get("cookie")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            authorization_header: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        }
    }

    fn apply(&self, mut request: reqwest::RequestBuilder) -> Option<reqwest::RequestBuilder> {
        if let Some(cookie_header) = self.cookie_header.as_deref() {
            request = request.header("cookie", cookie_header);
        }
        if let Some(authorization_header) = self.authorization_header.as_deref() {
            request = request.header("authorization", authorization_header);
        }
        if self.cookie_header.is_none() && self.authorization_header.is_none() {
            return None;
        }

        Some(request)
    }
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

#[derive(Deserialize)]
struct ContactListResponse {
    items: Vec<ContactSummary>,
}

#[derive(Deserialize)]
struct ContactSummary {
    id: String,
    #[serde(default)]
    inbound_request: bool,
    #[serde(default)]
    pending_request: bool,
}

#[cfg(test)]
pub(crate) fn route_inbound_event(raw: &str, session_identity_id: &str) -> String {
    crate::domain::events::service::route_inbound_event(raw, session_identity_id).sender_response
}

fn connection_ready_banner() -> String {
    crate::domain::events::service::connection_ready_banner()
}

fn build_error_event(code: &str, message: &str) -> String {
    crate::domain::events::service::build_error_event(code, message)
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SignalDispatchSummary {
    queued_count: usize,
    saturated_count: usize,
    stale_count: usize,
    no_connection: bool,
}

async fn signaling_recipient_authorized(
    state: &AppState,
    auth_context: &SessionAuthContext,
    target_identity_id: &str,
) -> bool {
    let url = format!("{}/contacts", state.api_base_url.trim_end_matches('/'));
    let Some(request) = auth_context.apply(state.http_client.get(url)) else {
        return false;
    };

    let response = match request.send().await {
        Ok(response) => response,
        Err(error) => {
            warn!(error = %error, "signaling contact authorization request failed");
            return false;
        }
    };

    if !response.status().is_success() {
        warn!(
            status = %response.status(),
            "signaling contact authorization returned non-success status"
        );
        return false;
    }

    let contacts = match response.json::<ContactListResponse>().await {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "signaling contact authorization payload decode failed");
            return false;
        }
    };

    contacts.items.into_iter().any(|contact| {
        contact.id == target_identity_id && !contact.inbound_request && !contact.pending_request
    })
}

async fn dispatch_recipient_signal(
    state: &AppState,
    recipient_identity_id: &str,
    payload: &str,
) -> SignalDispatchSummary {
    let mut guard = state.connection_senders.lock().await;
    let Some(connections) = guard.get_mut(recipient_identity_id) else {
        return SignalDispatchSummary {
            no_connection: true,
            ..SignalDispatchSummary::default()
        };
    };

    let mut stale_connection_ids = Vec::new();
    let mut queued_count = 0usize;
    let mut saturated_count = 0usize;
    for (connection_id, entry) in connections.iter() {
        match entry.sender.try_send(payload.to_owned()) {
            Ok(()) => queued_count += 1,
            Err(mpsc::error::TrySendError::Full(_)) => saturated_count += 1,
            Err(mpsc::error::TrySendError::Closed(_)) => {
                stale_connection_ids.push(connection_id.clone());
            }
        }
    }

    for connection_id in &stale_connection_ids {
        connections.remove(connection_id);
    }
    if connections.is_empty() {
        guard.remove(recipient_identity_id);
    }

    if queued_count == 0 && (saturated_count > 0 || !stale_connection_ids.is_empty()) {
        warn!(
            recipient_identity_id = %recipient_identity_id,
            saturated_count,
            stale_count = stale_connection_ids.len(),
            "recipient-targeted signaling could not queue to any active websocket"
        );
    }

    SignalDispatchSummary {
        queued_count,
        saturated_count,
        stale_count: stale_connection_ids.len(),
        no_connection: false,
    }
}

fn websocket_device_id(headers: &HeaderMap, query: Option<&str>) -> Option<String> {
    headers
        .get("x-hexrelay-device-id")
        .and_then(|value| value.to_str().ok())
        .and_then(validate_device_id)
        .or_else(|| query.and_then(websocket_query_device_id))
}

fn websocket_query_device_id(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "device_id" {
            validate_device_id(value)
        } else {
            None
        }
    })
}

fn websocket_device_secret(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-hexrelay-device-secret")
        .and_then(|value| value.to_str().ok())
        .and_then(validate_device_secret)
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

fn validate_device_secret(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() < MIN_DEVICE_SECRET_LEN || trimmed.len() > MAX_DEVICE_SECRET_LEN {
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
    let url = format!(
        "{}/auth/sessions/validate",
        state.api_base_url.trim_end_matches('/')
    );
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
        ConnectionSenderEntry {
            sender,
            device_id,
            dm_device_verified: false,
        },
    );
}

async fn mark_connection_dm_device_verified(
    state: &AppState,
    identity_id: &str,
    connection_id: &str,
) {
    let mut guard = state.connection_senders.lock().await;
    if let Some(entry) = guard
        .get_mut(identity_id)
        .and_then(|connections| connections.get_mut(connection_id))
    {
        entry.dm_device_verified = true;
    }
}

async fn connection_dm_device_verified(
    state: &AppState,
    identity_id: &str,
    connection_id: &str,
    device_id: Option<&str>,
) -> bool {
    let Some(device_id) = device_id else {
        return false;
    };
    let guard = state.connection_senders.lock().await;
    guard
        .get(identity_id)
        .and_then(|connections| connections.get(connection_id))
        .is_some_and(|entry| {
            entry.dm_device_verified && entry.device_id.as_deref() == Some(device_id)
        })
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
        cache_validated_session, dispatch_recipient_signal,
        release_connection_slot_after_failed_upgrade, should_drop_dev_fault, stable_hash,
        websocket_device_id, websocket_device_secret, SignalDispatchSummary,
    };
    use crate::state::{AppState, ConnectionSenderEntry, DevFaultConfig, DevFaultState};
    use axum::http::{HeaderMap, HeaderValue};
    use chrono::{Duration as ChronoDuration, Utc};
    use std::collections::HashMap;
    use tokio::sync::mpsc;
    use tokio::time::{sleep, Duration};

    fn test_state() -> AppState {
        AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
            None,
            false,
            60,
            60,
            16384,
            120,
            60,
            3,
            30,
            10000,
        )
        .expect("build state")
    }

    #[test]
    fn stable_hash_is_deterministic_across_processes() {
        assert_eq!(stable_hash("test-value"), 6_562_878_253_510_288_723);
    }

    #[test]
    fn dev_fault_drop_accumulator_matches_fractional_rate() {
        let mut faults = DevFaultState {
            config: DevFaultConfig {
                delay_ms: 0,
                drop_rate: 0.4,
                disconnect_after_seconds: None,
            },
            drop_debt: 0.0,
        };

        let drops = (0..10)
            .filter(|_| should_drop_dev_fault(&mut faults))
            .count();

        assert_eq!(drops, 4);
    }

    #[test]
    fn dev_fault_drop_accumulator_preserves_high_rates() {
        let mut faults = DevFaultState {
            config: DevFaultConfig {
                delay_ms: 0,
                drop_rate: 0.9,
                disconnect_after_seconds: None,
            },
            drop_debt: 0.0,
        };

        let decisions = (0..10)
            .map(|_| should_drop_dev_fault(&mut faults))
            .collect::<Vec<_>>();

        assert_eq!(decisions.iter().filter(|drop| **drop).count(), 9);
        assert!(decisions.iter().any(|drop| !*drop));
    }

    #[test]
    fn websocket_device_id_accepts_query_for_browser_clients() {
        let headers = HeaderMap::new();

        assert_eq!(
            websocket_device_id(&headers, Some("device_id=web-main")),
            Some("web-main".to_string())
        );
    }

    #[test]
    fn websocket_device_id_prefers_header_over_query() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-hexrelay-device-id",
            HeaderValue::from_static("native-main"),
        );

        assert_eq!(
            websocket_device_id(&headers, Some("device_id=web-main")),
            Some("native-main".to_string())
        );
    }

    #[test]
    fn websocket_device_id_rejects_invalid_query_value() {
        let headers = HeaderMap::new();

        assert_eq!(
            websocket_device_id(&headers, Some("device_id=bad%2Fdevice")),
            None
        );
    }

    #[test]
    fn websocket_device_secret_accepts_header_for_native_clients() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-hexrelay-device-secret",
            HeaderValue::from_static("secret-native-main"),
        );

        assert_eq!(
            websocket_device_secret(&headers),
            Some("secret-native-main".to_string())
        );
    }

    #[test]
    fn websocket_device_secret_rejects_query_for_browser_clients() {
        let headers = HeaderMap::new();

        assert_eq!(websocket_device_secret(&headers), None);
    }

    #[test]
    fn websocket_device_secret_rejects_short_or_non_url_safe_header_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-hexrelay-device-secret",
            HeaderValue::from_static("short"),
        );

        assert_eq!(websocket_device_secret(&headers), None);
        headers.insert(
            "x-hexrelay-device-secret",
            HeaderValue::from_static("bad/secret-value"),
        );
        assert_eq!(websocket_device_secret(&headers), None);
    }

    #[tokio::test]
    async fn cache_eviction_respects_max_entries_bound() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
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
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
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

    #[tokio::test]
    async fn dispatch_recipient_signal_queues_to_all_active_connections() {
        let state = test_state();
        let (first_tx, mut first_rx) = mpsc::channel::<String>(1);
        let (second_tx, mut second_rx) = mpsc::channel::<String>(1);
        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            HashMap::from([
                (
                    "conn-1".to_string(),
                    ConnectionSenderEntry {
                        sender: first_tx,
                        device_id: None,
                        dm_device_verified: false,
                    },
                ),
                (
                    "conn-2".to_string(),
                    ConnectionSenderEntry {
                        sender: second_tx,
                        device_id: None,
                        dm_device_verified: false,
                    },
                ),
            ]),
        );

        let summary = dispatch_recipient_signal(&state, "usr-recipient", "signal-payload").await;

        assert_eq!(
            summary,
            SignalDispatchSummary {
                queued_count: 2,
                saturated_count: 0,
                stale_count: 0,
                no_connection: false,
            }
        );
        assert_eq!(first_rx.recv().await.as_deref(), Some("signal-payload"));
        assert_eq!(second_rx.recv().await.as_deref(), Some("signal-payload"));
    }

    #[tokio::test]
    async fn dispatch_recipient_signal_reports_offline_recipient() {
        let state = test_state();

        let summary = dispatch_recipient_signal(&state, "usr-recipient", "signal-payload").await;

        assert_eq!(
            summary,
            SignalDispatchSummary {
                no_connection: true,
                ..SignalDispatchSummary::default()
            }
        );
    }

    #[tokio::test]
    async fn dispatch_recipient_signal_keeps_full_connections_registered() {
        let state = test_state();
        let (full_tx, mut full_rx) = mpsc::channel::<String>(1);
        full_tx
            .try_send("seed".to_string())
            .expect("fill recipient queue");
        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            HashMap::from([(
                "conn-full".to_string(),
                ConnectionSenderEntry {
                    sender: full_tx,
                    device_id: None,
                    dm_device_verified: false,
                },
            )]),
        );

        let summary = dispatch_recipient_signal(&state, "usr-recipient", "signal-payload").await;

        assert_eq!(
            summary,
            SignalDispatchSummary {
                queued_count: 0,
                saturated_count: 1,
                stale_count: 0,
                no_connection: false,
            }
        );
        assert!(state
            .connection_senders
            .lock()
            .await
            .get("usr-recipient")
            .and_then(|connections| connections.get("conn-full"))
            .is_some());
        assert_eq!(full_rx.recv().await.as_deref(), Some("seed"));
    }

    #[tokio::test]
    async fn dispatch_recipient_signal_removes_stale_connection() {
        let state = test_state();
        let (stale_tx, stale_rx) = mpsc::channel::<String>(1);
        drop(stale_rx);
        state.connection_senders.lock().await.insert(
            "usr-recipient".to_string(),
            HashMap::from([(
                "conn-stale".to_string(),
                ConnectionSenderEntry {
                    sender: stale_tx,
                    device_id: None,
                    dm_device_verified: false,
                },
            )]),
        );

        let summary = dispatch_recipient_signal(&state, "usr-recipient", "signal-payload").await;

        assert_eq!(
            summary,
            SignalDispatchSummary {
                queued_count: 0,
                saturated_count: 0,
                stale_count: 1,
                no_connection: false,
            }
        );
        assert!(!state
            .connection_senders
            .lock()
            .await
            .contains_key("usr-recipient"));
    }
}
