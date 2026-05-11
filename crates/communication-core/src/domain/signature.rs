use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ring::signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};

use super::{
    DescriptorSignatureVerifier, DiscoveryPolicy, DmForwardingPolicy, NetworkMode, NodeDescriptor,
    NodeRateLimit, NodeSignatureAlgorithm, PeeringPolicy, RateLimitScope, RelayPolicy,
    StoragePolicy,
};

const DESCRIPTOR_SIGNING_DOMAIN: &str = "hexrelay.node_descriptor.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeDescriptorSignatureError {
    UnsupportedAlgorithm,
    InvalidPublicKeyEncoding,
    InvalidSignatureEncoding,
    InvalidPrivateKey,
    SignatureVerificationFailed,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Ed25519DescriptorVerifier;

impl DescriptorSignatureVerifier for Ed25519DescriptorVerifier {
    fn verify(&self, descriptor: &NodeDescriptor) -> bool {
        verify_descriptor_ed25519(descriptor).is_ok()
    }
}

pub fn sign_descriptor_ed25519_pkcs8(
    descriptor: &NodeDescriptor,
    private_key_pkcs8: &[u8],
) -> Result<String, NodeDescriptorSignatureError> {
    if descriptor.signature.algorithm != NodeSignatureAlgorithm::Ed25519 {
        return Err(NodeDescriptorSignatureError::UnsupportedAlgorithm);
    }

    let key_pair = Ed25519KeyPair::from_pkcs8(private_key_pkcs8)
        .map_err(|_| NodeDescriptorSignatureError::InvalidPrivateKey)?;

    let payload = canonical_descriptor_signing_payload(descriptor);
    let signature = key_pair.sign(&payload);

    Ok(hex::encode(signature.as_ref()))
}

pub fn ed25519_public_key_hex(
    private_key_pkcs8: &[u8],
) -> Result<String, NodeDescriptorSignatureError> {
    let key_pair = Ed25519KeyPair::from_pkcs8(private_key_pkcs8)
        .map_err(|_| NodeDescriptorSignatureError::InvalidPrivateKey)?;

    Ok(hex::encode(key_pair.public_key().as_ref()))
}

pub fn verify_descriptor_ed25519(
    descriptor: &NodeDescriptor,
) -> Result<(), NodeDescriptorSignatureError> {
    if descriptor.signature.algorithm != NodeSignatureAlgorithm::Ed25519 {
        return Err(NodeDescriptorSignatureError::UnsupportedAlgorithm);
    }

    let public_key = decode_fixed_len(&descriptor.node_public_key, 32)
        .ok_or(NodeDescriptorSignatureError::InvalidPublicKeyEncoding)?;
    let signature = decode_fixed_len(&descriptor.signature.value, 64)
        .ok_or(NodeDescriptorSignatureError::InvalidSignatureEncoding)?;
    let payload = canonical_descriptor_signing_payload(descriptor);

    let key = UnparsedPublicKey::new(&ED25519, public_key);
    key.verify(&payload, &signature)
        .map_err(|_| NodeDescriptorSignatureError::SignatureVerificationFailed)
}

pub fn canonical_descriptor_signing_payload(descriptor: &NodeDescriptor) -> Vec<u8> {
    let mut payload = Vec::new();

    push_str(&mut payload, "domain", DESCRIPTOR_SIGNING_DOMAIN);
    push_str(&mut payload, "node_id", &descriptor.node_id);
    push_str(&mut payload, "node_public_key", &descriptor.node_public_key);
    push_str(&mut payload, "descriptor_id", &descriptor.descriptor_id);
    push_i64(
        &mut payload,
        "issued_at_epoch_seconds",
        descriptor.issued_at_epoch_seconds,
    );
    push_i64(
        &mut payload,
        "expires_at_epoch_seconds",
        descriptor.expires_at_epoch_seconds,
    );
    push_str(
        &mut payload,
        "network_mode",
        network_mode_name(descriptor.network_mode),
    );
    push_str(
        &mut payload,
        "discovery_policy",
        discovery_policy_name(descriptor.discovery_policy),
    );
    push_str(
        &mut payload,
        "peering_policy",
        peering_policy_name(descriptor.peering_policy),
    );
    push_str(
        &mut payload,
        "relay_policy",
        relay_policy_name(descriptor.relay_policy),
    );
    push_str(
        &mut payload,
        "dm_forwarding_policy",
        dm_forwarding_policy_name(descriptor.dm_forwarding_policy),
    );
    push_str(
        &mut payload,
        "storage_policy",
        storage_policy_name(descriptor.storage_policy),
    );
    push_string_list(&mut payload, "addresses", &descriptor.addresses);
    push_string_list(
        &mut payload,
        "supported_protocols",
        &descriptor.supported_protocols,
    );
    push_rate_limits(&mut payload, &descriptor.rate_limits);
    push_string_list(&mut payload, "trust_labels", &descriptor.trust_labels);
    push_optional_str(
        &mut payload,
        "revocation_pointer",
        descriptor.revocation_pointer.as_deref(),
    );
    push_str(
        &mut payload,
        "signature_algorithm",
        signature_algorithm_name(descriptor.signature.algorithm),
    );

    payload
}

