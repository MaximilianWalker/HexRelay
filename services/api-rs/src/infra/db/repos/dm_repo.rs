use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Row, Transaction};

use crate::models::{
    DmFanoutDeliveryRecord, DmOutboundForwardRecord, DmPolicy, DmProfileDeviceRecord,
};

const DM_FANOUT_ACK_ADVANCE_WINDOW: i64 = 100;

pub struct DmOutboundForwardWrite<'a> {
    pub sender_identity_id: &'a str,
    pub destination_node_id: &'a str,
    pub message_id: &'a str,
    pub thread_id: &'a str,
    pub recipient_identity_id: &'a str,
    pub ciphertext: &'a str,
    pub source_device_id: Option<&'a str>,
    pub delivery_cursor: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DmDeliveryMetadataRetentionSummary {
    pub fanout_delivery_records_deleted: u64,
    pub outbound_forward_records_deleted: u64,
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

pub async fn upsert_dm_profile_device(
    pool: &PgPool,
    identity_id: &str,
    record: &DmProfileDeviceRecord,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "
        INSERT INTO dm_profile_devices (identity_id, device_id, device_secret_hash, active, last_seen_epoch, updated_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        ON CONFLICT (identity_id, device_id) DO UPDATE
        SET device_secret_hash = EXCLUDED.device_secret_hash,
            active = EXCLUDED.active,
            last_seen_epoch = EXCLUDED.last_seen_epoch,
            updated_at = NOW()
        WHERE dm_profile_devices.device_secret_hash = EXCLUDED.device_secret_hash
           OR dm_profile_devices.device_secret_hash = ''
        ",
    )
    .bind(identity_id)
    .bind(&record.device_id)
    .bind(&record.device_secret_hash)
    .bind(record.active)
    .bind(record.last_seen_epoch)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_dm_profile_devices(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<DmProfileDeviceRecord>, sqlx::Error> {
    let rows = sqlx::query(
        "
        SELECT device_id, device_secret_hash, active, last_seen_epoch
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

pub async fn get_dm_profile_device(
    pool: &PgPool,
    identity_id: &str,
    device_id: &str,
) -> Result<Option<DmProfileDeviceRecord>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT device_id, device_secret_hash, active, last_seen_epoch
        FROM dm_profile_devices
        WHERE identity_id = $1 AND device_id = $2
        ",
    )
    .bind(identity_id)
    .bind(device_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_dm_profile_device_row).transpose()
}

fn map_dm_profile_device_row(
    row: sqlx::postgres::PgRow,
) -> Result<DmProfileDeviceRecord, sqlx::Error> {
    Ok(DmProfileDeviceRecord {
        device_id: row.try_get::<String, _>("device_id")?,
        device_secret_hash: row.try_get::<String, _>("device_secret_hash")?,
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
            delivery_state,
            reachability_state,
            delivered_device_ids,
            created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, NOW())
        ON CONFLICT (identity_id, cursor) DO UPDATE
        SET thread_id = EXCLUDED.thread_id,
            message_id = EXCLUDED.message_id,
            sender_identity_id = EXCLUDED.sender_identity_id,
            ciphertext = EXCLUDED.ciphertext,
            source_device_id = EXCLUDED.source_device_id,
            delivery_state = EXCLUDED.delivery_state,
            reachability_state = EXCLUDED.reachability_state,
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
    .bind(&record.delivery_state)
    .bind(&record.reachability_state)
    .bind(
        serde_json::to_string(&record.delivered_device_ids)
            .map_err(|_| sqlx::Error::Protocol("failed to encode delivered_device_ids".into()))?,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

pub async fn list_dm_fanout_delivery_records_page(
    pool: &PgPool,
    identity_id: &str,
    device_id: &str,
    after_cursor: u64,
    limit: u32,
) -> Result<Vec<DmFanoutDeliveryRecord>, sqlx::Error> {
    let after_cursor = i64::try_from(after_cursor)
        .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?;
    let rows = sqlx::query(
        "
        SELECT cursor, thread_id, message_id, sender_identity_id, ciphertext, source_device_id, delivery_state, reachability_state, delivered_device_ids
        FROM dm_fanout_delivery_log
        WHERE identity_id = $1
          AND cursor > $2
          AND NOT (delivered_device_ids ? $3)
        ORDER BY cursor ASC
        LIMIT $4
        ",
    )
    .bind(identity_id)
    .bind(after_cursor)
    .bind(device_id)
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_dm_fanout_delivery_row).collect()
}

pub async fn get_dm_fanout_delivery_record_by_message(
    pool: &PgPool,
    identity_id: &str,
    message_id: &str,
) -> Result<Option<DmFanoutDeliveryRecord>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT cursor, thread_id, message_id, sender_identity_id, ciphertext, source_device_id, delivery_state, reachability_state, delivered_device_ids
        FROM dm_fanout_delivery_log
        WHERE identity_id = $1
          AND message_id = $2
        ORDER BY cursor ASC
        LIMIT 1
        ",
    )
    .bind(identity_id)
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_dm_fanout_delivery_row).transpose()
}

fn map_dm_fanout_delivery_row(
    row: sqlx::postgres::PgRow,
) -> Result<DmFanoutDeliveryRecord, sqlx::Error> {
    let cursor = row.try_get::<i64, _>("cursor")?;
    let delivered_device_ids_json = row.try_get::<serde_json::Value, _>("delivered_device_ids")?;
    let delivered_device_ids = serde_json::from_value::<Vec<String>>(delivered_device_ids_json)
        .map_err(|_| sqlx::Error::Protocol("invalid delivered_device_ids json".into()))?;
    Ok(DmFanoutDeliveryRecord {
        cursor: u64::try_from(cursor)
            .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))?,
        thread_id: row.try_get::<String, _>("thread_id")?,
        message_id: row.try_get::<String, _>("message_id")?,
        sender_identity_id: row.try_get::<String, _>("sender_identity_id")?,
        ciphertext: row.try_get::<String, _>("ciphertext")?,
        source_device_id: row.try_get::<Option<String>, _>("source_device_id")?,
        delivery_state: row.try_get::<String, _>("delivery_state")?,
        reachability_state: row.try_get::<String, _>("reachability_state")?,
        delivered_device_ids,
    })
}

