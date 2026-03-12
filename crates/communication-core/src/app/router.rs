use crate::app::PolicyEngine;
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, PolicyContext, PolicyError,
    SendEnvelope, SessionProvenance, TransportProfile,
};
use crate::transport::{DirectPeerTransport, NodeClientTransport};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationError {
    pub code: CommunicationReasonCode,
    pub mode: CommunicationMode,
    pub profile: Option<TransportProfile>,
}

pub struct CommunicationRouter<D, N> {
    policy: PolicyContext,
    direct_peer: D,
    node_client: N,
}

impl<D, N> CommunicationRouter<D, N>
where
    D: DirectPeerTransport,
    N: NodeClientTransport,
{
    pub fn new(policy: PolicyContext, direct_peer: D, node_client: N) -> Self {
        Self {
            policy,
            direct_peer,
            node_client,
        }
    }

    pub fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, CommunicationError> {
        let profile = self.route_profile(intent.mode, Some(intent))?;

        match profile {
            TransportProfile::DirectPeer => self.direct_peer.connect(intent).map_err(|_| {
                transport_error(
                    CommunicationReasonCode::TransportConnectFailed,
                    intent.mode,
                    profile,
                )
            }),
            TransportProfile::NodeClient => self.node_client.connect(intent).map_err(|_| {
                transport_error(
                    CommunicationReasonCode::TransportConnectFailed,
                    intent.mode,
                    profile,
                )
            }),
        }
    }

    pub fn send(&self, envelope: &SendEnvelope) -> Result<(), CommunicationError> {
        let profile = self.route_profile(envelope.mode, None)?;

        match profile {
            TransportProfile::DirectPeer => self.direct_peer.send(envelope).map_err(|_| {
                transport_error(
                    CommunicationReasonCode::TransportSendFailed,
                    envelope.mode,
                    profile,
                )
            }),
            TransportProfile::NodeClient => self.node_client.send(envelope).map_err(|_| {
                transport_error(
                    CommunicationReasonCode::TransportSendFailed,
                    envelope.mode,
                    profile,
                )
            }),
        }
    }

    fn route_profile(
        &self,
        mode: CommunicationMode,
        intent: Option<&ConnectIntent>,
    ) -> Result<TransportProfile, CommunicationError> {
        let profile = PolicyEngine::route_mode(mode, &self.policy)
            .map_err(|error| map_policy_error(error, mode))?;

        if let Some(intent) = intent {
            PolicyEngine::validate_connect_intent(profile, intent)
                .map_err(|error| map_policy_error(error, mode))?;
        }

        assert_dm_direct_profile(mode, profile)?;

        Ok(profile)
    }
}

pub(crate) fn assert_dm_direct_profile(
    mode: CommunicationMode,
    profile: TransportProfile,
) -> Result<(), CommunicationError> {
    if mode == CommunicationMode::DmDirect && profile != TransportProfile::DirectPeer {
        return Err(CommunicationError {
            code: CommunicationReasonCode::DmDirectPolicyViolation,
            mode,
            profile: Some(profile),
        });
    }

    Ok(())
}

fn map_policy_error(error: PolicyError, mode: CommunicationMode) -> CommunicationError {
    match error {
        PolicyError::ModeDisabled { .. } => CommunicationError {
            code: CommunicationReasonCode::ModeDisabled,
            mode,
            profile: None,
        },
        PolicyError::TargetProfileMismatch { profile, .. } => CommunicationError {
            code: CommunicationReasonCode::TargetProfileMismatch,
            mode,
            profile: Some(profile),
        },
    }
}

fn transport_error(
    code: CommunicationReasonCode,
    mode: CommunicationMode,
    profile: TransportProfile,
) -> CommunicationError {
    CommunicationError {
        code,
        mode,
        profile: Some(profile),
    }
}
