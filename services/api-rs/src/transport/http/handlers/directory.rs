use std::collections::{BTreeSet, HashSet};

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
    transport::http::middleware::{auth::AuthSession, authorization::AuthorizedServerMembership},
};

pub async fn list_servers(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ServerListQuery>,
) -> ApiResult<Json<ServerListResponse>> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server listing requires configured database pool",
        )
    })?;

    let mut items = servers_repo::list_servers_for_identity(pool, &auth.identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to list servers"))?;

    if query.favorites_only.unwrap_or(false) {
        items.retain(|item| item.favorite);
    }
    if query.unread_only.unwrap_or(false) {
        items.retain(|item| item.unread > 0);
    }
    if query.muted_only.unwrap_or(false) {
        items.retain(|item| item.muted);
    }
    if let Some(search) = query.search.as_ref() {
        if !search.trim().is_empty() {
            let needle = search.to_lowercase();
            items.retain(|item| item.name.to_lowercase().contains(&needle));
        }
    }

    Ok(Json(ServerListResponse { items }))
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
            .expect("authorized membership must resolve server");

    Ok(Json(ServerDetailResponse { item }))
}

pub async fn list_contacts(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ContactListQuery>,
) -> ApiResult<Json<ContactListResponse>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let identity_id = auth.identity_id;

        let rows = directory_repo::list_contact_relationships(pool, &identity_id)
            .await
            .map_err(|_| {
                internal_error(
                    "storage_unavailable",
                    "failed to list contact relationships",
                )
            })?;

        let mut contacts = BTreeSet::new();
        let mut inbound_pending = HashSet::new();
        let mut outbound_pending = HashSet::new();

        for row in rows {
            let requester = row.requester_identity_id;
            let target = row.target_identity_id;
            let status = row.status;

            let requester_is_self = requester == identity_id;
            let peer = if requester_is_self {
                target
            } else {
                requester.clone()
            };
            if peer.is_empty() {
                continue;
            }

            if status == "accepted" {
                contacts.insert(peer.clone());
            } else if status == "pending" {
                contacts.insert(peer.clone());
                if requester_is_self {
                    outbound_pending.insert(peer);
                } else {
                    inbound_pending.insert(peer);
                }
            }
        }

        let mut items = contacts
            .into_iter()
            .map(|id| ContactSummary {
                id: id.clone(),
                name: id.clone(),
                status: "offline".to_string(),
                unread: 0,
                favorite: false,
                inbound_request: inbound_pending.contains(&id),
                pending_request: outbound_pending.contains(&id),
            })
            .collect::<Vec<_>>();

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

        apply_contact_filters(&mut items, &query);
        return Ok(Json(ContactListResponse { items }));
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
    Ok(Json(ContactListResponse { items }))
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
