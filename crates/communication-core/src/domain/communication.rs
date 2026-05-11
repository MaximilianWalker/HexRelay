use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationMode {
    DmEnvelope,
    ServerChannel,
    Presence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProfile {
    NodeClient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectTarget {
    NodeEndpoint { endpoint: String },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommunicationReasonCode {
    DmEnvelopeNodeRouteSelected,
    ServerChannelRouteSelected,
    PresenceRouteSelected,
    ModeDisabled,
    TargetProfileMismatch,
    TransportConnectFailed,
    TransportSendFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmTransportPolicy {
    EncryptedEnvelopeNode,
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
            dm_transport_policy: DmTransportPolicy::EncryptedEnvelopeNode,
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
