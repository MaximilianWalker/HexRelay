use communication_core::{
    app::CommunicationRouter,
    domain::{
        CommunicationMode, CommunicationReasonCode, ConnectIntent, SendEnvelope, SessionProvenance,
        TransportProfile,
    },
    transport::{DirectPeerTransport, NodeClientTransport, TransportError},
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::state::AppState;

const INTERNAL_CHANNEL_MESSAGE_CREATED_PATH: &str = "/internal/channels/messages/created";
const INTERNAL_CHANNEL_MESSAGE_UPDATED_PATH: &str = "/internal/channels/messages/updated";
const INTERNAL_CHANNEL_MESSAGE_DELETED_PATH: &str = "/internal/channels/messages/deleted";

#[derive(Serialize)]
struct ChannelMessageCreatedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    sender_id: &'a str,
    created_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

#[derive(Serialize)]
struct ChannelMessageUpdatedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    editor_id: &'a str,
    edited_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

#[derive(Serialize)]
struct ChannelMessageDeletedDispatchRequest<'a> {
    message_id: &'a str,
    guild_id: &'a str,
    channel_id: &'a str,
    deleted_by: &'a str,
    deleted_at: &'a str,
    channel_seq: u64,
    recipients: &'a [String],
}

#[derive(Deserialize, Serialize)]
struct OwnedChannelMessageCreatedDispatchRequest {
    message_id: String,
    guild_id: String,
    channel_id: String,
    sender_id: String,
    created_at: String,
    channel_seq: u64,
    recipients: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct OwnedChannelMessageUpdatedDispatchRequest {
    message_id: String,
    guild_id: String,
    channel_id: String,
    editor_id: String,
    edited_at: String,
    channel_seq: u64,
    recipients: Vec<String>,
}

#[derive(Deserialize, Serialize)]
struct OwnedChannelMessageDeletedDispatchRequest {
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
        let path = dispatch.path().to_string();
        let message_id = dispatch.message_id().to_string();
        let guild_id = dispatch.guild_id().to_string();
        let channel_id = dispatch.channel_id().to_string();
        let internal_token = self.internal_token.clone();
        let body = dispatch.body().to_vec();
        let handle =
            tokio::runtime::Handle::try_current().map_err(|_| TransportError::SendFailed)?;
        handle.spawn(async move {
            match http_client
                .post(url)
                .header("x-hexrelay-internal-token", internal_token)
                .header("content-type", "application/json")
                .body(body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {}
                Ok(response) => {
                    warn!(
                        %path,
                        %message_id,
                        %guild_id,
                        %channel_id,
                        status = %response.status(),
                        "NodeClientTransport server-channel dispatch failed"
                    );
                }
                Err(error) => {
                    warn!(
                        %path,
                        %message_id,
                        %guild_id,
                        %channel_id,
                        error = %error,
                        "NodeClientTransport server-channel dispatch errored"
                    );
                }
            }
        });

        Ok(())
    }
}

enum RealtimeNodeDispatch {
    Created(DispatchPayload),
    Updated(DispatchPayload),
    Deleted(DispatchPayload),
}

struct DispatchPayload {
    body: Vec<u8>,
    message_id: String,
    guild_id: String,
    channel_id: String,
}

impl RealtimeNodeDispatch {
    fn from_payload(payload: &[u8]) -> Result<Self, TransportError> {
        let envelope: OwnedRealtimeNodeDispatchEnvelope =
            serde_json::from_slice(payload).map_err(|_| TransportError::SendFailed)?;
        match envelope {
            OwnedRealtimeNodeDispatchEnvelope::Created(body) => {
                let message_id = body.message_id.clone();
                let guild_id = body.guild_id.clone();
                let channel_id = body.channel_id.clone();
                Ok(Self::Created(DispatchPayload::new(
                    &body, message_id, guild_id, channel_id,
                )?))
            }
            OwnedRealtimeNodeDispatchEnvelope::Updated(body) => {
                let message_id = body.message_id.clone();
                let guild_id = body.guild_id.clone();
                let channel_id = body.channel_id.clone();
                Ok(Self::Updated(DispatchPayload::new(
                    &body, message_id, guild_id, channel_id,
                )?))
            }
            OwnedRealtimeNodeDispatchEnvelope::Deleted(body) => {
                let message_id = body.message_id.clone();
                let guild_id = body.guild_id.clone();
                let channel_id = body.channel_id.clone();
                Ok(Self::Deleted(DispatchPayload::new(
                    &body, message_id, guild_id, channel_id,
                )?))
            }
        }
    }

