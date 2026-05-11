use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    Offline,
    LocalOnly,
    LanOnly,
    PrivatePeers,
    PublicDiscovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryPolicy {
    None,
    LanAnnounce,
    PrivateAllowlist,
    MemberVisible,
    UserConsentedIntroduction,
    PublicRegistry,
    PublicDht,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryPath {
    LanAnnounce,
    PrivateAllowlist,
    MemberVisible,
    UserConsentedIntroduction,
    PublicRegistry,
    PublicDht,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeeringPolicy {
    None,
    StaticAllowlist,
    InviteToken,
    MemberIntroduced,
    PublicAuthenticated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelayPolicy {
    None,
    OwnUsersOnly,
    AllowlistedPeers,
    OpenLimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DmForwardingPolicy {
    Disabled,
    LocalRecipientsOnly,
    AllowlistedRoute,
    RelayAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoragePolicy {
    TransientOnly,
    DurableEncryptedEnvelopes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeSignatureAlgorithm {
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSignature {
    pub algorithm: NodeSignatureAlgorithm,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitScope {
    Node,
    Peer,
    User,
    Route,
    DescriptorSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRateLimit {
    pub scope: RateLimitScope,
    pub max_per_minute: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub node_id: String,
    pub node_public_key: String,
    pub descriptor_id: String,
    pub issued_at_epoch_seconds: i64,
    pub expires_at_epoch_seconds: i64,
    pub network_mode: NetworkMode,
    pub discovery_policy: DiscoveryPolicy,
    pub peering_policy: PeeringPolicy,
    pub relay_policy: RelayPolicy,
    pub dm_forwarding_policy: DmForwardingPolicy,
    pub storage_policy: StoragePolicy,
    pub addresses: Vec<String>,
    pub supported_protocols: Vec<String>,
    pub rate_limits: Vec<NodeRateLimit>,
    pub trust_labels: Vec<String>,
    pub revocation_pointer: Option<String>,
    pub signature: NodeSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorValidationContext {
    pub now_epoch_seconds: i64,
    pub max_ttl_seconds: i64,
    pub revoked_descriptor_ids: Vec<String>,
}

pub trait DescriptorSignatureVerifier {
    fn verify(&self, descriptor: &NodeDescriptor) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeDescriptorValidationError {
    MissingField(&'static str),
    InvalidTimeRange,
    DescriptorExpired,
    DescriptorTtlTooLong {
        ttl_seconds: i64,
        max_seconds: i64,
    },
    DescriptorRevoked,
    SignatureRequired,
    SignatureVerificationFailed,
    AddressRequired,
    InvalidRateLimit,
    DiscoveryPolicyConflict {
        network_mode: NetworkMode,
        discovery_policy: DiscoveryPolicy,
    },
    PeeringPolicyConflict {
        network_mode: NetworkMode,
        peering_policy: PeeringPolicy,
    },
    RelayPolicyConflict {
        relay_policy: RelayPolicy,
        dm_forwarding_policy: DmForwardingPolicy,
    },
    DmForwardingPolicyConflict {
        relay_policy: RelayPolicy,
        dm_forwarding_policy: DmForwardingPolicy,
    },
    DiscoveryExposureRefused {
        requested_path: DiscoveryPath,
        discovery_policy: DiscoveryPolicy,
    },
    RelayRefused {
        relay_policy: RelayPolicy,
    },
}

impl NodeDescriptor {
    pub fn validate(
        &self,
        context: &DescriptorValidationContext,
    ) -> Result<(), NodeDescriptorValidationError> {
        validate_required(&self.node_id, "node_id")?;
        validate_required(&self.node_public_key, "node_public_key")?;
        validate_required(&self.descriptor_id, "descriptor_id")?;
        validate_required(&self.signature.value, "signature")?;

        if self.issued_at_epoch_seconds >= self.expires_at_epoch_seconds {
            return Err(NodeDescriptorValidationError::InvalidTimeRange);
        }

        if self.expires_at_epoch_seconds <= context.now_epoch_seconds {
            return Err(NodeDescriptorValidationError::DescriptorExpired);
        }

        let ttl_seconds = self.expires_at_epoch_seconds - self.issued_at_epoch_seconds;
        if ttl_seconds > context.max_ttl_seconds {
            return Err(NodeDescriptorValidationError::DescriptorTtlTooLong {
                ttl_seconds,
                max_seconds: context.max_ttl_seconds,
            });
        }

        if context
            .revoked_descriptor_ids
            .iter()
            .any(|revoked| revoked == &self.descriptor_id)
        {
            return Err(NodeDescriptorValidationError::DescriptorRevoked);
        }

        if self.signature.algorithm != NodeSignatureAlgorithm::Ed25519 {
            return Err(NodeDescriptorValidationError::SignatureRequired);
        }

        self.validate_policy_shape()?;

        if self
            .rate_limits
            .iter()
            .any(|limit| limit.max_per_minute == 0)
        {
            return Err(NodeDescriptorValidationError::InvalidRateLimit);
        }

        if self.requires_reachable_address()
            && self
                .addresses
                .iter()
                .all(|address| address.trim().is_empty())
        {
            return Err(NodeDescriptorValidationError::AddressRequired);
        }

        Ok(())
    }

    pub fn validate_with_signature<V: DescriptorSignatureVerifier>(
        &self,
        context: &DescriptorValidationContext,
        verifier: &V,
    ) -> Result<(), NodeDescriptorValidationError> {
        self.validate(context)?;

        if verifier.verify(self) {
            Ok(())
        } else {
            Err(NodeDescriptorValidationError::SignatureVerificationFailed)
        }
    }

    pub fn validate_discovery_exposure(
        &self,
        requested_path: DiscoveryPath,
    ) -> Result<(), NodeDescriptorValidationError> {
        if self.discovery_policy.allows_path(requested_path) {
            Ok(())
        } else {
            Err(NodeDescriptorValidationError::DiscoveryExposureRefused {
                requested_path,
                discovery_policy: self.discovery_policy,
            })
        }
    }

    pub fn validate_relay_use(&self) -> Result<(), NodeDescriptorValidationError> {
        if self.allows_relay() {
            Ok(())
        } else {
            Err(NodeDescriptorValidationError::RelayRefused {
                relay_policy: self.relay_policy,
            })
        }
    }

    pub fn allows_relay(&self) -> bool {
        self.relay_policy != RelayPolicy::None
            && matches!(
                self.dm_forwarding_policy,
                DmForwardingPolicy::AllowlistedRoute | DmForwardingPolicy::RelayAllowed
            )
    }

    pub fn allows_intermediate_relay(&self) -> bool {
        matches!(
            self.relay_policy,
            RelayPolicy::AllowlistedPeers | RelayPolicy::OpenLimited
        ) && matches!(
            self.dm_forwarding_policy,
            DmForwardingPolicy::AllowlistedRoute | DmForwardingPolicy::RelayAllowed
        )
    }

    pub fn accepts_local_recipient_delivery(&self) -> bool {
        matches!(
            self.dm_forwarding_policy,
            DmForwardingPolicy::LocalRecipientsOnly
                | DmForwardingPolicy::AllowlistedRoute
                | DmForwardingPolicy::RelayAllowed
        )
    }

    pub fn can_be_user_introduced(&self) -> bool {
        self.discovery_policy
            .allows_path(DiscoveryPath::UserConsentedIntroduction)
    }

    fn requires_reachable_address(&self) -> bool {
        if matches!(
            self.network_mode,
            NetworkMode::Offline | NetworkMode::LocalOnly
        ) {
            return false;
        }

        self.discovery_policy != DiscoveryPolicy::None
            || self.peering_policy != PeeringPolicy::None
            || self.dm_forwarding_policy != DmForwardingPolicy::Disabled
    }

    fn validate_policy_shape(&self) -> Result<(), NodeDescriptorValidationError> {
        match self.network_mode {
            NetworkMode::Offline => {
                if self.discovery_policy != DiscoveryPolicy::None {
                    return Err(NodeDescriptorValidationError::DiscoveryPolicyConflict {
                        network_mode: self.network_mode,
                        discovery_policy: self.discovery_policy,
                    });
                }

                if self.peering_policy != PeeringPolicy::None {
                    return Err(NodeDescriptorValidationError::PeeringPolicyConflict {
                        network_mode: self.network_mode,
                        peering_policy: self.peering_policy,
                    });
                }

                if self.relay_policy != RelayPolicy::None {
                    return Err(NodeDescriptorValidationError::RelayPolicyConflict {
                        relay_policy: self.relay_policy,
                        dm_forwarding_policy: self.dm_forwarding_policy,
                    });
                }

                if self.dm_forwarding_policy != DmForwardingPolicy::Disabled {
                    return Err(NodeDescriptorValidationError::DmForwardingPolicyConflict {
                        relay_policy: self.relay_policy,
                        dm_forwarding_policy: self.dm_forwarding_policy,
                    });
                }
            }
            NetworkMode::LocalOnly => {
                if self.discovery_policy != DiscoveryPolicy::None {
                    return Err(NodeDescriptorValidationError::DiscoveryPolicyConflict {
                        network_mode: self.network_mode,
                        discovery_policy: self.discovery_policy,
                    });
                }

                if self.peering_policy != PeeringPolicy::None {
                    return Err(NodeDescriptorValidationError::PeeringPolicyConflict {
                        network_mode: self.network_mode,
                        peering_policy: self.peering_policy,
                    });
                }

                if self.relay_policy != RelayPolicy::None {
                    return Err(NodeDescriptorValidationError::RelayPolicyConflict {
                        relay_policy: self.relay_policy,
                        dm_forwarding_policy: self.dm_forwarding_policy,
                    });
                }

                if self.dm_forwarding_policy == DmForwardingPolicy::RelayAllowed {
                    return Err(NodeDescriptorValidationError::DmForwardingPolicyConflict {
                        relay_policy: self.relay_policy,
                        dm_forwarding_policy: self.dm_forwarding_policy,
                    });
                }
            }
            NetworkMode::LanOnly => {
                if matches!(
                    self.discovery_policy,
                    DiscoveryPolicy::PublicRegistry
                        | DiscoveryPolicy::PublicDht
                        | DiscoveryPolicy::UserConsentedIntroduction
                ) {
                    return Err(NodeDescriptorValidationError::DiscoveryPolicyConflict {
                        network_mode: self.network_mode,
                        discovery_policy: self.discovery_policy,
                    });
                }

                if self.peering_policy == PeeringPolicy::PublicAuthenticated {
                    return Err(NodeDescriptorValidationError::PeeringPolicyConflict {
                        network_mode: self.network_mode,
                        peering_policy: self.peering_policy,
                    });
                }
            }
            NetworkMode::PrivatePeers => {
                if matches!(
                    self.discovery_policy,
                    DiscoveryPolicy::LanAnnounce
                        | DiscoveryPolicy::PublicRegistry
                        | DiscoveryPolicy::PublicDht
                ) {
                    return Err(NodeDescriptorValidationError::DiscoveryPolicyConflict {
                        network_mode: self.network_mode,
                        discovery_policy: self.discovery_policy,
                    });
                }

                if self.peering_policy == PeeringPolicy::PublicAuthenticated {
                    return Err(NodeDescriptorValidationError::PeeringPolicyConflict {
                        network_mode: self.network_mode,
                        peering_policy: self.peering_policy,
                    });
                }
            }
            NetworkMode::PublicDiscovery => {
                if self.discovery_policy == DiscoveryPolicy::LanAnnounce {
                    return Err(NodeDescriptorValidationError::DiscoveryPolicyConflict {
                        network_mode: self.network_mode,
                        discovery_policy: self.discovery_policy,
                    });
                }
            }
        }

        if self.relay_policy == RelayPolicy::None
            && self.dm_forwarding_policy == DmForwardingPolicy::RelayAllowed
        {
            return Err(NodeDescriptorValidationError::DmForwardingPolicyConflict {
                relay_policy: self.relay_policy,
                dm_forwarding_policy: self.dm_forwarding_policy,
            });
        }

        if self.relay_policy != RelayPolicy::None
            && !matches!(
                self.dm_forwarding_policy,
                DmForwardingPolicy::AllowlistedRoute | DmForwardingPolicy::RelayAllowed
            )
        {
            return Err(NodeDescriptorValidationError::RelayPolicyConflict {
                relay_policy: self.relay_policy,
                dm_forwarding_policy: self.dm_forwarding_policy,
            });
        }

        Ok(())
    }
}

impl DiscoveryPolicy {
    pub fn allows_path(self, requested_path: DiscoveryPath) -> bool {
        matches!(
            (self, requested_path),
            (DiscoveryPolicy::LanAnnounce, DiscoveryPath::LanAnnounce)
                | (
                    DiscoveryPolicy::PrivateAllowlist,
                    DiscoveryPath::PrivateAllowlist
                )
                | (DiscoveryPolicy::MemberVisible, DiscoveryPath::MemberVisible)
                | (
                    DiscoveryPolicy::UserConsentedIntroduction,
                    DiscoveryPath::UserConsentedIntroduction
                )
                | (
                    DiscoveryPolicy::PublicRegistry,
                    DiscoveryPath::PublicRegistry
                )
                | (DiscoveryPolicy::PublicDht, DiscoveryPath::PublicDht)
        )
    }
}

fn validate_required(
    value: &str,
    field: &'static str,
) -> Result<(), NodeDescriptorValidationError> {
    if value.trim().is_empty() {
        Err(NodeDescriptorValidationError::MissingField(field))
    } else {
        Ok(())
    }
}
