use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, TimeZone, Utc};
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::{
    domain::dm::validation::{
        validate_connectivity_preflight, validate_dm_policy_update,
        validate_endpoint_card_register, validate_endpoint_card_revoke, validate_fanout_dispatch,
        validate_lan_discovery_announce, validate_pairing_envelope_create,
        validate_pairing_envelope_import, validate_parallel_dial_request,
        validate_profile_device_heartbeat, validate_wan_wizard_request,
        DM_ENDPOINT_CARD_DEFAULT_EXPIRY_SECONDS, DM_ENDPOINT_CARD_DEFAULT_RTT_MS,
        DM_OFFLINE_DELIVERY_MODE, DM_PAIRING_ENVELOPE_VERSION, DM_PARALLEL_DIAL_DEFAULT_ATTEMPTS,
    },
    models::{
        ApiError, DmConnectivityPreflightRequest, DmConnectivityPreflightResponse, DmEndpointCard,
        DmEndpointCardRecord, DmEndpointCardRegisterRequest, DmEndpointCardRegisterResponse,
        DmEndpointCardRevokeRequest, DmEndpointCardRevokeResponse, DmFanoutDispatchRequest,
        DmFanoutDispatchResponse, DmLanDiscoveryAnnounceRequest, DmLanDiscoveryAnnounceResponse,
        DmLanPeerListResponse, DmLanPeerSummary, DmLanPresenceRecord, DmMessagePage,
        DmMessageRecord, DmPairingEnvelopeCreateRequest, DmPairingEnvelopeImportRequest,
        DmPairingEnvelopeResponse, DmPairingImportResponse, DmParallelDialAttempt,
        DmParallelDialRequest, DmParallelDialResponse, DmPolicy, DmPolicyUpdate,
        DmProfileDeviceHeartbeatRequest, DmProfileDeviceHeartbeatResponse, DmProfileDeviceRecord,
        DmProfileDeviceSummary, DmThreadListQuery, DmThreadMessageListQuery, DmThreadPage,
        DmThreadSummary, DmWanWizardRequest, DmWanWizardResponse,
    },
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const LAN_DISCOVERY_TTL_SECONDS: i64 = 120;

#[derive(Serialize, Deserialize)]
struct PairingEnvelopeClaims {
    version: u32,
    inviter_identity_id: String,
    endpoint_hints: Vec<String>,
    nonce: String,
    issued_at: i64,
    expires_at: i64,
}

#[derive(Serialize, Deserialize)]
struct SignedPairingEnvelope {
    key_id: String,
    claims: PairingEnvelopeClaims,
    signature: String,
}

pub async fn get_dm_policy(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
) -> Json<DmPolicy> {
    let default = default_dm_policy();
    let policy = state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(&auth.identity_id)
        .cloned()
        .unwrap_or(default);
    Json(policy)
}

pub async fn update_dm_policy(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmPolicyUpdate>,
) -> ApiResult<Json<DmPolicy>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_dm_policy_update(&payload)?;

    let normalized = payload.inbound_policy.trim().to_string();
    let policy = DmPolicy {
        inbound_policy: normalized,
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    };

    state
        .dm_policies
        .write()
        .expect("acquire dm policy write lock")
        .insert(auth.identity_id, policy.clone());

    Ok(Json(policy))
}

pub async fn create_dm_pairing_envelope(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmPairingEnvelopeCreateRequest>,
) -> ApiResult<(StatusCode, Json<DmPairingEnvelopeResponse>)> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let expires_in_seconds = validate_pairing_envelope_create(&payload)?;

    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(expires_in_seconds as i64);
    let nonce = random_hex(16);

    let claims = PairingEnvelopeClaims {
        version: DM_PAIRING_ENVELOPE_VERSION,
        inviter_identity_id: auth.identity_id,
        endpoint_hints: payload.endpoint_hints,
        nonce: nonce.clone(),
        issued_at: issued_at.timestamp(),
        expires_at: expires_at.timestamp(),
    };

    let key_id = state.active_signing_key_id.clone();
    let key_secret = state
        .session_signing_keys
        .get(&key_id)
        .ok_or_else(|| bad_request("pairing_invalid", "active pairing signing key missing"))?;
    let signature = sign_pairing_claims(&claims, key_secret)?;

    let signed = SignedPairingEnvelope {
        key_id,
        claims,
        signature,
    };
    let envelope_json = serde_json::to_vec(&signed)
        .map_err(|_| bad_request("pairing_invalid", "failed to encode pairing envelope"))?;
    let envelope = URL_SAFE_NO_PAD.encode(envelope_json);

    Ok((
        StatusCode::CREATED,
        Json(DmPairingEnvelopeResponse {
            short_code: short_code_from_envelope(&envelope),
            envelope,
            expires_at: expires_at.to_rfc3339(),
            pairing_nonce: nonce,
        }),
    ))
}

