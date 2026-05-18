use chrono::{SecondsFormat, Utc};
use sqlx::{PgPool, Row};

use crate::models::{
    AccountDataDmMessageExport, AccountDataDmThreadExport, AccountDataDmThreadParticipantExport,
    AccountDataExportPackage, AccountDataFriendRequestExport, AccountDataIdentityExport,
    AccountDataProfileDeviceExport, AccountDataRetentionExport,
    AccountDataServerChannelMessageExport, AccountDataServerMembershipExport,
    AccountDataSessionExport,
};

const ACCOUNT_DATA_EXPORT_KIND: &str = "hexrelay.account_data_export";

pub async fn export_account_data(
    pool: &PgPool,
    identity_id: &str,
    current_session_expires_at: &str,
) -> Result<AccountDataExportPackage, sqlx::Error> {
    Ok(AccountDataExportPackage {
        kind: ACCOUNT_DATA_EXPORT_KIND.to_string(),
        generated_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        identity: load_identity(pool, identity_id).await?,
        sessions: AccountDataSessionExport {
            active_count: load_active_session_count(pool, identity_id).await?,
            current_session_expires_at: current_session_expires_at.to_string(),
        },
        contacts: load_contacts(pool, identity_id).await?,
        servers: load_servers(pool, identity_id).await?,
        dm_profile_devices: load_dm_profile_devices(pool, identity_id).await?,
        dm_threads: load_dm_threads(pool, identity_id).await?,
        dm_messages: load_dm_messages(pool, identity_id).await?,
        server_channel_messages: load_server_channel_messages(pool, identity_id).await?,
        retention: AccountDataRetentionExport {
            sessions: "Exports include active session counts and current session expiry only; session ids and tokens stay local to the runtime and are not imported.".to_string(),
            dm_history: "Exports include canonical encrypted DM history for threads where the identity is a participant; decrypted message bodies and client-held key material remain device-only.".to_string(),
            dm_delivery_metadata: "Bounded delivery, fanout, forwarding, retry, and device-secret metadata are not exported as portable account data.".to_string(),
            server_channel_messages: "Exports include messages authored by the identity, including tombstone state; server retention policy remains node-owned.".to_string(),
        },
        limitations: vec![
            "Mutating import is not enabled by this runtime surface; POST /account/import performs dry-run validation only.".to_string(),
            "Private keys, recovery material, session tokens, DM device secrets, endpoint hints, and LAN/WAN transport metadata are intentionally excluded.".to_string(),
        ],
    })
}

async fn load_identity(
    pool: &PgPool,
    identity_id: &str,
) -> Result<AccountDataIdentityExport, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT identity_id,
               public_key,
               algorithm,
               TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM identity_keys
        WHERE identity_id = $1
        "#,
    )
    .bind(identity_id)
    .fetch_one(pool)
    .await?;

    Ok(AccountDataIdentityExport {
        identity_id: row.try_get::<String, _>("identity_id")?,
        public_key: row.try_get::<String, _>("public_key")?,
        algorithm: row.try_get::<String, _>("algorithm")?,
        created_at: row.try_get::<String, _>("created_at")?,
    })
}

async fn load_active_session_count(pool: &PgPool, identity_id: &str) -> Result<u32, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>(
        "
        SELECT COUNT(*)
        FROM sessions
        WHERE identity_id = $1
          AND revoked_at IS NULL
          AND expires_at > NOW()
        ",
    )
    .bind(identity_id)
    .fetch_one(pool)
    .await?;

    to_u32(count, "active session count")
}

async fn load_contacts(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataFriendRequestExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT request_id,
               requester_identity_id,
               target_identity_id,
               status,
               TO_CHAR(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at
        FROM friend_requests
        WHERE requester_identity_id = $1 OR target_identity_id = $1
        ORDER BY created_at ASC, request_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AccountDataFriendRequestExport {
                request_id: row.try_get::<String, _>("request_id")?,
                requester_identity_id: row.try_get::<String, _>("requester_identity_id")?,
                target_identity_id: row.try_get::<String, _>("target_identity_id")?,
                status: row.try_get::<String, _>("status")?,
                created_at: row.try_get::<String, _>("created_at")?,
            })
        })
        .collect()
}

