use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool, Postgres, Row};

pub struct InviteRedeemRow {
    pub invite_id: Option<String>,
    pub creator_identity_id: Option<String>,
    pub node_fingerprint: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
    pub uses: i32,
}

pub struct InviteInsertParams<'a> {
    pub invite_id: &'a str,
    pub token_hash: &'a str,
    pub mode: &'a str,
    pub creator_identity_id: &'a str,
    pub node_fingerprint: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<i32>,
}

pub async fn insert_invite(
    pool: &PgPool,
    params: InviteInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO invites (invite_id, token, mode, creator_identity_id, node_fingerprint, expires_at, max_uses, uses)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0)
        ",
    )
    .bind(params.invite_id)
    .bind(params.token_hash)
    .bind(params.mode)
    .bind(params.creator_identity_id)
    .bind(params.node_fingerprint)
    .bind(params.expires_at)
    .bind(params.max_uses)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn load_invite_for_update(
    executor: impl Executor<'_, Database = Postgres>,
    token_hash: &str,
) -> Result<Option<InviteRedeemRow>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT invite_id, creator_identity_id, node_fingerprint, expires_at, max_uses, uses
        FROM invites
        WHERE token = $1
        FOR UPDATE
        ",
    )
    .bind(token_hash)
    .fetch_optional(executor)
    .await?;

    row.map(map_invite_redeem_row).transpose()
}

pub async fn increment_invite_use(
    executor: impl Executor<'_, Database = Postgres>,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        UPDATE invites
        SET uses = uses + 1
        WHERE token = $1
        ",
    )
    .bind(token_hash)
    .execute(executor)
    .await?;

    Ok(())
}

fn map_invite_redeem_row(row: sqlx::postgres::PgRow) -> Result<InviteRedeemRow, sqlx::Error> {
    Ok(InviteRedeemRow {
        invite_id: row.try_get::<Option<String>, _>("invite_id")?,
        creator_identity_id: row.try_get::<Option<String>, _>("creator_identity_id")?,
        node_fingerprint: row.try_get::<String, _>("node_fingerprint")?,
        expires_at: row.try_get::<Option<DateTime<Utc>>, _>("expires_at")?,
        max_uses: row.try_get::<Option<i32>, _>("max_uses")?,
        uses: row.try_get::<i32, _>("uses")?,
    })
}
