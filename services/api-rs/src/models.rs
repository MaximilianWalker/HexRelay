use serde::{Deserialize, Serialize};

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
