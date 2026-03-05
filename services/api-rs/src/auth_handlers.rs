use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use ring::signature::{UnparsedPublicKey, ED25519};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{csrf_cookie_name, enforce_csrf_for_cookie_auth, session_cookie_name, AuthSession},
    errors::{bad_request, conflict, too_many_requests, unauthorized, ApiResult},
    models::{
        AuthChallengeRecord, AuthChallengeRequest, AuthChallengeResponse, AuthVerifyRequest,
        AuthVerifyResponse, IdentityKeyRegistrationRequest, RegisteredIdentityKey,
        SessionRevokeRequest, SessionValidateResponse,
    },
    rate_limit::allow_distributed,
    session_token::issue_session_token,
    state::AppState,
    validation::{
        decode_32_bytes, decode_64_bytes, validate_auth_challenge_request,
        validate_auth_verify_request, validate_identity_registration,
        validate_session_revoke_request,
    },
};

#[cfg(not(test))]
use crate::errors::internal_error;

#[cfg(test)]
use crate::models::SessionRecord;

const CHALLENGE_TTL_SECONDS: i64 = 60;

pub async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> ApiResult<StatusCode> {
    validate_identity_registration(&payload)?;

    if let Some(pool) = state.db_pool.as_ref() {
        let inserted = sqlx::query(
            "
            INSERT INTO identity_keys (identity_id, public_key, algorithm)
            VALUES ($1, $2, $3)
            ON CONFLICT (identity_id) DO NOTHING
            ",
        )
        .bind(&payload.identity_id)
        .bind(&payload.public_key)
        .bind(&payload.algorithm)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to persist identity key")
        })?;

        if inserted.rows_affected() == 0 {
            return Err(conflict(
                "identity_exists",
                "identity_id already has a registered key",
            ));
        }

        return Ok(StatusCode::CREATED);
    }

    #[cfg(not(test))]
    {
        Err(internal_error(
            "storage_unavailable",
            "identity registration requires configured database pool",
        ))
    }

    #[cfg(test)]
    {
        let mut guard = state
            .identity_keys
            .write()
            .expect("acquire identity key write lock");

        if guard.contains_key(&payload.identity_id) {
            return Err(conflict(
                "identity_exists",
                "identity_id already has a registered key",
            ));
        }

        guard.insert(
            payload.identity_id,
            RegisteredIdentityKey {
                public_key: payload.public_key,
                algorithm: payload.algorithm,
            },
        );

        Ok(StatusCode::CREATED)
    }
}

pub async fn validate_session(auth: AuthSession) -> ApiResult<Json<SessionValidateResponse>> {
    Ok(Json(SessionValidateResponse {
        session_id: auth.session_id,
        identity_id: auth.identity_id,
        expires_at: auth.expires_at,
    }))
}

pub async fn issue_auth_challenge(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<AuthChallengeRequest>,
) -> ApiResult<Json<AuthChallengeResponse>> {
    validate_auth_challenge_request(&payload)?;

    let rate_key = rate_limit_key(&payload.identity_id);
    let allowed = allow_rate_limit(
        &state,
        "auth_challenge",
        &rate_key,
        state.rate_limits.auth_challenge_per_window,
    )
    .await;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many auth challenge requests",
        ));
    }

    let identity_exists = if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query_scalar::<_, i64>(
            "
            SELECT COUNT(*)
            FROM identity_keys
            WHERE identity_id = $1
            ",
        )
        .bind(&payload.identity_id)
        .fetch_one(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to read identity keys")
        })? > 0
    } else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "auth challenge issuance requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            state
                .identity_keys
                .read()
                .expect("acquire identity key read lock")
                .contains_key(&payload.identity_id)
        }
    };

    if !identity_exists {
        return Err(bad_request(
            "identity_invalid",
            "identity_id is not registered",
        ));
    }

    let challenge_id = Uuid::new_v4().to_string();
    let nonce = random_hex(32);
    let challenge_expires_at = Utc::now() + Duration::seconds(CHALLENGE_TTL_SECONDS);
    let expires_at = challenge_expires_at.to_rfc3339();

    if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query(
            "
            INSERT INTO auth_challenges (challenge_id, identity_id, nonce, expires_at)
            VALUES ($1, $2, $3, $4)
            ",
        )
        .bind(&challenge_id)
        .bind(&payload.identity_id)
        .bind(&nonce)
        .bind(challenge_expires_at)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to persist challenge")
        })?;

        return Ok(Json(AuthChallengeResponse {
            challenge_id,
            nonce,
            expires_at,
        }));
    }

    #[cfg(not(test))]
    {
        Err(internal_error(
            "storage_unavailable",
            "auth challenge issuance requires configured database pool",
        ))
    }

    #[cfg(test)]
    {
        state
            .auth_challenges
            .write()
            .expect("acquire challenge write lock")
            .insert(
                challenge_id.clone(),
                AuthChallengeRecord {
                    identity_id: payload.identity_id,
                    nonce: nonce.clone(),
                    expires_at: challenge_expires_at,
                },
            );

        Ok(Json(AuthChallengeResponse {
            challenge_id,
            nonce,
            expires_at,
        }))
    }
}

