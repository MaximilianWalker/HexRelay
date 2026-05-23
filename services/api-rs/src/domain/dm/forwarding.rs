use axum::http::HeaderMap;
use chrono::DateTime;
use chrono::Utc;
use communication_core::{
    CandidatePeerPolicy, DescriptorValidationContext, DiscoveryPath, Ed25519DescriptorVerifier,
    PeerRouteKind, PeeringPolicy, SelectedPeerRoute, ServerDescriptor,
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
        server_identity::LocalServerIdentity,
    },
    state::AppState,
};

pub(crate) const SERVER_FORWARD_PATH: &str = "/internal/dm/envelopes/forward";
pub(crate) const SERVER_FORWARD_SIGNATURE_DOMAIN: &str = "hexrelay.server_forward_request";
const SERVER_FORWARD_SIGNATURE_ALGORITHM: &str = "ed25519";
const SERVER_FORWARD_ROUTE_KIND_DIRECT: &str = "static_peer_direct";
const SERVER_FORWARD_SIGNATURE_MAX_SKEW_SECONDS: i64 = 300;
const SERVER_FORWARD_NONCE_MIN_LENGTH: usize = 16;
const SERVER_FORWARD_NONCE_MAX_LENGTH: usize = 128;
const SERVER_FORWARD_TARGET_DEVICE_MAX_COUNT: usize = 100;
const HEADER_SERVER_ID: &str = "x-hexrelay-server-id";
const HEADER_SERVER_DESCRIPTOR_ID: &str = "x-hexrelay-server-descriptor-id";
const HEADER_SIGNATURE_ALGORITHM: &str = "x-hexrelay-server-signature-algorithm";
const HEADER_SIGNATURE_TIMESTAMP: &str = "x-hexrelay-server-signature-timestamp";
const HEADER_SIGNATURE_NONCE: &str = "x-hexrelay-server-signature-nonce";
const HEADER_SIGNATURE: &str = "x-hexrelay-server-signature";

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
pub struct ServerForwardDmEnvelopeRequest {
    pub route_kind: String,
    pub origin_server_descriptor: ServerDescriptor,
    pub destination_server_id: String,
    pub relay_server_id: Option<String>,
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
pub struct AuthenticatedServerForwardRequest {
    pub origin_server_id: String,
    pub request: ServerForwardDmEnvelopeRequest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerForwardRequestErrorStatus {
    BadRequest,
    Unauthorized,
    Forbidden,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerForwardRequestError {
    pub status: ServerForwardRequestErrorStatus,
    pub code: &'static str,
    pub message: &'static str,
}

impl ServerForwardRequestError {
    const fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self {
            status: ServerForwardRequestErrorStatus::BadRequest,
            code,
            message,
        }
    }

    const fn unauthorized(code: &'static str, message: &'static str) -> Self {
        Self {
            status: ServerForwardRequestErrorStatus::Unauthorized,
            code,
            message,
        }
    }

    const fn forbidden(code: &'static str, message: &'static str) -> Self {
        Self {
            status: ServerForwardRequestErrorStatus::Forbidden,
            code,
            message,
        }
    }

    const fn conflict(code: &'static str, message: &'static str) -> Self {
        Self {
            status: ServerForwardRequestErrorStatus::Conflict,
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
            "server-to-server relay forwarding transport is not implemented for route kind {:?}",
            route.kind
        ));
    }

