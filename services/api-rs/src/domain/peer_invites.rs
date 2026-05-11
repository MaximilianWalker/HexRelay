use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use communication_core::{
    sign_peer_invite_ed25519_pkcs8, verify_peer_invite_ed25519, DescriptorValidationContext,
    DiscoveryPath, Ed25519DescriptorVerifier, NodeDescriptor, NodeSignature,
    NodeSignatureAlgorithm, PeerInvite, PeerInviteEnvelope, PeerInviteSignatureError,
    PeerInviteValidationContext, PeerInviteValidationError, PeeringPolicy,
};
use uuid::Uuid;

use crate::domain::node_identity::LocalNodeIdentity;

pub const DEFAULT_PEER_INVITE_TTL_SECONDS: i64 = 3_600;
pub const DEFAULT_PEER_INVITE_MAX_TTL_SECONDS: i64 = 86_400;
pub const DEFAULT_PEER_INVITE_MAX_USES: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerInviteIssueOptions {
    pub invite_id: Option<String>,
    pub subject_node_id: Option<String>,
    pub allow_unbound: bool,
    pub ttl_seconds: i64,
    pub max_ttl_seconds: i64,
    pub discovery_path: DiscoveryPath,
    pub max_uses: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerInviteIssueCliOptions {
    pub issue_options: PeerInviteIssueOptions,
    pub max_ttl_seconds_override: Option<i64>,
    pub json_compact: bool,
}

#[derive(Debug)]
pub enum PeerInviteIssueError {
    InvalidArgs(String),
    InvalidClock(String),
    DescriptorInvalid(String),
    InviteInvalid(PeerInviteValidationError),
    Signature(PeerInviteSignatureError),
}

impl fmt::Display for PeerInviteIssueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(message) => write!(f, "{message}"),
            Self::InvalidClock(message) => write!(f, "{message}"),
            Self::DescriptorInvalid(message) => write!(f, "{message}"),
            Self::InviteInvalid(error) => write!(f, "peer invite validation failed: {error:?}"),
            Self::Signature(error) => write!(f, "peer invite signing failed: {error:?}"),
        }
    }
}

impl std::error::Error for PeerInviteIssueError {}

