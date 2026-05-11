use std::fmt;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use communication_core::{
    ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, verify_descriptor_ed25519,
    DescriptorValidationContext, DiscoveryPolicy, DmForwardingPolicy, NetworkMode, NodeDescriptor,
    NodeSignature, NodeSignatureAlgorithm, PeeringPolicy, RelayPolicy, StoragePolicy,
};
use ring::{rand::SystemRandom, signature::Ed25519KeyPair};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocalNodeIdentity {
    pub descriptor: NodeDescriptor,
    pub private_key_pkcs8: Vec<u8>,
}

pub const DEFAULT_NODE_DESCRIPTOR_TTL_SECONDS: i64 = 86_400;
pub const DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS: i64 = 86_400;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeIdentityGenerateOptions {
    pub node_id: String,
    pub descriptor_id: Option<String>,
    pub ttl_seconds: i64,
    pub max_ttl_seconds: i64,
    pub network_mode: NetworkMode,
    pub discovery_policy: DiscoveryPolicy,
    pub peering_policy: PeeringPolicy,
    pub relay_policy: RelayPolicy,
    pub dm_forwarding_policy: DmForwardingPolicy,
    pub storage_policy: StoragePolicy,
    pub addresses: Vec<String>,
    pub supported_protocols: Vec<String>,
    pub trust_labels: Vec<String>,
    pub revocation_pointer: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeIdentityGenerateCliOptions {
    pub generate_options: NodeIdentityGenerateOptions,
    pub ttl_seconds_override: Option<i64>,
    pub max_ttl_seconds_override: Option<i64>,
    pub json_compact: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneratedNodeIdentity {
    pub api_local_node_descriptor_json: NodeDescriptor,
    pub api_local_node_private_key_pkcs8_base64: String,
}

#[derive(Debug)]
pub enum NodeIdentityGenerateError {
    InvalidArgs(String),
    KeyGenerationFailed,
    Signature(String),
    DescriptorInvalid(String),
}

impl fmt::Display for NodeIdentityGenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgs(message) => write!(f, "{message}"),
            Self::KeyGenerationFailed => write!(f, "failed to generate Ed25519 keypair"),
            Self::Signature(message) => write!(f, "{message}"),
            Self::DescriptorInvalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for NodeIdentityGenerateError {}

impl NodeIdentityGenerateCliOptions {
    pub fn parse<I>(args: I) -> Result<Self, NodeIdentityGenerateError>
    where
        I: IntoIterator<Item = String>,
    {
        let mut node_id = None;
        let mut descriptor_id = None;
        let mut ttl_seconds_override = None;
        let mut max_ttl_seconds_override = None;
        let mut network_mode = NetworkMode::PrivatePeers;
        let mut discovery_policy = DiscoveryPolicy::PrivateAllowlist;
        let mut peering_policy = PeeringPolicy::InviteToken;
        let mut relay_policy = RelayPolicy::None;
        let mut dm_forwarding_policy = DmForwardingPolicy::LocalRecipientsOnly;
        let mut storage_policy = StoragePolicy::DurableEncryptedEnvelopes;
        let mut addresses = Vec::new();
        let mut supported_protocols = vec!["hexrelay-node-http".to_string()];
        let mut trust_labels = Vec::new();
        let mut revocation_pointer = None;
        let mut json_compact = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--node-id" => {
                    node_id = Some(next_arg("--node-id", &mut args)?);
                }
                "--descriptor-id" => {
                    descriptor_id = Some(next_arg("--descriptor-id", &mut args)?);
                }
                "--ttl-seconds" => {
                    ttl_seconds_override = Some(parse_i64_arg("--ttl-seconds", &mut args)?);
                }
                "--max-ttl-seconds" => {
                    max_ttl_seconds_override = Some(parse_i64_arg("--max-ttl-seconds", &mut args)?);
                }
                "--network-mode" => {
                    network_mode = parse_network_mode(&next_arg("--network-mode", &mut args)?)?;
                }
                "--discovery-policy" => {
                    discovery_policy =
                        parse_discovery_policy(&next_arg("--discovery-policy", &mut args)?)?;
                }
                "--peering-policy" => {
                    peering_policy =
                        parse_peering_policy(&next_arg("--peering-policy", &mut args)?)?;
                }
                "--relay-policy" => {
                    relay_policy = parse_relay_policy(&next_arg("--relay-policy", &mut args)?)?;
                }
                "--dm-forwarding-policy" => {
                    dm_forwarding_policy = parse_dm_forwarding_policy(&next_arg(
                        "--dm-forwarding-policy",
                        &mut args,
                    )?)?;
                }
                "--storage-policy" => {
                    storage_policy =
                        parse_storage_policy(&next_arg("--storage-policy", &mut args)?)?;
                }
                "--address" => addresses.push(next_arg("--address", &mut args)?),
                "--protocol" => {
                    let protocol = next_arg("--protocol", &mut args)?;
                    if supported_protocols.len() == 1
                        && supported_protocols[0] == "hexrelay-node-http"
                    {
                        supported_protocols.clear();
                    }
                    supported_protocols.push(protocol);
                }
                "--trust-label" => trust_labels.push(next_arg("--trust-label", &mut args)?),
                "--revocation-pointer" => {
                    revocation_pointer = Some(next_arg("--revocation-pointer", &mut args)?);
                }
                "--compact" => json_compact = true,
                "--help" | "-h" => {
                    return Err(NodeIdentityGenerateError::InvalidArgs(
                        node_identity_generate_usage().to_string(),
                    ));
                }
                value if value.starts_with('-') => {
                    return Err(NodeIdentityGenerateError::InvalidArgs(format!(
                        "unknown node identity option: {value}\n{}",
                        node_identity_generate_usage()
                    )));
                }
                value => {
                    return Err(NodeIdentityGenerateError::InvalidArgs(format!(
                        "unexpected positional argument: {value}\n{}",
                        node_identity_generate_usage()
                    )));
                }
            }
        }

        let node_id = node_id.ok_or_else(|| {
            NodeIdentityGenerateError::InvalidArgs("--node-id is required".to_string())
        })?;

        validate_required_arg("--node-id", &node_id)?;
        if let Some(descriptor_id) = &descriptor_id {
            validate_required_arg("--descriptor-id", descriptor_id)?;
        }
        validate_repeated_values("--address", &addresses)?;
        validate_repeated_values("--protocol", &supported_protocols)?;
        validate_repeated_values("--trust-label", &trust_labels)?;
        if let Some(revocation_pointer) = &revocation_pointer {
            validate_required_arg("--revocation-pointer", revocation_pointer)?;
        }

        if let Some(max_ttl_seconds) = max_ttl_seconds_override {
            validate_positive_i64("--max-ttl-seconds", max_ttl_seconds)?;
        }
        let effective_max_ttl_seconds =
            max_ttl_seconds_override.unwrap_or(DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS);
        let ttl_seconds = ttl_seconds_override.unwrap_or(DEFAULT_NODE_DESCRIPTOR_TTL_SECONDS);
        validate_positive_i64("--ttl-seconds", ttl_seconds)?;
        if ttl_seconds > effective_max_ttl_seconds {
            return Err(NodeIdentityGenerateError::InvalidArgs(format!(
                "--ttl-seconds must be less than or equal to --max-ttl-seconds ({effective_max_ttl_seconds})"
            )));
        }

        let generate_options = NodeIdentityGenerateOptions {
            node_id,
            descriptor_id,
            ttl_seconds,
            max_ttl_seconds: effective_max_ttl_seconds,
            network_mode,
            discovery_policy,
            peering_policy,
            relay_policy,
            dm_forwarding_policy,
            storage_policy,
            addresses,
            supported_protocols,
            trust_labels,
            revocation_pointer,
        };
        validate_generation_options(&generate_options)?;

        Ok(Self {
            generate_options,
            ttl_seconds_override,
            max_ttl_seconds_override,
            json_compact,
        })
    }
}

