use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use crate::{
    errors::{bad_request, ApiResult},
    models::{
        AuthChallengeRequest, AuthVerifyRequest, FriendRequestCreate, FriendRequestListQuery,
        IdentityKeyRegistrationRequest, InviteCreateRequest, InviteRedeemRequest,
        SessionRevokeRequest,
    },
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

pub fn validate_invite_create_request(payload: &InviteCreateRequest) -> ApiResult<()> {
    if payload.mode != "one_time" && payload.mode != "multi_use" {
        return Err(bad_request(
            "invite_invalid",
            "mode must be one_time or multi_use",
        ));
    }

    if let Some(max_uses) = payload.max_uses {
        if max_uses == 0 {
            return Err(bad_request("invite_invalid", "max_uses must be at least 1"));
        }
    }

    Ok(())
}

pub fn validate_invite_redeem_request(payload: &InviteRedeemRequest) -> ApiResult<()> {
    if payload.token.trim().is_empty() {
        return Err(bad_request("invite_invalid", "token must not be empty"));
    }

    if payload.node_fingerprint.trim().is_empty() {
        return Err(bad_request(
            "fingerprint_mismatch",
            "node_fingerprint must not be empty",
        ));
    }

    Ok(())
}

pub fn validate_friend_request_create(payload: &FriendRequestCreate) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.requester_identity_id)
        || !is_valid_identity_id(&payload.target_identity_id)
    {
        return Err(bad_request(
            "identity_invalid",
            "requester_identity_id and target_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if payload.requester_identity_id == payload.target_identity_id {
        return Err(bad_request(
            "identity_invalid",
            "requester and target must be different identities",
        ));
    }

    Ok(())
}

pub fn validate_friend_request_list_query(query: &FriendRequestListQuery) -> ApiResult<()> {
    if !is_valid_identity_id(&query.identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if let Some(direction) = query.direction.as_ref() {
        if direction != "inbound" && direction != "outbound" {
            return Err(bad_request(
                "identity_invalid",
                "direction must be inbound or outbound when provided",
            ));
        }
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

fn is_valid_identity_id(value: &str) -> bool {
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
