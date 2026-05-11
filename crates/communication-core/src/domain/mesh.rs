use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DmForwardingPolicy,
    NodeDescriptor, NodeDescriptorValidationError, PeeringPolicy, RelayPolicy, StoragePolicy,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticPeerRegistry {
    descriptors: Vec<NodeDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePeerPolicy {
    pub discovery_path: DiscoveryPath,
    pub allowed_peering_policies: Vec<PeeringPolicy>,
    pub require_delivery: bool,
    pub require_relay: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerCandidate {
    pub descriptor: NodeDescriptor,
    pub discovery_path: DiscoveryPath,
    pub relay_allowed: bool,
    pub delivery_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSelectionPolicy {
    pub destination_policy: CandidatePeerPolicy,
    pub relay_policy: CandidatePeerPolicy,
    pub max_hops: u8,
    pub allow_relay: bool,
    pub unavailable_direct_node_ids: Vec<String>,
    pub excluded_relay_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerRouteKind {
    Direct,
    OneHopRelay,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedPeerRoute {
    pub kind: PeerRouteKind,
    pub destination: PeerCandidate,
    pub relay: Option<PeerCandidate>,
    pub hop_count: u8,
    pub policy_assertions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticPeerRegistryError {
    DuplicateNodeId(String),
    DuplicateDescriptorId(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerCandidateValidationError {
    CandidateNotFound {
        node_id: String,
    },
    DescriptorInvalid(NodeDescriptorValidationError),
    DiscoveryNotAllowed(NodeDescriptorValidationError),
    PeeringRefused {
        peering_policy: PeeringPolicy,
    },
    DmDeliveryRefused {
        dm_forwarding_policy: DmForwardingPolicy,
    },
    RelayRefused(NodeDescriptorValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerRouteSelectionError {
    InvalidMaxHops,
    DestinationRefused(PeerCandidateValidationError),
    DirectRouteUnavailable { destination_node_id: String },
    RelayRouteUnavailable { destination_node_id: String },
}

impl CandidatePeerPolicy {
    pub fn private_mesh() -> Self {
        Self {
            discovery_path: DiscoveryPath::PrivateAllowlist,
            allowed_peering_policies: vec![
                PeeringPolicy::StaticAllowlist,
                PeeringPolicy::InviteToken,
            ],
            require_delivery: true,
            require_relay: false,
        }
    }

    pub fn private_mesh_relay() -> Self {
        Self {
            require_relay: true,
            ..Self::private_mesh()
        }
    }
}

impl RouteSelectionPolicy {
    pub fn private_mesh_direct() -> Self {
        Self {
            destination_policy: CandidatePeerPolicy::private_mesh(),
            relay_policy: CandidatePeerPolicy::private_mesh_relay(),
            max_hops: 1,
            allow_relay: false,
            unavailable_direct_node_ids: Vec::new(),
            excluded_relay_node_ids: Vec::new(),
        }
    }

    pub fn private_mesh_with_one_hop_relay() -> Self {
        Self {
            max_hops: 2,
            allow_relay: true,
            ..Self::private_mesh_direct()
        }
    }

    pub fn with_unavailable_direct_node(mut self, node_id: impl Into<String>) -> Self {
        self.unavailable_direct_node_ids.push(node_id.into());
        self
    }

    pub fn with_excluded_relay_node(mut self, node_id: impl Into<String>) -> Self {
        self.excluded_relay_node_ids.push(node_id.into());
        self
    }
}

impl StaticPeerRegistry {
    pub fn try_new(descriptors: Vec<NodeDescriptor>) -> Result<Self, StaticPeerRegistryError> {
        let mut node_ids = BTreeSet::new();
        let mut descriptor_ids = BTreeSet::new();

        for descriptor in &descriptors {
            if !node_ids.insert(descriptor.node_id.clone()) {
                return Err(StaticPeerRegistryError::DuplicateNodeId(
                    descriptor.node_id.clone(),
                ));
            }

            if !descriptor_ids.insert(descriptor.descriptor_id.clone()) {
                return Err(StaticPeerRegistryError::DuplicateDescriptorId(
                    descriptor.descriptor_id.clone(),
                ));
            }
        }

        Ok(Self { descriptors })
    }

    pub fn descriptors(&self) -> &[NodeDescriptor] {
        &self.descriptors
    }

    pub fn find(&self, node_id: &str) -> Option<&NodeDescriptor> {
        self.descriptors
            .iter()
            .find(|descriptor| descriptor.node_id == node_id)
    }

    pub fn validate_candidate<V: DescriptorSignatureVerifier>(
        &self,
        node_id: &str,
        context: &DescriptorValidationContext,
        verifier: &V,
        policy: &CandidatePeerPolicy,
    ) -> Result<PeerCandidate, PeerCandidateValidationError> {
        let descriptor =
            self.find(node_id)
                .ok_or_else(|| PeerCandidateValidationError::CandidateNotFound {
                    node_id: node_id.to_string(),
                })?;

        descriptor
            .validate_with_signature(context, verifier)
            .map_err(PeerCandidateValidationError::DescriptorInvalid)?;

        descriptor
            .validate_discovery_exposure(policy.discovery_path)
            .map_err(PeerCandidateValidationError::DiscoveryNotAllowed)?;

        if !policy
            .allowed_peering_policies
            .contains(&descriptor.peering_policy)
        {
            return Err(PeerCandidateValidationError::PeeringRefused {
                peering_policy: descriptor.peering_policy,
            });
        }

        let delivery_allowed = descriptor.accepts_local_recipient_delivery();
        if policy.require_delivery && !delivery_allowed {
            return Err(PeerCandidateValidationError::DmDeliveryRefused {
                dm_forwarding_policy: descriptor.dm_forwarding_policy,
            });
        }

        if policy.require_relay {
            descriptor
                .validate_relay_use()
                .map_err(PeerCandidateValidationError::RelayRefused)?;
        }

        Ok(PeerCandidate {
            descriptor: descriptor.clone(),
            discovery_path: policy.discovery_path,
            relay_allowed: descriptor.allows_relay(),
            delivery_allowed,
        })
    }

    pub fn select_route<V: DescriptorSignatureVerifier>(
        &self,
        destination_node_id: &str,
        context: &DescriptorValidationContext,
        verifier: &V,
        policy: &RouteSelectionPolicy,
    ) -> Result<SelectedPeerRoute, PeerRouteSelectionError> {
        if policy.max_hops == 0 {
            return Err(PeerRouteSelectionError::InvalidMaxHops);
        }

        let destination = self
            .validate_candidate(
                destination_node_id,
                context,
                verifier,
                &policy.destination_policy,
            )
            .map_err(PeerRouteSelectionError::DestinationRefused)?;

        if !contains_node_id(
            &policy.unavailable_direct_node_ids,
            &destination.descriptor.node_id,
        ) {
            return Ok(SelectedPeerRoute::direct(destination));
        }

        if !policy.allow_relay || policy.max_hops < 2 {
            return Err(PeerRouteSelectionError::DirectRouteUnavailable {
                destination_node_id: destination.descriptor.node_id,
            });
        }

        let destination_route_node_id = destination.descriptor.node_id.clone();

        self.select_relay_candidate(&destination, context, verifier, policy)
            .map(|relay| SelectedPeerRoute::one_hop_relay(destination, relay))
            .ok_or(PeerRouteSelectionError::RelayRouteUnavailable {
                destination_node_id: destination_route_node_id,
            })
    }

    fn select_relay_candidate<V: DescriptorSignatureVerifier>(
        &self,
        destination: &PeerCandidate,
        context: &DescriptorValidationContext,
        verifier: &V,
        policy: &RouteSelectionPolicy,
    ) -> Option<PeerCandidate> {
        let mut candidates = self
            .descriptors
            .iter()
            .filter(|descriptor| descriptor.node_id != destination.descriptor.node_id)
            .filter(|descriptor| {
                !contains_node_id(&policy.excluded_relay_node_ids, &descriptor.node_id)
            })
            .filter(|descriptor| descriptor.allows_intermediate_relay())
            .filter_map(|descriptor| {
                self.validate_candidate(
                    &descriptor.node_id,
                    context,
                    verifier,
                    &policy.relay_policy,
                )
                .ok()
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            relay_sort_key(&left.descriptor).cmp(&relay_sort_key(&right.descriptor))
        });
        candidates.into_iter().next()
    }
}

impl SelectedPeerRoute {
    fn direct(destination: PeerCandidate) -> Self {
        Self {
            kind: PeerRouteKind::Direct,
            destination,
            relay: None,
            hop_count: 1,
            policy_assertions: vec![
                "destination_descriptor_policy_valid".to_string(),
                "direct_static_peer_route_selected".to_string(),
            ],
        }
    }

    fn one_hop_relay(destination: PeerCandidate, relay: PeerCandidate) -> Self {
        Self {
            kind: PeerRouteKind::OneHopRelay,
            destination,
            relay: Some(relay),
            hop_count: 2,
            policy_assertions: vec![
                "destination_descriptor_policy_valid".to_string(),
                "relay_descriptor_policy_valid".to_string(),
                "one_hop_static_peer_route_selected".to_string(),
            ],
        }
    }
}

fn contains_node_id(values: &[String], node_id: &str) -> bool {
    values.iter().any(|value| value == node_id)
}

fn relay_sort_key(descriptor: &NodeDescriptor) -> (u8, u8, &str, &str) {
    (
        relay_policy_rank(descriptor.relay_policy),
        storage_policy_rank(descriptor.storage_policy),
        descriptor.node_id.as_str(),
        descriptor.descriptor_id.as_str(),
    )
}

fn relay_policy_rank(value: RelayPolicy) -> u8 {
    match value {
        RelayPolicy::AllowlistedPeers => 0,
        RelayPolicy::OpenLimited => 1,
        RelayPolicy::OwnUsersOnly => 2,
        RelayPolicy::None => 3,
    }
}

fn storage_policy_rank(value: StoragePolicy) -> u8 {
    match value {
        StoragePolicy::DurableEncryptedEnvelopes => 0,
        StoragePolicy::TransientOnly => 1,
    }
}
