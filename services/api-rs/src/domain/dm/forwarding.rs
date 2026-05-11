use chrono::Utc;
use communication_core::{NodeDescriptor, PeerRouteKind, SelectedPeerRoute};
use reqwest::Url;
use ring::{
    digest::{digest, SHA256},
    signature::{Ed25519KeyPair, KeyPair},
};
use serde::Serialize;
use uuid::Uuid;

use crate::{domain::node_identity::LocalNodeIdentity, state::AppState};

const NODE_FORWARD_PATH: &str = "/internal/dm/envelopes/forward";
const NODE_FORWARD_SIGNATURE_DOMAIN: &str = "hexrelay.node_forward_request";
const NODE_FORWARD_SIGNATURE_ALGORITHM: &str = "ed25519";

pub struct ForwardDmEnvelopeInput<'a> {
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

#[derive(Serialize)]
struct NodeForwardDmEnvelopeRequest<'a> {
    route_kind: &'a str,
    origin_node_descriptor: &'a NodeDescriptor,
    destination_node_id: &'a str,
    relay_node_id: Option<&'a str>,
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

pub async fn forward_dm_envelope_to_static_peer(
    state: &AppState,
    route: &SelectedPeerRoute,
    input: ForwardDmEnvelopeInput<'_>,
) -> Result<(), String> {
    if route.kind != PeerRouteKind::Direct {
        return Err(format!(
            "server-node relay forwarding transport is not implemented for route kind {:?}",
            route.kind
        ));
    }

    let identity = state
        .local_node_identity
        .as_ref()
        .ok_or_else(|| "local node identity is required for server-node forwarding".to_string())?;
    let url = peer_forward_url(&route.destination.descriptor)?;
    let request = NodeForwardDmEnvelopeRequest {
        route_kind: "static_peer_direct",
        origin_node_descriptor: &identity.descriptor,
        destination_node_id: &route.destination.descriptor.node_id,
        relay_node_id: None,
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
    let body = serde_json::to_vec(&request)
        .map_err(|error| format!("encode node-forwarded DM envelope: {error}"))?;
    let timestamp = Utc::now().timestamp().to_string();
    let nonce = Uuid::new_v4().to_string();
    let signature = sign_forward_request(
        identity,
        "POST",
        NODE_FORWARD_PATH,
        &timestamp,
        &nonce,
        &body,
    )?;

    let response = state
        .http_client
        .post(url)
        .header("content-type", "application/json")
        .header("x-hexrelay-node-id", identity.descriptor.node_id.as_str())
        .header(
            "x-hexrelay-node-descriptor-id",
            identity.descriptor.descriptor_id.as_str(),
        )
        .header(
            "x-hexrelay-node-signature-algorithm",
            NODE_FORWARD_SIGNATURE_ALGORITHM,
        )
        .header("x-hexrelay-node-signature-timestamp", timestamp)
        .header("x-hexrelay-node-signature-nonce", nonce)
        .header("x-hexrelay-node-signature", signature)
        .body(body)
        .send()
        .await
        .map_err(|error| format!("send node-forwarded DM envelope: {error}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "node-forwarded DM envelope rejected with status {}",
            response.status()
        ))
    }
}

fn peer_forward_url(descriptor: &NodeDescriptor) -> Result<String, String> {
    let address = descriptor
        .addresses
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .ok_or_else(|| "destination node descriptor has no forwarding address".to_string())?;
    let parsed = Url::parse(address)
        .map_err(|_| "destination node descriptor address must be an absolute URL".to_string())?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("destination node descriptor address must use http or https".to_string());
    }
    if scheme == "http" && !is_loopback_host(parsed.host_str()) {
        return Err(
            "destination node descriptor address must use https for non-loopback hosts".to_string(),
        );
    }

    Ok(format!(
        "{}{}",
        address.trim_end_matches('/'),
        NODE_FORWARD_PATH
    ))
}

