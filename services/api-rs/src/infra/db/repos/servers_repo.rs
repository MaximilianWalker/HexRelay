use sqlx::{Executor, PgPool, Postgres, Row};

use crate::models::ServerSummary;

pub struct ServerInsertParams<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

pub struct ServerMembershipInsertParams<'a> {
    pub identity_id: &'a str,
    pub pinned: bool,
    pub muted: bool,
    pub unread_count: i32,
}

pub struct ServerPreferenceUpdateParams<'a> {
    pub identity_id: &'a str,
    pub pinned: Option<bool>,
    pub muted: Option<bool>,
}

pub struct ServerAdministratorInsertParams<'a> {
    pub identity_id: &'a str,
    pub is_owner: bool,
    pub is_admin: bool,
}

pub struct ServerBootstrapCredentialInsertParams<'a> {
    pub credential_id: &'a str,
    pub credential_secret_hash: &'a str,
    pub created_by_identity_id: &'a str,
}

pub async fn insert_server(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO local_server (singleton, name, description)
        VALUES (TRUE, $1, $2)
        ON CONFLICT (singleton) DO UPDATE
        SET name = EXCLUDED.name,
            description = EXCLUDED.description
        ",
    )
    .bind(params.name)
    .bind(params.description)
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
        INSERT INTO server_memberships (identity_id, pinned, muted, unread_count)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (identity_id) DO UPDATE
        SET pinned = EXCLUDED.pinned,
            muted = EXCLUDED.muted,
            unread_count = EXCLUDED.unread_count
        ",
    )
    .bind(params.identity_id)
    .bind(params.pinned)
    .bind(params.muted)
    .bind(params.unread_count)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn insert_server_administrator(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerAdministratorInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO server_administrators (identity_id, is_owner, is_admin)
        VALUES ($1, $2, $3)
        ON CONFLICT (identity_id) DO UPDATE
        SET is_owner = server_administrators.is_owner OR EXCLUDED.is_owner,
            is_admin = server_administrators.is_admin OR EXCLUDED.is_admin
        ",
    )
    .bind(params.identity_id)
    .bind(params.is_owner)
    .bind(params.is_admin)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn insert_server_bootstrap_credential(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerBootstrapCredentialInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO server_bootstrap_credentials (
            credential_id,
            credential_secret_hash,
            created_by_identity_id
        )
        VALUES ($1, $2, $3)
        ON CONFLICT (credential_secret_hash) DO NOTHING
        ",
    )
    .bind(params.credential_id)
    .bind(params.credential_secret_hash)
    .bind(params.created_by_identity_id)
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
        SELECT s.name, m.unread_count, m.pinned, m.muted
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
        SELECT s.name, m.unread_count, m.pinned, m.muted
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

pub async fn update_server_membership_preferences(
    pool: &PgPool,
    params: ServerPreferenceUpdateParams<'_>,
    server_id: &str,
) -> Result<Option<ServerSummary>, sqlx::Error> {
    let row = sqlx::query(
        "
        UPDATE server_memberships
        SET pinned = COALESCE($2, pinned),
            muted = COALESCE($3, muted)
        WHERE identity_id = $1
        RETURNING unread_count, pinned, muted
        ",
    )
    .bind(params.identity_id)
    .bind(params.pinned)
    .bind(params.muted)
    .fetch_optional(pool)
    .await?;

    let Some(membership_row) = row else {
        return Ok(None);
    };

    let server_name = sqlx::query_scalar::<_, String>(
        "
        SELECT name
        FROM local_server
        WHERE singleton = TRUE
        ",
    )
    .fetch_one(pool)
    .await?;

    let unread_count = membership_row.try_get::<i32, _>("unread_count")?;
    let unread = u32::try_from(unread_count)
        .map_err(|_| sqlx::Error::Protocol("unread_count must be non-negative".into()))?;

    Ok(Some(ServerSummary {
        id: server_id.to_string(),
        name: server_name,
        unread,
        pinned: membership_row.try_get::<bool, _>("pinned")?,
        muted: membership_row.try_get::<bool, _>("muted")?,
    }))
}

pub async fn delete_server_membership(
    pool: &PgPool,
    identity_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "
        DELETE FROM server_memberships
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn server_administration_for_identity(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Option<(bool, bool)>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT is_owner, is_admin
        FROM server_administrators
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        Ok((
            row.try_get::<bool, _>("is_owner")?,
            row.try_get::<bool, _>("is_admin")?,
        ))
    })
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
        pinned: row.try_get::<bool, _>("pinned")?,
        muted: row.try_get::<bool, _>("muted")?,
    })
}
