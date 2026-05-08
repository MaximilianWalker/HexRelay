use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{TimeZone, Utc};
use ring::digest;
use std::collections::HashSet;

use crate::domain::block_mute::service::is_blocked_bidirectional;
use crate::infra::db::repos::{auth_repo, dm_history_repo, dm_repo, friends_repo, servers_repo};
use crate::{
    domain::dm::validation::{
        validate_dm_policy_update, validate_fanout_catch_up, validate_fanout_dispatch,
        validate_profile_device_heartbeat, DM_OFFLINE_DELIVERY_MODE,
    },
    models::{
        ApiError, DmFanoutCatchUpItem, DmFanoutCatchUpRequest, DmFanoutCatchUpResponse,
        DmFanoutDeliveryRecord, DmFanoutDispatchRequest, DmFanoutDispatchResponse, DmMessagePage,
        DmPolicy, DmPolicyUpdate, DmProfileDeviceHeartbeatRequest,
        DmProfileDeviceHeartbeatResponse, DmProfileDeviceRecord, DmProfileDeviceSummary,
        DmThreadListQuery, DmThreadMarkReadRequest, DmThreadMarkReadResponse,
        DmThreadMessageListQuery, DmThreadPage,
    },
    shared::errors::{bad_request, conflict, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const DM_ENVELOPE_NODE_TRANSPORT_PROFILE: &str = "encrypted_envelope_node";

pub async fn get_dm_policy(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
) -> ApiResult<Json<DmPolicy>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let policy = dm_repo::get_dm_policy(pool, &auth.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load dm policy"))?
            .unwrap_or_else(default_dm_policy);
        return Ok(Json(policy));
    }

    let default = default_dm_policy();
    let policy = state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(&auth.identity_id)
        .cloned()
        .unwrap_or(default);
    Ok(Json(policy))
}

pub async fn update_dm_policy(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmPolicyUpdate>,
) -> ApiResult<Json<DmPolicy>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_dm_policy_update(&payload)?;

    let normalized = payload.inbound_policy.trim().to_string();
    let policy = DmPolicy {
        inbound_policy: normalized,
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    };

    if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::upsert_dm_policy(pool, &auth.identity_id, &policy)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to persist dm policy"))?;
    }

    state
        .dm_policies
        .write()
        .expect("acquire dm policy write lock")
        .insert(auth.identity_id, policy.clone());

    Ok(Json(policy))
}

pub async fn heartbeat_dm_profile_device(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmProfileDeviceHeartbeatRequest>,
) -> ApiResult<Json<DmProfileDeviceHeartbeatResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_profile_device_heartbeat(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let device_id = payload.device_id.trim().to_string();
    let identity_id = auth.identity_id.clone();
    let record = DmProfileDeviceRecord {
        device_id: device_id.clone(),
        active: payload.active,
        last_seen_epoch: now_epoch,
    };

    let devices = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::upsert_dm_profile_device(pool, &identity_id, &record)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist profile device")
            })?;
        dm_repo::list_dm_profile_devices(pool, &identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load profile devices"))?
            .into_iter()
            .map(|record| (record.device_id.clone(), record))
            .collect::<std::collections::HashMap<_, _>>()
    } else {
        let mut devices_by_identity = state
            .dm_profile_devices
            .write()
            .expect("acquire dm profile devices write lock");
        let devices = devices_by_identity.entry(identity_id.clone()).or_default();
        devices.insert(device_id, record);
        devices.clone()
    };

    if state.db_pool.is_none() {
        state
            .dm_profile_devices
            .write()
            .expect("acquire dm profile devices write lock")
            .insert(identity_id.clone(), devices.clone());
    }

    Ok(Json(DmProfileDeviceHeartbeatResponse {
        identity_id,
        devices: profile_devices_to_response(&devices, now_epoch),
    }))
}

