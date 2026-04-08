use std::collections::BTreeSet;

use crate::state::AppState;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::error::TrySendError,
    sync::mpsc::Sender,
    time::{sleep, Duration},
};
use tracing::warn;

use crate::domain::replay_store;

const CHANNEL_EVENTS_CHANNEL: &str = "channels:v1:events";
const CHANNEL_REPLAY_LOG_MAX_ENTRIES: usize = 256;
const CHANNEL_DEVICE_CURSOR_TTL_SECONDS: u64 = 86_400;

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageCreatedData {
    pub message_id: String,
    pub server_id: String,
    pub channel_id: String,
    pub sender_identity_id: String,
    pub created_at: String,
    pub channel_seq: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageUpdatedData {
    pub message_id: String,
    pub server_id: String,
    pub channel_id: String,
    pub editor_identity_id: String,
    pub edited_at: String,
    pub channel_seq: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageDeletedData {
    pub message_id: String,
    pub server_id: String,
    pub channel_id: String,
    pub deleter_identity_id: String,
    pub deleted_at: String,
    pub channel_seq: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelRecipientCursor {
    pub recipient_identity_id: String,
    pub cursor: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageCreatedEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub event_version: u8,
    pub occurred_at: String,
    pub correlation_id: String,
    pub producer: String,
    pub recipients: Vec<String>,
    #[serde(default)]
    pub recipient_cursors: Vec<ChannelRecipientCursor>,
    pub data: ChannelMessageCreatedData,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageUpdatedEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub event_version: u8,
    pub occurred_at: String,
    pub correlation_id: String,
    pub producer: String,
    pub recipients: Vec<String>,
    #[serde(default)]
    pub recipient_cursors: Vec<ChannelRecipientCursor>,
    pub data: ChannelMessageUpdatedData,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ChannelMessageDeletedEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub event_version: u8,
    pub occurred_at: String,
    pub correlation_id: String,
    pub producer: String,
    pub recipients: Vec<String>,
    #[serde(default)]
    pub recipient_cursors: Vec<ChannelRecipientCursor>,
    pub data: ChannelMessageDeletedData,
}

#[derive(Deserialize)]
struct ChannelPubsubEnvelope {
    event_type: String,
    correlation_id: String,
    recipients: Vec<String>,
    #[serde(default)]
    recipient_cursors: Vec<ChannelRecipientCursor>,
    data: serde_json::Value,
}

#[derive(Clone)]
pub struct PublishChannelMessageCreatedInput {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub created_at: Option<String>,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Clone)]
pub struct PublishChannelMessageUpdatedInput {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub editor_id: String,
    pub edited_at: Option<String>,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

#[derive(Clone)]
pub struct PublishChannelMessageDeletedInput {
    pub message_id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub deleted_by: String,
    pub deleted_at: Option<String>,
    pub channel_seq: u64,
    pub recipients: Vec<String>,
}

pub fn spawn_channel_subscriber(state: AppState) {
    if state.presence_redis_client.is_none() {
        return;
    }

    tokio::spawn(async move {
        loop {
            let Some(client) = state.presence_redis_client.clone() else {
                return;
            };

            let mut pubsub = match client.get_async_pubsub().await {
                Ok(value) => value,
                Err(error) => {
                    warn!(error = %error, "failed to open channel pubsub connection");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if let Err(error) = pubsub.subscribe(CHANNEL_EVENTS_CHANNEL).await {
                warn!(error = %error, "failed to subscribe to channel events");
                sleep(Duration::from_secs(1)).await;
                continue;
            }

            let mut messages = pubsub.on_message();
            while let Some(message) = messages.next().await {
                let payload = match message.get_payload::<String>() {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(error = %error, "failed to decode channel pubsub payload");
                        continue;
                    }
                };

                let event = match serde_json::from_str::<ChannelPubsubEnvelope>(&payload) {
                    Ok(value) => value,
                    Err(error) => {
                        warn!(error = %error, "failed to parse channel pubsub envelope");
                        continue;
                    }
                };

                match event.event_type.as_str() {
                    "channel.message.created" => {
                        let data = match serde_json::from_value::<ChannelMessageCreatedData>(
                            event.data,
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(error = %error, "failed to parse channel.message.created data");
                                continue;
                            }
                        };

                        let client_payload =
                            crate::domain::events::service::build_channel_message_created_event(
                                &data.message_id,
                                &data.server_id,
                                &data.channel_id,
                                &data.sender_identity_id,
                                &data.created_at,
                                data.channel_seq,
                                Some(event.correlation_id.clone()),
                            );

                        dispatch_channel_event_locally(
                            &state,
                            &client_payload,
                            &event.recipients,
                            &event.recipient_cursors,
                        )
                        .await;
                    }
                    "channel.message.updated" => {
                        let data = match serde_json::from_value::<ChannelMessageUpdatedData>(
                            event.data,
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(error = %error, "failed to parse channel.message.updated data");
                                continue;
                            }
                        };

                        let client_payload =
                            crate::domain::events::service::build_channel_message_updated_event(
                                &data.message_id,
                                &data.server_id,
                                &data.channel_id,
                                &data.editor_identity_id,
                                &data.edited_at,
                                data.channel_seq,
                                Some(event.correlation_id.clone()),
                            );

                        dispatch_channel_event_locally(
                            &state,
                            &client_payload,
                            &event.recipients,
                            &event.recipient_cursors,
                        )
                        .await;
                    }
                    "channel.message.deleted" => {
                        let data = match serde_json::from_value::<ChannelMessageDeletedData>(
                            event.data,
                        ) {
                            Ok(value) => value,
                            Err(error) => {
                                warn!(error = %error, "failed to parse channel.message.deleted data");
                                continue;
                            }
                        };

                        let client_payload =
                            crate::domain::events::service::build_channel_message_deleted_event(
                                &data.message_id,
                                &data.server_id,
                                &data.channel_id,
                                &data.deleter_identity_id,
                                &data.deleted_at,
                                data.channel_seq,
                                Some(event.correlation_id.clone()),
                            );

                        dispatch_channel_event_locally(
                            &state,
                            &client_payload,
                            &event.recipients,
                            &event.recipient_cursors,
                        )
                        .await;
                    }
                    other => {
                        warn!(event_type = %other, "unsupported channel pubsub event type");
                        continue;
                    }
                }
            }

            warn!("channel pubsub stream ended; retrying subscription");
            sleep(Duration::from_secs(1)).await;
        }
    });
}

pub async fn hydrate_channel_backlog_if_needed(
    state: &AppState,
    identity_id: &str,
    device_id: Option<&str>,
    outbound_tx: &Sender<String>,
) {
    let Some(device_id) = device_id else {
        return;
    };
    let Some(client) = state.presence_redis_client.as_ref() else {
        return;
    };

    let mut connection = match client.get_multiplexed_tokio_connection().await {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to open Redis connection for channel hydration");
            return;
        }
    };

    let current_cursor = match replay_store::get_device_cursor(
        &mut connection,
        channel_device_cursor_key,
        identity_id,
        device_id,
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to load channel device cursor");
            return;
        }
    };

    let replay_entries = match replay_store::list_replay_entries(
        &mut connection,
        channel_replay_log_key,
        identity_id,
    )
    .await
    {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to load channel replay entries");
            return;
        }
    };

    let mut latest_cursor = current_cursor;
    for entry in replay_entries
        .into_iter()
        .filter(|entry| entry.cursor > current_cursor)
    {
        match outbound_tx.try_send(entry.payload.clone()) {
            Ok(()) => latest_cursor = latest_cursor.max(entry.cursor),
            Err(TrySendError::Closed(_)) | Err(TrySendError::Full(_)) => break,
        }
    }

    if latest_cursor > current_cursor {
        if let Err(error) = replay_store::set_device_cursor(
            &mut connection,
            channel_device_cursor_key,
            CHANNEL_DEVICE_CURSOR_TTL_SECONDS,
            identity_id,
            device_id,
            latest_cursor,
        )
        .await
        {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to persist hydrated channel device cursor");
        }
    }
}

