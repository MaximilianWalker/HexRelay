use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, TimeZone, Utc};
use ring::digest;
use serde::Deserialize;
use std::collections::HashSet;
use tracing::warn;

use crate::domain::auth::validation::is_valid_identity_id;
use crate::domain::block_mute::service::is_blocked_bidirectional;
use crate::infra::db::repos::{auth_repo, dm_history_repo, dm_repo, friends_repo, servers_repo};
use crate::transport::http::middleware::rate_limit;
use crate::{
    domain::dm::forwarding::{
        authenticate_node_forward_request, NodeForwardRequestError, NodeForwardRequestErrorStatus,
    },
    domain::dm::outbound_forwarding::{
        forwarding_error_summary, next_retry_attempt_after_failure,
        DM_OUTBOUND_FORWARD_MAX_ATTEMPTS,
    },
    domain::dm::realtime::{dispatch_dm_envelope, DispatchDmEnvelopeInput},
    domain::dm::validation::{
        validate_device_id, validate_device_secret, validate_dm_policy_update,
        validate_fanout_catch_up, validate_fanout_dispatch, validate_profile_device_heartbeat,
        DM_OFFLINE_DELIVERY_MODE, DM_PROFILE_DEVICE_ID_MAX_LENGTH,
    },
    models::{
        ApiError, DmFanoutCatchUpItem, DmFanoutCatchUpRequest, DmFanoutCatchUpResponse,
        DmFanoutDeliveryRecord, DmFanoutDispatchRequest, DmFanoutDispatchResponse, DmMessagePage,
        DmPolicy, DmPolicyUpdate, DmProfileDeviceHeartbeatRequest,
        DmProfileDeviceHeartbeatResponse, DmProfileDeviceRecord, DmProfileDeviceSummary,
        DmThreadListQuery, DmThreadMarkReadRequest, DmThreadMarkReadResponse,
        DmThreadMessageListQuery, DmThreadPage,
    },
    shared::errors::{
        bad_request, conflict, forbidden, internal_error, too_many_requests, unauthorized,
        ApiResult,
    },
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const DM_ENVELOPE_NODE_TRANSPORT_PROFILE: &str = "encrypted_envelope_node";
const DM_RATE_SCOPE_DISPATCH: &str = "dm_fanout_dispatch";
const DM_RATE_SCOPE_CATCH_UP: &str = "dm_fanout_catch_up";
const DM_RATE_SCOPE_ACK: &str = "dm_envelope_ack";
const DM_RATE_SCOPE_INTERNAL_FORWARD: &str = "dm_internal_forward";

#[derive(Deserialize)]
pub struct DmEnvelopeAckInternalRequest {
    pub envelope_id: String,
    pub message_id: String,
    pub thread_id: String,
    pub recipient_identity_id: String,
    pub device_id: String,
    pub delivery_cursor: String,
    pub ack_status: String,
    pub received_at: String,
}

#[derive(Deserialize)]
pub struct DmProfileDeviceVerifyInternalRequest {
    pub identity_id: String,
    pub device_id: String,
    pub device_secret: String,
}

struct DmFanoutAcceptanceInput<'a> {
    sender_identity_id: &'a str,
    recipient_identity_id: &'a str,
    message_id: &'a str,
    ciphertext: &'a str,
    source_device_id: Option<&'a str>,
}

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
    let device_secret_hash = device_secret_hash(payload.device_secret.trim());
    let identity_id = auth.identity_id.clone();
    let record = DmProfileDeviceRecord {
        device_id: device_id.clone(),
        device_secret_hash,
        active: payload.active,
        last_seen_epoch: now_epoch,
    };

    let devices = if let Some(pool) = state.db_pool.as_ref() {
        let upserted = dm_repo::upsert_dm_profile_device(pool, &identity_id, &record)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist profile device")
            })?;
        if !upserted {
            return Err(unauthorized(
                "profile_device_secret_invalid",
                "device_secret does not match this profile device",
            ));
        }
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
        if devices
            .get(&device_id)
            .is_some_and(|existing| existing.device_secret_hash != record.device_secret_hash)
        {
            return Err(unauthorized(
                "profile_device_secret_invalid",
                "device_secret does not match this profile device",
            ));
        }
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

