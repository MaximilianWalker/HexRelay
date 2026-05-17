use sqlx::{PgPool, Postgres, Row, Transaction};

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

pub fn direct_dm_thread_id(identity_a: &str, identity_b: &str) -> String {
    let (left, right) = if identity_a <= identity_b {
        (identity_a, identity_b)
    } else {
        (identity_b, identity_a)
    };
    format!("dm-{left}-{right}")
}

pub fn direct_dm_thread_title(identity_a: &str, identity_b: &str) -> String {
    let (left, right) = if identity_a <= identity_b {
        (identity_a, identity_b)
    } else {
        (identity_b, identity_a)
    };
    format!("{left} + {right}")
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

    refresh_dm_thread_last_message(pool, params.thread_id).await?;

    Ok(())
}

pub async fn insert_dm_thread_participant(
    pool: &PgPool,
    params: DmThreadParticipantInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_thread_participants (thread_id, identity_id, last_read_seq, last_message_seq)
        VALUES (
            $1,
            $2,
            $3,
            COALESCE((SELECT last_message_seq FROM dm_threads WHERE thread_id = $1), 0)
        )
        ON CONFLICT (thread_id, identity_id) DO UPDATE
        SET last_read_seq = EXCLUDED.last_read_seq,
            last_message_seq = EXCLUDED.last_message_seq
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

    refresh_dm_thread_last_message(pool, params.thread_id).await?;

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

    refresh_dm_thread_last_message(pool, params.thread_id).await?;

    Ok(())
}

pub async fn ensure_direct_dm_thread_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    identity_a: &str,
    identity_b: &str,
) -> Result<String, sqlx::Error> {
    let thread_id = direct_dm_thread_id(identity_a, identity_b);
    let title = direct_dm_thread_title(identity_a, identity_b);

    sqlx::query(
        "
        INSERT INTO dm_threads (thread_id, kind, title)
        VALUES ($1, 'dm', $2)
        ON CONFLICT (thread_id) DO NOTHING
        ",
    )
    .bind(&thread_id)
    .bind(&title)
    .execute(&mut **tx)
    .await?;

    for identity_id in [identity_a, identity_b] {
        sqlx::query(
            "
            INSERT INTO dm_thread_participants (thread_id, identity_id, last_read_seq)
            VALUES ($1, $2, 0)
            ON CONFLICT (thread_id, identity_id) DO NOTHING
            ",
        )
        .bind(&thread_id)
        .bind(identity_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(thread_id)
}

pub async fn next_dm_message_seq_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    thread_id: &str,
) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT thread_id FROM dm_threads WHERE thread_id = $1 FOR UPDATE",
    )
    .bind(thread_id)
    .fetch_one(&mut **tx)
    .await?;

    let current = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(seq), 0) FROM dm_messages WHERE thread_id = $1",
    )
    .bind(thread_id)
    .fetch_one(&mut **tx)
    .await?;

    let next = current
        .checked_add(1)
        .ok_or_else(|| sqlx::Error::Protocol("dm message seq overflow".into()))?;
    u64::try_from(next).map_err(|_| sqlx::Error::Protocol("seq must be non-negative".into()))
}

pub async fn insert_dm_message_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    params: DmMessageInsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_messages (message_id, thread_id, author_id, seq, ciphertext, created_at, edited_at)
        VALUES ($1, $2, $3, $4, $5, $6::timestamptz, $7::timestamptz)
        ",
    )
    .bind(params.message_id)
    .bind(params.thread_id)
    .bind(params.author_id)
    .bind(i64::try_from(params.seq).map_err(|_| sqlx::Error::Protocol("seq too large for storage".into()))?)
    .bind(params.ciphertext)
    .bind(params.created_at)
    .bind(params.edited_at)
    .execute(&mut **tx)
    .await?;

    refresh_dm_thread_last_message_in_tx(tx, params.thread_id).await?;

    Ok(())
}

