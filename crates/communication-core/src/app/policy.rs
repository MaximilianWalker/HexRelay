use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SessionProvenance, TransportProfile,
};

pub struct PolicyEngine;

impl PolicyEngine {
    pub fn route_mode(
        mode: CommunicationMode,
        policy: &PolicyContext,
    ) -> Result<TransportProfile, PolicyError> {
        match mode {
            CommunicationMode::DmDirect => match policy.dm_transport_policy {
                DmTransportPolicy::DirectOnly => Ok(TransportProfile::DirectPeer),
            },
            CommunicationMode::ServerChannel => {
                if policy.enable_server_channel {
                    Ok(TransportProfile::NodeClient)
                } else {
                    Err(PolicyError::ModeDisabled { mode })
                }
            }
            CommunicationMode::Presence => {
                if policy.enable_presence {
                    Ok(TransportProfile::NodeClient)
                } else {
                    Err(PolicyError::ModeDisabled { mode })
                }
            }
        }
    }

    pub fn validate_connect_intent(
        profile: TransportProfile,
        intent: &ConnectIntent,
    ) -> Result<(), PolicyError> {
        match (profile, &intent.target) {
            (TransportProfile::DirectPeer, ConnectTarget::PeerIdentity { .. }) => Ok(()),
            (TransportProfile::NodeClient, ConnectTarget::NodeEndpoint { .. }) => Ok(()),
            (profile, target) => Err(PolicyError::TargetProfileMismatch {
                profile,
                target: target.clone(),
            }),
        }
    }

    pub fn build_provenance(
        mode: CommunicationMode,
        profile: TransportProfile,
    ) -> SessionProvenance {
        let (reason_code, assertion) = match (mode, profile) {
            (CommunicationMode::DmDirect, TransportProfile::DirectPeer) => (
                CommunicationReasonCode::DmDirectRouteSelected,
                "dm_direct_policy_compliant",
            ),
            (CommunicationMode::ServerChannel, TransportProfile::NodeClient) => (
                CommunicationReasonCode::ServerChannelRouteSelected,
                "server_channel_policy_compliant",
            ),
            (CommunicationMode::Presence, TransportProfile::NodeClient) => (
                CommunicationReasonCode::PresenceRouteSelected,
                "presence_policy_compliant",
            ),
            _ => (
                CommunicationReasonCode::TargetProfileMismatch,
                "policy_violation",
            ),
        };

        SessionProvenance {
            mode,
            profile,
            reason_code,
            policy_assertions: vec![assertion.to_string()],
        }
    }
}
