use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use crate::{
    models::{
        AuthChallengeRequest, AuthVerifyRequest, IdentityKeyRegistrationRequest,
        SessionRevokeRequest,
    },
    shared::errors::{bad_request, ApiResult},
};

pub fn validate_identity_registration(payload: &IdentityKeyRegistrationRequest) -> ApiResult<()> {
    if payload.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "algorithm must be ed25519",
        ));
    }

    if !is_valid_identity_id(&payload.identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
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

pub fn validate_auth_challenge_request(payload: &AuthChallengeRequest) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    Ok(())
}

pub fn validate_auth_verify_request(payload: &AuthVerifyRequest) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if payload.challenge_id.trim().is_empty() {
        return Err(bad_request(
            "nonce_invalid",
            "challenge_id must not be empty",
        ));
    }

    if payload.signature.trim().is_empty() {
        return Err(bad_request(
            "signature_invalid",
            "signature must not be empty",
        ));
    }

    Ok(())
}

pub fn validate_session_revoke_request(payload: &SessionRevokeRequest) -> ApiResult<()> {
    if payload.session_id.trim().is_empty() {
        return Err(bad_request(
            "session_invalid",
            "session_id must not be empty",
        ));
    }

    Ok(())
}

pub fn decode_32_bytes(value: &str) -> Option<[u8; 32]> {
    decode_fixed_len(value, 32).and_then(|bytes| bytes.try_into().ok())
}

pub fn decode_64_bytes(value: &str) -> Option<[u8; 64]> {
    decode_fixed_len(value, 64).and_then(|bytes| bytes.try_into().ok())
}

fn is_valid_public_key(value: &str) -> bool {
    decode_32_bytes(value).is_some()
}

fn decode_fixed_len(value: &str, len: usize) -> Option<Vec<u8>> {
    let trimmed = value.trim();

    if trimmed.len() == len * 2 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return hex::decode(trimmed).ok();
    }

    BASE64
        .decode(trimmed)
        .ok()
        .filter(|decoded| decoded.len() == len)
}

pub fn is_valid_identity_id(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed != value {
        return false;
    }

    let len = trimmed.len();
    if !(3..=64).contains(&len) {
        return false;
    }

    trimmed
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

#[cfg(test)]
mod tests {
    use super::{
        decode_32_bytes, decode_64_bytes, is_valid_identity_id, validate_auth_challenge_request,
        validate_auth_verify_request, validate_identity_registration,
        validate_session_revoke_request,
    };
    use crate::models::{
        AuthChallengeRequest, AuthVerifyRequest, IdentityKeyRegistrationRequest,
        SessionRevokeRequest,
    };
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

    #[test]
    fn validates_identity_registration_with_hex_and_base64_keys() {
        let hex_key = "11".repeat(32);
        let payload = IdentityKeyRegistrationRequest {
            identity_id: "user_01".to_string(),
            public_key: hex_key,
            algorithm: "ed25519".to_string(),
        };
        assert!(validate_identity_registration(&payload).is_ok());

        let base64_key = BASE64.encode(vec![0x22; 32]);
        let base64_payload = IdentityKeyRegistrationRequest {
            identity_id: "user-02".to_string(),
            public_key: base64_key,
            algorithm: "ed25519".to_string(),
        };
        assert!(validate_identity_registration(&base64_payload).is_ok());
    }

    #[test]
    fn rejects_identity_registration_with_invalid_fields() {
        let invalid_algorithm = IdentityKeyRegistrationRequest {
            identity_id: "user-1".to_string(),
            public_key: "11".repeat(32),
            algorithm: "rsa".to_string(),
        };
        assert!(validate_identity_registration(&invalid_algorithm).is_err());

        let invalid_identity = IdentityKeyRegistrationRequest {
            identity_id: " bad ".to_string(),
            public_key: "11".repeat(32),
            algorithm: "ed25519".to_string(),
        };
        assert!(validate_identity_registration(&invalid_identity).is_err());

        let invalid_key = IdentityKeyRegistrationRequest {
            identity_id: "user-1".to_string(),
            public_key: "abcd".to_string(),
            algorithm: "ed25519".to_string(),
        };
        assert!(validate_identity_registration(&invalid_key).is_err());
    }

    #[test]
    fn validates_auth_requests_and_rejects_empty_values() {
        let challenge_ok = AuthChallengeRequest {
            identity_id: "valid_user".to_string(),
        };
        assert!(validate_auth_challenge_request(&challenge_ok).is_ok());

        let verify_ok = AuthVerifyRequest {
            identity_id: "valid_user".to_string(),
            challenge_id: "nonce-123".to_string(),
            signature: "signature-123".to_string(),
        };
        assert!(validate_auth_verify_request(&verify_ok).is_ok());

        let verify_missing_challenge = AuthVerifyRequest {
            challenge_id: "  ".to_string(),
            ..verify_ok
        };
        assert!(validate_auth_verify_request(&verify_missing_challenge).is_err());

        let verify_missing_signature = AuthVerifyRequest {
            signature: "  ".to_string(),
            ..verify_missing_challenge
        };
        assert!(validate_auth_verify_request(&verify_missing_signature).is_err());
    }

    #[test]
    fn validates_session_revoke_request() {
        let valid = SessionRevokeRequest {
            session_id: "sess-123".to_string(),
        };
        assert!(validate_session_revoke_request(&valid).is_ok());

        let invalid = SessionRevokeRequest {
            session_id: "  ".to_string(),
        };
        assert!(validate_session_revoke_request(&invalid).is_err());
    }

    #[test]
    fn decodes_fixed_length_hex_and_base64_payloads() {
        let key32_hex = "ab".repeat(32);
        assert!(decode_32_bytes(&key32_hex).is_some());

        let key64_hex = "cd".repeat(64);
        assert!(decode_64_bytes(&key64_hex).is_some());

        let key32_b64 = BASE64.encode(vec![0x33; 32]);
        assert!(decode_32_bytes(&key32_b64).is_some());

        let short_b64 = BASE64.encode(vec![0x44; 31]);
        assert!(decode_32_bytes(&short_b64).is_none());
    }

    #[test]
    fn validates_identity_id_rules() {
        assert!(is_valid_identity_id("abc"));
        assert!(is_valid_identity_id("user_name-123"));
        assert!(!is_valid_identity_id("ab"));
        assert!(!is_valid_identity_id("name with spaces"));
        assert!(!is_valid_identity_id(" leading"));
    }
}
