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

pub trait ServerClientTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError>;
    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError>;
}

pub struct UnsupportedServerClientTransport;

impl ServerClientTransport for UnsupportedServerClientTransport {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

pub trait ServerDispatch {
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError>;
}

pub struct DispatchingServerClientTransport<D> {
    mode: CommunicationMode,
    dispatch: D,
}

impl<D> DispatchingServerClientTransport<D> {
    pub fn new(mode: CommunicationMode, dispatch: D) -> Self {
        Self { mode, dispatch }
    }
}

impl<D> ServerClientTransport for DispatchingServerClientTransport<D>
where
    D: ServerDispatch,
{
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        if intent.mode != self.mode {
            return Err(TransportError::ConnectFailed);
        }

        Ok(PolicyEngine::build_provenance(
            intent.mode,
            TransportProfile::ServerClient,
        ))
    }

    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError> {
        if envelope.mode != self.mode {
            return Err(TransportError::SendFailed);
        }

        self.dispatch.send_payload(&envelope.payload)
    }
}

pub fn send_via_server_dispatch<D>(
    mode: CommunicationMode,
    policy: PolicyContext,
    dispatch: D,
    payload: Vec<u8>,
) -> Result<(), CommunicationError>
where
    D: ServerDispatch,
{
    CommunicationRouter::new(
        policy,
        DispatchingServerClientTransport::new(mode, dispatch),
    )
    .send(&SendEnvelope { mode, payload })
}

pub fn send_via_server_dispatch_with_provenance<D>(
    mode: CommunicationMode,
    policy: PolicyContext,
    dispatch: D,
    payload: Vec<u8>,
) -> Result<DispatchOutcome, CommunicationError>
where
    D: ServerDispatch,
{
    CommunicationRouter::new(
        policy,
        DispatchingServerClientTransport::new(mode, dispatch),
    )
    .send_with_provenance(&SendEnvelope { mode, payload })
}
