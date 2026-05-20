use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::models::{BlockRecord, ContactSummary, MuteRecord};

pub struct ContactPreferenceUpdateParams<'a> {
    pub owner_identity_id: &'a str,
    pub contact_identity_id: &'a str,
    pub pinned: Option<bool>,
    pub muted: Option<bool>,
}

pub async fn list_contacts_for_identity(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<ContactSummary>, sqlx::Error> {
    let rows = sqlx::query(
        "
        WITH contact_rows AS (
            SELECT
                CASE
                    WHEN requester_identity_id = $1 THEN target_identity_id
                    ELSE requester_identity_id
                END AS contact_identity_id,
                status,
                requester_identity_id = $1 AS requester_is_self,
                created_at
            FROM friend_requests
            WHERE (requester_identity_id = $1 OR target_identity_id = $1)
              AND status IN ('accepted', 'pending')
        ),
        ranked_contacts AS (
            SELECT DISTINCT ON (contact_identity_id)
                contact_identity_id,
                status,
                requester_is_self,
                created_at
            FROM contact_rows
            ORDER BY contact_identity_id,
                CASE status WHEN 'accepted' THEN 2 WHEN 'pending' THEN 1 ELSE 0 END DESC,
                created_at DESC
        )
        SELECT
            c.contact_identity_id,
            c.status,
            c.requester_is_self,
            COALESCE(p.pinned, FALSE) AS pinned,
            COALESCE(p.muted, FALSE) OR (um.muted_identity_id IS NOT NULL) AS muted
        FROM ranked_contacts c
        LEFT JOIN contact_preferences p
          ON p.owner_identity_id = $1
         AND p.contact_identity_id = c.contact_identity_id
        LEFT JOIN user_mutes um
          ON um.muter_identity_id = $1
         AND um.muted_identity_id = c.contact_identity_id
        WHERE NOT EXISTS (
            SELECT 1
            FROM user_blocks ub
            WHERE (ub.blocker_identity_id = $1 AND ub.blocked_identity_id = c.contact_identity_id)
               OR (ub.blocker_identity_id = c.contact_identity_id AND ub.blocked_identity_id = $1)
        )
        ORDER BY pinned DESC, c.created_at DESC, c.contact_identity_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let id = row.try_get::<String, _>("contact_identity_id")?;
            let status = row.try_get::<String, _>("status")?;
            let requester_is_self = row.try_get::<bool, _>("requester_is_self")?;
            Ok(ContactSummary {
                id: id.clone(),
                name: id,
                status: "offline".to_string(),
                unread: 0,
                pinned: row.try_get::<bool, _>("pinned")?,
                muted: row.try_get::<bool, _>("muted")?,
                inbound_request: status == "pending" && !requester_is_self,
                pending_request: status == "pending" && requester_is_self,
            })
        })
        .collect()
}

pub async fn upsert_contact_preferences(
    pool: &PgPool,
    params: ContactPreferenceUpdateParams<'_>,
) -> Result<Option<ContactSummary>, sqlx::Error> {
    let exists = sqlx::query_scalar::<_, bool>(
        "
        SELECT EXISTS (
            SELECT 1
            FROM friend_requests
            WHERE status IN ('accepted', 'pending')
              AND (
                (requester_identity_id = $1 AND target_identity_id = $2)
                OR (requester_identity_id = $2 AND target_identity_id = $1)
              )
        )
        ",
    )
    .bind(params.owner_identity_id)
    .bind(params.contact_identity_id)
    .fetch_one(pool)
    .await?;

    if !exists {
        return Ok(None);
    }

    let mut tx = pool.begin().await?;
    upsert_contact_preferences_in_tx(&mut tx, &params).await?;

    if let Some(muted) = params.muted {
        if muted {
            upsert_user_mute_in_tx(
                &mut tx,
                params.owner_identity_id,
                params.contact_identity_id,
            )
            .await?;
        } else {
            delete_user_mute_in_tx(
                &mut tx,
                params.owner_identity_id,
                params.contact_identity_id,
            )
            .await?;
        }
    }

    tx.commit().await?;

    let contacts = list_contacts_for_identity(pool, params.owner_identity_id).await?;
    Ok(contacts
        .into_iter()
        .find(|item| item.id == params.contact_identity_id))
}