pub async fn purge_expired_dm_delivery_metadata(
    pool: &PgPool,
    delivery_log_cutoff: DateTime<Utc>,
    outbound_forwarding_cutoff: DateTime<Utc>,
) -> Result<DmDeliveryMetadataRetentionSummary, sqlx::Error> {
    let fanout_deleted = sqlx::query(
        "
        DELETE FROM dm_fanout_delivery_log log
        WHERE log.created_at < $1
          AND (
              NOT EXISTS (
                  SELECT 1
                  FROM dm_profile_devices device
                  WHERE device.identity_id = log.identity_id
              )
              OR NOT EXISTS (
                  SELECT 1
                  FROM dm_profile_devices device
                  LEFT JOIN dm_fanout_device_cursors cursor
                    ON cursor.identity_id = device.identity_id
                   AND cursor.device_id = device.device_id
                  WHERE device.identity_id = log.identity_id
                    AND COALESCE(cursor.cursor, 0) < log.cursor
              )
          )
        ",
    )
    .bind(delivery_log_cutoff)
    .execute(pool)
    .await?
    .rows_affected();

    let outbound_deleted = sqlx::query(
        "
        DELETE FROM dm_outbound_forwarding_log
        WHERE created_at < $1
          AND (
              forwarding_state = 'forwarded'
              OR (forwarding_state = 'failed' AND next_attempt_at IS NULL)
          )
        ",
    )
    .bind(outbound_forwarding_cutoff)
    .execute(pool)
    .await?
    .rows_affected();

    Ok(DmDeliveryMetadataRetentionSummary {
        fanout_delivery_records_deleted: fanout_deleted,
        outbound_forward_records_deleted: outbound_deleted,
    })
}

