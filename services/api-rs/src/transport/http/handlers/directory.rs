use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use ring::digest::{digest, SHA256};

use crate::{
    domain::block_mute::validation::validate_block_request,
    infra::{
        db::repos::{contacts_repo, invites_repo, server_channels_repo, servers_repo},
        presence::redis_presence,
    },
    models::{
        BlockUserRequest, ContactBlockRemoveResponse, ContactListQuery, ContactListResponse,
        ContactSummary, HubPreferenceUpdateRequest, ServerCreateRequest, ServerCreateResponse,
        ServerDetailResponse, ServerJoinRequest, ServerJoinResponse, ServerLeaveRequest,
        ServerLeaveResponse, ServerListQuery, ServerListResponse,
    },
    shared::errors::{bad_request, forbidden, internal_error, ApiResult},
    state::AppState,
    transport::http::{
        handlers::block_mute::remember_block,
        middleware::{
            auth::{enforce_csrf_for_cookie_auth, AuthSession},
            authorization::AuthorizedServerMembership,
        },
    },
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

    let mut items =
        servers_repo::list_servers_for_identity(pool, &auth.identity_id, &state.server_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to list servers"))?;

    if query.pinned_only.unwrap_or(false) {
        items.retain(|item| item.pinned);
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

pub async fn create_server(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<ServerCreateRequest>,
) -> ApiResult<(StatusCode, Json<ServerCreateResponse>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let name = normalize_server_name(&payload.name)?;
    let description = payload
        .description
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    if description.len() > 280 {
        return Err(bad_request(
            "server_invalid",
            "server description exceeds maximum length",
        ));
    }

    let bootstrap_credential = payload
        .bootstrap_credential
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("srv-bootstrap-{}", uuid::Uuid::new_v4().simple()));
    if bootstrap_credential.len() > 256 {
        return Err(bad_request(
            "server_invalid",
            "bootstrap credential exceeds maximum length",
        ));
    }

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server creation requires configured database pool",
        )
    })?;

    let credential_id = format!("sbc-{}", uuid::Uuid::new_v4().simple());
    let credential_hash = hash_secret(&bootstrap_credential);

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to start server creation"))?;

    servers_repo::lock_server_bootstrap_state(&mut *tx)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to lock server bootstrap"))?;
    let is_configured_owner = state.server_owner_identity_ids.contains(&auth.identity_id);
    let is_db_owner = servers_repo::server_administration_for_identity(&mut *tx, &auth.identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load server owner"))?
        .is_some_and(|(is_owner, _)| is_owner);
    let is_initial_bootstrap = !servers_repo::server_has_memberships(&mut *tx)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load server membership"))?;
    if !(is_configured_owner || is_db_owner || is_initial_bootstrap) {
        return Err(forbidden(
            "server_owner_required",
            "server creation requires server owner authorization",
        ));
    }

    servers_repo::insert_server(
        &mut *tx,
        servers_repo::ServerInsertParams {
            name: &name,
            description: &description,
        },
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to save server"))?;

    servers_repo::insert_server_membership(
        &mut *tx,
        servers_repo::ServerMembershipInsertParams {
            identity_id: &auth.identity_id,
            pinned: true,
            muted: false,
            unread_count: 0,
        },
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to save server membership"))?;

    servers_repo::insert_server_administrator(
        &mut *tx,
        servers_repo::ServerAdministratorInsertParams {
            identity_id: &auth.identity_id,
            is_owner: true,
            is_admin: true,
        },
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to save server owner"))?;

    servers_repo::insert_server_bootstrap_credential(
        &mut *tx,
        servers_repo::ServerBootstrapCredentialInsertParams {
            credential_id: &credential_id,
            credential_secret_hash: &credential_hash,
            created_by_identity_id: &auth.identity_id,
        },
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to save server bootstrap credential",
        )
    })?;

    server_channels_repo::insert_server_channel(
        &mut *tx,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: "chn-general",
            name: "general",
            kind: "text",
        },
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to save default text channel"))?;
    server_channels_repo::insert_server_channel(
        &mut *tx,
        server_channels_repo::ServerChannelInsertParams {
            channel_id: "chn-voice-lounge",
            name: "Voice Lounge",
            kind: "voice",
        },
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to save default voice channel",
        )
    })?;

    tx.commit()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to commit server creation"))?;

    let item = servers_repo::get_server_for_identity(pool, &auth.identity_id, &state.server_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load created server"))?
        .ok_or_else(|| {
            internal_error("storage_unavailable", "created server membership missing")
        })?;

    Ok((
        StatusCode::CREATED,
        Json(ServerCreateResponse {
            item,
            owner_identity_id: auth.identity_id,
            bootstrap_credential,
        }),
    ))
}