    fn path(&self) -> &'static str {
        match self {
            Self::Created(_) => INTERNAL_CHANNEL_MESSAGE_CREATED_PATH,
            Self::Updated(_) => INTERNAL_CHANNEL_MESSAGE_UPDATED_PATH,
            Self::Deleted(_) => INTERNAL_CHANNEL_MESSAGE_DELETED_PATH,
        }
    }

    fn body(&self) -> &[u8] {
        match self {
            Self::Created(payload) | Self::Updated(payload) | Self::Deleted(payload) => {
                &payload.body
            }
        }
    }

    fn message_id(&self) -> &str {
        match self {
            Self::Created(payload) | Self::Updated(payload) | Self::Deleted(payload) => {
                &payload.message_id
            }
        }
    }

    fn guild_id(&self) -> &str {
        match self {
            Self::Created(payload) | Self::Updated(payload) | Self::Deleted(payload) => {
                &payload.guild_id
            }
        }
    }

    fn channel_id(&self) -> &str {
        match self {
            Self::Created(payload) | Self::Updated(payload) | Self::Deleted(payload) => {
                &payload.channel_id
            }
        }
    }
}

impl DispatchPayload {
    fn new<T>(
        body: &T,
        message_id: String,
        guild_id: String,
        channel_id: String,
    ) -> Result<Self, TransportError>
    where
        T: Serialize,
    {
        Ok(Self {
            body: serde_json::to_vec(body).map_err(|_| TransportError::SendFailed)?,
            message_id,
            guild_id,
            channel_id,
        })
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "body")]
enum RealtimeNodeDispatchEnvelope<'a> {
    #[serde(rename = "channel_message_created")]
    Created(ChannelMessageCreatedDispatchRequest<'a>),
    #[serde(rename = "channel_message_updated")]
    Updated(ChannelMessageUpdatedDispatchRequest<'a>),
    #[serde(rename = "channel_message_deleted")]
    Deleted(ChannelMessageDeletedDispatchRequest<'a>),
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "body")]
enum OwnedRealtimeNodeDispatchEnvelope {
    #[serde(rename = "channel_message_created")]
    Created(OwnedChannelMessageCreatedDispatchRequest),
    #[serde(rename = "channel_message_updated")]
    Updated(OwnedChannelMessageUpdatedDispatchRequest),
    #[serde(rename = "channel_message_deleted")]
    Deleted(OwnedChannelMessageDeletedDispatchRequest),
}

pub async fn dispatch_channel_message_created(
    state: &AppState,
    input: DispatchChannelMessageCreatedInput<'_>,
) -> Result<(), String> {
    dispatch_server_channel_payload(
        state,
        &RealtimeNodeDispatchEnvelope::Created(ChannelMessageCreatedDispatchRequest {
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            sender_id: input.sender_id,
            created_at: input.created_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
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
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            editor_id: input.editor_id,
            edited_at: input.edited_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
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
            message_id: input.message_id,
            guild_id: input.server_id,
            channel_id: input.channel_id,
            deleted_by: input.deleted_by,
            deleted_at: input.deleted_at,
            channel_seq: input.channel_seq,
            recipients: input.recipients,
        }),
    )
}

fn dispatch_server_channel_payload(
    state: &AppState,
    envelope: &RealtimeNodeDispatchEnvelope<'_>,
) -> Result<(), String> {
    let payload = serde_json::to_vec(envelope)
        .map_err(|error| format!("encode server channel dispatch payload: {error}"))?;

    let router = CommunicationRouter::new(
        communication_core::PolicyContext::default(),
        UnusedDirectPeerTransport,
        RealtimeNodeClientTransport {
            http_client: state.http_client.clone(),
            realtime_base_url: state.realtime_base_url.clone(),
            internal_token: state.channel_dispatch_internal_token.clone(),
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
        let recipients = vec!["usr-1".to_string()];
        let payload = serde_json::to_vec(&RealtimeNodeDispatchEnvelope::Created(
            ChannelMessageCreatedDispatchRequest {
                message_id: "msg-1",
                guild_id: "guild-1",
                channel_id: "channel-1",
                sender_id: "usr-1",
                created_at: "2026-03-26T00:00:00Z",
                channel_seq: 1,
                recipients: &recipients,
            },
        ))
        .expect("encode payload");

        let dispatch =
            RealtimeNodeDispatch::from_payload(&payload).expect("parse dispatch payload");
        assert_eq!(dispatch.path(), INTERNAL_CHANNEL_MESSAGE_CREATED_PATH);
        let body_value: serde_json::Value =
            serde_json::from_slice(dispatch.body()).expect("parse dispatch body as json");
        assert_eq!(
            body_value,
            serde_json::json!({
                "message_id": "msg-1",
                "guild_id": "guild-1",
                "channel_id": "channel-1",
                "sender_id": "usr-1",
                "created_at": "2026-03-26T00:00:00Z",
                "channel_seq": 1,
                "recipients": ["usr-1"]
            })
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
