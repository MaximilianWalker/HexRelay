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
    pub token: String,
    pub mode: String,
    pub expires_at: Option<String>,
    pub max_uses: Option<u32>,
}

#[derive(Clone)]
pub struct InviteRecord {
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
