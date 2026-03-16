use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::Row;

use crate::{
    infra::crypto::session_token::validate_session_token,
    models::ApiError,
    shared::errors::{internal_error, unauthorized},
    state::AppState,
};

const SESSION_COOKIE_NAME: &str = "hexrelay_session";
const CSRF_COOKIE_NAME: &str = "hexrelay_csrf";
const CSRF_HEADER_NAME: &str = "x-csrf-token";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

        let Some((token, transport)) = select_auth_token(cookie_token, bearer_token) else {
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

fn select_auth_token(
    cookie_token: Option<&str>,
    bearer_token: Option<&str>,
) -> Option<(String, AuthTransport)> {
    if let Some(token) = bearer_token {
        return Some((token.to_string(), AuthTransport::Bearer));
    }

    cookie_token.map(|token| (token.to_string(), AuthTransport::Cookie))
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
    .map_err(|_| internal_error("storage_unavailable", "session lookup failed"))?
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

#[cfg(test)]
mod tests {
    use super::{
        cookie_value, enforce_csrf_for_cookie_auth, parse_bearer_token, resolve_memory_session,
        select_auth_token, AuthInput, AuthSession, AuthTransport,
    };
    use crate::{
        infra::crypto::session_token::issue_session_token, models::SessionRecord, state::AppState,
    };
    use axum::http::HeaderMap;
    use chrono::{Duration, Utc};

    #[test]
    fn parses_bearer_token_and_cookie_values() {
        assert_eq!(parse_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(parse_bearer_token("Token abc123"), None);

        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "a=1; hexrelay_session=session-token"
                .parse()
                .expect("header"),
        );
        assert_eq!(
            cookie_value(&headers, "hexrelay_session"),
            Some("session-token")
        );
        assert_eq!(cookie_value(&headers, "missing"), None);
    }

    #[test]
    fn select_auth_token_prefers_bearer_when_both_present() {
        let selected =
            select_auth_token(Some("cookie-token"), Some("bearer-token")).expect("select token");
        assert_eq!(selected.0, "bearer-token");
        assert_eq!(selected.1, AuthTransport::Bearer);

        let cookie_only = select_auth_token(Some("cookie-token"), None).expect("cookie token");
        assert_eq!(cookie_only.0, "cookie-token");
        assert_eq!(cookie_only.1, AuthTransport::Cookie);

        assert!(select_auth_token(None, None).is_none());
    }

    #[test]
    fn csrf_enforcement_accepts_bearer_and_valid_cookie_flow() {
        let bearer_auth = AuthSession {
            session_id: "sess-1".to_string(),
            identity_id: "usr-1".to_string(),
            expires_at: Utc::now().to_rfc3339(),
            transport: AuthTransport::Bearer,
        };
        assert!(enforce_csrf_for_cookie_auth(&bearer_auth, &HeaderMap::new()).is_ok());

        let cookie_auth = AuthSession {
            transport: AuthTransport::Cookie,
            ..bearer_auth
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "hexrelay_csrf=token-123".parse().expect("cookie header"),
        );
        headers.insert("x-csrf-token", "token-123".parse().expect("csrf header"));
        assert!(enforce_csrf_for_cookie_auth(&cookie_auth, &headers).is_ok());

        headers.insert("x-csrf-token", "mismatch".parse().expect("csrf mismatch"));
        assert!(enforce_csrf_for_cookie_auth(&cookie_auth, &headers).is_err());
    }

    #[test]
    fn csrf_enforcement_rejects_missing_cookie_or_header_token() {
        let auth = AuthSession {
            session_id: "sess-1".to_string(),
            identity_id: "usr-1".to_string(),
            expires_at: Utc::now().to_rfc3339(),
            transport: AuthTransport::Cookie,
        };

        let mut missing_cookie = HeaderMap::new();
        missing_cookie.insert("x-csrf-token", "token-123".parse().expect("csrf header"));
        assert!(enforce_csrf_for_cookie_auth(&auth, &missing_cookie).is_err());

        let mut missing_header = HeaderMap::new();
        missing_header.insert(
            "cookie",
            "hexrelay_csrf=token-123".parse().expect("csrf cookie"),
        );
        assert!(enforce_csrf_for_cookie_auth(&auth, &missing_header).is_err());
    }

    #[test]
    fn resolve_memory_session_handles_token_mismatch_and_success() {
        let app_state = AppState::default();
        let session_id = "sess-test-1".to_string();
        let identity_id = "usr-auth".to_string();
        let expires_at = Utc::now() + Duration::hours(1);

        app_state.sessions.write().expect("session lock").insert(
            session_id.clone(),
            SessionRecord {
                identity_id: identity_id.clone(),
                expires_at,
            },
        );

        let ok_input = AuthInput {
            session_id: session_id.clone(),
            token_identity_id: identity_id.clone(),
            token_expires_at: expires_at.timestamp(),
        };
        let resolved = match resolve_memory_session(&app_state, &ok_input) {
            Ok(value) => value,
            Err(_) => panic!("session resolves"),
        };
        assert_eq!(resolved.session_id, session_id);

        let bad_identity = AuthInput {
            token_identity_id: "usr-other".to_string(),
            ..ok_input
        };
        assert!(resolve_memory_session(&app_state, &bad_identity).is_err());
    }

    #[test]
    fn resolve_memory_session_rejects_missing_expired_and_expiry_mismatch() {
        let app_state = AppState::default();

        let missing = AuthInput {
            session_id: "missing".to_string(),
            token_identity_id: "usr-1".to_string(),
            token_expires_at: Utc::now().timestamp(),
        };
        assert!(resolve_memory_session(&app_state, &missing).is_err());

        let expired_session_id = "sess-expired".to_string();
        let expired_at = Utc::now() - Duration::minutes(1);
        app_state.sessions.write().expect("session lock").insert(
            expired_session_id.clone(),
            SessionRecord {
                identity_id: "usr-1".to_string(),
                expires_at: expired_at,
            },
        );
        let expired_input = AuthInput {
            session_id: expired_session_id,
            token_identity_id: "usr-1".to_string(),
            token_expires_at: expired_at.timestamp(),
        };
        assert!(resolve_memory_session(&app_state, &expired_input).is_err());

        let valid_session_id = "sess-valid".to_string();
        let valid_expires_at = Utc::now() + Duration::hours(1);
        app_state.sessions.write().expect("session lock").insert(
            valid_session_id.clone(),
            SessionRecord {
                identity_id: "usr-2".to_string(),
                expires_at: valid_expires_at,
            },
        );
        let expiry_mismatch = AuthInput {
            session_id: valid_session_id,
            token_identity_id: "usr-2".to_string(),
            token_expires_at: valid_expires_at.timestamp() + 1,
        };
        assert!(resolve_memory_session(&app_state, &expiry_mismatch).is_err());
    }

    #[test]
    fn issued_session_token_validates_with_default_keyring() {
        let state = AppState::default();
        let expires_at = (Utc::now() + Duration::hours(1)).timestamp();
        let secret = state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active key exists");

        let token = issue_session_token(
            "sess-issued",
            "usr-issued",
            expires_at,
            &state.active_signing_key_id,
            secret,
        );

        let claims = crate::infra::crypto::session_token::validate_session_token(
            &token,
            &state.session_signing_keys,
        )
        .expect("token validates");

        assert_eq!(claims.session_id, "sess-issued");
        assert_eq!(claims.identity_id, "usr-issued");
    }
}
