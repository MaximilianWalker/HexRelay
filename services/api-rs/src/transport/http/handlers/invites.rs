use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use ring::digest::{digest, SHA256};
use uuid::Uuid;

use crate::{
    domain::invites::validation::{
        validate_contact_invite_redeem_request, validate_invite_create_request,
        validate_invite_redeem_request,
    },
    infra::db::repos::{
        friends_repo::{self, FriendRequestRepoError},
        invites_repo,
    },
    models::{
        ContactInviteRedeemRequest, FriendRequestCreate, FriendRequestRecord, InviteCreateRequest,
        InviteCreateResponse, InviteRedeemRequest, InviteRedeemResponse,
    },
    shared::errors::{bad_request, conflict, internal_error, too_many_requests, ApiResult},
    state::AppState,
    transport::http::middleware::{
        auth::{enforce_csrf_for_cookie_auth, AuthSession},
        rate_limit::allow_distributed,
    },
};

#[cfg(test)]
use crate::models::InviteRecord;

pub async fn create_invite(
    auth: AuthSession,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<InviteCreateRequest>,
) -> ApiResult<(StatusCode, Json<InviteCreateResponse>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_invite_create_request(&payload)?;

    let allowed = allow_rate_limit(
        &state,
        "invite_create",
        &auth.identity_id,
        state.rate_limits.invite_create_per_window,
    )
    .await?;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many invite create requests",
        ));
    }

    let expires_at = if let Some(raw_expires_at) = payload.expires_at.as_ref() {
        let parsed = chrono::DateTime::parse_from_rfc3339(raw_expires_at)
            .map_err(|_| bad_request("invite_invalid", "expires_at must be RFC3339 date-time"))?
            .with_timezone(&Utc);

        if parsed <= Utc::now() {
            return Err(bad_request(
                "invite_invalid",
                "expires_at must be in the future",
            ));
        }

        Some(parsed)
    } else {
        None
    };

    let max_uses = if payload.mode == "one_time" {
        match payload.max_uses {
            None => Some(1),
            Some(1) => Some(1),
            Some(_) => {
                return Err(bad_request(
                    "invite_invalid",
                    "one_time invite max_uses must be 1 if provided",
                ));
            }
        }
    } else {
        payload.max_uses
    };

    let invite_id = Uuid::new_v4().to_string();
    let token = Uuid::new_v4().to_string();
    let token_hash = hash_invite_token(&token);
    let created_at = Utc::now().to_rfc3339();

    if let Some(pool) = state.db_pool.as_ref() {
        invites_repo::insert_invite(
            pool,
            invites_repo::InviteInsertParams {
                invite_id: &invite_id,
                token_hash: &token_hash,
                mode: &payload.mode,
                creator_identity_id: &auth.identity_id,
                node_fingerprint: &state.node_fingerprint,
                expires_at,
                max_uses: max_uses.map(|value| value as i32),
            },
        )
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to persist invite"))?;

        return Ok((
            StatusCode::CREATED,
            Json(InviteCreateResponse {
                invite_id,
                token,
                mode: payload.mode,
                expires_at: expires_at.map(|value| value.to_rfc3339()),
                max_uses,
                created_at,
            }),
        ));
    }

    #[cfg(not(test))]
    {
        Err(internal_error(
            "storage_unavailable",
            "invite creation requires configured database pool",
        ))
    }

    #[cfg(test)]
    {
        state
            .invites
            .write()
            .expect("acquire invite write lock")
            .insert(
                token_hash,
                InviteRecord {
                    invite_id: Some(invite_id.clone()),
                    creator_identity_id: Some(auth.identity_id.clone()),
                    mode: payload.mode.clone(),
                    node_fingerprint: state.node_fingerprint.clone(),
                    expires_at,
                    max_uses,
                    uses: 0,
                },
            );

        Ok((
            StatusCode::CREATED,
            Json(InviteCreateResponse {
                invite_id,
                token,
                mode: payload.mode,
                expires_at: expires_at.map(|value| value.to_rfc3339()),
                max_uses,
                created_at,
            }),
        ))
    }
}

pub async fn create_contact_invite(
    auth: AuthSession,
    headers: HeaderMap,
    state: State<AppState>,
    payload: Json<InviteCreateRequest>,
) -> ApiResult<(StatusCode, Json<InviteCreateResponse>)> {
    create_invite(auth, headers, state, payload).await
}

