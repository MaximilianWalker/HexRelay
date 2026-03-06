use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct ContactRelationship {
    pub requester_identity_id: String,
    pub target_identity_id: String,
    pub status: String,
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
