pub struct AuthVerifyRequest {
    pub identity_id: String,
}

pub struct AuthVerifyResponse {}

pub struct SessionRevokeRequest {
    pub session_id: String,
}
