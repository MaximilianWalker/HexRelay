use crate::domain::{
    CandidatePeerPolicy, DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath,
    DiscoveryPolicy, DmForwardingPolicy, NetworkMode, PeerCandidateValidationError, PeerRouteKind,
    PeerRouteSelectionError, PeeringPolicy, RelayPolicy, RouteSelectionPolicy, ServerDescriptor,
    ServerDescriptorValidationError, ServerSignature, ServerSignatureAlgorithm, StaticPeerRegistry,
    StaticPeerRegistryError, StoragePolicy,
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

fn descriptor(server_id: &str, descriptor_id: &str) -> ServerDescriptor {
    ServerDescriptor {
        server_id: server_id.to_string(),
        server_public_key: format!("ed25519-public-key-{server_id}"),
        descriptor_id: descriptor_id.to_string(),
        issued_at_epoch_seconds: 1_000,
        expires_at_epoch_seconds: 1_300,
        network_mode: NetworkMode::PrivatePeers,
        discovery_policy: DiscoveryPolicy::PrivateAllowlist,
        peering_policy: PeeringPolicy::StaticAllowlist,
        relay_policy: RelayPolicy::None,
        dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
        storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
        addresses: vec![format!("https://{server_id}.example")],
        supported_protocols: vec!["hexrelay-server-http".to_string()],
        rate_limits: Vec::new(),
        trust_labels: Vec::new(),
        revocation_pointer: None,
        signature: ServerSignature {
            algorithm: ServerSignatureAlgorithm::Ed25519,
            value: format!("signed-{descriptor_id}"),
        },
    }
}

fn registry_with(peer: ServerDescriptor) -> StaticPeerRegistry {
    registry_with_many(vec![peer])
}

fn registry_with_many(peers: Vec<ServerDescriptor>) -> StaticPeerRegistry {
    StaticPeerRegistry::try_new(peers).expect("registry should be valid")
}

#[test]
fn validates_static_private_mesh_candidate() {
    let peer = descriptor("server-a", "descriptor-a");
    let registry = registry_with(peer);

    let candidate = registry
        .validate_candidate(
            "server-a",
            &validation_context(),
            &StaticVerifier { valid: true },
            &CandidatePeerPolicy::private_mesh(),
        )
        .expect("candidate should validate");

    assert_eq!(candidate.descriptor.server_id, "server-a");
    assert_eq!(candidate.discovery_path, DiscoveryPath::PrivateAllowlist);
    assert!(candidate.delivery_allowed);
    assert!(!candidate.relay_allowed);
}

#[test]
fn validates_invite_token_peer_for_private_mesh() {
    let mut peer = descriptor("server-a", "descriptor-a");
    peer.peering_policy = PeeringPolicy::InviteToken;
    let registry = registry_with(peer);

    let candidate = registry.validate_candidate(
        "server-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert!(candidate.is_ok());
}

#[test]
fn rejects_duplicate_server_ids() {
    let first = descriptor("server-a", "descriptor-a");
    let second = descriptor("server-a", "descriptor-b");

    let result = StaticPeerRegistry::try_new(vec![first, second]);

    assert_eq!(
        result,
        Err(StaticPeerRegistryError::DuplicateServerId(
            "server-a".to_string()
        ))
    );
}

#[test]
fn rejects_duplicate_descriptor_ids() {
    let first = descriptor("server-a", "descriptor-a");
    let second = descriptor("server-b", "descriptor-a");

    let result = StaticPeerRegistry::try_new(vec![first, second]);

    assert_eq!(
        result,
        Err(StaticPeerRegistryError::DuplicateDescriptorId(
            "descriptor-a".to_string()
        ))
    );
}

#[test]
fn rejects_unknown_candidate_server() {
    let registry = registry_with(descriptor("server-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "server-missing",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::CandidateNotFound {
            server_id: "server-missing".to_string(),
        })
    );
}

#[test]
fn rejects_peer_when_signature_verifier_fails() {
    let registry = registry_with(descriptor("server-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "server-a",
        &validation_context(),
        &StaticVerifier { valid: false },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::DescriptorInvalid(
            ServerDescriptorValidationError::SignatureVerificationFailed
        ))
    );
}

#[test]
fn rejects_public_descriptor_for_private_mesh_candidate() {
    let mut peer = descriptor("server-a", "descriptor-a");
    peer.network_mode = NetworkMode::PublicDiscovery;
    peer.discovery_policy = DiscoveryPolicy::PublicRegistry;
    peer.peering_policy = PeeringPolicy::PublicAuthenticated;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "server-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::DiscoveryNotAllowed(
            ServerDescriptorValidationError::DiscoveryExposureRefused {
                requested_path: DiscoveryPath::PrivateAllowlist,
                discovery_policy: DiscoveryPolicy::PublicRegistry,
            }
        ))
    );
}

