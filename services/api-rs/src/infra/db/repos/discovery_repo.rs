use std::collections::{HashMap, HashSet};

use sqlx::{PgPool, Row};

use crate::models::DiscoveryUserRecord;

#[derive(Clone)]
pub struct DiscoveryRelationshipRow {
    pub peer_identity_id: String,
    pub status: String,
    pub requester_is_self: bool,
}

pub async fn list_global_discovery_candidates(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DiscoveryUserRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT DISTINCT identity_id
        FROM (
            SELECT requester_identity_id AS identity_id
            FROM friend_requests
            WHERE requester_identity_id <> $1
            UNION
            SELECT target_identity_id AS identity_id
            FROM friend_requests
            WHERE target_identity_id <> $1
            UNION
            SELECT identity_id
            FROM server_memberships
            WHERE identity_id <> $1
        ) candidates
        ORDER BY identity_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let identity_id = row
                .try_get::<String, _>("identity_id")
                .expect("identity_id column available");
            DiscoveryUserRecord {
                display_name: identity_id.clone(),
                identity_id,
                avatar_url: None,
            }
        })
        .collect())
}

pub async fn list_shared_server_discovery_candidates(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DiscoveryUserRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT DISTINCT other.identity_id
        FROM server_memberships self
        INNER JOIN server_memberships other
            ON other.server_id = self.server_id
        WHERE self.identity_id = $1
          AND other.identity_id <> $1
        ORDER BY other.identity_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let identity_id = row
                .try_get::<String, _>("identity_id")
                .expect("identity_id column available");
            DiscoveryUserRecord {
                display_name: identity_id.clone(),
                identity_id,
                avatar_url: None,
            }
        })
        .collect())
}

pub async fn list_relationship_rows(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DiscoveryRelationshipRow>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT requester_identity_id, target_identity_id, status
        FROM friend_requests
        WHERE requester_identity_id = $1 OR target_identity_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let requester_identity_id = row.try_get::<String, _>("requester_identity_id")?;
            let target_identity_id = row.try_get::<String, _>("target_identity_id")?;
            let peer_identity_id = if requester_identity_id == identity_id {
                target_identity_id
            } else {
                requester_identity_id.clone()
            };

            Ok(DiscoveryRelationshipRow {
                peer_identity_id,
                status: row.try_get::<String, _>("status")?,
                requester_is_self: requester_identity_id == identity_id,
            })
        })
        .collect()
}

pub async fn shared_server_counts(
    pool: &PgPool,
    identity_id: &str,
) -> Result<HashMap<String, u32>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT other.identity_id, COUNT(*)::BIGINT AS shared_count
        FROM server_memberships self
        INNER JOIN server_memberships other
            ON other.server_id = self.server_id
        WHERE self.identity_id = $1
          AND other.identity_id <> $1
        GROUP BY other.identity_id
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let shared_count = row.try_get::<i64, _>("shared_count")?;
            Ok((
                row.try_get::<String, _>("identity_id")?,
                u32::try_from(shared_count).map_err(|_| {
                    sqlx::Error::Protocol("shared_count must be non-negative".into())
                })?,
            ))
        })
        .collect()
}

pub async fn blocked_peers(
    pool: &PgPool,
    identity_id: &str,
) -> Result<HashSet<String>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT blocked_identity_id AS peer_identity_id
        FROM blocked_users
        WHERE blocker_identity_id = $1
        UNION
        SELECT blocker_identity_id AS peer_identity_id
        FROM blocked_users
        WHERE blocked_identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("peer_identity_id").ok())
        .collect())
}
