use sqlx::{Executor, PgPool, Postgres, Row};

use crate::models::ServerSummary;

pub struct ServerInsertParams<'a> {
    pub name: &'a str,
}

pub struct ServerMembershipInsertParams<'a> {
    pub identity_id: &'a str,
    pub favorite: bool,
    pub muted: bool,
    pub unread_count: i32,
}

pub async fn insert_server(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO local_server (singleton, name)
        VALUES (TRUE, $1)
        ON CONFLICT (singleton) DO UPDATE
        SET name = EXCLUDED.name
        ",
    )
    .bind(params.name)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn insert_server_membership(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerMembershipInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO server_memberships (identity_id, favorite, muted, unread_count)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (identity_id) DO UPDATE
        SET favorite = EXCLUDED.favorite,
            muted = EXCLUDED.muted,
            unread_count = EXCLUDED.unread_count
        ",
    )
    .bind(params.identity_id)
    .bind(params.favorite)
    .bind(params.muted)
    .bind(params.unread_count)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn identities_share_server(
    pool: &PgPool,
    identity_a: &str,
    identity_b: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM server_memberships first
        CROSS JOIN server_memberships second
        WHERE first.identity_id = $1
          AND second.identity_id = $2
        ",
    )
    .bind(identity_a)
    .bind(identity_b)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn list_servers_for_identity(
    pool: &PgPool,
    identity_id: &str,
    local_server_id: &str,
) -> Result<Vec<ServerSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT s.name, m.unread_count, m.favorite, m.muted
        FROM server_memberships m
        CROSS JOIN local_server s
        WHERE m.identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| map_server_summary_row(row, local_server_id))
        .collect()
}

pub async fn identity_has_server_membership(
    pool: &PgPool,
    identity_id: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM server_memberships
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_one(pool)
    .await?;

    Ok(count > 0)
}

pub async fn get_server_for_identity(
    pool: &PgPool,
    identity_id: &str,
    server_id: &str,
) -> Result<Option<ServerSummary>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT s.name, m.unread_count, m.favorite, m.muted
        FROM server_memberships m
        CROSS JOIN local_server s
        WHERE m.identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| map_server_summary_row(row, server_id))
        .transpose()
}

pub async fn list_server_member_identity_ids(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "
        SELECT identity_id
        FROM server_memberships
        ORDER BY identity_id ASC
        ",
    )
    .fetch_all(pool)
    .await
}

fn map_server_summary_row(
    row: sqlx::postgres::PgRow,
    server_id: &str,
) -> Result<ServerSummary, sqlx::Error> {
    let unread_count = row.try_get::<i32, _>("unread_count")?;
    let unread = u32::try_from(unread_count)
        .map_err(|_| sqlx::Error::Protocol("unread_count must be non-negative".into()))?;

    Ok(ServerSummary {
        id: server_id.to_string(),
        name: row.try_get::<String, _>("name")?,
        unread,
        favorite: row.try_get::<bool, _>("favorite")?,
        muted: row.try_get::<bool, _>("muted")?,
    })
}
