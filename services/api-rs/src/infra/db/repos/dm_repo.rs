use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::models::{
    DmEndpointCardRecord, DmFanoutDeliveryRecord, DmPolicy, DmProfileDeviceRecord,
};

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

pub async fn get_dm_policy(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Option<DmPolicy>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT inbound_policy, offline_delivery_mode
        FROM dm_policies
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_dm_policy_row).transpose()
}

pub async fn upsert_dm_policy(
    pool: &PgPool,
    identity_id: &str,
    policy: &DmPolicy,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_policies (identity_id, inbound_policy, offline_delivery_mode, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (identity_id) DO UPDATE
        SET inbound_policy = EXCLUDED.inbound_policy,
            offline_delivery_mode = EXCLUDED.offline_delivery_mode,
            updated_at = NOW()
        ",
    )
    .bind(identity_id)
    .bind(&policy.inbound_policy)
    .bind(&policy.offline_delivery_mode)
    .execute(pool)
    .await?;

    Ok(())
}

fn map_dm_policy_row(row: sqlx::postgres::PgRow) -> Result<DmPolicy, sqlx::Error> {
    Ok(DmPolicy {
        inbound_policy: row.try_get::<String, _>("inbound_policy")?,
        offline_delivery_mode: row.try_get::<String, _>("offline_delivery_mode")?,
    })
}

pub async fn upsert_dm_endpoint_cards_batch(
    pool: &PgPool,
    identity_id: &str,
    records: &[DmEndpointCardRecord],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for record in records {
        sqlx::query(
            "
            INSERT INTO dm_endpoint_cards (
                identity_id,
                endpoint_id,
                endpoint_hint,
                estimated_rtt_ms,
                priority,
                expires_at_epoch,
                revoked,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (identity_id, endpoint_id) DO UPDATE
            SET endpoint_hint = EXCLUDED.endpoint_hint,
                estimated_rtt_ms = EXCLUDED.estimated_rtt_ms,
                priority = EXCLUDED.priority,
                expires_at_epoch = EXCLUDED.expires_at_epoch,
                revoked = EXCLUDED.revoked,
                updated_at = NOW()
            ",
        )
        .bind(identity_id)
        .bind(&record.endpoint_id)
        .bind(&record.endpoint_hint)
        .bind(
            i32::try_from(record.estimated_rtt_ms)
                .map_err(|_| sqlx::Error::Protocol("estimated_rtt_ms too large".into()))?,
        )
        .bind(i16::from(record.priority))
        .bind(record.expires_at_epoch)
        .bind(record.revoked)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_dm_endpoint_cards(
    pool: &PgPool,
    identity_id: &str,
    now_epoch: i64,
) -> Result<Vec<DmEndpointCardRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT endpoint_id, endpoint_hint, estimated_rtt_ms, priority, expires_at_epoch, revoked
        FROM dm_endpoint_cards
        WHERE identity_id = $1
          AND expires_at_epoch >= $2
        ORDER BY endpoint_id ASC
        ",
    )
    .bind(identity_id)
    .bind(now_epoch)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_dm_endpoint_card_row).collect()
}

pub async fn mark_dm_endpoint_cards_revoked(
    pool: &PgPool,
    identity_id: &str,
    endpoint_ids: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    if endpoint_ids.is_empty() {
        return Ok(Vec::new());
    }

    let revoked_rows = sqlx::query(
        "
        UPDATE dm_endpoint_cards
        SET revoked = TRUE,
            updated_at = NOW()
        WHERE identity_id = $1
          AND endpoint_id = ANY($2)
          AND revoked = FALSE
        RETURNING endpoint_id
        ",
    )
    .bind(identity_id)
    .bind(endpoint_ids)
    .fetch_all(pool)
    .await?;

    let mut revoked_lookup = revoked_rows
        .into_iter()
        .map(|row| row.try_get::<String, _>("endpoint_id"))
        .collect::<Result<std::collections::HashSet<_>, _>>()?;

    Ok(endpoint_ids
        .iter()
        .filter(|endpoint_id| revoked_lookup.remove(endpoint_id.as_str()))
        .cloned()
        .collect())
}