pub async fn import_dm_pairing_envelope(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmPairingEnvelopeImportRequest>,
) -> ApiResult<Json<DmPairingImportResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_pairing_envelope_import(&payload)?;

    let signed = decode_signed_pairing_envelope(&payload.envelope)?;
    verify_pairing_envelope_signature(&state, &signed)?;

    if signed.claims.version != DM_PAIRING_ENVELOPE_VERSION {
        return Err(bad_request(
            "pairing_invalid",
            "unsupported pairing envelope version",
        ));
    }

    let now = Utc::now().timestamp();
    if now > signed.claims.expires_at {
        return Err(bad_request(
            "pairing_expired",
            "pairing envelope is expired",
        ));
    }

    if signed.claims.inviter_identity_id == auth.identity_id {
        return Err(bad_request(
            "identity_invalid",
            "cannot import a pairing envelope created by the same identity",
        ));
    }

    {
        let mut nonce_guard = state
            .dm_pairing_nonces
            .write()
            .expect("acquire dm pairing nonce write lock");
        nonce_guard.retain(|_, expiry| *expiry >= now);
        if nonce_guard.contains_key(&signed.claims.nonce) {
            return Err(bad_request(
                "pairing_replayed",
                "pairing envelope nonce was already consumed",
            ));
        }
        nonce_guard.insert(signed.claims.nonce.clone(), signed.claims.expires_at);
    }

    let expires_at = Utc
        .timestamp_opt(signed.claims.expires_at, 0)
        .single()
        .ok_or_else(|| bad_request("pairing_invalid", "invalid pairing expiry timestamp"))?;

    Ok(Json(DmPairingImportResponse {
        inviter_identity_id: signed.claims.inviter_identity_id,
        endpoint_hints: signed.claims.endpoint_hints,
        imported_at: Utc::now().to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn dm_connectivity_preflight(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    Json(payload): Json<DmConnectivityPreflightRequest>,
) -> ApiResult<Json<DmConnectivityPreflightResponse>> {
    validate_connectivity_preflight(&payload)?;

    if !payload.pairing_envelope_present.unwrap_or(false) {
        return Ok(Json(preflight_blocked(
            "pairing_missing",
            vec![
                "Import a signed pairing envelope from your contact.",
                "Ask your contact to regenerate the envelope if needed.",
            ],
        )));
    }

    if !payload.local_bind_allowed.unwrap_or(true) {
        return Ok(Json(preflight_blocked(
            "port_unavailable",
            vec![
                "Allow the app to bind a local port in your firewall settings.",
                "Close conflicting local apps and rerun preflight.",
            ],
        )));
    }

    let policy = state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(&auth.identity_id)
        .cloned()
        .unwrap_or_else(default_dm_policy);

    let same_server = payload.same_server_context.unwrap_or(false);
    match policy.inbound_policy.as_str() {
        "friends_only" => {
            let Some(peer_identity_id) = payload.peer_identity_id.as_deref() else {
                return Ok(Json(preflight_blocked(
                    "policy_blocked",
                    vec![
                        "Select a peer identity before running DM preflight.",
                        "Your DM policy currently allows only friends.",
                    ],
                )));
            };

            if !is_friend(&state, &auth.identity_id, peer_identity_id) {
                return Ok(Json(preflight_blocked(
                    "policy_blocked",
                    vec![
                        "Send and accept a friend request before starting this DM.",
                        "Or change your DM inbound policy from friends_only.",
                    ],
                )));
            }
        }
        "same_server" if !same_server => {
            return Ok(Json(preflight_blocked(
                "policy_blocked",
                vec![
                    "Join a shared server with this contact.",
                    "Or change your DM inbound policy from same_server.",
                ],
            )));
        }
        _ => {}
    }

    if !payload.peer_reachable_hint.unwrap_or(true) {
        return Ok(Json(preflight_blocked(
            "peer_unreachable",
            vec![
                "Ask your contact to keep the app online and rerun preflight.",
                "Confirm both clients use fresh pairing envelopes.",
            ],
        )));
    }

    if let Some(peer_identity_id) = payload.peer_identity_id.as_deref() {
        if has_fresh_lan_peer(&state, peer_identity_id, Utc::now().timestamp()) {
            return Ok(Json(DmConnectivityPreflightResponse {
                status: "ready".to_string(),
                reason_code: "preflight_ok_lan".to_string(),
                transport_profile: "direct_only".to_string(),
                remediation: vec![
                    "Peer discovered on local network; prioritize LAN direct path.".to_string(),
                ],
            }));
        }
    }

    Ok(Json(DmConnectivityPreflightResponse {
        status: "ready".to_string(),
        reason_code: "preflight_ok".to_string(),
        transport_profile: "direct_only".to_string(),
        remediation: vec!["Start direct DM connection.".to_string()],
    }))
}

pub async fn announce_dm_lan_discovery(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmLanDiscoveryAnnounceRequest>,
) -> ApiResult<Json<DmLanDiscoveryAnnounceResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_lan_discovery_announce(&payload)?;

    let now = Utc::now();
    let record = DmLanPresenceRecord {
        identity_id: auth.identity_id.clone(),
        endpoint_hints: payload.endpoint_hints,
        last_seen_epoch: now.timestamp(),
    };

    state
        .dm_lan_presence
        .write()
        .expect("acquire dm lan presence write lock")
        .insert(auth.identity_id.clone(), record.clone());

    Ok(Json(DmLanDiscoveryAnnounceResponse {
        identity_id: record.identity_id,
        endpoint_hints: record.endpoint_hints,
        scope: "lan_subnet".to_string(),
        last_seen_at: now.to_rfc3339(),
    }))
}

pub async fn list_dm_lan_peers(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
) -> Json<DmLanPeerListResponse> {
    let now = Utc::now().timestamp();
    let mut guard = state
        .dm_lan_presence
        .write()
        .expect("acquire dm lan presence write lock");

    guard.retain(|_, record| (now - record.last_seen_epoch) <= LAN_DISCOVERY_TTL_SECONDS);

    let items = guard
        .values()
        .filter(|record| record.identity_id != auth.identity_id)
        .map(|record| DmLanPeerSummary {
            identity_id: record.identity_id.clone(),
            endpoint_hints: record.endpoint_hints.clone(),
            last_seen_at: Utc
                .timestamp_opt(record.last_seen_epoch, 0)
                .single()
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339()),
        })
        .collect::<Vec<_>>();

    Json(DmLanPeerListResponse { items })
}

pub async fn run_dm_wan_wizard(
    _auth: AuthSession,
    Json(payload): Json<DmWanWizardRequest>,
) -> ApiResult<Json<DmWanWizardResponse>> {
    validate_wan_wizard_request(&payload)?;

    let profile = payload
        .network_profile
        .unwrap_or_else(|| "home_nat".to_string());
    let upnp_available = payload.upnp_available.unwrap_or(false);
    let nat_pmp_available = payload.nat_pmp_available.unwrap_or(false);
    let auto_mapping_succeeds = payload.auto_mapping_succeeds.unwrap_or(false);
    let external_port_open = payload.external_port_open.unwrap_or(false);
    let port = payload.preferred_port.unwrap_or(4040);

    if upnp_available && auto_mapping_succeeds {
        return Ok(Json(DmWanWizardResponse {
            outcome: "success".to_string(),
            method: "upnp".to_string(),
            reason_code: "wan_path_ready".to_string(),
            checklist: vec![
                format!("UPnP opened port {port} successfully."),
                "Use direct DM connection over WAN now.".to_string(),
            ],
        }));
    }

    if nat_pmp_available && auto_mapping_succeeds {
        return Ok(Json(DmWanWizardResponse {
            outcome: "success".to_string(),
            method: "nat_pmp".to_string(),
            reason_code: "wan_path_ready".to_string(),
            checklist: vec![
                format!("NAT-PMP opened port {port} successfully."),
                "Use direct DM connection over WAN now.".to_string(),
            ],
        }));
    }

    if external_port_open {
        return Ok(Json(DmWanWizardResponse {
            outcome: "success".to_string(),
            method: "manual".to_string(),
            reason_code: "wan_path_ready_manual".to_string(),
            checklist: vec![
                format!("Port {port} is externally reachable."),
                "Proceed with direct DM connection.".to_string(),
            ],
        }));
    }

    if matches!(
        profile.as_str(),
        "symmetric_nat" | "carrier_nat" | "enterprise_restricted"
    ) {
        return Ok(Json(DmWanWizardResponse {
            outcome: "network_incompatible".to_string(),
            method: "none".to_string(),
            reason_code: "wan_path_unavailable".to_string(),
            checklist: vec![
                "Current network profile blocks direct inbound WAN connectivity.".to_string(),
                "Try connecting on a shared LAN or different home network.".to_string(),
            ],
        }));
    }

    Ok(Json(DmWanWizardResponse {
        outcome: "manual_required".to_string(),
        method: "manual".to_string(),
        reason_code: "wan_manual_mapping_required".to_string(),
        checklist: vec![
            format!("Create manual router port-forward for UDP/TCP {port}."),
            "Re-run WAN wizard after applying router changes.".to_string(),
        ],
    }))
}

pub async fn register_dm_endpoint_cards(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmEndpointCardRegisterRequest>,
) -> ApiResult<Json<DmEndpointCardRegisterResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_endpoint_card_register(&payload)?;

    let now = Utc::now();
    let now_epoch = now.timestamp();
    let mut cards_by_identity = state
        .dm_endpoint_cards
        .write()
        .expect("acquire endpoint cards write lock");
    let cards = cards_by_identity
        .entry(auth.identity_id.clone())
        .or_default();

    cards.retain(|_, record| record.expires_at_epoch >= now_epoch);

    for card in payload.cards {
        let endpoint_id = card.endpoint_id.trim().to_string();
        let expires_in_seconds = card
            .expires_in_seconds
            .unwrap_or(DM_ENDPOINT_CARD_DEFAULT_EXPIRY_SECONDS);
        let record = DmEndpointCardRecord {
            endpoint_id: endpoint_id.clone(),
            endpoint_hint: card.endpoint_hint.trim().to_string(),
            estimated_rtt_ms: card
                .estimated_rtt_ms
                .unwrap_or(DM_ENDPOINT_CARD_DEFAULT_RTT_MS),
            priority: card.priority.unwrap_or(0),
            expires_at_epoch: now_epoch + expires_in_seconds as i64,
            revoked: false,
        };
        cards.insert(endpoint_id, record);
    }

    Ok(Json(DmEndpointCardRegisterResponse {
        identity_id: auth.identity_id,
        cards: cards_to_response(cards, now_epoch),
    }))
}

