use crate::{
    app::{CommunicationError, CommunicationRouter, PolicyEngine},
    domain::{
        CommunicationMode, ConnectIntent, ConnectTarget, PolicyContext, SendEnvelope,
        SessionProvenance, TransportProfile,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    ConnectFailed,
    SendFailed,
}

pub trait DirectPeerTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError>;
    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError>;
}

pub trait DirectPeerDispatch {
    fn connect_peer(&self, target: &ConnectTarget) -> Result<(), TransportError>;
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError>;
}

pub struct DispatchingDirectPeerTransport<D> {
    dispatch: D,
}

impl<D> DispatchingDirectPeerTransport<D> {
    pub fn new(dispatch: D) -> Self {
        Self { dispatch }
    }
}

impl<D> DirectPeerTransport for DispatchingDirectPeerTransport<D>
where
    D: DirectPeerDispatch,
{
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        if intent.mode != CommunicationMode::DmDirect {
            return Err(TransportError::ConnectFailed);
        }

        self.dispatch.connect_peer(&intent.target)?;

        Ok(PolicyEngine::build_provenance(
            intent.mode,
            TransportProfile::DirectPeer,
        ))
    }

    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError> {
        if envelope.mode != CommunicationMode::DmDirect {
            return Err(TransportError::SendFailed);
        }

        self.dispatch.send_payload(&envelope.payload)
    }
}

pub struct UnsupportedDirectPeerTransport;

impl DirectPeerTransport for UnsupportedDirectPeerTransport {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
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
    CommunicationRouter::new(
        policy,
        UnsupportedDirectPeerTransport,
        DispatchingNodeClientTransport::new(mode, dispatch),
    )
    .send(&SendEnvelope { mode, payload })
}

pub fn connect_via_direct_peer<D>(
    policy: PolicyContext,
    dispatch: D,
    target: ConnectTarget,
) -> Result<SessionProvenance, CommunicationError>
where
    D: DirectPeerDispatch,
{
    CommunicationRouter::new(
        policy,
        DispatchingDirectPeerTransport::new(dispatch),
        UnsupportedNodeClientTransport,
    )
    .connect(&ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target,
    })
}

pub fn send_via_direct_peer_dispatch<D>(
    policy: PolicyContext,
    dispatch: D,
    payload: Vec<u8>,
) -> Result<(), CommunicationError>
where
    D: DirectPeerDispatch,
{
    CommunicationRouter::new(
        policy,
        DispatchingDirectPeerTransport::new(dispatch),
        UnsupportedNodeClientTransport,
    )
    .send(&SendEnvelope {
        mode: CommunicationMode::DmDirect,
        payload,
    })
}
