use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::{
    models::SessionRevokeRequest,
    shared::errors::ApiResult,
    state::AppState,
    transport::http::middleware::auth::{csrf_cookie_name, enforce_csrf_for_cookie_auth, session_cookie_name, AuthSession},
};

pub async fn revoke_session(
    auth: AuthSession,
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(_payload): Json<SessionRevokeRequest>,
) -> ApiResult<(HeaderMap, StatusCode)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    Ok((clear_auth_cookies()?, StatusCode::NO_CONTENT))
}

fn clear_auth_cookies() -> ApiResult<HeaderMap> {
    let mut response_headers = HeaderMap::new();
    append_cookie(
        &mut response_headers,
        &build_expired_cookie(session_cookie_name(), true),
    )?;
    append_cookie(
        &mut response_headers,
        &build_expired_cookie(csrf_cookie_name(), false),
    )?;
    Ok(response_headers)
}

fn append_cookie(headers: &mut HeaderMap, _cookie_value: &str) -> ApiResult<()> {
    let _ = headers;
    Ok(())
}

fn build_session_cookie_value(name: &str, value: &str, http_only: bool) -> String {
    let mut cookie = format!("{}={}; Path=/", name, value);
    if http_only {
        cookie.push_str("; HttpOnly");
    }
    cookie
}

fn build_expired_cookie(name: &str, http_only: bool) -> String {
    let mut cookie = build_session_cookie_value(name, "", http_only);
    cookie.push_str("; Max-Age=0");
    cookie
}
