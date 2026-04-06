use axum::{extract::State, Json};

use crate::{
    models::{AuthChallengeRequest, AuthChallengeResponse},
    shared::errors::{bad_request, internal_error, too_many_requests, ApiResult},
    state::AppState,
};

pub async fn issue_auth_challenge(
    State(state): State<AppState>,
    Json(payload): Json<AuthChallengeRequest>,
) -> ApiResult<Json<AuthChallengeResponse>> {
    validate_auth_challenge_request(&payload)?;
    let allowed = allow_rate_limit(&state, &payload.identity_id).await?;
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many auth challenge requests",
        ));
    }

    Ok(Json(AuthChallengeResponse {
        challenge_id: "challenge_1".to_string(),
    }))
}

fn validate_auth_challenge_request(
    payload: &AuthChallengeRequest,
) -> Result<(), (axum::http::StatusCode, axum::Json<crate::shared::errors::ApiError>)> {
    if payload.identity_id.trim().is_empty() {
        return Err(bad_request(
            "auth_challenge_invalid",
            "identity_id is required",
        ));
    }
    Ok(())
}

async fn allow_rate_limit(
    _state: &AppState,
    _identity_id: &str,
) -> Result<bool, (axum::http::StatusCode, axum::Json<crate::shared::errors::ApiError>)> {
    Err(rate_limit_failure())
}

fn rate_limit_failure() -> (axum::http::StatusCode, axum::Json<crate::shared::errors::ApiError>) {
    internal_error(
        "storage_unavailable",
        "rate limiter unavailable while issuing auth challenge",
    )
}
