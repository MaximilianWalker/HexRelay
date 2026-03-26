use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use tracing::warn;

use crate::{
    domain::auth::validation::is_valid_identity_id,
    domain::server_channels::realtime::{
        self as server_channel_realtime, DispatchChannelMessageCreatedInput,
        DispatchChannelMessageDeletedInput, DispatchChannelMessageUpdatedInput,
    },
    infra::db::repos::server_channels_repo::{
        self, CreateServerChannelMessageError, SoftDeleteServerChannelMessageError,
        UpdateServerChannelMessageError,
    },
    infra::db::repos::servers_repo,
    models::{
        ApiError, ServerChannelListResponse, ServerChannelMessageCreateRequest,
        ServerChannelMessageEditRequest, ServerChannelMessageListQuery, ServerChannelMessagePage,
        ServerChannelMessageRecord,
    },
    shared::errors::{bad_request, conflict, forbidden, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::{
        auth::{enforce_csrf_for_cookie_auth, AuthSession},
        authorization::AuthorizedServerMembership,
    },
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const MAX_MESSAGE_CONTENT_LENGTH: usize = 4000;

fn current_timestamp() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

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

pub async fn list_server_channels(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
) -> ApiResult<Json<ServerChannelListResponse>> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel listing requires configured database pool",
        )
    })?;

    let items = server_channels_repo::list_server_channels(pool, &membership.server_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to list server channels"))?;

    Ok(Json(ServerChannelListResponse { items }))
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

    let content = normalize_message_content(&payload.content)?;

    let mention_identity_ids = normalize_mentions(payload.mention_identity_ids)?;
    let reply_to_message_id = normalize_reply_to_message_id(payload.reply_to_message_id)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel history requires configured database pool",
        )
    })?;
    let server_id = membership.server_id.clone();
    let author_id = membership.identity_id.clone();

    let created_at = current_timestamp();
    let message = server_channels_repo::create_server_channel_message(
        pool,
        server_channels_repo::CreateServerChannelMessageParams {
            server_id: server_id.clone(),
            channel_id,
            message_id: format!("scm-{}", uuid::Uuid::new_v4().simple()),
            author_id,
            content,
            reply_to_message_id,
            mention_identity_ids,
            created_at,
        },
    )
    .await
    .map_err(map_create_message_error)?;

    notify_channel_message_created(&state, pool, &server_id, &message).await;

    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn edit_server_channel_message(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Path((_, channel_id, message_id)): Path<(String, String, String)>,
    Json(payload): Json<ServerChannelMessageEditRequest>,
) -> ApiResult<Json<ServerChannelMessageRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    let content = normalize_message_content(&payload.content)?;
    let mention_identity_ids = normalize_mentions(payload.mention_identity_ids)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel history requires configured database pool",
        )
    })?;
    let server_id = membership.server_id.clone();
    let author_id = membership.identity_id.clone();

    let edited_at = current_timestamp();
    let message = server_channels_repo::update_server_channel_message(
        pool,
        server_channels_repo::UpdateServerChannelMessageParams {
            server_id: server_id.clone(),
            channel_id,
            message_id,
            author_id,
            content,
            mention_identity_ids,
            edited_at: edited_at.clone(),
        },
    )
    .await
    .map_err(map_update_message_error)?;

    if message.edited_at.as_deref() == Some(edited_at.as_str()) {
        notify_channel_message_updated(&state, pool, &server_id, &message).await;
    }

    Ok(Json(message))
}

pub async fn soft_delete_server_channel_message(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Path((_, channel_id, message_id)): Path<(String, String, String)>,
) -> ApiResult<Json<ServerChannelMessageRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server channel history requires configured database pool",
        )
    })?;
    let server_id = membership.server_id.clone();

    let deleted_at = current_timestamp();
    let message = server_channels_repo::soft_delete_server_channel_message(
        pool,
        &server_id,
        &channel_id,
        &message_id,
        &membership.identity_id,
        &deleted_at,
    )
    .await
    .map_err(map_soft_delete_message_error)?;

    if message.deleted_at.as_deref() == Some(deleted_at.as_str()) {
        notify_channel_message_deleted(&state, pool, &server_id, &message).await;
    }

    Ok(Json(message))
}

