use crate::{
    app::{CommunicationError, CommunicationRouter, PolicyEngine},
    domain::{
        CommunicationMode, ConnectIntent, DispatchOutcome, PolicyContext, SendEnvelope,
        SessionProvenance, TransportProfile,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    ConnectFailed,
    SendFailed,
}

pub trait NodeClientTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError>;
    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError>;
}

pub struct UnsupportedNodeClientTransport;

impl NodeClientTransport for UnsupportedNodeClientTransport {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

pub trait NodeDispatch {
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError>;
}

pub struct DispatchingNodeClientTransport<D> {
    mode: CommunicationMode,
    dispatch: D,
}

impl<D> DispatchingNodeClientTransport<D> {
    pub fn new(mode: CommunicationMode, dispatch: D) -> Self {
        Self { mode, dispatch }
    }
}

impl<D> NodeClientTransport for DispatchingNodeClientTransport<D>
where
    D: NodeDispatch,
{
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        if intent.mode != self.mode {
            return Err(TransportError::ConnectFailed);
        }

        Ok(PolicyEngine::build_provenance(
            intent.mode,
            TransportProfile::NodeClient,
        ))
    }

    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError> {
        if envelope.mode != self.mode {
            return Err(TransportError::SendFailed);
        }

        self.dispatch.send_payload(&envelope.payload)
    }
}

pub fn send_via_node_dispatch<D>(
    mode: CommunicationMode,
    policy: PolicyContext,
    dispatch: D,
    payload: Vec<u8>,
) -> Result<(), CommunicationError>
where
    D: NodeDispatch,
{
    send_via_node_dispatch_with_provenance(mode, policy, dispatch, payload).map(|_| ())
}

pub fn send_via_node_dispatch_with_provenance<D>(
    mode: CommunicationMode,
    policy: PolicyContext,
    dispatch: D,
    payload: Vec<u8>,
) -> Result<DispatchOutcome, CommunicationError>
where
    D: NodeDispatch,
{
    CommunicationRouter::new(policy, DispatchingNodeClientTransport::new(mode, dispatch))
        .send_with_provenance(&SendEnvelope { mode, payload })
}
