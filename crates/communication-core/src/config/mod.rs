use crate::domain::{DmTransportPolicy, PolicyContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationConfig {
    pub dm_transport_policy: DmTransportPolicy,
    pub enable_server_channel: bool,
    pub enable_presence: bool,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            dm_transport_policy: DmTransportPolicy::DirectOnly,
            enable_server_channel: true,
            enable_presence: true,
        }
    }
}

impl CommunicationConfig {
    pub fn policy_context(&self) -> PolicyContext {
        PolicyContext {
            dm_transport_policy: self.dm_transport_policy,
            enable_server_channel: self.enable_server_channel,
            enable_presence: self.enable_presence,
        }
    }
}
