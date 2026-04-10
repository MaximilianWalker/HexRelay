use serde::{Deserialize, Serialize};

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
pub struct ApiError {
    pub code: &'static str,
    pub message: &'static str,
}