pub async fn revoke_dm_endpoint_cards(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmEndpointCardRevokeRequest>,
) -> ApiResult<Json<DmEndpointCardRevokeResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_endpoint_card_revoke(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let mut cards_by_identity = state
        .dm_endpoint_cards
        .write()
        .expect("acquire endpoint cards write lock");
    let cards = cards_by_identity
        .entry(auth.identity_id.clone())
        .or_default();

    cards.retain(|_, record| record.expires_at_epoch >= now_epoch);

    let mut revoked_endpoint_ids = Vec::new();
    for endpoint_id in payload.endpoint_ids {
        let normalized_endpoint_id = endpoint_id.trim().to_string();
        if let Some(record) = cards.get_mut(&normalized_endpoint_id) {
            if !record.revoked {
                record.revoked = true;
                revoked_endpoint_ids.push(normalized_endpoint_id);
            }
        }
    }

    Ok(Json(DmEndpointCardRevokeResponse {
        identity_id: auth.identity_id,
        revoked_endpoint_ids,
        remaining_cards: cards_to_response(cards, now_epoch),
    }))
}

pub async fn run_dm_parallel_dial(
    axum::extract::State(state): axum::extract::State<AppState>,
    _auth: AuthSession,
    Json(payload): Json<DmParallelDialRequest>,
) -> ApiResult<Json<DmParallelDialResponse>> {
    validate_parallel_dial_request(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let max_attempts = payload
        .max_parallel_attempts
        .unwrap_or(DM_PARALLEL_DIAL_DEFAULT_ATTEMPTS) as usize;
    let peer_identity_id = payload.peer_identity_id.trim().to_string();
    let unreachable: HashSet<String> = payload
        .unreachable_endpoint_ids
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .collect();

    let mut candidates = {
        let mut cards_by_identity = state
            .dm_endpoint_cards
            .write()
            .expect("acquire endpoint cards write lock");
        let Some(cards) = cards_by_identity.get_mut(&peer_identity_id) else {
            return Ok(Json(DmParallelDialResponse {
                status: "blocked".to_string(),
                reason_code: "endpoint_cards_missing".to_string(),
                transport_profile: "direct_only".to_string(),
                winner_endpoint_id: None,
                canceled_endpoint_ids: vec![],
                attempts: vec![],
                remediation: vec![
                    "Ask your contact to publish fresh endpoint cards.".to_string(),
                    "Retry parallel dial after endpoint-card sync completes.".to_string(),
                ],
            }));
        };
        cards.retain(|_, record| record.expires_at_epoch >= now_epoch);

        cards
            .values()
            .filter(|record| !record.revoked)
            .cloned()
            .collect::<Vec<_>>()
    };

    if candidates.is_empty() {
        return Ok(Json(DmParallelDialResponse {
            status: "blocked".to_string(),
            reason_code: "endpoint_cards_missing".to_string(),
            transport_profile: "direct_only".to_string(),
            winner_endpoint_id: None,
            canceled_endpoint_ids: vec![],
            attempts: vec![],
            remediation: vec![
                "Ask your contact to publish fresh endpoint cards.".to_string(),
                "Retry parallel dial after endpoint-card sync completes.".to_string(),
            ],
        }));
    }

    candidates.sort_by(|a, b| {
        a.estimated_rtt_ms
            .cmp(&b.estimated_rtt_ms)
            .then_with(|| b.priority.cmp(&a.priority))
    });
    candidates.truncate(max_attempts);

    let winner = candidates
        .iter()
        .find(|record| !unreachable.contains(&record.endpoint_id));

    let mut attempts = Vec::with_capacity(candidates.len());
    let mut canceled_endpoint_ids = Vec::new();

    if let Some(winner) = winner {
        for record in &candidates {
            if unreachable.contains(&record.endpoint_id) {
                attempts.push(DmParallelDialAttempt {
                    endpoint_id: record.endpoint_id.clone(),
                    endpoint_hint: record.endpoint_hint.clone(),
                    estimated_rtt_ms: record.estimated_rtt_ms,
                    status: "failed".to_string(),
                    cancellation_reason: Some("dial_unreachable".to_string()),
                });
            } else if record.endpoint_id == winner.endpoint_id {
                attempts.push(DmParallelDialAttempt {
                    endpoint_id: record.endpoint_id.clone(),
                    endpoint_hint: record.endpoint_hint.clone(),
                    estimated_rtt_ms: record.estimated_rtt_ms,
                    status: "connected".to_string(),
                    cancellation_reason: None,
                });
            } else {
                canceled_endpoint_ids.push(record.endpoint_id.clone());
                attempts.push(DmParallelDialAttempt {
                    endpoint_id: record.endpoint_id.clone(),
                    endpoint_hint: record.endpoint_hint.clone(),
                    estimated_rtt_ms: record.estimated_rtt_ms,
                    status: "cancelled".to_string(),
                    cancellation_reason: Some("winner_selected".to_string()),
                });
            }
        }

        return Ok(Json(DmParallelDialResponse {
            status: "ready".to_string(),
            reason_code: "parallel_dial_connected".to_string(),
            transport_profile: "direct_only".to_string(),
            winner_endpoint_id: Some(winner.endpoint_id.clone()),
            canceled_endpoint_ids,
            attempts,
            remediation: vec![
                "Parallel dial selected the fastest reachable endpoint card.".to_string(),
            ],
        }));
    }

    for record in &candidates {
        attempts.push(DmParallelDialAttempt {
            endpoint_id: record.endpoint_id.clone(),
            endpoint_hint: record.endpoint_hint.clone(),
            estimated_rtt_ms: record.estimated_rtt_ms,
            status: "failed".to_string(),
            cancellation_reason: Some("dial_unreachable".to_string()),
        });
    }

    Ok(Json(DmParallelDialResponse {
        status: "blocked".to_string(),
        reason_code: "parallel_dial_exhausted".to_string(),
        transport_profile: "direct_only".to_string(),
        winner_endpoint_id: None,
        canceled_endpoint_ids,
        attempts,
        remediation: vec![
            "All attempted endpoint cards were unreachable.".to_string(),
            "Refresh endpoint cards and retry direct connection.".to_string(),
        ],
    }))
}

pub async fn heartbeat_dm_profile_device(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmProfileDeviceHeartbeatRequest>,
) -> ApiResult<Json<DmProfileDeviceHeartbeatResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_profile_device_heartbeat(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let mut devices_by_identity = state
        .dm_profile_devices
        .write()
        .expect("acquire dm profile devices write lock");
    let devices = devices_by_identity
        .entry(auth.identity_id.clone())
        .or_default();
    let device_id = payload.device_id.trim().to_string();

    devices.insert(
        device_id.clone(),
        DmProfileDeviceRecord {
            device_id,
            active: payload.active,
            last_seen_epoch: now_epoch,
        },
    );

    Ok(Json(DmProfileDeviceHeartbeatResponse {
        identity_id: auth.identity_id,
        devices: profile_devices_to_response(devices, now_epoch),
    }))
}

pub async fn run_dm_active_fanout(
    axum::extract::State(state): axum::extract::State<AppState>,
    _auth: AuthSession,
    Json(payload): Json<DmFanoutDispatchRequest>,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    validate_fanout_dispatch(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let recipient_identity_id = payload.recipient_identity_id.trim();
    let source_device_id = payload
        .source_device_id
        .as_ref()
        .map(|value| value.trim().to_string());

    let (mut delivered_device_ids, mut skipped_device_ids) = {
        let mut devices_by_identity = state
            .dm_profile_devices
            .write()
            .expect("acquire dm profile devices write lock");

        let Some(devices) = devices_by_identity.get_mut(recipient_identity_id) else {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_no_active_devices".to_string(),
                transport_profile: "direct_only".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        };

        let mut delivered = Vec::new();
        let mut skipped = Vec::new();
        for record in devices.values_mut() {
            if !record.active {
                skipped.push(record.device_id.clone());
                continue;
            }

            if source_device_id
                .as_ref()
                .map(|value| value == &record.device_id)
                .unwrap_or(false)
            {
                skipped.push(record.device_id.clone());
                continue;
            }

            record.last_seen_epoch = now_epoch;
            delivered.push(record.device_id.clone());
        }

        (delivered, skipped)
    };

    delivered_device_ids.sort();
    skipped_device_ids.sort();

    if delivered_device_ids.is_empty() {
        return Ok(Json(DmFanoutDispatchResponse {
            status: "blocked".to_string(),
            reason_code: "fanout_no_active_devices".to_string(),
            transport_profile: "direct_only".to_string(),
            fanout_count: 0,
            delivered_device_ids,
            skipped_device_ids,
        }));
    }

    Ok(Json(DmFanoutDispatchResponse {
        status: "ready".to_string(),
        reason_code: "fanout_ok".to_string(),
        transport_profile: "direct_only".to_string(),
        fanout_count: delivered_device_ids.len() as u32,
        delivered_device_ids,
        skipped_device_ids,
    }))
}

pub async fn list_dm_threads(
    _auth: AuthSession,
    Query(query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    let limit = parse_limit(query.limit)?;
    let mut items = dm_thread_fixtures();

    if query.unread_only.unwrap_or(false) {
        items.retain(|item| item.unread > 0);
    }

    let start = if let Some(cursor) = query.cursor {
        items
            .iter()
            .position(|item| item.thread_id == cursor)
            .map(|idx| idx + 1)
            .ok_or_else(|| bad_request("cursor_invalid", "unknown dm thread cursor"))?
    } else {
        0
    };

    let page_items = items
        .iter()
        .skip(start)
        .take(limit)
        .cloned()
        .collect::<Vec<_>>();
    let has_more = start + page_items.len() < items.len();
    let next_cursor = if has_more {
        page_items.last().map(|item| item.thread_id.clone())
    } else {
        None
    };

    Ok(Json(DmThreadPage {
        items: page_items,
        next_cursor,
    }))
}

pub async fn list_dm_thread_messages(
    _auth: AuthSession,
    Path(thread_id): Path<String>,
    Query(query): Query<DmThreadMessageListQuery>,
) -> ApiResult<Json<DmMessagePage>> {
    let limit = parse_limit(query.limit)?;
    let cursor = parse_message_cursor(query.cursor)?;

    let mut items = dm_message_fixtures(&thread_id).ok_or({
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "thread_not_found",
                message: "dm thread was not found",
            }),
        )
    })?;

    if let Some(cursor) = cursor {
        items.retain(|item| item.seq < cursor);
    }

    let page_items = items.iter().take(limit).cloned().collect::<Vec<_>>();
    let has_more = page_items.len() < items.len();
    let next_cursor = if has_more {
        page_items.last().map(|item| item.seq.to_string())
    } else {
        None
    };

    Ok(Json(DmMessagePage {
        items: page_items,
        next_cursor,
    }))
}

