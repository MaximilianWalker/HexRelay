pub mod app;
pub mod config;
pub mod domain;
pub mod transport;

pub use app::PolicyEngine;
pub use config::CommunicationConfig;
pub use domain::{
    CandidatePeerPolicy, PeerCandidate, PeerCandidateValidationError, StaticPeerRegistry,
    StaticPeerRegistryError,
};
pub use domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};
pub use domain::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, NodeDescriptor, NodeDescriptorValidationError, NodeRateLimit,
    NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RateLimitScope, RelayPolicy,
    StoragePolicy,
};
pub use transport::{
    send_via_node_dispatch, DispatchingNodeClientTransport, NodeClientTransport, NodeDispatch,
    TransportError, UnsupportedNodeClientTransport,
};

#[cfg(test)]
mod tests;
