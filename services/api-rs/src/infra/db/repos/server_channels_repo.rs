use sqlx::{Executor, PgPool, Postgres, Row, Transaction};

use crate::models::{ServerChannelMessage, ServerChannelSummary};

pub struct ServerChannelInsertParams<'a> {
    pub channel_id: &'a str,
    pub server_id: &'a str,
    pub name: &'a str,
    pub kind: &'a str,
}

pub struct ServerChannelMessageInsertParams<'a> {
    pub message_id: &'a str,
    pub channel_id: &'a str,
    pub author_id: &'a str,
    pub channel_seq: u64,
    pub content: &'a str,
    pub reply_to_message_id: Option<&'a str>,
    pub created_at: &'a str,
    pub edited_at: Option<&'a str>,
    pub deleted_at: Option<&'a str>,
}

pub struct CreateServerChannelMessageParams {
    pub server_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    pub content: String,
    pub reply_to_message_id: Option<String>,
    pub mention_identity_ids: Vec<String>,
    pub created_at: String,
}

pub struct UpdateServerChannelMessageParams {
    pub server_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    pub content: String,
    pub mention_identity_ids: Vec<String>,
    pub edited_at: String,
}

pub struct UpdateServerChannelMessageResult {
    pub message: ServerChannelMessage,
    pub changed: bool,
}

pub struct SoftDeleteServerChannelMessageResult {
    pub message: ServerChannelMessage,
    pub changed: bool,
}

pub enum CreateServerChannelMessageError {
    ChannelNotFound,
    AuthorNotMember,
    ReplyTargetInvalid,
    MentionTargetInvalid,
    Storage(sqlx::Error),
}

pub enum UpdateServerChannelMessageError {
    ChannelNotFound,
    MessageNotFound,
    AuthorNotMember,
    EditForbidden,
    MessageDeleted,
    MentionTargetInvalid,
    Storage(sqlx::Error),
}

pub enum SoftDeleteServerChannelMessageError {
    ChannelNotFound,
    MessageNotFound,
    AuthorNotMember,
    DeleteForbidden,
    Storage(sqlx::Error),
}

impl From<sqlx::Error> for CreateServerChannelMessageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(value)
    }
}

impl From<sqlx::Error> for UpdateServerChannelMessageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(value)
    }
}

impl From<sqlx::Error> for SoftDeleteServerChannelMessageError {
    fn from(value: sqlx::Error) -> Self {
        Self::Storage(value)
    }
}

pub async fn insert_server_channel(
    executor: impl Executor<'_, Database = Postgres>,
    params: ServerChannelInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO server_channels (channel_id, server_id, name, kind)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (channel_id) DO UPDATE
        SET server_id = EXCLUDED.server_id,
            name = EXCLUDED.name,
            kind = EXCLUDED.kind
        ",
    )
    .bind(params.channel_id)
    .bind(params.server_id)
    .bind(params.name)
    .bind(params.kind)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn insert_server_channel_message(
    pool: &PgPool,
    params: ServerChannelMessageInsertParams<'_>,
    mention_identity_ids: &[&str],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "
        UPDATE server_channels
        SET last_message_seq = GREATEST(last_message_seq, $2)
        WHERE channel_id = $1
        ",
    )
    .bind(params.channel_id)
    .bind(
        i64::try_from(params.channel_seq)
            .map_err(|_| sqlx::Error::Protocol("channel_seq too large for storage".into()))?,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "
        INSERT INTO server_channel_messages (
            message_id,
            channel_id,
            author_id,
            channel_seq,
            content,
            reply_to_message_id,
            created_at,
            edited_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, $8::timestamptz, $9::timestamptz)
        ON CONFLICT (message_id) DO UPDATE
        SET author_id = EXCLUDED.author_id,
            channel_seq = EXCLUDED.channel_seq,
            content = EXCLUDED.content,
            reply_to_message_id = EXCLUDED.reply_to_message_id,
            created_at = EXCLUDED.created_at,
            edited_at = EXCLUDED.edited_at,
            deleted_at = EXCLUDED.deleted_at
        ",
    )
    .bind(params.message_id)
    .bind(params.channel_id)
    .bind(params.author_id)
    .bind(
        i64::try_from(params.channel_seq)
            .map_err(|_| sqlx::Error::Protocol("channel_seq too large for storage".into()))?,
    )
    .bind(params.content)
    .bind(params.reply_to_message_id)
    .bind(params.created_at)
    .bind(params.edited_at)
    .bind(params.deleted_at)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM server_channel_message_mentions WHERE message_id = $1")
        .bind(params.message_id)
        .execute(&mut *tx)
        .await?;

    for mentioned_identity_id in mention_identity_ids {
        sqlx::query(
            "
            INSERT INTO server_channel_message_mentions (message_id, mentioned_identity_id)
            VALUES ($1, $2)
            ON CONFLICT (message_id, mentioned_identity_id) DO NOTHING
            ",
        )
        .bind(params.message_id)
        .bind(mentioned_identity_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

pub async fn list_server_channels(
    pool: &PgPool,
    server_id: &str,
) -> Result<Vec<ServerChannelSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT channel_id, name, kind, last_message_seq
        FROM server_channels
        WHERE server_id = $1
        ORDER BY created_at ASC, channel_id ASC
        ",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_server_channel_summary_row)
        .collect()
}

pub async fn server_channel_exists(
    pool: &PgPool,
    server_id: &str,
    channel_id: &str,
) -> Result<bool, sqlx::Error> {
    server_channel_exists_with_executor(pool, server_id, channel_id).await
}

pub async fn channel_id_exists(pool: &PgPool, channel_id: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "
        SELECT EXISTS (
            SELECT 1
            FROM server_channels
            WHERE channel_id = $1
        )
        ",
    )
    .bind(channel_id)
    .fetch_one(pool)
    .await
}

