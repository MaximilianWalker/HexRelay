use std::collections::BTreeSet;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use crate::{
    domain::block_mute::service::is_blocked_bidirectional,
    infra::db::repos::directory_repo,
    shared::errors::{internal_error, unauthorized, ApiResult},
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

    if internal_token != Some(state.presence_internal_token.as_str()) {
        return Err(unauthorized(
            "internal_token_invalid",
            "presence watcher lookup requires a valid internal token",
        ));
    }

    let mut watchers = BTreeSet::from([identity_id.clone()]);
    for peer_identity_id in accepted_friend_ids(&state, &identity_id).await? {
        if !is_blocked_bidirectional(&state, &identity_id, &peer_identity_id)? {
            watchers.insert(peer_identity_id);
        }
    }

    Ok(Json(PresenceWatcherListResponse {
        watchers: watchers.into_iter().collect(),
    }))
}

async fn accepted_friend_ids(state: &AppState, identity_id: &str) -> ApiResult<Vec<String>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let rows = directory_repo::list_contact_relationships(pool, identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to load friend relationships")
            })?;

        let mut peers = BTreeSet::new();
        for row in rows {
            if row.status != "accepted" {
                continue;
            }

            if row.requester_identity_id == identity_id {
                peers.insert(row.target_identity_id);
            } else {
                peers.insert(row.requester_identity_id);
            }
        }

        return Ok(peers.into_iter().collect());
    }

    let requests = state
        .friend_requests
        .read()
        .expect("acquire friend request read lock");
    let mut peers = BTreeSet::new();
    for request in requests.values() {
        if request.status != "accepted" {
            continue;
        }

        if request.requester_identity_id == identity_id {
            peers.insert(request.target_identity_id.clone());
        } else if request.target_identity_id == identity_id {
            peers.insert(request.requester_identity_id.clone());
        }
    }

    Ok(peers.into_iter().collect())
}