pub async fn upsert_dm_profile_device(
    pool: &PgPool,
    identity_id: &str,
    record: &DmProfileDeviceRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_profile_devices (identity_id, device_id, active, last_seen_epoch, updated_at)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (identity_id, device_id) DO UPDATE
        SET active = EXCLUDED.active,
            last_seen_epoch = EXCLUDED.last_seen_epoch,
            updated_at = NOW()
        ",
    )
    .bind(identity_id)
    .bind(&record.device_id)
    .bind(record.active)
    .bind(record.last_seen_epoch)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_dm_profile_devices(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DmProfileDeviceRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT device_id, active, last_seen_epoch
        FROM dm_profile_devices
        WHERE identity_id = $1
        ORDER BY device_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_dm_profile_device_row).collect()
}

fn map_dm_endpoint_card_row(
    row: sqlx::postgres::PgRow,
) -> Result<DmEndpointCardRecord, sqlx::Error> {
    let estimated_rtt_ms = row.try_get::<i32, _>("estimated_rtt_ms")?;
    let priority = row.try_get::<i16, _>("priority")?;

    Ok(DmEndpointCardRecord {
        endpoint_id: row.try_get::<String, _>("endpoint_id")?,
        endpoint_hint: row.try_get::<String, _>("endpoint_hint")?,
        estimated_rtt_ms: u32::try_from(estimated_rtt_ms)
            .map_err(|_| sqlx::Error::Protocol("estimated_rtt_ms must be non-negative".into()))?,
        priority: u8::try_from(priority)
            .map_err(|_| sqlx::Error::Protocol("priority must be in u8 range".into()))?,
        expires_at_epoch: row.try_get::<i64, _>("expires_at_epoch")?,
        revoked: row.try_get::<bool, _>("revoked")?,
    })
}

fn map_dm_profile_device_row(
    row: sqlx::postgres::PgRow,
) -> Result<DmProfileDeviceRecord, sqlx::Error> {
    Ok(DmProfileDeviceRecord {
        device_id: row.try_get::<String, _>("device_id")?,
        active: row.try_get::<bool, _>("active")?,
        last_seen_epoch: row.try_get::<i64, _>("last_seen_epoch")?,
    })
}

pub async fn get_dm_fanout_stream_head(
    pool: &PgPool,
    identity_id: &str,
) -> Result<u64, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT latest_cursor
        FROM dm_fanout_stream_heads
        WHERE identity_id = $1
        ",
    )
    .bind(identity_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(0);
    };

    let latest_cursor = row.try_get::<i64, _>("latest_cursor")?;
    u64::try_from(latest_cursor)
        .map_err(|_| sqlx::Error::Protocol("latest_cursor must be non-negative".into()))
}

pub async fn advance_dm_fanout_stream_head(
    pool: &PgPool,
    identity_id: &str,
) -> Result<u64, sqlx::Error> {
    let row = sqlx::query(
        "
        INSERT INTO dm_fanout_stream_heads (identity_id, latest_cursor, updated_at)
        VALUES ($1, 1, NOW())
        ON CONFLICT (identity_id) DO UPDATE
        SET latest_cursor = dm_fanout_stream_heads.latest_cursor + 1,
            updated_at = NOW()
        RETURNING latest_cursor
        ",
    )
    .bind(identity_id)
    .fetch_one(pool)
    .await?;

    let latest_cursor = row.try_get::<i64, _>("latest_cursor")?;
    u64::try_from(latest_cursor)
        .map_err(|_| sqlx::Error::Protocol("latest_cursor must be non-negative".into()))
}

pub async fn advance_dm_fanout_stream_head_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    identity_id: &str,
) -> Result<u64, sqlx::Error> {
    let row = sqlx::query(
        "
        INSERT INTO dm_fanout_stream_heads (identity_id, latest_cursor, updated_at)
        VALUES ($1, 1, NOW())
        ON CONFLICT (identity_id) DO UPDATE
        SET latest_cursor = dm_fanout_stream_heads.latest_cursor + 1,
            updated_at = NOW()
        RETURNING latest_cursor
        ",
    )
    .bind(identity_id)
    .fetch_one(&mut **tx)
    .await?;

    let latest_cursor = row.try_get::<i64, _>("latest_cursor")?;
    u64::try_from(latest_cursor)
        .map_err(|_| sqlx::Error::Protocol("latest_cursor must be non-negative".into()))
}