    let identity = state.local_server_identity.as_ref().ok_or_else(|| {
        "local server identity is required for server-to-server forwarding".to_string()
    })?;
    let url = peer_forward_url(&route.destination.descriptor)?;
    let request = ServerForwardDmEnvelopeRequest {
        route_kind: SERVER_FORWARD_ROUTE_KIND_DIRECT.to_string(),
        origin_server_descriptor: identity.descriptor.clone(),
        destination_server_id: route.destination.descriptor.server_id.clone(),
        relay_server_id: None,
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
        .map_err(|error| format!("encode server-forwarded DM envelope: {error}"))?;
    let timestamp = Utc::now().timestamp().to_string();
    let nonce = Uuid::new_v4().to_string();
    let signature = sign_forward_request(
        identity,
        "POST",
        SERVER_FORWARD_PATH,
        &timestamp,
        &nonce,
        &body,
    )?;

    let response = state
        .http_client
        .post(url)
        .header("content-type", "application/json")
        .header(HEADER_SERVER_ID, identity.descriptor.server_id.as_str())
        .header(
            HEADER_SERVER_DESCRIPTOR_ID,
            identity.descriptor.descriptor_id.as_str(),
        )
        .header(
            HEADER_SIGNATURE_ALGORITHM,
            SERVER_FORWARD_SIGNATURE_ALGORITHM,
        )
        .header(HEADER_SIGNATURE_TIMESTAMP, timestamp)
        .header(HEADER_SIGNATURE_NONCE, nonce)
        .header(HEADER_SIGNATURE, signature)
        .body(body)
        .send()
        .await
        .map_err(|error| format!("send server-forwarded DM envelope: {error}"))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!(
            "server-forwarded DM envelope rejected with status {}",
            response.status()
        ))
    }
}

pub fn authenticate_server_forward_request(
    state: &AppState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<AuthenticatedServerForwardRequest, ServerForwardRequestError> {
    let request = serde_json::from_slice::<ServerForwardDmEnvelopeRequest>(body).map_err(|_| {
        ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "server-forwarded DM envelope body must be valid JSON",
        )
    })?;
    validate_server_forward_body(&request)?;

    let local_identity = state.local_server_identity.as_ref().ok_or_else(|| {
        ServerForwardRequestError::forbidden(
            "server_forward_local_identity_required",
            "local server identity is required to receive server-to-server forwarded DM envelopes",
        )
    })?;
    if request.destination_server_id != local_identity.descriptor.server_id
        || request.destination_server_id != state.server_id
    {
        return Err(ServerForwardRequestError::forbidden(
            "server_forward_destination_mismatch",
            "server-forwarded DM envelope destination does not match this server",
        ));
    }
    if request.origin_server_descriptor.server_id == local_identity.descriptor.server_id {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "origin server must differ from destination server",
        ));
    }
    if !local_identity.descriptor.accepts_local_recipient_delivery() {
        return Err(ServerForwardRequestError::forbidden(
            "server_forward_delivery_disabled",
            "local server descriptor does not accept local recipient delivery",
        ));
    }

    let header_server_id = required_header(headers, HEADER_SERVER_ID)?;
    let header_descriptor_id = required_header(headers, HEADER_SERVER_DESCRIPTOR_ID)?;
    let algorithm = required_header(headers, HEADER_SIGNATURE_ALGORITHM)?;
    let timestamp = required_header(headers, HEADER_SIGNATURE_TIMESTAMP)?;
    let nonce = required_header(headers, HEADER_SIGNATURE_NONCE)?;
    let signature = required_header(headers, HEADER_SIGNATURE)?;

    if header_server_id != request.origin_server_descriptor.server_id {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_server_mismatch",
            "server forwarding server header does not match the origin descriptor",
        ));
    }
    if header_descriptor_id != request.origin_server_descriptor.descriptor_id {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_descriptor_mismatch",
            "server forwarding descriptor header does not match the origin descriptor",
        ));
    }
    if algorithm != SERVER_FORWARD_SIGNATURE_ALGORITHM {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_signature_invalid",
            "server forwarding signature algorithm is unsupported",
        ));
    }
    validate_nonce(nonce)?;

    let now = Utc::now().timestamp();
    let timestamp_epoch = timestamp.parse::<i64>().map_err(|_| {
        ServerForwardRequestError::unauthorized(
            "server_forward_timestamp_invalid",
            "server forwarding signature timestamp must be numeric",
        )
    })?;
    if timestamp_epoch < now - SERVER_FORWARD_SIGNATURE_MAX_SKEW_SECONDS
        || timestamp_epoch > now + SERVER_FORWARD_SIGNATURE_MAX_SKEW_SECONDS
    {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_timestamp_invalid",
            "server forwarding signature timestamp is outside the allowed window",
        ));
    }

    let configured_descriptor = state
        .static_peer_registry
        .find(&request.origin_server_descriptor.server_id)
        .ok_or_else(|| {
            ServerForwardRequestError::unauthorized(
                "server_forward_peer_not_allowed",
                "origin server is not an allowed static peer",
            )
        })?;
    if configured_descriptor != &request.origin_server_descriptor {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_descriptor_mismatch",
            "origin server descriptor does not match the allowed static peer descriptor",
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
            &request.origin_server_descriptor.server_id,
            &context,
            &Ed25519DescriptorVerifier,
            &origin_policy,
        )
        .map_err(|_| {
            ServerForwardRequestError::unauthorized(
                "server_forward_peer_not_allowed",
                "origin server descriptor is not valid for private static peering",
            )
        })?;

    verify_forward_signature(
        &request.origin_server_descriptor,
        signature,
        timestamp,
        nonce,
        body,
    )?;
    remember_forward_nonce(
        state,
        &request.origin_server_descriptor.server_id,
        &request.origin_server_descriptor.descriptor_id,
        nonce,
        now,
    )?;

    Ok(AuthenticatedServerForwardRequest {
        origin_server_id: request.origin_server_descriptor.server_id.clone(),
        request,
    })
}

