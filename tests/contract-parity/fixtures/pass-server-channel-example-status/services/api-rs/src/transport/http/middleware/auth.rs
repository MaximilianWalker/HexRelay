use axum::http::HeaderMap;

use crate::shared::errors::ApiResult;

pub struct AuthSession {
    pub session_id: String,
    pub identity_id: String,
}

pub fn enforce_csrf_for_cookie_auth(_auth: &AuthSession, _headers: &HeaderMap) -> ApiResult<()> {
    Ok(())
}
