use axum::http::HeaderMap;
use chrono::DateTime;
use chrono::Utc;
use communication_core::{
    CandidatePeerPolicy, DescriptorValidationContext, DiscoveryPath, Ed25519DescriptorVerifier,
    NodeDescriptor, PeerRouteKind, PeeringPolicy, SelectedPeerRoute,
};
use reqwest::Url;
use ring::{
    digest::{digest, SHA256},
    signature::{Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        auth::validation::is_valid_identity_id,
        dm::validation::{
            DM_FANOUT_CIPHERTEXT_MAX_LENGTH, DM_FANOUT_MESSAGE_ID_MAX_LENGTH,
            DM_PROFILE_DEVICE_ID_MAX_LENGTH,
        },
        node_identity::LocalNodeIdentity,
    },
    state::AppState,
};

pub(crate) const NODE_FORWARD_PATH: &str = "/internal/dm/envelopes/forward";
pub(crate) const NODE_FORWARD_SIGNATURE_DOMAIN: &str = "hexrelay.node_forward_request";
const NODE_FORWARD_SIGNATURE_ALGORITHM: &str = "ed25519";
const NODE_FORWARD_ROUTE_KIND_DIRECT: &str = "static_peer_direct";
const NODE_FORWARD_SIGNATURE_MAX_SKEW_SECONDS: i64 = 300;
const NODE_FORWARD_NONCE_MIN_LENGTH: usize = 16;
const NODE_FORWARD_NONCE_MAX_LENGTH: usize = 128;
const NODE_FORWARD_TARGET_DEVICE_MAX_COUNT: usize = 100;
const HEADER_NODE_ID: &str = "x-hexrelay-node-id";
const HEADER_NODE_DESCRIPTOR_ID: &str = "x-hexrelay-node-descriptor-id";
const HEADER_SIGNATURE_ALGORITHM: &str = "x-hexrelay-node-signature-algorithm";
const HEADER_SIGNATURE_TIMESTAMP: &str = "x-hexrelay-node-signature-timestamp";
const HEADER_SIGNATURE_NONCE: &str = "x-hexrelay-node-signature-nonce";
const HEADER_SIGNATURE: &str = "x-hexrelay-node-signature";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeForwardDmEnvelopeRequest {
    pub route_kind: String,
    pub origin_node_descriptor: NodeDescriptor,
    pub destination_node_id: String,
    pub relay_node_id: Option<String>,
    pub message_id: String,
    pub thread_id: String,
    pub sender_identity_id: String,
    pub recipient_identity_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
    pub accepted_at: String,
    pub delivery_cursor: u64,
    pub target_device_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedNodeForwardRequest {
    pub origin_node_id: String,
    pub request: NodeForwardDmEnvelopeRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeForwardRequestErrorStatus {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeForwardRequestError {
    pub status: NodeForwardRequestErrorStatus,
    pub code: &'static str,
    pub message: &'static str,
}

impl NodeForwardRequestError {
    const fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status: NodeForwardRequestErrorStatus::BadRequest,
            code,
            message,
        }
    }

    const fn unauthorized(code: &'static str, message: &'static str) -> Self {
        Self {
            status: NodeForwardRequestErrorStatus::Unauthorized,
            code,
            message,
        }
    }

    const fn forbidden(code: &'static str, message: &'static str) -> Self {
        Self {
            status: NodeForwardRequestErrorStatus::Forbidden,
            code,
            message,
        }
    }

    const fn conflict(code: &'static str, message: &'static str) -> Self {
        Self {
            status: NodeForwardRequestErrorStatus::Conflict,
            code,
            message,
        }
    }
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
        route_kind: NODE_FORWARD_ROUTE_KIND_DIRECT.to_string(),
        origin_node_descriptor: identity.descriptor.clone(),
        destination_node_id: route.destination.descriptor.node_id.clone(),
        relay_node_id: None,
        message_id: input.message_id.to_string(),
        thread_id: input.thread_id.to_string(),
        sender_identity_id: input.sender_identity_id.to_string(),
        recipient_identity_id: input.recipient_identity_id.to_string(),
        ciphertext: input.ciphertext.to_string(),
        source_device_id: input.source_device_id.map(str::to_string),
        accepted_at: input.accepted_at.to_string(),
        delivery_cursor: input.delivery_cursor,
        target_device_ids: input.target_device_ids.to_vec(),
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
        .header(HEADER_NODE_ID, identity.descriptor.node_id.as_str())
        .header(
            HEADER_NODE_DESCRIPTOR_ID,
            identity.descriptor.descriptor_id.as_str(),
        )
        .header(HEADER_SIGNATURE_ALGORITHM, NODE_FORWARD_SIGNATURE_ALGORITHM)
        .header(HEADER_SIGNATURE_TIMESTAMP, timestamp)
        .header(HEADER_SIGNATURE_NONCE, nonce)
        .header(HEADER_SIGNATURE, signature)
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

pub fn authenticate_node_forward_request(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<AuthenticatedNodeForwardRequest, NodeForwardRequestError> {
    let request = serde_json::from_slice::<NodeForwardDmEnvelopeRequest>(body).map_err(|_| {
        NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "node-forwarded DM envelope body must be valid JSON",
        )
    })?;
    validate_node_forward_body(&request)?;

    let local_identity = state.local_node_identity.as_ref().ok_or_else(|| {
        NodeForwardRequestError::forbidden(
            "node_forward_local_identity_required",
            "local node identity is required to receive server-node forwarded DM envelopes",
        )
    })?;
    if request.destination_node_id != local_identity.descriptor.node_id
        || request.destination_node_id != state.node_fingerprint
    {
        return Err(NodeForwardRequestError::forbidden(
            "node_forward_destination_mismatch",
            "node-forwarded DM envelope destination does not match this node",
        ));
    }
    if request.origin_node_descriptor.node_id == local_identity.descriptor.node_id {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "origin node must differ from destination node",
        ));
    }
    if !local_identity.descriptor.accepts_local_recipient_delivery() {
        return Err(NodeForwardRequestError::forbidden(
            "node_forward_delivery_disabled",
            "local node descriptor does not accept local recipient delivery",
        ));
    }

    let header_node_id = required_header(headers, HEADER_NODE_ID)?;
    let header_descriptor_id = required_header(headers, HEADER_NODE_DESCRIPTOR_ID)?;
    let algorithm = required_header(headers, HEADER_SIGNATURE_ALGORITHM)?;
    let timestamp = required_header(headers, HEADER_SIGNATURE_TIMESTAMP)?;
    let nonce = required_header(headers, HEADER_SIGNATURE_NONCE)?;
    let signature = required_header(headers, HEADER_SIGNATURE)?;

    if header_node_id != request.origin_node_descriptor.node_id {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_node_mismatch",
            "node forwarding node header does not match the origin descriptor",
        ));
    }
    if header_descriptor_id != request.origin_node_descriptor.descriptor_id {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_descriptor_mismatch",
            "node forwarding descriptor header does not match the origin descriptor",
        ));
    }
    if algorithm != NODE_FORWARD_SIGNATURE_ALGORITHM {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_signature_invalid",
            "node forwarding signature algorithm is unsupported",
        ));
    }
    validate_nonce(nonce)?;

    let now = Utc::now().timestamp();
    let timestamp_epoch = timestamp.parse::<i64>().map_err(|_| {
        NodeForwardRequestError::unauthorized(
            "node_forward_timestamp_invalid",
            "node forwarding signature timestamp must be numeric",
        )
    })?;
    if timestamp_epoch < now - NODE_FORWARD_SIGNATURE_MAX_SKEW_SECONDS
        || timestamp_epoch > now + NODE_FORWARD_SIGNATURE_MAX_SKEW_SECONDS
    {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_timestamp_invalid",
            "node forwarding signature timestamp is outside the allowed window",
        ));
    }

    let configured_descriptor = state
        .static_peer_registry
        .find(&request.origin_node_descriptor.node_id)
        .ok_or_else(|| {
            NodeForwardRequestError::unauthorized(
                "node_forward_peer_not_allowed",
                "origin node is not an allowed static peer",
            )
        })?;
    if configured_descriptor != &request.origin_node_descriptor {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_descriptor_mismatch",
            "origin node descriptor does not match the allowed static peer descriptor",
        ));
    }

    let context = DescriptorValidationContext {
        now_epoch_seconds: now,
        max_ttl_seconds: i64::MAX,
        revoked_descriptor_ids: Vec::new(),
    };
    let origin_policy = CandidatePeerPolicy {
        discovery_path: DiscoveryPath::PrivateAllowlist,
        allowed_peering_policies: vec![PeeringPolicy::StaticAllowlist, PeeringPolicy::InviteToken],
        require_delivery: false,
        require_relay: false,
    };
    state
        .static_peer_registry
        .validate_candidate(
            &request.origin_node_descriptor.node_id,
            &context,
            &Ed25519DescriptorVerifier,
            &origin_policy,
        )
        .map_err(|_| {
            NodeForwardRequestError::unauthorized(
                "node_forward_peer_not_allowed",
                "origin node descriptor is not valid for private static peering",
            )
        })?;

    verify_forward_signature(
        &request.origin_node_descriptor,
        signature,
        timestamp,
        nonce,
        body,
    )?;
    remember_forward_nonce(
        state,
        &request.origin_node_descriptor.node_id,
        &request.origin_node_descriptor.descriptor_id,
        nonce,
        now,
    )?;

    Ok(AuthenticatedNodeForwardRequest {
        origin_node_id: request.origin_node_descriptor.node_id.clone(),
        request,
    })
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

