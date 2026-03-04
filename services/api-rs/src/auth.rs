use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{models::ApiError, session_token::validate_session_token, state::AppState};

#[derive(Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub identity_id: String,
    pub expires_at: String,
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

        let bearer_token = parts
            .headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(parse_bearer_token);

        let auth_input = if let Some(token) = bearer_token {
            let claims = validate_session_token(token, &app_state.session_signing_key).ok_or({
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
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: "session_invalid",
                    message: "missing authorization header",
                }),
            ));
        };

        let session = if let Some(pool) = app_state.db_pool.as_ref() {
            resolve_db_session(pool, &auth_input).await?
        } else {
            resolve_memory_session(&app_state, &auth_input)?
        };

        Ok(Self {
            session_id: session.session_id,
            identity_id: session.identity_id,
            expires_at: session.expires_at,
        })
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
