use crate::app::PolicyEngine;
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, DispatchOutcome, PolicyContext,
    PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};
use crate::transport::NodeClientTransport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationError {
    pub code: CommunicationReasonCode,
    pub mode: CommunicationMode,
    pub profile: Option<TransportProfile>,
}

pub struct CommunicationRouter<N> {
    policy: PolicyContext,
    node_client: N,
}

impl<N> CommunicationRouter<N>
where
    N: NodeClientTransport,
{
    pub fn new(policy: PolicyContext, node_client: N) -> Self {
        Self {
            policy,
            node_client,
        }
    }

    pub fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, CommunicationError> {
        let profile = self.route_profile(intent.mode, Some(intent))?;

        match profile {
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
        self.send_with_profile(envelope, profile)
    }

    pub fn send_with_provenance(
        &self,
        envelope: &SendEnvelope,
    ) -> Result<DispatchOutcome, CommunicationError> {
        let profile = self.route_profile(envelope.mode, None)?;
        self.send_with_profile(envelope, profile)?;

        Ok(DispatchOutcome {
            provenance: PolicyEngine::build_provenance(envelope.mode, profile),
        })
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
        Ok(profile)
    }

    fn send_with_profile(
        &self,
        envelope: &SendEnvelope,
        profile: TransportProfile,
    ) -> Result<(), CommunicationError> {
        match profile {
            TransportProfile::NodeClient => self.node_client.send(envelope).map_err(|_| {
                transport_error(
                    CommunicationReasonCode::TransportSendFailed,
                    envelope.mode,
                    profile,
                )
            }),
        }
    }
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
