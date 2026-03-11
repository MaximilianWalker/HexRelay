use axum::{
    extract::ConnectInfo,
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use ring::signature::{UnparsedPublicKey, ED25519};
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use tracing::warn;
use uuid::Uuid;

use crate::{
    domain::auth::validation::{
        decode_32_bytes, decode_64_bytes, validate_auth_challenge_request,
        validate_auth_verify_request, validate_identity_registration,
        validate_session_revoke_request,
    },
    infra::{crypto::session_token::issue_session_token, db::repos::auth_repo},
    models::{
        AuthChallengeRequest, AuthChallengeResponse, AuthVerifyRequest, AuthVerifyResponse,
        IdentityKeyRegistrationRequest, SessionRevokeRequest, SessionValidateResponse,
    },
    shared::errors::{
        bad_request, conflict, internal_error, too_many_requests, unauthorized, ApiResult,
    },
    state::AppState,
    transport::http::middleware::{
        auth::{csrf_cookie_name, enforce_csrf_for_cookie_auth, session_cookie_name, AuthSession},
        rate_limit::allow_distributed,
    },
};

#[cfg(test)]
use crate::models::{AuthChallengeRecord, RegisteredIdentityKey, SessionRecord};

const CHALLENGE_TTL_SECONDS: i64 = 60;

pub async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> ApiResult<StatusCode> {
    validate_identity_registration(&payload)?;

    if let Some(pool) = state.db_pool.as_ref() {
        let inserted = auth_repo::insert_identity_key(
            pool,
            &payload.identity_id,
            &payload.public_key,
            &payload.algorithm,
        )
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to persist identity key"))?;

        if !inserted {
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
    headers: HeaderMap,
    peer_addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<AuthChallengeRequest>,
) -> ApiResult<Json<AuthChallengeResponse>> {
    validate_auth_challenge_request(&payload)?;

    let source_key =
        source_rate_limit_key(&state, &headers, peer_addr.as_ref().map(|value| value.0));
    let source_allowed = if source_key == "source:unknown" {
        true
    } else {
        allow_rate_limit(
            &state,
            "auth_challenge_source",
            &source_key,
            state.rate_limits.auth_challenge_per_window,
        )
        .await?
    };
    if !source_allowed {
        warn!("auth challenge source rate limit exceeded");
        return Err(too_many_requests(
            "rate_limited",
            "too many auth challenge requests",
        ));
    }

    let rate_key = identity_rate_limit_key(
        &state,
        &payload.identity_id,
        &headers,
        peer_addr.as_ref().map(|value| value.0),
    );
    let allowed = allow_rate_limit(
        &state,
        "auth_challenge",
        &rate_key,
        state.rate_limits.auth_challenge_per_window,
    )
    .await?;
    if !allowed {
        warn!(
            identity_id = %payload.identity_id,
            "auth challenge rate limit exceeded"
        );
        return Err(too_many_requests(
            "rate_limited",
            "too many auth challenge requests",
        ));
    }

    let identity_exists = if let Some(pool) = state.db_pool.as_ref() {
        auth_repo::identity_exists(pool, &payload.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to read identity keys"))?
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

    let challenge_id = Uuid::new_v4().to_string();
    let nonce = random_hex(32);
    let challenge_expires_at = Utc::now() + Duration::seconds(CHALLENGE_TTL_SECONDS);
    let expires_at = challenge_expires_at.to_rfc3339();

    if identity_exists {
        if let Some(pool) = state.db_pool.as_ref() {
            auth_repo::insert_auth_challenge(
                pool,
                &challenge_id,
                &payload.identity_id,
                &nonce,
                challenge_expires_at,
            )
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to persist challenge"))?;

            return Ok(Json(AuthChallengeResponse {
                challenge_id,
                nonce,
                expires_at,
            }));
        }

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
        }
    }

    Ok(Json(AuthChallengeResponse {
        challenge_id,
        nonce,
        expires_at,
    }))
}

pub async fn verify_auth_challenge(
    State(state): State<AppState>,
    headers: HeaderMap,
    peer_addr: Option<ConnectInfo<SocketAddr>>,
    Json(payload): Json<AuthVerifyRequest>,
) -> ApiResult<(HeaderMap, Json<AuthVerifyResponse>)> {
    validate_auth_verify_request(&payload)?;

    let source_key =
        source_rate_limit_key(&state, &headers, peer_addr.as_ref().map(|value| value.0));
    let source_allowed = if source_key == "source:unknown" {
        true
    } else {
        allow_rate_limit(
            &state,
            "auth_verify_source",
            &source_key,
            state.rate_limits.auth_verify_per_window,
        )
        .await?
    };
    if !source_allowed {
        warn!("auth verify source rate limit exceeded");
        return Err(too_many_requests(
            "rate_limited",
            "too many auth verify requests",
        ));
    }

    let rate_key = identity_rate_limit_key(
        &state,
        &payload.identity_id,
        &headers,
        peer_addr.as_ref().map(|value| value.0),
    );
    let allowed = allow_rate_limit(
        &state,
        "auth_verify",
        &rate_key,
        state.rate_limits.auth_verify_per_window,
    )
    .await?;
    if !allowed {
        warn!(
            identity_id = %payload.identity_id,
            "auth verify rate limit exceeded"
        );
        return Err(too_many_requests(
            "rate_limited",
            "too many auth verify requests",
        ));
    }

    let challenge_record = if let Some(pool) = state.db_pool.as_ref() {
        auth_repo::consume_auth_challenge(pool, &payload.challenge_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to read challenge"))?
            .ok_or_else(|| unauthorized("nonce_invalid", "auth verification failed"))?
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
                .ok_or_else(|| unauthorized("nonce_invalid", "auth verification failed"))?
        }
    };

    if challenge_record.identity_id != payload.identity_id {
        return Err(unauthorized("nonce_invalid", "auth verification failed"));
    }

    if Utc::now() > challenge_record.expires_at {
        return Err(unauthorized("nonce_invalid", "auth verification failed"));
    }

    let key_record = if let Some(pool) = state.db_pool.as_ref() {
        auth_repo::get_identity_key(pool, &payload.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to read identity key"))?
            .ok_or_else(|| unauthorized("nonce_invalid", "auth verification failed"))?
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
                .ok_or_else(|| unauthorized("nonce_invalid", "auth verification failed"))?
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
    .map_err(|_| unauthorized("nonce_invalid", "auth verification failed"))?;

    let identity_id = payload.identity_id.clone();

    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(12);

    if let Some(pool) = state.db_pool.as_ref() {
        auth_repo::insert_session(pool, &session_id, &identity_id, expires_at)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to persist session"))?;
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
        let updated = auth_repo::revoke_session(pool, &payload.session_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to revoke session"))?;

        if !updated {
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
    let header_value = HeaderValue::from_str(cookie_value)
        .map_err(|_| internal_error("cookie_invalid", "failed to encode cookie header"))?;
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

fn identity_rate_limit_key(
    state: &AppState,
    identity_hint: &str,
    headers: &HeaderMap,
    peer_addr: Option<SocketAddr>,
) -> String {
    format!(
        "identity:{}:source:{}",
        identity_hint,
        request_source_fingerprint(state, headers, peer_addr)
    )
}

fn source_rate_limit_key(
    state: &AppState,
    headers: &HeaderMap,
    peer_addr: Option<SocketAddr>,
) -> String {
    format!(
        "source:{}",
        request_source_fingerprint(state, headers, peer_addr)
    )
}

fn request_source_fingerprint(
    state: &AppState,
    headers: &HeaderMap,
    peer_addr: Option<SocketAddr>,
) -> String {
    if !state.trust_proxy_headers {
        return peer_addr
            .map(|value| format!("peer:{:016x}", stable_hash(&value.ip().to_string())))
            .unwrap_or_else(|| "unknown".to_string());
    }

    if let Some(value) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("xff:{:016x}", stable_hash(value));
    }

    if let Some(value) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("xri:{:016x}", stable_hash(value));
    }

    peer_addr
        .map(|value| format!("peer:{:016x}", stable_hash(&value.ip().to_string())))
        .unwrap_or_else(|| "unknown".to_string())
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<(), ()> {
    let key = UnparsedPublicKey::new(&ED25519, public_key);
    key.verify(message, signature).map_err(|_| ())
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
            .map_err(|error| {
                warn!(
                    scope = scope,
                    key = key,
                    error = %error,
                    "distributed rate-limit enforcement failed"
                );
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
