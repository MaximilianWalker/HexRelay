use communication_core::{
    app::CommunicationRouter,
    domain::{
        CommunicationMode, CommunicationReasonCode, ConnectIntent, SendEnvelope, SessionProvenance,
        TransportProfile,
    },
    transport::{DirectPeerTransport, NodeClientTransport, TransportError},
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

const INTERNAL_CHANNEL_MESSAGE_CREATED_PATH: &str = "/internal/channels/messages/created";
const INTERNAL_CHANNEL_MESSAGE_UPDATED_PATH: &str = "/internal/channels/messages/updated";
const INTERNAL_CHANNEL_MESSAGE_DELETED_PATH: &str = "/internal/channels/messages/deleted";

#[derive(Serialize, Deserialize)]
struct ChannelMessageCreatedDispatchRequest {
    message_id: String,
    guild_id: String,
    channel_id: String,
    sender_id: String,
    created_at: String,
    channel_seq: u64,
    recipients: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ChannelMessageUpdatedDispatchRequest {
    message_id: String,
    guild_id: String,
    channel_id: String,
    editor_id: String,
    edited_at: String,
    channel_seq: u64,
    recipients: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ChannelMessageDeletedDispatchRequest {
    message_id: String,
    guild_id: String,
    channel_id: String,
    deleted_by: String,
    deleted_at: String,
    channel_seq: u64,
    recipients: Vec<String>,
}

pub struct DispatchChannelMessageCreatedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub sender_id: &'a str,
    pub created_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

pub struct DispatchChannelMessageUpdatedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub editor_id: &'a str,
    pub edited_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

pub struct DispatchChannelMessageDeletedInput<'a> {
    pub server_id: &'a str,
    pub channel_id: &'a str,
    pub message_id: &'a str,
    pub deleted_by: &'a str,
    pub deleted_at: &'a str,
    pub channel_seq: u64,
    pub recipients: &'a [String],
}

#[derive(Clone)]
struct RealtimeNodeClientTransport {
    http_client: reqwest::Client,
    realtime_base_url: String,
    internal_token: String,
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

impl NodeClientTransport for RealtimeNodeClientTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::NodeClient,
            reason_code: match intent.mode {
                CommunicationMode::ServerChannel => {
                    CommunicationReasonCode::ServerChannelRouteSelected
                }
                CommunicationMode::Presence => CommunicationReasonCode::PresenceRouteSelected,
                CommunicationMode::DmDirect => CommunicationReasonCode::DmDirectPolicyViolation,
            },
            policy_assertions: vec!["node_client_transport_selected".to_string()],
        })
    }

    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError> {
        let dispatch = RealtimeNodeDispatch::from_payload(&envelope.payload)?;
        let http_client = self.http_client.clone();
        let url = format!(
            "{}{}",
            self.realtime_base_url.trim_end_matches('/'),
            dispatch.path()
        );
        let internal_token = self.internal_token.clone();
        let body = dispatch.body();
        let response = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| TransportError::SendFailed)?;
            runtime.block_on(async move {
                http_client
                    .post(url)
                    .header("x-hexrelay-internal-token", internal_token)
                    .header("content-type", "application/json")
                    .body(body)
                    .send()
                    .await
                    .map_err(|_| TransportError::SendFailed)
            })
        })
        .join()
        .map_err(|_| TransportError::SendFailed)??;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(TransportError::SendFailed)
        }
    }
}

enum RealtimeNodeDispatch {
    Created(Vec<u8>),
    Updated(Vec<u8>),
    Deleted(Vec<u8>),
}

impl RealtimeNodeDispatch {
    fn from_payload(payload: &[u8]) -> Result<Self, TransportError> {
        let envelope: RealtimeNodeDispatchEnvelope =
            serde_json::from_slice(payload).map_err(|_| TransportError::SendFailed)?;
        match envelope {
            RealtimeNodeDispatchEnvelope::Created(body) => Ok(Self::Created(
                serde_json::to_vec(&body).map_err(|_| TransportError::SendFailed)?,
            )),
            RealtimeNodeDispatchEnvelope::Updated(body) => Ok(Self::Updated(
                serde_json::to_vec(&body).map_err(|_| TransportError::SendFailed)?,
            )),
            RealtimeNodeDispatchEnvelope::Deleted(body) => Ok(Self::Deleted(
                serde_json::to_vec(&body).map_err(|_| TransportError::SendFailed)?,
            )),
        }
    }