async fn refresh_dm_thread_last_message(pool: &PgPool, thread_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        WITH latest AS (
            SELECT seq, ciphertext, created_at
            FROM dm_messages
            WHERE thread_id = $1
            ORDER BY seq DESC
            LIMIT 1
        ),
        updated_thread AS (
            UPDATE dm_threads
            SET last_message_seq = COALESCE((SELECT seq FROM latest), 0),
                last_message_preview = COALESCE((SELECT ciphertext FROM latest), ''),
                last_message_at = (SELECT created_at FROM latest)
            WHERE thread_id = $1
            RETURNING last_message_seq
        )
        UPDATE dm_thread_participants
        SET last_message_seq = (SELECT last_message_seq FROM updated_thread)
        WHERE thread_id = $1
        "#,
    )
    .bind(thread_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub(crate) async fn refresh_dm_thread_last_message_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    thread_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        WITH latest AS (
            SELECT seq, ciphertext, created_at
            FROM dm_messages
            WHERE thread_id = $1
            ORDER BY seq DESC
            LIMIT 1
        ),
        updated_thread AS (
            UPDATE dm_threads
            SET last_message_seq = COALESCE((SELECT seq FROM latest), 0),
                last_message_preview = COALESCE((SELECT ciphertext FROM latest), ''),
                last_message_at = (SELECT created_at FROM latest)
            WHERE thread_id = $1
            RETURNING last_message_seq
        )
        UPDATE dm_thread_participants
        SET last_message_seq = (SELECT last_message_seq FROM updated_thread)
        WHERE thread_id = $1
        "#,
    )
    .bind(thread_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

const LIST_DM_THREADS_SQL: &str = r#"
        WITH cursor_position AS (
            SELECT pt.last_message_seq
            FROM dm_thread_participants pt
            WHERE pt.identity_id = $1
              AND pt.thread_id = $2::TEXT
              AND ((NOT $4) OR GREATEST(pt.last_message_seq - pt.last_read_seq, 0) > 0)
        ),
        page_threads AS (
            SELECT
                t.thread_id,
                t.kind,
                t.title,
                pt.last_message_seq,
                pt.last_read_seq,
                GREATEST(pt.last_message_seq - pt.last_read_seq, 0) AS unread,
                t.last_message_preview,
                TO_CHAR(
                    COALESCE(t.last_message_at, t.created_at) AT TIME ZONE 'UTC',
                    'YYYY-MM-DD"T"HH24:MI:SS"Z"'
                ) AS last_message_at
            FROM dm_thread_participants pt
            INNER JOIN dm_threads t ON t.thread_id = pt.thread_id
            WHERE pt.identity_id = $1
              AND ((NOT $4) OR GREATEST(pt.last_message_seq - pt.last_read_seq, 0) > 0)
              AND (
                  $2::TEXT IS NULL
                  OR NOT EXISTS (SELECT 1 FROM cursor_position)
                  OR pt.last_message_seq < (SELECT last_message_seq FROM cursor_position)
                  OR (
                      pt.last_message_seq = (SELECT last_message_seq FROM cursor_position)
                      AND pt.thread_id > $2::TEXT
                  )
              )
            ORDER BY pt.last_message_seq DESC, pt.thread_id ASC
            LIMIT $3
        )
        SELECT
            page_threads.thread_id,
            page_threads.kind,
            page_threads.title,
            participants.participant_ids,
            page_threads.last_message_seq,
            page_threads.last_read_seq,
            page_threads.unread,
            page_threads.last_message_preview,
            page_threads.last_message_at
        FROM page_threads
        CROSS JOIN LATERAL (
            SELECT ARRAY_AGG(identity_id ORDER BY identity_id) AS participant_ids
            FROM dm_thread_participants
            WHERE thread_id = page_threads.thread_id
        ) participants
        ORDER BY page_threads.last_message_seq DESC, page_threads.thread_id ASC
        "#;

pub async fn list_dm_threads_for_identity(
    pool: &PgPool,
    identity_id: &str,
    cursor: Option<&str>,
    limit: usize,
    unread_only: bool,
) -> Result<Vec<DmThreadSummary>, sqlx::Error> {
    let fetch_limit = limit
        .checked_add(1)
        .ok_or_else(|| sqlx::Error::Protocol("limit too large for pagination".into()))?;
    let fetch_limit = i64::try_from(fetch_limit)
        .map_err(|_| sqlx::Error::Protocol("limit too large for storage".into()))?;

    if let Some(cursor_thread_id) = cursor {
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dm_thread_participants WHERE identity_id = $1 AND thread_id = $2",
        )
        .bind(identity_id)
        .bind(cursor_thread_id)
        .fetch_one(pool)
        .await?;

        if exists == 0 {
            return Err(sqlx::Error::RowNotFound);
        }
    }

    let rows = sqlx::query(LIST_DM_THREADS_SQL)
        .bind(identity_id)
        .bind(cursor)
        .bind(fetch_limit)
        .bind(unread_only)
        .fetch_all(pool)
        .await?;

    rows.into_iter().map(map_dm_thread_row).collect()
}

