use communication_core::{
    domain::CommunicationMode,
    send_via_node_dispatch_with_provenance,
    transport::{NodeDispatch, TransportError},
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    domain::dm::routing::{
        plan_dm_envelope_route, DmEnvelopeForwardingRoute, DmEnvelopeRouteRequest,
    },
    state::AppState,
    transport::http::adapters::dm_forwarding::{
        forward_dm_envelope_to_static_peer, ForwardDmEnvelopeInput,
    },
};

const INTERNAL_DM_ENVELOPE_DISPATCH_PATH: &str = "/internal/dm/envelopes/dispatch";

#[derive(Serialize)]
struct DmEnvelopeDispatchRequest<'a> {
    message_id: &'a str,
    thread_id: &'a str,
    sender_identity_id: &'a str,
    recipient_identity_id: &'a str,
    ciphertext: &'a str,
    source_device_id: Option<&'a str>,
    accepted_at: &'a str,
    delivery_cursor: u64,
    target_device_ids: &'a [String],
}

#[derive(Deserialize, Serialize)]
struct OwnedDmEnvelopeDispatchRequest {
    message_id: String,
    thread_id: String,
    sender_identity_id: String,
    recipient_identity_id: String,
    ciphertext: String,
    source_device_id: Option<String>,
    accepted_at: String,
    delivery_cursor: u64,
    target_device_ids: Vec<String>,
}

#[derive(Deserialize)]
struct DmEnvelopeDispatchInternalResponse {
    summary: DmEnvelopeDispatchSummary,
}

#[derive(Deserialize)]
struct DmEnvelopeDispatchSummary {
    message_id: String,
    recipient_identity_id: String,
    target_device_count: u32,
    queued_device_ids: Vec<String>,
    pending_device_ids: Vec<String>,
    no_connection_device_ids: Vec<String>,
    unverified_device_ids: Vec<String>,
    saturated_device_ids: Vec<String>,
    stale_connection_count: u32,
}

pub struct DispatchDmEnvelopeInput<'a> {
    pub destination_node_id: Option<&'a str>,
    pub message_id: &'a str,
    pub thread_id: &'a str,
    pub sender_identity_id: &'a str,
    pub recipient_identity_id: &'a str,
    pub ciphertext: &'a str,
    pub source_device_id: Option<&'a str>,
    pub accepted_at: &'a str,
    pub delivery_cursor: u64,
    pub target_device_ids: &'a [String],
}

#[derive(Clone)]
struct RealtimeNodeDispatchSender {
    http_client: reqwest::Client,
    realtime_base_url: String,
    internal_token: String,
}

impl NodeDispatch for RealtimeNodeDispatchSender {
    fn send_payload(&self, payload: &[u8]) -> Result<(), TransportError> {
        let dispatch = RealtimeNodeDispatch::from_payload(payload)?;
        let http_client = self.http_client.clone();
        let url = format!(
            "{}{}",
            self.realtime_base_url.trim_end_matches('/'),
            dispatch.path()
        );
        let path = dispatch.path().to_string();
        let message_id = dispatch.message_id().to_string();
        let thread_id = dispatch.thread_id().to_string();
        let recipient_identity_id = dispatch.recipient_identity_id().to_string();
        let internal_token = self.internal_token.clone();
        let body = dispatch.body().to_vec();
        let handle =
            tokio::runtime::Handle::try_current().map_err(|_| TransportError::SendFailed)?;
        handle.spawn(async move {
            match http_client
                .post(url)
                .header("x-hexrelay-internal-token", internal_token)
                .header("content-type", "application/json")
                .body(body)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    match response.json::<DmEnvelopeDispatchInternalResponse>().await {
                        Ok(report) => {
                            info!(
                                %path,
                                message_id = %report.summary.message_id,
                                thread_id = %thread_id,
                                recipient_identity_id = %report.summary.recipient_identity_id,
                                target_device_count = report.summary.target_device_count,
                                queued_device_count = report.summary.queued_device_ids.len(),
                                pending_device_count = report.summary.pending_device_ids.len(),
                                no_connection_device_count = report.summary.no_connection_device_ids.len(),
                                unverified_device_count = report.summary.unverified_device_ids.len(),
                                saturated_device_count = report.summary.saturated_device_ids.len(),
                                stale_connection_count = report.summary.stale_connection_count,
                                "NodeClientTransport DM envelope dispatch accepted by realtime"
                            );
                        }
                        Err(error) => {
                            warn!(
                                %path,
                                %message_id,
                                %thread_id,
                                %recipient_identity_id,
                                error = %error,
                                "NodeClientTransport DM envelope dispatch summary decode failed"
                            );
                        }
                    }
                }
                Ok(response) => {
                    warn!(
                        %path,
                        %message_id,
                        %thread_id,
                        %recipient_identity_id,
                        status = %response.status(),
                        "NodeClientTransport DM envelope dispatch failed"
                    );
                }
                Err(error) => {
                    warn!(
                        %path,
                        %message_id,
                        %thread_id,
                        %recipient_identity_id,
                        error = %error,
                        "NodeClientTransport DM envelope dispatch errored"
                    );
                }
            }
        });

        Ok(())
    }
}