fn parse_limit(value: Option<u32>) -> ApiResult<usize> {
    let raw = value.unwrap_or(DEFAULT_PAGE_LIMIT as u32);
    if raw == 0 {
        return Err(bad_request(
            "limit_invalid",
            "limit must be greater than zero",
        ));
    }
    if raw as usize > MAX_PAGE_LIMIT {
        return Err(bad_request(
            "limit_invalid",
            "limit exceeds maximum page size",
        ));
    }

    Ok(raw as usize)
}

fn parse_message_cursor(value: Option<String>) -> ApiResult<Option<u64>> {
    let Some(cursor) = value else {
        return Ok(None);
    };

    cursor
        .parse::<u64>()
        .map(Some)
        .map_err(|_| bad_request("cursor_invalid", "message cursor must be numeric"))
}

fn dm_thread_fixtures() -> Vec<DmThreadSummary> {
    vec![
        DmThreadSummary {
            thread_id: "dm-thread-nora-jules".to_string(),
            kind: "dm".to_string(),
            title: "Nora K + Jules P".to_string(),
            participant_ids: vec!["usr-nora-k".to_string(), "usr-jules-p".to_string()],
            unread: 3,
            last_read_seq: 401,
            last_message_seq: 404,
            last_message_preview: "See you in the relay standup".to_string(),
            last_message_at: "2026-03-12T09:21:11Z".to_string(),
        },
        DmThreadSummary {
            thread_id: "gdm-thread-atlas".to_string(),
            kind: "group_dm".to_string(),
            title: "Atlas Draft Squad".to_string(),
            participant_ids: vec![
                "usr-nora-k".to_string(),
                "usr-mina-s".to_string(),
                "usr-alex-r".to_string(),
            ],
            unread: 1,
            last_read_seq: 144,
            last_message_seq: 145,
            last_message_preview: "Pushed the draft, review when free".to_string(),
            last_message_at: "2026-03-12T08:10:00Z".to_string(),
        },
        DmThreadSummary {
            thread_id: "dm-thread-nora-alex".to_string(),
            kind: "dm".to_string(),
            title: "Nora K + Alex R".to_string(),
            participant_ids: vec!["usr-nora-k".to_string(), "usr-alex-r".to_string()],
            unread: 0,
            last_read_seq: 220,
            last_message_seq: 220,
            last_message_preview: "Thanks for confirming the schedule".to_string(),
            last_message_at: "2026-03-11T21:45:30Z".to_string(),
        },
    ]
}

