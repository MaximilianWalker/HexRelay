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
    send_via_server_dispatch, send_via_server_dispatch_with_provenance,
    DispatchingServerClientTransport, ServerClientTransport, ServerDispatch, TransportError,
};

#[derive(Clone)]
struct RecordingServerClient {
    connect_calls: Arc<AtomicUsize>,
    send_calls: Arc<AtomicUsize>,
}

impl ServerClientTransport for RecordingServerClient {
    fn connect(&self, intent: &ConnectIntent) -> Result<SessionProvenance, TransportError> {
        self.connect_calls.fetch_add(1, Ordering::SeqCst);
        Ok(SessionProvenance {
            mode: intent.mode,
            profile: TransportProfile::ServerClient,
            reason_code: CommunicationReasonCode::DmEnvelopeServerRouteSelected,
            policy_assertions: vec!["dm_envelope_server_policy_compliant".to_string()],
        })
    }

    fn send(&self, _envelope: &SendEnvelope) -> Result<(), TransportError> {
        self.send_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Clone)]
struct FailingServerClient;

impl ServerClientTransport for FailingServerClient {
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

impl ServerDispatch for RecordingDispatch {
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError> {
        self.payloads
            .lock()
            .expect("acquire payload lock")
            .push(payload.to_vec());
        Ok(())
    }
}

#[test]
fn routes_dm_envelope_connect_through_server_adapter() {
    let server_connect_calls = Arc::new(AtomicUsize::new(0));
    let server_send_calls = Arc::new(AtomicUsize::new(0));
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingServerClient {
            connect_calls: Arc::clone(&server_connect_calls),
            send_calls: server_send_calls,
        },
    );

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmEnvelope,
        target: ConnectTarget::ServerEndpoint {
            endpoint: "https://server.example".to_string(),
        },
    });

    assert!(result.is_ok());
    assert_eq!(server_connect_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn routes_dm_envelope_send_through_server_adapter() {
    let server_connect_calls = Arc::new(AtomicUsize::new(0));
    let server_send_calls = Arc::new(AtomicUsize::new(0));
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingServerClient {
            connect_calls: server_connect_calls,
            send_calls: Arc::clone(&server_send_calls),
        },
    );

    let result = router.send(&SendEnvelope {
        mode: CommunicationMode::DmEnvelope,
        payload: b"dm-envelope".to_vec(),
    });

    assert_eq!(result, Ok(()));
    assert_eq!(server_send_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn send_with_provenance_returns_stable_server_channel_outcome() {
    let server_connect_calls = Arc::new(AtomicUsize::new(0));
    let server_send_calls = Arc::new(AtomicUsize::new(0));
    let router = CommunicationRouter::new(
        PolicyContext::default(),
        RecordingServerClient {
            connect_calls: server_connect_calls,
            send_calls: Arc::clone(&server_send_calls),
        },
    );

    let result = router
        .send_with_provenance(&SendEnvelope {
            mode: CommunicationMode::ServerChannel,
            payload: b"server-channel".to_vec(),
        })
        .expect("server channel dispatch should route");

    assert_eq!(server_send_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.provenance.mode.as_str(), "server_channel");
    assert_eq!(result.provenance.profile.as_str(), "server_client");
    assert_eq!(
        result.provenance.reason_code.as_str(),
        "server_channel_route_selected"
    );
    assert_eq!(
        result.provenance.policy_assertions,
        vec!["server_channel_policy_compliant".to_string()]
    );
}

#[test]
fn maps_transport_connect_failure_to_reason_code() {
    let router = CommunicationRouter::new(PolicyContext::default(), FailingServerClient);

    let result = router.connect(&ConnectIntent {
        mode: CommunicationMode::DmEnvelope,
        target: ConnectTarget::ServerEndpoint {
            endpoint: "https://server.invalid".to_string(),
        },
    });

    assert_eq!(
        result,
        Err(CommunicationError {
            code: CommunicationReasonCode::TransportConnectFailed,
            mode: CommunicationMode::DmEnvelope,
            profile: Some(TransportProfile::ServerClient),
        })
    );
}

#[test]
fn maps_mode_disabled_to_reason_code() {
    let policy = PolicyContext {
        enable_server_channel: false,
        ..PolicyContext::default()
    };
    let router = CommunicationRouter::new(policy, FailingServerClient);

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
fn dispatching_server_client_transport_rejects_wrong_mode_payload() {
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let transport = DispatchingServerClientTransport::new(
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
fn dispatching_server_client_transport_rejects_wrong_mode_connect() {
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let transport = DispatchingServerClientTransport::new(
        CommunicationMode::Presence,
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
    );

    let result = transport.connect(&ConnectIntent {
        mode: CommunicationMode::ServerChannel,
        target: ConnectTarget::ServerEndpoint {
            endpoint: "https://server.invalid".to_string(),
        },
    });

    assert_eq!(result, Err(TransportError::ConnectFailed));
}

#[test]
fn dispatching_server_client_transport_forwards_payload_for_matching_mode() {
    let payloads = Arc::new(Mutex::new(Vec::new()));
    let transport = DispatchingServerClientTransport::new(
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
fn send_via_server_dispatch_routes_dm_envelope_through_server_client_bootstrap() {
    let payloads = Arc::new(Mutex::new(Vec::new()));

    let result = send_via_server_dispatch(
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

#[test]
fn send_via_server_dispatch_with_provenance_returns_presence_outcome() {
    let payloads = Arc::new(Mutex::new(Vec::new()));

    let result = send_via_server_dispatch_with_provenance(
        CommunicationMode::Presence,
        PolicyContext::default(),
        RecordingDispatch {
            payloads: Arc::clone(&payloads),
        },
        b"presence".to_vec(),
    )
    .expect("presence dispatch should route");

    assert_eq!(
        payloads.lock().expect("acquire payload lock").as_slice(),
        &[b"presence".to_vec()]
    );
    assert_eq!(result.provenance.mode.as_str(), "presence");
    assert_eq!(
        result.provenance.reason_code.as_str(),
        "presence_route_selected"
    );
}