pub(crate) fn forward_signature_payload(
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

fn validate_node_forward_body(
    request: &NodeForwardDmEnvelopeRequest,
) -> Result<(), NodeForwardRequestError> {
    if request.route_kind != NODE_FORWARD_ROUTE_KIND_DIRECT {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_route_unsupported",
            "only direct static-peer node forwarding is supported",
        ));
    }
    if request.relay_node_id.is_some() {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_route_unsupported",
            "relay node forwarding is not implemented",
        ));
    }
    validate_required_id(
        &request.destination_node_id,
        "node_forward_invalid",
        "destination_node_id must be non-empty and <= 128 chars",
    )?;
    validate_required_id(
        &request.thread_id,
        "node_forward_invalid",
        "thread_id must be non-empty and <= 128 chars",
    )?;
    validate_required_id(
        &request.message_id,
        "node_forward_invalid",
        "message_id must be non-empty and <= 128 chars",
    )?;
    if request.message_id.len() > DM_FANOUT_MESSAGE_ID_MAX_LENGTH {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "message_id must be non-empty and <= 128 chars",
        ));
    }
    if !is_valid_identity_id(&request.sender_identity_id) {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "sender_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    if !is_valid_identity_id(&request.recipient_identity_id) {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    if request.ciphertext.trim().is_empty()
        || request.ciphertext.len() > DM_FANOUT_CIPHERTEXT_MAX_LENGTH
    {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "ciphertext must be non-empty and <= 8192 chars",
        ));
    }
    if let Some(source_device_id) = &request.source_device_id {
        validate_device_id(source_device_id, "source_device_id")?;
    }
    if DateTime::parse_from_rfc3339(&request.accepted_at).is_err()
        || request.accepted_at.trim() != request.accepted_at
    {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "accepted_at must be an RFC3339 date-time without surrounding whitespace",
        ));
    }
    if request.delivery_cursor == 0 {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "delivery_cursor must be greater than zero",
        ));
    }
    if request.target_device_ids.len() > NODE_FORWARD_TARGET_DEVICE_MAX_COUNT {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            "target_device_ids exceeds maximum item count",
        ));
    }
    for device_id in &request.target_device_ids {
        validate_device_id(device_id, "target_device_ids")?;
    }

    Ok(())
}

