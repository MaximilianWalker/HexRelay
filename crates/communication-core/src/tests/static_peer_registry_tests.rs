use crate::domain::{
    CandidatePeerPolicy, DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath,
    DiscoveryPolicy, DmForwardingPolicy, NetworkMode, NodeDescriptor,
    NodeDescriptorValidationError, NodeSignature, NodeSignatureAlgorithm,
    PeerCandidateValidationError, PeeringPolicy, RelayPolicy, StaticPeerRegistry,
    StaticPeerRegistryError, StoragePolicy,
};

struct StaticVerifier {
    valid: bool,
}

impl DescriptorSignatureVerifier for StaticVerifier {
    fn verify(&self, _descriptor: &NodeDescriptor) -> bool {
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

fn descriptor(node_id: &str, descriptor_id: &str) -> NodeDescriptor {
    NodeDescriptor {
        node_id: node_id.to_string(),
        node_public_key: format!("ed25519-public-key-{node_id}"),
        descriptor_id: descriptor_id.to_string(),
        issued_at_epoch_seconds: 1_000,
        expires_at_epoch_seconds: 1_300,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy: PeeringPolicy::StaticAllowlist,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec![format!("https://{node_id}.example")],
        supported_protocols: vec!["hexrelay-node-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: None,
        signature: NodeSignature {
            algorithm: NodeSignatureAlgorithm::Ed25519,
            value: format!("signed-{descriptor_id}"),
        },
    }
}

fn registry_with(peer: NodeDescriptor) -> StaticPeerRegistry {
    StaticPeerRegistry::try_new(vec![peer]).expect("registry should be valid")
}

#[test]
fn validates_static_private_mesh_candidate() {
    let peer = descriptor("node-a", "descriptor-a");
    let registry = registry_with(peer);

    let candidate = registry
        .validate_candidate(
            "node-a",
            &validation_context(),
            &StaticVerifier { valid: true },
            &CandidatePeerPolicy::private_mesh(),
        )
        .expect("candidate should validate");

    assert_eq!(candidate.descriptor.node_id, "node-a");
    assert_eq!(candidate.discovery_path, DiscoveryPath::PrivateAllowlist);
    assert!(candidate.delivery_allowed);
    assert!(!candidate.relay_allowed);
}

#[test]
fn validates_invite_token_peer_for_private_mesh() {
    let mut peer = descriptor("node-a", "descriptor-a");
    peer.peering_policy = PeeringPolicy::InviteToken;
    let registry = registry_with(peer);

    let candidate = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert!(candidate.is_ok());
}

#[test]
fn rejects_duplicate_node_ids() {
    let first = descriptor("node-a", "descriptor-a");
    let second = descriptor("node-a", "descriptor-b");

    let result = StaticPeerRegistry::try_new(vec![first, second]);

    assert_eq!(
        result,
        Err(StaticPeerRegistryError::DuplicateNodeId(
            "node-a".to_string()
        ))
    );
}

#[test]
fn rejects_duplicate_descriptor_ids() {
    let first = descriptor("node-a", "descriptor-a");
    let second = descriptor("node-b", "descriptor-a");

    let result = StaticPeerRegistry::try_new(vec![first, second]);

    assert_eq!(
        result,
        Err(StaticPeerRegistryError::DuplicateDescriptorId(
            "descriptor-a".to_string()
        ))
    );
}

#[test]
fn rejects_unknown_candidate_node() {
    let registry = registry_with(descriptor("node-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "node-missing",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::CandidateNotFound {
            node_id: "node-missing".to_string(),
        })
    );
}

#[test]
fn rejects_peer_when_signature_verifier_fails() {
    let registry = registry_with(descriptor("node-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: false },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::DescriptorInvalid(
            NodeDescriptorValidationError::SignatureVerificationFailed
        ))
    );
}

#[test]
fn rejects_public_descriptor_for_private_mesh_candidate() {
    let mut peer = descriptor("node-a", "descriptor-a");
    peer.network_mode = NetworkMode::PublicDiscovery;
    peer.discovery_policy = DiscoveryPolicy::PublicRegistry;
    peer.peering_policy = PeeringPolicy::PublicAuthenticated;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::DiscoveryNotAllowed(
            NodeDescriptorValidationError::DiscoveryExposureRefused {
                requested_path: DiscoveryPath::PrivateAllowlist,
                discovery_policy: DiscoveryPolicy::PublicRegistry,
            }
        ))
    );
}

#[test]
fn rejects_private_mesh_candidate_when_peer_refuses_peering() {
    let mut peer = descriptor("node-a", "descriptor-a");
    peer.peering_policy = PeeringPolicy::None;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::PeeringRefused {
            peering_policy: PeeringPolicy::None,
        })
    );
}

#[test]
fn rejects_private_mesh_candidate_when_delivery_is_disabled() {
    let mut peer = descriptor("node-a", "descriptor-a");
    peer.dm_forwarding_policy = DmForwardingPolicy::Disabled;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::DmDeliveryRefused {
            dm_forwarding_policy: DmForwardingPolicy::Disabled,
        })
    );
}

#[test]
fn rejects_relay_candidate_when_peer_refuses_relay() {
    let registry = registry_with(descriptor("node-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "node-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh_relay(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::RelayRefused(
            NodeDescriptorValidationError::RelayRefused {
                relay_policy: RelayPolicy::None,
            }
        ))
    );
}

#[test]
fn validates_allowlisted_relay_candidate() {
    let mut peer = descriptor("node-a", "descriptor-a");
    peer.relay_policy = RelayPolicy::AllowlistedPeers;
    peer.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with(peer);

    let candidate = registry
        .validate_candidate(
            "node-a",
            &validation_context(),
            &StaticVerifier { valid: true },
            &CandidatePeerPolicy::private_mesh_relay(),
        )
        .expect("relay candidate should validate");

    assert!(candidate.delivery_allowed);
    assert!(candidate.relay_allowed);
}
