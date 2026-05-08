use crate::{
    is_valid_lan_discovery_signature_hex, lan_discovery_signing_payload, parse_lan_endpoint_hint,
    validate_lan_endpoint_hint, LanEndpointHintError, LAN_DISCOVERY_MULTICAST_ADDR,
    LAN_DISCOVERY_MULTICAST_HOP_LIMIT, LAN_DISCOVERY_SCOPE, LAN_DISCOVERY_TTL_SECONDS,
};

#[test]
fn accepts_private_and_link_local_lan_endpoint_hints() {
    for hint in [
        "udp://192.168.1.12:4040",
        "tcp://10.0.0.8:4040",
        "quic://172.16.5.9:4040",
        "udp://169.254.1.9:4040",
    ] {
        assert!(validate_lan_endpoint_hint(hint).is_ok(), "{hint}");
    }
}

#[test]
fn parses_lan_endpoint_hint_scheme_and_address() {
    let hint = parse_lan_endpoint_hint("udp://192.168.1.12:4040").expect("valid LAN hint");

    assert_eq!(hint.scheme, "udp");
    assert_eq!(hint.address.port(), 4040);
}

#[test]
fn rejects_non_local_or_non_ip_lan_endpoint_hints() {
    for (hint, expected) in [
        ("udp://8.8.8.8:4040", LanEndpointHintError::NonLocalAddress),
        (
            "udp://127.0.0.1:4040",
            LanEndpointHintError::NonLocalAddress,
        ),
        ("udp://[::1]:4040", LanEndpointHintError::NonIpv4Address),
        (
            "udp://lan-peer.local:4040",
            LanEndpointHintError::InvalidSocketAddress,
        ),
        ("udp://[fd00::1]:4040", LanEndpointHintError::NonIpv4Address),
        ("udp://[fe80::1]:4040", LanEndpointHintError::NonIpv4Address),
        ("udp://192.168.1.12:0", LanEndpointHintError::ZeroPort),
        (
            "http://192.168.1.12:4040",
            LanEndpointHintError::UnsupportedScheme,
        ),
        (
            "UDP://192.168.1.12:4040",
            LanEndpointHintError::UppercaseScheme,
        ),
    ] {
        assert_eq!(parse_lan_endpoint_hint(hint), Err(expected), "{hint}");
    }
}

#[test]
fn validates_lan_discovery_signature_hex_shape() {
    assert!(is_valid_lan_discovery_signature_hex(&"aa".repeat(64)));
    assert!(is_valid_lan_discovery_signature_hex(&"AA".repeat(64)));
    assert!(!is_valid_lan_discovery_signature_hex("aa"));
    assert!(!is_valid_lan_discovery_signature_hex(&"aa".repeat(65)));
    assert!(!is_valid_lan_discovery_signature_hex(&format!(
        "{}zz",
        "aa".repeat(63)
    )));
}

#[test]
fn exposes_local_only_multicast_constants() {
    assert_eq!(LAN_DISCOVERY_SCOPE, "lan_subnet");
    assert_eq!(LAN_DISCOVERY_TTL_SECONDS, 120);
    assert_eq!(LAN_DISCOVERY_MULTICAST_HOP_LIMIT, 1);
    assert_eq!(LAN_DISCOVERY_MULTICAST_ADDR, "239.255.48.31:48999");
}

#[test]
fn builds_deterministic_lan_discovery_signing_payload() {
    let endpoint_hints = vec!["udp://192.168.1.12:4040".to_string()];

    let payload = lan_discovery_signing_payload(
        "usr-nora-k",
        &endpoint_hints,
        1_775_000_000,
        1_775_000_120,
        "nonce-1",
    );

    assert_eq!(
        String::from_utf8(payload).expect("payload is utf8"),
        "hexrelay-lan-discovery-v1\nusr-nora-k\nlan_subnet\n1775000000\n1775000120\nnonce-1\n23:udp://192.168.1.12:4040\n"
    );
}