pub fn node_identity_generate_usage() -> &'static str {
    "Usage: generate_node_identity --node-id NODE_ID --address URL [--descriptor-id ID] [--ttl-seconds 86400] [--network-mode private_peers] [--discovery-policy private_allowlist] [--peering-policy invite_token] [--relay-policy none] [--dm-forwarding-policy local_recipients_only] [--storage-policy durable_encrypted_envelopes] [--protocol hexrelay-node-http] [--trust-label LABEL] [--revocation-pointer URL] [--compact]"
}

pub fn generate_node_identity(
    options: &NodeIdentityGenerateOptions,
    issued_at_epoch_seconds: i64,
) -> Result<(GeneratedNodeIdentity, LocalNodeIdentity), NodeIdentityGenerateError> {
    validate_generation_options(options)?;

    let private_key_pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .map_err(|_| NodeIdentityGenerateError::KeyGenerationFailed)?;
    let private_key_pkcs8 = private_key_pkcs8.as_ref().to_vec();
    let node_public_key = ed25519_public_key_hex(&private_key_pkcs8).map_err(|error| {
        NodeIdentityGenerateError::Signature(format!(
            "failed to derive generated Ed25519 public key: {error:?}"
        ))
    })?;
    let mut descriptor = NodeDescriptor {
        node_id: options.node_id.clone(),
        node_public_key,
        descriptor_id: options
            .descriptor_id
            .clone()
            .unwrap_or_else(|| format!("descriptor-{}", Uuid::new_v4().simple())),
        issued_at_epoch_seconds,
        expires_at_epoch_seconds: issued_at_epoch_seconds + options.ttl_seconds,
        network_mode: options.network_mode,
        discovery_policy: options.discovery_policy,
        peering_policy: options.peering_policy,
        relay_policy: options.relay_policy,
        dm_forwarding_policy: options.dm_forwarding_policy,
        storage_policy: options.storage_policy,
        addresses: options.addresses.clone(),
        supported_protocols: options.supported_protocols.clone(),
        rate_limits: Vec::new(),
        trust_labels: options.trust_labels.clone(),
        revocation_pointer: options.revocation_pointer.clone(),
        signature: NodeSignature {
            algorithm: NodeSignatureAlgorithm::Ed25519,
            value: String::new(),
        },
    };
    descriptor.signature.value = sign_descriptor_ed25519_pkcs8(&descriptor, &private_key_pkcs8)
        .map_err(|error| {
            NodeIdentityGenerateError::Signature(format!(
                "failed to sign generated node descriptor: {error:?}"
            ))
        })?;
    validate_generated_descriptor(
        &descriptor,
        issued_at_epoch_seconds,
        options.max_ttl_seconds,
    )?;
    verify_descriptor_ed25519(&descriptor).map_err(|error| {
        NodeIdentityGenerateError::Signature(format!(
            "generated node descriptor signature did not verify: {error:?}"
        ))
    })?;

    let output = GeneratedNodeIdentity {
        api_local_node_descriptor_json: descriptor.clone(),
        api_local_node_private_key_pkcs8_base64: BASE64.encode(&private_key_pkcs8),
    };
    let identity = LocalNodeIdentity {
        descriptor,
        private_key_pkcs8,
    };

    Ok((output, identity))
}