pub async fn run_dm_active_fanout(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmFanoutDispatchRequest>,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_fanout_dispatch(&payload)?;

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "durable dm delivery requires configured database storage",
        )
    })?;

    let recipient_identity_id = payload.recipient_identity_id.trim();

    let recipient_exists = auth_repo::identity_exists(pool, recipient_identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load recipient identity"))?;
    if !recipient_exists {
        return Err(bad_request(
            "fanout_invalid",
            "recipient_identity_id must reference a registered identity",
        ));
    }

    if is_blocked_bidirectional(&state, &auth.identity_id, recipient_identity_id)? {
        return Ok(Json(DmFanoutDispatchResponse {
            status: "blocked".to_string(),
            reason_code: "fanout_blocked_user".to_string(),
            transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
            delivery_state: "rejected".to_string(),
            reachability_state: "blocked".to_string(),
            fanout_count: 0,
            delivered_device_ids: vec![],
            skipped_device_ids: vec![],
        }));
    }

    match dm_interaction_policy_decision(&state, &auth.identity_id, recipient_identity_id).await? {
        DmInteractionPolicyDecision::Allowed => {}
        DmInteractionPolicyDecision::BlockedFriendsOnly
        | DmInteractionPolicyDecision::BlockedUnknown => {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_policy_blocked".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                delivery_state: "rejected".to_string(),
                reachability_state: "blocked".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        }
        DmInteractionPolicyDecision::BlockedSameServer => {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_same_server_context_required".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                delivery_state: "rejected".to_string(),
                reachability_state: "blocked".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        }
    }

    let source_device_id = payload
        .source_device_id
        .as_ref()
        .map(|value| value.trim().to_string());

    let profile_devices = dm_repo::list_dm_profile_devices(pool, recipient_identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load profile devices"))?
        .into_iter()
        .map(|record| (record.device_id.clone(), record))
        .collect::<std::collections::HashMap<_, _>>();

    let (mut active_device_ids, mut skipped_device_ids) = {
        if profile_devices.is_empty() {
            (Vec::new(), Vec::new())
        } else {
            let mut active = Vec::new();
            let mut skipped = Vec::new();
            for record in profile_devices.values() {
                if !record.active {
                    skipped.push(record.device_id.clone());
                    continue;
                }

                active.push(record.device_id.clone());
            }

            (active, skipped)
        }
    };

    active_device_ids.sort();
    skipped_device_ids.sort();

    let delivered_device_ids = Vec::new();
    let fanout_count = 0;

    let reachability_state = if active_device_ids.is_empty() {
        "unreachable"
    } else {
        "unknown"
    };
    let message_id = payload.message_id.trim().to_string();
    let delivery_state = "pending_delivery".to_string();
    let reachability_state = reachability_state.to_string();
    let reason_code = "fanout_pending_delivery".to_string();

    let mut tx = pool.begin().await.map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to start dm acceptance transaction",
        )
    })?;
    let thread_id = dm_history_repo::ensure_direct_dm_thread_in_tx(
        &mut tx,
        &auth.identity_id,
        recipient_identity_id,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to ensure dm thread"))?;
    let seq = dm_history_repo::next_dm_message_seq_in_tx(&mut tx, &thread_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to allocate dm message seq"))?;
    let created_at = Utc::now().to_rfc3339();
    dm_history_repo::insert_dm_message_in_tx(
        &mut tx,
        dm_history_repo::DmMessageInsertParams {
            message_id: &message_id,
            thread_id: &thread_id,
            author_id: &auth.identity_id,
            seq,
            ciphertext: &payload.ciphertext,
            created_at: &created_at,
            edited_at: None,
        },
    )
    .await
    .map_err(|error| match error {
        sqlx::Error::Database(db_error)
            if db_error.code().as_deref() == Some("23505")
                && db_error.constraint() == Some("dm_messages_pkey") =>
        {
            conflict(
                "fanout_message_id_conflict",
                "message_id already exists for an accepted DM",
            )
        }
        _ => internal_error(
            "storage_unavailable",
            "failed to persist dm message history",
        ),
    })?;
    let cursor = dm_repo::advance_dm_fanout_stream_head_in_tx(&mut tx, recipient_identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to advance fanout cursor"))?;
    let delivery_record = DmFanoutDeliveryRecord {
        cursor,
        message_id: message_id.clone(),
        sender_identity_id: auth.identity_id.clone(),
        ciphertext: payload.ciphertext.clone(),
        source_device_id: source_device_id.clone(),
        delivery_state: delivery_state.clone(),
        reachability_state: reachability_state.clone(),
        delivered_device_ids: delivered_device_ids.clone(),
    };
    dm_repo::append_dm_fanout_delivery_record_in_tx(
        &mut tx,
        recipient_identity_id,
        &thread_id,
        &delivery_record,
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to persist dm delivery metadata",
        )
    })?;
    tx.commit().await.map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to commit dm acceptance transaction",
        )
    })?;

    Ok(Json(DmFanoutDispatchResponse {
        status: "accepted".to_string(),
        reason_code,
        transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
        delivery_state,
        reachability_state,
        fanout_count,
        delivered_device_ids,
        skipped_device_ids,
    }))
}

