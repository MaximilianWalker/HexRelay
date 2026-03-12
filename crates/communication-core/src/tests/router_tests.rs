use std::sync::atomic::{AtomicUsize, Ordering};

use crate::app::{CommunicationError, CommunicationRouter};
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, PolicyContext,
    SendEnvelope, SessionProvenance, TransportProfile,
};
use crate::transport::{DirectPeerTransport, NodeClientTransport, TransportError};

#[derive(Default)]
struct RecordingDirectPeer;

impl DirectPeerTransport for RecordingDirectPeer {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        DIRECT_CONNECT_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::DirectPeer,
            reason_code: CommunicationReasonCode::DmDirectRouteSelected,
            policy_assertions: vec!["dm_direct_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        DIRECT_SEND_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Default)]
struct RecordingNodeClient;

impl NodeClientTransport for RecordingNodeClient {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        NODE_CONNECT_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::NodeClient,
            reason_code: CommunicationReasonCode::ServerChannelRouteSelected,
            policy_assertions: vec!["server_channel_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        NODE_SEND_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

static DIRECT_CONNECT_CALLS: AtomicUsize = AtomicUsize::new(0);
static DIRECT_SEND_CALLS: AtomicUsize = AtomicUsize::new(0);
static NODE_CONNECT_CALLS: AtomicUsize = AtomicUsize::new(0);
static NODE_SEND_CALLS: AtomicUsize = AtomicUsize::new(0);

struct FailingDirectPeer;

impl DirectPeerTransport for FailingDirectPeer {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

fn reset_counters() {
    DIRECT_CONNECT_CALLS.store(0, Ordering::SeqCst);
    DIRECT_SEND_CALLS.store(0, Ordering::SeqCst);
    NODE_CONNECT_CALLS.store(0, Ordering::SeqCst);
    NODE_SEND_CALLS.store(0, Ordering::SeqCst);
}

#[test]
fn routes_dm_connect_through_direct_peer_adapter() {
    reset_counters();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingDirectPeer,
        RecordingNodeClient,
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::PeerIdentity {
            identity_id: "peer-a".to_string(),
        },
    });

    assert!(result.is_ok());
    assert_eq!(DIRECT_CONNECT_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(NODE_CONNECT_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn routes_server_send_through_node_adapter() {
    reset_counters();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingDirectPeer,
        RecordingNodeClient,
    );

    let result = router.send(&SendEnvelope {
        mode: CommunicationMode::ServerChannel,
        payload: b"hello".to_vec(),
    });

    assert_eq!(result, Ok(()));
    assert_eq!(NODE_SEND_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(DIRECT_SEND_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn rejects_target_profile_mismatch_before_adapter_call() {
    reset_counters();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingDirectPeer,
        RecordingNodeClient,
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.invalid".to_string(),
        },
    });

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::TargetProfileMismatch,
            mode: CommunicationMode::DmDirect,
            profile: Some(TransportProfile::DirectPeer),
        })
    );
    assert_eq!(DIRECT_CONNECT_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(NODE_CONNECT_CALLS.load(Ordering::SeqCst), 0);
}

#[test]
fn maps_transport_connect_failure_to_reason_code() {
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        FailingDirectPeer,
        RecordingNodeClient,
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::PeerIdentity {
            identity_id: "peer-a".to_string(),
        },
    });

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::TransportConnectFailed,
            mode: CommunicationMode::DmDirect,
            profile: Some(TransportProfile::DirectPeer),
        })
    );
}

#[test]
fn maps_mode_disabled_to_reason_code() {
    let policy = PolicyContext {
        enable_server_channel: false,
        ..PolicyContext::default()
    };
    let router = CommunicationRouter::new(policy, RecordingDirectPeer, RecordingNodeClient);

    let result = router.send(&SendEnvelope {
        mode: CommunicationMode::ServerChannel,
        payload: b"hello".to_vec(),
    });

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::ModeDisabled,
            mode: CommunicationMode::ServerChannel,
            profile: None,
        })
    );
}
