use std::collections::{BTreeSet, HashSet};

use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    infra::db::repos::directory_repo,
    models::{
        ContactListQuery, ContactListResponse, ContactSummary, ServerListQuery, ServerListResponse,
        ServerSummary,
    },
    shared::errors::{internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::AuthSession,
};

pub async fn list_servers(
    auth: AuthSession,
    Query(query): Query<ServerListQuery>,
) -> Json<ServerListResponse> {
    let mut items = server_fixtures_for_identity(&auth.identity_id);

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

fn server_fixtures_for_identity(identity_id: &str) -> Vec<ServerSummary> {
    match identity_id {
        "usr-nora-k" => vec![
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
        ],
        "usr-alex-r" => vec![
            ServerSummary {
                id: "srv-alex-craft".to_string(),
                name: "Alex Craft".to_string(),
                unread: 1,
                favorite: true,
                muted: false,
            },
            ServerSummary {
                id: "srv-night-shift".to_string(),
                name: "Night Shift".to_string(),
                unread: 0,
                favorite: false,
                muted: true,
            },
        ],
        "usr-mina-s" => vec![
            ServerSummary {
                id: "srv-mina-labs".to_string(),
                name: "Mina Labs".to_string(),
                unread: 4,
                favorite: true,
                muted: false,
            },
            ServerSummary {
                id: "srv-release-watch".to_string(),
                name: "Release Watch".to_string(),
                unread: 0,
                favorite: false,
                muted: false,
            },
        ],
        _ => Vec::new(),
    }
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