impl PeerInviteIssueCliOptions {
    pub fn parse<I>(args: I) -> Result<Self, PeerInviteIssueError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut invite_id = None;
        let mut subject_node_id = None;
        let mut allow_unbound = false;
        let mut ttl_seconds = DEFAULT_PEER_INVITE_TTL_SECONDS;
        let mut max_ttl_seconds_override = None;
        let mut discovery_path = DiscoveryPath::PrivateAllowlist;
        let mut max_uses = Some(DEFAULT_PEER_INVITE_MAX_USES);
        let mut json_compact = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--invite-id" => {
                    invite_id = Some(args.next().ok_or_else(|| {
                        PeerInviteIssueError::InvalidArgs(
                            "--invite-id requires a value".to_string(),
                        )
                    })?);
                }
                "--subject-node-id" => {
                    subject_node_id = Some(args.next().ok_or_else(|| {
                        PeerInviteIssueError::InvalidArgs(
                            "--subject-node-id requires a value".to_string(),
                        )
                    })?);
                }
                "--allow-unbound" => allow_unbound = true,
                "--ttl-seconds" => {
                    ttl_seconds = parse_i64_arg("--ttl-seconds", args.next())?;
                }
                "--max-ttl-seconds" => {
                    max_ttl_seconds_override =
                        Some(parse_i64_arg("--max-ttl-seconds", args.next())?);
                }
                "--discovery-path" => {
                    discovery_path = parse_discovery_path(&args.next().ok_or_else(|| {
                        PeerInviteIssueError::InvalidArgs(
                            "--discovery-path requires a value".to_string(),
                        )
                    })?)?;
                }
                "--max-uses" => {
                    max_uses = Some(parse_u32_arg("--max-uses", args.next())?);
                }
                "--unlimited-uses" => max_uses = None,
                "--compact" => json_compact = true,
                "--help" | "-h" => {
                    return Err(PeerInviteIssueError::InvalidArgs(
                        peer_invite_issue_usage().to_string(),
                    ));
                }
                value if value.starts_with('-') => {
                    return Err(PeerInviteIssueError::InvalidArgs(format!(
                        "unknown peer invite option: {value}\n{}",
                        peer_invite_issue_usage()
                    )));
                }
                value => {
                    return Err(PeerInviteIssueError::InvalidArgs(format!(
                        "unexpected positional argument: {value}\n{}",
                        peer_invite_issue_usage()
                    )));
                }
            }
        }

        if invite_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(PeerInviteIssueError::InvalidArgs(
                "--invite-id must not be empty".to_string(),
            ));
        }

        if subject_node_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(PeerInviteIssueError::InvalidArgs(
                "--subject-node-id must not be empty".to_string(),
            ));
        }

        if subject_node_id.is_none() && !allow_unbound {
            return Err(PeerInviteIssueError::InvalidArgs(
                "--subject-node-id is required unless --allow-unbound is set".to_string(),
            ));
        }

        validate_positive_i64("--ttl-seconds", ttl_seconds)?;
        if let Some(max_ttl_seconds) = max_ttl_seconds_override {
            validate_positive_i64("--max-ttl-seconds", max_ttl_seconds)?;
        }
        let effective_max_ttl_seconds =
            max_ttl_seconds_override.unwrap_or(DEFAULT_PEER_INVITE_MAX_TTL_SECONDS);
        if ttl_seconds > effective_max_ttl_seconds {
            return Err(PeerInviteIssueError::InvalidArgs(format!(
                "--ttl-seconds must be less than or equal to --max-ttl-seconds ({effective_max_ttl_seconds})"
            )));
        }

        if max_uses == Some(0) {
            return Err(PeerInviteIssueError::InvalidArgs(
                "--max-uses must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            issue_options: PeerInviteIssueOptions {
                invite_id,
                subject_node_id,
                allow_unbound,
                ttl_seconds,
                max_ttl_seconds: effective_max_ttl_seconds,
                discovery_path,
                max_uses,
            },
            max_ttl_seconds_override,
            json_compact,
        })
    }
}

pub fn peer_invite_issue_usage() -> &'static str {
    "Usage: issue_peer_invite --subject-node-id NODE_ID [--invite-id ID] [--ttl-seconds 3600] [--max-ttl-seconds 86400] [--discovery-path private_allowlist] [--max-uses 1|--unlimited-uses] [--compact]\n       issue_peer_invite --allow-unbound [same options]"
}

pub fn issue_peer_invite(
    local_identity: &LocalNodeIdentity,
    options: &PeerInviteIssueOptions,
    issued_at_epoch_seconds: i64,
) -> Result<PeerInviteEnvelope, PeerInviteIssueError> {
    validate_issue_options(options)?;
    validate_issuer_descriptor(
        &local_identity.descriptor,
        issued_at_epoch_seconds,
        options.max_ttl_seconds,
    )?;

    let invite_id = options
        .invite_id
        .clone()
        .unwrap_or_else(|| format!("peer-invite-{}", Uuid::new_v4().simple()));
    let mut invite = PeerInvite {
        invite_id,
        issuer_node_id: local_identity.descriptor.node_id.clone(),
        issuer_descriptor_id: local_identity.descriptor.descriptor_id.clone(),
        subject_node_id: options.subject_node_id.clone(),
        issued_at_epoch_seconds,
        expires_at_epoch_seconds: issued_at_epoch_seconds + options.ttl_seconds,
        discovery_path: options.discovery_path,
        peering_policy: PeeringPolicy::InviteToken,
        max_uses: options.max_uses,
        signature: NodeSignature {
            algorithm: NodeSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    };

    invite.signature.value =
        sign_peer_invite_ed25519_pkcs8(&invite, &local_identity.private_key_pkcs8)
            .map_err(PeerInviteIssueError::Signature)?;

    let context = PeerInviteValidationContext {
        now_epoch_seconds: issued_at_epoch_seconds,
        max_ttl_seconds: options.max_ttl_seconds,
        revoked_invite_ids: Vec::new(),
        expected_subject_node_id: options.subject_node_id.clone(),
    };
    invite
        .validate(&local_identity.descriptor, &context)
        .map_err(PeerInviteIssueError::InviteInvalid)?;
    verify_peer_invite_ed25519(&invite, &local_identity.descriptor)
        .map_err(PeerInviteIssueError::Signature)?;

    Ok(PeerInviteEnvelope {
        issuer_descriptor: local_identity.descriptor.clone(),
        invite,
    })
}

pub fn current_epoch_seconds() -> Result<i64, PeerInviteIssueError> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| {
            PeerInviteIssueError::InvalidClock("system clock is before UNIX epoch".to_string())
        })?
        .as_secs();

    i64::try_from(seconds).map_err(|_| {
        PeerInviteIssueError::InvalidClock("system clock value is too large".to_string())
    })
}