pub async fn block_and_remove_contact(
    pool: &PgPool,
    blocker_identity_id: &str,
    blocked_identity_id: &str,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    upsert_user_block_in_tx(&mut tx, blocker_identity_id, blocked_identity_id).await?;
    delete_contact_preferences_in_tx(&mut tx, blocker_identity_id, blocked_identity_id).await?;

    let result = sqlx::query(
        "
        DELETE FROM friend_requests
        WHERE status IN ('accepted', 'pending')
          AND (
            (requester_identity_id = $1 AND target_identity_id = $2)
            OR (requester_identity_id = $2 AND target_identity_id = $1)
          )
        ",
    )
    .bind(blocker_identity_id)
    .bind(blocked_identity_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}

pub async fn upsert_user_block(
    pool: &PgPool,
    blocker_identity_id: &str,
    blocked_identity_id: &str,
) -> Result<(BlockRecord, bool), sqlx::Error> {
    let row = sqlx::query(
        "
        INSERT INTO user_blocks (blocker_identity_id, blocked_identity_id)
        VALUES ($1, $2)
        ON CONFLICT (blocker_identity_id, blocked_identity_id) DO NOTHING
        RETURNING blocker_identity_id, blocked_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        ",
    )
    .bind(blocker_identity_id)
    .bind(blocked_identity_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok((map_block_record(row)?, true));
    }

    let existing = sqlx::query(
        "
        SELECT blocker_identity_id, blocked_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        FROM user_blocks
        WHERE blocker_identity_id = $1 AND blocked_identity_id = $2
        ",
    )
    .bind(blocker_identity_id)
    .bind(blocked_identity_id)
    .fetch_one(pool)
    .await?;

    Ok((map_block_record(existing)?, false))
}

pub async fn delete_user_block(
    pool: &PgPool,
    blocker_identity_id: &str,
    blocked_identity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        DELETE FROM user_blocks
        WHERE blocker_identity_id = $1 AND blocked_identity_id = $2
        ",
    )
    .bind(blocker_identity_id)
    .bind(blocked_identity_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_user_blocks(
    pool: &PgPool,
    blocker_identity_id: &str,
) -> Result<Vec<BlockRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT blocker_identity_id, blocked_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        FROM user_blocks
        WHERE blocker_identity_id = $1
        ORDER BY blocked_identity_id ASC
        ",
    )
    .bind(blocker_identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_block_record).collect()
}

pub async fn is_blocked_bidirectional(
    pool: &PgPool,
    first_identity_id: &str,
    second_identity_id: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "
        SELECT EXISTS (
            SELECT 1
            FROM user_blocks
            WHERE (blocker_identity_id = $1 AND blocked_identity_id = $2)
               OR (blocker_identity_id = $2 AND blocked_identity_id = $1)
        )
        ",
    )
    .bind(first_identity_id)
    .bind(second_identity_id)
    .fetch_one(pool)
    .await
}

pub async fn blocked_peers_for_identity(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "
        SELECT blocked_identity_id
        FROM user_blocks
        WHERE blocker_identity_id = $1
        UNION
        SELECT blocker_identity_id
        FROM user_blocks
        WHERE blocked_identity_id = $1
        ORDER BY 1 ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await
}

pub async fn upsert_user_mute(
    pool: &PgPool,
    muter_identity_id: &str,
    muted_identity_id: &str,
) -> Result<(MuteRecord, bool), sqlx::Error> {
    let row = sqlx::query(
        "
        INSERT INTO user_mutes (muter_identity_id, muted_identity_id)
        VALUES ($1, $2)
        ON CONFLICT (muter_identity_id, muted_identity_id) DO NOTHING
        RETURNING muter_identity_id, muted_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        ",
    )
    .bind(muter_identity_id)
    .bind(muted_identity_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok((map_mute_record(row)?, true));
    }

    let existing = sqlx::query(
        "
        SELECT muter_identity_id, muted_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        FROM user_mutes
        WHERE muter_identity_id = $1 AND muted_identity_id = $2
        ",
    )
    .bind(muter_identity_id)
    .bind(muted_identity_id)
    .fetch_one(pool)
    .await?;

    Ok((map_mute_record(existing)?, false))
}