fn validate_generation_options(
    options: &NodeIdentityGenerateOptions,
) -> Result<(), NodeIdentityGenerateError> {
    validate_required_arg("node_id", &options.node_id)?;
    if let Some(descriptor_id) = &options.descriptor_id {
        validate_required_arg("descriptor_id", descriptor_id)?;
    }
    validate_positive_i64("ttl_seconds", options.ttl_seconds)?;
    validate_positive_i64("max_ttl_seconds", options.max_ttl_seconds)?;
    if options.ttl_seconds > options.max_ttl_seconds {
        return Err(NodeIdentityGenerateError::InvalidArgs(
            "ttl_seconds must not exceed max_ttl_seconds".to_string(),
        ));
    }
    validate_repeated_values("addresses", &options.addresses)?;
    validate_repeated_values("supported_protocols", &options.supported_protocols)?;
    validate_repeated_values("trust_labels", &options.trust_labels)?;
    if let Some(revocation_pointer) = &options.revocation_pointer {
        validate_required_arg("revocation_pointer", revocation_pointer)?;
    }

    Ok(())
}

fn validate_generated_descriptor(
    descriptor: &NodeDescriptor,
    now_epoch_seconds: i64,
    max_ttl_seconds: i64,
) -> Result<(), NodeIdentityGenerateError> {
    let context = DescriptorValidationContext {
        now_epoch_seconds,
        max_ttl_seconds,
        revoked_descriptor_ids: Vec::new(),
    };
    descriptor.validate(&context).map_err(|error| {
        NodeIdentityGenerateError::DescriptorInvalid(format!(
            "generated node descriptor '{}' is invalid: {error:?}",
            descriptor.descriptor_id
        ))
    })?;

    Ok(())
}

