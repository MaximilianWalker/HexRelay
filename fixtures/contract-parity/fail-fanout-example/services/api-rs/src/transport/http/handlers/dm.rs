use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    models::{DmFanoutDispatchRequest, DmFanoutDispatchResponse},
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn run_dm_active_fanout(
    auth: AuthSession,
    headers: HeaderMap,
    State(_state): State<AppState>,
    Json(payload): Json<DmFanoutDispatchRequest>,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_fanout_dispatch(&payload)?;
    Ok(Json(DmFanoutDispatchResponse {}))
}

fn validate_fanout_dispatch(payload: &DmFanoutDispatchRequest) -> ApiResult<()> {
    if payload.recipient_identity_id.trim().is_empty() {
        return Err(bad_request(
            "fanout_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    Ok(())
}
