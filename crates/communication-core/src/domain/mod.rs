mod communication;
mod e2ee;
mod mesh;
mod peer_invite;
mod server;
mod signature;

pub use communication::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DispatchOutcome,
    DmTransportPolicy, PolicyContext, PolicyError, SendEnvelope, SessionProvenance,
    TransportProfile,
};
pub use e2ee::{
    ed25519_public_key_base64, sign_dm_session_bootstrap_ed25519_pkcs8,
    verify_dm_session_bootstrap_ed25519, DmCiphertextEnvelope, DmClientEncryptResult,
    DmClientSession, DmE2eeError, DmEphemeralPublicKey, DmEphemeralSecret, DmGroupRekeyPlan,
    DmGroupSecret, DmGroupSessionBootstrap, DmGroupSessionRing, DmOneToOneRotationPlan,
    DmSessionBootstrap, DmSessionContext, DmSessionKey, DmSessionKind, DmSessionRotationState,
    DM_SESSION_KEY_BYTES, DM_SESSION_NONCE_BYTES, DM_SESSION_ROTATE_AFTER_MESSAGES,
    DM_SESSION_ROTATE_AFTER_SECONDS,
};
pub use mesh::{
    CandidatePeerPolicy, PeerCandidate, PeerCandidateValidationError, PeerRouteKind,
    PeerRouteSelectionError, RouteSelectionPolicy, SelectedPeerRoute, StaticPeerRegistry,
    StaticPeerRegistryError,
};
pub use peer_invite::{
    PeerInvite, PeerInviteEnvelope, PeerInviteValidationContext, PeerInviteValidationError,
};
pub use server::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, PeeringPolicy, RateLimitScope, RelayPolicy, ServerDescriptor,
    ServerDescriptorValidationError, ServerRateLimit, ServerSignature, ServerSignatureAlgorithm,
    StoragePolicy,
};
pub use signature::{
    canonical_descriptor_signing_payload, canonical_peer_invite_signing_payload,
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, sign_peer_invite_ed25519_pkcs8,
    verify_descriptor_ed25519, verify_peer_invite_ed25519, Ed25519DescriptorVerifier,
    PeerInviteSignatureError, ServerDescriptorSignatureError,
};