struct RealtimeNodeDispatch {
    body: Vec<u8>,
    message_id: String,
    thread_id: String,
    recipient_identity_id: String,
}

impl RealtimeNodeDispatch {
    fn from_payload(payload: &[u8]) -> Result<Self, TransportError> {
        let body: OwnedDmEnvelopeDispatchRequest =
            serde_json::from_slice(payload).map_err(|_| TransportError::SendFailed)?;
        Ok(Self {
            body: serde_json::to_vec(&body).map_err(|_| TransportError::SendFailed)?,
            message_id: body.message_id,
            thread_id: body.thread_id,
            recipient_identity_id: body.recipient_identity_id,
        })
    }

    fn path(&self) -> &'static str {
        INTERNAL_DM_ENVELOPE_DISPATCH_PATH
    }

    fn body(&self) -> &[u8] {
        &self.body
    }

    fn message_id(&self) -> &str {
        &self.message_id
    }

    fn thread_id(&self) -> &str {
        &self.thread_id
    }

    fn recipient_identity_id(&self) -> &str {
        &self.recipient_identity_id
    }
}

pub async fn dispatch_dm_envelope(
    state: &AppState,
    input: DispatchDmEnvelopeInput<'_>,
) -> Result<(), String> {
    let route = plan_dm_envelope_route(
        &state.node_fingerprint,
        &state.static_peer_registry,
        DmEnvelopeRouteRequest {
            destination_node_id: input.destination_node_id,
            ..DmEnvelopeRouteRequest::local_realtime()
        },
    )
    .map_err(|error| format!("plan DM envelope route: {error}"))?;

    if let DmEnvelopeForwardingRoute::StaticPeer { route } = route {
        return forward_dm_envelope_to_static_peer(
            state,
            &route,
            ForwardDmEnvelopeInput {
                message_id: input.message_id,
                thread_id: input.thread_id,
                sender_identity_id: input.sender_identity_id,
                recipient_identity_id: input.recipient_identity_id,
                ciphertext: input.ciphertext,
                source_device_id: input.source_device_id,
                accepted_at: input.accepted_at,
                delivery_cursor: input.delivery_cursor,
                target_device_ids: input.target_device_ids,
            },
        )
        .await;
    }

    let request = DmEnvelopeDispatchRequest {
        message_id: input.message_id,
        thread_id: input.thread_id,
        sender_identity_id: input.sender_identity_id,
        recipient_identity_id: input.recipient_identity_id,
        ciphertext: input.ciphertext,
        source_device_id: input.source_device_id,
        accepted_at: input.accepted_at,
        delivery_cursor: input.delivery_cursor,
        target_device_ids: input.target_device_ids,
    };
    let payload = serde_json::to_vec(&request)
        .map_err(|error| format!("encode DM envelope dispatch payload: {error}"))?;

    let outcome = send_via_node_dispatch_with_provenance(
        CommunicationMode::DmEnvelope,
        communication_core::PolicyContext::default(),
        RealtimeNodeDispatchSender {
            http_client: state.http_client.clone(),
            realtime_base_url: state.realtime_base_url.clone(),
            internal_token: state.channel_dispatch_internal_token.clone(),
        },
        payload,
    )
    .map_err(|error| {
        format!(
            "dispatch DM envelope via NodeClientTransport: {}",
            error.code.as_str()
        )
    })?;

    debug!(
        mode = outcome.provenance.mode.as_str(),
        profile = outcome.provenance.profile.as_str(),
        reason_code = outcome.provenance.reason_code.as_str(),
        policy_assertions = ?outcome.provenance.policy_assertions,
        "NodeClientTransport DM envelope dispatch provenance emitted"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
        NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy,
        RelayPolicy, StaticPeerRegistry, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::Ed25519KeyPair;

    fn signed_descriptor(node_id: &str, descriptor_id: &str) -> NodeDescriptor {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = chrono::Utc::now().timestamp();
        let mut descriptor = NodeDescriptor {
            node_id: node_id.to_string(),
            node_public_key: public_key,
            descriptor_id: descriptor_id.to_string(),
            issued_at_epoch_seconds: now - 1,
            expires_at_epoch_seconds: now + 300,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy: PeeringPolicy::StaticAllowlist,
            relay_policy: RelayPolicy::None,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec![format!("https://{node_id}.example")],
            supported_protocols: vec!["hexrelay-node-http".to_string()],
            rate_limits: Vec::new(),
            trust_labels: Vec::new(),
            revocation_pointer: None,
            signature: NodeSignature {
                algorithm: NodeSignatureAlgorithm::Ed25519,
                value: String::new(),
            },
        };
        descriptor.signature.value =
            sign_descriptor_ed25519_pkcs8(&descriptor, pkcs8.as_ref()).expect("sign descriptor");
        descriptor
    }

    #[test]
    fn dispatch_payload_maps_to_internal_dm_envelope_path() {
        let target_device_ids = vec!["desktop-main".to_string(), "phone-main".to_string()];
        let payload = serde_json::to_vec(&DmEnvelopeDispatchRequest {
            message_id: "msg-1",
            thread_id: "thread-1",
            sender_identity_id: "usr-1",
            recipient_identity_id: "usr-2",
            ciphertext: "enc:abcdefghijklmnopqrstuvwxyz",
            source_device_id: Some("sender-desktop"),
            accepted_at: "2026-03-26T00:00:00Z",
            delivery_cursor: 7,
            target_device_ids: &target_device_ids,
        })
        .expect("encode dispatch request");

        let dispatch = RealtimeNodeDispatch::from_payload(&payload).expect("parse dispatch");
        assert_eq!(dispatch.path(), INTERNAL_DM_ENVELOPE_DISPATCH_PATH);

        let body_value: serde_json::Value =
            serde_json::from_slice(dispatch.body()).expect("parse dispatch body");
        assert_eq!(body_value["message_id"], "msg-1");
        assert_eq!(body_value["thread_id"], "thread-1");
        assert_eq!(body_value["recipient_identity_id"], "usr-2");
        assert_eq!(body_value["delivery_cursor"], 7);
        assert_eq!(
            body_value["target_device_ids"],
            serde_json::json!(target_device_ids)
        );
    }

    #[tokio::test]
    async fn dispatch_fails_closed_for_static_peer_destination_without_local_identity() {
        let registry =
            StaticPeerRegistry::try_new(vec![signed_descriptor("node-peer", "descriptor-peer")])
                .expect("registry should build");
        let state = AppState::default().with_static_peer_registry(registry);

        let error = dispatch_dm_envelope(
            &state,
            DispatchDmEnvelopeInput {
                destination_node_id: Some("node-peer"),
                message_id: "msg-1",
                thread_id: "thread-1",
                sender_identity_id: "usr-1",
                recipient_identity_id: "usr-2",
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz",
                source_device_id: Some("sender-desktop"),
                accepted_at: "2026-03-26T00:00:00Z",
                delivery_cursor: 7,
                target_device_ids: &["desktop-main".to_string()],
            },
        )
        .await
        .expect_err("static peer dispatch should fail closed without local node identity");

        assert!(error.contains("local node identity"));
    }
}
