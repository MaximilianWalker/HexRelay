use crate::{
    models::{DmPairingEnvelopeCreateRequest, DmPairingEnvelopeImportRequest, DmPolicyUpdate},
    shared::errors::{bad_request, ApiResult},
};

pub const DM_OFFLINE_DELIVERY_MODE: &str = "best_effort_online";
pub const DM_PAIRING_ENVELOPE_VERSION: u32 = 1;
pub const DM_PAIRING_DEFAULT_EXPIRY_SECONDS: u32 = 600;
pub const DM_PAIRING_MAX_EXPIRY_SECONDS: u32 = 3600;
pub const DM_PAIRING_MAX_ENDPOINT_HINTS: usize = 8;

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

pub fn validate_pairing_envelope_create(
    payload: &DmPairingEnvelopeCreateRequest,
) -> ApiResult<u32> {
    if payload.endpoint_hints.len() > DM_PAIRING_MAX_ENDPOINT_HINTS {
        return Err(bad_request(
            "pairing_invalid",
            "too many endpoint hints in pairing request",
        ));
    }

    for hint in &payload.endpoint_hints {
        let value = hint.trim();
        if value.is_empty() || value.len() > 200 {
            return Err(bad_request(
                "pairing_invalid",
                "endpoint hints must be non-empty and <= 200 chars",
            ));
        }
    }

    let expires_in_seconds = payload
        .expires_in_seconds
        .unwrap_or(DM_PAIRING_DEFAULT_EXPIRY_SECONDS);
    if expires_in_seconds == 0 || expires_in_seconds > DM_PAIRING_MAX_EXPIRY_SECONDS {
        return Err(bad_request(
            "pairing_invalid",
            "expires_in_seconds must be between 1 and 3600",
        ));
    }

    Ok(expires_in_seconds)
}

pub fn validate_pairing_envelope_import(payload: &DmPairingEnvelopeImportRequest) -> ApiResult<()> {
    let envelope = payload.envelope.trim();
    if envelope.is_empty() || envelope.len() > 8192 {
        return Err(bad_request(
            "pairing_invalid",
            "pairing envelope must be non-empty and <= 8192 chars",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::{
        DmPairingEnvelopeCreateRequest, DmPairingEnvelopeImportRequest, DmPolicyUpdate,
    };

    use super::{
        validate_dm_policy_update, validate_pairing_envelope_create,
        validate_pairing_envelope_import,
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
    fn validates_pairing_envelope_create_defaults_and_limits() {
        let payload = DmPairingEnvelopeCreateRequest {
            endpoint_hints: vec!["tcp://127.0.0.1:4040".to_string()],
            expires_in_seconds: None,
        };
        let expiry = match validate_pairing_envelope_create(&payload) {
            Ok(value) => value,
            Err(_) => panic!("valid defaults"),
        };
        assert_eq!(expiry, 600);

        let invalid = DmPairingEnvelopeCreateRequest {
            endpoint_hints: vec![],
            expires_in_seconds: Some(0),
        };
        assert!(validate_pairing_envelope_create(&invalid).is_err());
    }

    #[test]
    fn validates_pairing_envelope_import_payload() {
        let payload = DmPairingEnvelopeImportRequest {
            envelope: "abc123".to_string(),
        };
        assert!(validate_pairing_envelope_import(&payload).is_ok());

        let invalid = DmPairingEnvelopeImportRequest {
            envelope: "  ".to_string(),
        };
        assert!(validate_pairing_envelope_import(&invalid).is_err());
    }
}
