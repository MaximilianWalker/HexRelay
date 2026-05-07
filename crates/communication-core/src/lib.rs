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
    connect_via_direct_peer, send_via_direct_peer_dispatch, send_via_node_dispatch,
    DirectPeerDispatch, DispatchingDirectPeerTransport, DispatchingNodeClientTransport,
    NodeClientTransport, NodeDispatch, TransportError, UnsupportedDirectPeerTransport,
    UnsupportedNodeClientTransport,
};

#[cfg(test)]
mod tests;
