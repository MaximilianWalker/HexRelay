pub mod app;
pub mod config;
pub mod domain;
pub mod transport;

pub use app::PolicyEngine;
pub use config::CommunicationConfig;
pub use domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};
pub use transport::{
    DispatchingNodeClientTransport, NodeClientTransport, NodeDispatch, TransportError,
    UnsupportedDirectPeerTransport,
};

#[cfg(test)]
mod tests;
