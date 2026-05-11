use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    DescriptorSignatureVerifier, DescriptorValidationContext, DiscoveryPath, DmForwardingPolicy,
    NodeDescriptor, NodeDescriptorValidationError, PeeringPolicy,
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
}
