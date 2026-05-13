pub mod app;
pub mod config;
pub mod domain;
pub mod transport;

pub use app::PolicyEngine;
pub use config::CommunicationConfig;
pub use domain::{
    canonical_descriptor_signing_payload, canonical_peer_invite_signing_payload,
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, sign_peer_invite_ed25519_pkcs8,
    verify_descriptor_ed25519, verify_peer_invite_ed25519, Ed25519DescriptorVerifier,
    NodeDescriptorSignatureError, PeerInviteSignatureError,
};
pub use domain::{
    ed25519_public_key_base64, sign_dm_session_bootstrap_ed25519_pkcs8,
    verify_dm_session_bootstrap_ed25519, DmCiphertextEnvelope, DmClientEncryptResult,
    DmClientSession, DmE2eeError, DmEphemeralPublicKey, DmEphemeralSecret, DmGroupRekeyPlan,
    DmGroupSecret, DmSessionBootstrap, DmSessionContext, DmSessionKey, DmSessionKind,
    DmSessionRotationState, DM_SESSION_KEY_BYTES, DM_SESSION_NONCE_BYTES,
    DM_SESSION_ROTATE_AFTER_MESSAGES, DM_SESSION_ROTATE_AFTER_SECONDS,
};
pub use domain::{
    CandidatePeerPolicy, PeerCandidate, PeerCandidateValidationError, PeerRouteKind,
    PeerRouteSelectionError, RouteSelectionPolicy, SelectedPeerRoute, StaticPeerRegistry,
    StaticPeerRegistryError,
};
pub use domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DispatchOutcome,
    DmTransportPolicy, PolicyContext, PolicyError, SendEnvelope, SessionProvenance,
    TransportProfile,
};
pub use domain::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, NodeDescriptor, NodeDescriptorValidationError, NodeRateLimit,
    NodeSignature, NodeSignatureAlgorithm, PeerInvite, PeerInviteEnvelope,
    PeerInviteValidationContext, PeerInviteValidationError, PeeringPolicy, RateLimitScope,
    RelayPolicy, StoragePolicy,
};
pub use transport::{
    send_via_node_dispatch, send_via_node_dispatch_with_provenance, DispatchingNodeClientTransport,
    NodeClientTransport, NodeDispatch, TransportError, UnsupportedNodeClientTransport,
};

#[cfg(test)]
mod tests;