    fn path(&self) -> &'static str {
        match self {
            Self::Created(_) => INTERNAL_CHANNEL_MESSAGE_CREATED_PATH,
            Self::Updated(_) => INTERNAL_CHANNEL_MESSAGE_UPDATED_PATH,
            Self::Deleted(_) => INTERNAL_CHANNEL_MESSAGE_DELETED_PATH,
        }
    }

    fn body(&self) -> Vec<u8> {
        match self {
            Self::Created(body) | Self::Updated(body) | Self::Deleted(body) => body.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", content = "body")]
enum RealtimeNodeDispatchEnvelope {
    #[serde(rename = "channel_message_created")]
    Created(ChannelMessageCreatedDispatchRequest),
    #[serde(rename = "channel_message_updated")]
    Updated(ChannelMessageUpdatedDispatchRequest),
    #[serde(rename = "channel_message_deleted")]
    Deleted(ChannelMessageDeletedDispatchRequest),
}

pub async fn dispatch_channel_message_created(
    state: &AppState,
    input: DispatchChannelMessageCreatedInput<'_>,
) -> Result<(), String> {
    dispatch_server_channel_payload(
        state,
        &RealtimeNodeDispatchEnvelope::Created(ChannelMessageCreatedDispatchRequest {
            message_id: input.message_id.to_string(),
            guild_id: input.server_id.to_string(),
            channel_id: input.channel_id.to_string(),
            sender_id: input.sender_id.to_string(),
            created_at: input.created_at.to_string(),
            channel_seq: input.channel_seq,
            recipients: input.recipients.to_vec(),
        }),
    )
}

pub async fn dispatch_channel_message_updated(
    state: &AppState,
    input: DispatchChannelMessageUpdatedInput<'_>,
) -> Result<(), String> {
    dispatch_server_channel_payload(
        state,
        &RealtimeNodeDispatchEnvelope::Updated(ChannelMessageUpdatedDispatchRequest {
            message_id: input.message_id.to_string(),
            guild_id: input.server_id.to_string(),
            channel_id: input.channel_id.to_string(),
            editor_id: input.editor_id.to_string(),
            edited_at: input.edited_at.to_string(),
            channel_seq: input.channel_seq,
            recipients: input.recipients.to_vec(),
        }),
    )
}

pub async fn dispatch_channel_message_deleted(
    state: &AppState,
    input: DispatchChannelMessageDeletedInput<'_>,
) -> Result<(), String> {
    dispatch_server_channel_payload(
        state,
        &RealtimeNodeDispatchEnvelope::Deleted(ChannelMessageDeletedDispatchRequest {
            message_id: input.message_id.to_string(),
            guild_id: input.server_id.to_string(),
            channel_id: input.channel_id.to_string(),
            deleted_by: input.deleted_by.to_string(),
            deleted_at: input.deleted_at.to_string(),
            channel_seq: input.channel_seq,
            recipients: input.recipients.to_vec(),
        }),
    )
}

fn dispatch_server_channel_payload(
    state: &AppState,
    envelope: &RealtimeNodeDispatchEnvelope,
) -> Result<(), String> {
    let payload = serde_json::to_vec(envelope)
        .map_err(|error| format!("encode server channel dispatch payload: {error}"))?;

    let router = CommunicationRouter::new(
        communication_core::PolicyContext::default(),
        UnusedDirectPeerTransport,
        RealtimeNodeClientTransport {
            http_client: state.http_client.clone(),
            realtime_base_url: state.realtime_base_url.clone(),
            internal_token: state.presence_internal_token.clone(),
        },
    );

    router
        .send(&SendEnvelope {
            mode: CommunicationMode::ServerChannel,
            payload,
        })
        .map_err(|error| {
            format!(
                "dispatch server channel event via NodeClientTransport: {:?}",
                error.code
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_payload_maps_created_kind_to_internal_path() {
        let payload = serde_json::to_vec(&RealtimeNodeDispatchEnvelope::Created(
            ChannelMessageCreatedDispatchRequest {
                message_id: "msg-1".to_string(),
                guild_id: "guild-1".to_string(),
                channel_id: "channel-1".to_string(),
                sender_id: "usr-1".to_string(),
                created_at: "2026-03-26T00:00:00Z".to_string(),
                channel_seq: 1,
                recipients: vec!["usr-1".to_string()],
            },
        ))
        .expect("encode payload");

        let dispatch =
            RealtimeNodeDispatch::from_payload(&payload).expect("parse dispatch payload");
        assert_eq!(dispatch.path(), INTERNAL_CHANNEL_MESSAGE_CREATED_PATH);
        assert_eq!(
            dispatch.body(),
            br#"{"message_id":"msg-1","guild_id":"guild-1","channel_id":"channel-1","sender_id":"usr-1","created_at":"2026-03-26T00:00:00Z","channel_seq":1,"recipients":["usr-1"]}"#.to_vec()
        );
    }

    #[test]
    fn dispatch_payload_rejects_unknown_kind() {
        let payload = br#"{"kind":"unknown","body":{"message_id":"msg-1"}}"#;

        assert!(matches!(
            RealtimeNodeDispatch::from_payload(payload),
            Err(TransportError::SendFailed)
        ));
    }
}
