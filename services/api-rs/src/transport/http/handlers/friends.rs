use crate::{
    domain::friends::{
        service::ActorRole,
        validation::{validate_friend_request_create, validate_friend_request_list_query},
    },
    infra::db::repos::friends_repo::{self, FriendRequestRepoError},
    models::{
        DmEndpointCard, DmProfileDeviceSummary, FriendRequestCreate, FriendRequestListQuery,
        FriendRequestPage, FriendRequestRecord, IdentityBootstrapBundle,
    },
    shared::errors::{bad_request, conflict, forbidden, unauthorized, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

use crate::shared::errors::internal_error;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::infra::db::repos::{auth_repo, dm_repo};

#[cfg(test)]
use crate::domain::friends::service::apply_friend_request_transition;

#[cfg(test)]
use chrono::Utc;

#[cfg(test)]
use uuid::Uuid;

pub async fn create_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<FriendRequestCreate>,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_friend_request_create(&payload)?;
    let actor_identity = auth.identity_id;

    if payload.requester_identity_id != actor_identity {
        return Err(unauthorized(
            "identity_invalid",
            "requester_identity_id must match authenticated session",
        ));
    }

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return create_friend_request_in_memory(state, payload);
        }
    };

    let record = friends_repo::create_friend_request(pool, payload)
        .await
        .map_err(map_friend_request_db_error)?;
    Ok((StatusCode::CREATED, Json(record)))
}

#[cfg(test)]
fn create_friend_request_in_memory(
    state: AppState,
    payload: FriendRequestCreate,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let existing = guard.values().find(|item| {
        item.requester_identity_id == payload.requester_identity_id
            && item.target_identity_id == payload.target_identity_id
            && item.status == "pending"
    });

    if existing.is_some() {
        return Err(conflict(
            "friend_request_exists",
            "pending friend request already exists",
        ));
    }

    let record = FriendRequestRecord {
        request_id: Uuid::new_v4().to_string(),
        requester_identity_id: payload.requester_identity_id,
        target_identity_id: payload.target_identity_id,
        status: "pending".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    guard.insert(record.request_id.clone(), record.clone());

    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn list_friend_requests(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<FriendRequestListQuery>,
) -> ApiResult<Json<FriendRequestPage>> {
    validate_friend_request_list_query(&query)?;
    let actor_identity = auth.identity_id;

    if query.identity_id != actor_identity {
        return Err(unauthorized(
            "identity_invalid",
            "identity_id must match authenticated session",
        ));
    }

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return list_friend_requests_in_memory(state, query);
        }
    };

    let items = friends_repo::list_friend_requests(pool, &query)
        .await
        .map_err(map_friend_request_db_error)?;
    Ok(Json(FriendRequestPage { items }))
}

#[cfg(test)]
fn list_friend_requests_in_memory(
    state: AppState,
    query: FriendRequestListQuery,
) -> ApiResult<Json<FriendRequestPage>> {
    let guard = state
        .friend_requests
        .read()
        .expect("acquire friend request read lock");

    let mut items: Vec<FriendRequestRecord> = guard
        .values()
        .filter(|item| match query.direction.as_deref() {
            Some("inbound") => item.target_identity_id == query.identity_id,
            Some("outbound") => item.requester_identity_id == query.identity_id,
            _ => {
                item.requester_identity_id == query.identity_id
                    || item.target_identity_id == query.identity_id
            }
        })
        .cloned()
        .collect();

    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(Json(FriendRequestPage { items }))
}

pub async fn accept_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<Json<FriendRequestRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return accept_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = friends_repo::update_friend_request_status(
        pool,
        &request_id,
        "accepted",
        &actor_identity,
        ActorRole::Target,
    )
    .await
    .map_err(map_friend_request_db_error)?
    .ok_or_else(|| {
        bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        )
    })?;
    Ok(Json(updated))
}

#[cfg(test)]
fn accept_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<Json<FriendRequestRecord>> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition(request, "accepted", &actor_identity, ActorRole::Target)?;

    Ok(Json(request.clone()))
}

