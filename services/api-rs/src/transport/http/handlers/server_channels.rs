use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;

use crate::{
    domain::auth::validation::is_valid_identity_id,
    infra::db::repos::server_channels_repo::{self, CreateServerChannelMessageError},
    models::{
        ApiError, ServerChannelMessageCreateRequest, ServerChannelMessageListQuery,
        ServerChannelMessagePage,
    },
    shared::errors::{bad_request, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::{
        auth::{enforce_csrf_for_cookie_auth, AuthSession},
        authorization::AuthorizedServerMembership,
    },
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const MAX_MESSAGE_CONTENT_LENGTH: usize = 4000;

pub async fn list_server_channel_messages(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    Path((_, channel_id)): Path<(String, String)>,
    Query(query): Query<ServerChannelMessageListQuery>,
) -> ApiResult<Json<ServerChannelMessagePage>> {
    let limit = parse_limit(query.limit)?;
    let cursor = parse_channel_message_cursor(query.cursor)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel history requires configured database pool",
        )
    })?;

    let mut items = server_channels_repo::list_server_channel_messages(
        pool,
        &membership.server_id,
        &channel_id,
        cursor,
        limit,
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to list server channel messages",
        )
    })?
    .ok_or({
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "channel_not_found",
                message: "server channel was not found",
            }),
        )
    })?;

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let next_cursor = if has_more {
        items.last().map(|item| item.channel_seq.to_string())
    } else {
        None
    };

    Ok(Json(ServerChannelMessagePage { items, next_cursor }))
}

pub async fn create_server_channel_message(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Path((_, channel_id)): Path<(String, String)>,
    Json(payload): Json<ServerChannelMessageCreateRequest>,
) -> ApiResult<(StatusCode, Json<crate::models::ServerChannelMessageRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    let content = payload.content.trim();
    if content.is_empty() {
        return Err(bad_request(
            "message_content_invalid",
            "message content must not be empty",
        ));
    }
    if content.len() > MAX_MESSAGE_CONTENT_LENGTH {
        return Err(bad_request(
            "message_content_invalid",
            "message content exceeds maximum length",
        ));
    }

    let mention_identity_ids = normalize_mentions(payload.mention_identity_ids)?;
    let reply_to_message_id = normalize_reply_to_message_id(payload.reply_to_message_id)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel history requires configured database pool",
        )
    })?;

    let created_at = Utc::now().to_rfc3339();
    let message = server_channels_repo::create_server_channel_message(
        pool,
        server_channels_repo::CreateServerChannelMessageParams {
            server_id: membership.server_id,
            channel_id,
            message_id: format!("scm-{}", uuid::Uuid::new_v4().simple()),
            author_id: membership.identity_id,
            content: content.to_string(),
            reply_to_message_id,
            mention_identity_ids,
            created_at,
        },
    )
    .await
    .map_err(map_create_message_error)?;

    Ok((StatusCode::CREATED, Json(message)))
}

fn parse_limit(value: Option<u32>) -> ApiResult<usize> {
    let raw = value.unwrap_or(DEFAULT_PAGE_LIMIT as u32);
    if raw == 0 {
        return Err(bad_request(
            "limit_invalid",
            "limit must be greater than zero",
        ));
    }
    if raw as usize > MAX_PAGE_LIMIT {
        return Err(bad_request(
            "limit_invalid",
            "limit exceeds maximum page size",
        ));
    }

    Ok(raw as usize)
}

fn parse_channel_message_cursor(value: Option<String>) -> ApiResult<Option<u64>> {
    let Some(cursor) = value else {
        return Ok(None);
    };

    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| bad_request("cursor_invalid", "message cursor must be numeric"))
        .and_then(|parsed| {
            if parsed.is_some_and(|value| value > i64::MAX as u64) {
                Err(bad_request(
                    "cursor_out_of_range",
                    "message cursor exceeds storage range",
                ))
            } else {
                Ok(parsed)
            }
        })
}

fn normalize_reply_to_message_id(value: Option<String>) -> ApiResult<Option<String>> {
    let Some(reply_to_message_id) = value else {
        return Ok(None);
    };

    let trimmed = reply_to_message_id.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(trimmed.to_string()))
}

fn normalize_mentions(mut mention_identity_ids: Vec<String>) -> ApiResult<Vec<String>> {
    for identity_id in &mut mention_identity_ids {
        *identity_id = identity_id.trim().to_string();
        if identity_id.is_empty() {
            return Err(bad_request(
                "mention_invalid",
                "mentioned identity ids must not be empty",
            ));
        }
        if !is_valid_identity_id(identity_id) {
            return Err(bad_request(
                "mention_invalid",
                "mentioned identity ids must be valid identity ids",
            ));
        }
    }

    mention_identity_ids.sort();
    mention_identity_ids.dedup();
    Ok(mention_identity_ids)
}

fn map_create_message_error(
    error: CreateServerChannelMessageError,
) -> (StatusCode, Json<ApiError>) {
    match error {
        CreateServerChannelMessageError::ChannelNotFound => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "channel_not_found",
                message: "server channel was not found",
            }),
        ),
        CreateServerChannelMessageError::ReplyTargetInvalid => bad_request(
            "reply_target_invalid",
            "reply target must exist in the same server channel",
        ),
        CreateServerChannelMessageError::MentionTargetInvalid => bad_request(
            "mention_invalid",
            "mentioned identities must be members of the same server",
        ),
        CreateServerChannelMessageError::Storage(_) => internal_error(
            "storage_unavailable",
            "failed to create server channel message",
        ),
    }
}
