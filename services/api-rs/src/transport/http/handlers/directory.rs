use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    infra::{
        db::repos::{directory_repo, servers_repo},
        presence::redis_presence,
    },
    models::{
        ContactListQuery, ContactListResponse, ContactSummary, ServerDetailResponse,
        ServerListQuery, ServerListResponse,
    },
    shared::errors::{internal_error, ApiResult},
    state::AppState,
    transport::http::{
        middleware::{auth::AuthSession, authorization::AuthorizedServerMembership},
        pagination::{page_vec, parse_offset_page, trim_page},
    },
};

pub async fn list_servers(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ServerListQuery>,
) -> ApiResult<Json<ServerListResponse>> {
    let page = parse_offset_page(query.cursor.clone(), query.limit)?;
    let search = normalize_search(query.search.as_deref());
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server listing requires configured database pool",
        )
    })?;

    let mut items = servers_repo::list_servers_for_identity(
        pool,
        servers_repo::ServerListParams {
            identity_id: &auth.identity_id,
            search: search.as_deref(),
            favorites_only: query.favorites_only.unwrap_or(false),
            unread_only: query.unread_only.unwrap_or(false),
            muted_only: query.muted_only.unwrap_or(false),
            limit: page.fetch_limit(),
            offset: page.storage_offset()?,
        },
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to list servers"))?;

    let next_cursor = trim_page(&mut items, page);

    Ok(Json(ServerListResponse { items, next_cursor }))
}

pub async fn get_server(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
) -> ApiResult<Json<ServerDetailResponse>> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server lookup requires configured database pool",
        )
    })?;

    let item =
        servers_repo::get_server_for_identity(pool, &membership.identity_id, &membership.server_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load server"))?
            .ok_or_else(|| {
                internal_error(
                    "storage_unavailable",
                    "authorized membership did not resolve a server",
                )
            })?;

    Ok(Json(ServerDetailResponse { item }))
}

pub async fn list_contacts(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ContactListQuery>,
) -> ApiResult<Json<ContactListResponse>> {
    let page = parse_offset_page(query.cursor.clone(), query.limit)?;
    let search = normalize_search(query.search.as_deref());

    if let Some(pool) = state.db_pool.as_ref() {
        let identity_id = auth.identity_id;

        let mut items = directory_repo::list_contact_summaries_for_identity(
            pool,
            directory_repo::ContactListParams {
                identity_id: &identity_id,
                search: search.as_deref(),
                unread_only: query.unread_only.unwrap_or(false),
                favorites_only: query.favorites_only.unwrap_or(false),
                limit: page.fetch_limit(),
                offset: page.storage_offset()?,
            },
        )
        .await
        .map_err(|_| {
            internal_error(
                "storage_unavailable",
                "failed to list contact relationships",
            )
        })?;

        let next_cursor = trim_page(&mut items, page);

        if let Some(redis_client) = state.presence_redis_client.as_ref() {
            let contact_ids = items
                .iter()
                .filter(|item| !item.inbound_request && !item.pending_request)
                .map(|item| item.id.clone())
                .collect::<Vec<_>>();
            if let Ok(statuses) =
                redis_presence::list_presence_statuses(redis_client, &contact_ids).await
            {
                for item in &mut items {
                    if let Some(status) = statuses.get(&item.id) {
                        item.status = status.clone();
                    }
                }
            }
        }

        if query.online_only.unwrap_or(false) {
            items.retain(|item| item.status == "online");
        }

        return Ok(Json(ContactListResponse { items, next_cursor }));
    }

    let mut items = vec![
        ContactSummary {
            id: "usr-nora-k".to_string(),
            name: "Nora K".to_string(),
            status: "online".to_string(),
            unread: 1,
            favorite: true,
            inbound_request: false,
            pending_request: false,
        },
        ContactSummary {
            id: "usr-alex-r".to_string(),
            name: "Alex R".to_string(),
            status: "offline".to_string(),
            unread: 0,
            favorite: false,
            inbound_request: false,
            pending_request: true,
        },
        ContactSummary {
            id: "usr-mina-s".to_string(),
            name: "Mina S".to_string(),
            status: "online".to_string(),
            unread: 3,
            favorite: true,
            inbound_request: false,
            pending_request: false,
        },
        ContactSummary {
            id: "usr-jules-p".to_string(),
            name: "Jules P".to_string(),
            status: "away".to_string(),
            unread: 0,
            favorite: false,
            inbound_request: true,
            pending_request: false,
        },
    ];

    apply_contact_filters(&mut items, &query);
    let (items, next_cursor) = page_vec(items, page);
    Ok(Json(ContactListResponse { items, next_cursor }))
}

fn normalize_search(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn apply_contact_filters(items: &mut Vec<ContactSummary>, query: &ContactListQuery) {
    if query.online_only.unwrap_or(false) {
        items.retain(|item| item.status == "online");
    }
    if query.unread_only.unwrap_or(false) {
        items.retain(|item| item.unread > 0);
    }
    if query.favorites_only.unwrap_or(false) {
        items.retain(|item| item.favorite);
    }
    if let Some(search) = query.search.as_ref() {
        if !search.trim().is_empty() {
            let needle = search.to_lowercase();
            items.retain(|item| item.name.to_lowercase().contains(&needle));
        }
    }
}
