use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::app::{router::assert_dm_direct_profile, CommunicationError, CommunicationRouter};
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, PolicyContext,
    SendEnvelope, SessionProvenance, TransportProfile,
};
use crate::transport::{
    send_via_node_dispatch, DirectPeerTransport, DispatchingNodeClientTransport,
    NodeClientTransport, NodeDispatch, TransportError,
};

#[derive(Clone)]
struct RecordingDirectPeer {
    connect_calls: Arc<AtomicUsize>,
    send_calls: Arc<AtomicUsize>,
}

impl DirectPeerTransport for RecordingDirectPeer {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        self.connect_calls.fetch_add(1, Ordering::SeqCst);
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::DirectPeer,
            reason_code: CommunicationReasonCode::DmDirectRouteSelected,
            policy_assertions: vec!["dm_direct_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        self.send_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct RecordingNodeClient {
    connect_calls: Arc<AtomicUsize>,
    send_calls: Arc<AtomicUsize>,
}

impl NodeClientTransport for RecordingNodeClient {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        self.connect_calls.fetch_add(1, Ordering::SeqCst);
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::NodeClient,
            reason_code: CommunicationReasonCode::ServerChannelRouteSelected,
            policy_assertions: vec!["server_channel_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        self.send_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct TestCounters {
    direct_connect_calls: Arc<AtomicUsize>,
    direct_send_calls: Arc<AtomicUsize>,
    node_connect_calls: Arc<AtomicUsize>,
    node_send_calls: Arc<AtomicUsize>,
}

impl TestCounters {
    fn new() -> Self {
        Self {
            direct_connect_calls: Arc::new(AtomicUsize::new(0)),
            direct_send_calls: Arc::new(AtomicUsize::new(0)),
            node_connect_calls: Arc::new(AtomicUsize::new(0)),
            node_send_calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn direct_peer(&self) -> RecordingDirectPeer {
        RecordingDirectPeer {
            connect_calls: Arc::clone(&self.direct_connect_calls),
            send_calls: Arc::clone(&self.direct_send_calls),
        }
    }

    fn node_client(&self) -> RecordingNodeClient {
        RecordingNodeClient {
            connect_calls: Arc::clone(&self.node_connect_calls),
            send_calls: Arc::clone(&self.node_send_calls),
        }
    }
}

struct FailingDirectPeer;

impl DirectPeerTransport for FailingDirectPeer {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

struct RecordingDispatch {
    payloads: Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
}

impl NodeDispatch for RecordingDispatch {
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError> {
        self.payloads
            .lock()
            .expect("acquire payload lock")
            .push(payload.to_vec());
        Ok(())
    }
}

#[test]
fn routes_dm_connect_through_direct_peer_adapter() {
    let counters = TestCounters::new();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        counters.direct_peer(),
        counters.node_client(),
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmDirect,
        target: ConnectTarget::PeerIdentity {
            identity_id: "peer-a".to_string(),
        },
    });

    assert!(result.is_ok());
    assert_eq!(counters.direct_connect_calls.load(Ordering::SeqCst), 1);
    assert_eq!(counters.node_connect_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn routes_server_send_through_node_adapter() {
    let counters = TestCounters::new();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        counters.direct_peer(),
        counters.node_client(),
    );

    let result = router.send(&SendEnvelope {
        mode: CommunicationMode::ServerChannel,
        payload: b"hello".to_vec(),
    });

    assert_eq!(result, Ok(()));
    assert_eq!(counters.node_send_calls.load(Ordering::SeqCst), 1);
    assert_eq!(counters.direct_send_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn rejects_target_profile_mismatch_before_adapter_call() {
    let counters = TestCounters::new();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        counters.direct_peer(),
        counters.node_client(),
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
    assert_eq!(counters.direct_connect_calls.load(Ordering::SeqCst), 0);
    assert_eq!(counters.node_connect_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn maps_transport_connect_failure_to_reason_code() {
    let counters = TestCounters::new();
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        FailingDirectPeer,
        counters.node_client(),
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
    let counters = TestCounters::new();
    let router = CommunicationRouter::new(policy, counters.direct_peer(), counters.node_client());

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

#[test]
fn rejects_non_direct_profile_for_dm_mode() {
    let result =
        assert_dm_direct_profile(CommunicationMode::DmDirect, TransportProfile::NodeClient);

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::DmDirectPolicyViolation,
            mode: CommunicationMode::DmDirect,
            profile: Some(TransportProfile::NodeClient),
        })
    );
}

#[test]
fn dispatching_node_client_transport_rejects_wrong_mode_payload() {
    let payloads = Arc::new(std::sync::Mutex::new(Vec::new()));
    let transport = DispatchingNodeClientTransport::new(
        CommunicationMode::Presence,
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
    );

    let result = transport.send(&SendEnvelope {
        mode: CommunicationMode::ServerChannel,
        payload: b"hello".to_vec(),
    });

    assert_eq!(result, Err(TransportError::SendFailed));
    assert!(payloads.lock().expect("acquire payload lock").is_empty());
}

#[test]
fn dispatching_node_client_transport_rejects_wrong_mode_connect() {
    let payloads = Arc::new(std::sync::Mutex::new(Vec::new()));
    let transport = DispatchingNodeClientTransport::new(
        CommunicationMode::Presence,
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
    );

    let result = transport.connect(&ConnectIntent {
        mode: CommunicationMode::ServerChannel,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.invalid".to_string(),
        },
    });

    assert_eq!(result, Err(TransportError::ConnectFailed));
}

#[test]
fn dispatching_node_client_transport_forwards_payload_for_matching_mode() {
    let payloads = Arc::new(std::sync::Mutex::new(Vec::new()));
    let transport = DispatchingNodeClientTransport::new(
        CommunicationMode::Presence,
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
    );

    let result = transport.send(&SendEnvelope {
        mode: CommunicationMode::Presence,
        payload: b"presence".to_vec(),
    });

    assert_eq!(result, Ok(()));
    assert_eq!(
        payloads.lock().expect("acquire payload lock").as_slice(),
        &[b"presence".to_vec()]
    );
}

#[test]
fn send_via_node_dispatch_routes_through_shared_node_client_bootstrap() {
    let payloads = Arc::new(std::sync::Mutex::new(Vec::new()));

    let result = send_via_node_dispatch(
        CommunicationMode::ServerChannel,
        PolicyContext::default(),
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
        b"server".to_vec(),
    );

    assert_eq!(result, Ok(()));
    assert_eq!(
        payloads.lock().expect("acquire payload lock").as_slice(),
        &[b"server".to_vec()]
    );
}