pub async fn run_dm_fanout_catch_up(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmFanoutCatchUpRequest>,
) -> ApiResult<Json<DmFanoutCatchUpResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let (limit, request_cursor) = validate_fanout_catch_up(&payload)?;

    let device_id = payload.device_id.trim().to_string();
    let identity_id = auth.identity_id;

    {
        let devices = if let Some(pool) = state.db_pool.as_ref() {
            dm_repo::list_dm_profile_devices(pool, &identity_id)
                .await
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to load profile devices")
                })?
                .into_iter()
                .map(|record| (record.device_id.clone(), record))
                .collect::<std::collections::HashMap<_, _>>()
        } else {
            state
                .dm_profile_devices
                .read()
                .expect("acquire dm profile devices read lock")
                .get(&identity_id)
                .cloned()
                .unwrap_or_default()
        };

        if devices.is_empty() {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_unknown".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        }

        let Some(record) = devices.get(&device_id) else {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_unknown".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        };

        if !record.active {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_inactive".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        }
    }

    let last_cursor = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::get_dm_fanout_device_cursor(pool, &identity_id, &device_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load fanout cursor"))?
    } else {
        state
            .dm_fanout_device_cursors
            .read()
            .expect("acquire dm fanout cursor read lock")
            .get(&identity_id)
            .and_then(|cursors| cursors.get(&device_id))
            .copied()
            .unwrap_or(0)
    };
    let user_cursor = request_cursor.unwrap_or(0);

    let tail_cursor = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::get_dm_fanout_stream_head(pool, &identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to load fanout stream head")
            })?
    } else {
        let delivery_log = state
            .dm_fanout_delivery_log
            .read()
            .expect("acquire dm fanout delivery log read lock");
        delivery_log
            .get(&identity_id)
            .and_then(|entries| entries.last())
            .map(|entry| entry.cursor)
            .unwrap_or(0)
    };
    if user_cursor > tail_cursor {
        return Err(bad_request(
            "cursor_out_of_range",
            "cursor exceeds available fanout history",
        ));
    }

    let entries = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::list_dm_fanout_delivery_records(pool, &identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to load fanout delivery log")
            })?
    } else {
        state
            .dm_fanout_delivery_log
            .read()
            .expect("acquire dm fanout delivery log read lock")
            .get(&identity_id)
            .cloned()
            .unwrap_or_default()
    };

    let effective_cursor = user_cursor.max(last_cursor);

    let mut items = Vec::new();
    let mut deduped_message_ids = Vec::new();
    let mut seen_delivery_keys = HashSet::new();
    let mut scanned_cursor = last_cursor;
    for entry in &entries {
        if entry.cursor <= effective_cursor {
            continue;
        }

        scanned_cursor = entry.cursor;

        if entry.delivered_device_ids.iter().any(|id| id == &device_id) {
            continue;
        }

        let dedupe_key = (
            entry.message_id.clone(),
            entry.sender_identity_id.clone(),
            entry.source_device_id.clone(),
            ciphertext_fingerprint(&entry.ciphertext),
        );
        if !seen_delivery_keys.insert(dedupe_key) {
            deduped_message_ids.push(entry.message_id.clone());
            continue;
        }

        items.push(DmFanoutCatchUpItem {
            cursor: entry.cursor.to_string(),
            message_id: entry.message_id.clone(),
            ciphertext: entry.ciphertext.clone(),
            source_device_id: entry.source_device_id.clone(),
        });

        if items.len() >= limit as usize {
            break;
        }
    }

    deduped_message_ids.sort();
    deduped_message_ids.dedup();

    let mut committed_cursor = last_cursor;
    if scanned_cursor > last_cursor {
        if let Some(pool) = state.db_pool.as_ref() {
            committed_cursor = dm_repo::upsert_dm_fanout_device_cursor(
                pool,
                &identity_id,
                &device_id,
                scanned_cursor,
            )
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist fanout cursor")
            })?;
        } else {
            let mut fanout_cursors = state
                .dm_fanout_device_cursors
                .write()
                .expect("acquire dm fanout cursor write lock");
            let device_cursors = fanout_cursors.entry(identity_id.clone()).or_default();
            let current = device_cursors.get(&device_id).copied().unwrap_or(0);
            committed_cursor = current.max(scanned_cursor);
            device_cursors.insert(device_id.clone(), committed_cursor);
        }
    }

    let reason_code = if items.is_empty() {
        "fanout_catch_up_no_missed"
    } else {
        "fanout_catch_up_ok"
    };

    Ok(Json(DmFanoutCatchUpResponse {
        status: "ready".to_string(),
        reason_code: reason_code.to_string(),
        transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
        device_id,
        replay_count: items.len() as u32,
        next_cursor: committed_cursor.to_string(),
        deduped_message_ids,
        items,
    }))
}