fn dm_message_fixtures(thread_id: &str) -> Option<Vec<DmMessageRecord>> {
    match thread_id {
        "dm-thread-nora-jules" => Some(vec![
            DmMessageRecord {
                message_id: "msg-404".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-jules-p".to_string(),
                seq: 404,
                ciphertext: "enc:95a0f4".to_string(),
                created_at: "2026-03-12T09:21:11Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-403".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 403,
                ciphertext: "enc:4bf120".to_string(),
                created_at: "2026-03-12T09:19:24Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-402".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-jules-p".to_string(),
                seq: 402,
                ciphertext: "enc:5c8e73".to_string(),
                created_at: "2026-03-12T09:12:00Z".to_string(),
                edited_at: Some("2026-03-12T09:12:39Z".to_string()),
            },
            DmMessageRecord {
                message_id: "msg-401".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 401,
                ciphertext: "enc:88f0ab".to_string(),
                created_at: "2026-03-12T09:05:08Z".to_string(),
                edited_at: None,
            },
        ]),
        "gdm-thread-atlas" => Some(vec![
            DmMessageRecord {
                message_id: "msg-145".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-mina-s".to_string(),
                seq: 145,
                ciphertext: "enc:10beef".to_string(),
                created_at: "2026-03-12T08:10:00Z".to_string(),
                edited_at: None,
            },
            DmMessageRecord {
                message_id: "msg-144".to_string(),
                thread_id: thread_id.to_string(),
                author_id: "usr-nora-k".to_string(),
                seq: 144,
                ciphertext: "enc:bada55".to_string(),
                created_at: "2026-03-12T08:03:19Z".to_string(),
                edited_at: None,
            },
        ]),
        "dm-thread-nora-alex" => Some(vec![DmMessageRecord {
            message_id: "msg-220".to_string(),
            thread_id: thread_id.to_string(),
            author_id: "usr-alex-r".to_string(),
            seq: 220,
            ciphertext: "enc:deed01".to_string(),
            created_at: "2026-03-11T21:45:30Z".to_string(),
            edited_at: None,
        }]),
        _ => None,
    }
}

