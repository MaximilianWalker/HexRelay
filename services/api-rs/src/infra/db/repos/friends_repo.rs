use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::friends::service::{assert_actor_can_transition, ActorRole},
    models::{FriendRequestCreate, FriendRequestListQuery, FriendRequestRecord},
};

pub enum FriendRequestRepoError {
    Sql(sqlx::Error),
    TransitionInvalid,
    ActorNotAuthorized,
}

impl From<sqlx::Error> for FriendRequestRepoError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value)
    }
}

pub async fn create_friend_request(
    pool: &PgPool,
    payload: FriendRequestCreate,
) -> Result<FriendRequestRecord, FriendRequestRepoError> {
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

    Ok(map_friend_request_row(row)?)
}

pub async fn list_friend_requests(
    pool: &PgPool,
    query: &FriendRequestListQuery,
) -> Result<Vec<FriendRequestRecord>, FriendRequestRepoError> {
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
        .map(|row| map_friend_request_row(row).map_err(FriendRequestRepoError::Sql))
        .collect()
}

pub async fn update_friend_request_status(
    pool: &PgPool,
    request_id: &str,
    next_status: &str,
    actor_identity_id: &str,
    actor_role: ActorRole,
) -> Result<Option<FriendRequestRecord>, FriendRequestRepoError> {
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

    let existing = map_friend_request_row(existing_row)?;

    assert_actor_can_transition(&existing, actor_identity_id, actor_role)
        .map_err(|_| FriendRequestRepoError::ActorNotAuthorized)?;

    if existing.status == next_status {
        return Ok(Some(existing));
    }

    if existing.status != "pending" {
        return Err(FriendRequestRepoError::TransitionInvalid);
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
        .map(|row| map_friend_request_row(row).map_err(FriendRequestRepoError::Sql))
        .transpose()
}

fn map_friend_request_row(row: sqlx::postgres::PgRow) -> Result<FriendRequestRecord, sqlx::Error> {
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