pub async fn list_dm_thread_messages_for_identity(
    pool: &PgPool,
    identity_id: &str,
    thread_id: &str,
    cursor: Option<u64>,
    limit: usize,
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

    let limit = limit
        .checked_add(1)
        .ok_or_else(|| sqlx::Error::Protocol("limit too large for pagination".into()))?;
    let limit = i64::try_from(limit)
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
        SELECT message_id, thread_id, author_id, seq, ciphertext,
               TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
               CASE
                   WHEN edited_at IS NULL THEN NULL
                   ELSE TO_CHAR(edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
               END AS edited_at
        FROM dm_messages
        WHERE thread_id = $1
          AND ($2::BIGINT IS NULL OR seq < $2)
        ORDER BY seq DESC
        LIMIT $3
        "#,
    )
    .bind(thread_id)
    .bind(cursor)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(map_dm_message_row)
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

pub async fn mark_dm_thread_read(
    pool: &PgPool,
    identity_id: &str,
    thread_id: &str,
    last_read_seq: u64,
) -> Result<Option<(u64, u32)>, sqlx::Error> {
    let seq = i64::try_from(last_read_seq)
        .map_err(|_| sqlx::Error::Protocol("last_read_seq too large for storage".into()))?;

    // Monotonic advance: only update if the new value exceeds the current one.
    // Returns NULL when the identity is not a participant (membership check).
    let row = sqlx::query(
        r#"
        UPDATE dm_thread_participants
        SET last_read_seq = GREATEST(last_read_seq, $3)
        WHERE thread_id = $1 AND identity_id = $2
        RETURNING last_read_seq,
                  GREATEST(last_message_seq - last_read_seq, 0) AS unread
        "#,
    )
    .bind(thread_id)
    .bind(identity_id)
    .bind(seq)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let new_seq = r.try_get::<i64, _>("last_read_seq")?;
            let unread = r.try_get::<i64, _>("unread")?;
            Ok(Some((
                u64::try_from(new_seq).map_err(|_| {
                    sqlx::Error::Protocol("last_read_seq must be non-negative".into())
                })?,
                u32::try_from(unread)
                    .map_err(|_| sqlx::Error::Protocol("unread must be non-negative".into()))?,
            )))
        }
        None => Ok(None),
    }
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

#[cfg(test)]
mod tests {
    use super::LIST_DM_THREADS_SQL;

    #[test]
    fn list_dm_threads_query_uses_identity_keyset_without_global_rank() {
        for marker in [
            concat!("message", "_stats AS"),
            concat!("participant", "_lists AS"),
            "ROW_NUMBER",
            concat!("GROUP BY m", ".thread_id"),
        ] {
            assert!(
                !LIST_DM_THREADS_SQL.contains(marker),
                "thread listing query should not contain global aggregate/rank marker {marker}"
            );
        }

        assert!(LIST_DM_THREADS_SQL.contains("FROM dm_thread_participants pt"));
        assert!(LIST_DM_THREADS_SQL.contains("pt.identity_id = $1"));
        assert!(LIST_DM_THREADS_SQL.contains("pt.last_message_seq <"));
        assert!(LIST_DM_THREADS_SQL.contains("LIMIT $3"));
        assert!(LIST_DM_THREADS_SQL.contains("CROSS JOIN LATERAL"));
    }
}