async fn load_servers(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataServerMembershipExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT s.server_id,
               s.name,
               m.favorite,
               m.muted,
               m.unread_count,
               TO_CHAR(m.joined_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS joined_at
        FROM server_memberships m
        INNER JOIN servers s ON s.server_id = m.server_id
        WHERE m.identity_id = $1
        ORDER BY m.joined_at ASC, s.server_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AccountDataServerMembershipExport {
                server_id: row.try_get::<String, _>("server_id")?,
                name: row.try_get::<String, _>("name")?,
                favorite: row.try_get::<bool, _>("favorite")?,
                muted: row.try_get::<bool, _>("muted")?,
                unread_count: to_u32(row.try_get::<i32, _>("unread_count")?, "unread count")?,
                joined_at: row.try_get::<String, _>("joined_at")?,
            })
        })
        .collect()
}

async fn load_dm_profile_devices(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataProfileDeviceExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT device.device_id,
               device.active,
               device.last_seen_epoch,
               COALESCE(cursor.cursor, 0) AS delivery_cursor,
               TO_CHAR(device.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS updated_at
        FROM dm_profile_devices device
        LEFT JOIN dm_fanout_device_cursors cursor
          ON cursor.identity_id = device.identity_id
         AND cursor.device_id = device.device_id
        WHERE device.identity_id = $1
        ORDER BY device.device_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AccountDataProfileDeviceExport {
                device_id: row.try_get::<String, _>("device_id")?,
                active: row.try_get::<bool, _>("active")?,
                last_seen_epoch: row.try_get::<i64, _>("last_seen_epoch")?,
                delivery_cursor: to_u64(
                    row.try_get::<i64, _>("delivery_cursor")?,
                    "delivery cursor",
                )?,
                updated_at: row.try_get::<String, _>("updated_at")?,
            })
        })
        .collect()
}

async fn load_dm_threads(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataDmThreadExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT thread.thread_id,
               thread.kind,
               thread.title,
               TO_CHAR(thread.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
               participant.identity_id,
               participant.last_read_seq
        FROM dm_thread_participants self
        INNER JOIN dm_threads thread ON thread.thread_id = self.thread_id
        INNER JOIN dm_thread_participants participant ON participant.thread_id = thread.thread_id
        WHERE self.identity_id = $1
        ORDER BY thread.created_at ASC, thread.thread_id ASC, participant.identity_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    let mut threads = Vec::<AccountDataDmThreadExport>::new();
    for row in rows {
        let thread_id = row.try_get::<String, _>("thread_id")?;
        if threads
            .last()
            .map(|thread| thread.thread_id.as_str() != thread_id.as_str())
            .unwrap_or(true)
        {
            threads.push(AccountDataDmThreadExport {
                thread_id: thread_id.clone(),
                kind: row.try_get::<String, _>("kind")?,
                title: row.try_get::<String, _>("title")?,
                created_at: row.try_get::<String, _>("created_at")?,
                participants: Vec::new(),
            });
        }

        let thread = threads
            .last_mut()
            .ok_or_else(|| sqlx::Error::Protocol("missing account data thread".into()))?;
        thread
            .participants
            .push(AccountDataDmThreadParticipantExport {
                identity_id: row.try_get::<String, _>("identity_id")?,
                last_read_seq: to_u64(
                    row.try_get::<i64, _>("last_read_seq")?,
                    "last read sequence",
                )?,
            });
    }

    Ok(threads)
}

async fn load_dm_messages(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataDmMessageExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT message.message_id,
               message.thread_id,
               message.author_id,
               message.seq,
               message.ciphertext,
               TO_CHAR(message.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
               CASE
                   WHEN message.edited_at IS NULL THEN NULL
                   ELSE TO_CHAR(message.edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
               END AS edited_at
        FROM dm_thread_participants self
        INNER JOIN dm_messages message ON message.thread_id = self.thread_id
        WHERE self.identity_id = $1
        ORDER BY message.thread_id ASC, message.seq ASC, message.message_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AccountDataDmMessageExport {
                message_id: row.try_get::<String, _>("message_id")?,
                thread_id: row.try_get::<String, _>("thread_id")?,
                author_id: row.try_get::<String, _>("author_id")?,
                seq: to_u64(row.try_get::<i64, _>("seq")?, "dm message sequence")?,
                ciphertext: row.try_get::<String, _>("ciphertext")?,
                created_at: row.try_get::<String, _>("created_at")?,
                edited_at: row.try_get::<Option<String>, _>("edited_at")?,
            })
        })
        .collect()
}

