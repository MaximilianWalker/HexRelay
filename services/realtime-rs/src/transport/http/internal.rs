use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    app::AppState,
    domain::channels::{
        publish_channel_message_created, publish_channel_message_deleted,
        publish_channel_message_updated, PublishChannelMessageCreatedInput,
        PublishChannelMessageDeletedInput, PublishChannelMessageUpdatedInput,
    },
    state::DevFaultConfig,
};

const MAX_DEV_FAULT_DELAY_MS: u64 = 600_000;
const MAX_DEV_FAULT_DISCONNECT_SECONDS: u64 = 86_400;

#[derive(Deserialize)]
pub struct ChannelMessageCreatedDispatchRequest {
    pub message_id: String,
    #[serde(alias = "guild_id")]
    pub server_id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub created_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Deserialize)]
pub struct ChannelMessageUpdatedDispatchRequest {
    pub message_id: String,
    #[serde(alias = "guild_id")]
    pub server_id: String,
    pub channel_id: String,
    pub editor_id: String,
    pub edited_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Deserialize)]
pub struct ChannelMessageDeletedDispatchRequest {
    pub message_id: String,
    #[serde(alias = "guild_id")]
    pub server_id: String,
    pub channel_id: String,
    pub deleted_by: String,
    pub deleted_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Deserialize)]
pub struct DevFaultUpdateRequest {
    pub delay_ms: Option<u64>,
    pub drop_rate: Option<f64>,
    pub disconnect_after_seconds: Option<u64>,
}

#[derive(Serialize)]
pub struct DevFaultResponse {
    pub enabled: bool,
    pub delay_ms: u64,
    pub drop_rate: f64,
    pub disconnect_after_seconds: Option<u64>,
}

pub async fn publish_channel_message_created_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChannelMessageCreatedDispatchRequest>,
) -> StatusCode {
    if !internal_token_valid(&state, &headers) {
        return StatusCode::UNAUTHORIZED;
    }

    match publish_channel_message_created(
        &state,
        PublishChannelMessageCreatedInput {
            message_id: payload.message_id,
            guild_id: payload.server_id,
            channel_id: payload.channel_id,
            sender_id: payload.sender_id,
            created_at: Some(payload.created_at),
            channel_seq: payload.channel_seq,
            recipients: payload.recipients,
        },
    )
    .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::BAD_GATEWAY,
    }
}

pub async fn publish_channel_message_updated_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChannelMessageUpdatedDispatchRequest>,
) -> StatusCode {
    if !internal_token_valid(&state, &headers) {
        return StatusCode::UNAUTHORIZED;
    }

    match publish_channel_message_updated(
        &state,
        PublishChannelMessageUpdatedInput {
            message_id: payload.message_id,
            guild_id: payload.server_id,
            channel_id: payload.channel_id,
            editor_id: payload.editor_id,
            edited_at: Some(payload.edited_at),
            channel_seq: payload.channel_seq,
            recipients: payload.recipients,
        },
    )
    .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::BAD_GATEWAY,
    }
}

pub async fn publish_channel_message_deleted_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChannelMessageDeletedDispatchRequest>,
) -> StatusCode {
    if !internal_token_valid(&state, &headers) {
        return StatusCode::UNAUTHORIZED;
    }

    match publish_channel_message_deleted(
        &state,
        PublishChannelMessageDeletedInput {
            message_id: payload.message_id,
            guild_id: payload.server_id,
            channel_id: payload.channel_id,
            deleted_by: payload.deleted_by,
            deleted_at: Some(payload.deleted_at),
            channel_seq: payload.channel_seq,
            recipients: payload.recipients,
        },
    )
    .await
    {
        Ok(()) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::BAD_GATEWAY,
    }
}

pub async fn get_dev_faults_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> (StatusCode, Json<DevFaultResponse>) {
    if !internal_token_valid(&state, &headers) {
        return dev_fault_response(StatusCode::UNAUTHORIZED, false, DevFaultConfig::default());
    }
    if !state.enable_dev_faults {
        return dev_fault_response(StatusCode::FORBIDDEN, false, DevFaultConfig::default());
    }

    let faults = state.dev_faults.lock().await;
    dev_fault_response(StatusCode::OK, true, faults.config.clone())
}

pub async fn set_dev_faults_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DevFaultUpdateRequest>,
) -> (StatusCode, Json<DevFaultResponse>) {
    if !internal_token_valid(&state, &headers) {
        return dev_fault_response(StatusCode::UNAUTHORIZED, false, DevFaultConfig::default());
    }
    if !state.enable_dev_faults {
        return dev_fault_response(StatusCode::FORBIDDEN, false, DevFaultConfig::default());
    }

    let Some(config) = validate_dev_fault_update(payload) else {
        let faults = state.dev_faults.lock().await;
        return dev_fault_response(StatusCode::BAD_REQUEST, true, faults.config.clone());
    };

    let mut faults = state.dev_faults.lock().await;
    faults.config = config.clone();
    faults.drop_counter = 0;
    dev_fault_response(StatusCode::OK, true, config)
}

