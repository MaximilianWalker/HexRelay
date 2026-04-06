use axum::{extract::{Query, State}, Json};

use crate::{
    models::{DiscoveryUserListQuery, DiscoveryUserListResponse},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_discovery_users(
    State(_state): State<AppState>,
    _auth: AuthSession,
    Query(query): Query<DiscoveryUserListQuery>,
) -> Result<Json<DiscoveryUserListResponse>, ()> {
    let _scope = normalize_scope(query.scope.as_deref()).unwrap_or("global");
    let _search = normalize_search(query.query.as_deref());
    let _limit = normalize_limit(query.limit);
    Ok(Json(DiscoveryUserListResponse { items: Vec::new() }))
}

fn normalize_scope(scope: Option<&str>) -> Result<&'static str, ()> {
    match scope.unwrap_or("global").trim() {
        "global" => Ok("global"),
        "shared_server" => Ok("shared_server"),
        _ => Err(()),
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