pub async fn list_dm_threads(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    let limit = parse_limit(query.limit)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;
    let unread_only = query.unread_only.unwrap_or(false);
    let mut items = dm_history_repo::list_dm_threads_for_identity(
        pool,
        &auth.identity_id,
        query.cursor.as_deref(),
        limit,
        unread_only,
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => bad_request("cursor_invalid", "unknown dm thread cursor"),
        _ => internal_error("storage_unavailable", "failed to list dm threads"),
    })?;

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let next_cursor = if has_more {
        items.last().map(|item| item.thread_id.clone())
    } else {
        None
    };

    Ok(Json(DmThreadPage { items, next_cursor }))
}

pub async fn list_dm_thread_messages(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(thread_id): Path<String>,
    Query(query): Query<DmThreadMessageListQuery>,
) -> ApiResult<Json<DmMessagePage>> {
    let limit = parse_limit(query.limit)?;
    let cursor = parse_message_cursor(query.cursor)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;

    let query_cursor = cursor.filter(|value| *value <= i64::MAX as u64);
    let mut items = dm_history_repo::list_dm_thread_messages_for_identity(
        pool,
        &auth.identity_id,
        &thread_id,
        query_cursor,
        limit,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to list dm thread messages"))?
    .ok_or({
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "thread_not_found",
                message: "dm thread was not found",
            }),
        )
    })?;

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let page_items = items;
    let next_cursor = if has_more {
        page_items.last().map(|item| item.seq.to_string())
    } else {
        None
    };

    Ok(Json(DmMessagePage {
        items: page_items,
        next_cursor,
    }))
}

pub async fn mark_dm_thread_read(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Path(thread_id): Path<String>,
    Json(body): Json<DmThreadMarkReadRequest>,
) -> ApiResult<Json<DmThreadMarkReadResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    if body.last_read_seq > i64::MAX as u64 {
        return Err(bad_request(
            "last_read_seq_invalid",
            "last_read_seq is out of range",
        ));
    }

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;

    let (new_seq, unread) = dm_history_repo::mark_dm_thread_read(
        pool,
        &auth.identity_id,
        &thread_id,
        body.last_read_seq,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to mark dm thread as read"))?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "thread_not_found",
            message: "dm thread was not found or identity is not a participant",
        }),
    ))?;

    Ok(Json(DmThreadMarkReadResponse {
        thread_id,
        last_read_seq: new_seq,
        unread,
    }))
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