fn default_dm_policy() -> DmPolicy {
    DmPolicy {
        inbound_policy: "friends_only".to_string(),
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    }
}

fn preflight_blocked(reason_code: &str, remediation: Vec<&str>) -> DmConnectivityPreflightResponse {
    DmConnectivityPreflightResponse {
        status: "blocked".to_string(),
        reason_code: reason_code.to_string(),
        transport_profile: "direct_only".to_string(),
        remediation: remediation
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect(),
    }
}

fn cards_to_response(
    cards: &std::collections::HashMap<String, DmEndpointCardRecord>,
    now_epoch: i64,
) -> Vec<DmEndpointCard> {
    let mut items = cards
        .values()
        .filter(|record| record.expires_at_epoch >= now_epoch)
        .map(|record| DmEndpointCard {
            endpoint_id: record.endpoint_id.clone(),
            endpoint_hint: record.endpoint_hint.clone(),
            estimated_rtt_ms: record.estimated_rtt_ms,
            priority: record.priority,
            expires_at: Utc
                .timestamp_opt(record.expires_at_epoch, 0)
                .single()
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| {
                    Utc.timestamp_opt(now_epoch, 0)
                        .single()
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| Utc::now().to_rfc3339())
                }),
            revoked: record.revoked,
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| a.endpoint_id.cmp(&b.endpoint_id));
    items
}