fn validate_required_id(
    value: &str,
    code: &'static str,
    message: &'static str,
) -> Result<(), NodeForwardRequestError> {
    if value.trim().is_empty() || value.len() > 128 || value.trim() != value {
        return Err(NodeForwardRequestError::bad_request(code, message));
    }

    Ok(())
}

fn validate_device_id(value: &str, field: &'static str) -> Result<(), NodeForwardRequestError> {
    if value.trim().is_empty()
        || value.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH
        || value.trim() != value
    {
        return Err(NodeForwardRequestError::bad_request(
            "node_forward_invalid",
            match field {
                "source_device_id" => {
                    "source_device_id must be non-empty and <= 64 chars without surrounding whitespace"
                }
                _ => "target_device_ids must contain non-empty values <= 64 chars without surrounding whitespace",
            },
        ));
    }

    Ok(())
}

fn required_header<'a>(
    headers: &'a HeaderMap,
    name: &'static str,
) -> Result<&'a str, NodeForwardRequestError> {
    let value = headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            NodeForwardRequestError::unauthorized(
                "node_forward_signature_missing",
                "node forwarding signature headers are required",
            )
        })?;
    if value.trim().is_empty() || value.trim() != value {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_signature_invalid",
            "node forwarding signature headers must not be empty or padded",
        ));
    }

    Ok(value)
}

