use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use ring::signature::{UnparsedPublicKey, ED25519};
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    errors::{bad_request, conflict, unauthorized, ApiResult},
    models::{
        AuthChallengeRecord, AuthChallengeRequest, AuthChallengeResponse, AuthVerifyRequest,
        AuthVerifyResponse, ContactListQuery, ContactListResponse, ContactSummary,
        FriendRequestCreate, FriendRequestListQuery, FriendRequestPage, FriendRequestRecord,
        HealthResponse, IdentityKeyRegistrationRequest, InviteCreateRequest, InviteCreateResponse,
        InviteRecord, InviteRedeemRequest, InviteRedeemResponse, RegisteredIdentityKey,
        ServerListQuery, ServerListResponse, ServerSummary, SessionRecord, SessionRevokeRequest,
        SessionValidateResponse,
    },
    session_token::issue_session_token,
    state::AppState,
    validation::{
        decode_32_bytes, decode_64_bytes, validate_auth_challenge_request,
        validate_auth_verify_request, validate_friend_request_create,
        validate_friend_request_list_query, validate_identity_registration,
        validate_invite_create_request, validate_invite_redeem_request,
        validate_session_revoke_request,
    },
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

pub async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> ApiResult<StatusCode> {
    validate_identity_registration(&payload)?;

    let mut guard = state
        .identity_keys
        .write()
        .expect("acquire identity key write lock");

    if guard.contains_key(&payload.identity_id) {
        return Err(conflict(
            "identity_exists",
            "identity_id already has a registered key",
        ));
    }

    guard.insert(
        payload.identity_id,
        RegisteredIdentityKey {
            public_key: payload.public_key,
            algorithm: payload.algorithm,
        },
    );

    Ok(StatusCode::CREATED)
}

pub async fn validate_session(auth: AuthSession) -> ApiResult<Json<SessionValidateResponse>> {
    Ok(Json(SessionValidateResponse {
        session_id: auth.session_id,
        identity_id: auth.identity_id,
        expires_at: auth.expires_at,
    }))
}

pub async fn issue_auth_challenge(
    State(state): State<AppState>,
    Json(payload): Json<AuthChallengeRequest>,
) -> ApiResult<Json<AuthChallengeResponse>> {
    validate_auth_challenge_request(&payload)?;

    let identity_exists = state
        .identity_keys
        .read()
        .expect("acquire identity key read lock")
        .contains_key(&payload.identity_id);

    if !identity_exists {
        return Err(bad_request(
            "identity_invalid",
            "identity_id is not registered",
        ));
    }

    let challenge_id = Uuid::new_v4().to_string();
    let nonce = random_hex(32);
    let expires_at = (Utc::now() + Duration::minutes(5)).to_rfc3339();

    state
        .auth_challenges
        .write()
        .expect("acquire challenge write lock")
        .insert(
            challenge_id.clone(),
            AuthChallengeRecord {
                identity_id: payload.identity_id,
                nonce: nonce.clone(),
                expires_at: Utc::now() + Duration::minutes(5),
            },
        );

    Ok(Json(AuthChallengeResponse {
        challenge_id,
        nonce,
        expires_at,
    }))
}

