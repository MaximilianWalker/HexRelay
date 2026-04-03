use std::collections::HashMap;

use sqlx::{PgPool, Row};

use crate::models::DiscoveryUserRecord;

#[derive(Clone)]
pub struct DiscoveryRelationshipRow {
    pub peer_identity_id: String,
    pub status: String,
    pub requester_is_self: bool,
    pub created_at: String,
}

pub async fn list_global_discovery_candidates(
    pool: &PgPool,
    identity_id: &str,
    search: Option<&str>,
    limit: usize,
    excluded_identity_ids: &[String],
) -> Result<Vec<DiscoveryUserRecord>, sqlx::Error> {
    let search = search.map(|value| format!("%{value}%"));
    let rows = sqlx::query(
        "
        WITH candidate_ids AS (
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
            UNION
            SELECT identity_id
            FROM identity_keys
            WHERE identity_id <> $1
        ), shared_counts AS (
            SELECT other.identity_id, COUNT(*)::BIGINT AS shared_count
            FROM server_memberships self
            INNER JOIN server_memberships other
                ON other.server_id = self.server_id
            WHERE self.identity_id = $1
              AND other.identity_id <> $1
            GROUP BY other.identity_id
        )
        SELECT candidate_ids.identity_id
        FROM candidate_ids
        LEFT JOIN shared_counts ON shared_counts.identity_id = candidate_ids.identity_id
        WHERE ($2::TEXT IS NULL OR candidate_ids.identity_id ILIKE $2)
          AND NOT (candidate_ids.identity_id = ANY($4))
        ORDER BY COALESCE(shared_counts.shared_count, 0) DESC, candidate_ids.identity_id ASC
        LIMIT $3
        ",
    )
    .bind(identity_id)
    .bind(search)
    .bind(limit as i64)
    .bind(excluded_identity_ids)
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
    search: Option<&str>,
    limit: usize,
    excluded_identity_ids: &[String],
) -> Result<Vec<DiscoveryUserRecord>, sqlx::Error> {
    let search = search.map(|value| format!("%{value}%"));
    let rows = sqlx::query(
        "
        WITH shared_counts AS (
            SELECT other.identity_id, COUNT(*)::BIGINT AS shared_count
            FROM server_memberships self
            INNER JOIN server_memberships other
                ON other.server_id = self.server_id
            WHERE self.identity_id = $1
              AND other.identity_id <> $1
            GROUP BY other.identity_id
        )
        SELECT identity_id
        FROM shared_counts
        WHERE ($2::TEXT IS NULL OR identity_id ILIKE $2)
          AND NOT (identity_id = ANY($4))
        ORDER BY shared_count DESC, identity_id ASC
        LIMIT $3
        ",
    )
    .bind(identity_id)
    .bind(search)
    .bind(limit as i64)
    .bind(excluded_identity_ids)
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
        SELECT requester_identity_id, target_identity_id, status, created_at
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
                created_at: row
                    .try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")?
                    .to_rfc3339(),
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