fn profile_devices_to_response(
    devices: &std::collections::HashMap<String, DmProfileDeviceRecord>,
    now_epoch: i64,
) -> Vec<DmProfileDeviceSummary> {
    let mut items = devices
        .values()
        .map(|record| DmProfileDeviceSummary {
            device_id: record.device_id.clone(),
            active: record.active,
            last_seen_at: Utc
                .timestamp_opt(record.last_seen_epoch, 0)
                .single()
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| {
                    Utc.timestamp_opt(now_epoch, 0)
                        .single()
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_else(|| Utc::now().to_rfc3339())
                }),
        })
        .collect::<Vec<_>>();

    items.sort_by(|a, b| a.device_id.cmp(&b.device_id));
    items
}

fn is_friend(state: &AppState, a: &str, b: &str) -> bool {
    state
        .friend_requests
        .read()
        .expect("acquire friend request read lock")
        .values()
        .any(|record| {
            record.status == "accepted"
                && ((record.requester_identity_id == a && record.target_identity_id == b)
                    || (record.requester_identity_id == b && record.target_identity_id == a))
        })
}

fn has_fresh_lan_peer(state: &AppState, peer_identity_id: &str, now: i64) -> bool {
    state
        .dm_lan_presence
        .read()
        .expect("acquire dm lan presence read lock")
        .get(peer_identity_id)
        .map(|record| (now - record.last_seen_epoch) <= LAN_DISCOVERY_TTL_SECONDS)
        .unwrap_or(false)
}