fn validate_issue_options(options: &PeerInviteIssueOptions) -> Result<(), PeerInviteIssueError> {
    if options
        .invite_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(PeerInviteIssueError::InvalidArgs(
            "invite_id must not be empty".to_string(),
        ));
    }

    if options
        .subject_node_id
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(PeerInviteIssueError::InvalidArgs(
            "subject_node_id must not be empty".to_string(),
        ));
    }

    if options.subject_node_id.is_none() && !options.allow_unbound {
        return Err(PeerInviteIssueError::InvalidArgs(
            "subject_node_id is required unless allow_unbound is true".to_string(),
        ));
    }

    validate_positive_i64("ttl_seconds", options.ttl_seconds)?;
    validate_positive_i64("max_ttl_seconds", options.max_ttl_seconds)?;
    if options.ttl_seconds > options.max_ttl_seconds {
        return Err(PeerInviteIssueError::InvalidArgs(
            "ttl_seconds must not exceed max_ttl_seconds".to_string(),
        ));
    }

    if options.max_uses == Some(0) {
        return Err(PeerInviteIssueError::InvalidArgs(
            "max_uses must be greater than zero".to_string(),
        ));
    }

    Ok(())
}

fn validate_issuer_descriptor(
    descriptor: &NodeDescriptor,
    now_epoch_seconds: i64,
    max_ttl_seconds: i64,
) -> Result<(), PeerInviteIssueError> {
    let context = DescriptorValidationContext {
        now_epoch_seconds,
        max_ttl_seconds,
        revoked_descriptor_ids: Vec::new(),
    };
    descriptor
        .validate_with_signature(&context, &Ed25519DescriptorVerifier)
        .map_err(|error| {
            PeerInviteIssueError::DescriptorInvalid(format!(
                "local node descriptor '{}' is invalid: {error:?}",
                descriptor.descriptor_id
            ))
        })?;

    Ok(())
}

fn parse_i64_arg(name: &'static str, value: Option<String>) -> Result<i64, PeerInviteIssueError> {
    value
        .ok_or_else(|| PeerInviteIssueError::InvalidArgs(format!("{name} requires a value")))?
        .trim()
        .parse::<i64>()
        .map_err(|_| PeerInviteIssueError::InvalidArgs(format!("{name} must be an integer")))
}

fn parse_u32_arg(name: &'static str, value: Option<String>) -> Result<u32, PeerInviteIssueError> {
    value
        .ok_or_else(|| PeerInviteIssueError::InvalidArgs(format!("{name} requires a value")))?
        .trim()
        .parse::<u32>()
        .map_err(|_| PeerInviteIssueError::InvalidArgs(format!("{name} must be an integer")))
}

fn validate_positive_i64(name: &'static str, value: i64) -> Result<(), PeerInviteIssueError> {
    if value > 0 {
        Ok(())
    } else {
        Err(PeerInviteIssueError::InvalidArgs(format!(
            "{name} must be greater than zero"
        )))
    }
}

