use crate::{
    models::{InviteCreateRequest, InviteRedeemRequest},
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