pub async fn verify_auth_challenge(
    State(state): State<AppState>,
    _headers: HeaderMap,
    Json(payload): Json<AuthVerifyRequest>,
) -> ApiResult<(HeaderMap, Json<AuthVerifyResponse>)> {
    validate_auth_verify_request(&payload)?;

    let rate_key = rate_limit_key(&payload.identity_id);
    let allowed = allow_rate_limit(
        &state,
        "auth_verify",
        &rate_key,
        state.rate_limits.auth_verify_per_window,
    )
    .await;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many auth verify requests",
        ));
    }

    let challenge_record = if let Some(pool) = state.db_pool.as_ref() {
        let row = sqlx::query(
            "
            DELETE FROM auth_challenges
            WHERE challenge_id = $1
            RETURNING identity_id, nonce, expires_at
            ",
        )
        .bind(&payload.challenge_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to read challenge")
        })?
        .ok_or_else(|| unauthorized("nonce_invalid", "challenge_id is invalid"))?;

        AuthChallengeRecord {
            identity_id: row.try_get::<String, _>("identity_id").map_err(|_| {
                crate::errors::internal_error("storage_unavailable", "failed to decode challenge")
            })?,
            nonce: row.try_get::<String, _>("nonce").map_err(|_| {
                crate::errors::internal_error("storage_unavailable", "failed to decode challenge")
            })?,
            expires_at: row
                .try_get::<chrono::DateTime<Utc>, _>("expires_at")
                .map_err(|_| {
                    crate::errors::internal_error(
                        "storage_unavailable",
                        "failed to decode challenge",
                    )
                })?,
        }
    } else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "auth verification requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            state
                .auth_challenges
                .write()
                .expect("acquire challenge write lock")
                .remove(&payload.challenge_id)
                .ok_or_else(|| unauthorized("nonce_invalid", "challenge_id is invalid"))?
        }
    };

    if challenge_record.identity_id != payload.identity_id {
        return Err(unauthorized(
            "nonce_invalid",
            "challenge does not match identity",
        ));
    }

    if Utc::now() > challenge_record.expires_at {
        return Err(unauthorized("nonce_invalid", "challenge has expired"));
    }

    let key_record = if let Some(pool) = state.db_pool.as_ref() {
        let row = sqlx::query(
            "
            SELECT public_key, algorithm
            FROM identity_keys
            WHERE identity_id = $1
            ",
        )
        .bind(&payload.identity_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to read identity key")
        })?
        .ok_or_else(|| unauthorized("identity_invalid", "identity_id is not registered"))?;

        let public_key = row.try_get::<String, _>("public_key").map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to decode identity key")
        })?;
        let algorithm = row.try_get::<String, _>("algorithm").map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to decode identity key")
        })?;

        RegisteredIdentityKey {
            public_key,
            algorithm,
        }
    } else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "auth verification requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            state
                .identity_keys
                .read()
                .expect("acquire identity key read lock")
                .get(&payload.identity_id)
                .cloned()
                .ok_or_else(|| unauthorized("identity_invalid", "identity_id is not registered"))?
        }
    };

    if key_record.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "registered algorithm must be ed25519",
        ));
    }

    let public_key = decode_32_bytes(&key_record.public_key)
        .ok_or_else(|| bad_request("public_key_invalid", "registered public key is invalid"))?;
    let signature_bytes = decode_64_bytes(&payload.signature).ok_or_else(|| {
        bad_request(
            "signature_invalid",
            "signature must be 64-byte hex or base64",
        )
    })?;

    verify_signature(
        &public_key,
        challenge_record.nonce.as_bytes(),
        &signature_bytes,
    )
    .map_err(|_| unauthorized("signature_invalid", "signature verification failed"))?;

    let identity_id = payload.identity_id.clone();

    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(12);

    if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query(
            "
            INSERT INTO sessions (session_id, identity_id, expires_at)
            VALUES ($1, $2, $3)
            ",
        )
        .bind(&session_id)
        .bind(&identity_id)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to persist session")
        })?;
    } else {
        #[cfg(not(test))]
        {
            return Err(internal_error(
                "storage_unavailable",
                "session issuance requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            state
                .sessions
                .write()
                .expect("acquire session write lock")
                .insert(
                    session_id.clone(),
                    SessionRecord {
                        identity_id: identity_id.clone(),
                        expires_at,
                    },
                );
        }
    }

    let signing_key = state
        .session_signing_keys
        .get(&state.active_signing_key_id)
        .expect("active signing key exists in keyring");
    let access_token = issue_session_token(
        &session_id,
        &identity_id,
        expires_at.timestamp(),
        &state.active_signing_key_id,
        signing_key,
    );

    let csrf_token = random_hex(24);
    let mut response_headers = HeaderMap::new();
    append_cookie(
        &mut response_headers,
        &build_session_cookie_value(
            session_cookie_name(),
            &access_token,
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            true,
        ),
    )?;
    append_cookie(
        &mut response_headers,
        &build_session_cookie_value(
            csrf_cookie_name(),
            &csrf_token,
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            false,
        ),
    )?;

    Ok((
        response_headers,
        Json(AuthVerifyResponse {
            session_id,
            expires_at: expires_at.to_rfc3339(),
        }),
    ))
}

pub async fn revoke_session(
    auth: AuthSession,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SessionRevokeRequest>,
) -> ApiResult<(HeaderMap, StatusCode)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_session_revoke_request(&payload)?;

    if payload.session_id != auth.session_id {
        return Err(unauthorized(
            "session_invalid",
            "session_id does not match authenticated session",
        ));
    }

    if let Some(pool) = state.db_pool.as_ref() {
        let updated = sqlx::query(
            "
            UPDATE sessions
            SET revoked_at = NOW()
            WHERE session_id = $1 AND revoked_at IS NULL
            ",
        )
        .bind(&payload.session_id)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to revoke session")
        })?;

        if updated.rows_affected() == 0 {
            return Err(bad_request("session_invalid", "session_id is invalid"));
        }

        return Ok((clear_auth_cookies(&state)?, StatusCode::NO_CONTENT));
    }

    #[cfg(not(test))]
    {
        Err(internal_error(
            "storage_unavailable",
            "session revoke requires configured database pool",
        ))
    }

    #[cfg(test)]
    {
        let removed = state
            .sessions
            .write()
            .expect("acquire session write lock")
            .remove(&payload.session_id);

        if removed.is_none() {
            return Err(bad_request("session_invalid", "session_id is invalid"));
        }

        Ok((clear_auth_cookies(&state)?, StatusCode::NO_CONTENT))
    }
}

