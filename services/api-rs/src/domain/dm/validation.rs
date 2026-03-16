use crate::{
    models::{
        DmConnectivityPreflightRequest, DmEndpointCardRegisterRequest, DmEndpointCardRevokeRequest,
        DmLanDiscoveryAnnounceRequest, DmPairingEnvelopeCreateRequest,
        DmPairingEnvelopeImportRequest, DmParallelDialRequest, DmPolicyUpdate, DmWanWizardRequest,
    },
    shared::errors::{bad_request, ApiResult},
};

pub const DM_OFFLINE_DELIVERY_MODE: &str = "best_effort_online";
pub const DM_PAIRING_ENVELOPE_VERSION: u32 = 1;
pub const DM_PAIRING_DEFAULT_EXPIRY_SECONDS: u32 = 600;
pub const DM_PAIRING_MAX_EXPIRY_SECONDS: u32 = 3600;
pub const DM_PAIRING_MAX_ENDPOINT_HINTS: usize = 8;
pub const DM_LAN_DISCOVERY_MAX_ENDPOINT_HINTS: usize = 8;
pub const DM_ENDPOINT_CARD_MAX_ITEMS: usize = 8;
pub const DM_ENDPOINT_CARD_DEFAULT_EXPIRY_SECONDS: u32 = 900;
pub const DM_ENDPOINT_CARD_MAX_EXPIRY_SECONDS: u32 = 3600;
pub const DM_ENDPOINT_CARD_DEFAULT_RTT_MS: u32 = 150;
pub const DM_ENDPOINT_CARD_MAX_RTT_MS: u32 = 5000;
pub const DM_PARALLEL_DIAL_DEFAULT_ATTEMPTS: u8 = 3;
pub const DM_PARALLEL_DIAL_MAX_ATTEMPTS: u8 = 8;

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

pub fn validate_connectivity_preflight(payload: &DmConnectivityPreflightRequest) -> ApiResult<()> {
    if let Some(peer_identity_id) = &payload.peer_identity_id {
        if peer_identity_id.trim().is_empty() {
            return Err(bad_request(
                "preflight_invalid",
                "peer_identity_id must not be empty when provided",
            ));
        }
    }

    Ok(())
}

pub fn validate_lan_discovery_announce(payload: &DmLanDiscoveryAnnounceRequest) -> ApiResult<()> {
    if payload.endpoint_hints.is_empty() {
        return Err(bad_request(
            "lan_discovery_invalid",
            "endpoint_hints must include at least one LAN endpoint",
        ));
    }
    if payload.endpoint_hints.len() > DM_LAN_DISCOVERY_MAX_ENDPOINT_HINTS {
        return Err(bad_request(
            "lan_discovery_invalid",
            "too many LAN endpoint hints",
        ));
    }

    for hint in &payload.endpoint_hints {
        let value = hint.trim();
        if value.is_empty() || value.len() > 200 {
            return Err(bad_request(
                "lan_discovery_invalid",
                "LAN endpoint hints must be non-empty and <= 200 chars",
            ));
        }
    }

    Ok(())
}

pub fn validate_wan_wizard_request(payload: &DmWanWizardRequest) -> ApiResult<()> {
    if let Some(profile) = &payload.network_profile {
        if !matches!(
            profile.as_str(),
            "home_nat" | "symmetric_nat" | "carrier_nat" | "enterprise_restricted"
        ) {
            return Err(bad_request(
                "wan_wizard_invalid",
                "network_profile must be one of: home_nat, symmetric_nat, carrier_nat, enterprise_restricted",
            ));
        }
    }

    if let Some(port) = payload.preferred_port {
        if port == 0 {
            return Err(bad_request(
                "wan_wizard_invalid",
                "preferred_port must be a valid non-zero port",
            ));
        }
    }

    Ok(())
}

