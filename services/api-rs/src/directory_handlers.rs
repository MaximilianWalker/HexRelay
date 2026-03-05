use std::collections::{BTreeSet, HashSet};

use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::Row;

use crate::{
    auth::AuthSession,
    errors::{internal_error, ApiResult},
    models::{
        ContactListQuery, ContactListResponse, ContactSummary, ServerListQuery, ServerListResponse,
        ServerSummary,
    },
    state::AppState,
};

pub async fn list_servers(
    _auth: AuthSession,
    Query(query): Query<ServerListQuery>,
) -> Json<ServerListResponse> {
    let mut items = vec![
        ServerSummary {
            id: "srv-atlas-core".to_string(),
            name: "Atlas Core".to_string(),
            unread: 2,
            favorite: true,
            muted: false,
        },
        ServerSummary {
            id: "srv-relay-lab".to_string(),
            name: "Relay Lab".to_string(),
            unread: 0,
            favorite: false,
            muted: true,
        },
        ServerSummary {
            id: "srv-dev-signals".to_string(),
            name: "Dev Signals".to_string(),
            unread: 5,
            favorite: true,
            muted: false,
        },
        ServerSummary {
            id: "srv-ops-watch".to_string(),
            name: "Ops Watch".to_string(),
            unread: 0,
            favorite: false,
            muted: false,
        },
    ];

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

    Json(ServerListResponse { items })
}

pub async fn list_contacts(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ContactListQuery>,
) -> ApiResult<Json<ContactListResponse>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let identity_id = auth.identity_id;

        let rows = sqlx::query(
            "
            SELECT requester_identity_id, target_identity_id, status
            FROM friend_requests
            WHERE requester_identity_id = $1 OR target_identity_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(&identity_id)
        .fetch_all(pool)
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
            let requester = row
                .try_get::<String, _>("requester_identity_id")
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to decode requester identity")
                })?;
            let target = row
                .try_get::<String, _>("target_identity_id")
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to decode target identity")
                })?;
            let status = row.try_get::<String, _>("status").map_err(|_| {
                internal_error("storage_unavailable", "failed to decode request status")
            })?;

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