fn sign_pairing_claims(claims: &PairingEnvelopeClaims, key_secret: &str) -> ApiResult<String> {
    let claims_json = serde_json::to_vec(claims)
        .map_err(|_| bad_request("pairing_invalid", "failed to encode pairing claims"))?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, key_secret.as_bytes());
    let digest = hmac::sign(&key, &claims_json);
    Ok(hex::encode(digest.as_ref()))
}

fn decode_signed_pairing_envelope(encoded: &str) -> ApiResult<SignedPairingEnvelope> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| bad_request("pairing_invalid", "pairing envelope is not valid base64url"))?;
    serde_json::from_slice::<SignedPairingEnvelope>(&bytes)
        .map_err(|_| bad_request("pairing_invalid", "pairing envelope payload is invalid"))
}

fn verify_pairing_envelope_signature(
    state: &AppState,
    envelope: &SignedPairingEnvelope,
) -> ApiResult<()> {
    let key_secret = state
        .session_signing_keys
        .get(&envelope.key_id)
        .ok_or_else(|| bad_request("pairing_invalid", "unknown pairing signing key"))?;

    let expected = sign_pairing_claims(&envelope.claims, key_secret)?;
    if expected != envelope.signature {
        return Err(bad_request(
            "pairing_invalid",
            "pairing envelope signature verification failed",
        ));
    }

    Ok(())
}

fn random_hex(bytes_len: usize) -> String {
    use rand::RngCore;

    let mut bytes = vec![0u8; bytes_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn short_code_from_envelope(envelope: &str) -> String {
    let digest = ring::digest::digest(&ring::digest::SHA256, envelope.as_bytes());
    let encoded = hex::encode(digest.as_ref());
    format!(
        "{}-{}-{}",
        &encoded[0..4].to_uppercase(),
        &encoded[4..8].to_uppercase(),
        &encoded[8..12].to_uppercase()
    )
}