pub fn validate_endpoint_card_register(payload: &DmEndpointCardRegisterRequest) -> ApiResult<()> {
    if payload.cards.is_empty() || payload.cards.len() > DM_ENDPOINT_CARD_MAX_ITEMS {
        return Err(bad_request(
            "endpoint_cards_invalid",
            "cards must include between 1 and 8 endpoint cards",
        ));
    }

    for card in &payload.cards {
        if card.endpoint_id.trim().is_empty() || card.endpoint_id.len() > 64 {
            return Err(bad_request(
                "endpoint_cards_invalid",
                "endpoint_id must be non-empty and <= 64 chars",
            ));
        }
        if card.endpoint_hint.trim().is_empty() || card.endpoint_hint.len() > 200 {
            return Err(bad_request(
                "endpoint_cards_invalid",
                "endpoint_hint must be non-empty and <= 200 chars",
            ));
        }

        if let Some(expires_in_seconds) = card.expires_in_seconds {
            if expires_in_seconds == 0 || expires_in_seconds > DM_ENDPOINT_CARD_MAX_EXPIRY_SECONDS {
                return Err(bad_request(
                    "endpoint_cards_invalid",
                    "expires_in_seconds must be between 1 and 3600",
                ));
            }
        }

        if let Some(estimated_rtt_ms) = card.estimated_rtt_ms {
            if estimated_rtt_ms == 0 || estimated_rtt_ms > DM_ENDPOINT_CARD_MAX_RTT_MS {
                return Err(bad_request(
                    "endpoint_cards_invalid",
                    "estimated_rtt_ms must be between 1 and 5000",
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_endpoint_card_revoke(payload: &DmEndpointCardRevokeRequest) -> ApiResult<()> {
    if payload.endpoint_ids.is_empty() || payload.endpoint_ids.len() > DM_ENDPOINT_CARD_MAX_ITEMS {
        return Err(bad_request(
            "endpoint_cards_invalid",
            "endpoint_ids must include between 1 and 8 ids",
        ));
    }

    for endpoint_id in &payload.endpoint_ids {
        if endpoint_id.trim().is_empty() || endpoint_id.len() > 64 {
            return Err(bad_request(
                "endpoint_cards_invalid",
                "endpoint_id must be non-empty and <= 64 chars",
            ));
        }
    }

    Ok(())
}

pub fn validate_parallel_dial_request(payload: &DmParallelDialRequest) -> ApiResult<()> {
    if payload.peer_identity_id.trim().is_empty() {
        return Err(bad_request(
            "parallel_dial_invalid",
            "peer_identity_id must not be empty",
        ));
    }

    if let Some(max_parallel_attempts) = payload.max_parallel_attempts {
        if max_parallel_attempts == 0 || max_parallel_attempts > DM_PARALLEL_DIAL_MAX_ATTEMPTS {
            return Err(bad_request(
                "parallel_dial_invalid",
                "max_parallel_attempts must be between 1 and 8",
            ));
        }
    }

    if let Some(unreachable_endpoint_ids) = &payload.unreachable_endpoint_ids {
        if unreachable_endpoint_ids.len() > DM_ENDPOINT_CARD_MAX_ITEMS {
            return Err(bad_request(
                "parallel_dial_invalid",
                "unreachable_endpoint_ids must not exceed 8 ids",
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::{
        DmConnectivityPreflightRequest, DmEndpointCardRegisterRequest, DmEndpointCardRevokeRequest,
        DmLanDiscoveryAnnounceRequest, DmPairingEnvelopeCreateRequest,
        DmPairingEnvelopeImportRequest, DmParallelDialRequest, DmPolicyUpdate, DmWanWizardRequest,
    };

    use super::{
        validate_connectivity_preflight, validate_dm_policy_update,
        validate_endpoint_card_register, validate_endpoint_card_revoke,
        validate_lan_discovery_announce, validate_pairing_envelope_create,
        validate_pairing_envelope_import, validate_parallel_dial_request,
        validate_wan_wizard_request,
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

    #[test]
    fn validates_connectivity_preflight_payload() {
        let payload = DmConnectivityPreflightRequest {
            peer_identity_id: Some("usr-a".to_string()),
            pairing_envelope_present: Some(true),
            local_bind_allowed: Some(true),
            peer_reachable_hint: Some(true),
            same_server_context: Some(false),
        };
        assert!(validate_connectivity_preflight(&payload).is_ok());

        let invalid = DmConnectivityPreflightRequest {
            peer_identity_id: Some("   ".to_string()),
            pairing_envelope_present: None,
            local_bind_allowed: None,
            peer_reachable_hint: None,
            same_server_context: None,
        };
        assert!(validate_connectivity_preflight(&invalid).is_err());
    }

    #[test]
    fn validates_lan_discovery_announce_payload() {
        let payload = DmLanDiscoveryAnnounceRequest {
            endpoint_hints: vec!["udp://192.168.1.11:4040".to_string()],
        };
        assert!(validate_lan_discovery_announce(&payload).is_ok());

        let invalid_empty = DmLanDiscoveryAnnounceRequest {
            endpoint_hints: vec![],
        };
        assert!(validate_lan_discovery_announce(&invalid_empty).is_err());

        let invalid_blank = DmLanDiscoveryAnnounceRequest {
            endpoint_hints: vec!["   ".to_string()],
        };
        assert!(validate_lan_discovery_announce(&invalid_blank).is_err());
    }

    #[test]
    fn validates_wan_wizard_payload() {
        let payload = DmWanWizardRequest {
            preferred_port: Some(4040),
            upnp_available: Some(true),
            nat_pmp_available: Some(false),
            auto_mapping_succeeds: Some(true),
            external_port_open: Some(true),
            network_profile: Some("home_nat".to_string()),
        };
        assert!(validate_wan_wizard_request(&payload).is_ok());

        let invalid = DmWanWizardRequest {
            preferred_port: Some(4040),
            upnp_available: None,
            nat_pmp_available: None,
            auto_mapping_succeeds: None,
            external_port_open: None,
            network_profile: Some("invalid".to_string()),
        };
        assert!(validate_wan_wizard_request(&invalid).is_err());
    }

    #[test]
    fn validates_endpoint_card_register_payload() {
        let payload = DmEndpointCardRegisterRequest {
            cards: vec![crate::models::DmEndpointCardInput {
                endpoint_id: "lan-1".to_string(),
                endpoint_hint: "udp://192.168.1.10:4040".to_string(),
                estimated_rtt_ms: Some(12),
                priority: Some(5),
                expires_in_seconds: Some(900),
            }],
        };
        assert!(validate_endpoint_card_register(&payload).is_ok());

        let invalid = DmEndpointCardRegisterRequest { cards: vec![] };
        assert!(validate_endpoint_card_register(&invalid).is_err());
    }

    #[test]
    fn validates_endpoint_card_revoke_payload() {
        let payload = DmEndpointCardRevokeRequest {
            endpoint_ids: vec!["lan-1".to_string()],
        };
        assert!(validate_endpoint_card_revoke(&payload).is_ok());

        let invalid = DmEndpointCardRevokeRequest {
            endpoint_ids: vec![],
        };
        assert!(validate_endpoint_card_revoke(&invalid).is_err());
    }

    #[test]
    fn validates_parallel_dial_payload() {
        let payload = DmParallelDialRequest {
            peer_identity_id: "usr-jules-p".to_string(),
            max_parallel_attempts: Some(3),
            unreachable_endpoint_ids: Some(vec!["wan-1".to_string()]),
        };
        assert!(validate_parallel_dial_request(&payload).is_ok());

        let invalid = DmParallelDialRequest {
            peer_identity_id: " ".to_string(),
            max_parallel_attempts: Some(0),
            unreachable_endpoint_ids: None,
        };
        assert!(validate_parallel_dial_request(&invalid).is_err());
    }
}
