use sqlx::{PgPool, Row};

use crate::models::ContactSummary;

#[derive(Clone)]
pub struct ContactRelationship {
    pub requester_identity_id: String,
    pub target_identity_id: String,
    pub status: String,
}

pub struct ContactListParams<'a> {
    pub identity_id: &'a str,
    pub search: Option<&'a str>,
    pub unread_only: bool,
    pub favorites_only: bool,
    pub limit: usize,
    pub offset: i64,
}

pub async fn list_contact_relationships(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<ContactRelationship>, sqlx::Error> {
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
            Ok(ContactRelationship {
                requester_identity_id: row.try_get::<String, _>("requester_identity_id")?,
                target_identity_id: row.try_get::<String, _>("target_identity_id")?,
                status: row.try_get::<String, _>("status")?,
            })
        })
        .collect()
}

pub async fn list_contact_summaries_for_identity(
    pool: &PgPool,
    params: ContactListParams<'_>,
) -> Result<Vec<ContactSummary>, sqlx::Error> {
    let limit = i64::try_from(params.limit)
        .map_err(|_| sqlx::Error::Protocol("limit too large for storage".into()))?;
    let rows = sqlx::query(
        "
        WITH contact_rows AS (
            SELECT
                CASE
                    WHEN requester_identity_id = $1 THEN target_identity_id
                    ELSE requester_identity_id
                END AS peer_identity_id,
                requester_identity_id,
                target_identity_id,
                status
            FROM friend_requests
            WHERE (requester_identity_id = $1 OR target_identity_id = $1)
              AND status IN ('accepted', 'pending')
        ),
        peers AS (
            SELECT
                peer_identity_id,
                BOOL_OR(status = 'pending' AND target_identity_id = $1) AS inbound_request,
                BOOL_OR(status = 'pending' AND requester_identity_id = $1) AS pending_request
            FROM contact_rows
            GROUP BY peer_identity_id
        )
        SELECT peer_identity_id, inbound_request, pending_request
        FROM peers
        WHERE ($2::TEXT IS NULL OR LOWER(peer_identity_id) LIKE '%' || LOWER($2::TEXT) || '%')
          AND ($3::BOOLEAN = FALSE)
          AND ($4::BOOLEAN = FALSE)
        ORDER BY peer_identity_id ASC
        LIMIT $5 OFFSET $6
        ",
    )
    .bind(params.identity_id)
    .bind(params.search)
    .bind(params.unread_only)
    .bind(params.favorites_only)
    .bind(limit)
    .bind(params.offset)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let id = row.try_get::<String, _>("peer_identity_id")?;
            Ok(ContactSummary {
                id: id.clone(),
                name: id,
                status: "offline".to_string(),
                unread: 0,
                favorite: false,
                inbound_request: row.try_get::<bool, _>("inbound_request")?,
                pending_request: row.try_get::<bool, _>("pending_request")?,
            })
        })
        .collect()
}
