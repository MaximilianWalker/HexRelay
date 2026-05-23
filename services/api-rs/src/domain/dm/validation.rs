use crate::{
    models::{
        DmFanoutCatchUpRequest, DmFanoutDispatchRequest, DmPolicyUpdate,
        DmProfileDeviceHeartbeatRequest,
    },
    shared::errors::{bad_request, ApiResult},
};

use crate::domain::auth::validation::is_valid_identity_id;

pub const DM_OFFLINE_DELIVERY_MODE: &str = "encrypted_envelope_catchup";
pub const DM_PROFILE_DEVICE_ID_MAX_LENGTH: usize = 64;
pub const DM_PROFILE_DEVICE_SECRET_MIN_LENGTH: usize = 16;
pub const DM_PROFILE_DEVICE_SECRET_MAX_LENGTH: usize = 128;
pub const DM_FANOUT_MESSAGE_ID_MAX_LENGTH: usize = 128;
pub const DM_FANOUT_CIPHERTEXT_MAX_LENGTH: usize = 8192;
pub const DM_FANOUT_CATCH_UP_DEFAULT_LIMIT: u32 = 50;
pub const DM_FANOUT_CATCH_UP_MAX_LIMIT: u32 = 100;

pub fn validate_dm_policy_update(payload: &DmPolicyUpdate) -> ApiResult<()> {
    let value = payload.inbound_policy.trim();
    if value.is_empty() {
        return Err(bad_request(
            "dm_policy_invalid",
            "inbound_policy must not be empty",
        ));
    }

    if !matches!(value, "friends_only" | "same_server" | "anyone") {
        return Err(bad_request(
            "dm_policy_invalid",
            "inbound_policy must be one of: friends_only, same_server, anyone",
        ));
    }

    Ok(())
}

pub fn validate_profile_device_heartbeat(
    payload: &DmProfileDeviceHeartbeatRequest,
) -> ApiResult<()> {
    validate_device_id(&payload.device_id, "profile_device_invalid")?;
    validate_device_secret(&payload.device_secret, "profile_device_invalid")?;

    Ok(())
}

