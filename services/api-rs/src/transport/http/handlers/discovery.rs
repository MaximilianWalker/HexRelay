use std::collections::{HashMap, HashSet};

use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    infra::db::repos::discovery_repo,
    models::{
        DiscoveryUserListQuery, DiscoveryUserListResponse, DiscoveryUserRecord,
        DiscoveryUserSummary,
    },
    shared::errors::{bad_request, internal_error, too_many_requests, ApiResult},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_discovery_users(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<DiscoveryUserListQuery>,
) -> ApiResult<Json<DiscoveryUserListResponse>> {
    let scope = normalize_scope(query.scope.as_deref())?;
    let search = normalize_search(query.query.as_deref());
    let limit = normalize_limit(query.limit);

    let allowed = state.rate_limiter.allow(
        "discovery_query",
        &auth.identity_id,
        state.rate_limits.discovery_query_per_window,
        state.rate_limits.window_seconds,
    );
    if !allowed {
        return Err(too_many_requests(
            "rate_limited",
            "too many discovery queries in current window",
        ));
    }

    if let Some(pool) = state.db_pool.as_ref() {
        let candidates = match scope {
            "global" => {
                discovery_repo::list_global_discovery_candidates(pool, &auth.identity_id).await
            }
            _ => {
                discovery_repo::list_shared_server_discovery_candidates(pool, &auth.identity_id)
                    .await
            }
        }
        .map_err(|_| internal_error("storage_unavailable", "failed to list discovery users"))?;

        let relationships = discovery_repo::list_relationship_rows(pool, &auth.identity_id)
            .await
            .map_err(|_| {
                internal_error(
                    "storage_unavailable",
                    "failed to list discovery relationships",
                )
            })?;

        let shared_counts = discovery_repo::shared_server_counts(pool, &auth.identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to list shared-server counts")
            })?;

        let blocked = in_memory_blocked_peers(&state, &auth.identity_id);

        let items = build_discovery_items(DiscoveryBuildInput {
            actor_identity_id: &auth.identity_id,
            candidates,
            relationships,
            shared_counts,
            blocked,
            denylist: state.discovery_denylist.as_ref(),
            search: search.as_deref(),
            scope,
            limit,
        });
        return Ok(Json(DiscoveryUserListResponse { items }));
    }

    #[cfg(test)]
    {
        let candidates = in_memory_candidates(&state, &auth.identity_id, scope);
        let relationships = in_memory_relationships(&state, &auth.identity_id);
        let shared_counts = in_memory_shared_counts(&state, &auth.identity_id);
        let blocked = in_memory_blocked_peers(&state, &auth.identity_id);
        let items = build_discovery_items(DiscoveryBuildInput {
            actor_identity_id: &auth.identity_id,
            candidates,
            relationships,
            shared_counts,
            blocked,
            denylist: state.discovery_denylist.as_ref(),
            search: search.as_deref(),
            scope,
            limit,
        });
        Ok(Json(DiscoveryUserListResponse { items }))
    }

    #[cfg(not(test))]
    Err(internal_error(
        "storage_unavailable",
        "discovery requires configured database pool",
    ))
}

fn normalize_scope(scope: Option<&str>) -> ApiResult<&'static str> {
    match scope.unwrap_or("global").trim() {
        "global" => Ok("global"),
        "shared_server" => Ok("shared_server"),
        _ => Err(bad_request(
            "scope_invalid",
            "scope must be either global or shared_server",
        )),
    }
}

fn normalize_search(query: Option<&str>) -> Option<String> {
    query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
}

fn normalize_limit(limit: Option<u32>) -> usize {
    limit.unwrap_or(20).clamp(1, 50) as usize
}

struct DiscoveryBuildInput<'a> {
    actor_identity_id: &'a str,
    candidates: Vec<DiscoveryUserRecord>,
    relationships: Vec<discovery_repo::DiscoveryRelationshipRow>,
    shared_counts: HashMap<String, u32>,
    blocked: HashSet<String>,
    denylist: &'a [String],
    search: Option<&'a str>,
    scope: &'a str,
    limit: usize,
}

