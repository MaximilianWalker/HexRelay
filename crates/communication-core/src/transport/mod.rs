use crate::domain::{ConnectIntent, SendEnvelope, SessionProvenance};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    ConnectFailed,
    SendFailed,
}

pub trait DirectPeerTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError>;
    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError>;
}

pub trait NodeClientTransport {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError>;
    fn send(&self, envelope: &SendEnvelope) -> Result<(), TransportError>;
}
