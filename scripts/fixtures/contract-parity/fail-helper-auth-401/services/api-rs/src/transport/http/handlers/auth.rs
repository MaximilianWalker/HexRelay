use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};

use crate::{
    models::{AuthVerifyRequest, AuthVerifyResponse},
    shared::errors::{bad_request, unauthorized, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{csrf_cookie_name, session_cookie_name},
};

pub async fn verify_auth_challenge(
    State(_state): State<AppState>,
    Json(payload): Json<AuthVerifyRequest>,
) -> ApiResult<(HeaderMap, Json<AuthVerifyResponse>)> {
    if payload.challenge_id.trim().is_empty() {
        return Err(bad_request(
            "signature_invalid",
            "signature must be 64-byte hex or base64",
        ));
    }

    if payload.identity_id.trim().is_empty() {
        return Err(auth_verify_failure());
    }

    let mut response_headers = HeaderMap::new();
    append_cookie(
        &mut response_headers,
        &build_session_cookie_value(session_cookie_name(), "issued", true),
    )?;
    append_cookie(
        &mut response_headers,
        &build_session_cookie_value(csrf_cookie_name(), "csrf", false),
    )?;
    Ok((response_headers, Json(AuthVerifyResponse {})))
}

fn auth_verify_failure() -> (axum::http::StatusCode, axum::Json<crate::shared::errors::ApiError>) {
    unauthorized("challenge_invalid", "auth challenge is invalid or expired")
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