fn build_discovery_items(input: DiscoveryBuildInput<'_>) -> Vec<DiscoveryUserSummary> {
    let DiscoveryBuildInput {
        actor_identity_id,
        candidates,
        relationships,
        shared_counts,
        blocked,
        denylist,
        search,
        scope,
        limit,
    } = input;

    let denylist = denylist.iter().cloned().collect::<HashSet<_>>();
    let mut relationship_index = HashMap::new();

    for row in relationships {
        relationship_index
            .entry(row.peer_identity_id)
            .or_insert((row.status, row.requester_is_self));
    }

    let mut items = candidates
        .into_iter()
        .filter(|candidate| candidate.identity_id != actor_identity_id)
        .filter(|candidate| !blocked.contains(&candidate.identity_id))
        .filter(|candidate| !denylist.contains(&candidate.identity_id))
        .filter(|candidate| {
            if scope == "shared_server" {
                shared_counts
                    .get(&candidate.identity_id)
                    .copied()
                    .unwrap_or_default()
                    > 0
            } else {
                true
            }
        })
        .filter(|candidate| match search {
            Some(needle) => {
                candidate.identity_id.to_lowercase().contains(needle)
                    || candidate.display_name.to_lowercase().contains(needle)
            }
            None => true,
        })
        .map(|candidate| {
            let shared_server_count = shared_counts
                .get(&candidate.identity_id)
                .copied()
                .unwrap_or_default();
            let (relationship_state, requester_is_self) = relationship_index
                .get(&candidate.identity_id)
                .cloned()
                .unwrap_or_else(|| ("none".to_string(), false));

            let has_pending_inbound_request = relationship_state == "pending" && !requester_is_self;
            let has_pending_outbound_request = relationship_state == "pending" && requester_is_self;
            let can_send_friend_request = relationship_state == "none";

            DiscoveryUserSummary {
                identity_id: candidate.identity_id,
                display_name: candidate.display_name,
                avatar_url: candidate.avatar_url,
                relationship_state,
                shared_server_count,
                can_send_friend_request,
                has_pending_inbound_request,
                has_pending_outbound_request,
            }
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| {
        b.shared_server_count
            .cmp(&a.shared_server_count)
            .then_with(|| a.display_name.cmp(&b.display_name))
            .then_with(|| a.identity_id.cmp(&b.identity_id))
    });
    items.truncate(limit);
    items
}

#[cfg(test)]
fn in_memory_candidates(
    state: &AppState,
    actor_identity_id: &str,
    scope: &str,
) -> Vec<DiscoveryUserRecord> {
    let mut ids = HashSet::new();

    let relationships = state
        .friend_requests
        .read()
        .expect("acquire friend_requests read lock");
    for request in relationships.values() {
        if request.requester_identity_id == actor_identity_id {
            ids.insert(request.target_identity_id.clone());
        } else if request.target_identity_id == actor_identity_id {
            ids.insert(request.requester_identity_id.clone());
        } else if scope == "global" {
            ids.insert(request.requester_identity_id.clone());
            ids.insert(request.target_identity_id.clone());
        }
    }

    let identity_keys = state
        .identity_keys
        .read()
        .expect("acquire identity_keys read lock");
    if scope == "global" {
        ids.extend(identity_keys.keys().cloned());
    }

    ids.into_iter()
        .filter(|identity_id| identity_id != actor_identity_id)
        .map(|identity_id| DiscoveryUserRecord {
            display_name: identity_id.clone(),
            identity_id,
            avatar_url: None,
        })
        .collect()
}

#[cfg(test)]
fn in_memory_relationships(
    state: &AppState,
    actor_identity_id: &str,
) -> Vec<discovery_repo::DiscoveryRelationshipRow> {
    let requests = state
        .friend_requests
        .read()
        .expect("acquire friend_requests read lock");

    requests
        .values()
        .filter_map(|request| relationship_from_request(request, actor_identity_id))
        .collect()
}

#[cfg(test)]
fn relationship_from_request(
    request: &crate::models::FriendRequestRecord,
    actor_identity_id: &str,
) -> Option<discovery_repo::DiscoveryRelationshipRow> {
    if request.requester_identity_id == actor_identity_id {
        Some(discovery_repo::DiscoveryRelationshipRow {
            peer_identity_id: request.target_identity_id.clone(),
            status: request.status.clone(),
            requester_is_self: true,
        })
    } else if request.target_identity_id == actor_identity_id {
        Some(discovery_repo::DiscoveryRelationshipRow {
            peer_identity_id: request.requester_identity_id.clone(),
            status: request.status.clone(),
            requester_is_self: false,
        })
    } else {
        None
    }
}

#[cfg(test)]
fn in_memory_shared_counts(_state: &AppState, _actor_identity_id: &str) -> HashMap<String, u32> {
    HashMap::new()
}

fn in_memory_blocked_peers(state: &AppState, actor_identity_id: &str) -> HashSet<String> {
    let guard = state
        .blocked_users
        .read()
        .expect("acquire blocked_users read lock");

    let mut blocked = HashSet::new();
    if let Some(items) = guard.get(actor_identity_id) {
        blocked.extend(items.keys().cloned());
    }

    for (blocker, items) in guard.iter() {
        if blocker != actor_identity_id && items.contains_key(actor_identity_id) {
            blocked.insert(blocker.clone());
        }
    }

    blocked
}