fn push_str(payload: &mut Vec<u8>, name: &str, value: &str) {
    push_field(payload, name, value.as_bytes());
}

fn push_optional_str(payload: &mut Vec<u8>, name: &str, value: Option<&str>) {
    match value {
        Some(value) => {
            push_u32(payload, 1);
            push_str(payload, name, value);
        }
        None => {
            push_u32(payload, 0);
            push_str(payload, name, "");
        }
    }
}

fn push_i64(payload: &mut Vec<u8>, name: &str, value: i64) {
    push_field(payload, name, &value.to_be_bytes());
}

fn push_u32(payload: &mut Vec<u8>, value: u32) {
    payload.extend_from_slice(&value.to_be_bytes());
}

fn push_string_list(payload: &mut Vec<u8>, name: &str, values: &[String]) {
    push_str(payload, name, "list");
    push_u32(payload, values.len() as u32);

    for value in values {
        push_str(payload, name, value);
    }
}

fn push_rate_limits(payload: &mut Vec<u8>, rate_limits: &[NodeRateLimit]) {
    push_str(payload, "rate_limits", "list");
    push_u32(payload, rate_limits.len() as u32);

    for rate_limit in rate_limits {
        push_str(
            payload,
            "rate_limit.scope",
            rate_limit_scope_name(rate_limit.scope),
        );
        push_field(
            payload,
            "rate_limit.max_per_minute",
            &rate_limit.max_per_minute.to_be_bytes(),
        );
    }
}

fn push_field(payload: &mut Vec<u8>, name: &str, value: &[u8]) {
    push_u32(payload, name.len() as u32);
    payload.extend_from_slice(name.as_bytes());
    push_u32(payload, value.len() as u32);
    payload.extend_from_slice(value);
}

fn decode_fixed_len(value: &str, len: usize) -> Option<Vec<u8>> {
    let trimmed = value.trim();

    if trimmed.len() == len * 2 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex::decode(trimmed).ok();
    }

    BASE64
        .decode(trimmed)
        .ok()
        .filter(|decoded| decoded.len() == len)
}

fn network_mode_name(value: NetworkMode) -> &'static str {
    match value {
        NetworkMode::Offline => "offline",
        NetworkMode::LocalOnly => "local_only",
        NetworkMode::LanOnly => "lan_only",
        NetworkMode::PrivatePeers => "private_peers",
        NetworkMode::PublicDiscovery => "public_discovery",
    }
}

fn discovery_policy_name(value: DiscoveryPolicy) -> &'static str {
    match value {
        DiscoveryPolicy::None => "none",
        DiscoveryPolicy::LanAnnounce => "lan_announce",
        DiscoveryPolicy::PrivateAllowlist => "private_allowlist",
        DiscoveryPolicy::MemberVisible => "member_visible",
        DiscoveryPolicy::UserConsentedIntroduction => "user_consented_introduction",
        DiscoveryPolicy::PublicRegistry => "public_registry",
        DiscoveryPolicy::PublicDht => "public_dht",
    }
}

fn peering_policy_name(value: PeeringPolicy) -> &'static str {
    match value {
        PeeringPolicy::None => "none",
        PeeringPolicy::StaticAllowlist => "static_allowlist",
        PeeringPolicy::InviteToken => "invite_token",
        PeeringPolicy::MemberIntroduced => "member_introduced",
        PeeringPolicy::PublicAuthenticated => "public_authenticated",
    }
}

fn relay_policy_name(value: RelayPolicy) -> &'static str {
    match value {
        RelayPolicy::None => "none",
        RelayPolicy::OwnUsersOnly => "own_users_only",
        RelayPolicy::AllowlistedPeers => "allowlisted_peers",
        RelayPolicy::OpenLimited => "open_limited",
    }
}

fn dm_forwarding_policy_name(value: DmForwardingPolicy) -> &'static str {
    match value {
        DmForwardingPolicy::Disabled => "disabled",
        DmForwardingPolicy::LocalRecipientsOnly => "local_recipients_only",
        DmForwardingPolicy::AllowlistedRoute => "allowlisted_route",
        DmForwardingPolicy::RelayAllowed => "relay_allowed",
    }
}

fn storage_policy_name(value: StoragePolicy) -> &'static str {
    match value {
        StoragePolicy::TransientOnly => "transient_only",
        StoragePolicy::DurableEncryptedEnvelopes => "durable_encrypted_envelopes",
    }
}

fn signature_algorithm_name(value: NodeSignatureAlgorithm) -> &'static str {
    match value {
        NodeSignatureAlgorithm::Ed25519 => "ed25519",
    }
}

fn rate_limit_scope_name(value: RateLimitScope) -> &'static str {
    match value {
        RateLimitScope::Node => "node",
        RateLimitScope::Peer => "peer",
        RateLimitScope::User => "user",
        RateLimitScope::Route => "route",
        RateLimitScope::DescriptorSource => "descriptor_source",
    }
}