fn validate_nonce(nonce: &str) -> Result<(), NodeForwardRequestError> {
    if nonce.len() < NODE_FORWARD_NONCE_MIN_LENGTH || nonce.len() > NODE_FORWARD_NONCE_MAX_LENGTH {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_nonce_invalid",
            "node forwarding signature nonce length is invalid",
        ));
    }
    if !nonce
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(NodeForwardRequestError::unauthorized(
            "node_forward_nonce_invalid",
            "node forwarding signature nonce contains unsupported characters",
        ));
    }

    Ok(())
}

fn verify_forward_signature(
    descriptor: &NodeDescriptor,
    signature: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> Result<(), NodeForwardRequestError> {
    let public_key = hex::decode(&descriptor.node_public_key).map_err(|_| {
        NodeForwardRequestError::unauthorized(
            "node_forward_signature_invalid",
            "origin node descriptor public key is invalid",
        )
    })?;
    let signature = hex::decode(signature).map_err(|_| {
        NodeForwardRequestError::unauthorized(
            "node_forward_signature_invalid",
            "node forwarding signature must be hex encoded",
        )
    })?;

    UnparsedPublicKey::new(&ED25519, public_key)
        .verify(
            &forward_signature_payload("POST", NODE_FORWARD_PATH, timestamp, nonce, body),
            &signature,
        )
        .map_err(|_| {
            NodeForwardRequestError::unauthorized(
                "node_forward_signature_invalid",
                "node forwarding signature could not be verified",
            )
        })
}

fn remember_forward_nonce(
    state: &AppState,
    node_id: &str,
    descriptor_id: &str,
    nonce: &str,
    now_epoch_seconds: i64,
) -> Result<(), NodeForwardRequestError> {
    let mut nonces = state
        .node_forwarding_nonces
        .write()
        .expect("acquire node forwarding nonce write lock");
    nonces.retain(|_, expires_at| *expires_at >= now_epoch_seconds);

    let key = format!("{node_id}:{descriptor_id}:{nonce}");
    if nonces.contains_key(&key) {
        return Err(NodeForwardRequestError::conflict(
            "node_forward_replay",
            "node forwarding signature nonce has already been used",
        ));
    }

    nonces.insert(
        key,
        now_epoch_seconds + NODE_FORWARD_SIGNATURE_MAX_SKEW_SECONDS,
    );
    Ok(())
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

    fn state_with_forwarding_identity(
        local: &SignedDescriptor,
        origin: &SignedDescriptor,
    ) -> AppState {
        AppState::default()
            .with_local_node_identity(Some(LocalNodeIdentity {
                descriptor: local.descriptor.clone(),
                private_key_pkcs8: local.private_key_pkcs8.clone(),
            }))
            .with_static_peer_registry(
                StaticPeerRegistry::try_new(vec![origin.descriptor.clone()]).expect("registry"),
            )
    }

    fn signed_forward_request(
        origin: &SignedDescriptor,
        destination_node_id: &str,
        nonce: &str,
    ) -> (HeaderMap, Vec<u8>) {
        let request = NodeForwardDmEnvelopeRequest {
            route_kind: NODE_FORWARD_ROUTE_KIND_DIRECT.to_string(),
            origin_node_descriptor: origin.descriptor.clone(),
            destination_node_id: destination_node_id.to_string(),
            relay_node_id: None,
            message_id: "msg-node-forward-1".to_string(),
            thread_id: "thread-origin-1".to_string(),
            sender_identity_id: "usr-sender".to_string(),
            recipient_identity_id: "usr-recipient".to_string(),
            ciphertext: "enc:abcdefghijklmnopqrstuvwxyz".to_string(),
            source_device_id: Some("desktop-main".to_string()),
            accepted_at: Utc::now().to_rfc3339(),
            delivery_cursor: 1,
            target_device_ids: vec!["phone-main".to_string()],
        };
        let body = serde_json::to_vec(&request).expect("encode node forward request");
        let timestamp = Utc::now().timestamp().to_string();
        let key_pair =
            Ed25519KeyPair::from_pkcs8(&origin.private_key_pkcs8).expect("decode origin key");
        let signature = hex::encode(key_pair.sign(&forward_signature_payload(
            "POST",
            NODE_FORWARD_PATH,
            &timestamp,
            nonce,
            &body,
        )));

        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_NODE_ID,
            origin.descriptor.node_id.parse().expect("node id header"),
        );
        headers.insert(
            HEADER_NODE_DESCRIPTOR_ID,
            origin
                .descriptor
                .descriptor_id
                .parse()
                .expect("descriptor id header"),
        );
        headers.insert(
            HEADER_SIGNATURE_ALGORITHM,
            NODE_FORWARD_SIGNATURE_ALGORITHM
                .parse()
                .expect("algorithm header"),
        );
        headers.insert(
            HEADER_SIGNATURE_TIMESTAMP,
            timestamp.parse().expect("timestamp header"),
        );
        headers.insert(HEADER_SIGNATURE_NONCE, nonce.parse().expect("nonce header"));
        headers.insert(
            HEADER_SIGNATURE,
            signature.parse().expect("signature header"),
        );

        (headers, body)
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

    #[test]
    fn authenticates_static_peer_forward_request() {
        let local = signed_descriptor(
            "hexrelay-local-fingerprint",
            "descriptor-local",
            "https://local.example",
        );
        let origin =
            signed_descriptor("node-origin", "descriptor-origin", "https://origin.example");
        let state = state_with_forwarding_identity(&local, &origin);
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.node_id, "nonce-forward-auth-1");

        let authenticated = authenticate_node_forward_request(&state, &headers, &body)
            .expect("forward request should authenticate");

        assert_eq!(authenticated.origin_node_id, "node-origin");
        assert_eq!(
            authenticated.request.destination_node_id,
            local.descriptor.node_id
        );
        assert_eq!(
            authenticated.request.ciphertext,
            "enc:abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn rejects_replayed_static_peer_forward_nonce() {
        let local = signed_descriptor(
            "hexrelay-local-fingerprint",
            "descriptor-local",
            "https://local.example",
        );
        let origin =
            signed_descriptor("node-origin", "descriptor-origin", "https://origin.example");
        let state = state_with_forwarding_identity(&local, &origin);
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.node_id, "nonce-forward-replay");

        authenticate_node_forward_request(&state, &headers, &body)
            .expect("first request should authenticate");
        let error = authenticate_node_forward_request(&state, &headers, &body)
            .expect_err("replayed nonce should fail");

        assert_eq!(error.status, NodeForwardRequestErrorStatus::Conflict);
        assert_eq!(error.code, "node_forward_replay");
    }

    #[test]
    fn rejects_forward_request_from_unconfigured_origin_node() {
        let local = signed_descriptor(
            "hexrelay-local-fingerprint",
            "descriptor-local",
            "https://local.example",
        );
        let origin =
            signed_descriptor("node-origin", "descriptor-origin", "https://origin.example");
        let state = AppState::default().with_local_node_identity(Some(LocalNodeIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }));
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.node_id, "nonce-forward-denied");

        let error = authenticate_node_forward_request(&state, &headers, &body)
            .expect_err("unconfigured origin should fail");

        assert_eq!(error.status, NodeForwardRequestErrorStatus::Unauthorized);
        assert_eq!(error.code, "node_forward_peer_not_allowed");
    }

    #[test]
    fn rejects_forward_request_with_invalid_signature() {
        let local = signed_descriptor(
            "hexrelay-local-fingerprint",
            "descriptor-local",
            "https://local.example",
        );
        let origin =
            signed_descriptor("node-origin", "descriptor-origin", "https://origin.example");
        let state = state_with_forwarding_identity(&local, &origin);
        let (mut headers, body) = signed_forward_request(
            &origin,
            &local.descriptor.node_id,
            "nonce-forward-bad-signature",
        );
        headers.insert(HEADER_SIGNATURE, "00".parse().expect("signature header"));

        let error = authenticate_node_forward_request(&state, &headers, &body)
            .expect_err("invalid signature should fail");

        assert_eq!(error.status, NodeForwardRequestErrorStatus::Unauthorized);
        assert_eq!(error.code, "node_forward_signature_invalid");
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
