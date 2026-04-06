pub struct AuthVerifyRequest {
    pub challenge_id: String,
    pub identity_id: String,
    pub signature: String,
}

pub struct AuthVerifyResponse {}
