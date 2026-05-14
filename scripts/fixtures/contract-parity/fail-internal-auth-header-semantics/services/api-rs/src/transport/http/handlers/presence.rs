use std::collections::BTreeSet;

use axum::{extract::{Path, State}, http::HeaderMap, Json};
use serde::Serialize;

use crate::{
    shared::errors::{unauthorized, ApiResult},
    state::AppState,
};

#[derive(Serialize)]
pub struct PresenceWatcherListResponse {
    pub watchers: Vec<String>,
}

pub async fn list_presence_watchers(
    Path(identity_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<PresenceWatcherListResponse>> {
    let internal_token = headers
        .get("x-hexrelay-internal-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if internal_token != Some(state.presence_watcher_internal_token.as_str()) {
        return Err(unauthorized(
            "internal_token_invalid",
            "presence watcher lookup requires a valid internal token",
        ));
    }

    let watchers = BTreeSet::from([identity_id]).into_iter().collect();
    Ok(Json(PresenceWatcherListResponse { watchers }))
}
