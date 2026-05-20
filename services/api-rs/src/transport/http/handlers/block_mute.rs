use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;

use crate::{
    domain::block_mute::validation::{validate_block_request, validate_mute_request},
    infra::db::repos::contacts_repo,
    models::{
        BlockListResponse, BlockRecord, BlockUserRequest, MuteListResponse, MuteRecord,
        MuteUserRequest,
    },
    shared::errors::{conflict, internal_error, ApiResult},
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

    if let Some(pool) = state.db_pool.as_ref() {
        let (record, inserted) = contacts_repo::upsert_user_block(pool, &auth.identity_id, &target)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to block user"))?;
        if !inserted {
            return Err(conflict("already_blocked", "user is already blocked"));
        }
        remember_block(
            &state,
            &record.blocker_identity_id,
            &record.blocked_identity_id,
            now,
        );
        return Ok((StatusCode::CREATED, Json(record)));
    }

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

    if let Some(pool) = state.db_pool.as_ref() {
        contacts_repo::delete_user_block(pool, &auth.identity_id, &target)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to unblock user"))?;
        forget_block(&state, &auth.identity_id, &target);
        return Ok(StatusCode::NO_CONTENT);
    }

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
    if let Some(pool) = state.db_pool.as_ref() {
        let items = contacts_repo::list_user_blocks(pool, &auth.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to list blocked users"))?;
        return Ok(Json(BlockListResponse { items }));
    }

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

    if let Some(pool) = state.db_pool.as_ref() {
        let (record, inserted) = contacts_repo::upsert_user_mute(pool, &auth.identity_id, &target)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to mute user"))?;
        if !inserted {
            return Err(conflict("already_muted", "user is already muted"));
        }
        remember_mute(
            &state,
            &record.muter_identity_id,
            &record.muted_identity_id,
            now,
        );
        return Ok((StatusCode::CREATED, Json(record)));
    }

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

    if let Some(pool) = state.db_pool.as_ref() {
        contacts_repo::delete_user_mute(pool, &auth.identity_id, &target)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to unmute user"))?;
        forget_mute(&state, &auth.identity_id, &target);
        return Ok(StatusCode::NO_CONTENT);
    }

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
    if let Some(pool) = state.db_pool.as_ref() {
        let items = contacts_repo::list_user_mutes(pool, &auth.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to list muted users"))?;
        return Ok(Json(MuteListResponse { items }));
    }

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

pub fn remember_block(state: &AppState, blocker: &str, blocked: &str, now: chrono::DateTime<Utc>) {
    let mut guard = state
        .blocked_users
        .write()
        .expect("acquire blocked_users write lock");
    guard
        .entry(blocker.to_string())
        .or_default()
        .insert(blocked.to_string(), now.timestamp());
}

pub fn forget_block(state: &AppState, blocker: &str, blocked: &str) {
    let mut guard = state
        .blocked_users
        .write()
        .expect("acquire blocked_users write lock");
    if let Some(entry) = guard.get_mut(blocker) {
        entry.remove(blocked);
        if entry.is_empty() {
            guard.remove(blocker);
        }
    }
}

fn remember_mute(state: &AppState, muter: &str, muted: &str, now: chrono::DateTime<Utc>) {
    let mut guard = state
        .muted_users
        .write()
        .expect("acquire muted_users write lock");
    guard
        .entry(muter.to_string())
        .or_default()
        .insert(muted.to_string(), now.timestamp());
}

fn forget_mute(state: &AppState, muter: &str, muted: &str) {
    let mut guard = state
        .muted_users
        .write()
        .expect("acquire muted_users write lock");
    if let Some(entry) = guard.get_mut(muter) {
        entry.remove(muted);
        if entry.is_empty() {
            guard.remove(muter);
        }
    }
}
