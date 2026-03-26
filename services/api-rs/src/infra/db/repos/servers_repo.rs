use sqlx::{Executor, PgPool, Postgres, Row};

use crate::models::ServerSummary;

pub struct ServerInsertParams<'a> {
    pub server_id: &'a str,
    pub name: &'a str,
}

pub struct ServerMembershipInsertParams<'a> {
    pub server_id: &'a str,
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
        INSERT INTO servers (server_id, name)
        VALUES ($1, $2)
        ON CONFLICT (server_id) DO UPDATE
        SET name = EXCLUDED.name
        ",
    )
    .bind(params.server_id)
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
        INSERT INTO server_memberships (server_id, identity_id, favorite, muted, unread_count)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (server_id, identity_id) DO UPDATE
        SET favorite = EXCLUDED.favorite,
            muted = EXCLUDED.muted,
            unread_count = EXCLUDED.unread_count
        ",
    )
    .bind(params.server_id)
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
        INNER JOIN server_memberships second
            ON second.server_id = first.server_id
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
) -> Result<Vec<ServerSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT s.server_id, s.name, m.unread_count, m.favorite, m.muted
        FROM server_memberships m
        INNER JOIN servers s ON s.server_id = m.server_id
        WHERE m.identity_id = $1
        ORDER BY m.favorite DESC, s.name ASC, s.server_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_server_summary_row).collect()
}

pub async fn identity_has_server_membership(
    pool: &PgPool,
    identity_id: &str,
    server_id: &str,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM server_memberships
        WHERE identity_id = $1
          AND server_id = $2
        ",
    )
    .bind(identity_id)
    .bind(server_id)
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
        SELECT s.server_id, s.name, m.unread_count, m.favorite, m.muted
        FROM server_memberships m
        INNER JOIN servers s ON s.server_id = m.server_id
        WHERE m.identity_id = $1
          AND m.server_id = $2
        ",
    )
    .bind(identity_id)
    .bind(server_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_server_summary_row).transpose()
}

pub async fn list_server_member_identity_ids(
    pool: &PgPool,
    server_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "
        SELECT identity_id
        FROM server_memberships
        WHERE server_id = $1
        ORDER BY identity_id ASC
        ",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

fn map_server_summary_row(row: sqlx::postgres::PgRow) -> Result<ServerSummary, sqlx::Error> {
    let unread_count = row.try_get::<i32, _>("unread_count")?;
    let unread = u32::try_from(unread_count)
        .map_err(|_| sqlx::Error::Protocol("unread_count must be non-negative".into()))?;

    Ok(ServerSummary {
        id: row.try_get::<String, _>("server_id")?,
        name: row.try_get::<String, _>("name")?,
        unread,
        favorite: row.try_get::<bool, _>("favorite")?,
        muted: row.try_get::<bool, _>("muted")?,
    })
}