fn sign_forward_request(
    identity: &LocalNodeIdentity,
    method: &str,
    path: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> Result<String, String> {
    let key_pair = Ed25519KeyPair::from_pkcs8(&identity.private_key_pkcs8)
        .map_err(|_| "local node private key is invalid".to_string())?;
    let public_key = hex::encode(key_pair.public_key().as_ref());
    if public_key != identity.descriptor.node_public_key {
        return Err("local node private key does not match descriptor".to_string());
    }

    Ok(hex::encode(key_pair.sign(&forward_signature_payload(
        method, path, timestamp, nonce, body,
    ))))
}

fn forward_signature_payload(
    method: &str,
    path: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> Vec<u8> {
    [
        NODE_FORWARD_SIGNATURE_DOMAIN,
        method,
        path,
        timestamp,
        nonce,
        &hex::encode(digest(&SHA256, body).as_ref()),
    ]
    .join("\n")
    .into_bytes()
}

fn is_loopback_host(host: Option<&str>) -> bool {
    let Some(host) = host else {
        return false;
    };

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    host.parse::<std::net::IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{extract::State, http::HeaderMap, routing::post, Router};
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
        NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy,
        RelayPolicy, StaticPeerRegistry, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::{UnparsedPublicKey, ED25519};
    use serde_json::Value;
    use tokio::sync::oneshot;

    use crate::{
        domain::dm::routing::{plan_dm_envelope_route, DmEnvelopeRouteRequest},
        state::AppState,
    };

    struct SignedDescriptor {
        descriptor: NodeDescriptor,
        private_key_pkcs8: Vec<u8>,
    }

    #[derive(Debug)]
    struct CapturedForward {
        headers: HeaderMap,
        body: Vec<u8>,
    }

    fn signed_descriptor(node_id: &str, descriptor_id: &str, address: &str) -> SignedDescriptor {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = Utc::now().timestamp();
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
            addresses: vec![address.to_string()],
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

        SignedDescriptor {
            descriptor,
            private_key_pkcs8: pkcs8.as_ref().to_vec(),
        }
    }

    #[tokio::test]
    async fn forwards_direct_static_peer_envelope_with_node_signature() {
        let (base_url, capture_rx) = start_capture_server().await;
        let local = signed_descriptor("node-local", "descriptor-local", "https://local.example");
        let destination =
            signed_descriptor("node-destination", "descriptor-destination", &base_url);
        let registry =
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry");
        let route = match plan_dm_envelope_route(
            "node-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("node-destination"),
        )
        .expect("route should plan")
        {
            crate::domain::dm::routing::DmEnvelopeForwardingRoute::StaticPeer { route } => route,
            _ => panic!("expected static peer route"),
        };
        let state = AppState::default()
            .with_local_node_identity(Some(LocalNodeIdentity {
                descriptor: local.descriptor.clone(),
                private_key_pkcs8: local.private_key_pkcs8,
            }))
            .with_static_peer_registry(registry);

        forward_dm_envelope_to_static_peer(
            &state,
            &route,
            ForwardDmEnvelopeInput {
                message_id: "msg-1",
                thread_id: "thread-1",
                sender_identity_id: "usr-1",
                recipient_identity_id: "usr-2",
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz",
                source_device_id: Some("desktop-main"),
                accepted_at: "2026-03-26T00:00:00Z",
                delivery_cursor: 7,
                target_device_ids: &["phone-main".to_string()],
            },
        )
        .await
        .expect("forward should succeed");

        let captured = capture_rx.await.expect("capture forwarded request");
        assert_eq!(
            captured
                .headers
                .get("x-hexrelay-node-id")
                .and_then(|value| value.to_str().ok()),
            Some("node-local")
        );
        assert_eq!(
            captured
                .headers
                .get("x-hexrelay-node-signature-algorithm")
                .and_then(|value| value.to_str().ok()),
            Some(NODE_FORWARD_SIGNATURE_ALGORITHM)
        );

        let body: Value = serde_json::from_slice(&captured.body).expect("decode body");
        assert_eq!(body["route_kind"], "static_peer_direct");
        assert_eq!(body["destination_node_id"], "node-destination");
        assert_eq!(body["ciphertext"], "enc:abcdefghijklmnopqrstuvwxyz");

        let timestamp = captured
            .headers
            .get("x-hexrelay-node-signature-timestamp")
            .and_then(|value| value.to_str().ok())
            .expect("timestamp header");
        let nonce = captured
            .headers
            .get("x-hexrelay-node-signature-nonce")
            .and_then(|value| value.to_str().ok())
            .expect("nonce header");
        let signature = captured
            .headers
            .get("x-hexrelay-node-signature")
            .and_then(|value| value.to_str().ok())
            .expect("signature header");
        let public_key = hex::decode(local.descriptor.node_public_key).expect("decode public key");
        let signature = hex::decode(signature).expect("decode signature");
        UnparsedPublicKey::new(&ED25519, public_key)
            .verify(
                &forward_signature_payload(
                    "POST",
                    NODE_FORWARD_PATH,
                    timestamp,
                    nonce,
                    &captured.body,
                ),
                &signature,
            )
            .expect("signature should verify");
    }

    #[tokio::test]
    async fn rejects_direct_static_peer_forward_without_local_node_identity() {
        let destination = signed_descriptor(
            "node-destination",
            "descriptor-destination",
            "https://node.example",
        );
        let registry =
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry");
        let route = match plan_dm_envelope_route(
            "node-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("node-destination"),
        )
        .expect("route should plan")
        {
            crate::domain::dm::routing::DmEnvelopeForwardingRoute::StaticPeer { route } => route,
            _ => panic!("expected static peer route"),
        };

        let error = forward_dm_envelope_to_static_peer(
            &AppState::default(),
            &route,
            ForwardDmEnvelopeInput {
                message_id: "msg-1",
                thread_id: "thread-1",
                sender_identity_id: "usr-1",
                recipient_identity_id: "usr-2",
                ciphertext: "enc:abcdefghijklmnopqrstuvwxyz",
                source_device_id: None,
                accepted_at: "2026-03-26T00:00:00Z",
                delivery_cursor: 7,
                target_device_ids: &["phone-main".to_string()],
            },
        )
        .await
        .expect_err("missing identity should fail");

        assert!(error.contains("local node identity"));
    }

    async fn start_capture_server() -> (String, oneshot::Receiver<CapturedForward>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind capture server");
        let addr = listener.local_addr().expect("capture server address");
        let (tx, rx) = oneshot::channel::<CapturedForward>();
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));
        let app = Router::new()
            .route(NODE_FORWARD_PATH, post(capture_forward))
            .with_state(state);

        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        (format!("http://{}", addr), rx)
    }

    async fn capture_forward(
        State(sender): State<
            std::sync::Arc<tokio::sync::Mutex<Option<oneshot::Sender<CapturedForward>>>>,
        >,
        headers: HeaderMap,
        body: axum::body::Bytes,
    ) -> axum::http::StatusCode {
        if let Some(sender) = sender.lock().await.take() {
            let _ = sender.send(CapturedForward {
                headers,
                body: body.to_vec(),
            });
        }

        axum::http::StatusCode::ACCEPTED
    }
}
