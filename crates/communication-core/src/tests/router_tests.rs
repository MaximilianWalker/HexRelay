use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use crate::app::{CommunicationError, CommunicationRouter};
use crate::domain::{
    CommunicationMode, CommunicationReasonCode, ConnectIntent, ConnectTarget, PolicyContext,
    SendEnvelope, SessionProvenance, TransportProfile,
};
use crate::transport::{
    send_via_node_dispatch, DispatchingNodeClientTransport, NodeClientTransport, NodeDispatch,
    TransportError,
};

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
            reason_code: CommunicationReasonCode::DmEnvelopeNodeRouteSelected,
            policy_assertions: vec!["dm_envelope_node_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        self.send_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct FailingNodeClient;

impl NodeClientTransport for FailingNodeClient {
    fn connect(&self, _intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        Err(TransportError::ConnectFailed)
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        Err(TransportError::SendFailed)
    }
}

struct RecordingDispatch {
    payloads: Arc<Mutex<Vec<Vec<u8>>>>,
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
fn routes_dm_envelope_connect_through_node_adapter() {
    let node_connect_calls = Arc::new(AtomicUsize::new(0));
    let node_send_calls = Arc::new(AtomicUsize::new(0));
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingNodeClient {
            connect_calls: Arc::clone(&node_connect_calls),
            send_calls: node_send_calls,
        },
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmEnvelope,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.example".to_string(),
        },
    });

    assert!(result.is_ok());
    assert_eq!(node_connect_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn routes_dm_envelope_send_through_node_adapter() {
    let node_connect_calls = Arc::new(AtomicUsize::new(0));
    let node_send_calls = Arc::new(AtomicUsize::new(0));
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingNodeClient {
            connect_calls: node_connect_calls,
            send_calls: Arc::clone(&node_send_calls),
        },
    );

    let result = router.send(&SendEnvelope {
        mode: CommunicationMode::DmEnvelope,
        payload: b"dm-envelope".to_vec(),
    });

    assert_eq!(result, Ok(()));
    assert_eq!(node_send_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn maps_transport_connect_failure_to_reason_code() {
    let router = CommunicationRouter::new(PolicyContext::default(), FailingNodeClient);

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmEnvelope,
        target: ConnectTarget::NodeEndpoint {
            endpoint: "https://node.invalid".to_string(),
        },
    });

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::TransportConnectFailed,
            mode: CommunicationMode::DmEnvelope,
            profile: Some(TransportProfile::NodeClient),
        })
    );
}

#[test]
fn maps_mode_disabled_to_reason_code() {
    let policy = PolicyContext {
        enable_server_channel: false,
        ..PolicyContext::default()
    };
    let router = CommunicationRouter::new(policy, FailingNodeClient);

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
fn dispatching_node_client_transport_rejects_wrong_mode_payload() {
    let payloads = Arc::new(Mutex::new(Vec::new()));
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
    let payloads = Arc::new(Mutex::new(Vec::new()));
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
    let payloads = Arc::new(Mutex::new(Vec::new()));
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
fn send_via_node_dispatch_routes_dm_envelope_through_node_client_bootstrap() {
    let payloads = Arc::new(Mutex::new(Vec::new()));

    let result = send_via_node_dispatch(
        CommunicationMode::DmEnvelope,
        PolicyContext::default(),
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
        b"dm-envelope".to_vec(),
    );

    assert_eq!(result, Ok(()));
    assert_eq!(
        payloads.lock().expect("acquire payload lock").as_slice(),
        &[b"dm-envelope".to_vec()]
    );
}
