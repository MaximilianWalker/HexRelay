use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMode {
    DmEnvelope,
    ServerChannel,
    Presence,
}

impl CommunicationMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DmEnvelope => "dm_envelope",
            Self::ServerChannel => "server_channel",
            Self::Presence => "presence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProfile {
    ServerClient,
}

impl TransportProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ServerClient => "server_client",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectTarget {
    ServerEndpoint { endpoint: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectIntent {
    pub mode: CommunicationMode,
    pub target: ConnectTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendEnvelope {
    pub mode: CommunicationMode,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionProvenance {
    pub mode: CommunicationMode,
    pub profile: TransportProfile,
    pub reason_code: CommunicationReasonCode,
    pub policy_assertions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchOutcome {
    pub provenance: SessionProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationReasonCode {
    DmEnvelopeServerRouteSelected,
    ServerChannelRouteSelected,
    PresenceRouteSelected,
    ModeDisabled,
    TargetProfileMismatch,
    TransportConnectFailed,
    TransportSendFailed,
}

impl CommunicationReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DmEnvelopeServerRouteSelected => "dm_envelope_server_route_selected",
            Self::ServerChannelRouteSelected => "server_channel_route_selected",
            Self::PresenceRouteSelected => "presence_route_selected",
            Self::ModeDisabled => "mode_disabled",
            Self::TargetProfileMismatch => "target_profile_mismatch",
            Self::TransportConnectFailed => "transport_connect_failed",
            Self::TransportSendFailed => "transport_send_failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmTransportPolicy {
    EncryptedEnvelopeServer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyContext {
    pub dm_transport_policy: DmTransportPolicy,
    pub enable_server_channel: bool,
    pub enable_presence: bool,
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self {
            dm_transport_policy: DmTransportPolicy::EncryptedEnvelopeServer,
            enable_server_channel: true,
            enable_presence: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyError {
    ModeDisabled {
        mode: CommunicationMode,
    },
    TargetProfileMismatch {
        profile: TransportProfile,
        target: ConnectTarget,
    },
}