pub async fn decline_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return decline_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = friends_repo::update_friend_request_status(
        pool,
        &request_id,
        "declined",
        &actor_identity,
        ActorRole::Target,
    )
    .await
    .map_err(map_friend_request_db_error)?;

    if updated.is_none() {
        return Err(bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
fn decline_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<StatusCode> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition(request, "declined", &actor_identity, ActorRole::Target)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return cancel_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = friends_repo::update_friend_request_status(
        pool,
        &request_id,
        "cancelled",
        &actor_identity,
        ActorRole::Requester,
    )
    .await
    .map_err(map_friend_request_db_error)?;

    if updated.is_none() {
        return Err(bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
fn cancel_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<StatusCode> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition(request, "cancelled", &actor_identity, ActorRole::Requester)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_friend_request_bootstrap(
    State(state): State<AppState>,
    auth: AuthSession,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<Json<IdentityBootstrapBundle>> {
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "bootstrap requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return get_friend_request_bootstrap_in_memory(state, request_id, actor_identity);
        }
    };

    let request = friends_repo::get_friend_request_by_id(pool, &request_id)
        .await
        .map_err(map_friend_request_db_error)?
        .ok_or_else(|| {
            bad_request(
                "identity_invalid",
                "friend request not found or not actionable by current session",
            )
        })?;

    let is_requester = request.requester_identity_id == actor_identity;
    let is_target = request.target_identity_id == actor_identity;
    if !is_requester && !is_target {
        return Err(unauthorized(
            "identity_invalid",
            "friend request cannot be accessed by this session",
        ));
    }

    if request.status != "accepted" {
        return Err(forbidden(
            "bootstrap_not_available",
            "identity bootstrap material is only available after friend request acceptance",
        ));
    }

    let peer_identity_id = if is_requester {
        &request.target_identity_id
    } else {
        &request.requester_identity_id
    };

    let identity_key = auth_repo::get_identity_key(pool, peer_identity_id)
        .await
        .map_err(|_| {
            internal_error(
                "storage_failure",
                "failed to retrieve peer identity key material",
            )
        })?
        .ok_or_else(|| {
            internal_error(
                "bootstrap_incomplete",
                "peer identity key is not registered",
            )
        })?;

    let now_epoch = chrono::Utc::now().timestamp();
    let card_records = dm_repo::list_dm_endpoint_cards(pool, peer_identity_id, now_epoch)
        .await
        .map_err(|_| internal_error("storage_failure", "failed to retrieve peer endpoint cards"))?;

    let endpoint_cards: Vec<DmEndpointCard> = card_records
        .into_iter()
        .filter(|card| !card.revoked)
        .map(|card| DmEndpointCard {
            endpoint_id: card.endpoint_id,
            endpoint_hint: card.endpoint_hint,
            estimated_rtt_ms: card.estimated_rtt_ms,
            priority: card.priority,
            expires_at: chrono::DateTime::from_timestamp(card.expires_at_epoch, 0)
                .unwrap_or_default()
                .to_rfc3339(),
            revoked: false,
        })
        .collect();

    let device_records = dm_repo::list_dm_profile_devices(pool, peer_identity_id)
        .await
        .map_err(|_| {
            internal_error("storage_failure", "failed to retrieve peer profile devices")
        })?;

    let devices: Vec<DmProfileDeviceSummary> = device_records
        .into_iter()
        .map(|device| DmProfileDeviceSummary {
            device_id: device.device_id,
            active: device.active,
            last_seen_at: chrono::DateTime::from_timestamp(device.last_seen_epoch, 0)
                .unwrap_or_default()
                .to_rfc3339(),
        })
        .collect();

    Ok(Json(IdentityBootstrapBundle {
        identity_id: peer_identity_id.to_string(),
        public_key: identity_key.public_key,
        algorithm: identity_key.algorithm,
        endpoint_cards,
        devices,
    }))
}

#[cfg(test)]
fn get_friend_request_bootstrap_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<Json<IdentityBootstrapBundle>> {
    let guard = state
        .friend_requests
        .read()
        .expect("acquire friend request read lock");

    let request = guard
        .get(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    let is_requester = request.requester_identity_id == actor_identity;
    let is_target = request.target_identity_id == actor_identity;
    if !is_requester && !is_target {
        return Err(unauthorized(
            "identity_invalid",
            "friend request cannot be accessed by this session",
        ));
    }

    if request.status != "accepted" {
        return Err(forbidden(
            "bootstrap_not_available",
            "identity bootstrap material is only available after friend request acceptance",
        ));
    }

    let peer_identity_id = if is_requester {
        &request.target_identity_id
    } else {
        &request.requester_identity_id
    };

    let keys_guard = state
        .identity_keys
        .read()
        .expect("acquire identity keys read lock");
    let identity_key = keys_guard.get(peer_identity_id).ok_or_else(|| {
        internal_error(
            "bootstrap_incomplete",
            "peer identity key is not registered",
        )
    })?;

    let cards_guard = state
        .dm_endpoint_cards
        .read()
        .expect("acquire endpoint cards read lock");
    let now_epoch = chrono::Utc::now().timestamp();
    let endpoint_cards: Vec<DmEndpointCard> = cards_guard
        .get(peer_identity_id)
        .map(|cards| {
            cards
                .values()
                .filter(|card| !card.revoked && card.expires_at_epoch >= now_epoch)
                .map(|card| DmEndpointCard {
                    endpoint_id: card.endpoint_id.clone(),
                    endpoint_hint: card.endpoint_hint.clone(),
                    estimated_rtt_ms: card.estimated_rtt_ms,
                    priority: card.priority,
                    expires_at: chrono::DateTime::from_timestamp(card.expires_at_epoch, 0)
                        .unwrap_or_default()
                        .to_rfc3339(),
                    revoked: false,
                })
                .collect()
        })
        .unwrap_or_default();

    let devices_guard = state
        .dm_profile_devices
        .read()
        .expect("acquire profile devices read lock");
    let devices: Vec<DmProfileDeviceSummary> = devices_guard
        .get(peer_identity_id)
        .map(|devices| {
            devices
                .values()
                .map(|device| DmProfileDeviceSummary {
                    device_id: device.device_id.clone(),
                    active: device.active,
                    last_seen_at: chrono::DateTime::from_timestamp(device.last_seen_epoch, 0)
                        .unwrap_or_default()
                        .to_rfc3339(),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Json(IdentityBootstrapBundle {
        identity_id: peer_identity_id.to_string(),
        public_key: identity_key.public_key.clone(),
        algorithm: identity_key.algorithm.clone(),
        endpoint_cards,
        devices,
    }))
}

fn map_friend_request_db_error(
    error: FriendRequestRepoError,
) -> (StatusCode, Json<crate::models::ApiError>) {
    if let FriendRequestRepoError::Sql(sqlx::Error::Database(db_error)) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return conflict(
                "friend_request_exists",
                "pending friend request already exists",
            );
        }
    }

    if let FriendRequestRepoError::TransitionInvalid = error {
        return conflict(
            "transition_invalid",
            "friend request transition is not allowed from current state",
        );
    }

    if let FriendRequestRepoError::ActorNotAuthorized = error {
        return unauthorized(
            "identity_invalid",
            "friend request cannot be mutated by this session",
        );
    }

    internal_error("storage_failure", "friend request storage failure")
}
