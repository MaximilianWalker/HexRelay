use std::collections::BTreeSet;

use crate::state::AppState;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::warn;

const PRESENCE_EVENTS_CHANNEL: &str = "presence:v1:events";

#[derive(Clone, Deserialize, Serialize)]
pub struct PresenceUpdatedData {
    pub user_id: String,
    pub status: String,
    pub updated_at: String,
    pub presence_seq: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PresenceUpdatedEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub event_version: u8,
    pub occurred_at: String,
    pub correlation_id: String,
    pub producer: String,
    pub watchers: Vec<String>,
    pub data: PresenceUpdatedData,
}

#[derive(Deserialize, Serialize)]
struct PresenceSnapshot {
    status: String,
    updated_at: String,
    presence_seq: u64,
}

#[derive(Deserialize)]
struct PresenceWatcherListResponse {
    watchers: Vec<String>,
}

pub fn spawn_presence_subscriber(state: AppState) {
    if state.presence_redis_client.is_none() {
        return;
    }

    tokio::spawn(async move {
        let Some(client) = state.presence_redis_client.clone() else {
            return;
        };

        let mut pubsub = match client.get_async_pubsub().await {
            Ok(value) => value,
            Err(error) => {
                warn!(error = %error, "failed to open presence pubsub connection");
                return;
            }
        };

        if let Err(error) = pubsub.subscribe(PRESENCE_EVENTS_CHANNEL).await {
            warn!(error = %error, "failed to subscribe to presence channel");
            return;
        }

        let mut messages = pubsub.on_message();
        while let Some(message) = messages.next().await {
            let payload = match message.get_payload::<String>() {
                Ok(value) => value,
                Err(error) => {
                    warn!(error = %error, "failed to decode presence pubsub payload");
                    continue;
                }
            };

            let event = match serde_json::from_str::<PresenceUpdatedEnvelope>(&payload) {
                Ok(value) => value,
                Err(error) => {
                    warn!(error = %error, "failed to parse presence pubsub payload");
                    continue;
                }
            };

            let client_payload = crate::domain::events::service::build_presence_updated_event(
                &event.data.user_id,
                &event.data.status,
                &event.data.updated_at,
                event.data.presence_seq,
                Some(event.correlation_id.clone()),
            );

            dispatch_presence_event_locally(&state, &client_payload, &event.watchers).await;
        }
    });
}

pub async fn publish_online_if_needed(state: &AppState, identity_id: &str) {
    if let Err(error) = publish_presence_edge(state, identity_id, true).await {
        warn!(identity_id = %identity_id, error = %error, "failed to publish online presence edge");
    }
}

pub async fn publish_offline_if_needed(state: &AppState, identity_id: &str) {
    if let Err(error) = publish_presence_edge(state, identity_id, false).await {
        warn!(identity_id = %identity_id, error = %error, "failed to publish offline presence edge");
    }
}

async fn publish_presence_edge(
    state: &AppState,
    identity_id: &str,
    online: bool,
) -> Result<(), String> {
    let Some(client) = state.presence_redis_client.as_ref() else {
        return Ok(());
    };

    let mut connection = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|error| format!("open Redis connection: {error}"))?;

    let count_key = format!("presence:v1:count:{identity_id}");
    let next_count: i64 = if online {
        redis::cmd("INCR")
            .arg(&count_key)
            .query_async(&mut connection)
            .await
            .map_err(|error| format!("increment presence count: {error}"))?
    } else {
        let decremented: i64 = redis::cmd("DECR")
            .arg(&count_key)
            .query_async(&mut connection)
            .await
            .map_err(|error| format!("decrement presence count: {error}"))?;
        if decremented <= 0 {
            let _: () = redis::cmd("DEL")
                .arg(&count_key)
                .query_async(&mut connection)
                .await
                .map_err(|error| format!("clear presence count: {error}"))?;
            0
        } else {
            decremented
        }
    };

    if online && next_count != 1 {
        return Ok(());
    }
    if !online && next_count > 0 {
        return Ok(());
    }

    let sequence_key = format!("presence:v1:seq:{identity_id}");
    let presence_seq: u64 = redis::cmd("INCR")
        .arg(sequence_key)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("advance presence sequence: {error}"))?;

    let updated_at = Utc::now().to_rfc3339();
    let status = if online { "online" } else { "offline" };
    let snapshot = PresenceSnapshot {
        status: status.to_string(),
        updated_at: updated_at.clone(),
        presence_seq,
    };
    let snapshot_json = serde_json::to_string(&snapshot)
        .map_err(|error| format!("serialize presence snapshot: {error}"))?;
    let snapshot_key = format!("presence:v1:snapshot:{identity_id}");
    let _: () = redis::cmd("SET")
        .arg(snapshot_key)
        .arg(snapshot_json)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("persist presence snapshot: {error}"))?;

    let watchers = resolve_watchers(state, identity_id).await;
    let client_payload = crate::domain::events::service::build_presence_updated_event(
        identity_id,
        status,
        &updated_at,
        presence_seq,
        None,
    );
    let client_event: serde_json::Value = serde_json::from_str(&client_payload)
        .map_err(|error| format!("decode client presence event: {error}"))?;
    let event = PresenceUpdatedEnvelope {
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
        watchers,
        data: PresenceUpdatedData {
            user_id: identity_id.to_string(),
            status: status.to_string(),
            updated_at,
            presence_seq,
        },
    };
    let event_json = serde_json::to_string(&event)
        .map_err(|error| format!("serialize presence event: {error}"))?;

    let _: () = redis::cmd("PUBLISH")
        .arg(PRESENCE_EVENTS_CHANNEL)
        .arg(event_json)
        .query_async(&mut connection)
        .await
        .map_err(|error| format!("publish presence event: {error}"))?;

    Ok(())
}

async fn resolve_watchers(state: &AppState, identity_id: &str) -> Vec<String> {
    let mut watchers = BTreeSet::from([identity_id.to_string()]);
    let url = format!(
        "{}/v1/internal/presence/watchers/{}",
        state.api_base_url.trim_end_matches('/'),
        identity_id
    );

    let response = match state
        .http_client
        .get(url)
        .header("x-hexrelay-internal-token", &state.presence_internal_token)
        .send()
        .await
    {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, error = %error, "presence watcher lookup failed");
            return watchers.into_iter().collect();
        }
    };

    if !response.status().is_success() {
        warn!(identity_id = %identity_id, status = %response.status(), "presence watcher lookup returned non-success status");
        return watchers.into_iter().collect();
    }

    match response.json::<PresenceWatcherListResponse>().await {
        Ok(payload) => {
            watchers.extend(payload.watchers);
            watchers.into_iter().collect()
        }
        Err(error) => {
            warn!(identity_id = %identity_id, error = %error, "presence watcher payload decode failed");
            watchers.into_iter().collect()
        }
    }
}

async fn dispatch_presence_event_locally(state: &AppState, payload: &str, watchers: &[String]) {
    let mut stale_connections = Vec::new();
    let mut guard = state.connection_senders.lock().await;

    for watcher_identity_id in watchers {
        let Some(connections) = guard.get_mut(watcher_identity_id) else {
            continue;
        };

        for (connection_id, sender) in connections.iter() {
            if sender.send(payload.to_string()).is_err() {
                stale_connections.push((watcher_identity_id.clone(), connection_id.clone()));
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
}