pub async fn join_server(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<ServerJoinRequest>,
) -> ApiResult<Json<ServerJoinResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let (server_id, invite_token) = normalize_join_payload(&state, &payload)?;
    if server_id != state.server_id {
        return Err(forbidden(
            "server_access_denied",
            "this runtime can only join its own server authority",
        ));
    }

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server join requires configured database pool",
        )
    })?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to start server join"))?;
    let token_hash = hash_secret(&invite_token);
    let invite = invites_repo::load_invite_for_update(&mut *tx, &token_hash)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load invite"))?
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

    if invite.server_id != server_id {
        return Err(bad_request("server_mismatch", "invite server id mismatch"));
    }

    let already_member = servers_repo::identity_has_server_membership(&mut *tx, &auth.identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load server membership"))?;
    let joined = if already_member {
        false
    } else {
        if let Some(expires_at) = invite.expires_at {
            if chrono::Utc::now() > expires_at {
                return Err(bad_request("invite_expired", "invite token is expired"));
            }
        }
        if let Some(max_uses) = invite.max_uses {
            if invite.uses >= max_uses {
                return Err(bad_request("invite_exhausted", "invite token is exhausted"));
            }
        }

        let inserted = servers_repo::insert_server_membership_if_absent(
            &mut *tx,
            servers_repo::ServerMembershipInsertParams {
                identity_id: &auth.identity_id,
                pinned: false,
                muted: false,
                unread_count: 0,
            },
        )
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to save server membership"))?;
        if inserted {
            invites_repo::increment_invite_use(&mut *tx, &token_hash)
                .await
                .map_err(|_| internal_error("storage_unavailable", "failed to update invite"))?;
        }

        inserted
    };

    tx.commit()
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to commit server join"))?;

    let item = servers_repo::get_server_for_identity(pool, &auth.identity_id, &state.server_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to load joined server"))?
        .ok_or_else(|| internal_error("storage_unavailable", "joined server membership missing"))?;

    Ok(Json(ServerJoinResponse { item, joined }))
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

pub async fn update_server_preferences(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<HubPreferenceUpdateRequest>,
) -> ApiResult<Json<ServerDetailResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server preference update requires configured database pool",
        )
    })?;

    let item = servers_repo::update_server_membership_preferences(
        pool,
        servers_repo::ServerPreferenceUpdateParams {
            identity_id: &membership.identity_id,
            pinned: payload.pinned,
            muted: payload.muted,
        },
        &membership.server_id,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to update server preferences"))?
    .ok_or_else(|| forbidden("server_access_denied", "server membership required"))?;

    Ok(Json(ServerDetailResponse { item }))
}

pub async fn leave_server(
    State(state): State<AppState>,
    membership: AuthorizedServerMembership,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<ServerLeaveRequest>,
) -> ApiResult<Json<ServerLeaveResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "server leave requires configured database pool",
        )
    })?;

    let left = servers_repo::delete_server_membership(pool, &membership.identity_id)
        .await
        .map_err(|_| internal_error("storage_unavailable", "failed to leave server"))?;

    Ok(Json(ServerLeaveResponse {
        left,
        deleted_local_data: payload.delete_local_data,
    }))
}

pub async fn list_contacts(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<ContactListQuery>,
) -> ApiResult<Json<ContactListResponse>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let identity_id = auth.identity_id;

        let mut items = contacts_repo::list_contacts_for_identity(pool, &identity_id)
            .await
            .map_err(|_| {
                internal_error(
                    "storage_unavailable",
                    "failed to list contact relationships",
                )
            })?;

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
            pinned: true,
            muted: false,
            inbound_request: false,
            pending_request: false,
        },
        ContactSummary {
            id: "usr-alex-r".to_string(),
            name: "Alex R".to_string(),
            status: "offline".to_string(),
            unread: 0,
            pinned: false,
            muted: false,
            inbound_request: false,
            pending_request: true,
        },
        ContactSummary {
            id: "usr-mina-s".to_string(),
            name: "Mina S".to_string(),
            status: "online".to_string(),
            unread: 3,
            pinned: true,
            muted: false,
            inbound_request: false,
            pending_request: false,
        },
        ContactSummary {
            id: "usr-jules-p".to_string(),
            name: "Jules P".to_string(),
            status: "away".to_string(),
            unread: 0,
            pinned: false,
            muted: true,
            inbound_request: true,
            pending_request: false,
        },
    ];

    apply_contact_filters(&mut items, &query);
    Ok(Json(ContactListResponse { items }))
}

