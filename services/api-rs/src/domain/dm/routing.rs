use std::fmt;

use chrono::Utc;
use communication_core::{
    DescriptorValidationContext, Ed25519DescriptorVerifier, PeerRouteKind, PeerRouteSelectionError,
    RouteSelectionPolicy, SelectedPeerRoute, StaticPeerRegistry,
};

#[derive(Debug, Clone)]
pub struct DmEnvelopeRouteRequest<'a> {
    pub destination_node_id: Option<&'a str>,
    pub allow_relay: bool,
    pub unavailable_direct_node_ids: &'a [String],
    pub excluded_relay_node_ids: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmEnvelopeForwardingRoute {
    LocalRealtime {
        node_id: String,
        policy_assertions: Vec<String>,
    },
    StaticPeer {
        route: Box<SelectedPeerRoute>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DmEnvelopeRouteError {
    MissingLocalNodeId,
    StaticPeerRouteUnavailable(PeerRouteSelectionError),
}

impl<'a> DmEnvelopeRouteRequest<'a> {
    pub fn local_realtime() -> Self {
        Self {
            destination_node_id: None,
            allow_relay: false,
            unavailable_direct_node_ids: &[],
            excluded_relay_node_ids: &[],
        }
    }

    pub fn static_destination(destination_node_id: &'a str) -> Self {
        Self {
            destination_node_id: Some(destination_node_id),
            allow_relay: false,
            unavailable_direct_node_ids: &[],
            excluded_relay_node_ids: &[],
        }
    }

    pub fn with_one_hop_relay(mut self) -> Self {
        self.allow_relay = true;
        self
    }
}

impl DmEnvelopeForwardingRoute {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::LocalRealtime { .. } => "local_realtime",
            Self::StaticPeer { route } => match route.kind {
                PeerRouteKind::Direct => "static_peer_direct",
                PeerRouteKind::OneHopRelay => "static_peer_one_hop_relay",
            },
        }
    }

    pub fn destination_node_id(&self) -> &str {
        match self {
            Self::LocalRealtime { node_id, .. } => node_id,
            Self::StaticPeer { route } => &route.destination.descriptor.node_id,
        }
    }

    pub fn relay_node_id(&self) -> Option<&str> {
        match self {
            Self::LocalRealtime { .. } => None,
            Self::StaticPeer { route } => route
                .relay
                .as_ref()
                .map(|relay| relay.descriptor.node_id.as_str()),
        }
    }

    pub fn policy_assertions(&self) -> &[String] {
        match self {
            Self::LocalRealtime {
                policy_assertions, ..
            } => policy_assertions,
            Self::StaticPeer { route } => &route.policy_assertions,
        }
    }
}

impl fmt::Display for DmEnvelopeRouteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocalNodeId => write!(formatter, "local node id is not configured"),
            Self::StaticPeerRouteUnavailable(error) => {
                write!(formatter, "static peer route unavailable: {error:?}")
            }
        }
    }
}