#[test]
fn rejects_private_mesh_candidate_when_peer_refuses_peering() {
    let mut peer = descriptor("server-a", "descriptor-a");
    peer.peering_policy = PeeringPolicy::None;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "server-a",
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
    let mut peer = descriptor("server-a", "descriptor-a");
    peer.dm_forwarding_policy = DmForwardingPolicy::Disabled;
    let registry = registry_with(peer);

    let result = registry.validate_candidate(
        "server-a",
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
    let registry = registry_with(descriptor("server-a", "descriptor-a"));

    let result = registry.validate_candidate(
        "server-a",
        &validation_context(),
        &StaticVerifier { valid: true },
        &CandidatePeerPolicy::private_mesh_relay(),
    );

    assert_eq!(
        result,
        Err(PeerCandidateValidationError::RelayRefused(
            ServerDescriptorValidationError::RelayRefused {
                relay_policy: RelayPolicy::None,
            }
        ))
    );
}

#[test]
fn validates_allowlisted_relay_candidate() {
    let mut peer = descriptor("server-a", "descriptor-a");
    peer.relay_policy = RelayPolicy::AllowlistedPeers;
    peer.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with(peer);

    let candidate = registry
        .validate_candidate(
            "server-a",
            &validation_context(),
            &StaticVerifier { valid: true },
            &CandidatePeerPolicy::private_mesh_relay(),
        )
        .expect("relay candidate should validate");

    assert!(candidate.delivery_allowed);
    assert!(candidate.relay_allowed);
}

#[test]
fn selects_direct_route_before_relay_for_known_destination() {
    let destination = descriptor("server-destination", "descriptor-destination");
    let mut relay = descriptor("server-relay", "descriptor-relay");
    relay.relay_policy = RelayPolicy::AllowlistedPeers;
    relay.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with_many(vec![destination, relay]);

    let route = registry
        .select_route(
            "server-destination",
            &validation_context(),
            &StaticVerifier { valid: true },
            &RouteSelectionPolicy::private_mesh_with_one_hop_relay(),
        )
        .expect("direct route should be selected");

    assert_eq!(route.kind, PeerRouteKind::Direct);
    assert_eq!(route.hop_count, 1);
    assert_eq!(route.destination.descriptor.server_id, "server-destination");
    assert!(route.relay.is_none());
    assert!(route
        .policy_assertions
        .iter()
        .any(|value| value == "direct_static_peer_route_selected"));
}

#[test]
fn selects_one_hop_relay_when_direct_destination_is_unavailable() {
    let destination = descriptor("server-destination", "descriptor-destination");
    let mut relay = descriptor("server-relay", "descriptor-relay");
    relay.relay_policy = RelayPolicy::AllowlistedPeers;
    relay.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with_many(vec![destination, relay]);

    let route = registry
        .select_route(
            "server-destination",
            &validation_context(),
            &StaticVerifier { valid: true },
            &RouteSelectionPolicy::private_mesh_with_one_hop_relay()
                .with_unavailable_direct_server("server-destination"),
        )
        .expect("relay route should be selected");

    assert_eq!(route.kind, PeerRouteKind::OneHopRelay);
    assert_eq!(route.hop_count, 2);
    assert_eq!(route.destination.descriptor.server_id, "server-destination");
    assert_eq!(
        route
            .relay
            .as_ref()
            .expect("relay should be present")
            .descriptor
            .server_id,
        "server-relay"
    );
}

#[test]
fn relay_route_requires_relay_enabled_policy_and_hop_limit() {
    let destination = descriptor("server-destination", "descriptor-destination");
    let mut relay = descriptor("server-relay", "descriptor-relay");
    relay.relay_policy = RelayPolicy::AllowlistedPeers;
    relay.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with_many(vec![destination, relay]);

    let result = registry.select_route(
        "server-destination",
        &validation_context(),
        &StaticVerifier { valid: true },
        &RouteSelectionPolicy::private_mesh_direct()
            .with_unavailable_direct_server("server-destination"),
    );

    assert_eq!(
        result,
        Err(PeerRouteSelectionError::DirectRouteUnavailable {
            destination_server_id: "server-destination".to_string()
        })
    );
}

#[test]
fn prefers_allowlisted_relay_over_open_limited_relay() {
    let destination = descriptor("server-destination", "descriptor-destination");
    let mut open_relay = descriptor("server-a-open-relay", "descriptor-open-relay");
    open_relay.relay_policy = RelayPolicy::OpenLimited;
    open_relay.dm_forwarding_policy = DmForwardingPolicy::RelayAllowed;
    let mut allowlisted_relay = descriptor("server-z-allowlisted-relay", "descriptor-allowlisted");
    allowlisted_relay.relay_policy = RelayPolicy::AllowlistedPeers;
    allowlisted_relay.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with_many(vec![destination, open_relay, allowlisted_relay]);

    let route = registry
        .select_route(
            "server-destination",
            &validation_context(),
            &StaticVerifier { valid: true },
            &RouteSelectionPolicy::private_mesh_with_one_hop_relay()
                .with_unavailable_direct_server("server-destination"),
        )
        .expect("relay route should be selected");

    assert_eq!(
        route
            .relay
            .as_ref()
            .expect("relay should be present")
            .descriptor
            .server_id,
        "server-z-allowlisted-relay"
    );
}

#[test]
fn does_not_route_around_destination_delivery_refusal() {
    let mut destination = descriptor("server-destination", "descriptor-destination");
    destination.dm_forwarding_policy = DmForwardingPolicy::Disabled;
    let mut relay = descriptor("server-relay", "descriptor-relay");
    relay.relay_policy = RelayPolicy::AllowlistedPeers;
    relay.dm_forwarding_policy = DmForwardingPolicy::AllowlistedRoute;
    let registry = registry_with_many(vec![destination, relay]);

    let result = registry.select_route(
        "server-destination",
        &validation_context(),
        &StaticVerifier { valid: true },
        &RouteSelectionPolicy::private_mesh_with_one_hop_relay()
            .with_unavailable_direct_server("server-destination"),
    );

    assert_eq!(
        result,
        Err(PeerRouteSelectionError::DestinationRefused(
            PeerCandidateValidationError::DmDeliveryRefused {
                dm_forwarding_policy: DmForwardingPolicy::Disabled,
            }
        ))
    );
}

#[test]
fn does_not_select_own_users_only_server_as_intermediate_relay() {
    let destination = descriptor("server-destination", "descriptor-destination");
    let mut relay = descriptor("server-relay", "descriptor-relay");
    relay.relay_policy = RelayPolicy::OwnUsersOnly;
    relay.dm_forwarding_policy = DmForwardingPolicy::RelayAllowed;
    let registry = registry_with_many(vec![destination, relay]);

    let result = registry.select_route(
        "server-destination",
        &validation_context(),
        &StaticVerifier { valid: true },
        &RouteSelectionPolicy::private_mesh_with_one_hop_relay()
            .with_unavailable_direct_server("server-destination"),
    );

    assert_eq!(
        result,
        Err(PeerRouteSelectionError::RelayRouteUnavailable {
            destination_server_id: "server-destination".to_string()
        })
    );
}