pub async fn reset_dev_faults_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> (StatusCode, Json<DevFaultResponse>) {
    if !internal_token_valid(&state, &headers) {
        return dev_fault_response(StatusCode::UNAUTHORIZED, false, DevFaultConfig::default());
    }
    if !state.enable_dev_faults {
        return dev_fault_response(StatusCode::FORBIDDEN, false, DevFaultConfig::default());
    }

    let mut faults = state.dev_faults.lock().await;
    faults.config = DevFaultConfig::default();
    faults.drop_counter = 0;
    dev_fault_response(StatusCode::OK, true, faults.config.clone())
}

fn validate_dev_fault_update(payload: DevFaultUpdateRequest) -> Option<DevFaultConfig> {
    let delay_ms = payload.delay_ms.unwrap_or(0);
    let drop_rate = payload.drop_rate.unwrap_or(0.0);
    let disconnect_after_seconds = payload.disconnect_after_seconds;

    if delay_ms > MAX_DEV_FAULT_DELAY_MS {
        return None;
    }
    if !drop_rate.is_finite() || !(0.0..=1.0).contains(&drop_rate) {
        return None;
    }
    if matches!(disconnect_after_seconds, Some(value) if value == 0 || value > MAX_DEV_FAULT_DISCONNECT_SECONDS)
    {
        return None;
    }

    Some(DevFaultConfig {
        delay_ms,
        drop_rate,
        disconnect_after_seconds,
    })
}

fn dev_fault_response(
    status: StatusCode,
    enabled: bool,
    config: DevFaultConfig,
) -> (StatusCode, Json<DevFaultResponse>) {
    (
        status,
        Json(DevFaultResponse {
            enabled,
            delay_ms: config.delay_ms,
            drop_rate: config.drop_rate,
            disconnect_after_seconds: config.disconnect_after_seconds,
        }),
    )
}

fn internal_token_valid(state: &AppState, headers: &HeaderMap) -> bool {
    headers
        .get("x-hexrelay-internal-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        == Some(state.channel_dispatch_internal_token.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

    const TOKEN: &str = "test-channel-dispatch-token-12345";

    fn test_state(enable_dev_faults: bool) -> AppState {
        AppState::new(
            "http://127.0.0.1:8080".to_string(),
            vec!["http://127.0.0.1:3002".to_string()],
            TOKEN.to_string(),
            "test-presence-watcher-token-12345".to_string(),
            None,
            false,
            60,
            60,
            16_384,
            120,
            60,
            3,
            0,
            10_000,
        )
        .expect("build state")
        .with_dev_faults_enabled(enable_dev_faults)
    }

    #[tokio::test]
    async fn dev_faults_require_enable_flag_and_internal_token() {
        let disabled = get_dev_faults_internal(
            State(test_state(false)),
            HeaderMap::from_iter([(
                "x-hexrelay-internal-token".parse().unwrap(),
                TOKEN.parse().unwrap(),
            )]),
        )
        .await;
        assert_eq!(disabled.0, StatusCode::FORBIDDEN);

        let unauthorized = get_dev_faults_internal(State(test_state(true)), HeaderMap::new()).await;
        assert_eq!(unauthorized.0, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn dev_faults_can_be_set_and_reset_through_router() {
        let app = crate::app::build_app(test_state(true));
        let request = Request::builder()
            .method("POST")
            .uri("/internal/dev/faults")
            .header("x-hexrelay-internal-token", TOKEN)
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"delay_ms":25,"drop_rate":1.0,"disconnect_after_seconds":5}"#,
            ))
            .expect("build set request");
        let response = app.clone().oneshot(request).await.expect("set response");
        assert_eq!(response.status(), StatusCode::OK);

        let request = Request::builder()
            .method("GET")
            .uri("/internal/dev/faults")
            .header("x-hexrelay-internal-token", TOKEN)
            .body(Body::empty())
            .expect("build get request");
        let response = app.clone().oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode body");
        assert_eq!(payload["delay_ms"], 25);
        assert_eq!(payload["drop_rate"], 1.0);
        assert_eq!(payload["disconnect_after_seconds"], 5);

        let request = Request::builder()
            .method("POST")
            .uri("/internal/dev/faults/reset")
            .header("x-hexrelay-internal-token", TOKEN)
            .body(Body::empty())
            .expect("build reset request");
        let response = app.oneshot(request).await.expect("reset response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read reset body");
        let payload: serde_json::Value = serde_json::from_slice(&body).expect("decode reset body");
        assert_eq!(payload["delay_ms"], 0);
        assert_eq!(payload["drop_rate"], 0.0);
        assert!(payload["disconnect_after_seconds"].is_null());
    }

    #[tokio::test]
    async fn dev_faults_reject_invalid_values() {
        let response = set_dev_faults_internal(
            State(test_state(true)),
            HeaderMap::from_iter([(
                "x-hexrelay-internal-token".parse().unwrap(),
                TOKEN.parse().unwrap(),
            )]),
            Json(DevFaultUpdateRequest {
                delay_ms: Some(MAX_DEV_FAULT_DELAY_MS + 1),
                drop_rate: Some(0.5),
                disconnect_after_seconds: None,
            }),
        )
        .await;
        assert_eq!(response.0, StatusCode::BAD_REQUEST);
    }
}