fn clear_auth_cookies(
    state: &AppState,
) -> Result<HeaderMap, (StatusCode, Json<crate::models::ApiError>)> {
    let mut response_headers = HeaderMap::new();
    append_cookie(
        &mut response_headers,
        &build_expired_cookie(
            session_cookie_name(),
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            true,
        ),
    )?;
    append_cookie(
        &mut response_headers,
        &build_expired_cookie(
            csrf_cookie_name(),
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            false,
        ),
    )?;

    Ok(response_headers)
}

fn append_cookie(
    headers: &mut HeaderMap,
    cookie_value: &str,
) -> Result<(), (StatusCode, Json<crate::models::ApiError>)> {
    let header_value = HeaderValue::from_str(cookie_value).map_err(|_| {
        crate::errors::internal_error("cookie_invalid", "failed to encode cookie header")
    })?;
    headers.append(SET_COOKIE, header_value);
    Ok(())
}

fn build_session_cookie_value(
    name: &str,
    value: &str,
    same_site: &str,
    secure: bool,
    domain: Option<&str>,
    http_only: bool,
) -> String {
    let mut parts = vec![
        format!("{}={}", name, value),
        "Path=/".to_string(),
        format!("SameSite={}", same_site),
    ];

    if let Some(domain) = domain {
        parts.push(format!("Domain={}", domain));
    }

    if secure {
        parts.push("Secure".to_string());
    }

    if http_only {
        parts.push("HttpOnly".to_string());
    }

    parts.join("; ")
}

fn build_expired_cookie(
    name: &str,
    same_site: &str,
    secure: bool,
    domain: Option<&str>,
    http_only: bool,
) -> String {
    let mut cookie = build_session_cookie_value(name, "", same_site, secure, domain, http_only);
    cookie.push_str("; Max-Age=0");
    cookie
}

fn random_hex(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn rate_limit_key(identity_hint: &str) -> String {
    identity_hint.to_string()
}

fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<(), ()> {
    let key = UnparsedPublicKey::new(&ED25519, public_key);
    key.verify(message, signature).map_err(|_| ())
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