pub async fn verify_dm_profile_device_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DmProfileDeviceVerifyInternalRequest>,
) -> ApiResult<StatusCode> {
    if !internal_token_valid(&state, &headers) {
        return Err(unauthorized(
            "internal_token_invalid",
            "DM profile device verification requires a valid internal token",
        ));
    }

    if !is_valid_identity_id(&payload.identity_id) {
        return Err(bad_request(
            "profile_device_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    validate_profile_device_secret_input(&payload.device_id, &payload.device_secret)?;

    let Some(record) =
        load_dm_profile_device(&state, &payload.identity_id, &payload.device_id).await?
    else {
        return Err(unauthorized(
            "profile_device_unknown",
            "profile device is not registered for this identity",
        ));
    };

    if record.device_secret_hash != device_secret_hash(payload.device_secret.trim()) {
        return Err(unauthorized(
            "profile_device_secret_invalid",
            "device_secret does not match this profile device",
        ));
    }

    if !record.active {
        return Err(unauthorized(
            "profile_device_inactive",
            "profile device is not active",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn forward_dm_envelope_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    let authenticated =
        authenticate_node_forward_request(&state, &headers, &body).map_err(node_forward_error)?;
    enforce_dm_rate_limit(
        &state,
        DM_RATE_SCOPE_INTERNAL_FORWARD,
        &authenticated.origin_node_id,
        state.rate_limits.dm_internal_forward_per_window,
    )
    .await?;
    apply_dm_delivery_metadata_retention(&state).await?;
    let request = authenticated.request;

    let response = accept_dm_envelope_for_local_recipient(
        &state,
        DmFanoutAcceptanceInput {
            sender_identity_id: &request.sender_identity_id,
            recipient_identity_id: &request.recipient_identity_id,
            message_id: &request.message_id,
            ciphertext: &request.ciphertext,
            source_device_id: request.source_device_id.as_deref(),
        },
    )
    .await?;

    Ok(Json(response))
}

pub async fn run_dm_active_fanout(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmFanoutDispatchRequest>,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_fanout_dispatch(&payload)?;
    enforce_dm_rate_limit(
        &state,
        DM_RATE_SCOPE_DISPATCH,
        &auth.identity_id,
        state.rate_limits.dm_dispatch_per_window,
    )
    .await?;
    apply_dm_delivery_metadata_retention(&state).await?;

    if let Some(destination_node_id) = payload
        .destination_node_id
        .as_deref()
        .filter(|node_id| *node_id != state.node_fingerprint)
    {
        return forward_dm_envelope_to_destination_node(
            &state,
            &auth,
            &payload,
            destination_node_id,
        )
        .await;
    }

    let response = accept_dm_envelope_for_local_recipient(
        &state,
        DmFanoutAcceptanceInput {
            sender_identity_id: &auth.identity_id,
            recipient_identity_id: &payload.recipient_identity_id,
            message_id: &payload.message_id,
            ciphertext: &payload.ciphertext,
            source_device_id: payload.source_device_id.as_deref(),
        },
    )
    .await?;

    Ok(Json(response))
}

async fn forward_dm_envelope_to_destination_node(
    state: &AppState,
    auth: &AuthSession,
    payload: &DmFanoutDispatchRequest,
    destination_node_id: &str,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "durable dm outbound forwarding requires configured database storage",
        )
    })?;
    let thread_id =
        dm_history_repo::direct_dm_thread_id(&auth.identity_id, &payload.recipient_identity_id);
    let accepted_at = Utc::now().to_rfc3339();
    let delivery_cursor = outbound_forward_delivery_cursor();
    let target_device_ids = Vec::new();
    let attempt_count = dm_repo::record_dm_outbound_forward_queued(
        pool,
        &dm_repo::DmOutboundForwardWrite {
            sender_identity_id: &auth.identity_id,
            destination_node_id,
            message_id: &payload.message_id,
            thread_id: &thread_id,
            recipient_identity_id: &payload.recipient_identity_id,
            ciphertext: &payload.ciphertext,
            source_device_id: payload.source_device_id.as_deref(),
            delivery_cursor,
        },
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to persist dm outbound forwarding attempt",
        )
    })?;

    let dispatch_result = dispatch_dm_envelope(
        state,
        DispatchDmEnvelopeInput {
            destination_node_id: Some(destination_node_id),
            message_id: &payload.message_id,
            thread_id: &thread_id,
            sender_identity_id: &auth.identity_id,
            recipient_identity_id: &payload.recipient_identity_id,
            ciphertext: &payload.ciphertext,
            source_device_id: payload.source_device_id.as_deref(),
            accepted_at: &accepted_at,
            delivery_cursor,
            target_device_ids: &target_device_ids,
        },
    )
    .await;

    match dispatch_result {
        Ok(()) => {
            let updated = dm_repo::mark_dm_outbound_forward_succeeded(
                pool,
                &auth.identity_id,
                destination_node_id,
                &payload.message_id,
            )
            .await
            .map_err(|_| {
                internal_error(
                    "storage_unavailable",
                    "failed to persist dm outbound forwarding success",
                )
            })?;
            if !updated {
                return Err(internal_error(
                    "storage_unavailable",
                    "dm outbound forwarding attempt was not found after transport success",
                ));
            }
        }
        Err(error) => {
            let stored_error = forwarding_error_summary(&error);
            let next_attempt_at = next_retry_attempt_after_failure(
                Utc::now(),
                destination_node_id,
                &payload.message_id,
                attempt_count,
                DM_OUTBOUND_FORWARD_MAX_ATTEMPTS,
                &stored_error,
            );
            if let Err(update_error) = dm_repo::mark_dm_outbound_forward_failed(
                pool,
                &auth.identity_id,
                destination_node_id,
                &payload.message_id,
                &stored_error,
                next_attempt_at,
            )
            .await
            {
                warn!(
                    message_id = %payload.message_id,
                    destination_node_id,
                    error = %update_error,
                    "failed to persist DM outbound forwarding failure"
                );
            }

            return Err(internal_error(
                "fanout_forwarding_failed",
                "failed to forward encrypted DM envelope to destination node",
            ));
        }
    }

    Ok(Json(DmFanoutDispatchResponse {
        status: "accepted".to_string(),
        reason_code: "fanout_forwarded_to_static_peer".to_string(),
        transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
        delivery_state: "forwarded".to_string(),
        reachability_state: "unknown".to_string(),
        fanout_count: 0,
        delivered_device_ids: vec![],
        skipped_device_ids: vec![],
    }))
}

fn outbound_forward_delivery_cursor() -> u64 {
    u64::try_from(Utc::now().timestamp_millis())
        .unwrap_or(1)
        .max(1)
}

async fn accept_dm_envelope_for_local_recipient(
    state: &AppState,
    input: DmFanoutAcceptanceInput<'_>,
) -> ApiResult<DmFanoutDispatchResponse> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "durable dm delivery requires configured database storage",
        )
    })?;

    let sender_identity_id = input.sender_identity_id.trim();
    let recipient_identity_id = input.recipient_identity_id.trim();

    let sender_exists = auth_repo::identity_exists(pool, sender_identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load sender identity"))?;
    if !sender_exists {
        return Err(bad_request(
            "fanout_invalid",
            "sender_identity_id must reference a registered identity",
        ));
    }

    let recipient_exists = auth_repo::identity_exists(pool, recipient_identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load recipient identity"))?;
    if !recipient_exists {
        return Err(bad_request(
            "fanout_invalid",
            "recipient_identity_id must reference a registered identity",
        ));
    }

    if is_blocked_bidirectional(state, sender_identity_id, recipient_identity_id)? {
        return Ok(DmFanoutDispatchResponse {
            status: "blocked".to_string(),
            reason_code: "fanout_blocked_user".to_string(),
            transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
            delivery_state: "rejected".to_string(),
            reachability_state: "blocked".to_string(),
            fanout_count: 0,
            delivered_device_ids: vec![],
            skipped_device_ids: vec![],
        });
    }

    match dm_interaction_policy_decision(state, sender_identity_id, recipient_identity_id).await? {
        DmInteractionPolicyDecision::Allowed => {}
        DmInteractionPolicyDecision::BlockedFriendsOnly
        | DmInteractionPolicyDecision::BlockedUnknown => {
            return Ok(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_policy_blocked".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                delivery_state: "rejected".to_string(),
                reachability_state: "blocked".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            });
        }
        DmInteractionPolicyDecision::BlockedSameServer => {
            return Ok(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_same_server_context_required".to_string(),
                transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
                delivery_state: "rejected".to_string(),
                reachability_state: "blocked".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            });
        }
    }

    let source_device_id = input.source_device_id.map(|value| value.trim().to_string());

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
    let message_id = input.message_id.trim().to_string();
    let ciphertext = input.ciphertext.to_string();
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
        sender_identity_id,
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
            author_id: sender_identity_id,
            seq,
            ciphertext: &ciphertext,
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
        thread_id: thread_id.clone(),
        message_id: message_id.clone(),
        sender_identity_id: sender_identity_id.to_string(),
        ciphertext: ciphertext.clone(),
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

    if !active_device_ids.is_empty() {
        if let Err(error) = dispatch_dm_envelope(
            state,
            DispatchDmEnvelopeInput {
                destination_node_id: None,
                message_id: &message_id,
                thread_id: &thread_id,
                sender_identity_id,
                recipient_identity_id,
                ciphertext: &ciphertext,
                source_device_id: source_device_id.as_deref(),
                accepted_at: &created_at,
                delivery_cursor: cursor,
                target_device_ids: &active_device_ids,
            },
        )
        .await
        {
            warn!(
                message_id = %message_id,
                thread_id = %thread_id,
                recipient_identity_id = %recipient_identity_id,
                error = %error,
                "failed to enqueue DM envelope realtime dispatch"
            );
        }
    }

    Ok(DmFanoutDispatchResponse {
        status: "accepted".to_string(),
        reason_code,
        transport_profile: DM_ENVELOPE_NODE_TRANSPORT_PROFILE.to_string(),
        delivery_state,
        reachability_state,
        fanout_count,
        delivered_device_ids,
        skipped_device_ids,
    })
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
    enforce_dm_rate_limit(
        &state,
        DM_RATE_SCOPE_CATCH_UP,
        &identity_id,
        state.rate_limits.dm_catch_up_per_window,
    )
    .await?;

    let Some(record) = load_dm_profile_device(&state, &identity_id, &device_id).await? else {
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

    if record.device_secret_hash != device_secret_hash(payload.device_secret.trim()) {
        return Err(unauthorized(
            "profile_device_secret_invalid",
            "device_secret does not match this profile device",
        ));
    }

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

    apply_dm_delivery_metadata_retention(&state).await?;

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

    let effective_cursor = last_cursor.max(user_cursor);

    let entries = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::list_pending_dm_fanout_delivery_records(
            pool,
            &identity_id,
            &device_id,
            effective_cursor,
            limit,
        )
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load fanout delivery log"))?
    } else {
        let delivery_log = state
            .dm_fanout_delivery_log
            .read()
            .expect("acquire dm fanout delivery log read lock");
        delivery_log
            .get(&identity_id)
            .into_iter()
            .flat_map(|records| records.iter())
            .filter(|entry| {
                entry.cursor > effective_cursor
                    && !entry.delivered_device_ids.iter().any(|id| id == &device_id)
            })
            .take(limit as usize)
            .cloned()
            .collect()
    };

    let mut items = Vec::new();
    let mut deduped_message_ids = Vec::new();
    let mut seen_delivery_keys = HashSet::new();
    let mut response_cursor = effective_cursor;
    for entry in &entries {
        response_cursor = entry.cursor;

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
            envelope_id: dm_envelope_id(&entry.message_id, &identity_id, &device_id, entry.cursor),
            cursor: entry.cursor.to_string(),
            thread_id: entry.thread_id.clone(),
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
        next_cursor: response_cursor.to_string(),
        deduped_message_ids,
        items,
    }))
}

