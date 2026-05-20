use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};

use crate::{
    models::{InviteCreateRequest, InviteCreateResponse},
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn create_invite(
    auth: AuthSession,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(payload): Json<InviteCreateRequest>,
) -> ApiResult<(StatusCode, Json<InviteCreateResponse>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_invite_create_request(&payload)?;
    Ok((
        StatusCode::CREATED,
        Json(InviteCreateResponse {
            invite_id: "inv-1".to_string(),
        }),
    ))
}

fn validate_invite_create_request(payload: &InviteCreateRequest) -> ApiResult<()> {
    if payload.mode != "one_time" && payload.mode != "multi_use" {
        return Err(bad_request(
            "invite_invalid",
            "mode must be one_time or multi_use",
        ));
    }
    Ok(())
}
