use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use sqlx::PgPool;
use sqlx::Row;
use uuid::Uuid;

use crate::{
    auth::{enforce_csrf_for_cookie_auth, AuthSession},
    errors::{bad_request, conflict, unauthorized, ApiResult},
    models::{
        FriendRequestCreate, FriendRequestListQuery, FriendRequestPage, FriendRequestRecord,
        HealthResponse,
    },
    state::AppState,
    validation::{validate_friend_request_create, validate_friend_request_list_query},
};

pub use crate::invite_handlers::{create_invite, redeem_invite};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

pub async fn create_friend_request(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<FriendRequestCreate>,
) -> ApiResult<(StatusCode, Json<FriendRequestRecord>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
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
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<Json<FriendRequestRecord>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
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
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
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
    headers: HeaderMap,
    axum::extract::Path(request_id): axum::extract::Path<String>,
) -> ApiResult<StatusCode> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
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
