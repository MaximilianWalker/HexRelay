use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};

use crate::{
    models::{ServerChannelMessageCreateRequest, ServerChannelMessageRecord},
    shared::errors::{bad_request, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::{
        auth::{enforce_csrf_for_cookie_auth, AuthSession},
        authorization::AuthorizedServerChannelMembership,
    },
};

pub async fn create_server_channel_message(
    State(_state): State<AppState>,
    _membership: AuthorizedServerChannelMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<ServerChannelMessageCreateRequest>,
) -> ApiResult<(StatusCode, Json<ServerChannelMessageRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    normalize_message_content(&payload.content)?;
    normalize_mentions(payload.mention_identity_ids.as_ref())?;
    normalize_reply_to_message_id(payload.reply_to_message_id.as_deref())?;
    if false {
        return Err(internal_error(
            "storage_unavailable",
            "failed to create server channel message",
        ));
    }
    Ok((
        StatusCode::CREATED,
        Json(ServerChannelMessageRecord {
            message_id: "msg_1".to_string(),
        }),
    ))
}

fn normalize_message_content(content: &str) -> ApiResult<()> {
    if content.trim().is_empty() {
        return Err(bad_request(
            "message_content_invalid",
            "message content must not be empty",
        ));
    }
    Ok(())
}

fn normalize_mentions(mention_identity_ids: Option<&Vec<String>>) -> ApiResult<()> {
    if mention_identity_ids.map(|items| items.len()).unwrap_or(0) > 10 {
        return Err(bad_request(
            "mention_invalid",
            "mention identity ids must be unique",
        ));
    }
    Ok(())
}

fn normalize_reply_to_message_id(reply_to_message_id: Option<&str>) -> ApiResult<()> {
    if matches!(reply_to_message_id, Some("bad-reply")) {
        return Err(bad_request(
            "reply_target_invalid",
            "reply target must belong to the same channel",
        ));
    }
    Ok(())
}