pub async fn record_dm_outbound_forward_queued(
    pool: &PgPool,
    record: &DmOutboundForwardWrite<'_>,
) -> Result<u32, sqlx::Error> {
    let row = sqlx::query(
        "
        INSERT INTO dm_outbound_forwarding_log (
            sender_identity_id,
            destination_node_id,
            message_id,
            thread_id,
            recipient_identity_id,
            ciphertext,
            source_device_id,
            delivery_cursor,
            forwarding_state,
            attempt_count,
            last_error,
            last_attempt_at,
            next_attempt_at,
            forwarded_at,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'queued', 1, NULL, NOW(), NULL, NULL, NOW(), NOW())
        ON CONFLICT (sender_identity_id, destination_node_id, message_id) DO UPDATE
        SET thread_id = EXCLUDED.thread_id,
            recipient_identity_id = EXCLUDED.recipient_identity_id,
            ciphertext = EXCLUDED.ciphertext,
            source_device_id = EXCLUDED.source_device_id,
            delivery_cursor = EXCLUDED.delivery_cursor,
            forwarding_state = 'queued',
            attempt_count = dm_outbound_forwarding_log.attempt_count + 1,
            last_error = NULL,
            last_attempt_at = NOW(),
            next_attempt_at = NULL,
            forwarded_at = NULL,
            updated_at = NOW()
        RETURNING attempt_count
        ",
    )
    .bind(record.sender_identity_id)
    .bind(record.destination_node_id)
    .bind(record.message_id)
    .bind(record.thread_id)
    .bind(record.recipient_identity_id)
    .bind(record.ciphertext)
    .bind(record.source_device_id)
    .bind(
        i64::try_from(record.delivery_cursor)
            .map_err(|_| sqlx::Error::Protocol("delivery_cursor too large for storage".into()))?,
    )
    .fetch_one(pool)
    .await?;

    let attempt_count = row.try_get::<i32, _>("attempt_count")?;
    u32::try_from(attempt_count)
        .map_err(|_| sqlx::Error::Protocol("attempt_count must be non-negative".into()))
}

pub async fn mark_dm_outbound_forward_succeeded(
    pool: &PgPool,
    sender_identity_id: &str,
    destination_node_id: &str,
    message_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "
        UPDATE dm_outbound_forwarding_log
        SET forwarding_state = 'forwarded',
            last_error = NULL,
            next_attempt_at = NULL,
            forwarded_at = NOW(),
            updated_at = NOW()
        WHERE sender_identity_id = $1
          AND destination_node_id = $2
          AND message_id = $3
        ",
    )
    .bind(sender_identity_id)
    .bind(destination_node_id)
    .bind(message_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn mark_dm_outbound_forward_failed(
    pool: &PgPool,
    sender_identity_id: &str,
    destination_node_id: &str,
    message_id: &str,
    error: &str,
    next_attempt_at: Option<DateTime<Utc>>,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "
        UPDATE dm_outbound_forwarding_log
        SET forwarding_state = 'failed',
            last_error = $4,
            next_attempt_at = $5,
            forwarded_at = NULL,
            updated_at = NOW()
        WHERE sender_identity_id = $1
          AND destination_node_id = $2
          AND message_id = $3
        ",
    )
    .bind(sender_identity_id)
    .bind(destination_node_id)
    .bind(message_id)
    .bind(error)
    .bind(next_attempt_at)
    .execute(pool)
    .await
    .map(|result| result.rows_affected() > 0)
}

pub async fn mark_dm_outbound_forward_retry_started(
    pool: &PgPool,
    sender_identity_id: &str,
    destination_node_id: &str,
    message_id: &str,
    max_attempts: u32,
    stale_after_seconds: i64,
) -> Result<Option<u32>, sqlx::Error> {
    let max_attempts = i32::try_from(max_attempts)
        .map_err(|_| sqlx::Error::Protocol("max_attempts too large for storage".into()))?;
    let row = sqlx::query(
        "
        UPDATE dm_outbound_forwarding_log
        SET forwarding_state = 'queued',
            attempt_count = attempt_count + 1,
            last_error = NULL,
            last_attempt_at = NOW(),
            next_attempt_at = NULL,
            forwarded_at = NULL,
            updated_at = NOW()
        WHERE sender_identity_id = $1
          AND destination_node_id = $2
          AND message_id = $3
          AND attempt_count < $4
          AND (
              (forwarding_state = 'failed'
               AND next_attempt_at IS NOT NULL
               AND next_attempt_at <= NOW())
              OR
              (forwarding_state = 'queued'
               AND (last_attempt_at IS NULL
                    OR last_attempt_at <= NOW() - ($5::DOUBLE PRECISION * INTERVAL '1 second')))
          )
        RETURNING attempt_count
        ",
    )
    .bind(sender_identity_id)
    .bind(destination_node_id)
    .bind(message_id)
    .bind(max_attempts)
    .bind(stale_after_seconds.max(0) as f64)
    .fetch_optional(pool)
    .await?;

    row.map(|row| {
        let attempt_count = row.try_get::<i32, _>("attempt_count")?;
        u32::try_from(attempt_count)
            .map_err(|_| sqlx::Error::Protocol("attempt_count must be non-negative".into()))
    })
    .transpose()
}