fn peer_forward_url(descriptor: &ServerDescriptor) -> Result<String, String> {
    let address = descriptor
        .addresses
        .iter()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .ok_or_else(|| "destination server descriptor has no forwarding address".to_string())?;
    let parsed = Url::parse(address)
        .map_err(|_| "destination server descriptor address must be an absolute URL".to_string())?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("destination server descriptor address must use http or https".to_string());
    }
    if scheme == "http" && !is_loopback_host(parsed.host_str()) {
        return Err(
            "destination server descriptor address must use https for non-loopback hosts"
                .to_string(),
        );
    }

    Ok(format!(
        "{}{}",
        address.trim_end_matches('/'),
        SERVER_FORWARD_PATH
    ))
}

fn sign_forward_request(
    identity: &LocalServerIdentity,
    method: &str,
    path: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> Result<String, String> {
    let key_pair = Ed25519KeyPair::from_pkcs8(&identity.private_key_pkcs8)
        .map_err(|_| "local server private key is invalid".to_string())?;
    let public_key = hex::encode(key_pair.public_key().as_ref());
    if public_key != identity.descriptor.server_public_key {
        return Err("local server private key does not match descriptor".to_string());
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
        SERVER_FORWARD_SIGNATURE_DOMAIN,
        method,
        path,
        timestamp,
        nonce,
        &hex::encode(digest(&SHA256, body).as_ref()),
    ]
    .join("\n")
    .into_bytes()
}

fn validate_server_forward_body(
    request: &ServerForwardDmEnvelopeRequest,
) -> Result<(), ServerForwardRequestError> {
    if request.route_kind != SERVER_FORWARD_ROUTE_KIND_DIRECT {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_route_unsupported",
            "only direct static-peer server forwarding is supported",
        ));
    }
    if request.relay_server_id.is_some() {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_route_unsupported",
            "relay server forwarding is not implemented",
        ));
    }
    validate_required_id(
        &request.destination_server_id,
        "server_forward_invalid",
        "destination_server_id must be non-empty and <= 128 chars",
    )?;
    validate_required_id(
        &request.thread_id,
        "server_forward_invalid",
        "thread_id must be non-empty and <= 128 chars",
    )?;
    validate_required_id(
        &request.message_id,
        "server_forward_invalid",
        "message_id must be non-empty and <= 128 chars",
    )?;
    if request.message_id.len() > DM_FANOUT_MESSAGE_ID_MAX_LENGTH {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "message_id must be non-empty and <= 128 chars",
        ));
    }
    if !is_valid_identity_id(&request.sender_identity_id) {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "sender_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    if !is_valid_identity_id(&request.recipient_identity_id) {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }
    if request.ciphertext.trim().is_empty()
        || request.ciphertext.len() > DM_FANOUT_CIPHERTEXT_MAX_LENGTH
    {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "ciphertext must be non-empty and <= 8192 chars",
        ));
    }
    if let Some(source_device_id) = &request.source_device_id {
        validate_device_id(source_device_id, "source_device_id")?;
    }
    if DateTime::parse_from_rfc3339(&request.accepted_at).is_err()
        || request.accepted_at.trim() != request.accepted_at
    {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "accepted_at must be an RFC3339 date-time without surrounding whitespace",
        ));
    }
    if request.delivery_cursor == 0 {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
            "delivery_cursor must be greater than zero",
        ));
    }
    if request.target_device_ids.len() > SERVER_FORWARD_TARGET_DEVICE_MAX_COUNT {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
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
) -> Result<(), ServerForwardRequestError> {
    if value.trim().is_empty() || value.len() > 128 || value.trim() != value {
        return Err(ServerForwardRequestError::bad_request(code, message));
    }

    Ok(())
}