pub async fn publish_channel_message_created(
    state: &AppState,
    input: PublishChannelMessageCreatedInput,
) -> Result<(), String> {
    let Some(client) = state.presence_redis_client.as_ref() else {
        return Ok(());
    };

    let recipients = normalize_recipients(&input.recipients);
    if recipients.is_empty() {
        return Ok(());
    }
    if input.channel_seq == 0 {
        return Err("channel_seq must be greater than zero".to_string());
    }

    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| format!("open Redis connection: {error}"))?;

    let created_at = input.created_at.unwrap_or_else(|| Utc::now().to_rfc3339());
    let client_payload = crate::domain::events::service::build_channel_message_created_event(
        &input.message_id,
        &input.guild_id,
        &input.channel_id,
        &input.sender_id,
        &created_at,
        input.channel_seq,
        None,
    );
    let client_event: serde_json::Value = serde_json::from_str(&client_payload)
        .map_err(|error| format!("decode channel event: {error}"))?;
    let replay_recipients = active_replay_recipients(state, &recipients).await;
    let recipient_cursors =
        persist_channel_replay_entries(&mut connection, &replay_recipients, &client_payload)
            .await
            .map_err(|error| format!("persist channel replay entries: {error}"))?;

    let event = ChannelMessageCreatedEnvelope {
        event_id: client_event["event_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_type: client_event["event_type"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_version: client_event["event_version"].as_u64().unwrap_or(1) as u8,
        occurred_at: client_event["occurred_at"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        correlation_id: client_event["correlation_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        producer: client_event["producer"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        recipients,
        recipient_cursors,
        data: ChannelMessageCreatedData {
            message_id: input.message_id,
            server_id: input.guild_id,
            channel_id: input.channel_id,
            sender_identity_id: input.sender_id,
            created_at,
            channel_seq: input.channel_seq,
        },
    };
    let event_json = serde_json::to_string(&event)
        .map_err(|error| format!("serialize channel event: {error}"))?;

    let _: () = redis::cmd("PUBLISH")
        .arg(CHANNEL_EVENTS_CHANNEL)
        .arg(event_json)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("publish channel event: {error}"))?;

    Ok(())
}

pub async fn publish_channel_message_updated(
    state: &AppState,
    input: PublishChannelMessageUpdatedInput,
) -> Result<(), String> {
    let Some(client) = state.presence_redis_client.as_ref() else {
        return Ok(());
    };

    let recipients = normalize_recipients(&input.recipients);
    if recipients.is_empty() {
        return Ok(());
    }
    if input.channel_seq == 0 {
        return Err("channel_seq must be greater than zero".to_string());
    }

    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| format!("open Redis connection: {error}"))?;

    let edited_at = input.edited_at.unwrap_or_else(|| Utc::now().to_rfc3339());
    let client_payload = crate::domain::events::service::build_channel_message_updated_event(
        &input.message_id,
        &input.guild_id,
        &input.channel_id,
        &input.editor_id,
        &edited_at,
        input.channel_seq,
        None,
    );
    let client_event: serde_json::Value = serde_json::from_str(&client_payload)
        .map_err(|error| format!("decode channel update event: {error}"))?;
    let replay_recipients = active_replay_recipients(state, &recipients).await;
    let recipient_cursors =
        persist_channel_replay_entries(&mut connection, &replay_recipients, &client_payload)
            .await
            .map_err(|error| format!("persist channel replay entries: {error}"))?;

    let event = ChannelMessageUpdatedEnvelope {
        event_id: client_event["event_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_type: client_event["event_type"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_version: client_event["event_version"].as_u64().unwrap_or(1) as u8,
        occurred_at: client_event["occurred_at"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        correlation_id: client_event["correlation_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        producer: client_event["producer"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        recipients,
        recipient_cursors,
        data: ChannelMessageUpdatedData {
            message_id: input.message_id,
            server_id: input.guild_id,
            channel_id: input.channel_id,
            editor_identity_id: input.editor_id,
            edited_at,
            channel_seq: input.channel_seq,
        },
    };
    let event_json = serde_json::to_string(&event)
        .map_err(|error| format!("serialize channel update event: {error}"))?;

    let _: () = redis::cmd("PUBLISH")
        .arg(CHANNEL_EVENTS_CHANNEL)
        .arg(event_json)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("publish channel update event: {error}"))?;

    Ok(())
}

pub async fn publish_channel_message_deleted(
    state: &AppState,
    input: PublishChannelMessageDeletedInput,
) -> Result<(), String> {
    let Some(client) = state.presence_redis_client.as_ref() else {
        return Ok(());
    };

    let recipients = normalize_recipients(&input.recipients);
    if recipients.is_empty() {
        return Ok(());
    }
    if input.channel_seq == 0 {
        return Err("channel_seq must be greater than zero".to_string());
    }

    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| format!("open Redis connection: {error}"))?;

    let deleted_at = input.deleted_at.unwrap_or_else(|| Utc::now().to_rfc3339());
    let client_payload = crate::domain::events::service::build_channel_message_deleted_event(
        &input.message_id,
        &input.guild_id,
        &input.channel_id,
        &input.deleted_by,
        &deleted_at,
        input.channel_seq,
        None,
    );
    let client_event: serde_json::Value = serde_json::from_str(&client_payload)
        .map_err(|error| format!("decode channel delete event: {error}"))?;
    let replay_recipients = active_replay_recipients(state, &recipients).await;
    let recipient_cursors =
        persist_channel_replay_entries(&mut connection, &replay_recipients, &client_payload)
            .await
            .map_err(|error| format!("persist channel replay entries: {error}"))?;

    let event = ChannelMessageDeletedEnvelope {
        event_id: client_event["event_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_type: client_event["event_type"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        event_version: client_event["event_version"].as_u64().unwrap_or(1) as u8,
        occurred_at: client_event["occurred_at"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        correlation_id: client_event["correlation_id"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        producer: client_event["producer"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        recipients,
        recipient_cursors,
        data: ChannelMessageDeletedData {
            message_id: input.message_id,
            server_id: input.guild_id,
            channel_id: input.channel_id,
            deleter_identity_id: input.deleted_by,
            deleted_at,
            channel_seq: input.channel_seq,
        },
    };
    let event_json = serde_json::to_string(&event)
        .map_err(|error| format!("serialize channel delete event: {error}"))?;

    let _: () = redis::cmd("PUBLISH")
        .arg(CHANNEL_EVENTS_CHANNEL)
        .arg(event_json)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("publish channel delete event: {error}"))?;

    Ok(())
}

fn normalize_recipients(recipients: &[String]) -> Vec<String> {
    let mut deduped = BTreeSet::new();
    for recipient in recipients {
        let trimmed = recipient.trim();
        if !trimmed.is_empty() {
            deduped.insert(trimmed.to_string());
        }
    }
    deduped.into_iter().collect()
}

async fn active_replay_recipients(state: &AppState, recipients: &[String]) -> Vec<String> {
    let guard = state.connection_senders.lock().await;
    recipients
        .iter()
        .filter(|recipient_identity_id| {
            guard
                .get(recipient_identity_id.as_str())
                .map(|connections| {
                    connections
                        .values()
                        .any(|entry| entry.device_id.as_ref().is_some())
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

async fn persist_channel_replay_entries(
    connection: &mut redis::aio::MultiplexedConnection,
    recipients: &[String],
    client_payload: &str,
) -> Result<Vec<ChannelRecipientCursor>, redis::RedisError> {
    replay_store::persist_replay_entries(
        connection,
        recipients,
        client_payload,
        CHANNEL_REPLAY_LOG_MAX_ENTRIES,
        channel_stream_head_key,
        channel_replay_log_key,
        |recipient_identity_id, cursor| ChannelRecipientCursor {
            recipient_identity_id: recipient_identity_id.to_string(),
            cursor,
        },
    )
    .await
}

fn channel_stream_head_key(identity_id: &str) -> String {
    format!("channels:v1:recipient_stream_head:{identity_id}")
}

fn channel_replay_log_key(identity_id: &str) -> String {
    format!("channels:v1:recipient_stream_log:{identity_id}")
}

fn channel_device_cursor_key(identity_id: &str, device_id: &str) -> String {
    format!("channels:v1:recipient_device_cursor:{identity_id}:{device_id}")
}

async fn dispatch_channel_event_locally(
    state: &AppState,
    payload: &str,
    recipients: &[String],
    recipient_cursors: &[ChannelRecipientCursor],
) {
    let mut stale_connections = Vec::new();
    let mut guard = state.connection_senders.lock().await;
    let mut delivered_device_cursors = BTreeSet::new();
    let recipient_cursor_map = recipient_cursors
        .iter()
        .map(|entry| (entry.recipient_identity_id.as_str(), entry.cursor))
        .collect::<std::collections::HashMap<_, _>>();

    for recipient_identity_id in recipients {
        let Some(connections) = guard.get_mut(recipient_identity_id) else {
            continue;
        };
        let recipient_cursor = recipient_cursor_map
            .get(recipient_identity_id.as_str())
            .copied();

        for (connection_id, entry) in connections.iter() {
            match entry.sender.try_send(payload.to_string()) {
                Ok(()) => {
                    if let (Some(device_id), Some(cursor)) =
                        (entry.device_id.as_ref(), recipient_cursor)
                    {
                        delivered_device_cursors.insert((
                            recipient_identity_id.clone(),
                            device_id.clone(),
                            cursor,
                        ));
                    }
                }
                Err(TrySendError::Closed(_)) => {
                    stale_connections.push((recipient_identity_id.clone(), connection_id.clone()));
                }
                Err(TrySendError::Full(_)) => {
                    warn!(
                        recipient_identity_id = %recipient_identity_id,
                        connection_id = %connection_id,
                        "channel outbound queue saturated; keeping websocket registered"
                    );
                }
            }
        }
    }

    for (identity_id, connection_id) in stale_connections {
        if let Some(connections) = guard.get_mut(&identity_id) {
            connections.remove(&connection_id);
            if connections.is_empty() {
                guard.remove(&identity_id);
            }
        }
    }

    drop(guard);

    if delivered_device_cursors.is_empty() {
        return;
    }
    let Some(client) = state.presence_redis_client.as_ref() else {
        return;
    };
    let mut connection = match client.get_multiplexed_tokio_connection().await {
        Ok(value) => value,
        Err(error) => {
            warn!(error = %error, "failed to open Redis connection for channel cursor updates");
            return;
        }
    };

    for (identity_id, device_id, cursor) in delivered_device_cursors {
        if let Err(error) = replay_store::set_device_cursor(
            &mut connection,
            channel_device_cursor_key,
            CHANNEL_DEVICE_CURSOR_TTL_SECONDS,
            &identity_id,
            &device_id,
            cursor,
        )
        .await
        {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to persist live channel device cursor");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn dispatch_channel_event_locally_keeps_full_connections_registered() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
            None,
            false,
            60,
            60,
            32 * 1024,
            120,
            60,
            3,
            5,
            2048,
        )
        .expect("build app state");
        let (full_tx, mut full_rx) = mpsc::channel::<String>(1);
        full_tx
            .try_send("seed".to_string())
            .expect("fill outbound queue");

        state.connection_senders.lock().await.insert(
            "usr-main".to_string(),
            std::collections::HashMap::from([(
                "conn-full".to_string(),
                crate::state::ConnectionSenderEntry {
                    sender: full_tx,
                    device_id: Some("device-a".to_string()),
                },
            )]),
        );

        dispatch_channel_event_locally(
            &state,
            "payload-1",
            &["usr-main".to_string()],
            &[ChannelRecipientCursor {
                recipient_identity_id: "usr-main".to_string(),
                cursor: 4,
            }],
        )
        .await;

        assert_eq!(full_rx.recv().await.as_deref(), Some("seed"));
        let guard = state.connection_senders.lock().await;
        let connections = guard.get("usr-main").expect("remaining connections");
        assert!(connections.contains_key("conn-full"));
    }
}