pub async fn list_due_dm_outbound_forward_records(
    pool: &PgPool,
    limit: u32,
    max_attempts: u32,
    stale_after_seconds: i64,
) -> Result<Vec<DmOutboundForwardRecord>, sqlx::Error> {
    let limit = i64::from(limit.max(1));
    let max_attempts = i32::try_from(max_attempts)
        .map_err(|_| sqlx::Error::Protocol("max_attempts too large for storage".into()))?;
    let rows = sqlx::query(
        "
        SELECT sender_identity_id,
               destination_node_id,
               message_id,
               thread_id,
               recipient_identity_id,
               ciphertext,
               source_device_id,
               delivery_cursor,
               forwarding_state,
               attempt_count,
               last_error,
               next_attempt_at
        FROM dm_outbound_forwarding_log
        WHERE attempt_count < $2
          AND (
              (forwarding_state = 'failed'
               AND next_attempt_at IS NOT NULL
               AND next_attempt_at <= NOW())
              OR
              (forwarding_state = 'queued'
               AND (last_attempt_at IS NULL
                    OR last_attempt_at <= NOW() - ($3::DOUBLE PRECISION * INTERVAL '1 second')))
          )
        ORDER BY COALESCE(next_attempt_at, last_attempt_at, created_at) ASC,
                 updated_at ASC,
                 sender_identity_id ASC,
                 destination_node_id ASC,
                 message_id ASC
        LIMIT $1
        ",
    )
    .bind(limit)
    .bind(max_attempts)
    .bind(stale_after_seconds.max(0) as f64)
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(map_dm_outbound_forward_row).collect()
}

pub async fn get_dm_outbound_forward_record(
    pool: &PgPool,
    sender_identity_id: &str,
    destination_node_id: &str,
    message_id: &str,
) -> Result<Option<DmOutboundForwardRecord>, sqlx::Error> {
    let row = sqlx::query(
        "
        SELECT sender_identity_id,
               destination_node_id,
               message_id,
               thread_id,
               recipient_identity_id,
               ciphertext,
               source_device_id,
               delivery_cursor,
               forwarding_state,
               attempt_count,
               last_error,
               next_attempt_at
        FROM dm_outbound_forwarding_log
        WHERE sender_identity_id = $1
          AND destination_node_id = $2
          AND message_id = $3
        ",
    )
    .bind(sender_identity_id)
    .bind(destination_node_id)
    .bind(message_id)
    .fetch_optional(pool)
    .await?;

    row.map(map_dm_outbound_forward_row).transpose()
}