pub fn plan_dm_envelope_route(
    local_node_id: &str,
    registry: &StaticPeerRegistry,
    request: DmEnvelopeRouteRequest<'_>,
) -> Result<DmEnvelopeForwardingRoute, DmEnvelopeRouteError> {
    let local_node_id = local_node_id.trim();
    if local_node_id.is_empty() {
        return Err(DmEnvelopeRouteError::MissingLocalNodeId);
    }

    let Some(destination_node_id) = request.destination_node_id else {
        return Ok(DmEnvelopeForwardingRoute::LocalRealtime {
            node_id: local_node_id.to_string(),
            policy_assertions: vec![
                "local_realtime_dispatch_selected".to_string(),
                "server_node_forwarding_not_required".to_string(),
            ],
        });
    };

    let mut policy = if request.allow_relay {
        RouteSelectionPolicy::private_mesh_with_one_hop_relay()
    } else {
        RouteSelectionPolicy::private_mesh_direct()
    };
    policy.unavailable_direct_node_ids = request.unavailable_direct_node_ids.to_vec();
    policy.excluded_relay_node_ids = request.excluded_relay_node_ids.to_vec();

    let context = DescriptorValidationContext {
        now_epoch_seconds: Utc::now().timestamp(),
        max_ttl_seconds: i64::MAX,
        revoked_descriptor_ids: Vec::new(),
    };

    registry
        .select_route(
            destination_node_id,
            &context,
            &Ed25519DescriptorVerifier,
            &policy,
        )
        .map(|route| DmEnvelopeForwardingRoute::StaticPeer {
            route: Box::new(route),
        })
        .map_err(DmEnvelopeRouteError::StaticPeerRouteUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
        NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy,
        RelayPolicy, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::Ed25519KeyPair;

    fn signed_descriptor(
        node_id: &str,
        descriptor_id: &str,
        relay_policy: RelayPolicy,
        dm_forwarding_policy: DmForwardingPolicy,
    ) -> NodeDescriptor {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = Utc::now().timestamp();
        let mut descriptor = NodeDescriptor {
            node_id: node_id.to_string(),
            node_public_key: public_key,
            descriptor_id: descriptor_id.to_string(),
            issued_at_epoch_seconds: now - 1,
            expires_at_epoch_seconds: now + 300,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy: PeeringPolicy::StaticAllowlist,
            relay_policy,
            dm_forwarding_policy,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec![format!("https://{node_id}.example")],
            supported_protocols: vec!["hexrelay-node-http".to_string()],
            rate_limits: Vec::new(),
            trust_labels: Vec::new(),
            revocation_pointer: None,
            signature: NodeSignature {
                algorithm: NodeSignatureAlgorithm::Ed25519,
                value: String::new(),
            },
        };
        descriptor.signature.value =
            sign_descriptor_ed25519_pkcs8(&descriptor, pkcs8.as_ref()).expect("sign descriptor");
        descriptor
    }

    #[test]
    fn plans_local_realtime_route_without_destination_node() {
        let registry = StaticPeerRegistry::default();

        let route = plan_dm_envelope_route(
            "node-local",
            &registry,
            DmEnvelopeRouteRequest::local_realtime(),
        )
        .expect("local route should plan");

        assert_eq!(route.kind(), "local_realtime");
        assert_eq!(route.destination_node_id(), "node-local");
        assert_eq!(route.relay_node_id(), None);
        assert!(route
            .policy_assertions()
            .iter()
            .any(|value| value == "server_node_forwarding_not_required"));
    }

    #[test]
    fn plans_static_direct_route_for_signed_destination_node() {
        let destination = signed_descriptor(
            "node-destination",
            "descriptor-destination",
            RelayPolicy::None,
            DmForwardingPolicy::LocalRecipientsOnly,
        );
        let registry = StaticPeerRegistry::try_new(vec![destination]).expect("registry");

        let route = plan_dm_envelope_route(
            "node-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("node-destination"),
        )
        .expect("static route should plan");

        assert_eq!(route.kind(), "static_peer_direct");
        assert_eq!(route.destination_node_id(), "node-destination");
        assert_eq!(route.relay_node_id(), None);
    }

    #[test]
    fn plans_static_one_hop_route_when_direct_destination_is_unavailable() {
        let destination = signed_descriptor(
            "node-destination",
            "descriptor-destination",
            RelayPolicy::None,
            DmForwardingPolicy::LocalRecipientsOnly,
        );
        let relay = signed_descriptor(
            "node-relay",
            "descriptor-relay",
            RelayPolicy::AllowlistedPeers,
            DmForwardingPolicy::AllowlistedRoute,
        );
        let registry = StaticPeerRegistry::try_new(vec![destination, relay]).expect("registry");
        let unavailable = vec!["node-destination".to_string()];

        let mut request =
            DmEnvelopeRouteRequest::static_destination("node-destination").with_one_hop_relay();
        request.unavailable_direct_node_ids = &unavailable;

        let route = plan_dm_envelope_route("node-local", &registry, request).expect("relay route");

        assert_eq!(route.kind(), "static_peer_one_hop_relay");
        assert_eq!(route.destination_node_id(), "node-destination");
        assert_eq!(route.relay_node_id(), Some("node-relay"));
    }

    #[test]
    fn rejects_static_route_when_destination_refuses_delivery() {
        let destination = signed_descriptor(
            "node-destination",
            "descriptor-destination",
            RelayPolicy::None,
            DmForwardingPolicy::Disabled,
        );
        let registry = StaticPeerRegistry::try_new(vec![destination]).expect("registry");

        let result = plan_dm_envelope_route(
            "node-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("node-destination"),
        );

        assert!(matches!(
            result,
            Err(DmEnvelopeRouteError::StaticPeerRouteUnavailable(_))
        ));
    }
}