pub async fn update_contact_preferences(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Path(contact_identity_id): Path<String>,
    Json(payload): Json<HubPreferenceUpdateRequest>,
) -> ApiResult<Json<ContactSummary>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let contact_identity_id = normalize_identity_path(&contact_identity_id)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "contact preference update requires configured database pool",
        )
    })?;

    let item = contacts_repo::upsert_contact_preferences(
        pool,
        contacts_repo::ContactPreferenceUpdateParams {
            owner_identity_id: &auth.identity_id,
            contact_identity_id: &contact_identity_id,
            pinned: payload.pinned,
            muted: payload.muted,
        },
    )
    .await
    .map_err(|_| {
        internal_error(
            "storage_unavailable",
            "failed to update contact preferences",
        )
    })?
    .ok_or_else(|| bad_request("contact_not_found", "contact relationship was not found"))?;

    Ok(Json(item))
}

pub async fn block_remove_contact(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Path(contact_identity_id): Path<String>,
) -> ApiResult<Json<ContactBlockRemoveResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let contact_identity_id = normalize_identity_path(&contact_identity_id)?;
    validate_block_request(
        &BlockUserRequest {
            target_identity_id: contact_identity_id.clone(),
        },
        &auth.identity_id,
    )?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "contact removal requires configured database pool",
        )
    })?;

    let relationship_removed =
        contacts_repo::block_and_remove_contact(pool, &auth.identity_id, &contact_identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to block and remove contact")
            })?;
    remember_block(
        &state,
        &auth.identity_id,
        &contact_identity_id,
        chrono::Utc::now(),
    );

    Ok(Json(ContactBlockRemoveResponse {
        blocked_identity_id: contact_identity_id,
        relationship_removed,
    }))
}

fn normalize_server_name(value: &str) -> ApiResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(bad_request(
            "server_invalid",
            "server name must not be empty",
        ));
    }
    if trimmed.len() > 80 {
        return Err(bad_request(
            "server_invalid",
            "server name exceeds maximum length",
        ));
    }

    Ok(trimmed.to_string())
}

fn normalize_identity_path(value: &str) -> ApiResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(bad_request(
            "identity_invalid",
            "identity id must not be empty",
        ));
    }

    Ok(trimmed.to_string())
}

fn normalize_join_payload(
    state: &AppState,
    payload: &ServerJoinRequest,
) -> ApiResult<(String, String)> {
    let parsed = payload
        .invite_link
        .as_deref()
        .map(parse_invite_link)
        .transpose()?;
    let server_id = payload
        .server_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| parsed.as_ref().and_then(|value| value.0.clone()))
        .unwrap_or_else(|| state.server_id.clone());
    let invite_token = payload
        .invite_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| parsed.and_then(|value| value.1))
        .ok_or_else(|| bad_request("invite_invalid", "invite token is required"))?;

    if payload
        .endpoint
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(bad_request("server_invalid", "endpoint must not be empty"));
    }

    Ok((server_id, invite_token))
}

fn parse_invite_link(value: &str) -> ApiResult<(Option<String>, Option<String>)> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok((None, None));
    }

    if !trimmed.contains('?') && !trimmed.contains("token=") {
        return Ok((None, Some(trimmed.to_string())));
    }

    let query = trimmed
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or(trimmed);
    let mut server_id = None;
    let mut token = None;
    for pair in query.split('&') {
        let Some((name, value)) = pair.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match name {
            "server_id" | "serverId" => server_id = Some(value.to_string()),
            "token" | "invite_token" | "inviteToken" => token = Some(value.to_string()),
            _ => {}
        }
    }

    if token.is_none() {
        return Err(bad_request(
            "invite_invalid",
            "invite link must include an invite token",
        ));
    }

    Ok((server_id, token))
}

fn hash_secret(secret: &str) -> String {
    hex::encode(digest(&SHA256, secret.as_bytes()).as_ref())
}

fn apply_contact_filters(items: &mut Vec<ContactSummary>, query: &ContactListQuery) {
    if query.unread_only.unwrap_or(false) {
        items.retain(|item| item.unread > 0);
    }
    if query.pinned_only.unwrap_or(false) {
        items.retain(|item| item.pinned);
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
}