fn validate_device_id(value: &str, field: &'static str) -> Result<(), ServerForwardRequestError> {
    if value.trim().is_empty()
        || value.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH
        || value.trim() != value
    {
        return Err(ServerForwardRequestError::bad_request(
            "server_forward_invalid",
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
) -> Result<&'a str, ServerForwardRequestError> {
    let value = headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ServerForwardRequestError::unauthorized(
                "server_forward_signature_missing",
                "server forwarding signature headers are required",
            )
        })?;
    if value.trim().is_empty() || value.trim() != value {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_signature_invalid",
            "server forwarding signature headers must not be empty or padded",
        ));
    }

    Ok(value)
}

fn validate_nonce(nonce: &str) -> Result<(), ServerForwardRequestError> {
    if nonce.len() < SERVER_FORWARD_NONCE_MIN_LENGTH
        || nonce.len() > SERVER_FORWARD_NONCE_MAX_LENGTH
    {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_nonce_invalid",
            "server forwarding signature nonce length is invalid",
        ));
    }
    if !nonce
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(ServerForwardRequestError::unauthorized(
            "server_forward_nonce_invalid",
            "server forwarding signature nonce contains unsupported characters",
        ));
    }

    Ok(())
}

fn verify_forward_signature(
    descriptor: &ServerDescriptor,
    signature: &str,
    timestamp: &str,
    nonce: &str,
    body: &[u8],
) -> Result<(), ServerForwardRequestError> {
    let public_key = hex::decode(&descriptor.server_public_key).map_err(|_| {
        ServerForwardRequestError::unauthorized(
            "server_forward_signature_invalid",
            "origin server descriptor public key is invalid",
        )
    })?;
    let signature = hex::decode(signature).map_err(|_| {
        ServerForwardRequestError::unauthorized(
            "server_forward_signature_invalid",
            "server forwarding signature must be hex encoded",
        )
    })?;

    UnparsedPublicKey::new(&ED25519, public_key)
        .verify(
            &forward_signature_payload("POST", SERVER_FORWARD_PATH, timestamp, nonce, body),
            &signature,
        )
        .map_err(|_| {
            ServerForwardRequestError::unauthorized(
                "server_forward_signature_invalid",
                "server forwarding signature could not be verified",
            )
        })
}