pub async fn ack_dm_envelope_internal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DmEnvelopeAckInternalRequest>,
) -> ApiResult<StatusCode> {
    if !internal_token_valid(&state, &headers) {
        return Err(unauthorized(
            "internal_token_invalid",
            "DM envelope ack requires a valid internal token",
        ));
    }

    let cursor = validate_dm_envelope_ack_internal(&payload)?;
    let ack_rate_key = format!(
        "{}:{}",
        payload.recipient_identity_id.trim(),
        payload.device_id.trim()
    );
    enforce_dm_rate_limit(
        &state,
        DM_RATE_SCOPE_ACK,
        &ack_rate_key,
        state.rate_limits.dm_ack_per_window,
    )
    .await?;
    let expected_envelope_id = dm_envelope_id(
        payload.message_id.trim(),
        payload.recipient_identity_id.trim(),
        payload.device_id.trim(),
        cursor,
    );
    if payload.envelope_id != expected_envelope_id {
        return Err(bad_request(
            "dm_ack_invalid",
            "envelope_id does not match the acknowledged DM envelope",
        ));
    }

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "durable dm ack requires configured database storage",
        )
    })?;

    let acked = dm_repo::ack_dm_fanout_delivery_device(
        pool,
        payload.recipient_identity_id.trim(),
        payload.thread_id.trim(),
        payload.message_id.trim(),
        payload.device_id.trim(),
        cursor,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to persist dm envelope ack"))?;

    if !acked {
        return Err(bad_request(
            "dm_ack_unknown",
            "ack did not match a pending DM delivery record",
        ));
    }

    Ok(StatusCode::ACCEPTED)
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

fn node_forward_error(error: NodeForwardRequestError) -> (StatusCode, Json<ApiError>) {
    match error.status {
        NodeForwardRequestErrorStatus::BadRequest => bad_request(error.code, error.message),
        NodeForwardRequestErrorStatus::Unauthorized => unauthorized(error.code, error.message),
        NodeForwardRequestErrorStatus::Forbidden => forbidden(error.code, error.message),
        NodeForwardRequestErrorStatus::Conflict => conflict(error.code, error.message),
    }
}

async fn enforce_dm_rate_limit(
    state: &AppState,
    scope: &str,
    key: &str,
    limit: usize,
) -> ApiResult<()> {
    let allowed = if let Some(pool) = state.db_pool.as_ref() {
        rate_limit::allow_distributed(pool, scope, key, limit, state.rate_limits.window_seconds)
            .await
            .map_err(|_| {
                internal_error("rate_limiter_unavailable", "failed to check DM rate limit")
            })?
    } else {
        state
            .rate_limiter
            .allow(scope, key, limit, state.rate_limits.window_seconds)
    };

    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many DM requests; retry later",
        ));
    }

    Ok(())
}

