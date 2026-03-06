use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

use crate::models::{AuthChallengeRecord, RegisteredIdentityKey};

pub async fn insert_identity_key(
    pool: &PgPool,
    identity_id: &str,
    public_key: &str,
    algorithm: &str,
) -> Result<bool, sqlx::Error> {
    let inserted = sqlx::query(
        "
        INSERT INTO identity_keys (identity_id, public_key, algorithm)
        VALUES ($1, $2, $3)
        ON CONFLICT (identity_id) DO NOTHING
        ",
    )
    .bind(identity_id)
    .bind(public_key)
    .bind(algorithm)
    .execute(pool)
    .await?;

    Ok(inserted.rows_affected() > 0)
}

pub async fn identity_exists(pool: &PgPool, identity_id: &str) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM identity_keys
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn insert_auth_challenge(
    pool: &PgPool,
    challenge_id: &str,
    identity_id: &str,
    nonce: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO auth_challenges (challenge_id, identity_id, nonce, expires_at)
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(challenge_id)
    .bind(identity_id)
    .bind(nonce)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn consume_auth_challenge(
    pool: &PgPool,
    challenge_id: &str,
) -> Result<Option<AuthChallengeRecord>, sqlx::Error> {
    let row = sqlx::query(
        "
        DELETE FROM auth_challenges
        WHERE challenge_id = $1
        RETURNING identity_id, nonce, expires_at
        ",
    )
    .bind(challenge_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_auth_challenge_row).transpose()
}

pub async fn get_identity_key(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Option<RegisteredIdentityKey>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT public_key, algorithm
        FROM identity_keys
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_registered_key_row).transpose()
}

pub async fn insert_session(
    pool: &PgPool,
    session_id: &str,
    identity_id: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO sessions (session_id, identity_id, expires_at)
        VALUES ($1, $2, $3)
        ",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_session(pool: &PgPool, session_id: &str) -> Result<bool, sqlx::Error> {
    let updated = sqlx::query(
        "
        UPDATE sessions
        SET revoked_at = NOW()
        WHERE session_id = $1 AND revoked_at IS NULL
        ",
    )
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(updated.rows_affected() > 0)
}

fn map_auth_challenge_row(row: sqlx::postgres::PgRow) -> Result<AuthChallengeRecord, sqlx::Error> {
    Ok(AuthChallengeRecord {
        identity_id: row.try_get::<String, _>("identity_id")?,
        nonce: row.try_get::<String, _>("nonce")?,
        expires_at: row.try_get::<DateTime<Utc>, _>("expires_at")?,
    })
}

fn map_registered_key_row(
    row: sqlx::postgres::PgRow,
) -> Result<RegisteredIdentityKey, sqlx::Error> {
    Ok(RegisteredIdentityKey {
        public_key: row.try_get::<String, _>("public_key")?,
        algorithm: row.try_get::<String, _>("algorithm")?,
    })
}
