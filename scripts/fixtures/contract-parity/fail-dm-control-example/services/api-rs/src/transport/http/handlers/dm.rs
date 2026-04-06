use axum::{extract::State, http::HeaderMap, Json};

use crate::{
    models::{DmPolicy, DmPolicyUpdate},
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

pub async fn update_dm_policy(
    State(_state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmPolicyUpdate>,
) -> ApiResult<Json<DmPolicy>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_dm_policy_update(&payload)?;
    Ok(Json(DmPolicy {
        inbound_policy: payload.inbound_policy,
    }))
}

fn validate_dm_policy_update(payload: &DmPolicyUpdate) -> ApiResult<()> {
    let value = payload.inbound_policy.trim();
    if value.is_empty() {
        return Err(bad_request("dm_policy_invalid", "inbound_policy must not be empty"));
    }
    if !matches!(value, "friends_only" | "same_server" | "anyone") {
        return Err(bad_request(
            "dm_policy_invalid",
            "inbound_policy must be one of: friends_only, same_server, anyone",
        ));
    }
    Ok(())
}