async fn load_server_channel_messages(
    pool: &PgPool,
    identity_id: &str,
) -> Result<Vec<AccountDataServerChannelMessageExport>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT message.message_id,
               channel.server_id,
               message.channel_id,
               message.author_id,
               message.channel_seq,
               message.content,
               message.reply_to_message_id,
               COALESCE(
                   ARRAY_AGG(mention.mentioned_identity_id ORDER BY mention.mentioned_identity_id)
                       FILTER (WHERE mention.mentioned_identity_id IS NOT NULL),
                   ARRAY[]::TEXT[]
               ) AS mention_identity_ids,
               TO_CHAR(message.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS created_at,
               CASE
                   WHEN message.edited_at IS NULL THEN NULL
                   ELSE TO_CHAR(message.edited_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
               END AS edited_at,
               CASE
                   WHEN message.deleted_at IS NULL THEN NULL
                   ELSE TO_CHAR(message.deleted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
               END AS deleted_at
        FROM server_channel_messages message
        INNER JOIN server_channels channel ON channel.channel_id = message.channel_id
        LEFT JOIN server_channel_message_mentions mention ON mention.message_id = message.message_id
        WHERE message.author_id = $1
        GROUP BY message.message_id,
                 channel.server_id,
                 message.channel_id,
                 message.author_id,
                 message.channel_seq,
                 message.content,
                 message.reply_to_message_id,
                 message.created_at,
                 message.edited_at,
                 message.deleted_at
        ORDER BY message.created_at ASC, message.message_id ASC
        "#,
    )
    .bind(identity_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(AccountDataServerChannelMessageExport {
                message_id: row.try_get::<String, _>("message_id")?,
                server_id: row.try_get::<String, _>("server_id")?,
                channel_id: row.try_get::<String, _>("channel_id")?,
                author_id: row.try_get::<String, _>("author_id")?,
                channel_seq: to_u64(
                    row.try_get::<i64, _>("channel_seq")?,
                    "server channel message sequence",
                )?,
                content: row.try_get::<String, _>("content")?,
                reply_to_message_id: row.try_get::<Option<String>, _>("reply_to_message_id")?,
                mention_identity_ids: row.try_get::<Vec<String>, _>("mention_identity_ids")?,
                created_at: row.try_get::<String, _>("created_at")?,
                edited_at: row.try_get::<Option<String>, _>("edited_at")?,
                deleted_at: row.try_get::<Option<String>, _>("deleted_at")?,
            })
        })
        .collect()
}

fn to_u32(value: impl TryInto<u32>, field_name: &'static str) -> Result<u32, sqlx::Error> {
    value
        .try_into()
        .map_err(|_| sqlx::Error::Protocol(format!("{field_name} must be non-negative")))
}

fn to_u64(value: impl TryInto<u64>, field_name: &'static str) -> Result<u64, sqlx::Error> {
    value
        .try_into()
        .map_err(|_| sqlx::Error::Protocol(format!("{field_name} must be non-negative")))
}
