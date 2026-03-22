use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;

use crate::{
    domain::block_mute::validation::{validate_block_request, validate_mute_request},
    models::{
        BlockListResponse, BlockRecord, BlockUserRequest, MuteListResponse, MuteRecord,
        MuteUserRequest,
    },
    shared::errors::{conflict, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn block_user(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<BlockUserRequest>,
) -> ApiResult<(StatusCode, Json<BlockRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_block_request(&payload, &auth.identity_id)?;

    let target = payload.target_identity_id.trim().to_string();
    let now = Utc::now();

    let mut guard = state
        .blocked_users
        .write()
        .expect("acquire blocked_users write lock");

    let entry = guard.entry(auth.identity_id.clone()).or_default();
    if entry.contains_key(&target) {
        return Err(conflict("already_blocked", "user is already blocked"));
    }

    entry.insert(target.clone(), now.timestamp());

    let record = BlockRecord {
        blocker_identity_id: auth.identity_id,
        blocked_identity_id: target,
        created_at: now.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn unblock_user(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<BlockUserRequest>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_block_request(&payload, &auth.identity_id)?;

    let target = payload.target_identity_id.trim().to_string();

    let mut guard = state
        .blocked_users
        .write()
        .expect("acquire blocked_users write lock");

    if let Some(entry) = guard.get_mut(&auth.identity_id) {
        entry.remove(&target);
        if entry.is_empty() {
            guard.remove(&auth.identity_id);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_blocked_users(
    State(state): State<AppState>,
    auth: AuthSession,
) -> ApiResult<Json<BlockListResponse>> {
    let guard = state
        .blocked_users
        .read()
        .expect("acquire blocked_users read lock");

    let mut items: Vec<BlockRecord> = guard
        .get(&auth.identity_id)
        .map(|blocked| {
            blocked
                .iter()
                .map(|(target, epoch)| {
                    let created_at = chrono::DateTime::from_timestamp(*epoch, 0)
                        .unwrap_or_else(chrono::Utc::now)
                        .to_rfc3339();
                    BlockRecord {
                        blocker_identity_id: auth.identity_id.clone(),
                        blocked_identity_id: target.clone(),
                        created_at,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    items.sort_by(|a, b| a.blocked_identity_id.cmp(&b.blocked_identity_id));

    Ok(Json(BlockListResponse { items }))
}

pub async fn mute_user(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<MuteUserRequest>,
) -> ApiResult<(StatusCode, Json<MuteRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_mute_request(&payload, &auth.identity_id)?;

    let target = payload.target_identity_id.trim().to_string();
    let now = Utc::now();

    let mut guard = state
        .muted_users
        .write()
        .expect("acquire muted_users write lock");

    let entry = guard.entry(auth.identity_id.clone()).or_default();
    if entry.contains_key(&target) {
        return Err(conflict("already_muted", "user is already muted"));
    }

    entry.insert(target.clone(), now.timestamp());

    let record = MuteRecord {
        muter_identity_id: auth.identity_id,
        muted_identity_id: target,
        created_at: now.to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn unmute_user(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<MuteUserRequest>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_mute_request(&payload, &auth.identity_id)?;

    let target = payload.target_identity_id.trim().to_string();

    let mut guard = state
        .muted_users
        .write()
        .expect("acquire muted_users write lock");

    if let Some(entry) = guard.get_mut(&auth.identity_id) {
        entry.remove(&target);
        if entry.is_empty() {
            guard.remove(&auth.identity_id);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_muted_users(
    State(state): State<AppState>,
    auth: AuthSession,
) -> ApiResult<Json<MuteListResponse>> {
    let guard = state
        .muted_users
        .read()
        .expect("acquire muted_users read lock");

    let mut items: Vec<MuteRecord> = guard
        .get(&auth.identity_id)
        .map(|muted| {
            muted
                .iter()
                .map(|(target, epoch)| {
                    let created_at = chrono::DateTime::from_timestamp(*epoch, 0)
                        .unwrap_or_else(chrono::Utc::now)
                        .to_rfc3339();
                    MuteRecord {
                        muter_identity_id: auth.identity_id.clone(),
                        muted_identity_id: target.clone(),
                        created_at,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    items.sort_by(|a, b| a.muted_identity_id.cmp(&b.muted_identity_id));

    Ok(Json(MuteListResponse { items }))
}