fn next_arg<I>(name: &'static str, args: &mut I) -> Result<String, NodeIdentityGenerateError>
where
    I: Iterator<Item = String>,
{
    args.next()
        .ok_or_else(|| NodeIdentityGenerateError::InvalidArgs(format!("{name} requires a value")))
}

fn parse_i64_arg<I>(name: &'static str, args: &mut I) -> Result<i64, NodeIdentityGenerateError>
where
    I: Iterator<Item = String>,
{
    next_arg(name, args)?
        .trim()
        .parse::<i64>()
        .map_err(|_| NodeIdentityGenerateError::InvalidArgs(format!("{name} must be an integer")))
}

fn validate_positive_i64(name: &'static str, value: i64) -> Result<(), NodeIdentityGenerateError> {
    if value > 0 {
        Ok(())
    } else {
        Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "{name} must be greater than zero"
        )))
    }
}

fn validate_required_arg(name: &'static str, value: &str) -> Result<(), NodeIdentityGenerateError> {
    if value.trim().is_empty() {
        Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "{name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn validate_repeated_values(
    name: &'static str,
    values: &[String],
) -> Result<(), NodeIdentityGenerateError> {
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "{name} must not contain empty values"
        )));
    }

    Ok(())
}

fn parse_network_mode(value: &str) -> Result<NetworkMode, NodeIdentityGenerateError> {
    match value.trim() {
        "offline" => Ok(NetworkMode::Offline),
        "local_only" => Ok(NetworkMode::LocalOnly),
        "lan_only" => Ok(NetworkMode::LanOnly),
        "private_peers" => Ok(NetworkMode::PrivatePeers),
        "public_discovery" => Ok(NetworkMode::PublicDiscovery),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --network-mode '{}'",
            value
        ))),
    }
}

fn parse_discovery_policy(value: &str) -> Result<DiscoveryPolicy, NodeIdentityGenerateError> {
    match value.trim() {
        "none" => Ok(DiscoveryPolicy::None),
        "lan_announce" => Ok(DiscoveryPolicy::LanAnnounce),
        "private_allowlist" => Ok(DiscoveryPolicy::PrivateAllowlist),
        "member_visible" => Ok(DiscoveryPolicy::MemberVisible),
        "user_consented_introduction" => Ok(DiscoveryPolicy::UserConsentedIntroduction),
        "public_registry" => Ok(DiscoveryPolicy::PublicRegistry),
        "public_dht" => Ok(DiscoveryPolicy::PublicDht),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --discovery-policy '{}'",
            value
        ))),
    }
}

fn parse_peering_policy(value: &str) -> Result<PeeringPolicy, NodeIdentityGenerateError> {
    match value.trim() {
        "none" => Ok(PeeringPolicy::None),
        "static_allowlist" => Ok(PeeringPolicy::StaticAllowlist),
        "invite_token" => Ok(PeeringPolicy::InviteToken),
        "member_introduced" => Ok(PeeringPolicy::MemberIntroduced),
        "public_authenticated" => Ok(PeeringPolicy::PublicAuthenticated),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --peering-policy '{}'",
            value
        ))),
    }
}

fn parse_relay_policy(value: &str) -> Result<RelayPolicy, NodeIdentityGenerateError> {
    match value.trim() {
        "none" => Ok(RelayPolicy::None),
        "own_users_only" => Ok(RelayPolicy::OwnUsersOnly),
        "allowlisted_peers" => Ok(RelayPolicy::AllowlistedPeers),
        "open_limited" => Ok(RelayPolicy::OpenLimited),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --relay-policy '{}'",
            value
        ))),
    }
}

