use crate::{
    models::{ContactInviteRedeemRequest, InviteCreateRequest, InviteRedeemRequest},
    shared::errors::{bad_request, ApiResult},
};

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

pub fn validate_contact_invite_redeem_request(
    payload: &ContactInviteRedeemRequest,
) -> ApiResult<()> {
    if payload.token.trim().is_empty() {
        return Err(bad_request("invite_invalid", "token must not be empty"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_invite_create_request, validate_invite_redeem_request};
    use crate::models::{InviteCreateRequest, InviteRedeemRequest};

    #[test]
    fn rejects_invalid_invite_mode() {
        let payload = InviteCreateRequest {
            mode: "forever".to_string(),
            expires_at: None,
            max_uses: None,
        };

        let err = validate_invite_create_request(&payload).expect_err("invalid mode");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn rejects_empty_redeem_values() {
        let payload = InviteRedeemRequest {
            token: "   ".to_string(),
            node_fingerprint: " ".to_string(),
        };

        let err = validate_invite_redeem_request(&payload).expect_err("empty values must fail");
        assert_eq!(err.0, axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn accepts_valid_multi_use_payload() {
        let payload = InviteCreateRequest {
            mode: "multi_use".to_string(),
            expires_at: None,
            max_uses: Some(3),
        };

        assert!(validate_invite_create_request(&payload).is_ok());
    }
}
