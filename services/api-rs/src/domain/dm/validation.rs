use chrono::DateTime;

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

#[derive(Debug, Clone, Copy)]
pub struct DmEnvelopeAckValidationInput<'a> {
    pub envelope_id: &'a str,
    pub message_id: &'a str,
    pub thread_id: &'a str,
    pub recipient_identity_id: &'a str,
    pub device_id: &'a str,
    pub ack_status: &'a str,
    pub received_at: &'a str,
    pub delivery_cursor: &'a str,
}

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

pub fn validate_profile_device_secret_input(device_id: &str, device_secret: &str) -> ApiResult<()> {
    validate_device_id(device_id, "profile_device_invalid")?;
    validate_device_secret(device_secret, "profile_device_invalid")
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

    if let Some(destination_node_id) = &payload.destination_node_id {
        let normalized = destination_node_id.trim();
        if normalized.is_empty() || normalized.len() > 128 {
            return Err(bad_request(
                "fanout_invalid",
                "destination_node_id must be non-empty and <= 128 chars when provided",
            ));
        }
        if normalized != destination_node_id {
            return Err(bad_request(
                "fanout_invalid",
                "destination_node_id must not include leading or trailing whitespace",
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

pub fn validate_dm_envelope_ack_internal(
    input: DmEnvelopeAckValidationInput<'_>,
) -> ApiResult<u64> {
    if input.envelope_id.trim().is_empty() || input.envelope_id.len() > 128 {
        return Err(bad_request(
            "dm_ack_invalid",
            "envelope_id must be non-empty and <= 128 chars",
        ));
    }
    if trimmed_invalid(input.envelope_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "envelope_id must not include leading or trailing whitespace",
        ));
    }

    for (field, value) in [
        ("message_id", input.message_id),
        ("thread_id", input.thread_id),
    ] {
        if value.trim().is_empty() || value.len() > 128 {
            return Err(bad_request(
                "dm_ack_invalid",
                match field {
                    "message_id" => "message_id must be non-empty and <= 128 chars",
                    _ => "thread_id must be non-empty and <= 128 chars",
                },
            ));
        }
        if trimmed_invalid(value) {
            return Err(bad_request(
                "dm_ack_invalid",
                match field {
                    "message_id" => "message_id must not include leading or trailing whitespace",
                    _ => "thread_id must not include leading or trailing whitespace",
                },
            ));
        }
    }

    if !is_valid_identity_id(input.recipient_identity_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "recipient_identity_id must be 3-64 chars using letters, numbers, _ or -",
        ));
    }

    let device_id = input.device_id.trim();
    if device_id.is_empty() || device_id.len() > DM_PROFILE_DEVICE_ID_MAX_LENGTH {
        return Err(bad_request(
            "dm_ack_invalid",
            "device_id must be non-empty and <= 64 chars",
        ));
    }
    if trimmed_invalid(input.device_id) {
        return Err(bad_request(
            "dm_ack_invalid",
            "device_id must not include leading or trailing whitespace",
        ));
    }

    if input.ack_status != "received" {
        return Err(bad_request("dm_ack_invalid", "ack_status must be received"));
    }

    DateTime::parse_from_rfc3339(input.received_at.trim())
        .map_err(|_| bad_request("dm_ack_invalid", "received_at must be an RFC3339 date-time"))?;
    if trimmed_invalid(input.received_at) {
        return Err(bad_request(
            "dm_ack_invalid",
            "received_at must not include leading or trailing whitespace",
        ));
    }

    let cursor = input
        .delivery_cursor
        .trim()
        .parse::<u64>()
        .map_err(|_| bad_request("dm_ack_invalid", "delivery_cursor must be numeric"))?;
    if cursor == 0 {
        return Err(bad_request(
            "dm_ack_invalid",
            "delivery_cursor must be greater than zero",
        ));
    }
    if trimmed_invalid(input.delivery_cursor) {
        return Err(bad_request(
            "dm_ack_invalid",
            "delivery_cursor must not include leading or trailing whitespace",
        ));
    }

    Ok(cursor)
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

fn trimmed_invalid(value: &str) -> bool {
    value.trim() != value
}

#[cfg(test)]
mod tests {
    use crate::models::{
        DmFanoutCatchUpRequest, DmFanoutDispatchRequest, DmPolicyUpdate,
        DmProfileDeviceHeartbeatRequest,
    };

    use super::{
        validate_dm_envelope_ack_internal, validate_dm_policy_update, validate_fanout_catch_up,
        validate_fanout_dispatch, validate_profile_device_heartbeat,
        validate_profile_device_secret_input, DmEnvelopeAckValidationInput,
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
    fn validates_profile_device_secret_input() {
        assert!(validate_profile_device_secret_input("desktop-1", "secret-desktop-1").is_ok());
        assert!(validate_profile_device_secret_input(" desktop-1", "secret-desktop-1").is_err());
    }

    #[test]
    fn validates_fanout_dispatch_payload() {
        let payload = DmFanoutDispatchRequest {
            recipient_identity_id: "usr-jules-p".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: Some("desktop-1".to_string()),
            destination_node_id: Some("node-destination".to_string()),
        };
        assert!(validate_fanout_dispatch(&payload).is_ok());

        let invalid = DmFanoutDispatchRequest {
            recipient_identity_id: "invalid id".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: None,
            destination_node_id: None,
        };
        assert!(validate_fanout_dispatch(&invalid).is_err());

        let invalid = DmFanoutDispatchRequest {
            recipient_identity_id: "usr-jules-p".to_string(),
            message_id: "msg-9001".to_string(),
            ciphertext: "enc:abc123".to_string(),
            source_device_id: None,
            destination_node_id: Some(" node-destination".to_string()),
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

    #[test]
    fn validates_internal_ack_payload() {
        let cursor = match validate_dm_envelope_ack_internal(DmEnvelopeAckValidationInput {
            envelope_id: "env-1",
            message_id: "msg-1",
            thread_id: "thread-1",
            recipient_identity_id: "alice",
            device_id: "desktop-1",
            ack_status: "received",
            received_at: "2026-05-19T00:00:00Z",
            delivery_cursor: "42",
        }) {
            Ok(cursor) => cursor,
            Err(_) => panic!("valid internal ack"),
        };

        assert_eq!(cursor, 42);
    }

    #[test]
    fn rejects_internal_ack_whitespace_and_zero_cursor() {
        assert!(
            validate_dm_envelope_ack_internal(DmEnvelopeAckValidationInput {
                envelope_id: " env-1",
                message_id: "msg-1",
                thread_id: "thread-1",
                recipient_identity_id: "alice",
                device_id: "desktop-1",
                ack_status: "received",
                received_at: "2026-05-19T00:00:00Z",
                delivery_cursor: "42",
            })
            .is_err()
        );

        assert!(
            validate_dm_envelope_ack_internal(DmEnvelopeAckValidationInput {
                envelope_id: "env-1",
                message_id: "msg-1",
                thread_id: "thread-1",
                recipient_identity_id: "alice",
                device_id: "desktop-1",
                ack_status: "received",
                received_at: "2026-05-19T00:00:00Z",
                delivery_cursor: "0",
            })
            .is_err()
        );
    }
}