async fn apply_dm_delivery_metadata_retention(state: &AppState) -> ApiResult<()> {
    let Some(pool) = state.db_pool.as_ref() else {
        return Ok(());
    };

    let now = Utc::now();
    let delivery_cutoff =
        now - chrono::Duration::seconds(state.dm_retention.delivery_log_retention_seconds);
    let outbound_cutoff = now
        - chrono::Duration::seconds(state.dm_retention.outbound_forwarding_log_retention_seconds);
    dm_repo::purge_expired_dm_delivery_metadata(pool, delivery_cutoff, outbound_cutoff)
        .await
        .map_err(|_| {
            internal_error(
                "storage_unavailable",
                "failed to apply DM delivery metadata retention",
            )
        })?;

    Ok(())
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

fn validate_dm_envelope_ack_internal(payload: &DmEnvelopeAckInternalRequest) -> ApiResult<u64> {
    if payload.envelope_id.trim().is_empty() || payload.envelope_id.len() > 128 {
        return Err(bad_request(
            "dm_ack_invalid",
            "envelope_id must be non-empty and <= 128 chars",
        ));
    }
    if trimmed_invalid(&payload.envelope_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "envelope_id must not include leading or trailing whitespace",
        ));
    }

    for (field, value) in [
        ("message_id", payload.message_id.as_str()),
        ("thread_id", payload.thread_id.as_str()),
    ] {
        if value.trim().is_empty() || value.len() > 128 {
            return Err(bad_request(
                "dm_ack_invalid",
                match field {
                    "message_id" => "message_id must be non-empty and <= 128 chars",
                    _ => "thread_id must be non-empty and <= 128 chars",
                },
            ));
        }
        if trimmed_invalid(value) {
            return Err(bad_request(
                "dm_ack_invalid",
                match field {
                    "message_id" => "message_id must not include leading or trailing whitespace",
                    _ => "thread_id must not include leading or trailing whitespace",
                },
            ));
        }
    }

    if !is_valid_identity_id(&payload.recipient_identity_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    let device_id = payload.device_id.trim();
    if device_id.is_empty() || device_id.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH {
        return Err(bad_request(
            "dm_ack_invalid",
            "device_id must be non-empty and <= 64 chars",
        ));
    }
    if trimmed_invalid(&payload.device_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "device_id must not include leading or trailing whitespace",
        ));
    }

    if payload.ack_status != "received" {
        return Err(bad_request("dm_ack_invalid", "ack_status must be received"));
    }

    DateTime::parse_from_rfc3339(payload.received_at.trim())
        .map_err(|_| bad_request("dm_ack_invalid", "received_at must be an RFC3339 date-time"))?;
    if trimmed_invalid(&payload.received_at) {
        return Err(bad_request(
            "dm_ack_invalid",
            "received_at must not include leading or trailing whitespace",
        ));
    }

    let cursor = payload
        .delivery_cursor
        .trim()
        .parse::<u64>()
        .map_err(|_| bad_request("dm_ack_invalid", "delivery_cursor must be numeric"))?;
    if cursor == 0 {
        return Err(bad_request(
            "dm_ack_invalid",
            "delivery_cursor must be greater than zero",
        ));
    }
    if trimmed_invalid(&payload.delivery_cursor) {
        return Err(bad_request(
            "dm_ack_invalid",
            "delivery_cursor must not include leading or trailing whitespace",
        ));
    }

    Ok(cursor)
}

