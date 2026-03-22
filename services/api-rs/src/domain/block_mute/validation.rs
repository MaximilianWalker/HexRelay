use crate::{
    domain::auth::validation::is_valid_identity_id,
    models::{BlockUserRequest, MuteUserRequest},
    shared::errors::{bad_request, ApiResult},
};

pub fn validate_block_request(
    payload: &BlockUserRequest,
    actor_identity_id: &str,
) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.target_identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "target_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if payload.target_identity_id == actor_identity_id {
        return Err(bad_request("identity_invalid", "cannot block yourself"));
    }

    Ok(())
}

pub fn validate_mute_request(payload: &MuteUserRequest, actor_identity_id: &str) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.target_identity_id) {
        return Err(bad_request(
            "identity_invalid",
            "target_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    if payload.target_identity_id == actor_identity_id {
        return Err(bad_request("identity_invalid", "cannot mute yourself"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_block_request, validate_mute_request};
    use crate::models::{BlockUserRequest, MuteUserRequest};

    #[test]
    fn rejects_block_self() {
        let payload = BlockUserRequest {
            target_identity_id: "usr-a".to_string(),
        };
        let err = validate_block_request(&payload, "usr-a").expect_err("block self must fail");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_mute_self() {
        let payload = MuteUserRequest {
            target_identity_id: "usr-a".to_string(),
        };
        let err = validate_mute_request(&payload, "usr-a").expect_err("mute self must fail");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn accepts_valid_block_request() {
        let payload = BlockUserRequest {
            target_identity_id: "usr-b".to_string(),
        };
        assert!(validate_block_request(&payload, "usr-a").is_ok());
    }

    #[test]
    fn accepts_valid_mute_request() {
        let payload = MuteUserRequest {
            target_identity_id: "usr-b".to_string(),
        };
        assert!(validate_mute_request(&payload, "usr-a").is_ok());
    }

    #[test]
    fn rejects_invalid_identity_in_block() {
        let payload = BlockUserRequest {
            target_identity_id: "x".to_string(), // too short
        };
        let err = validate_block_request(&payload, "usr-a").expect_err("invalid identity");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_invalid_identity_in_mute() {
        let payload = MuteUserRequest {
            target_identity_id: "x".to_string(), // too short
        };
        let err = validate_mute_request(&payload, "usr-a").expect_err("invalid identity");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }
}
