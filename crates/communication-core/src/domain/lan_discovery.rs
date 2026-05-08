use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use serde::{Deserialize, Serialize};

pub const LAN_DISCOVERY_SCOPE: &str = "lan_subnet";
pub const LAN_DISCOVERY_TTL_SECONDS: i64 = 120;
pub const LAN_DISCOVERY_MULTICAST_HOP_LIMIT: u32 = 1;
pub const LAN_DISCOVERY_MULTICAST_ADDR: &str = "239.255.48.31:48999";

const LAN_ENDPOINT_HINT_ALLOWED_SCHEMES: [&str; 3] = ["tcp", "udp", "quic"];
const LAN_ENDPOINT_HINT_MAX_LENGTH: usize = 200;
const LAN_DISCOVERY_ED25519_SIGNATURE_HEX_LENGTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanEndpointHint {
    pub scheme: String,
    pub address: SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanDiscoveryAdvertisement {
    pub version: u8,
    pub identity_id: String,
    pub endpoint_hints: Vec<String>,
    pub scope: String,
    pub issued_at_epoch: i64,
    pub expires_at_epoch: i64,
    pub nonce: String,
    pub signature: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanEndpointHintError {
    EmptyOrTooLong,
    Whitespace,
    MissingScheme,
    EmptyAddress,
    UppercaseScheme,
    UnsupportedScheme,
    InvalidSocketAddress,
    ZeroPort,
    NonIpv4Address,
    NonLocalAddress,
}

pub fn validate_lan_endpoint_hint(hint: &str) -> Result<(), LanEndpointHintError> {
    parse_lan_endpoint_hint(hint).map(|_| ())
}

pub fn lan_discovery_signing_payload(
    identity_id: &str,
    endpoint_hints: &[String],
    issued_at_epoch: i64,
    expires_at_epoch: i64,
    nonce: &str,
) -> Vec<u8> {
    let mut payload = format!(
        "hexrelay-lan-discovery-v1\n{}\n{}\n{}\n{}\n{}\n",
        identity_id, LAN_DISCOVERY_SCOPE, issued_at_epoch, expires_at_epoch, nonce
    );

    for hint in endpoint_hints {
        payload.push_str(&hint.len().to_string());
        payload.push(':');
        payload.push_str(hint);
        payload.push('\n');
    }

    payload.into_bytes()
}

pub fn is_valid_lan_discovery_signature_hex(signature: &str) -> bool {
    signature.len() == LAN_DISCOVERY_ED25519_SIGNATURE_HEX_LENGTH
        && signature.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn parse_lan_endpoint_hint(hint: &str) -> Result<LanEndpointHint, LanEndpointHintError> {
    let value = hint.trim();
    if value.is_empty() || value.len() > LAN_ENDPOINT_HINT_MAX_LENGTH {
        return Err(LanEndpointHintError::EmptyOrTooLong);
    }
    if value != hint {
        return Err(LanEndpointHintError::Whitespace);
    }

    let (scheme, address) = value
        .split_once("://")
        .ok_or(LanEndpointHintError::MissingScheme)?;
    if address.trim().is_empty() {
        return Err(LanEndpointHintError::EmptyAddress);
    }

    let normalized_scheme = scheme.to_ascii_lowercase();
    if scheme != normalized_scheme {
        return Err(LanEndpointHintError::UppercaseScheme);
    }
    if !LAN_ENDPOINT_HINT_ALLOWED_SCHEMES
        .iter()
        .any(|allowed| &normalized_scheme == allowed)
    {
        return Err(LanEndpointHintError::UnsupportedScheme);
    }

    let address = address
        .parse::<SocketAddr>()
        .map_err(|_| LanEndpointHintError::InvalidSocketAddress)?;
    if address.port() == 0 {
        return Err(LanEndpointHintError::ZeroPort);
    }
    let ip = address.ip();
    if !matches!(ip, IpAddr::V4(_)) {
        return Err(LanEndpointHintError::NonIpv4Address);
    }
    if !is_lan_only_ip(ip) {
        return Err(LanEndpointHintError::NonLocalAddress);
    }

    Ok(LanEndpointHint {
        scheme: normalized_scheme,
        address,
    })
}

pub fn is_lan_only_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_private() || ip.is_link_local(),
        IpAddr::V6(ip) => is_ipv6_unique_local(ip) || is_ipv6_unicast_link_local(ip),
    }
}

fn is_ipv6_unique_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_unicast_link_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}
