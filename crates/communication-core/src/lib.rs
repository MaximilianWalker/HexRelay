pub mod app;
pub mod config;
pub mod domain;
pub mod transport;

pub use app::PolicyEngine;
pub use config::CommunicationConfig;
pub use domain::{
    is_lan_only_ip, lan_discovery_signing_payload, parse_lan_endpoint_hint,
    validate_lan_endpoint_hint,
};
pub use domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, DmTransportPolicy,
    LanDiscoveryAdvertisement, LanEndpointHint, LanEndpointHintError, PolicyContext, PolicyError,
    SendEnvelope, SessionProvenance, TransportProfile, LAN_DISCOVERY_MULTICAST_ADDR,
    LAN_DISCOVERY_MULTICAST_HOP_LIMIT, LAN_DISCOVERY_SCOPE, LAN_DISCOVERY_TTL_SECONDS,
};
pub use transport::{
    connect_via_direct_peer, send_via_direct_peer_dispatch, send_via_node_dispatch,
    DirectPeerDispatch, DispatchingDirectPeerTransport, DispatchingNodeClientTransport,
    NodeClientTransport, NodeDispatch, TransportError, UnsupportedDirectPeerTransport,
    UnsupportedNodeClientTransport,
};

#[cfg(test)]
mod tests;
