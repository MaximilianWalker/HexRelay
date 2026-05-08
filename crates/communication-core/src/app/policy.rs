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
            CommunicationMode::DmEnvelope => match policy.dm_transport_policy {
                DmTransportPolicy::EncryptedEnvelopeNode => Ok(TransportProfile::NodeClient),
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
        let TransportProfile::NodeClient = profile;
        let ConnectTarget::NodeEndpoint { .. } = &intent.target;
        Ok(())
    }

    pub fn build_provenance(
        mode: CommunicationMode,
        profile: TransportProfile,
    ) -> SessionProvenance {
        let TransportProfile::NodeClient = profile;
        let (reason_code, assertion) = match mode {
            CommunicationMode::DmEnvelope => (
                CommunicationReasonCode::DmEnvelopeNodeRouteSelected,
                "dm_envelope_node_policy_compliant",
            ),
            CommunicationMode::ServerChannel => (
                CommunicationReasonCode::ServerChannelRouteSelected,
                "server_channel_policy_compliant",
            ),
            CommunicationMode::Presence => (
                CommunicationReasonCode::PresenceRouteSelected,
                "presence_policy_compliant",
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
