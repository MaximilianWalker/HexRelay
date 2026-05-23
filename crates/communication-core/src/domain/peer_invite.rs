use serde::{Deserialize, Serialize};

use super::{
    DiscoveryPath, PeeringPolicy, ServerDescriptor, ServerDescriptorValidationError,
    ServerSignature, ServerSignatureAlgorithm,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerInviteEnvelope {
    pub issuer_descriptor: ServerDescriptor,
    pub invite: PeerInvite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerInvite {
    pub invite_id: String,
    pub issuer_server_id: String,
    pub issuer_descriptor_id: String,
    pub subject_server_id: Option<String>,
    pub issued_at_epoch_seconds: i64,
    pub expires_at_epoch_seconds: i64,
    pub discovery_path: DiscoveryPath,
    pub peering_policy: PeeringPolicy,
    pub max_uses: Option<u32>,
    pub signature: ServerSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerInviteValidationContext {
    pub now_epoch_seconds: i64,
    pub max_ttl_seconds: i64,
    pub revoked_invite_ids: Vec<String>,
    pub expected_subject_server_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerInviteValidationError {
    MissingField(&'static str),
    InvalidTimeRange,
    InviteExpired,
    InviteTtlTooLong {
        ttl_seconds: i64,
        max_seconds: i64,
    },
    InviteRevoked,
    SignatureRequired,
    InvalidMaxUses,
    IssuerServerMismatch,
    IssuerDescriptorMismatch,
    SubjectServerMismatch {
        expected_subject_server_id: Option<String>,
        invite_subject_server_id: String,
    },
    DiscoveryExposureRefused(ServerDescriptorValidationError),
    PeeringPolicyRefused {
        issuer_peering_policy: PeeringPolicy,
        invite_peering_policy: PeeringPolicy,
    },
}

impl PeerInvite {
    pub fn validate(
        &self,
        issuer_descriptor: &ServerDescriptor,
        context: &PeerInviteValidationContext,
    ) -> Result<(), PeerInviteValidationError> {
        validate_required(&self.invite_id, "invite_id")?;
        validate_required(&self.issuer_server_id, "issuer_server_id")?;
        validate_required(&self.issuer_descriptor_id, "issuer_descriptor_id")?;
        validate_required(&self.signature.value, "signature")?;

        if let Some(subject_server_id) = &self.subject_server_id {
            validate_required(subject_server_id, "subject_server_id")?;
        }

        if self.issued_at_epoch_seconds >= self.expires_at_epoch_seconds {
            return Err(PeerInviteValidationError::InvalidTimeRange);
        }

        if self.expires_at_epoch_seconds <= context.now_epoch_seconds {
            return Err(PeerInviteValidationError::InviteExpired);
        }

        let ttl_seconds = self.expires_at_epoch_seconds - self.issued_at_epoch_seconds;
        if ttl_seconds > context.max_ttl_seconds {
            return Err(PeerInviteValidationError::InviteTtlTooLong {
                ttl_seconds,
                max_seconds: context.max_ttl_seconds,
            });
        }

        if context
            .revoked_invite_ids
            .iter()
            .any(|revoked| revoked == &self.invite_id)
        {
            return Err(PeerInviteValidationError::InviteRevoked);
        }

        if self.signature.algorithm != ServerSignatureAlgorithm::Ed25519 {
            return Err(PeerInviteValidationError::SignatureRequired);
        }

        if self.max_uses == Some(0) {
            return Err(PeerInviteValidationError::InvalidMaxUses);
        }

        if self.issuer_server_id != issuer_descriptor.server_id {
            return Err(PeerInviteValidationError::IssuerServerMismatch);
        }

        if self.issuer_descriptor_id != issuer_descriptor.descriptor_id {
            return Err(PeerInviteValidationError::IssuerDescriptorMismatch);
        }

        if let Some(subject_server_id) = &self.subject_server_id {
            if context.expected_subject_server_id.as_deref() != Some(subject_server_id.as_str()) {
                return Err(PeerInviteValidationError::SubjectServerMismatch {
                    expected_subject_server_id: context.expected_subject_server_id.clone(),
                    invite_subject_server_id: subject_server_id.clone(),
                });
            }
        }

        issuer_descriptor
            .validate_discovery_exposure(self.discovery_path)
            .map_err(PeerInviteValidationError::DiscoveryExposureRefused)?;

        if issuer_descriptor.peering_policy != PeeringPolicy::InviteToken
            || self.peering_policy != PeeringPolicy::InviteToken
        {
            return Err(PeerInviteValidationError::PeeringPolicyRefused {
                issuer_peering_policy: issuer_descriptor.peering_policy,
                invite_peering_policy: self.peering_policy,
            });
        }

        Ok(())
    }
}

fn validate_required(value: &str, field: &'static str) -> Result<(), PeerInviteValidationError> {
    if value.trim().is_empty() {
        Err(PeerInviteValidationError::MissingField(field))
    } else {
        Ok(())
    }
}