pub async fn get_dm_fanout_device_cursor(
    pool: &PgPool,
    identity_id: &str,
    device_id: &str,
) -> Result<u64, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT cursor
        FROM dm_fanout_device_cursors
        WHERE identity_id = $1 AND device_id = $2
        ",
    )
    .bind(identity_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(0);
    };

    let cursor = row.try_get::<i64, _>("cursor")?;
    u64::try_from(cursor).map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))
}

pub async fn upsert_dm_fanout_device_cursor(
    pool: &PgPool,
    identity_id: &str,
    device_id: &str,
    cursor: u64,
) -> Result<u64, sqlx::Error> {
    let cursor_i64 = i64::try_from(cursor)
        .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?;

    let row = sqlx::query(
        "
        INSERT INTO dm_fanout_device_cursors (identity_id, device_id, cursor, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (identity_id, device_id) DO UPDATE
        SET cursor = GREATEST(dm_fanout_device_cursors.cursor, EXCLUDED.cursor),
            updated_at = NOW()
        RETURNING cursor
        ",
    )
    .bind(identity_id)
    .bind(device_id)
    .bind(cursor_i64)
    .fetch_one(pool)
    .await?;

    let stored_cursor = row.try_get::<i64, _>("cursor")?;
    u64::try_from(stored_cursor)
        .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))
}

pub async fn list_dm_fanout_device_cursors(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<(String, u64)>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT device_id, cursor
        FROM dm_fanout_device_cursors
        WHERE identity_id = $1
        ORDER BY device_id ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let cursor = row.try_get::<i64, _>("cursor")?;
            Ok((
                row.try_get::<String, _>("device_id")?,
                u64::try_from(cursor)
                    .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))?,
            ))
        })
        .collect()
}

pub async fn append_dm_fanout_delivery_record_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    identity_id: &str,
    thread_id: &str,
    record: &DmFanoutDeliveryRecord,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
        INSERT INTO dm_fanout_delivery_log (
            identity_id,
            cursor,
            thread_id,
            message_id,
            sender_identity_id,
            ciphertext,
            source_device_id,
            delivered_device_ids,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, NOW())
        ON CONFLICT (identity_id, cursor) DO UPDATE
        SET thread_id = EXCLUDED.thread_id,
            message_id = EXCLUDED.message_id,
            sender_identity_id = EXCLUDED.sender_identity_id,
            ciphertext = EXCLUDED.ciphertext,
            source_device_id = EXCLUDED.source_device_id,
            delivered_device_ids = EXCLUDED.delivered_device_ids,
            created_at = NOW()
        ",
    )
    .bind(identity_id)
    .bind(
        i64::try_from(record.cursor)
            .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?,
    )
    .bind(thread_id)
    .bind(&record.message_id)
    .bind(&record.sender_identity_id)
    .bind(&record.ciphertext)
    .bind(&record.source_device_id)
    .bind(
        serde_json::to_string(&record.delivered_device_ids)
            .map_err(|_| sqlx::Error::Protocol("failed to encode delivered_device_ids".into()))?,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn list_dm_fanout_delivery_records(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DmFanoutDeliveryRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT cursor, message_id, sender_identity_id, ciphertext, source_device_id, delivered_device_ids
        FROM dm_fanout_delivery_log
        WHERE identity_id = $1
        ORDER BY cursor ASC
        ",
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            let cursor = row.try_get::<i64, _>("cursor")?;
            let delivered_device_ids_json =
                row.try_get::<serde_json::Value, _>("delivered_device_ids")?;
            let delivered_device_ids =
                serde_json::from_value::<Vec<String>>(delivered_device_ids_json).map_err(|_| {
                    sqlx::Error::Protocol("invalid delivered_device_ids json".into())
                })?;
            Ok(DmFanoutDeliveryRecord {
                cursor: u64::try_from(cursor)
                    .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))?,
                message_id: row.try_get::<String, _>("message_id")?,
                sender_identity_id: row.try_get::<String, _>("sender_identity_id")?,
                ciphertext: row.try_get::<String, _>("ciphertext")?,
                source_device_id: row.try_get::<Option<String>, _>("source_device_id")?,
                delivered_device_ids,
            })
        })
        .collect()
}
