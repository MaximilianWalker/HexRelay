use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    Json,
};
use chrono::Utc;

use crate::{models::ApiError, state::AppState};

#[derive(Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub identity_id: String,
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

        let session_id = parts
            .headers
            .get("x-session-id")
            .and_then(|value| value.to_str().ok())
            .ok_or({
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        code: "session_invalid",
                        message: "missing x-session-id header",
                    }),
                )
            })?
            .to_string();

        let session = app_state
            .sessions
            .read()
            .expect("acquire session read lock")
            .get(&session_id)
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

        Ok(Self {
            session_id,
            identity_id: session.identity_id,
        })
    }
}