pub async fn list_server_channel_messages(
    pool: &PgPool,
    server_id: &str,
    channel_id: &str,
    cursor: Option<u64>,
    limit: usize,
) -> Result<Option<Vec<ServerChannelMessage>>, sqlx::Error> {
    if !server_channel_exists(pool, server_id, channel_id).await? {
        return Ok(None);
    }

    let fetch_limit = limit
        .checked_add(1)
        .ok_or_else(|| sqlx::Error::Protocol("limit too large for pagination".into()))?;
    let fetch_limit = i64::try_from(fetch_limit)
        .map_err(|_| sqlx::Error::Protocol("limit too large for storage".into()))?;
    let cursor = match cursor {
        Some(value) => Some(
            i64::try_from(value)
                .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?,
        ),
        None => None,
    };

    let rows = sqlx::query(
        r#"
        SELECT
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            COALESCE(
                ARRAY_AGG(scm.mentioned_identity_id ORDER BY scm.mentioned_identity_id)
                    FILTER (WHERE scm.mentioned_identity_id IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS mentions,
            TO_CHAR(m.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            CASE
                WHEN m.edited_at IS NULL THEN NULL
                ELSE TO_CHAR(m.edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS edited_at,
            CASE
                WHEN m.deleted_at IS NULL THEN NULL
                ELSE TO_CHAR(m.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS deleted_at
        FROM server_channel_messages m
        LEFT JOIN server_channel_message_mentions scm ON scm.message_id = m.message_id
        WHERE m.channel_id = $1
          AND ($2::BIGINT IS NULL OR m.channel_seq < $2)
        GROUP BY
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            m.created_at,
            m.edited_at,
            m.deleted_at
        ORDER BY m.channel_seq DESC
        LIMIT $3
        "#,
    )
    .bind(channel_id)
    .bind(cursor)
    .bind(fetch_limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_server_channel_message_row)
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub async fn create_server_channel_message(
    pool: &PgPool,
    params: CreateServerChannelMessageParams,
) -> Result<ServerChannelMessage, CreateServerChannelMessageError> {
    let mut tx = pool.begin().await?;

    let next_seq = sqlx::query_scalar::<_, i64>(
        "
        UPDATE server_channels
        SET last_message_seq = last_message_seq + 1
        WHERE server_id = $1 AND channel_id = $2
        RETURNING last_message_seq
        ",
    )
    .bind(&params.server_id)
    .bind(&params.channel_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(CreateServerChannelMessageError::ChannelNotFound)?;

    if !server_membership_exists(&mut tx, &params.server_id, &params.author_id).await? {
        return Err(CreateServerChannelMessageError::AuthorNotMember);
    }

    if let Some(reply_to_message_id) = params.reply_to_message_id.as_ref() {
        let reply_exists = sqlx::query_scalar::<_, i64>(
            "
            SELECT COUNT(*)
            FROM server_channel_messages
            WHERE channel_id = $1 AND message_id = $2
            ",
        )
        .bind(&params.channel_id)
        .bind(reply_to_message_id)
        .fetch_one(&mut *tx)
        .await?;

        if reply_exists == 0 {
            return Err(CreateServerChannelMessageError::ReplyTargetInvalid);
        }
    }

    if !params.mention_identity_ids.is_empty() {
        let mention_count = sqlx::query_scalar::<_, i64>(
            "
            SELECT COUNT(*)
            FROM server_memberships
            WHERE server_id = $1 AND identity_id = ANY($2)
            ",
        )
        .bind(&params.server_id)
        .bind(&params.mention_identity_ids)
        .fetch_one(&mut *tx)
        .await?;

        if mention_count != params.mention_identity_ids.len() as i64 {
            return Err(CreateServerChannelMessageError::MentionTargetInvalid);
        }
    }

    sqlx::query(
        "
        INSERT INTO server_channel_messages (
            message_id,
            channel_id,
            author_id,
            channel_seq,
            content,
            reply_to_message_id,
            created_at,
            edited_at,
            deleted_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7::timestamptz, NULL, NULL)
        ",
    )
    .bind(&params.message_id)
    .bind(&params.channel_id)
    .bind(&params.author_id)
    .bind(next_seq)
    .bind(&params.content)
    .bind(params.reply_to_message_id.as_deref())
    .bind(&params.created_at)
    .execute(&mut *tx)
    .await?;

    for mentioned_identity_id in &params.mention_identity_ids {
        sqlx::query(
            "
            INSERT INTO server_channel_message_mentions (message_id, mentioned_identity_id)
            VALUES ($1, $2)
            ",
        )
        .bind(&params.message_id)
        .bind(mentioned_identity_id)
        .execute(&mut *tx)
        .await?;
    }

    let row = sqlx::query(
        r#"
        SELECT
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            COALESCE(
                ARRAY_AGG(scm.mentioned_identity_id ORDER BY scm.mentioned_identity_id)
                    FILTER (WHERE scm.mentioned_identity_id IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS mentions,
            TO_CHAR(m.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            CASE
                WHEN m.edited_at IS NULL THEN NULL
                ELSE TO_CHAR(m.edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS edited_at,
            CASE
                WHEN m.deleted_at IS NULL THEN NULL
                ELSE TO_CHAR(m.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS deleted_at
        FROM server_channel_messages m
        LEFT JOIN server_channel_message_mentions scm ON scm.message_id = m.message_id
        WHERE m.message_id = $1
        GROUP BY
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            m.created_at,
            m.edited_at,
            m.deleted_at
        "#,
    )
    .bind(&params.message_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    map_server_channel_message_row(row).map_err(CreateServerChannelMessageError::Storage)
}

pub async fn update_server_channel_message(
    pool: &PgPool,
    params: UpdateServerChannelMessageParams,
) -> Result<UpdateServerChannelMessageResult, UpdateServerChannelMessageError> {
    let mut tx = pool.begin().await?;

    let message = get_message_for_mutation(
        &mut tx,
        &params.server_id,
        &params.channel_id,
        &params.message_id,
    )
    .await?;

    let message = match message {
        Some(message) => message,
        None => {
            if channel_exists(&mut tx, &params.server_id, &params.channel_id).await? {
                return Err(UpdateServerChannelMessageError::MessageNotFound);
            }
            return Err(UpdateServerChannelMessageError::ChannelNotFound);
        }
    };

    if message.author_id != params.author_id {
        return Err(UpdateServerChannelMessageError::EditForbidden);
    }

    if !server_membership_exists(&mut tx, &params.server_id, &params.author_id).await? {
        return Err(UpdateServerChannelMessageError::AuthorNotMember);
    }

    if message.deleted_at.is_some() {
        return Err(UpdateServerChannelMessageError::MessageDeleted);
    }

    if !params.mention_identity_ids.is_empty() {
        let mention_count = sqlx::query_scalar::<_, i64>(
            "
            SELECT COUNT(*)
            FROM server_memberships
            WHERE server_id = $1 AND identity_id = ANY($2)
            ",
        )
        .bind(&params.server_id)
        .bind(&params.mention_identity_ids)
        .fetch_one(&mut *tx)
        .await?;

        if mention_count != params.mention_identity_ids.len() as i64 {
            return Err(UpdateServerChannelMessageError::MentionTargetInvalid);
        }
    }

    let is_noop =
        message.content == params.content && message.mentions == params.mention_identity_ids;
    if !is_noop {
        sqlx::query(
            "
            UPDATE server_channel_messages
            SET content = $1,
                edited_at = $2::timestamptz
            WHERE message_id = $3
              AND channel_id = $4
              AND author_id = $5
              AND deleted_at IS NULL
            ",
        )
        .bind(&params.content)
        .bind(&params.edited_at)
        .bind(&params.message_id)
        .bind(&params.channel_id)
        .bind(&params.author_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM server_channel_message_mentions WHERE message_id = $1")
            .bind(&params.message_id)
            .execute(&mut *tx)
            .await?;

        for mentioned_identity_id in &params.mention_identity_ids {
            sqlx::query(
                "
                INSERT INTO server_channel_message_mentions (message_id, mentioned_identity_id)
                VALUES ($1, $2)
                ",
            )
            .bind(&params.message_id)
            .bind(mentioned_identity_id)
            .execute(&mut *tx)
            .await?;
        }
    }

    let row = fetch_message_row(&mut tx, &params.message_id).await?;
    tx.commit().await?;
    map_server_channel_message_row(row)
        .map(|message| UpdateServerChannelMessageResult {
            message,
            changed: !is_noop,
        })
        .map_err(UpdateServerChannelMessageError::Storage)
}

pub async fn soft_delete_server_channel_message(
    pool: &PgPool,
    server_id: &str,
    channel_id: &str,
    message_id: &str,
    author_id: &str,
    deleted_at: &str,
) -> Result<SoftDeleteServerChannelMessageResult, SoftDeleteServerChannelMessageError> {
    let mut tx = pool.begin().await?;

    let message = get_message_for_mutation(&mut tx, server_id, channel_id, message_id).await?;

    let message = match message {
        Some(message) => message,
        None => {
            if channel_exists(&mut tx, server_id, channel_id).await? {
                return Err(SoftDeleteServerChannelMessageError::MessageNotFound);
            }
            return Err(SoftDeleteServerChannelMessageError::ChannelNotFound);
        }
    };

    if message.author_id != author_id {
        return Err(SoftDeleteServerChannelMessageError::DeleteForbidden);
    }

    if !server_membership_exists(&mut tx, server_id, author_id).await? {
        return Err(SoftDeleteServerChannelMessageError::AuthorNotMember);
    }

    let changed = message.deleted_at.is_none();
    if changed {
        sqlx::query(
            "
            UPDATE server_channel_messages
            SET content = '',
                deleted_at = $1::timestamptz
            WHERE message_id = $2
              AND channel_id = $3
              AND author_id = $4
              AND deleted_at IS NULL
            ",
        )
        .bind(deleted_at)
        .bind(message_id)
        .bind(channel_id)
        .bind(author_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query("DELETE FROM server_channel_message_mentions WHERE message_id = $1")
            .bind(message_id)
            .execute(&mut *tx)
            .await?;
    }

    let row = fetch_message_row(&mut tx, message_id).await?;
    tx.commit().await?;
    map_server_channel_message_row(row)
        .map(|message| SoftDeleteServerChannelMessageResult { message, changed })
        .map_err(SoftDeleteServerChannelMessageError::Storage)
}

struct MutationMessageState {
    author_id: String,
    content: String,
    mentions: Vec<String>,
    deleted_at: Option<String>,
}

async fn channel_exists(
    tx: &mut Transaction<'_, Postgres>,
    server_id: &str,
    channel_id: &str,
) -> Result<bool, sqlx::Error> {
    server_channel_exists_with_executor(&mut **tx, server_id, channel_id).await
}

async fn server_membership_exists(
    tx: &mut Transaction<'_, Postgres>,
    server_id: &str,
    identity_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "
        SELECT EXISTS (
            SELECT 1
            FROM server_memberships
            WHERE server_id = $1 AND identity_id = $2
        )
        ",
    )
    .bind(server_id)
    .bind(identity_id)
    .fetch_one(&mut **tx)
    .await
}

async fn server_channel_exists_with_executor<'e, E>(
    executor: E,
    server_id: &str,
    channel_id: &str,
) -> Result<bool, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar::<_, bool>(
        "
        SELECT EXISTS (
            SELECT 1
            FROM server_channels
            WHERE server_id = $1 AND channel_id = $2
        )
        ",
    )
    .bind(server_id)
    .bind(channel_id)
    .fetch_one(executor)
    .await
}

async fn get_message_for_mutation(
    tx: &mut Transaction<'_, Postgres>,
    server_id: &str,
    channel_id: &str,
    message_id: &str,
) -> Result<Option<MutationMessageState>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        WITH locked_message AS (
            SELECT m.message_id, m.author_id, m.content, m.deleted_at
            FROM server_channel_messages m
            INNER JOIN server_channels c ON c.channel_id = m.channel_id
            WHERE c.server_id = $1 AND m.channel_id = $2 AND m.message_id = $3
            FOR UPDATE OF m
        )
        SELECT
            lm.author_id,
            lm.content,
            COALESCE(
                ARRAY_AGG(scm.mentioned_identity_id ORDER BY scm.mentioned_identity_id)
                    FILTER (WHERE scm.mentioned_identity_id IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS mentions,
            CASE
                WHEN lm.deleted_at IS NULL THEN NULL
                ELSE TO_CHAR(lm.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS deleted_at
        FROM locked_message lm
        LEFT JOIN server_channel_message_mentions scm ON scm.message_id = lm.message_id
        GROUP BY lm.author_id, lm.content, lm.deleted_at
        "#,
    )
    .bind(server_id)
    .bind(channel_id)
    .bind(message_id)
    .fetch_optional(&mut **tx)
    .await?;

    row.map(|row| {
        Ok(MutationMessageState {
            author_id: row.try_get::<String, _>("author_id")?,
            content: row.try_get::<String, _>("content")?,
            mentions: row.try_get::<Vec<String>, _>("mentions")?,
            deleted_at: row.try_get::<Option<String>, _>("deleted_at")?,
        })
    })
    .transpose()
}

async fn fetch_message_row(
    tx: &mut Transaction<'_, Postgres>,
    message_id: &str,
) -> Result<sqlx::postgres::PgRow, sqlx::Error> {
    sqlx::query(
        r#"
        SELECT
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            COALESCE(
                ARRAY_AGG(scm.mentioned_identity_id ORDER BY scm.mentioned_identity_id)
                    FILTER (WHERE scm.mentioned_identity_id IS NOT NULL),
                ARRAY[]::TEXT[]
            ) AS mentions,
            TO_CHAR(m.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
            CASE
                WHEN m.edited_at IS NULL THEN NULL
                ELSE TO_CHAR(m.edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS edited_at,
            CASE
                WHEN m.deleted_at IS NULL THEN NULL
                ELSE TO_CHAR(m.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            END AS deleted_at
        FROM server_channel_messages m
        LEFT JOIN server_channel_message_mentions scm ON scm.message_id = m.message_id
        WHERE m.message_id = $1
        GROUP BY
            m.message_id,
            m.channel_id,
            m.author_id,
            m.channel_seq,
            m.content,
            m.reply_to_message_id,
            m.created_at,
            m.edited_at,
            m.deleted_at
        "#,
    )
    .bind(message_id)
    .fetch_one(&mut **tx)
    .await
}

fn map_server_channel_message_row(
    row: sqlx::postgres::PgRow,
) -> Result<ServerChannelMessage, sqlx::Error> {
    let channel_seq = row.try_get::<i64, _>("channel_seq")?;

    Ok(ServerChannelMessage {
        message_id: row.try_get::<String, _>("message_id")?,
        channel_id: row.try_get::<String, _>("channel_id")?,
        author_id: row.try_get::<String, _>("author_id")?,
        channel_seq: u64::try_from(channel_seq)
            .map_err(|_| sqlx::Error::Protocol("channel_seq must be non-negative".into()))?,
        content: row.try_get::<String, _>("content")?,
        reply_to_message_id: row.try_get::<Option<String>, _>("reply_to_message_id")?,
        mentions: row.try_get::<Vec<String>, _>("mentions")?,
        created_at: row.try_get::<String, _>("created_at")?,
        edited_at: row.try_get::<Option<String>, _>("edited_at")?,
        deleted_at: row.try_get::<Option<String>, _>("deleted_at")?,
    })
}

fn map_server_channel_summary_row(
    row: sqlx::postgres::PgRow,
) -> Result<ServerChannelSummary, sqlx::Error> {
    let last_message_seq = row.try_get::<i64, _>("last_message_seq")?;
    let last_message_seq = u64::try_from(last_message_seq)
        .map_err(|_| sqlx::Error::Protocol("last_message_seq must be non-negative".into()))?;

    Ok(ServerChannelSummary {
        id: row.try_get::<String, _>("channel_id")?,
        name: row.try_get::<String, _>("name")?,
        kind: row.try_get::<String, _>("kind")?,
        last_message_seq,
    })
}
