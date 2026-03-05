use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use ring::digest::{digest, SHA256};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{enforce_csrf_for_cookie_auth, AuthSession},
    errors::{bad_request, too_many_requests, ApiResult},
    models::{
        InviteCreateRequest, InviteCreateResponse, InviteRedeemRequest, InviteRedeemResponse,
    },
    rate_limit::allow_distributed,
    state::AppState,
    validation::{validate_invite_create_request, validate_invite_redeem_request},
};

#[cfg(test)]
use crate::models::InviteRecord;

#[cfg(not(test))]
use crate::errors::internal_error;

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
    .await;
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

    let token = Uuid::new_v4().to_string();
    let token_hash = hash_invite_token(&token);

    if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query(
            "
            INSERT INTO invites (token, mode, node_fingerprint, expires_at, max_uses, uses)
            VALUES ($1, $2, $3, $4, $5, 0)
            ",
        )
        .bind(&token_hash)
        .bind(&payload.mode)
        .bind(&state.node_fingerprint)
        .bind(expires_at)
        .bind(max_uses.map(|value| value as i32))
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to persist invite")
        })?;

        return Ok((
            StatusCode::CREATED,
            Json(InviteCreateResponse {
                token,
                mode: payload.mode,
                expires_at: expires_at.map(|value| value.to_rfc3339()),
                max_uses,
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
                token,
                mode: payload.mode,
                expires_at: expires_at.map(|value| value.to_rfc3339()),
                max_uses,
            }),
        ))
    }
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
    .await;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many invite redeem requests",
        ));
    }

    if let Some(pool) = state.db_pool.as_ref() {
        let mut tx = pool.begin().await.map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to start invite tx")
        })?;

        let row = sqlx::query(
            "
            SELECT node_fingerprint, expires_at, max_uses, uses
            FROM invites
            WHERE token = $1
            FOR UPDATE
            ",
        )
        .bind(&token_hash)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| crate::errors::internal_error("storage_unavailable", "failed to read invite"))?
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

        let node_fingerprint = row.try_get::<String, _>("node_fingerprint").map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to decode invite")
        })?;
        let expires_at = row
            .try_get::<Option<chrono::DateTime<Utc>>, _>("expires_at")
            .map_err(|_| {
                crate::errors::internal_error("storage_unavailable", "failed to decode invite")
            })?;
        let max_uses = row.try_get::<Option<i32>, _>("max_uses").map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to decode invite")
        })?;
        let uses = row.try_get::<i32, _>("uses").map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to decode invite")
        })?;

        if node_fingerprint != payload.node_fingerprint {
            return Err(bad_request(
                "fingerprint_mismatch",
                "invite node fingerprint mismatch",
            ));
        }

        if let Some(expires_at) = expires_at {
            if Utc::now() > expires_at {
                return Err(bad_request("invite_expired", "invite token is expired"));
            }
        }

        if let Some(max_uses) = max_uses {
            if uses >= max_uses {
                return Err(bad_request("invite_exhausted", "invite token is exhausted"));
            }
        }

        sqlx::query(
            "
            UPDATE invites
            SET uses = uses + 1
            WHERE token = $1
            ",
        )
        .bind(&token_hash)
        .execute(&mut *tx)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to update invite")
        })?;

        tx.commit().await.map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to commit invite")
        })?;

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

fn hash_invite_token(token: &str) -> String {
    hex::encode(digest(&SHA256, token.as_bytes()).as_ref())
}

async fn allow_rate_limit(state: &AppState, scope: &str, key: &str, limit: usize) -> bool {
    if let Some(pool) = state.db_pool.as_ref() {
        return match allow_distributed(pool, scope, key, limit, state.rate_limits.window_seconds)
            .await
        {
            Ok(allowed) => allowed,
            Err(_) => state
                .rate_limiter
                .allow(scope, key, limit, state.rate_limits.window_seconds),
        };
    }

    state
        .rate_limiter
        .allow(scope, key, limit, state.rate_limits.window_seconds)
}