fn parse_message_cursor(value: Option<String>) -> ApiResult<Option<u64>> {
    let Some(cursor) = value else {
        return Ok(None);
    };

    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| bad_request("cursor_invalid", "message cursor must be numeric"))
}

fn default_dm_policy() -> DmPolicy {
    DmPolicy {
        inbound_policy: "friends_only".to_string(),
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    }
}

async fn current_dm_policy(state: &AppState, identity_id: &str) -> ApiResult<DmPolicy> {
    if let Some(pool) = state.db_pool.as_ref() {
        return dm_repo::get_dm_policy(pool, identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load dm policy"))
            .map(|policy| policy.unwrap_or_else(default_dm_policy));
    }

    Ok(state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(identity_id)
        .cloned()
        .unwrap_or_else(default_dm_policy))
}

fn profile_devices_to_response(
    devices: &std::collections::HashMap<String, DmProfileDeviceRecord>,
    now_epoch: i64,
) -> Vec<DmProfileDeviceSummary> {
    let mut items = devices
        .values()
        .map(|record| DmProfileDeviceSummary {
            device_id: record.device_id.clone(),
            active: record.active,
            last_seen_at: Utc
                .timestamp_opt(record.last_seen_epoch, 0)
                .single()
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| {
                    Utc.timestamp_opt(now_epoch, 0)
                        .single()
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| Utc::now().to_rfc3339())
                }),
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| a.device_id.cmp(&b.device_id));
    items
}

async fn is_friend(state: &AppState, a: &str, b: &str) -> ApiResult<bool> {
    if let Some(pool) = state.db_pool.as_ref() {
        return friends_repo::are_friends(pool, a, b).await.map_err(|_| {
            internal_error(
                "friendship_lookup_failed",
                "failed to evaluate friendship state for DM policy",
            )
        });
    }

    Ok(state
        .friend_requests
        .read()
        .expect("acquire friend request read lock")
        .values()
        .any(|record| {
            record.status == "accepted"
                && ((record.requester_identity_id == a && record.target_identity_id == b)
                    || (record.requester_identity_id == b && record.target_identity_id == a))
        }))
}

enum DmInteractionPolicyDecision {
    Allowed,
    BlockedFriendsOnly,
    BlockedSameServer,
    BlockedUnknown,
}

async fn dm_interaction_policy_decision(
    state: &AppState,
    sender_identity_id: &str,
    recipient_identity_id: &str,
) -> ApiResult<DmInteractionPolicyDecision> {
    let policy = current_dm_policy(state, recipient_identity_id).await?;

    match policy.inbound_policy.as_str() {
        "anyone" => Ok(DmInteractionPolicyDecision::Allowed),
        "friends_only" => {
            if is_friend(state, sender_identity_id, recipient_identity_id).await? {
                Ok(DmInteractionPolicyDecision::Allowed)
            } else {
                Ok(DmInteractionPolicyDecision::BlockedFriendsOnly)
            }
        }
        "same_server" => {
            if let Some(pool) = state.db_pool.as_ref() {
                if servers_repo::identities_share_server(
                    pool,
                    sender_identity_id,
                    recipient_identity_id,
                )
                .await
                .map_err(|_| {
                    internal_error(
                        "storage_unavailable",
                        "failed to evaluate shared-server DM policy",
                    )
                })? {
                    Ok(DmInteractionPolicyDecision::Allowed)
                } else {
                    Ok(DmInteractionPolicyDecision::BlockedSameServer)
                }
            } else {
                Ok(DmInteractionPolicyDecision::BlockedSameServer)
            }
        }
        _ => Ok(DmInteractionPolicyDecision::BlockedUnknown),
    }
}

fn ciphertext_fingerprint(value: &str) -> [u8; 32] {
    let digest = digest::digest(&digest::SHA256, value.as_bytes());
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(digest.as_ref());
    bytes
}