async fn notify_channel_message_created(
    state: &AppState,
    pool: &sqlx::PgPool,
    server_id: &str,
    message: &ServerChannelMessageRecord,
) {
    let recipients = match list_server_event_recipients(pool, server_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to load server channel event recipients");
            return;
        }
    };

    if let Err(error) = server_channel_realtime::dispatch_channel_message_created(
        state,
        DispatchChannelMessageCreatedInput {
            server_id,
            channel_id: &message.channel_id,
            message_id: &message.message_id,
            sender_id: &message.author_id,
            created_at: &message.created_at,
            channel_seq: message.channel_seq,
            recipients: &recipients,
        },
    )
    .await
    {
        warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to dispatch server channel create event");
    }
}

async fn notify_channel_message_updated(
    state: &AppState,
    pool: &sqlx::PgPool,
    server_id: &str,
    message: &ServerChannelMessageRecord,
) {
    let recipients = match list_server_event_recipients(pool, server_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to load server channel event recipients");
            return;
        }
    };

    let Some(edited_at) = message.edited_at.as_deref() else {
        return;
    };

    if let Err(error) = server_channel_realtime::dispatch_channel_message_updated(
        state,
        DispatchChannelMessageUpdatedInput {
            server_id,
            channel_id: &message.channel_id,
            message_id: &message.message_id,
            editor_id: &message.author_id,
            edited_at,
            channel_seq: message.channel_seq,
            recipients: &recipients,
        },
    )
    .await
    {
        warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to dispatch server channel update event");
    }
}

async fn notify_channel_message_deleted(
    state: &AppState,
    pool: &sqlx::PgPool,
    server_id: &str,
    message: &ServerChannelMessageRecord,
) {
    let recipients = match list_server_event_recipients(pool, server_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to load server channel event recipients");
            return;
        }
    };

    let Some(deleted_at) = message.deleted_at.as_deref() else {
        return;
    };

    if let Err(error) = server_channel_realtime::dispatch_channel_message_deleted(
        state,
        DispatchChannelMessageDeletedInput {
            server_id,
            channel_id: &message.channel_id,
            message_id: &message.message_id,
            deleted_by: &message.author_id,
            deleted_at,
            channel_seq: message.channel_seq,
            recipients: &recipients,
        },
    )
    .await
    {
        warn!(server_id = %server_id, channel_id = %message.channel_id, message_id = %message.message_id, error = %error, "failed to dispatch server channel delete event");
    }
}

async fn list_server_event_recipients(
    pool: &sqlx::PgPool,
    server_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    servers_repo::list_server_member_identity_ids(pool, server_id).await
}

fn normalize_message_content(value: &str) -> ApiResult<String> {
    let content = value.trim();
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

    Ok(content.to_string())
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

fn map_update_message_error(
    error: UpdateServerChannelMessageError,
) -> (StatusCode, Json<ApiError>) {
    match error {
        UpdateServerChannelMessageError::ChannelNotFound => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "channel_not_found",
                message: "server channel was not found",
            }),
        ),
        UpdateServerChannelMessageError::MessageNotFound => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "message_not_found",
                message: "server channel message was not found",
            }),
        ),
        UpdateServerChannelMessageError::EditForbidden => forbidden(
            "message_edit_forbidden",
            "only the author may edit this server channel message",
        ),
        UpdateServerChannelMessageError::MessageDeleted => conflict(
            "message_deleted",
            "deleted server channel messages cannot be edited",
        ),
        UpdateServerChannelMessageError::MentionTargetInvalid => bad_request(
            "mention_invalid",
            "mentioned identities must be members of the same server",
        ),
        UpdateServerChannelMessageError::Storage(_) => internal_error(
            "storage_unavailable",
            "failed to update server channel message",
        ),
    }
}

fn map_soft_delete_message_error(
    error: SoftDeleteServerChannelMessageError,
) -> (StatusCode, Json<ApiError>) {
    match error {
        SoftDeleteServerChannelMessageError::ChannelNotFound => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "channel_not_found",
                message: "server channel was not found",
            }),
        ),
        SoftDeleteServerChannelMessageError::MessageNotFound => (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "message_not_found",
                message: "server channel message was not found",
            }),
        ),
        SoftDeleteServerChannelMessageError::DeleteForbidden => forbidden(
            "message_delete_forbidden",
            "only the author may delete this server channel message",
        ),
        SoftDeleteServerChannelMessageError::Storage(_) => internal_error(
            "storage_unavailable",
            "failed to delete server channel message",
        ),
    }
}
