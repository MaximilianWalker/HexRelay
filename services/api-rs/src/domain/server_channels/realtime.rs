use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct ChannelMessageCreatedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    sender_id: &'a str,
    created_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

#[derive(Serialize)]
struct ChannelMessageUpdatedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    editor_id: &'a str,
    edited_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

#[derive(Serialize)]
struct ChannelMessageDeletedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    deleted_by: &'a str,
    deleted_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

pub struct DispatchChannelMessageCreatedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub sender_id: &'a str,
    pub created_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

pub struct DispatchChannelMessageUpdatedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub editor_id: &'a str,
    pub edited_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

pub struct DispatchChannelMessageDeletedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub deleted_by: &'a str,
    pub deleted_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

pub async fn dispatch_channel_message_created(
    state: &AppState,
    input: DispatchChannelMessageCreatedInput<'_>,
) -> Result<(), String> {
    post_internal_event(
        state,
        "/internal/channels/messages/created",
        &ChannelMessageCreatedDispatchRequest {
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            sender_id: input.sender_id,
            created_at: input.created_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
        },
    )
    .await
}

pub async fn dispatch_channel_message_updated(
    state: &AppState,
    input: DispatchChannelMessageUpdatedInput<'_>,
) -> Result<(), String> {
    post_internal_event(
        state,
        "/internal/channels/messages/updated",
        &ChannelMessageUpdatedDispatchRequest {
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            editor_id: input.editor_id,
            edited_at: input.edited_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
        },
    )
    .await
}

pub async fn dispatch_channel_message_deleted(
    state: &AppState,
    input: DispatchChannelMessageDeletedInput<'_>,
) -> Result<(), String> {
    post_internal_event(
        state,
        "/internal/channels/messages/deleted",
        &ChannelMessageDeletedDispatchRequest {
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            deleted_by: input.deleted_by,
            deleted_at: input.deleted_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
        },
    )
    .await
}

async fn post_internal_event<T>(state: &AppState, path: &str, payload: &T) -> Result<(), String>
where
    T: Serialize + ?Sized,
{
    let url = format!("{}{}", state.realtime_base_url.trim_end_matches('/'), path);
    let response = state
        .http_client
        .post(url)
        .header("x-hexrelay-internal-token", &state.presence_internal_token)
        .json(payload)
        .send()
        .await
        .map_err(|error| format!("dispatch realtime channel event: {error}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    Err(format!(
        "dispatch realtime channel event failed with status {}",
        response.status()
    ))
}