fn parse_dm_forwarding_policy(
    value: &str,
) -> Result<DmForwardingPolicy, NodeIdentityGenerateError> {
    match value.trim() {
        "disabled" => Ok(DmForwardingPolicy::Disabled),
        "local_recipients_only" => Ok(DmForwardingPolicy::LocalRecipientsOnly),
        "allowlisted_route" => Ok(DmForwardingPolicy::AllowlistedRoute),
        "relay_allowed" => Ok(DmForwardingPolicy::RelayAllowed),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --dm-forwarding-policy '{}'",
            value
        ))),
    }
}

fn parse_storage_policy(value: &str) -> Result<StoragePolicy, NodeIdentityGenerateError> {
    match value.trim() {
        "transient_only" => Ok(StoragePolicy::TransientOnly),
        "durable_encrypted_envelopes" => Ok(StoragePolicy::DurableEncryptedEnvelopes),
        _ => Err(NodeIdentityGenerateError::InvalidArgs(format!(
            "unsupported --storage-policy '{}'",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        generate_node_identity, NodeIdentityGenerateCliOptions, NodeIdentityGenerateOptions,
        DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS,
    };
    use communication_core::{
        ed25519_public_key_hex, verify_descriptor_ed25519, DiscoveryPolicy, DmForwardingPolicy,
        NetworkMode, PeeringPolicy, RelayPolicy, StoragePolicy,
    };

    fn default_options() -> NodeIdentityGenerateOptions {
        NodeIdentityGenerateOptions {
            node_id: "node-local".to_string(),
            descriptor_id: Some("descriptor-local".to_string()),
            ttl_seconds: 300,
            max_ttl_seconds: DEFAULT_NODE_DESCRIPTOR_MAX_TTL_SECONDS,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy: PeeringPolicy::InviteToken,
            relay_policy: RelayPolicy::None,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec!["https://node-local.example".to_string()],
            supported_protocols: vec!["hexrelay-node-http".to_string()],
            trust_labels: Vec::new(),
            revocation_pointer: None,
        }
    }

    #[test]
    fn generates_signed_private_mesh_node_identity() {
        let (output, identity) =
            generate_node_identity(&default_options(), 1_700_000_000).expect("generate identity");

        assert_eq!(identity.descriptor.node_id, "node-local");
        assert_eq!(identity.descriptor.descriptor_id, "descriptor-local");
        assert_eq!(
            identity.descriptor.peering_policy,
            PeeringPolicy::InviteToken
        );
        assert_eq!(identity.descriptor.relay_policy, RelayPolicy::None);
        assert_eq!(
            identity.descriptor.dm_forwarding_policy,
            DmForwardingPolicy::LocalRecipientsOnly
        );
        assert_eq!(
            output.api_local_node_descriptor_json,
            identity.descriptor.clone()
        );
        assert!(!output.api_local_node_private_key_pkcs8_base64.is_empty());
        assert_eq!(
            ed25519_public_key_hex(&identity.private_key_pkcs8).expect("derive public key"),
            identity.descriptor.node_public_key
        );
        verify_descriptor_ed25519(&identity.descriptor).expect("signature verifies");
    }

    #[test]
    fn cli_requires_node_id() {
        let err = NodeIdentityGenerateCliOptions::parse([
            "--address".to_string(),
            "https://node-local.example".to_string(),
        ])
        .expect_err("missing node id should fail");

        assert!(err.to_string().contains("--node-id is required"));
    }

    #[test]
    fn cli_rejects_ttl_above_configured_limit() {
        let err = NodeIdentityGenerateCliOptions::parse([
            "--node-id".to_string(),
            "node-local".to_string(),
            "--address".to_string(),
            "https://node-local.example".to_string(),
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

    #[test]
    fn validation_rejects_incoherent_policy_shape() {
        let mut options = default_options();
        options.network_mode = NetworkMode::PrivatePeers;
        options.discovery_policy = DiscoveryPolicy::LanAnnounce;

        let err = match generate_node_identity(&options, 1_700_000_000) {
            Ok(_) => panic!("invalid policy should fail"),
            Err(error) => error,
        };
        assert!(err.to_string().contains("DiscoveryPolicyConflict"));
    }
}
