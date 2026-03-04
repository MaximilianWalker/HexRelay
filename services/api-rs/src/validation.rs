use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use crate::{
    errors::{bad_request, ApiResult},
    models::IdentityKeyRegistrationRequest,
};

pub fn validate_identity_registration(payload: &IdentityKeyRegistrationRequest) -> ApiResult<()> {
    if payload.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "algorithm must be ed25519",
        ));
    }

    if payload.identity_id.trim().is_empty() {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must not be empty",
        ));
    }

    if !is_valid_public_key(&payload.public_key) {
        return Err(bad_request(
            "public_key_invalid",
            "public_key must be 32-byte ed25519 key in hex or base64",
        ));
    }

    Ok(())
}

fn is_valid_public_key(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    BASE64
        .decode(trimmed)
        .map(|decoded| decoded.len() == 32)
        .unwrap_or(false)
}
