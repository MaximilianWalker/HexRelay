mod communication;
mod mesh;
mod node;
mod signature;

pub use communication::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};
pub use mesh::{
    CandidatePeerPolicy, PeerCandidate, PeerCandidateValidationError, StaticPeerRegistry,
    StaticPeerRegistryError,
};
pub use node::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, NodeDescriptor, NodeDescriptorValidationError, NodeRateLimit,
    NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RateLimitScope, RelayPolicy,
    StoragePolicy,
};
pub use signature::{
    canonical_descriptor_signing_payload, ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8,
    verify_descriptor_ed25519, Ed25519DescriptorVerifier, NodeDescriptorSignatureError,
};
