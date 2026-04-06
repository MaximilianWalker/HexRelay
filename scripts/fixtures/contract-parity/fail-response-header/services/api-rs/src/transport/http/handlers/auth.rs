use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};

use crate::{
    models::{AuthVerifyRequest, AuthVerifyResponse},
    shared::errors::ApiResult,
    state::AppState,
};

pub async fn verify_auth_challenge(
    State(_state): State<AppState>,
    Json(_payload): Json<AuthVerifyRequest>,
) -> ApiResult<(HeaderMap, Json<AuthVerifyResponse>)> {
    let mut response_headers = HeaderMap::new();
    append_cookie(&mut response_headers, "hexrelay_session=issued; Path=/; HttpOnly")?;
    Ok((response_headers, Json(AuthVerifyResponse)))
}

fn append_cookie(headers: &mut HeaderMap, _cookie_value: &str) -> ApiResult<()> {
    let _ = headers;
    Ok(())
}
