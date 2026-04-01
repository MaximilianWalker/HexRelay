use std::collections::BTreeSet;

use crate::state::AppState;
use chrono::Utc;
use communication_core::{
    app::CommunicationRouter,
    domain::{
        CommunicationMode, CommunicationReasonCode, ConnectIntent, SendEnvelope, SessionProvenance,
        TransportProfile,
    },
    transport::{DirectPeerTransport, NodeClientTransport, TransportError},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::error::TrySendError,
    sync::mpsc::Sender,
    time::{sleep, Duration},
};
use tracing::warn;

const PRESENCE_EVENTS_CHANNEL: &str = "presence:v1:events";
const PRESENCE_SNAPSHOT_TTL_SECONDS: u64 = 120;
const PRESENCE_DEVICE_CURSOR_TTL_SECONDS: u64 = 86_400;
const PRESENCE_REPLAY_LOG_MAX_ENTRIES: usize = 128;

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
    #[serde(default)]
    pub watcher_cursors: Vec<PresenceWatcherCursor>,
    pub data: PresenceUpdatedData,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct PresenceWatcherCursor {
    pub watcher_identity_id: String,
    pub cursor: u64,
}

#[derive(Clone, Deserialize, Serialize)]
struct PresenceReplayEntry {
    cursor: u64,
    payload: String,
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

#[derive(Serialize)]
#[serde(tag = "kind", content = "body")]
enum PresenceDispatchEnvelope<'a> {
    #[serde(rename = "presence_edge")]
    Edge(PresenceEdgeDispatchRequest<'a>),
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "body")]
enum OwnedPresenceDispatchEnvelope {
    #[serde(rename = "presence_edge")]
    Edge(OwnedPresenceEdgeDispatchRequest),
}

#[derive(Serialize)]
struct PresenceEdgeDispatchRequest<'a> {
    identity_id: &'a str,
    online: bool,
}

#[derive(Deserialize)]
struct OwnedPresenceEdgeDispatchRequest {
    identity_id: String,
    online: bool,
}

#[derive(Clone)]
struct LocalPresenceNodeClientTransport {
    state: AppState,
}

struct UnusedDirectPeerTransport;

impl DirectPeerTransport for UnusedDirectPeerTransport {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

impl NodeClientTransport for LocalPresenceNodeClientTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::NodeClient,
            reason_code: match intent.mode {
                CommunicationMode::Presence => CommunicationReasonCode::PresenceRouteSelected,
                CommunicationMode::ServerChannel => {
                    CommunicationReasonCode::ServerChannelRouteSelected
                }
                CommunicationMode::DmDirect => CommunicationReasonCode::DmDirectPolicyViolation,
            },
            policy_assertions: vec!["node_client_transport_selected".to_string()],
        })
    }

    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError> {
        let dispatch = PresenceNodeDispatch::from_payload(&envelope.payload)?;
        let state = self.state.clone();
        let handle =
            tokio::runtime::Handle::try_current().map_err(|_| TransportError::SendFailed)?;
        let identity_id = dispatch.identity_id.clone();
        let online = dispatch.online;

        let result = match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::CurrentThread => std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|_| TransportError::SendFailed)?
                    .block_on(async move {
                        dispatch
                            .publish(&state)
                            .await
                            .map_err(|_| TransportError::SendFailed)
                    })
            })
            .join()
            .map_err(|_| TransportError::SendFailed)?,
            tokio::runtime::RuntimeFlavor::MultiThread => tokio::task::block_in_place(move || {
                handle.block_on(async move {
                    dispatch
                        .publish(&state)
                        .await
                        .map_err(|_| TransportError::SendFailed)
                })
            }),
            _ => Err(TransportError::SendFailed),
        };

        if result.is_err() {
            warn!(
                identity_id = %identity_id,
                online,
                "NodeClientTransport presence dispatch failed"
            );
        }

        result
    }
}

struct PresenceNodeDispatch {
    identity_id: String,
    online: bool,
}

impl PresenceNodeDispatch {
    fn from_payload(payload: &[u8]) -> Result<Self, TransportError> {
        let envelope: OwnedPresenceDispatchEnvelope =
            serde_json::from_slice(payload).map_err(|_| TransportError::SendFailed)?;
        match envelope {
            OwnedPresenceDispatchEnvelope::Edge(body) => Ok(Self {
                identity_id: body.identity_id,
                online: body.online,
            }),
        }
    }

    async fn publish(&self, state: &AppState) -> Result<(), String> {
        publish_presence_edge_direct(state, &self.identity_id, self.online).await
    }
}