pub async fn verify_auth_challenge(
    State(state): State<AppState>,
    Json(payload): Json<AuthVerifyRequest>,
) -> ApiResult<Json<AuthVerifyResponse>> {
    validate_auth_verify_request(&payload)?;

    let challenge_record = state
        .auth_challenges
        .write()
        .expect("acquire challenge write lock")
        .remove(&payload.challenge_id)
        .ok_or_else(|| unauthorized("nonce_invalid", "challenge_id is invalid"))?;

    if challenge_record.identity_id != payload.identity_id {
        return Err(unauthorized(
            "nonce_invalid",
            "challenge does not match identity",
        ));
    }

    if Utc::now() > challenge_record.expires_at {
        return Err(unauthorized("nonce_invalid", "challenge has expired"));
    }

    let key_record = state
        .identity_keys
        .read()
        .expect("acquire identity key read lock")
        .get(&payload.identity_id)
        .cloned()
        .ok_or_else(|| unauthorized("identity_invalid", "identity_id is not registered"))?;

    if key_record.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "registered algorithm must be ed25519",
        ));
    }

    let public_key = decode_32_bytes(&key_record.public_key)
        .ok_or_else(|| bad_request("public_key_invalid", "registered public key is invalid"))?;
    let signature_bytes = decode_64_bytes(&payload.signature).ok_or_else(|| {
        bad_request(
            "signature_invalid",
            "signature must be 64-byte hex or base64",
        )
    })?;

    verify_signature(
        &public_key,
        challenge_record.nonce.as_bytes(),
        &signature_bytes,
    )
    .map_err(|_| unauthorized("signature_invalid", "signature verification failed"))?;

    let identity_id = payload.identity_id.clone();

    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(12);

    state
        .sessions
        .write()
        .expect("acquire session write lock")
        .insert(
            session_id.clone(),
            SessionRecord {
                identity_id: identity_id.clone(),
                expires_at,
            },
        );

    if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query(
            "
            INSERT INTO sessions (session_id, identity_id, expires_at)
            VALUES ($1, $2, $3)
            ",
        )
        .bind(&session_id)
        .bind(&identity_id)
        .bind(expires_at)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to persist session")
        })?;
    }

    let access_token = issue_session_token(
        &session_id,
        &identity_id,
        expires_at.timestamp(),
        &state.session_signing_key,
    );

    Ok(Json(AuthVerifyResponse {
        session_id,
        access_token,
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn revoke_session(
    auth: AuthSession,
    State(state): State<AppState>,
    Json(payload): Json<SessionRevokeRequest>,
) -> ApiResult<StatusCode> {
    validate_session_revoke_request(&payload)?;

    if payload.session_id != auth.session_id {
        return Err(unauthorized(
            "session_invalid",
            "session_id does not match authenticated session",
        ));
    }

    let removed = state
        .sessions
        .write()
        .expect("acquire session write lock")
        .remove(&payload.session_id);

    if removed.is_none() {
        if let Some(pool) = state.db_pool.as_ref() {
            let updated = sqlx::query(
                "
                UPDATE sessions
                SET revoked_at = NOW()
                WHERE session_id = $1 AND revoked_at IS NULL
                ",
            )
            .bind(&payload.session_id)
            .execute(pool)
            .await
            .map_err(|_| {
                crate::errors::internal_error("storage_unavailable", "failed to revoke session")
            })?;

            if updated.rows_affected() == 0 {
                return Err(bad_request("session_invalid", "session_id is invalid"));
            }

            return Ok(StatusCode::NO_CONTENT);
        }

        return Err(bad_request("session_invalid", "session_id is invalid"));
    }

    if let Some(pool) = state.db_pool.as_ref() {
        sqlx::query(
            "
            UPDATE sessions
            SET revoked_at = NOW()
            WHERE session_id = $1 AND revoked_at IS NULL
            ",
        )
        .bind(&payload.session_id)
        .execute(pool)
        .await
        .map_err(|_| {
            crate::errors::internal_error("storage_unavailable", "failed to revoke session")
        })?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_invite(
    State(state): State<AppState>,
    Json(payload): Json<InviteCreateRequest>,
) -> ApiResult<(StatusCode, Json<InviteCreateResponse>)> {
    validate_invite_create_request(&payload)?;

    let expires_at = if let Some(raw_expires_at) = payload.expires_at.as_ref() {
        let parsed = chrono::DateTime::parse_from_rfc3339(raw_expires_at)
            .map_err(|_| bad_request("invite_invalid", "expires_at must be RFC3339 date-time"))?
            .with_timezone(&Utc);

        if parsed <= Utc::now() {
            return Err(bad_request(
                "invite_invalid",
                "expires_at must be in the future",
            ));
        }

        Some(parsed)
    } else {
        None
    };

    let max_uses = if payload.mode == "one_time" {
        match payload.max_uses {
            None => Some(1),
            Some(1) => Some(1),
            Some(_) => {
                return Err(bad_request(
                    "invite_invalid",
                    "one_time invite max_uses must be 1 if provided",
                ));
            }
        }
    } else {
        payload.max_uses
    };

    let token = Uuid::new_v4().to_string();
    state
        .invites
        .write()
        .expect("acquire invite write lock")
        .insert(
            token.clone(),
            InviteRecord {
                mode: payload.mode.clone(),
                node_fingerprint: state.node_fingerprint.clone(),
                expires_at,
                max_uses,
                uses: 0,
            },
        );

    Ok((
        StatusCode::CREATED,
        Json(InviteCreateResponse {
            token,
            mode: payload.mode,
            expires_at: expires_at.map(|value| value.to_rfc3339()),
            max_uses,
        }),
    ))
}

pub async fn redeem_invite(
    State(state): State<AppState>,
    Json(payload): Json<InviteRedeemRequest>,
) -> ApiResult<Json<InviteRedeemResponse>> {
    validate_invite_redeem_request(&payload)?;

    let mut guard = state.invites.write().expect("acquire invite write lock");
    let invite = guard
        .get_mut(&payload.token)
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

    if invite.node_fingerprint != payload.node_fingerprint {
        return Err(bad_request(
            "fingerprint_mismatch",
            "invite node fingerprint mismatch",
        ));
    }

    if let Some(expires_at) = invite.expires_at {
        if Utc::now() > expires_at {
            return Err(bad_request("invite_expired", "invite token is expired"));
        }
    }

    if let Some(max_uses) = invite.max_uses {
        if invite.uses >= max_uses {
            return Err(bad_request("invite_exhausted", "invite token is exhausted"));
        }
    }

    invite.uses += 1;

    Ok(Json(InviteRedeemResponse { accepted: true }))
}

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

pub async fn create_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    Json(payload): Json<FriendRequestCreate>,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    validate_friend_request_create(&payload)?;
    let actor_identity = auth.identity_id;

    if payload.requester_identity_id != actor_identity {
        return Err(unauthorized(
            "identity_invalid",
            "requester_identity_id must match authenticated session",
        ));
    }

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(crate::errors::internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return create_friend_request_in_memory(state, payload);
        }
    };

    let record = create_friend_request_db(pool, payload)
        .await
        .map_err(map_friend_request_db_error)?;
    Ok((StatusCode::CREATED, Json(record)))
}

#[cfg(test)]
fn create_friend_request_in_memory(
    state: AppState,
    payload: FriendRequestCreate,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let existing = guard.values().find(|item| {
        item.requester_identity_id == payload.requester_identity_id
            && item.target_identity_id == payload.target_identity_id
            && item.status == "pending"
    });

    if existing.is_some() {
        return Err(bad_request(
            "identity_invalid",
            "pending friend request already exists",
        ));
    }

    let record = FriendRequestRecord {
        request_id: Uuid::new_v4().to_string(),
        requester_identity_id: payload.requester_identity_id,
        target_identity_id: payload.target_identity_id,
        status: "pending".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    guard.insert(record.request_id.clone(), record.clone());

    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn list_friend_requests(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<FriendRequestListQuery>,
) -> ApiResult<Json<FriendRequestPage>> {
    validate_friend_request_list_query(&query)?;
    let actor_identity = auth.identity_id;

    if query.identity_id != actor_identity {
        return Err(unauthorized(
            "identity_invalid",
            "identity_id must match authenticated session",
        ));
    }

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(crate::errors::internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return list_friend_requests_in_memory(state, query);
        }
    };

    let items = list_friend_requests_db(pool, &query)
        .await
        .map_err(map_friend_request_db_error)?;
    Ok(Json(FriendRequestPage { items }))
}

#[cfg(test)]
fn list_friend_requests_in_memory(
    state: AppState,
    query: FriendRequestListQuery,
) -> ApiResult<Json<FriendRequestPage>> {
    let guard = state
        .friend_requests
        .read()
        .expect("acquire friend request read lock");

    let mut items: Vec<FriendRequestRecord> = guard
        .values()
        .filter(|item| match query.direction.as_deref() {
            Some("inbound") => item.target_identity_id == query.identity_id,
            Some("outbound") => item.requester_identity_id == query.identity_id,
            _ => {
                item.requester_identity_id == query.identity_id
                    || item.target_identity_id == query.identity_id
            }
        })
        .cloned()
        .collect();

    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(Json(FriendRequestPage { items }))
}

pub async fn accept_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<Json<FriendRequestRecord>> {
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(crate::errors::internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return accept_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = update_friend_request_status_db(
        pool,
        &request_id,
        "accepted",
        &actor_identity,
        ActorRole::Target,
    )
    .await
    .map_err(map_friend_request_db_error)?
    .ok_or_else(|| {
        bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        )
    })?;
    Ok(Json(updated))
}

#[cfg(test)]
fn accept_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<Json<FriendRequestRecord>> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition_in_memory(
        request,
        "accepted",
        &actor_identity,
        ActorRole::Target,
    )?;

    Ok(Json(request.clone()))
}

pub async fn decline_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(crate::errors::internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return decline_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = update_friend_request_status_db(
        pool,
        &request_id,
        "declined",
        &actor_identity,
        ActorRole::Target,
    )
    .await
    .map_err(map_friend_request_db_error)?;

    if updated.is_none() {
        return Err(bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
fn decline_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<StatusCode> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition_in_memory(
        request,
        "declined",
        &actor_identity,
        ActorRole::Target,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn cancel_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    let actor_identity = auth.identity_id;

    let Some(pool) = state.db_pool.as_ref() else {
        #[cfg(not(test))]
        {
            return Err(crate::errors::internal_error(
                "storage_unavailable",
                "friend request storage requires configured database pool",
            ));
        }

        #[cfg(test)]
        {
            return cancel_friend_request_in_memory(state, request_id, actor_identity);
        }
    };

    let updated = update_friend_request_status_db(
        pool,
        &request_id,
        "cancelled",
        &actor_identity,
        ActorRole::Requester,
    )
    .await
    .map_err(map_friend_request_db_error)?;

    if updated.is_none() {
        return Err(bad_request(
            "identity_invalid",
            "friend request not found or not actionable by current session",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
fn cancel_friend_request_in_memory(
    state: AppState,
    request_id: String,
    actor_identity: String,
) -> ApiResult<StatusCode> {
    let mut guard = state
        .friend_requests
        .write()
        .expect("acquire friend request write lock");

    let request = guard
        .get_mut(&request_id)
        .ok_or_else(|| bad_request("identity_invalid", "friend request not found"))?;

    apply_friend_request_transition_in_memory(
        request,
        "cancelled",
        &actor_identity,
        ActorRole::Requester,
    )?;

    Ok(StatusCode::NO_CONTENT)
}

fn random_hex(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<(), ()> {
    let key = UnparsedPublicKey::new(&ED25519, public_key);
    key.verify(message, signature).map_err(|_| ())
}

async fn create_friend_request_db(
    pool: &PgPool,
    payload: FriendRequestCreate,
) -> Result<FriendRequestRecord, sqlx::Error> {
    let request_id = Uuid::new_v4().to_string();

    sqlx::query(
        "
        INSERT INTO friend_requests (request_id, requester_identity_id, target_identity_id, status)
        VALUES ($1, $2, $3, 'pending')
        ",
    )
    .bind(&request_id)
    .bind(&payload.requester_identity_id)
    .bind(&payload.target_identity_id)
    .execute(pool)
    .await?;

    let row = sqlx::query(
        "
        SELECT request_id, requester_identity_id, target_identity_id, status, created_at
        FROM friend_requests
        WHERE request_id = $1
        ",
    )
    .bind(&request_id)
    .fetch_one(pool)
    .await?;

    Ok(FriendRequestRecord {
        request_id: row.try_get::<String, _>("request_id")?,
        requester_identity_id: row.try_get::<String, _>("requester_identity_id")?,
        target_identity_id: row.try_get::<String, _>("target_identity_id")?,
        status: row.try_get::<String, _>("status")?,
        created_at: row
            .try_get::<chrono::DateTime<Utc>, _>("created_at")?
            .to_rfc3339(),
    })
}

async fn list_friend_requests_db(
    pool: &PgPool,
    query: &FriendRequestListQuery,
) -> Result<Vec<FriendRequestRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT request_id, requester_identity_id, target_identity_id, status, created_at
        FROM friend_requests
        WHERE (
            $2::TEXT = 'inbound' AND target_identity_id = $1
        ) OR (
            $2::TEXT = 'outbound' AND requester_identity_id = $1
        ) OR (
            $2::TEXT IS NULL AND (requester_identity_id = $1 OR target_identity_id = $1)
        )
        ORDER BY created_at DESC
        ",
    )
    .bind(&query.identity_id)
    .bind(query.direction.as_deref())
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(FriendRequestRecord {
                request_id: row.try_get::<String, _>("request_id")?,
                requester_identity_id: row.try_get::<String, _>("requester_identity_id")?,
                target_identity_id: row.try_get::<String, _>("target_identity_id")?,
                status: row.try_get::<String, _>("status")?,
                created_at: row
                    .try_get::<chrono::DateTime<Utc>, _>("created_at")?
                    .to_rfc3339(),
            })
        })
        .collect()
}

async fn update_friend_request_status_db(
    pool: &PgPool,
    request_id: &str,
    next_status: &str,
    actor_identity_id: &str,
    actor_role: ActorRole,
) -> Result<Option<FriendRequestRecord>, sqlx::Error> {
    let maybe_existing = sqlx::query(
        "
        SELECT request_id, requester_identity_id, target_identity_id, status, created_at
        FROM friend_requests
        WHERE request_id = $1
        ",
    )
    .bind(request_id)
    .fetch_optional(pool)
    .await?;

    let Some(existing_row) = maybe_existing else {
        return Ok(None);
    };

    let existing = FriendRequestRecord {
        request_id: existing_row.try_get::<String, _>("request_id")?,
        requester_identity_id: existing_row.try_get::<String, _>("requester_identity_id")?,
        target_identity_id: existing_row.try_get::<String, _>("target_identity_id")?,
        status: existing_row.try_get::<String, _>("status")?,
        created_at: existing_row
            .try_get::<chrono::DateTime<Utc>, _>("created_at")?
            .to_rfc3339(),
    };

    assert_actor_can_transition(&existing, actor_identity_id, actor_role)
        .map_err(|_| sqlx::Error::Protocol("actor_not_authorized".to_string()))?;

    if existing.status == next_status {
        return Ok(Some(existing));
    }

    if existing.status != "pending" {
        return Err(sqlx::Error::Protocol("transition_invalid".to_string()));
    }

    let maybe_row = sqlx::query(
        "
        UPDATE friend_requests
        SET status = $2
        WHERE request_id = $1 AND status = 'pending'
        RETURNING request_id, requester_identity_id, target_identity_id, status, created_at
        ",
    )
    .bind(request_id)
    .bind(next_status)
    .fetch_optional(pool)
    .await?;

    maybe_row
        .map(|row| {
            Ok(FriendRequestRecord {
                request_id: row.try_get::<String, _>("request_id")?,
                requester_identity_id: row.try_get::<String, _>("requester_identity_id")?,
                target_identity_id: row.try_get::<String, _>("target_identity_id")?,
                status: row.try_get::<String, _>("status")?,
                created_at: row
                    .try_get::<chrono::DateTime<Utc>, _>("created_at")?
                    .to_rfc3339(),
            })
        })
        .transpose()
}

fn map_friend_request_db_error(error: sqlx::Error) -> (StatusCode, Json<crate::models::ApiError>) {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.code().as_deref() == Some("23505") {
            return bad_request("identity_invalid", "pending friend request already exists");
        }
    }

    if let sqlx::Error::Protocol(message) = &error {
        if message == "transition_invalid" {
            return conflict(
                "transition_invalid",
                "friend request transition is not allowed from current state",
            );
        }

        if message == "actor_not_authorized" {
            return unauthorized(
                "identity_invalid",
                "friend request cannot be mutated by this session",
            );
        }
    }

    bad_request("identity_invalid", "friend request storage failure")
}

#[derive(Clone, Copy)]
enum ActorRole {
    Requester,
    Target,
}

#[cfg(test)]
fn apply_friend_request_transition_in_memory(
    request: &mut FriendRequestRecord,
    next_status: &str,
    actor_identity: &str,
    actor_role: ActorRole,
) -> ApiResult<()> {
    assert_actor_can_transition(request, actor_identity, actor_role)?;

    if request.status == next_status {
        return Ok(());
    }

    if request.status != "pending" {
        return Err(conflict(
            "transition_invalid",
            "friend request transition is not allowed from current state",
        ));
    }

    request.status = next_status.to_string();
    Ok(())
}

fn assert_actor_can_transition(
    request: &FriendRequestRecord,
    actor_identity: &str,
    actor_role: ActorRole,
) -> ApiResult<()> {
    let allowed = match actor_role {
        ActorRole::Requester => request.requester_identity_id == actor_identity,
        ActorRole::Target => request.target_identity_id == actor_identity,
    };

    if !allowed {
        return Err(unauthorized(
            "identity_invalid",
            "friend request cannot be mutated by this session",
        ));
    }

    Ok(())
}
