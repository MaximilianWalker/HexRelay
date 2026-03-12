use crate::app::PolicyEngine;
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, PolicyContext,
    PolicyError, TransportProfile,
};

#[test]
fn routes_dm_direct_to_direct_peer_profile() {
    let policy = PolicyContext::default();

    let routed = PolicyEngine::route_mode(CommunicationMode::DmDirect, &policy)
        .expect("dm direct mode should route");

    assert_eq!(routed, TransportProfile::DirectPeer);
}

#[test]
fn routes_server_mode_to_node_client_profile() {
    let policy = PolicyContext::default();

    let routed = PolicyEngine::route_mode(CommunicationMode::ServerChannel, &policy)
        .expect("server channel mode should route");

    assert_eq!(routed, TransportProfile::NodeClient);
}

#[test]
fn rejects_disabled_server_mode() {
    let policy = PolicyContext {
        enable_server_channel: false,
        ..PolicyContext::default()
    };

    let routed = PolicyEngine::route_mode(CommunicationMode::ServerChannel, &policy);

    assert_eq!(
        routed,
        Err(PolicyError::ModeDisabled {
            mode: CommunicationMode::ServerChannel,
        })
    );
}

#[test]
fn validates_connect_target_for_routed_profile() {
    let intent = ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::PeerIdentity {
            identity_id: "user-123".to_string(),
        },
    };

    let result = PolicyEngine::validate_connect_intent(TransportProfile::DirectPeer, &intent);

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_connect_target_mismatch_for_profile() {
    let intent = ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.example".to_string(),
        },
    };

    let result = PolicyEngine::validate_connect_intent(TransportProfile::DirectPeer, &intent);

    assert_eq!(
        result,
        Err(PolicyError::TargetProfileMismatch {
            profile: TransportProfile::DirectPeer,
            target: ConnectTarget::NodeEndpoint {
                endpoint: "https://node.example".to_string(),
            },
        })
    );
}

#[test]
fn builds_policy_compliant_provenance_for_dm() {
    let provenance =
        PolicyEngine::build_provenance(CommunicationMode::DmDirect, TransportProfile::DirectPeer);

    assert_eq!(provenance.mode, CommunicationMode::DmDirect);
    assert_eq!(provenance.profile, TransportProfile::DirectPeer);
    assert_eq!(
        provenance.reason_code,
        CommunicationReasonCode::DmDirectRouteSelected
    );
    assert_eq!(
        provenance.policy_assertions,
        vec!["dm_direct_policy_compliant".to_string()]
    );
}