pub fn spawn_presence_subscriber(state: AppState) {
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
                    warn!(error = %error, "failed to open presence pubsub connection");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if let Err(error) = pubsub.subscribe(PRESENCE_EVENTS_CHANNEL).await {
                warn!(error = %error, "failed to subscribe to presence channel");
                sleep(Duration::from_secs(1)).await;
                continue;
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

                dispatch_presence_event_locally(
                    &state,
                    &client_payload,
                    &event.watchers,
                    &event.watcher_cursors,
                )
                .await;
            }

            warn!("presence pubsub stream ended; retrying subscription");
            sleep(Duration::from_secs(1)).await;
        }
    });
}

pub async fn publish_online_if_needed(state: &AppState, identity_id: &str) {
    if let Err(error) = dispatch_presence_edge(state, identity_id, true) {
        warn!(identity_id = %identity_id, error = %error, "failed to publish online presence edge");
    }
}

pub async fn publish_offline_if_needed(state: &AppState, identity_id: &str) {
    if let Err(error) = dispatch_presence_edge(state, identity_id, false) {
        warn!(identity_id = %identity_id, error = %error, "failed to publish offline presence edge");
    }
}

pub async fn hydrate_presence_backlog_if_needed(
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
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to open Redis connection for presence hydration");
            return;
        }
    };

    let current_cursor = match get_presence_device_cursor(&mut connection, identity_id, device_id)
        .await
    {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to load presence device cursor");
            return;
        }
    };

    let replay_entries = match list_presence_replay_entries(&mut connection, identity_id).await {
        Ok(value) => value,
        Err(error) => {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to load presence replay entries");
            return;
        }
    };

    let mut latest_cursor = current_cursor;
    for entry in replay_entries
        .into_iter()
        .filter(|entry| entry.cursor > current_cursor)
    {
        match outbound_tx.try_send(entry.payload.clone()) {
            Ok(()) => {
                latest_cursor = latest_cursor.max(entry.cursor);
            }
            Err(TrySendError::Closed(_)) | Err(TrySendError::Full(_)) => {
                break;
            }
        }
    }

    if latest_cursor > current_cursor {
        if let Err(error) =
            set_presence_device_cursor(&mut connection, identity_id, device_id, latest_cursor).await
        {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to persist hydrated presence device cursor");
        }
    }
}

fn dispatch_presence_edge(state: &AppState, identity_id: &str, online: bool) -> Result<(), String> {
    let payload = serde_json::to_vec(&PresenceDispatchEnvelope::Edge(
        PresenceEdgeDispatchRequest {
            identity_id,
            online,
        },
    ))
    .map_err(|error| format!("encode presence dispatch payload: {error}"))?;

    let router = CommunicationRouter::new(
        communication_core::PolicyContext::default(),
        UnusedDirectPeerTransport,
        LocalPresenceNodeClientTransport {
            state: state.clone(),
        },
    );

    router
        .send(&SendEnvelope {
            mode: CommunicationMode::Presence,
            payload,
        })
        .map_err(|error| {
            format!(
                "dispatch presence event via NodeClientTransport: {:?}",
                error.code
            )
        })
}

async fn publish_presence_edge_direct(
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
        let decremented: i64 = redis::cmd("EVAL")
            .arg(
                r#"
                local key = KEYS[1]
                local val = redis.call('GET', key)
                if not val then
                  return -1
                end
                local num = tonumber(val)
                if not num or num <= 0 then
                  redis.call('DEL', key)
                  return 0
                end
                num = num - 1
                if num <= 0 then
                  redis.call('DEL', key)
                  return 0
                end
                redis.call('SET', key, num)
                return num
                "#,
            )
            .arg(1)
            .arg(&count_key)
            .query_async(&mut connection)
            .await
            .map_err(|error| format!("decrement presence count: {error}"))?;
        if decremented < 0 {
            return Ok(());
        } else if decremented == 0 {
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
        .arg("EX")
        .arg(PRESENCE_SNAPSHOT_TTL_SECONDS)
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
    let replay_watchers = active_replay_watchers(state, &watchers).await;
    let watcher_cursors =
        persist_presence_replay_entries(&mut connection, &replay_watchers, &client_payload)
            .await
            .map_err(|error| format!("persist presence replay entries: {error}"))?;
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
        watcher_cursors,
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

async fn persist_presence_replay_entries(
    connection: &mut redis::aio::MultiplexedConnection,
    watchers: &[String],
    client_payload: &str,
) -> Result<Vec<PresenceWatcherCursor>, redis::RedisError> {
    let mut cursors = Vec::with_capacity(watchers.len());
    for watcher_identity_id in watchers {
        let cursor = advance_presence_stream_head(connection, watcher_identity_id).await?;
        let replay_entry = serde_json::to_string(&PresenceReplayEntry {
            cursor,
            payload: client_payload.to_string(),
        })
        .map_err(|error| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "serialize presence replay entry",
                error.to_string(),
            ))
        })?;

        let replay_log_key = presence_replay_log_key(watcher_identity_id);
        let _: () = redis::cmd("LPUSH")
            .arg(&replay_log_key)
            .arg(replay_entry)
            .query_async(connection)
            .await?;
        let _: () = redis::cmd("LTRIM")
            .arg(&replay_log_key)
            .arg(0)
            .arg((PRESENCE_REPLAY_LOG_MAX_ENTRIES - 1) as i64)
            .query_async(connection)
            .await?;

        cursors.push(PresenceWatcherCursor {
            watcher_identity_id: watcher_identity_id.clone(),
            cursor,
        });
    }

    Ok(cursors)
}

