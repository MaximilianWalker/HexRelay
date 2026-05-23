use crate::domain::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DiscoveryPolicy,
    DmForwardingPolicy, NetworkMode, PeeringPolicy, RelayPolicy, ServerDescriptor,
    ServerDescriptorValidationError, ServerSignature, ServerSignatureAlgorithm, StoragePolicy,
};

struct StaticVerifier {
    valid: bool,
}

impl DescriptorSignatureVerifier for StaticVerifier {
    fn verify(&self, _descriptor: &ServerDescriptor) -> bool {
        self.valid
    }
}

fn validation_context() -> DescriptorValidationContext {
    DescriptorValidationContext {
        now_epoch_seconds: 1_000,
        max_ttl_seconds: 600,
        revoked_descriptor_ids: Vec::new(),
    }
}

fn descriptor() -> ServerDescriptor {
    ServerDescriptor {
        server_id: "server-a".to_string(),
        server_public_key: "ed25519-server-public-key".to_string(),
        descriptor_id: "descriptor-a".to_string(),
        issued_at_epoch_seconds: 1_000,
        expires_at_epoch_seconds: 1_300,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy: PeeringPolicy::InviteToken,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec!["https://server-a.example".to_string()],
        supported_protocols: vec!["hexrelay-server-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: Some("https://server-a.example/revocations".to_string()),
        signature: ServerSignature {
            algorithm: ServerSignatureAlgorithm::Ed25519,
            value: "signed-descriptor".to_string(),
        },
    }
}

#[test]
fn validates_private_descriptor_with_signature_verifier() {
    let descriptor = descriptor();

    let result =
        descriptor.validate_with_signature(&validation_context(), &StaticVerifier { valid: true });

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_expired_descriptor() {
    let mut descriptor = descriptor();
    descriptor.issued_at_epoch_seconds = 500;
    descriptor.expires_at_epoch_seconds = 999;

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DescriptorExpired)
    );
}

#[test]
fn rejects_descriptor_ttl_over_context_limit() {
    let mut descriptor = descriptor();
    descriptor.expires_at_epoch_seconds = 2_000;

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DescriptorTtlTooLong {
            ttl_seconds: 1_000,
            max_seconds: 600,
        })
    );
}

#[test]
fn rejects_revoked_descriptor() {
    let mut context = validation_context();
    context
        .revoked_descriptor_ids
        .push("descriptor-a".to_string());

    let result = descriptor().validate(&context);

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DescriptorRevoked)
    );
}

#[test]
fn rejects_forged_descriptor_signature() {
    let descriptor = descriptor();

    let result =
        descriptor.validate_with_signature(&validation_context(), &StaticVerifier { valid: false });

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::SignatureVerificationFailed)
    );
}

#[test]
fn rejects_hidden_descriptor_exposure_through_public_registry() {
    let mut descriptor = descriptor();
    descriptor.discovery_policy = DiscoveryPolicy::None;

    let result = descriptor.validate_discovery_exposure(DiscoveryPath::PublicRegistry);

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DiscoveryExposureRefused {
            requested_path: DiscoveryPath::PublicRegistry,
            discovery_policy: DiscoveryPolicy::None,
        })
    );
}

#[test]
fn rejects_private_descriptor_exposure_through_user_introduction() {
    let descriptor = descriptor();

    let result = descriptor.validate_discovery_exposure(DiscoveryPath::UserConsentedIntroduction);

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DiscoveryExposureRefused {
            requested_path: DiscoveryPath::UserConsentedIntroduction,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        })
    );
    assert!(!descriptor.can_be_user_introduced());
}

#[test]
fn allows_user_introduction_only_when_descriptor_policy_permits_it() {
    let mut descriptor = descriptor();
    descriptor.discovery_policy = DiscoveryPolicy::UserConsentedIntroduction;

    let result = descriptor.validate_discovery_exposure(DiscoveryPath::UserConsentedIntroduction);

    assert_eq!(result, Ok(()));
    assert!(descriptor.can_be_user_introduced());
}

#[test]
fn rejects_public_discovery_for_private_peer_network_mode() {
    let mut descriptor = descriptor();
    descriptor.discovery_policy = DiscoveryPolicy::PublicRegistry;

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::DiscoveryPolicyConflict {
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PublicRegistry,
        })
    );
}

#[test]
fn rejects_lan_only_descriptor_with_public_peering() {
    let mut descriptor = descriptor();
    descriptor.network_mode = NetworkMode::LanOnly;
    descriptor.discovery_policy = DiscoveryPolicy::LanAnnounce;
    descriptor.peering_policy = PeeringPolicy::PublicAuthenticated;

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::PeeringPolicyConflict {
            network_mode: NetworkMode::LanOnly,
            peering_policy: PeeringPolicy::PublicAuthenticated,
        })
    );
}

#[test]
fn allows_local_only_descriptor_without_network_address() {
    let mut descriptor = descriptor();
    descriptor.network_mode = NetworkMode::LocalOnly;
    descriptor.discovery_policy = DiscoveryPolicy::None;
    descriptor.peering_policy = PeeringPolicy::None;
    descriptor.relay_policy = RelayPolicy::None;
    descriptor.dm_forwarding_policy = DmForwardingPolicy::LocalRecipientsOnly;
    descriptor.addresses = Vec::new();

    let result = descriptor.validate(&validation_context());

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_offline_descriptor_with_dm_forwarding() {
    let mut descriptor = descriptor();
    descriptor.network_mode = NetworkMode::Offline;
    descriptor.discovery_policy = DiscoveryPolicy::None;
    descriptor.peering_policy = PeeringPolicy::None;
    descriptor.relay_policy = RelayPolicy::None;
    descriptor.dm_forwarding_policy = DmForwardingPolicy::LocalRecipientsOnly;
    descriptor.addresses = Vec::new();

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(
            ServerDescriptorValidationError::DmForwardingPolicyConflict {
                relay_policy: RelayPolicy::None,
                dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            }
        )
    );
}

#[test]
fn rejects_relay_use_when_server_peers_but_refuses_relay() {
    let mut descriptor = descriptor();
    descriptor.peering_policy = PeeringPolicy::InviteToken;
    descriptor.relay_policy = RelayPolicy::None;
    descriptor.dm_forwarding_policy = DmForwardingPolicy::LocalRecipientsOnly;

    let result = descriptor.validate_relay_use();

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::RelayRefused {
            relay_policy: RelayPolicy::None,
        })
    );
    assert!(!descriptor.allows_relay());
}

#[test]
fn accepts_relay_only_when_relay_and_forwarding_policy_match() {
    let mut descriptor = descriptor();
    descriptor.relay_policy = RelayPolicy::AllowlistedPeers;
    descriptor.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;

    assert_eq!(descriptor.validate(&validation_context()), Ok(()));
    assert_eq!(descriptor.validate_relay_use(), Ok(()));
    assert!(descriptor.allows_relay());
}

#[test]
fn rejects_relay_policy_without_route_forwarding_permission() {
    let mut descriptor = descriptor();
    descriptor.relay_policy = RelayPolicy::AllowlistedPeers;
    descriptor.dm_forwarding_policy = DmForwardingPolicy::LocalRecipientsOnly;

    let result = descriptor.validate(&validation_context());

    assert_eq!(
        result,
        Err(ServerDescriptorValidationError::RelayPolicyConflict {
            relay_policy: RelayPolicy::AllowlistedPeers,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        })
    );
}
