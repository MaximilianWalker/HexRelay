use crate::app::PolicyEngine;
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, PolicyContext,
    PolicyError, TransportProfile,
};

#[test]
fn exposes_stable_mode_profile_and_reason_code_strings() {
    assert_eq!(CommunicationMode::DmEnvelope.as_str(), "dm_envelope");
    assert_eq!(CommunicationMode::ServerChannel.as_str(), "server_channel");
    assert_eq!(CommunicationMode::Presence.as_str(), "presence");
    assert_eq!(TransportProfile::NodeClient.as_str(), "node_client");
    assert_eq!(
        CommunicationReasonCode::DmEnvelopeNodeRouteSelected.as_str(),
        "dm_envelope_node_route_selected"
    );
    assert_eq!(
        CommunicationReasonCode::ServerChannelRouteSelected.as_str(),
        "server_channel_route_selected"
    );
    assert_eq!(
        CommunicationReasonCode::PresenceRouteSelected.as_str(),
        "presence_route_selected"
    );
    assert_eq!(
        CommunicationReasonCode::TransportSendFailed.as_str(),
        "transport_send_failed"
    );
}

#[test]
fn routes_dm_envelope_to_node_client_profile() {
    let policy = PolicyContext::default();

    let routed = PolicyEngine::route_mode(CommunicationMode::DmEnvelope, &policy)
        .expect("dm envelope mode should route");

    assert_eq!(routed, TransportProfile::NodeClient);
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
fn validates_node_connect_target_for_routed_profile() {
    let intent = ConnectIntent {
        mode: CommunicationMode::DmEnvelope,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.example".to_string(),
        },
    };

    let result = PolicyEngine::validate_connect_intent(TransportProfile::NodeClient, &intent);

    assert_eq!(result, Ok(()));
}

#[test]
fn builds_policy_compliant_provenance_for_dm_envelope() {
    let provenance =
        PolicyEngine::build_provenance(CommunicationMode::DmEnvelope, TransportProfile::NodeClient);

    assert_eq!(provenance.mode, CommunicationMode::DmEnvelope);
    assert_eq!(provenance.profile, TransportProfile::NodeClient);
    assert_eq!(
        provenance.reason_code,
        CommunicationReasonCode::DmEnvelopeNodeRouteSelected
    );
    assert_eq!(
        provenance.policy_assertions,
        vec!["dm_envelope_node_policy_compliant".to_string()]
    );
}

#[test]
fn builds_policy_compliant_provenance_for_server_channel() {
    let provenance = PolicyEngine::build_provenance(
        CommunicationMode::ServerChannel,
        TransportProfile::NodeClient,
    );

    assert_eq!(provenance.mode, CommunicationMode::ServerChannel);
    assert_eq!(provenance.profile, TransportProfile::NodeClient);
    assert_eq!(
        provenance.reason_code,
        CommunicationReasonCode::ServerChannelRouteSelected
    );
    assert_eq!(
        provenance.policy_assertions,
        vec!["server_channel_policy_compliant".to_string()]
    );
}

#[test]
fn builds_policy_compliant_provenance_for_presence() {
    let provenance =
        PolicyEngine::build_provenance(CommunicationMode::Presence, TransportProfile::NodeClient);

    assert_eq!(provenance.mode, CommunicationMode::Presence);
    assert_eq!(provenance.profile, TransportProfile::NodeClient);
    assert_eq!(
        provenance.reason_code,
        CommunicationReasonCode::PresenceRouteSelected
    );
    assert_eq!(
        provenance.policy_assertions,
        vec!["presence_policy_compliant".to_string()]
    );
}