pub async fn ack_dm_fanout_delivery_device(
    pool: &PgPool,
    recipient_identity_id: &str,
    thread_id: &str,
    message_id: &str,
    device_id: &str,
    cursor: u64,
) -> Result<bool, sqlx::Error> {
    let cursor_i64 = i64::try_from(cursor)
        .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?;
    let mut tx = pool.begin().await?;

    let device_exists = sqlx::query_scalar::<_, i64>(
        "
        SELECT 1::BIGINT
        FROM dm_profile_devices
        WHERE identity_id = $1 AND device_id = $2
        ",
    )
    .bind(recipient_identity_id)
    .bind(device_id)
    .fetch_optional(&mut *tx)
    .await?
    .is_some();
    if !device_exists {
        tx.rollback().await?;
        return Ok(false);
    }

    let row = sqlx::query(
        "
        SELECT delivered_device_ids
        FROM dm_fanout_delivery_log
        WHERE identity_id = $1
          AND thread_id = $2
          AND message_id = $3
          AND cursor = $4
        FOR UPDATE
        ",
    )
    .bind(recipient_identity_id)
    .bind(thread_id)
    .bind(message_id)
    .bind(cursor_i64)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.rollback().await?;
        return Ok(false);
    };

    let delivered_device_ids_json = row.try_get::<serde_json::Value, _>("delivered_device_ids")?;
    let mut delivered_device_ids = serde_json::from_value::<Vec<String>>(delivered_device_ids_json)
        .map_err(|_| sqlx::Error::Protocol("invalid delivered_device_ids json".into()))?;
    if !delivered_device_ids.iter().any(|value| value == device_id) {
        delivered_device_ids.push(device_id.to_string());
        delivered_device_ids.sort();
        delivered_device_ids.dedup();
    }

    let delivered_device_ids_json = serde_json::to_string(&delivered_device_ids)
        .map_err(|_| sqlx::Error::Protocol("failed to encode delivered_device_ids".into()))?;
    sqlx::query(
        "
        UPDATE dm_fanout_delivery_log
        SET delivered_device_ids = $5::jsonb,
            delivery_state = 'acked',
            reachability_state = 'reachable'
        WHERE identity_id = $1
          AND thread_id = $2
          AND message_id = $3
          AND cursor = $4
        ",
    )
    .bind(recipient_identity_id)
    .bind(thread_id)
    .bind(message_id)
    .bind(cursor_i64)
    .bind(delivered_device_ids_json)
    .execute(&mut *tx)
    .await?;

    let current_cursor = sqlx::query(
        "
        SELECT cursor
        FROM dm_fanout_device_cursors
        WHERE identity_id = $1 AND device_id = $2
        FOR UPDATE
        ",
    )
    .bind(recipient_identity_id)
    .bind(device_id)
    .fetch_optional(&mut *tx)
    .await?
    .map(|row| row.try_get::<i64, _>("cursor"))
    .transpose()?
    .map(u64::try_from)
    .transpose()
    .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))?
    .unwrap_or(0);

    let rows = sqlx::query(
        "
        SELECT cursor, delivered_device_ids
        FROM dm_fanout_delivery_log
        WHERE identity_id = $1 AND cursor > $2
        ORDER BY cursor ASC
        LIMIT $3
        FOR UPDATE
        ",
    )
    .bind(recipient_identity_id)
    .bind(
        i64::try_from(current_cursor)
            .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?,
    )
    .bind(DM_FANOUT_ACK_ADVANCE_WINDOW)
    .fetch_all(&mut *tx)
    .await?;

    let mut contiguous_cursor = current_cursor;
    for row in rows {
        let row_cursor = row.try_get::<i64, _>("cursor")?;
        let row_cursor = u64::try_from(row_cursor)
            .map_err(|_| sqlx::Error::Protocol("cursor must be non-negative".into()))?;
        if row_cursor != contiguous_cursor + 1 {
            break;
        }
        let row_device_ids_json = row.try_get::<serde_json::Value, _>("delivered_device_ids")?;
        let row_device_ids = serde_json::from_value::<Vec<String>>(row_device_ids_json)
            .map_err(|_| sqlx::Error::Protocol("invalid delivered_device_ids json".into()))?;
        if !row_device_ids.iter().any(|value| value == device_id) {
            break;
        }
        contiguous_cursor = row_cursor;
    }

    if contiguous_cursor > current_cursor {
        sqlx::query(
            "
            INSERT INTO dm_fanout_device_cursors (identity_id, device_id, cursor, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (identity_id, device_id) DO UPDATE
            SET cursor = EXCLUDED.cursor,
                updated_at = NOW()
            ",
        )
        .bind(recipient_identity_id)
        .bind(device_id)
        .bind(
            i64::try_from(contiguous_cursor)
                .map_err(|_| sqlx::Error::Protocol("cursor too large for storage".into()))?,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(true)
}

fn map_dm_outbound_forward_row(
    row: sqlx::postgres::PgRow,
) -> Result<DmOutboundForwardRecord, sqlx::Error> {
    let delivery_cursor = row.try_get::<i64, _>("delivery_cursor")?;
    let attempt_count = row.try_get::<i32, _>("attempt_count")?;

    Ok(DmOutboundForwardRecord {
        sender_identity_id: row.try_get::<String, _>("sender_identity_id")?,
        destination_node_id: row.try_get::<String, _>("destination_node_id")?,
        message_id: row.try_get::<String, _>("message_id")?,
        thread_id: row.try_get::<String, _>("thread_id")?,
        recipient_identity_id: row.try_get::<String, _>("recipient_identity_id")?,
        ciphertext: row.try_get::<String, _>("ciphertext")?,
        source_device_id: row.try_get::<Option<String>, _>("source_device_id")?,
        delivery_cursor: u64::try_from(delivery_cursor)
            .map_err(|_| sqlx::Error::Protocol("delivery_cursor must be positive".into()))?,
        forwarding_state: row.try_get::<String, _>("forwarding_state")?,
        attempt_count: u32::try_from(attempt_count)
            .map_err(|_| sqlx::Error::Protocol("attempt_count must be non-negative".into()))?,
        last_error: row.try_get::<Option<String>, _>("last_error")?,
        next_attempt_at: row.try_get::<Option<DateTime<Utc>>, _>("next_attempt_at")?,
    })
}