fn remember_forward_nonce(
    state: &AppState,
    server_id: &str,
    descriptor_id: &str,
    nonce: &str,
    now_epoch_seconds: i64,
) -> Result<(), ServerForwardRequestError> {
    let mut nonces = state
        .server_forwarding_nonces
        .write()
        .expect("acquire server forwarding nonce write lock");
    nonces.retain(|_, expires_at| *expires_at >= now_epoch_seconds);

    let key = format!("{server_id}:{descriptor_id}:{nonce}");
    if nonces.contains_key(&key) {
        return Err(ServerForwardRequestError::conflict(
            "server_forward_replay",
            "server forwarding signature nonce has already been used",
        ));
    }

    nonces.insert(
        key,
        now_epoch_seconds + SERVER_FORWARD_SIGNATURE_MAX_SKEW_SECONDS,
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
        NetworkMode, PeeringPolicy, RelayPolicy, ServerDescriptor, ServerSignature,
        ServerSignatureAlgorithm, StaticPeerRegistry, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::{UnparsedPublicKey, ED25519};
    use serde_json::Value;
    use tokio::sync::oneshot;

    use crate::{
        domain::{
            dm::routing::{plan_dm_envelope_route, DmEnvelopeRouteRequest},
            peer_invites::{issue_peer_invite, PeerInviteIssueOptions},
            server_identity::{generate_server_identity, ServerIdentityGenerateOptions},
        },
        state::AppState,
    };

    struct SignedDescriptor {
        descriptor: ServerDescriptor,
        private_key_pkcs8: Vec<u8>,
    }

    #[derive(Debug)]
    struct CapturedForward {
        headers: HeaderMap,
        body: Vec<u8>,
    }

    fn signed_descriptor(server_id: &str, descriptor_id: &str, address: &str) -> SignedDescriptor {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = Utc::now().timestamp();
        let mut descriptor = ServerDescriptor {
            server_id: server_id.to_string(),
            server_public_key: public_key,
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
            supported_protocols: vec!["hexrelay-server-http".to_string()],
            rate_limits: Vec::new(),
            trust_labels: Vec::new(),
            revocation_pointer: None,
            signature: ServerSignature {
                algorithm: ServerSignatureAlgorithm::Ed25519,
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
            .with_local_server_identity(Some(LocalServerIdentity {
                descriptor: local.descriptor.clone(),
                private_key_pkcs8: local.private_key_pkcs8.clone(),
            }))
            .with_static_peer_registry(
                StaticPeerRegistry::try_new(vec![origin.descriptor.clone()]).expect("registry"),
            )
    }

    fn state_for_generated_identity(
        local: &LocalServerIdentity,
        registry: StaticPeerRegistry,
    ) -> AppState {
        let mut state = AppState::default()
            .with_local_server_identity(Some(local.clone()))
            .with_static_peer_registry(registry);
        state.server_id = local.descriptor.server_id.clone();
        state
    }

    fn generated_identity(
        server_id: &str,
        descriptor_id: &str,
        address: &str,
    ) -> LocalServerIdentity {
        let (_, identity) = generate_server_identity(
            &ServerIdentityGenerateOptions {
                server_id: server_id.to_string(),
                descriptor_id: Some(descriptor_id.to_string()),
                ttl_seconds: 300,
                max_ttl_seconds: 86_400,
                network_mode: NetworkMode::PrivatePeers,
                discovery_policy: DiscoveryPolicy::PrivateAllowlist,
                peering_policy: PeeringPolicy::InviteToken,
                relay_policy: RelayPolicy::None,
                dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
                storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
                addresses: vec![address.to_string()],
                supported_protocols: vec!["hexrelay-server-http".to_string()],
                trust_labels: Vec::new(),
                revocation_pointer: None,
            },
            Utc::now().timestamp() - 1,
        )
        .expect("generate server identity");

        identity
    }

    fn registry_from_signed_invite(
        issuer: &LocalServerIdentity,
        subject_server_id: &str,
    ) -> StaticPeerRegistry {
        let envelope = issue_peer_invite(
            issuer,
            &PeerInviteIssueOptions {
                invite_id: Some(format!(
                    "peer-invite-{}-to-{subject_server_id}",
                    issuer.descriptor.server_id
                )),
                subject_server_id: Some(subject_server_id.to_string()),
                allow_unbound: false,
                ttl_seconds: 300,
                max_ttl_seconds: 86_400,
                discovery_path: DiscoveryPath::PrivateAllowlist,
                max_uses: Some(1),
            },
            Utc::now().timestamp() - 1,
        )
        .expect("issue peer invite");

        StaticPeerRegistry::try_new(vec![envelope.issuer_descriptor]).expect("invite registry")
    }

    fn signed_forward_request(
        origin: &SignedDescriptor,
        destination_server_id: &str,
        nonce: &str,
    ) -> (HeaderMap, Vec<u8>) {
        let request = ServerForwardDmEnvelopeRequest {
            route_kind: SERVER_FORWARD_ROUTE_KIND_DIRECT.to_string(),
            origin_server_descriptor: origin.descriptor.clone(),
            destination_server_id: destination_server_id.to_string(),
            relay_server_id: None,
            message_id: "msg-server-forward-1".to_string(),
            thread_id: "thread-origin-1".to_string(),
            sender_identity_id: "usr-sender".to_string(),
            recipient_identity_id: "usr-recipient".to_string(),
            ciphertext: "enc:abcdefghijklmnopqrstuvwxyz".to_string(),
            source_device_id: Some("desktop-main".to_string()),
            accepted_at: Utc::now().to_rfc3339(),
            delivery_cursor: 1,
            target_device_ids: vec!["phone-main".to_string()],
        };
        let body = serde_json::to_vec(&request).expect("encode server forward request");
        let timestamp = Utc::now().timestamp().to_string();
        let key_pair =
            Ed25519KeyPair::from_pkcs8(&origin.private_key_pkcs8).expect("decode origin key");
        let signature = hex::encode(key_pair.sign(&forward_signature_payload(
            "POST",
            SERVER_FORWARD_PATH,
            &timestamp,
            nonce,
            &body,
        )));

        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_SERVER_ID,
            origin
                .descriptor
                .server_id
                .parse()
                .expect("server id header"),
        );
        headers.insert(
            HEADER_SERVER_DESCRIPTOR_ID,
            origin
                .descriptor
                .descriptor_id
                .parse()
                .expect("descriptor id header"),
        );
        headers.insert(
            HEADER_SIGNATURE_ALGORITHM,
            SERVER_FORWARD_SIGNATURE_ALGORITHM
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
    async fn forwards_direct_static_peer_envelope_with_server_signature() {
        let (base_url, capture_rx) = start_capture_server().await;
        let local = signed_descriptor("server-local", "descriptor-local", "https://local.example");
        let destination =
            signed_descriptor("server-destination", "descriptor-destination", &base_url);
        let registry =
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry");
        let route = match plan_dm_envelope_route(
            "server-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("server-destination"),
        )
        .expect("route should plan")
        {
            crate::domain::dm::routing::DmEnvelopeForwardingRoute::StaticPeer { route } => route,
            _ => panic!("expected static peer route"),
        };
        let state = AppState::default()
            .with_local_server_identity(Some(LocalServerIdentity {
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
                .get("x-hexrelay-server-id")
                .and_then(|value| value.to_str().ok()),
            Some("server-local")
        );
        assert_eq!(
            captured
                .headers
                .get("x-hexrelay-server-signature-algorithm")
                .and_then(|value| value.to_str().ok()),
            Some(SERVER_FORWARD_SIGNATURE_ALGORITHM)
        );

        let body: Value = serde_json::from_slice(&captured.body).expect("decode body");
        assert_eq!(body["route_kind"], "static_peer_direct");
        assert_eq!(body["destination_server_id"], "server-destination");
        assert_eq!(body["ciphertext"], "enc:abcdefghijklmnopqrstuvwxyz");

        let timestamp = captured
            .headers
            .get("x-hexrelay-server-signature-timestamp")
            .and_then(|value| value.to_str().ok())
            .expect("timestamp header");
        let nonce = captured
            .headers
            .get("x-hexrelay-server-signature-nonce")
            .and_then(|value| value.to_str().ok())
            .expect("nonce header");
        let signature = captured
            .headers
            .get("x-hexrelay-server-signature")
            .and_then(|value| value.to_str().ok())
            .expect("signature header");
        let public_key =
            hex::decode(local.descriptor.server_public_key).expect("decode public key");
        let signature = hex::decode(signature).expect("decode signature");
        UnparsedPublicKey::new(&ED25519, public_key)
            .verify(
                &forward_signature_payload(
                    "POST",
                    SERVER_FORWARD_PATH,
                    timestamp,
                    nonce,
                    &captured.body,
                ),
                &signature,
            )
            .expect("signature should verify");
    }

    #[tokio::test]
    async fn forwards_invite_backed_private_mesh_envelope_with_authenticated_server_request() {
        let (server_a_forward_url, capture_rx) = start_capture_server().await;
        let server_a = generated_identity("server-a", "descriptor-server-a", &server_a_forward_url);
        let server_b = generated_identity(
            "server-b",
            "descriptor-server-b",
            "https://server-b.example",
        );
        let server_b_registry = registry_from_signed_invite(&server_a, "server-b");
        let route = match plan_dm_envelope_route(
            "server-b",
            &server_b_registry,
            DmEnvelopeRouteRequest::static_destination("server-a"),
        )
        .expect("invite-backed route should plan")
        {
            crate::domain::dm::routing::DmEnvelopeForwardingRoute::StaticPeer { route } => route,
            _ => panic!("expected static peer route"),
        };
        assert_eq!(route.destination.descriptor.server_id, "server-a");
        assert_eq!(
            route.destination.descriptor.peering_policy,
            PeeringPolicy::InviteToken
        );

        let server_b_state = state_for_generated_identity(&server_b, server_b_registry);
        forward_dm_envelope_to_static_peer(
            &server_b_state,
            &route,
            ForwardDmEnvelopeInput {
                message_id: "msg-private-mesh-1",
                thread_id: "thread-private-mesh-1",
                sender_identity_id: "usr-sender",
                recipient_identity_id: "usr-recipient",
                ciphertext: "sealed:private-mesh-envelope-ciphertext",
                source_device_id: Some("desktop-main"),
                accepted_at: "2026-05-11T00:00:00Z",
                delivery_cursor: 42,
                target_device_ids: &["phone-main".to_string()],
            },
        )
        .await
        .expect("forward should succeed");

        let captured = capture_rx.await.expect("capture forwarded request");
        assert_eq!(
            captured
                .headers
                .get(HEADER_SERVER_ID)
                .and_then(|value| value.to_str().ok()),
            Some("server-b")
        );
        assert!(captured.headers.contains_key(HEADER_SIGNATURE));

        let server_a_registry = registry_from_signed_invite(&server_b, "server-a");
        let server_a_state = state_for_generated_identity(&server_a, server_a_registry);
        let authenticated =
            authenticate_server_forward_request(&server_a_state, &captured.headers, &captured.body)
                .expect("captured request should authenticate at destination server");
        assert_eq!(authenticated.origin_server_id, "server-b");
        assert_eq!(authenticated.request.destination_server_id, "server-a");
        assert_eq!(
            authenticated.request.ciphertext,
            "sealed:private-mesh-envelope-ciphertext"
        );

        let body: Value = serde_json::from_slice(&captured.body).expect("decode body");
        assert_eq!(body["route_kind"], SERVER_FORWARD_ROUTE_KIND_DIRECT);
        assert_eq!(body["origin_server_descriptor"]["server_id"], "server-b");
        assert_eq!(body["destination_server_id"], "server-a");
        assert_eq!(body["relay_server_id"], Value::Null);
        assert_eq!(
            body["ciphertext"],
            "sealed:private-mesh-envelope-ciphertext"
        );
        assert_eq!(body["target_device_ids"], serde_json::json!(["phone-main"]));
        assert!(body.get("plaintext").is_none());
        assert!(body.get("content").is_none());
    }

    #[tokio::test]
    async fn rejects_direct_static_peer_forward_without_local_server_identity() {
        let destination = signed_descriptor(
            "server-destination",
            "descriptor-destination",
            "https://server.example",
        );
        let registry =
            StaticPeerRegistry::try_new(vec![destination.descriptor.clone()]).expect("registry");
        let route = match plan_dm_envelope_route(
            "server-local",
            &registry,
            DmEnvelopeRouteRequest::static_destination("server-destination"),
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

        assert!(error.contains("local server identity"));
    }

    #[test]
    fn authenticates_static_peer_forward_request() {
        let local = signed_descriptor(
            "hexrelay-local-server",
            "descriptor-local",
            "https://local.example",
        );
        let origin = signed_descriptor(
            "server-origin",
            "descriptor-origin",
            "https://origin.example",
        );
        let state = state_with_forwarding_identity(&local, &origin);
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.server_id, "nonce-forward-auth-1");

        let authenticated = authenticate_server_forward_request(&state, &headers, &body)
            .expect("forward request should authenticate");

        assert_eq!(authenticated.origin_server_id, "server-origin");
        assert_eq!(
            authenticated.request.destination_server_id,
            local.descriptor.server_id
        );
        assert_eq!(
            authenticated.request.ciphertext,
            "enc:abcdefghijklmnopqrstuvwxyz"
        );
    }

    #[test]
    fn rejects_replayed_static_peer_forward_nonce() {
        let local = signed_descriptor(
            "hexrelay-local-server",
            "descriptor-local",
            "https://local.example",
        );
        let origin = signed_descriptor(
            "server-origin",
            "descriptor-origin",
            "https://origin.example",
        );
        let state = state_with_forwarding_identity(&local, &origin);
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.server_id, "nonce-forward-replay");

        authenticate_server_forward_request(&state, &headers, &body)
            .expect("first request should authenticate");
        let error = authenticate_server_forward_request(&state, &headers, &body)
            .expect_err("replayed nonce should fail");

        assert_eq!(error.status, ServerForwardRequestErrorStatus::Conflict);
        assert_eq!(error.code, "server_forward_replay");
    }

    #[test]
    fn rejects_forward_request_from_unconfigured_origin_server() {
        let local = signed_descriptor(
            "hexrelay-local-server",
            "descriptor-local",
            "https://local.example",
        );
        let origin = signed_descriptor(
            "server-origin",
            "descriptor-origin",
            "https://origin.example",
        );
        let state = AppState::default().with_local_server_identity(Some(LocalServerIdentity {
            descriptor: local.descriptor.clone(),
            private_key_pkcs8: local.private_key_pkcs8,
        }));
        let (headers, body) =
            signed_forward_request(&origin, &local.descriptor.server_id, "nonce-forward-denied");

        let error = authenticate_server_forward_request(&state, &headers, &body)
            .expect_err("unconfigured origin should fail");

        assert_eq!(error.status, ServerForwardRequestErrorStatus::Unauthorized);
        assert_eq!(error.code, "server_forward_peer_not_allowed");
    }

    #[test]
    fn rejects_forward_request_with_invalid_signature() {
        let local = signed_descriptor(
            "hexrelay-local-server",
            "descriptor-local",
            "https://local.example",
        );
        let origin = signed_descriptor(
            "server-origin",
            "descriptor-origin",
            "https://origin.example",
        );
        let state = state_with_forwarding_identity(&local, &origin);
        let (mut headers, body) = signed_forward_request(
            &origin,
            &local.descriptor.server_id,
            "nonce-forward-bad-signature",
        );
        headers.insert(HEADER_SIGNATURE, "00".parse().expect("signature header"));

        let error = authenticate_server_forward_request(&state, &headers, &body)
            .expect_err("invalid signature should fail");

        assert_eq!(error.status, ServerForwardRequestErrorStatus::Unauthorized);
        assert_eq!(error.code, "server_forward_signature_invalid");
    }

    async fn start_capture_server() -> (String, oneshot::Receiver<CapturedForward>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind capture server");
        let addr = listener.local_addr().expect("capture server address");
        let (tx, rx) = oneshot::channel::<CapturedForward>();
        let state = std::sync::Arc::new(tokio::sync::Mutex::new(Some(tx)));
        let app = Router::new()
            .route(SERVER_FORWARD_PATH, post(capture_forward))
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
