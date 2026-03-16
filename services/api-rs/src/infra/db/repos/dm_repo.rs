use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn consume_dm_pairing_nonce(
    pool: &PgPool,
    nonce: &str,
    expires_at: DateTime<Utc>,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "
        DELETE FROM dm_pairing_nonces
        WHERE expires_at < NOW()
        ",
    )
    .execute(&mut *tx)
    .await?;

    let inserted = sqlx::query(
        "
        INSERT INTO dm_pairing_nonces (nonce, expires_at)
        VALUES ($1, $2)
        ON CONFLICT (nonce) DO NOTHING
        ",
    )
    .bind(nonce)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(inserted.rows_affected() > 0)
}