pub fn validate_fanout_dispatch(payload: &DmFanoutDispatchRequest) -> ApiResult<()> {
    if !is_valid_identity_id(&payload.recipient_identity_id) {
        return Err(bad_request(
            "fanout_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    let message_id = payload.message_id.trim();
    if message_id.is_empty() || message_id.len() > DM_FANOUT_MESSAGE_ID_MAX_LENGTH {
        return Err(bad_request(
            "fanout_invalid",
            "message_id must be non-empty and <= 128 chars",
        ));
    }
    if message_id != payload.message_id {
        return Err(bad_request(
            "fanout_invalid",
            "message_id must not include leading or trailing whitespace",
        ));
    }

    let ciphertext = payload.ciphertext.trim();
    if ciphertext.is_empty() || ciphertext.len() > DM_FANOUT_CIPHERTEXT_MAX_LENGTH {
        return Err(bad_request(
            "fanout_invalid",
            "ciphertext must be non-empty and <= 8192 chars",
        ));
    }

    if let Some(source_device_id) = &payload.source_device_id {
        let normalized = source_device_id.trim();
        if normalized.is_empty() || normalized.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH {
            return Err(bad_request(
                "fanout_invalid",
                "source_device_id must be non-empty and <= 64 chars when provided",
            ));
        }
        if normalized != source_device_id {
            return Err(bad_request(
                "fanout_invalid",
                "source_device_id must not include leading or trailing whitespace",
            ));
        }
    }

    if let Some(destination_server_id) = &payload.destination_server_id {
        let normalized = destination_server_id.trim();
        if normalized.is_empty() || normalized.len() > 128 {
            return Err(bad_request(
                "fanout_invalid",
                "destination_server_id must be non-empty and <= 128 chars when provided",
            ));
        }
        if normalized != destination_server_id {
            return Err(bad_request(
                "fanout_invalid",
                "destination_server_id must not include leading or trailing whitespace",
            ));
        }
    }

    Ok(())
}

pub fn validate_fanout_catch_up(payload: &DmFanoutCatchUpRequest) -> ApiResult<(u32, Option<u64>)> {
    validate_device_id(&payload.device_id, "fanout_invalid")?;
    validate_device_secret(&payload.device_secret, "fanout_invalid")?;

    let limit = payload.limit.unwrap_or(DM_FANOUT_CATCH_UP_DEFAULT_LIMIT);
    if limit == 0 || limit > DM_FANOUT_CATCH_UP_MAX_LIMIT {
        return Err(bad_request(
            "fanout_invalid",
            "limit must be between 1 and 100 when provided",
        ));
    }

    let cursor = if let Some(cursor) = &payload.cursor {
        let normalized = cursor.trim();
        if normalized.is_empty() {
            return Err(bad_request(
                "fanout_invalid",
                "cursor must be a non-empty numeric string when provided",
            ));
        }
        if normalized != cursor {
            return Err(bad_request(
                "fanout_invalid",
                "cursor must not include leading or trailing whitespace",
            ));
        }

        Some(
            normalized
                .parse::<u64>()
                .map_err(|_| bad_request("fanout_invalid", "cursor must be a numeric string"))?,
        )
    } else {
        None
    };

    Ok((limit, cursor))
}

pub fn validate_device_id(value: &str, code: &'static str) -> ApiResult<()> {
    let device_id = value.trim();
    if device_id.is_empty() || device_id.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH {
        return Err(bad_request(
            code,
            "device_id must be non-empty and <= 64 chars",
        ));
    }
    if device_id != value {
        return Err(bad_request(
            code,
            "device_id must not include leading or trailing whitespace",
        ));
    }

    Ok(())
}

pub fn validate_device_secret(value: &str, code: &'static str) -> ApiResult<()> {
    let secret = value.trim();
    if secret.len() < DM_PROFILE_DEVICE_SECRET_MIN_LENGTH
        || secret.len() > DM_PROFILE_DEVICE_SECRET_MAX_LENGTH
    {
        return Err(bad_request(code, "device_secret must be 16-128 chars"));
    }
    if secret != value {
        return Err(bad_request(
            code,
            "device_secret must not include leading or trailing whitespace",
        ));
    }
    if !secret
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(bad_request(
            code,
            "device_secret must use only letters, numbers, _ or -",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::{
        DmFanoutCatchUpRequest, DmFanoutDispatchRequest, DmPolicyUpdate,
        DmProfileDeviceHeartbeatRequest,
    };

    use super::{
        validate_dm_policy_update, validate_fanout_catch_up, validate_fanout_dispatch,
        validate_profile_device_heartbeat,
    };

    #[test]
    fn validates_supported_dm_policy_values() {
        for value in ["friends_only", "same_server", "anyone"] {
            let payload = DmPolicyUpdate {
                inbound_policy: value.to_string(),
            };
            assert!(validate_dm_policy_update(&payload).is_ok());
        }
    }

    #[test]
    fn rejects_invalid_dm_policy_values() {
        let payload = DmPolicyUpdate {
            inbound_policy: "invalid".to_string(),
        };
        assert!(validate_dm_policy_update(&payload).is_err());

        let payload = DmPolicyUpdate {
            inbound_policy: "   ".to_string(),
        };
        assert!(validate_dm_policy_update(&payload).is_err());
    }

    #[test]
    fn validates_profile_device_heartbeat_payload() {
        let payload = DmProfileDeviceHeartbeatRequest {
            device_id: "desktop-1".to_string(),
            device_secret: "secret-desktop-1".to_string(),
            active: true,
        };
        assert!(validate_profile_device_heartbeat(&payload).is_ok());

        let invalid = DmProfileDeviceHeartbeatRequest {
            device_id: "  desktop-1".to_string(),
            device_secret: "secret-desktop-1".to_string(),
            active: true,
        };
        assert!(validate_profile_device_heartbeat(&invalid).is_err());
    }

    #[test]
    fn validates_fanout_dispatch_payload() {
        let payload = DmFanoutDispatchRequest {
            recipient_identity_id: "usr-jules-p".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: Some("desktop-1".to_string()),
            destination_server_id: Some("server-destination".to_string()),
        };
        assert!(validate_fanout_dispatch(&payload).is_ok());

        let invalid = DmFanoutDispatchRequest {
            recipient_identity_id: "invalid id".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: None,
            destination_server_id: None,
        };
        assert!(validate_fanout_dispatch(&invalid).is_err());

        let invalid = DmFanoutDispatchRequest {
            recipient_identity_id: "usr-jules-p".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: None,
            destination_server_id: Some(" server-destination".to_string()),
        };
        assert!(validate_fanout_dispatch(&invalid).is_err());
    }

    #[test]
    fn validates_fanout_catch_up_payload() {
        let payload = DmFanoutCatchUpRequest {
            device_id: "desktop-main".to_string(),
            device_secret: "secret-desktop-main".to_string(),
            cursor: Some("2".to_string()),
            limit: Some(25),
        };
        assert!(matches!(
            validate_fanout_catch_up(&payload),
            Ok((25, Some(2)))
        ));

        let invalid_device = DmFanoutCatchUpRequest {
            device_id: "  ".to_string(),
            device_secret: "secret-desktop-main".to_string(),
            cursor: None,
            limit: None,
        };
        assert!(validate_fanout_catch_up(&invalid_device).is_err());

        let invalid_limit = DmFanoutCatchUpRequest {
            device_id: "desktop-main".to_string(),
            device_secret: "secret-desktop-main".to_string(),
            cursor: None,
            limit: Some(0),
        };
        assert!(validate_fanout_catch_up(&invalid_limit).is_err());
    }
}
