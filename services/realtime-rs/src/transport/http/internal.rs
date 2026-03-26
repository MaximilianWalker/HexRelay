use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Deserialize;

use crate::{
    app::AppState,
    domain::channels::{
        publish_channel_message_created, publish_channel_message_deleted,
        publish_channel_message_updated, PublishChannelMessageCreatedInput,
        PublishChannelMessageDeletedInput, PublishChannelMessageUpdatedInput,
    },
};

#[derive(Deserialize)]
pub struct ChannelMessageCreatedDispatchRequest {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub created_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Deserialize)]
pub struct ChannelMessageUpdatedDispatchRequest {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub editor_id: String,
    pub edited_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Deserialize)]
pub struct ChannelMessageDeletedDispatchRequest {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub deleted_by: String,
    pub deleted_at: String,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
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
            guild_id: payload.guild_id,
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
            guild_id: payload.guild_id,
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
            guild_id: payload.guild_id,
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

fn internal_token_valid(state: &AppState, headers: &HeaderMap) -> bool {
    headers
        .get("x-hexrelay-internal-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        == Some(state.presence_internal_token.as_str())
}
