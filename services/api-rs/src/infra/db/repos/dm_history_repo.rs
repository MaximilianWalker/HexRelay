use sqlx::{PgPool, Row};

use crate::models::{DmMessageRecord, DmThreadSummary};

pub struct DmThreadInsertParams<'a> {
    pub thread_id: &'a str,
    pub kind: &'a str,
    pub title: &'a str,
}

pub struct DmThreadParticipantInsertParams<'a> {
    pub thread_id: &'a str,
    pub identity_id: &'a str,
    pub last_read_seq: u64,
}

pub struct DmMessageInsertParams<'a> {
    pub message_id: &'a str,
    pub thread_id: &'a str,
    pub author_id: &'a str,
    pub seq: u64,
    pub ciphertext: &'a str,
    pub created_at: &'a str,
    pub edited_at: Option<&'a str>,
}

pub async fn insert_dm_thread(
    pool: &PgPool,
    params: DmThreadInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_threads (thread_id, kind, title)
        VALUES ($1, $2, $3)
        ON CONFLICT (thread_id) DO UPDATE
        SET kind = EXCLUDED.kind,
            title = EXCLUDED.title
        ",
    )
    .bind(params.thread_id)
    .bind(params.kind)
    .bind(params.title)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_dm_thread_participant(
    pool: &PgPool,
    params: DmThreadParticipantInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_thread_participants (thread_id, identity_id, last_read_seq)
        VALUES ($1, $2, $3)
        ON CONFLICT (thread_id, identity_id) DO UPDATE
        SET last_read_seq = EXCLUDED.last_read_seq
        ",
    )
    .bind(params.thread_id)
    .bind(params.identity_id)
    .bind(
        i64::try_from(params.last_read_seq)
            .map_err(|_| sqlx::Error::Protocol("last_read_seq too large for storage".into()))?,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_dm_message(
    pool: &PgPool,
    params: DmMessageInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_messages (message_id, thread_id, author_id, seq, ciphertext, created_at, edited_at)
        VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::timestamptz)
        ON CONFLICT (message_id) DO UPDATE
        SET author_id = EXCLUDED.author_id,
            seq = EXCLUDED.seq,
            ciphertext = EXCLUDED.ciphertext,
            created_at = EXCLUDED.created_at,
            edited_at = EXCLUDED.edited_at
        ",
    )
    .bind(params.message_id)
    .bind(params.thread_id)
    .bind(params.author_id)
    .bind(i64::try_from(params.seq).map_err(|_| sqlx::Error::Protocol("seq too large for storage".into()))?)
    .bind(params.ciphertext)
    .bind(params.created_at)
    .bind(params.edited_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_dm_threads_for_identity(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DmThreadSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH participant_threads AS (
            SELECT thread_id, last_read_seq
            FROM dm_thread_participants
            WHERE identity_id = $1
        ),
        message_stats AS (
            SELECT
                m.thread_id,
                MAX(m.seq) AS last_message_seq,
                ARRAY_AGG(m.ciphertext ORDER BY m.seq DESC)[1] AS last_message_preview,
                ARRAY_AGG(TO_CHAR(m.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') ORDER BY m.seq DESC)[1] AS last_message_at
            FROM dm_messages m
            GROUP BY m.thread_id
        ),
        participant_lists AS (
            SELECT thread_id, ARRAY_AGG(identity_id ORDER BY identity_id) AS participant_ids
            FROM dm_thread_participants
            GROUP BY thread_id
        )
        SELECT
            t.thread_id,
            t.kind,
            t.title,
            pl.participant_ids,
            COALESCE(ms.last_message_seq, 0) AS last_message_seq,
            pt.last_read_seq,
            GREATEST(COALESCE(ms.last_message_seq, 0) - pt.last_read_seq, 0) AS unread,
            COALESCE(ms.last_message_preview, '') AS last_message_preview,
            COALESCE(ms.last_message_at, TO_CHAR(t.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')) AS last_message_at
        FROM participant_threads pt
        INNER JOIN dm_threads t ON t.thread_id = pt.thread_id
        INNER JOIN participant_lists pl ON pl.thread_id = t.thread_id
        LEFT JOIN message_stats ms ON ms.thread_id = t.thread_id
        ORDER BY COALESCE(ms.last_message_seq, 0) DESC, t.thread_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_dm_thread_row).collect()
}

pub async fn list_dm_thread_messages_for_identity(
    pool: &PgPool,
    identity_id: &str,
    thread_id: &str,
) -> Result<Option<Vec<DmMessageRecord>>, sqlx::Error> {
    let visible = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM dm_thread_participants
        WHERE identity_id = $1 AND thread_id = $2
        ",
    )
    .bind(identity_id)
    .bind(thread_id)
    .fetch_one(pool)
    .await?;

    if visible == 0 {
        return Ok(None);
    }

    let rows = sqlx::query(
        r#"
        SELECT message_id, thread_id, author_id, seq, ciphertext,
               TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
               CASE
                   WHEN edited_at IS NULL THEN NULL
                   ELSE TO_CHAR(edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
               END AS edited_at
        FROM dm_messages
        WHERE thread_id = $1
        ORDER BY seq DESC
        "#,
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_dm_message_row)
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn map_dm_thread_row(row: sqlx::postgres::PgRow) -> Result<DmThreadSummary, sqlx::Error> {
    let unread = row.try_get::<i64, _>("unread")?;
    let last_read_seq = row.try_get::<i64, _>("last_read_seq")?;
    let last_message_seq = row.try_get::<i64, _>("last_message_seq")?;

    Ok(DmThreadSummary {
        thread_id: row.try_get::<String, _>("thread_id")?,
        kind: row.try_get::<String, _>("kind")?,
        title: row.try_get::<String, _>("title")?,
        participant_ids: row.try_get::<Vec<String>, _>("participant_ids")?,
        unread: u32::try_from(unread)
            .map_err(|_| sqlx::Error::Protocol("unread must be non-negative".into()))?,
        last_read_seq: u64::try_from(last_read_seq)
            .map_err(|_| sqlx::Error::Protocol("last_read_seq must be non-negative".into()))?,
        last_message_seq: u64::try_from(last_message_seq)
            .map_err(|_| sqlx::Error::Protocol("last_message_seq must be non-negative".into()))?,
        last_message_preview: row.try_get::<String, _>("last_message_preview")?,
        last_message_at: row.try_get::<String, _>("last_message_at")?,
    })
}

fn map_dm_message_row(row: sqlx::postgres::PgRow) -> Result<DmMessageRecord, sqlx::Error> {
    let seq = row.try_get::<i64, _>("seq")?;

    Ok(DmMessageRecord {
        message_id: row.try_get::<String, _>("message_id")?,
        thread_id: row.try_get::<String, _>("thread_id")?,
        author_id: row.try_get::<String, _>("author_id")?,
        seq: u64::try_from(seq)
            .map_err(|_| sqlx::Error::Protocol("seq must be non-negative".into()))?,
        ciphertext: row.try_get::<String, _>("ciphertext")?,
        created_at: row.try_get::<String, _>("created_at")?,
        edited_at: row.try_get::<Option<String>, _>("edited_at")?,
    })
}