fn internal_token_valid(state: &AppState, headers: &HeaderMap) -> bool {
    headers
        .get("x-hexrelay-internal-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        == Some(state.channel_dispatch_internal_token.as_str())
}

async fn load_dm_profile_device(
    state: &AppState,
    identity_id: &str,
    device_id: &str,
) -> ApiResult<Option<DmProfileDeviceRecord>> {
    if let Some(pool) = state.db_pool.as_ref() {
        return dm_repo::get_dm_profile_device(pool, identity_id, device_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load profile device"));
    }

    Ok(state
        .dm_profile_devices
        .read()
        .expect("acquire dm profile devices read lock")
        .get(identity_id)
        .and_then(|devices| devices.get(device_id))
        .cloned())
}

fn validate_profile_device_secret_input(device_id: &str, device_secret: &str) -> ApiResult<()> {
    validate_device_id(device_id, "profile_device_invalid")?;
    validate_device_secret(device_secret, "profile_device_invalid")
}

fn trimmed_invalid(value: &str) -> bool {
    value.trim() != value
}

fn device_secret_hash(value: &str) -> String {
    let digest = digest::digest(&digest::SHA256, value.as_bytes());
    lower_hex(digest.as_ref())
}

fn dm_envelope_id(
    message_id: &str,
    recipient_identity_id: &str,
    target_device_id: &str,
    delivery_cursor: u64,
) -> String {
    let material =
        format!("{message_id}:{recipient_identity_id}:{target_device_id}:{delivery_cursor}");
    let digest = digest::digest(&digest::SHA256, material.as_bytes());
    format!("dm-env-{}", lower_hex(digest.as_ref()))
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn ciphertext_fingerprint(value: &str) -> [u8; 32] {
    let digest = digest::digest(&digest::SHA256, value.as_bytes());
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(digest.as_ref());
    bytes
}
