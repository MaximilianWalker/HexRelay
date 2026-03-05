use axum::{extract::Query, Json};

use crate::{
    auth::AuthSession,
    models::{
        ContactListQuery, ContactListResponse, ContactSummary, ServerListQuery, ServerListResponse,
        ServerSummary,
    },
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
    _auth: AuthSession,
    Query(query): Query<ContactListQuery>,
) -> Json<ContactListResponse> {
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

    Json(ContactListResponse { items })
}