fn parse_discovery_path(value: &str) -> Result<DiscoveryPath, PeerInviteIssueError> {
    match value.trim() {
        "lan_announce" => Ok(DiscoveryPath::LanAnnounce),
        "private_allowlist" => Ok(DiscoveryPath::PrivateAllowlist),
        "member_visible" => Ok(DiscoveryPath::MemberVisible),
        "user_consented_introduction" => Ok(DiscoveryPath::UserConsentedIntroduction),
        "public_registry" => Ok(DiscoveryPath::PublicRegistry),
        "public_dht" => Ok(DiscoveryPath::PublicDht),
        _ => Err(PeerInviteIssueError::InvalidArgs(format!(
            "unsupported --discovery-path '{}'",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        issue_peer_invite, PeerInviteIssueCliOptions, PeerInviteIssueOptions,
        DEFAULT_PEER_INVITE_MAX_TTL_SECONDS,
    };
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, verify_peer_invite_ed25519,
        DiscoveryPath, DiscoveryPolicy, DmForwardingPolicy, NetworkMode, NodeDescriptor,
        NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RelayPolicy, StoragePolicy,
    };
    use ring::{rand::SystemRandom, signature::Ed25519KeyPair};

    use crate::domain::node_identity::LocalNodeIdentity;

    fn local_identity() -> LocalNodeIdentity {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let mut descriptor = NodeDescriptor {
            node_id: "node-inviter".to_string(),
            node_public_key: public_key,
            descriptor_id: "descriptor-inviter".to_string(),
            issued_at_epoch_seconds: 1_700_000_000,
            expires_at_epoch_seconds: 1_700_000_600,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy: PeeringPolicy::InviteToken,
            relay_policy: RelayPolicy::None,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec!["https://node-inviter.example".to_string()],
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

        LocalNodeIdentity {
            descriptor,
            private_key_pkcs8: pkcs8.as_ref().to_vec(),
        }
    }

    fn default_options() -> PeerInviteIssueOptions {
        PeerInviteIssueOptions {
            invite_id: Some("peer-invite-test".to_string()),
            subject_node_id: Some("node-recipient".to_string()),
            allow_unbound: false,
            ttl_seconds: 300,
            max_ttl_seconds: DEFAULT_PEER_INVITE_MAX_TTL_SECONDS,
            discovery_path: DiscoveryPath::PrivateAllowlist,
            max_uses: Some(1),
        }
    }

    #[test]
    fn issues_subject_bound_signed_peer_invite_envelope() {
        let identity = local_identity();
        let envelope =
            issue_peer_invite(&identity, &default_options(), 1_700_000_010).expect("issue invite");

        assert_eq!(envelope.invite.invite_id, "peer-invite-test");
        assert_eq!(
            envelope.invite.subject_node_id.as_deref(),
            Some("node-recipient")
        );
        assert_eq!(envelope.invite.issuer_node_id, "node-inviter");
        assert_eq!(envelope.invite.max_uses, Some(1));
        verify_peer_invite_ed25519(&envelope.invite, &envelope.issuer_descriptor)
            .expect("signature verifies");
    }

    #[test]
    fn cli_requires_subject_unless_unbound_is_explicit() {
        let err = PeerInviteIssueCliOptions::parse(Vec::<String>::new())
            .expect_err("missing subject should fail");
        assert!(err
            .to_string()
            .contains("--subject-node-id is required unless --allow-unbound is set"));

        let parsed = PeerInviteIssueCliOptions::parse(["--allow-unbound".to_string()])
            .expect("unbound is explicit");
        assert!(parsed.issue_options.subject_node_id.is_none());
        assert!(parsed.issue_options.allow_unbound);
    }

    #[test]
    fn rejects_ttl_above_configured_limit() {
        let err = PeerInviteIssueCliOptions::parse([
            "--subject-node-id".to_string(),
            "node-recipient".to_string(),
            "--ttl-seconds".to_string(),
            "120".to_string(),
            "--max-ttl-seconds".to_string(),
            "60".to_string(),
        ])
        .expect_err("ttl above limit should fail");

        assert!(err
            .to_string()
            .contains("--ttl-seconds must be less than or equal"));
    }
}