async fn active_replay_watchers(state: &AppState, watchers: &[String]) -> Vec<String> {
    let guard = state.connection_senders.lock().await;
    watchers
        .iter()
        .filter(|watcher_identity_id| {
            guard
                .get(watcher_identity_id.as_str())
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

async fn advance_presence_stream_head(
    connection: &mut redis::aio::MultiplexedConnection,
    identity_id: &str,
) -> Result<u64, redis::RedisError> {
    redis::cmd("INCR")
        .arg(presence_stream_head_key(identity_id))
        .query_async(connection)
        .await
}

async fn list_presence_replay_entries(
    connection: &mut redis::aio::MultiplexedConnection,
    identity_id: &str,
) -> Result<Vec<PresenceReplayEntry>, redis::RedisError> {
    let values: Vec<String> = redis::cmd("LRANGE")
        .arg(presence_replay_log_key(identity_id))
        .arg(0)
        .arg(-1)
        .query_async(connection)
        .await?;

    let mut entries = values
        .into_iter()
        .filter_map(|value| serde_json::from_str::<PresenceReplayEntry>(&value).ok())
        .collect::<Vec<_>>();
    entries.reverse();
    Ok(entries)
}

async fn get_presence_device_cursor(
    connection: &mut redis::aio::MultiplexedConnection,
    identity_id: &str,
    device_id: &str,
) -> Result<u64, redis::RedisError> {
    redis::cmd("GET")
        .arg(presence_device_cursor_key(identity_id, device_id))
        .query_async::<Option<u64>>(connection)
        .await
        .map(|value| value.unwrap_or(0))
}

async fn set_presence_device_cursor(
    connection: &mut redis::aio::MultiplexedConnection,
    identity_id: &str,
    device_id: &str,
    cursor: u64,
) -> Result<(), redis::RedisError> {
    let _: () = redis::cmd("EVAL")
        .arg(
            r#"
            local key = KEYS[1]
            local incoming = tonumber(ARGV[1])
            local ttl = tonumber(ARGV[2])
            local current = tonumber(redis.call('GET', key) or '0')
            if incoming > current then
              current = incoming
            end
            redis.call('SET', key, current, 'EX', ttl)
            return current
            "#,
        )
        .arg(1)
        .arg(presence_device_cursor_key(identity_id, device_id))
        .arg(cursor)
        .arg(PRESENCE_DEVICE_CURSOR_TTL_SECONDS)
        .query_async(connection)
        .await?;
    Ok(())
}

fn presence_stream_head_key(identity_id: &str) -> String {
    format!("presence:v1:watcher_stream_head:{identity_id}")
}

fn presence_replay_log_key(identity_id: &str) -> String {
    format!("presence:v1:watcher_stream_log:{identity_id}")
}

fn presence_device_cursor_key(identity_id: &str, device_id: &str) -> String {
    format!("presence:v1:watcher_device_cursor:{identity_id}:{device_id}")
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

async fn dispatch_presence_event_locally(
    state: &AppState,
    payload: &str,
    watchers: &[String],
    watcher_cursors: &[PresenceWatcherCursor],
) {
    let mut stale_connections = Vec::new();
    let mut guard = state.connection_senders.lock().await;
    let mut delivered_device_cursors = BTreeSet::new();
    let watcher_cursor_map = watcher_cursors
        .iter()
        .map(|entry| (entry.watcher_identity_id.as_str(), entry.cursor))
        .collect::<std::collections::HashMap<_, _>>();

    for watcher_identity_id in watchers {
        let Some(connections) = guard.get_mut(watcher_identity_id) else {
            continue;
        };
        let watcher_cursor = watcher_cursor_map
            .get(watcher_identity_id.as_str())
            .copied();

        for (connection_id, entry) in connections.iter() {
            match entry.sender.try_send(payload.to_string()) {
                Ok(()) => {
                    if let (Some(device_id), Some(cursor)) =
                        (entry.device_id.as_ref(), watcher_cursor)
                    {
                        delivered_device_cursors.insert((
                            watcher_identity_id.clone(),
                            device_id.clone(),
                            cursor,
                        ));
                    }
                }
                Err(TrySendError::Closed(_)) | Err(TrySendError::Full(_)) => {
                    stale_connections.push((watcher_identity_id.clone(), connection_id.clone()));
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
            warn!(error = %error, "failed to open Redis connection for presence cursor updates");
            return;
        }
    };

    for (identity_id, device_id, cursor) in delivered_device_cursors {
        if let Err(error) =
            set_presence_device_cursor(&mut connection, &identity_id, &device_id, cursor).await
        {
            warn!(identity_id = %identity_id, device_id = %device_id, error = %error, "failed to persist live presence device cursor");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Json, Router};
    use serde_json::json;
    use tokio::{net::TcpListener, sync::mpsc};

    async fn start_watcher_server(
        status: axum::http::StatusCode,
        body: serde_json::Value,
    ) -> String {
        let app = Router::new().route(
            "/v1/internal/presence/watchers/usr-main",
            get(move || {
                let body = body.clone();
                async move { (status, Json(body)) }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind watcher server");
        let addr = listener.local_addr().expect("watcher server addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve watcher app");
        });
        format!("http://{}", addr)
    }

    #[tokio::test]
    async fn publish_presence_edge_is_noop_without_redis() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
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

        publish_online_if_needed(&state, "usr-main").await;
        publish_offline_if_needed(&state, "usr-main").await;
    }

    #[tokio::test]
    async fn resolve_watchers_returns_self_when_lookup_fails() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
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

        let watchers = resolve_watchers(&state, "usr-main").await;

        assert_eq!(watchers, vec!["usr-main".to_string()]);
    }

    #[tokio::test]
    async fn resolve_watchers_merges_remote_watchers() {
        let api_base_url = start_watcher_server(
            axum::http::StatusCode::OK,
            json!({"watchers": ["usr-friend", "usr-main", "usr-other"]}),
        )
        .await;
        let state = AppState::new(
            api_base_url,
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
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

        let watchers = resolve_watchers(&state, "usr-main").await;

        assert_eq!(
            watchers,
            vec![
                "usr-friend".to_string(),
                "usr-main".to_string(),
                "usr-other".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn dispatch_presence_event_locally_sends_payload_and_removes_stale_connections() {
        let state = AppState::new(
            "http://127.0.0.1:1".to_string(),
            vec!["http://localhost:3002".to_string()],
            "hexrelay-dev-presence-token-change-me".to_string(),
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
        let (open_tx, mut open_rx) = mpsc::channel::<String>(4);
        let (stale_tx, stale_rx) = mpsc::channel::<String>(1);
        drop(stale_rx);

        state.connection_senders.lock().await.insert(
            "usr-main".to_string(),
            std::collections::HashMap::from([
                (
                    "conn-open".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: open_tx,
                        device_id: Some("device-a".to_string()),
                    },
                ),
                (
                    "conn-stale".to_string(),
                    crate::state::ConnectionSenderEntry {
                        sender: stale_tx,
                        device_id: Some("device-b".to_string()),
                    },
                ),
            ]),
        );

        dispatch_presence_event_locally(
            &state,
            "payload-1",
            &["usr-main".to_string()],
            &[PresenceWatcherCursor {
                watcher_identity_id: "usr-main".to_string(),
                cursor: 4,
            }],
        )
        .await;

        assert_eq!(open_rx.recv().await.as_deref(), Some("payload-1"));
        let guard = state.connection_senders.lock().await;
        let connections = guard.get("usr-main").expect("remaining connections");
        assert!(connections.contains_key("conn-open"));
        assert!(!connections.contains_key("conn-stale"));
    }

    #[test]
    fn presence_dispatch_payload_parses_edge_request() {
        let payload = serde_json::to_vec(&PresenceDispatchEnvelope::Edge(
            PresenceEdgeDispatchRequest {
                identity_id: "usr-main",
                online: true,
            },
        ))
        .expect("encode presence payload");

        let dispatch =
            PresenceNodeDispatch::from_payload(&payload).expect("parse presence payload");
        assert_eq!(dispatch.identity_id, "usr-main");
        assert!(dispatch.online);
    }

    #[test]
    fn presence_dispatch_payload_rejects_unknown_kind() {
        let payload = br#"{"kind":"unknown","body":{"identity_id":"usr-main","online":true}}"#;

        assert!(matches!(
            PresenceNodeDispatch::from_payload(payload),
            Err(TransportError::SendFailed)
        ));
    }
}