pub async fn redeem_invite(
    State(state): State<AppState>,
    Json(payload): Json<InviteRedeemRequest>,
) -> ApiResult<Json<InviteRedeemResponse>> {
    validate_invite_redeem_request(&payload)?;
    let token_hash = hash_invite_token(&payload.token);
    let allowed = allow_rate_limit(
        &state,
        "invite_redeem",
        &format!("{}:{}", payload.node_fingerprint, token_hash),
        state.rate_limits.invite_redeem_per_window,
    )
    .await?;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many invite redeem requests",
        ));
    }

    if let Some(pool) = state.db_pool.as_ref() {
        let mut tx = pool
            .begin()
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to start invite tx"))?;

        let row = invites_repo::load_invite_for_update(&mut *tx, &token_hash)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to read invite"))?
            .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

        if row.node_fingerprint != payload.node_fingerprint {
            return Err(bad_request(
                "fingerprint_mismatch",
                "invite node fingerprint mismatch",
            ));
        }

        if let Some(expires_at) = row.expires_at {
            if Utc::now() > expires_at {
                return Err(bad_request("invite_expired", "invite token is expired"));
            }
        }

        if let Some(max_uses) = row.max_uses {
            if row.uses >= max_uses {
                return Err(bad_request("invite_exhausted", "invite token is exhausted"));
            }
        }

        invites_repo::increment_invite_use(&mut *tx, &token_hash)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to update invite"))?;

        tx.commit()
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to commit invite"))?;

        return Ok(Json(InviteRedeemResponse { accepted: true }));
    }

    #[cfg(not(test))]
    {
        Err(internal_error(
            "storage_unavailable",
            "invite redemption requires configured database pool",
        ))
    }

    #[cfg(test)]
    {
        let mut guard = state.invites.write().expect("acquire invite write lock");
        let invite = guard
            .get_mut(&token_hash)
            .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

        if invite.node_fingerprint != payload.node_fingerprint {
            return Err(bad_request(
                "fingerprint_mismatch",
                "invite node fingerprint mismatch",
            ));
        }

        if let Some(expires_at) = invite.expires_at {
            if Utc::now() > expires_at {
                return Err(bad_request("invite_expired", "invite token is expired"));
            }
        }

        if let Some(max_uses) = invite.max_uses {
            if invite.uses >= max_uses {
                return Err(bad_request("invite_exhausted", "invite token is exhausted"));
            }
        }

        invite.uses += 1;

        Ok(Json(InviteRedeemResponse { accepted: true }))
    }
}

pub async fn redeem_contact_invite(
    auth: AuthSession,
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<ContactInviteRedeemRequest>,
) -> ApiResult<Json<FriendRequestRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_contact_invite_redeem_request(&payload)?;

    let token_hash = hash_invite_token(&payload.token);
    let allowed = allow_rate_limit(
        &state,
        "invite_redeem",
        &format!("contact:{}:{}", auth.identity_id, token_hash),
        state.rate_limits.invite_redeem_per_window,
    )
    .await?;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many invite redeem requests",
        ));
    }

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "contact invite redemption requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            let mut invites = state.invites.write().expect("acquire invite write lock");
            let invite = invites
                .get_mut(&token_hash)
                .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

            let inviter_identity_id = invite
                .creator_identity_id
                .clone()
                .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

            if inviter_identity_id == auth.identity_id {
                return Err(conflict(
                    "invite_invalid",
                    "cannot redeem a contact invite created by the same identity",
                ));
            }

            if let Some(expires_at) = invite.expires_at {
                if Utc::now() > expires_at {
                    return Err(bad_request("invite_expired", "invite token is expired"));
                }
            }

            if let Some(max_uses) = invite.max_uses {
                if invite.uses >= max_uses {
                    return Err(bad_request("invite_exhausted", "invite token is exhausted"));
                }
            }
            invite.uses += 1;
            drop(invites);

            let mut friend_requests = state
                .friend_requests
                .write()
                .expect("acquire friend request write lock");

            if let Some(existing) = friend_requests.values().find(|value| {
                value.requester_identity_id == auth.identity_id
                    && value.target_identity_id == inviter_identity_id
                    && value.status == "pending"
            }) {
                return Ok(Json(existing.clone()));
            }

            let record = FriendRequestRecord {
                request_id: Uuid::new_v4().to_string(),
                requester_identity_id: auth.identity_id,
                target_identity_id: inviter_identity_id,
                status: "pending".to_string(),
                created_at: Utc::now().to_rfc3339(),
            };

            friend_requests.insert(record.request_id.clone(), record.clone());
            return Ok(Json(record));
        }
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to start invite tx"))?;

    let row = invites_repo::load_invite_for_update(&mut *tx, &token_hash)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to read invite"))?
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

    let inviter_identity_id = row
        .creator_identity_id
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

    if inviter_identity_id == auth.identity_id {
        return Err(conflict(
            "invite_invalid",
            "cannot redeem a contact invite created by the same identity",
        ));
    }

    if let Some(expires_at) = row.expires_at {
        if Utc::now() > expires_at {
            return Err(bad_request("invite_expired", "invite token is expired"));
        }
    }

    if let Some(max_uses) = row.max_uses {
        if row.uses >= max_uses {
            return Err(bad_request("invite_exhausted", "invite token is exhausted"));
        }
    }

    invites_repo::increment_invite_use(&mut *tx, &token_hash)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to update invite"))?;

    let friend_request = friends_repo::create_friend_request_in_tx(
        &mut tx,
        FriendRequestCreate {
            requester_identity_id: auth.identity_id.clone(),
            target_identity_id: inviter_identity_id.clone(),
        },
    )
    .await
    .map_err(map_friend_request_db_error)?;

    tx.commit()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to commit invite"))?;

    Ok(Json(friend_request))
}

fn hash_invite_token(token: &str) -> String {
    hex::encode(digest(&SHA256, token.as_bytes()).as_ref())
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

    internal_error("storage_failure", "friend request storage failure")
}

async fn allow_rate_limit(
    state: &AppState,
    scope: &str,
    key: &str,
    limit: usize,
) -> ApiResult<bool> {
    if let Some(pool) = state.db_pool.as_ref() {
        let allowed = allow_distributed(pool, scope, key, limit, state.rate_limits.window_seconds)
            .await
            .map_err(|_| {
                internal_error(
                    "rate_limiter_unavailable",
                    "failed to enforce distributed rate limit",
                )
            })?;

        return Ok(allowed);
    }

    Ok(state
        .rate_limiter
        .allow(scope, key, limit, state.rate_limits.window_seconds))
}