pub async fn delete_user_mute(
    pool: &PgPool,
    muter_identity_id: &str,
    muted_identity_id: &str,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    delete_user_mute_in_tx(&mut tx, muter_identity_id, muted_identity_id).await?;
    sqlx::query(
        "
        UPDATE contact_preferences
        SET muted = FALSE,
            updated_at = NOW()
        WHERE owner_identity_id = $1
          AND contact_identity_id = $2
        ",
    )
    .bind(muter_identity_id)
    .bind(muted_identity_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn list_user_mutes(
    pool: &PgPool,
    muter_identity_id: &str,
) -> Result<Vec<MuteRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT muter_identity_id, muted_identity_id,
            TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
        FROM user_mutes
        WHERE muter_identity_id = $1
        ORDER BY muted_identity_id ASC
        ",
    )
    .bind(muter_identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_mute_record).collect()
}

async fn upsert_contact_preferences_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    params: &ContactPreferenceUpdateParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO contact_preferences (
            owner_identity_id,
            contact_identity_id,
            pinned,
            muted,
            updated_at
        )
        VALUES ($1, $2, COALESCE($3, FALSE), COALESCE($4, FALSE), NOW())
        ON CONFLICT (owner_identity_id, contact_identity_id) DO UPDATE
        SET pinned = COALESCE($3, contact_preferences.pinned),
            muted = COALESCE($4, contact_preferences.muted),
            updated_at = NOW()
        ",
    )
    .bind(params.owner_identity_id)
    .bind(params.contact_identity_id)
    .bind(params.pinned)
    .bind(params.muted)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn delete_contact_preferences_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    owner_identity_id: &str,
    contact_identity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        DELETE FROM contact_preferences
        WHERE owner_identity_id = $1
          AND contact_identity_id = $2
        ",
    )
    .bind(owner_identity_id)
    .bind(contact_identity_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn upsert_user_block_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    blocker_identity_id: &str,
    blocked_identity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO user_blocks (blocker_identity_id, blocked_identity_id)
        VALUES ($1, $2)
        ON CONFLICT (blocker_identity_id, blocked_identity_id) DO NOTHING
        ",
    )
    .bind(blocker_identity_id)
    .bind(blocked_identity_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn upsert_user_mute_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    muter_identity_id: &str,
    muted_identity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO user_mutes (muter_identity_id, muted_identity_id)
        VALUES ($1, $2)
        ON CONFLICT (muter_identity_id, muted_identity_id) DO NOTHING
        ",
    )
    .bind(muter_identity_id)
    .bind(muted_identity_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn delete_user_mute_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    muter_identity_id: &str,
    muted_identity_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        DELETE FROM user_mutes
        WHERE muter_identity_id = $1
          AND muted_identity_id = $2
        ",
    )
    .bind(muter_identity_id)
    .bind(muted_identity_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn map_block_record(row: sqlx::postgres::PgRow) -> Result<BlockRecord, sqlx::Error> {
    Ok(BlockRecord {
        blocker_identity_id: row.try_get::<String, _>("blocker_identity_id")?,
        blocked_identity_id: row.try_get::<String, _>("blocked_identity_id")?,
        created_at: row.try_get::<String, _>("created_at")?,
    })
}

fn map_mute_record(row: sqlx::postgres::PgRow) -> Result<MuteRecord, sqlx::Error> {
    Ok(MuteRecord {
        muter_identity_id: row.try_get::<String, _>("muter_identity_id")?,
        muted_identity_id: row.try_get::<String, _>("muted_identity_id")?,
        created_at: row.try_get::<String, _>("created_at")?,
    })
}
