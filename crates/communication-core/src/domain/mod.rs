mod communication;
mod mesh;
mod node;
mod peer_invite;
mod signature;

pub use communication::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DispatchOutcome,
    DmTransportPolicy, PolicyContext, PolicyError, SendEnvelope, SessionProvenance,
    TransportProfile,
};
pub use mesh::{
    CandidatePeerPolicy, PeerCandidate, PeerCandidateValidationError, PeerRouteKind,
    PeerRouteSelectionError, RouteSelectionPolicy, SelectedPeerRoute, StaticPeerRegistry,
    StaticPeerRegistryError,
};
pub use node::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, NodeDescriptor, NodeDescriptorValidationError, NodeRateLimit,
    NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RateLimitScope, RelayPolicy,
    StoragePolicy,
};
pub use peer_invite::{
    PeerInvite, PeerInviteEnvelope, PeerInviteValidationContext, PeerInviteValidationError,
};
pub use signature::{
    canonical_descriptor_signing_payload, canonical_peer_invite_signing_payload,
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, sign_peer_invite_ed25519_pkcs8,
    verify_descriptor_ed25519, verify_peer_invite_ed25519, Ed25519DescriptorVerifier,
    NodeDescriptorSignatureError, PeerInviteSignatureError,
};
