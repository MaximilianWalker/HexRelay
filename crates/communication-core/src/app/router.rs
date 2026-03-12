use crate::app::PolicyEngine;
use crate::domain::{
    CommunicationMode, ConnectIntent, PolicyContext, SendEnvelope, SessionProvenance,
    TransportProfile,
};
use crate::transport::{DirectPeerTransport, NodeClientTransport, TransportError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationError {
    PolicyViolation,
    TransportFailure,
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
            TransportProfile::DirectPeer => self
                .direct_peer
                .connect(intent)
                .map_err(map_transport_error),
            TransportProfile::NodeClient => self
                .node_client
                .connect(intent)
                .map_err(map_transport_error),
        }
    }

    pub fn send(&self, envelope: &SendEnvelope) -> Result<(), CommunicationError> {
        let profile = self.route_profile(envelope.mode, None)?;

        match profile {
            TransportProfile::DirectPeer => {
                self.direct_peer.send(envelope).map_err(map_transport_error)
            }
            TransportProfile::NodeClient => {
                self.node_client.send(envelope).map_err(map_transport_error)
            }
        }
    }

    fn route_profile(
        &self,
        mode: CommunicationMode,
        intent: Option<&ConnectIntent>,
    ) -> Result<TransportProfile, CommunicationError> {
        let profile = PolicyEngine::route_mode(mode, &self.policy)
            .map_err(|_| CommunicationError::PolicyViolation)?;

        if let Some(intent) = intent {
            PolicyEngine::validate_connect_intent(profile, intent)
                .map_err(|_| CommunicationError::PolicyViolation)?;
        }

        Ok(profile)
    }
}

fn map_transport_error(_: TransportError) -> CommunicationError {
    CommunicationError::TransportFailure
}
