use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{
    infra::crypto::session_token::validate_session_token, models::ApiError,
    shared::errors::unauthorized, state::AppState,
};

const SESSION_COOKIE_NAME: &str = "hexrelay_session";
const CSRF_COOKIE_NAME: &str = "hexrelay_csrf";
const CSRF_HEADER_NAME: &str = "x-csrf-token";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AuthTransport {
    Cookie,
    Bearer,
}

#[derive(Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub identity_id: String,
    pub expires_at: String,
    pub transport: AuthTransport,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<ApiError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let cookie_token = cookie_value(&parts.headers, SESSION_COOKIE_NAME);
        let bearer_token = parts
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(parse_bearer_token);

        let (token, transport) = if let Some(token) = cookie_token {
            (token.to_string(), AuthTransport::Cookie)
        } else if let Some(token) = bearer_token {
            (token.to_string(), AuthTransport::Bearer)
        } else {
            return Err(unauthorized(
                "session_invalid",
                "missing session cookie or authorization header",
            ));
        };

        let auth_input = {
            let claims =
                validate_session_token(&token, &app_state.session_signing_keys).ok_or({
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(ApiError {
                            code: "session_invalid",
                            message: "invalid bearer token",
                        }),
                    )
                })?;

            AuthInput {
                session_id: claims.session_id,
                token_identity_id: claims.identity_id,
                token_expires_at: claims.expires_at,
            }
        };

        let session = if let Some(pool) = app_state.db_pool.as_ref() {
            resolve_db_session(pool, &auth_input).await?
        } else {
            #[cfg(test)]
            {
                resolve_memory_session(&app_state, &auth_input)?
            }

            #[cfg(not(test))]
            {
                return Err(crate::shared::errors::internal_error(
                    "storage_unavailable",
                    "session validation requires configured database pool",
                ));
            }
        };

        Ok(Self {
            session_id: session.session_id,
            identity_id: session.identity_id,
            expires_at: session.expires_at,
            transport,
        })
    }
}

pub fn csrf_cookie_name() -> &'static str {
    CSRF_COOKIE_NAME
}

pub fn session_cookie_name() -> &'static str {
    SESSION_COOKIE_NAME
}

pub fn enforce_csrf_for_cookie_auth(
    auth: &AuthSession,
    headers: &HeaderMap,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if auth.transport != AuthTransport::Cookie {
        return Ok(());
    }

    let cookie_token = cookie_value(headers, CSRF_COOKIE_NAME);
    let header_token = headers
        .get(CSRF_HEADER_NAME)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (cookie_token, header_token) {
        (Some(cookie), Some(header)) if cookie == header => Ok(()),
        _ => Err(unauthorized(
            "csrf_invalid",
            "missing or invalid csrf token",
        )),
    }
}

struct AuthInput {
    session_id: String,
    token_identity_id: String,
    token_expires_at: i64,
}

struct ResolvedSession {
    session_id: String,
    identity_id: String,
    expires_at: String,
}

fn parse_bearer_token(raw: &str) -> Option<&str> {
    raw.strip_prefix("Bearer ")
}

pub fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let raw = headers.get("cookie")?.to_str().ok()?;
    for pair in raw.split(';') {
        let trimmed = pair.trim();
        let Some((cookie_name, cookie_value)) = trimmed.split_once('=') else {
            continue;
        };
        if cookie_name.trim() == name {
            return Some(cookie_value.trim());
        }
    }

    None
}

async fn resolve_db_session(
    pool: &sqlx::PgPool,
    input: &AuthInput,
) -> Result<ResolvedSession, (StatusCode, Json<ApiError>)> {
    let row = sqlx::query(
        "
        SELECT session_id, identity_id, expires_at, revoked_at
        FROM sessions
        WHERE session_id = $1
        ",
    )
    .bind(&input.session_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "session lookup failed",
            }),
        )
    })?
    .ok_or({
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "session not found",
            }),
        )
    })?;

    let revoked_at = row
        .try_get::<Option<chrono::DateTime<Utc>>, _>("revoked_at")
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: "session_invalid",
                    message: "invalid session row",
                }),
            )
        })?;

    if revoked_at.is_some() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "session revoked",
            }),
        ));
    }

    let identity_id = row.try_get::<String, _>("identity_id").map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "invalid session row",
            }),
        )
    })?;
    let expires_at = row
        .try_get::<chrono::DateTime<Utc>, _>("expires_at")
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: "session_invalid",
                    message: "invalid session row",
                }),
            )
        })?;

    if Utc::now() > expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "session expired",
            }),
        ));
    }

    if input.token_identity_id != identity_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "token identity mismatch",
            }),
        ));
    }

    if input.token_expires_at != expires_at.timestamp() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "token expiry mismatch",
            }),
        ));
    }

    Ok(ResolvedSession {
        session_id: input.session_id.clone(),
        identity_id,
        expires_at: expires_at.to_rfc3339(),
    })
}

#[cfg(test)]
fn resolve_memory_session(
    app_state: &AppState,
    input: &AuthInput,
) -> Result<ResolvedSession, (StatusCode, Json<ApiError>)> {
    let session = app_state
        .sessions
        .read()
        .expect("acquire session read lock")
        .get(&input.session_id)
        .cloned()
        .ok_or({
            (
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: "session_invalid",
                    message: "session not found",
                }),
            )
        })?;

    if Utc::now() > session.expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "session expired",
            }),
        ));
    }

    if input.token_identity_id != session.identity_id {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "token identity mismatch",
            }),
        ));
    }

    if input.token_expires_at != session.expires_at.timestamp() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: "session_invalid",
                message: "token expiry mismatch",
            }),
        ));
    }

    Ok(ResolvedSession {
        session_id: input.session_id.clone(),
        identity_id: session.identity_id,
        expires_at: session.expires_at.to_rfc3339(),
    })
}
