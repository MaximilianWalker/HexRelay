mod communication;
mod lan_discovery;

pub use communication::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    PolicyContext, PolicyError, SendEnvelope, SessionProvenance, TransportProfile,
};
pub use lan_discovery::{
    is_lan_only_ip, lan_discovery_signing_payload, parse_lan_endpoint_hint,
    validate_lan_endpoint_hint, LanDiscoveryAdvertisement, LanEndpointHint, LanEndpointHintError,
    LAN_DISCOVERY_MULTICAST_ADDR, LAN_DISCOVERY_MULTICAST_HOP_LIMIT, LAN_DISCOVERY_SCOPE,
    LAN_DISCOVERY_TTL_SECONDS,
};
