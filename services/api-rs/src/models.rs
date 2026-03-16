use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthChallengeRequest {
    pub identity_id: String,
}

#[derive(Serialize)]
pub struct AuthChallengeResponse {
    pub challenge_id: String,
    pub nonce: String,
    pub expires_at: String,
}

#[derive(Clone)]
pub struct AuthChallengeRecord {
    pub identity_id: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct AuthVerifyRequest {
    pub identity_id: String,
    pub challenge_id: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct AuthVerifyResponse {
    pub session_id: String,
    pub expires_at: String,
}

#[derive(Serialize)]
pub struct SessionValidateResponse {
    pub session_id: String,
    pub identity_id: String,
    pub expires_at: String,
}

#[derive(Clone)]
pub struct SessionRecord {
    pub identity_id: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct SessionRevokeRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct InviteCreateRequest {
    pub mode: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<u32>,
}

#[derive(Serialize)]
pub struct InviteCreateResponse {
    pub invite_id: String,
    pub token: String,
    pub mode: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<u32>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct InviteRecord {
    pub invite_id: Option<String>,
    pub creator_identity_id: Option<String>,
    pub mode: String,
    pub node_fingerprint: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<u32>,
    pub uses: u32,
}

#[derive(Deserialize)]
pub struct InviteRedeemRequest {
    pub token: String,
    pub node_fingerprint: String,
}

#[derive(Serialize)]
pub struct InviteRedeemResponse {
    pub accepted: bool,
}

#[derive(Deserialize)]
pub struct ContactInviteRedeemRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ServerListQuery {
    pub search: Option<String>,
    pub favorites_only: Option<bool>,
    pub unread_only: Option<bool>,
    pub muted_only: Option<bool>,
}

#[derive(Clone, Serialize)]
pub struct ServerSummary {
    pub id: String,
    pub name: String,
    pub unread: u32,
    pub favorite: bool,
    pub muted: bool,
}

#[derive(Serialize)]
pub struct ServerListResponse {
    pub items: Vec<ServerSummary>,
}

#[derive(Deserialize)]
pub struct ContactListQuery {
    pub search: Option<String>,
    pub online_only: Option<bool>,
    pub unread_only: Option<bool>,
    pub favorites_only: Option<bool>,
}

#[derive(Clone, Serialize)]
pub struct ContactSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub unread: u32,
    pub favorite: bool,
    pub inbound_request: bool,
    pub pending_request: bool,
}

#[derive(Serialize)]
pub struct ContactListResponse {
    pub items: Vec<ContactSummary>,
}

#[derive(Deserialize)]
pub struct DmThreadListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub unread_only: Option<bool>,
}

#[derive(Clone, Serialize)]
pub struct DmPolicy {
    pub inbound_policy: String,
    pub offline_delivery_mode: String,
}

#[derive(Deserialize)]
pub struct DmPolicyUpdate {
    pub inbound_policy: String,
}

#[derive(Deserialize)]
pub struct DmPairingEnvelopeCreateRequest {
    pub endpoint_hints: Vec<String>,
    pub expires_in_seconds: Option<u32>,
}

#[derive(Serialize)]
pub struct DmPairingEnvelopeResponse {
    pub envelope: String,
    pub short_code: String,
    pub expires_at: String,
    pub pairing_nonce: String,
}

#[derive(Deserialize)]
pub struct DmPairingEnvelopeImportRequest {
    pub envelope: String,
}

#[derive(Deserialize)]
pub struct DmConnectivityPreflightRequest {
    pub peer_identity_id: Option<String>,
    pub pairing_envelope_present: Option<bool>,
    pub local_bind_allowed: Option<bool>,
    pub peer_reachable_hint: Option<bool>,
    pub same_server_context: Option<bool>,
}

#[derive(Serialize)]
pub struct DmConnectivityPreflightResponse {
    pub status: String,
    pub reason_code: String,
    pub transport_profile: String,
    pub remediation: Vec<String>,
}

#[derive(Deserialize)]
pub struct DmLanDiscoveryAnnounceRequest {
    pub endpoint_hints: Vec<String>,
}

#[derive(Serialize)]
pub struct DmLanDiscoveryAnnounceResponse {
    pub identity_id: String,
    pub endpoint_hints: Vec<String>,
    pub scope: String,
    pub last_seen_at: String,
}

#[derive(Clone, Serialize)]
pub struct DmLanPeerSummary {
    pub identity_id: String,
    pub endpoint_hints: Vec<String>,
    pub last_seen_at: String,
}

#[derive(Serialize)]
pub struct DmLanPeerListResponse {
    pub items: Vec<DmLanPeerSummary>,
}

#[derive(Clone)]
pub struct DmLanPresenceRecord {
    pub identity_id: String,
    pub endpoint_hints: Vec<String>,
    pub last_seen_epoch: i64,
}

#[derive(Clone, Deserialize)]
pub struct DmEndpointCardInput {
    pub endpoint_id: String,
    pub endpoint_hint: String,
    pub estimated_rtt_ms: Option<u32>,
    pub priority: Option<u8>,
    pub expires_in_seconds: Option<u32>,
}

#[derive(Deserialize)]
pub struct DmEndpointCardRegisterRequest {
    pub cards: Vec<DmEndpointCardInput>,
}

#[derive(Clone, Serialize)]
pub struct DmEndpointCard {
    pub endpoint_id: String,
    pub endpoint_hint: String,
    pub estimated_rtt_ms: u32,
    pub priority: u8,
    pub expires_at: String,
    pub revoked: bool,
}

#[derive(Serialize)]
pub struct DmEndpointCardRegisterResponse {
    pub identity_id: String,
    pub cards: Vec<DmEndpointCard>,
}

#[derive(Deserialize)]
pub struct DmEndpointCardRevokeRequest {
    pub endpoint_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct DmEndpointCardRevokeResponse {
    pub identity_id: String,
    pub revoked_endpoint_ids: Vec<String>,
    pub remaining_cards: Vec<DmEndpointCard>,
}

#[derive(Deserialize)]
pub struct DmParallelDialRequest {
    pub peer_identity_id: String,
    pub max_parallel_attempts: Option<u8>,
    pub unreachable_endpoint_ids: Option<Vec<String>>,
}

#[derive(Clone, Serialize)]
pub struct DmParallelDialAttempt {
    pub endpoint_id: String,
    pub endpoint_hint: String,
    pub estimated_rtt_ms: u32,
    pub status: String,
    pub cancellation_reason: Option<String>,
}

#[derive(Serialize)]
pub struct DmParallelDialResponse {
    pub status: String,
    pub reason_code: String,
    pub transport_profile: String,
    pub winner_endpoint_id: Option<String>,
    pub canceled_endpoint_ids: Vec<String>,
    pub attempts: Vec<DmParallelDialAttempt>,
    pub remediation: Vec<String>,
}

#[derive(Deserialize)]
pub struct DmProfileDeviceHeartbeatRequest {
    pub device_id: String,
    pub active: bool,
}

#[derive(Clone, Serialize)]
pub struct DmProfileDeviceSummary {
    pub device_id: String,
    pub active: bool,
    pub last_seen_at: String,
}

#[derive(Serialize)]
pub struct DmProfileDeviceHeartbeatResponse {
    pub identity_id: String,
    pub devices: Vec<DmProfileDeviceSummary>,
}

#[derive(Deserialize)]
pub struct DmFanoutDispatchRequest {
    pub recipient_identity_id: String,
    pub message_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
}

#[derive(Serialize)]
pub struct DmFanoutDispatchResponse {
    pub status: String,
    pub reason_code: String,
    pub transport_profile: String,
    pub fanout_count: u32,
    pub delivered_device_ids: Vec<String>,
    pub skipped_device_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct DmFanoutCatchUpRequest {
    pub device_id: String,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Clone, Serialize)]
pub struct DmFanoutCatchUpItem {
    pub cursor: String,
    pub message_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
}

#[derive(Serialize)]
pub struct DmFanoutCatchUpResponse {
    pub status: String,
    pub reason_code: String,
    pub transport_profile: String,
    pub device_id: String,
    pub replay_count: u32,
    pub next_cursor: String,
    pub deduped_message_ids: Vec<String>,
    pub items: Vec<DmFanoutCatchUpItem>,
}

#[derive(Clone)]
pub struct DmEndpointCardRecord {
    pub endpoint_id: String,
    pub endpoint_hint: String,
    pub estimated_rtt_ms: u32,
    pub priority: u8,
    pub expires_at_epoch: i64,
    pub revoked: bool,
}

#[derive(Clone)]
pub struct DmProfileDeviceRecord {
    pub device_id: String,
    pub active: bool,
    pub last_seen_epoch: i64,
}

#[derive(Clone)]
pub struct DmFanoutDeliveryRecord {
    pub cursor: u64,
    pub message_id: String,
    pub sender_identity_id: String,
    pub ciphertext: String,
    pub source_device_id: Option<String>,
    pub delivered_device_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct DmWanWizardRequest {
    pub preferred_port: Option<u16>,
    pub upnp_available: Option<bool>,
    pub nat_pmp_available: Option<bool>,
    pub auto_mapping_succeeds: Option<bool>,
    pub external_port_open: Option<bool>,
    pub network_profile: Option<String>,
}

#[derive(Serialize)]
pub struct DmWanWizardResponse {
    pub outcome: String,
    pub method: String,
    pub reason_code: String,
    pub checklist: Vec<String>,
}

#[derive(Serialize)]
pub struct DmPairingImportResponse {
    pub inviter_identity_id: String,
    pub endpoint_hints: Vec<String>,
    pub imported_at: String,
    pub expires_at: String,
}

#[derive(Clone, Serialize)]
pub struct DmThreadSummary {
    pub thread_id: String,
    pub kind: String,
    pub title: String,
    pub participant_ids: Vec<String>,
    pub unread: u32,
    pub last_read_seq: u64,
    pub last_message_seq: u64,
    pub last_message_preview: String,
    pub last_message_at: String,
}

#[derive(Serialize)]
pub struct DmThreadPage {
    pub items: Vec<DmThreadSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Deserialize)]
pub struct DmThreadMessageListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Clone, Serialize)]
pub struct DmMessageRecord {
    pub message_id: String,
    pub thread_id: String,
    pub author_id: String,
    pub seq: u64,
    pub ciphertext: String,
    pub created_at: String,
    pub edited_at: Option<String>,
}

#[derive(Serialize)]
pub struct DmMessagePage {
    pub items: Vec<DmMessageRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct FriendRequestRecord {
    pub request_id: String,
    pub requester_identity_id: String,
    pub target_identity_id: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct FriendRequestCreate {
    pub requester_identity_id: String,
    pub target_identity_id: String,
}

#[derive(Deserialize)]
pub struct FriendRequestListQuery {
    pub identity_id: String,
    pub direction: Option<String>,
}

#[derive(Serialize)]
pub struct FriendRequestPage {
    pub items: Vec<FriendRequestRecord>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub service: &'static str,
    pub status: &'static str,
}

#[derive(Clone)]
pub struct RegisteredIdentityKey {
    pub public_key: String,
    pub algorithm: String,
}

#[derive(Deserialize)]
pub struct IdentityKeyRegistrationRequest {
    pub identity_id: String,
    pub public_key: String,
    pub algorithm: String,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
}
