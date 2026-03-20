use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, TimeZone, Utc};
use ring::{digest, hmac};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::infra::db::repos::{dm_history_repo, dm_repo, friends_repo, servers_repo};
use crate::{
    domain::dm::validation::{
        validate_connectivity_preflight, validate_dm_policy_update,
        validate_endpoint_card_register, validate_endpoint_card_revoke, validate_fanout_catch_up,
        validate_fanout_dispatch, validate_lan_discovery_announce,
        validate_pairing_envelope_create, validate_pairing_envelope_import,
        validate_parallel_dial_request, validate_profile_device_heartbeat,
        validate_wan_wizard_request, DM_ENDPOINT_CARD_DEFAULT_EXPIRY_SECONDS,
        DM_ENDPOINT_CARD_DEFAULT_RTT_MS, DM_OFFLINE_DELIVERY_MODE, DM_PAIRING_ENVELOPE_VERSION,
        DM_PARALLEL_DIAL_DEFAULT_ATTEMPTS,
    },
    models::{
        ApiError, DmConnectivityPreflightRequest, DmConnectivityPreflightResponse, DmEndpointCard,
        DmEndpointCardRecord, DmEndpointCardRegisterRequest, DmEndpointCardRegisterResponse,
        DmEndpointCardRevokeRequest, DmEndpointCardRevokeResponse, DmFanoutCatchUpItem,
        DmFanoutCatchUpRequest, DmFanoutCatchUpResponse, DmFanoutDeliveryRecord,
        DmFanoutDispatchRequest, DmFanoutDispatchResponse, DmLanDiscoveryAnnounceRequest,
        DmLanDiscoveryAnnounceResponse, DmLanPeerListResponse, DmLanPeerSummary,
        DmLanPresenceRecord, DmMessagePage, DmPairingEnvelopeCreateRequest,
        DmPairingEnvelopeImportRequest, DmPairingEnvelopeResponse, DmPairingImportResponse,
        DmParallelDialAttempt, DmParallelDialRequest, DmParallelDialResponse, DmPolicy,
        DmPolicyUpdate, DmProfileDeviceHeartbeatRequest, DmProfileDeviceHeartbeatResponse,
        DmProfileDeviceRecord, DmProfileDeviceSummary, DmThreadListQuery, DmThreadMarkReadRequest,
        DmThreadMarkReadResponse, DmThreadMessageListQuery, DmThreadPage, DmWanWizardRequest,
        DmWanWizardResponse,
    },
    shared::errors::{bad_request, internal_error, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;
const LAN_DISCOVERY_TTL_SECONDS: i64 = 120;
const DM_FANOUT_MAX_LOG_ENTRIES: usize = 1024;

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
) -> ApiResult<Json<DmPolicy>> {
    if let Some(pool) = state.db_pool.as_ref() {
        let policy = dm_repo::get_dm_policy(pool, &auth.identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load dm policy"))?
            .unwrap_or_else(default_dm_policy);
        return Ok(Json(policy));
    }

    let default = default_dm_policy();
    let policy = state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(&auth.identity_id)
        .cloned()
        .unwrap_or(default);
    Ok(Json(policy))
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

    if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::upsert_dm_policy(pool, &auth.identity_id, &policy)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to persist dm policy"))?;
    }

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

    let expires_at = Utc
        .timestamp_opt(signed.claims.expires_at, 0)
        .single()
        .ok_or_else(|| bad_request("pairing_invalid", "invalid pairing expiry timestamp"))?;

    let nonce_consumed = consume_pairing_nonce(
        &state,
        &signed.claims.nonce,
        signed.claims.expires_at,
        expires_at,
    )
    .await?;
    if !nonce_consumed {
        return Err(bad_request(
            "pairing_replayed",
            "pairing envelope nonce was already consumed",
        ));
    }

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

    if let Some(peer_identity_id) = payload.peer_identity_id.as_deref() {
        match dm_interaction_policy_decision(&state, &auth.identity_id, peer_identity_id).await? {
            DmInteractionPolicyDecision::Allowed => {}
            DmInteractionPolicyDecision::BlockedFriendsOnly
            | DmInteractionPolicyDecision::BlockedUnknown => {
                return Ok(Json(preflight_blocked(
                    "policy_blocked",
                    vec![
                        "Send and accept a friend request before starting this DM.",
                        "Or ask your contact to change their DM inbound policy.",
                    ],
                )));
            }
            DmInteractionPolicyDecision::BlockedSameServer => {
                return Ok(Json(preflight_blocked(
                    "policy_blocked",
                    vec![
                        "No shared server membership could be confirmed for this contact.",
                        "Join a shared server with your contact, or ask them to change their DM inbound policy.",
                    ],
                )));
            }
        }
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
) -> ApiResult<Json<DmLanPeerListResponse>> {
    let now = Utc::now().timestamp();
    let candidate_records = {
        let mut guard = state
            .dm_lan_presence
            .write()
            .expect("acquire dm lan presence write lock");

        guard.retain(|_, record| (now - record.last_seen_epoch) <= LAN_DISCOVERY_TTL_SECONDS);

        guard
            .values()
            .filter(|record| record.identity_id != auth.identity_id)
            .cloned()
            .collect::<Vec<_>>()
    };

    let mut items = Vec::new();
    for record in candidate_records {
        if matches!(
            dm_interaction_policy_decision(&state, &auth.identity_id, &record.identity_id).await?,
            DmInteractionPolicyDecision::Allowed
        ) {
            items.push(DmLanPeerSummary {
                identity_id: record.identity_id,
                endpoint_hints: record.endpoint_hints,
                last_seen_at: Utc
                    .timestamp_opt(record.last_seen_epoch, 0)
                    .single()
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| Utc::now().to_rfc3339()),
            });
        }
    }

    items.sort_by(|a, b| a.identity_id.cmp(&b.identity_id));

    Ok(Json(DmLanPeerListResponse { items }))
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
    let identity_id = auth.identity_id.clone();
    let mut cards = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::list_dm_endpoint_cards(pool, &identity_id, now_epoch)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load endpoint cards"))?
            .into_iter()
            .map(|record| (record.endpoint_id.clone(), record))
            .collect::<std::collections::HashMap<_, _>>()
    } else {
        let mut cards_by_identity = state
            .dm_endpoint_cards
            .write()
            .expect("acquire endpoint cards write lock");
        let cards = cards_by_identity.entry(identity_id.clone()).or_default();
        cards.retain(|_, record| record.expires_at_epoch >= now_epoch);
        cards.clone()
    };

    let mut new_records = Vec::with_capacity(payload.cards.len());
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
        cards.insert(endpoint_id, record.clone());
        new_records.push(record);
    }

    if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::upsert_dm_endpoint_cards_batch(pool, &identity_id, &new_records)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist endpoint cards")
            })?;
    } else {
        state
            .dm_endpoint_cards
            .write()
            .expect("acquire endpoint cards write lock")
            .insert(identity_id.clone(), cards.clone());
    }

    Ok(Json(DmEndpointCardRegisterResponse {
        identity_id,
        cards: cards_to_response(&cards, now_epoch),
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
    let identity_id = auth.identity_id.clone();
    let endpoint_ids = payload
        .endpoint_ids
        .iter()
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();

    let mut cards = if let Some(pool) = state.db_pool.as_ref() {
        let revoked_endpoint_ids =
            dm_repo::mark_dm_endpoint_cards_revoked(pool, &identity_id, &endpoint_ids)
                .await
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to revoke endpoint cards")
                })?;
        let cards = dm_repo::list_dm_endpoint_cards(pool, &identity_id, now_epoch)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load endpoint cards"))?
            .into_iter()
            .map(|record| (record.endpoint_id.clone(), record))
            .collect::<std::collections::HashMap<_, _>>();

        return Ok(Json(DmEndpointCardRevokeResponse {
            identity_id,
            revoked_endpoint_ids,
            remaining_cards: cards_to_response(&cards, now_epoch),
        }));
    } else {
        let mut cards_by_identity = state
            .dm_endpoint_cards
            .write()
            .expect("acquire endpoint cards write lock");
        let cards = cards_by_identity.entry(identity_id.clone()).or_default();
        cards.retain(|_, record| record.expires_at_epoch >= now_epoch);
        cards.clone()
    };

    let mut revoked_endpoint_ids = Vec::new();
    for normalized_endpoint_id in endpoint_ids {
        if let Some(record) = cards.get_mut(&normalized_endpoint_id) {
            if !record.revoked {
                record.revoked = true;
                revoked_endpoint_ids.push(normalized_endpoint_id);
            }
        }
    }

    state
        .dm_endpoint_cards
        .write()
        .expect("acquire endpoint cards write lock")
        .insert(identity_id.clone(), cards.clone());

    Ok(Json(DmEndpointCardRevokeResponse {
        identity_id,
        revoked_endpoint_ids,
        remaining_cards: cards_to_response(&cards, now_epoch),
    }))
}

pub async fn run_dm_parallel_dial(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmParallelDialRequest>,
) -> ApiResult<Json<DmParallelDialResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_parallel_dial_request(&payload)?;

    let now_epoch = Utc::now().timestamp();
    let max_attempts = payload
        .max_parallel_attempts
        .unwrap_or(DM_PARALLEL_DIAL_DEFAULT_ATTEMPTS) as usize;
    let peer_identity_id = payload.peer_identity_id.trim().to_string();
    match dm_interaction_policy_decision(&state, &auth.identity_id, &peer_identity_id).await? {
        DmInteractionPolicyDecision::Allowed => {}
        DmInteractionPolicyDecision::BlockedFriendsOnly
        | DmInteractionPolicyDecision::BlockedUnknown => {
            return Ok(Json(DmParallelDialResponse {
                status: "blocked".to_string(),
                reason_code: "parallel_dial_policy_blocked".to_string(),
                transport_profile: "direct_only".to_string(),
                winner_endpoint_id: None,
                canceled_endpoint_ids: vec![],
                attempts: vec![],
                remediation: vec![
                    "Recipient DM policy currently blocks direct connectivity attempts from this sender."
                        .to_string(),
                    "Send and accept a friend request, or ask recipient to change DM inbound policy."
                        .to_string(),
                ],
            }));
        }
        DmInteractionPolicyDecision::BlockedSameServer => {
            return Ok(Json(DmParallelDialResponse {
                status: "blocked".to_string(),
                reason_code: "parallel_dial_same_server_context_required".to_string(),
                transport_profile: "direct_only".to_string(),
                winner_endpoint_id: None,
                canceled_endpoint_ids: vec![],
                attempts: vec![],
                remediation: vec![
                    "Recipient DM policy is same_server but no shared server membership could be confirmed."
                        .to_string(),
                    "Join a shared server with the recipient, or ask them to change their DM inbound policy."
                        .to_string(),
                ],
            }));
        }
    }

    let unreachable: HashSet<String> = payload
        .unreachable_endpoint_ids
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.trim().to_string())
        .collect();

    let mut candidates = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::list_dm_endpoint_cards(pool, &peer_identity_id, now_epoch)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load endpoint cards"))?
            .into_iter()
            .filter(|record| !record.revoked)
            .collect::<Vec<_>>()
    } else {
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
    let device_id = payload.device_id.trim().to_string();
    let identity_id = auth.identity_id.clone();
    let record = DmProfileDeviceRecord {
        device_id: device_id.clone(),
        active: payload.active,
        last_seen_epoch: now_epoch,
    };

    let devices = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::upsert_dm_profile_device(pool, &identity_id, &record)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist profile device")
            })?;
        dm_repo::list_dm_profile_devices(pool, &identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load profile devices"))?
            .into_iter()
            .map(|record| (record.device_id.clone(), record))
            .collect::<std::collections::HashMap<_, _>>()
    } else {
        let mut devices_by_identity = state
            .dm_profile_devices
            .write()
            .expect("acquire dm profile devices write lock");
        let devices = devices_by_identity.entry(identity_id.clone()).or_default();
        devices.insert(device_id, record);
        devices.clone()
    };

    if state.db_pool.is_none() {
        state
            .dm_profile_devices
            .write()
            .expect("acquire dm profile devices write lock")
            .insert(identity_id.clone(), devices.clone());
    }

    Ok(Json(DmProfileDeviceHeartbeatResponse {
        identity_id,
        devices: profile_devices_to_response(&devices, now_epoch),
    }))
}

pub async fn run_dm_active_fanout(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmFanoutDispatchRequest>,
) -> ApiResult<Json<DmFanoutDispatchResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    validate_fanout_dispatch(&payload)?;

    let recipient_identity_id = payload.recipient_identity_id.trim();
    match dm_interaction_policy_decision(&state, &auth.identity_id, recipient_identity_id).await? {
        DmInteractionPolicyDecision::Allowed => {}
        DmInteractionPolicyDecision::BlockedFriendsOnly
        | DmInteractionPolicyDecision::BlockedUnknown => {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_policy_blocked".to_string(),
                transport_profile: "direct_only".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        }
        DmInteractionPolicyDecision::BlockedSameServer => {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_same_server_context_required".to_string(),
                transport_profile: "direct_only".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        }
    }

    let source_device_id = payload
        .source_device_id
        .as_ref()
        .map(|value| value.trim().to_string());

    let profile_devices = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::list_dm_profile_devices(pool, recipient_identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load profile devices"))?
            .into_iter()
            .map(|record| (record.device_id.clone(), record))
            .collect::<std::collections::HashMap<_, _>>()
    } else {
        state
            .dm_profile_devices
            .read()
            .expect("acquire dm profile devices read lock")
            .get(recipient_identity_id)
            .cloned()
            .unwrap_or_default()
    };

    let (mut delivered_device_ids, mut skipped_device_ids) = {
        if profile_devices.is_empty() {
            return Ok(Json(DmFanoutDispatchResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_no_active_devices".to_string(),
                transport_profile: "direct_only".to_string(),
                fanout_count: 0,
                delivered_device_ids: vec![],
                skipped_device_ids: vec![],
            }));
        }

        let mut delivered = Vec::new();
        let mut skipped = Vec::new();
        for record in profile_devices.values() {
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

    let known_device_ids = profile_devices.keys().cloned().collect::<Vec<_>>();
    let min_cursor = if let Some(pool) = state.db_pool.as_ref() {
        let persisted = dm_repo::list_dm_fanout_device_cursors(pool, recipient_identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load fanout cursors"))?
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();

        known_device_ids
            .iter()
            .map(|device_id| persisted.get(device_id).copied().unwrap_or(0))
            .min()
            .unwrap_or(0)
    } else {
        state
            .dm_fanout_device_cursors
            .read()
            .expect("acquire dm fanout cursor read lock")
            .get(recipient_identity_id)
            .map(|cursors| {
                known_device_ids
                    .iter()
                    .map(|device_id| cursors.get(device_id).copied().unwrap_or(0))
                    .min()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    };

    let message_id = payload.message_id.trim().to_string();
    let cursor = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::advance_dm_fanout_stream_head(pool, recipient_identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to advance fanout cursor"))?
    } else {
        let fanout_delivery_log = state
            .dm_fanout_delivery_log
            .read()
            .expect("acquire dm fanout delivery log read lock");
        fanout_delivery_log
            .get(recipient_identity_id)
            .and_then(|delivery_log| delivery_log.last())
            .map(|record| record.cursor.saturating_add(1))
            .unwrap_or(1)
    };

    let mut fanout_delivery_log = state
        .dm_fanout_delivery_log
        .write()
        .expect("acquire dm fanout delivery log write lock");
    let delivery_log = fanout_delivery_log
        .entry(recipient_identity_id.to_string())
        .or_default();
    if min_cursor > 0 {
        delivery_log.retain(|record| record.cursor > min_cursor);
    }
    if delivery_log.len() >= DM_FANOUT_MAX_LOG_ENTRIES {
        return Ok(Json(DmFanoutDispatchResponse {
            status: "blocked".to_string(),
            reason_code: "fanout_backlog_full".to_string(),
            transport_profile: "direct_only".to_string(),
            fanout_count: 0,
            delivered_device_ids: vec![],
            skipped_device_ids,
        }));
    }

    delivery_log.push(DmFanoutDeliveryRecord {
        cursor,
        message_id,
        sender_identity_id: auth.identity_id.clone(),
        ciphertext: payload.ciphertext.clone(),
        source_device_id: source_device_id.clone(),
        delivered_device_ids: delivered_device_ids.clone(),
    });

    Ok(Json(DmFanoutDispatchResponse {
        status: "ready".to_string(),
        reason_code: "fanout_ok".to_string(),
        transport_profile: "direct_only".to_string(),
        fanout_count: delivered_device_ids.len() as u32,
        delivered_device_ids,
        skipped_device_ids,
    }))
}

pub async fn run_dm_fanout_catch_up(
    axum::extract::State(state): axum::extract::State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Json(payload): Json<DmFanoutCatchUpRequest>,
) -> ApiResult<Json<DmFanoutCatchUpResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;
    let (limit, request_cursor) = validate_fanout_catch_up(&payload)?;

    let device_id = payload.device_id.trim().to_string();
    let identity_id = auth.identity_id;

    {
        let devices = if let Some(pool) = state.db_pool.as_ref() {
            dm_repo::list_dm_profile_devices(pool, &identity_id)
                .await
                .map_err(|_| {
                    internal_error("storage_unavailable", "failed to load profile devices")
                })?
                .into_iter()
                .map(|record| (record.device_id.clone(), record))
                .collect::<std::collections::HashMap<_, _>>()
        } else {
            state
                .dm_profile_devices
                .read()
                .expect("acquire dm profile devices read lock")
                .get(&identity_id)
                .cloned()
                .unwrap_or_default()
        };

        if devices.is_empty() {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_unknown".to_string(),
                transport_profile: "direct_only".to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        }

        let Some(record) = devices.get(&device_id) else {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_unknown".to_string(),
                transport_profile: "direct_only".to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        };

        if !record.active {
            return Ok(Json(DmFanoutCatchUpResponse {
                status: "blocked".to_string(),
                reason_code: "fanout_device_inactive".to_string(),
                transport_profile: "direct_only".to_string(),
                device_id,
                replay_count: 0,
                next_cursor: "0".to_string(),
                deduped_message_ids: vec![],
                items: vec![],
            }));
        }
    }

    let last_cursor = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::get_dm_fanout_device_cursor(pool, &identity_id, &device_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load fanout cursor"))?
    } else {
        state
            .dm_fanout_device_cursors
            .read()
            .expect("acquire dm fanout cursor read lock")
            .get(&identity_id)
            .and_then(|cursors| cursors.get(&device_id))
            .copied()
            .unwrap_or(0)
    };
    let user_cursor = request_cursor.unwrap_or(0);

    let tail_cursor = if let Some(pool) = state.db_pool.as_ref() {
        dm_repo::get_dm_fanout_stream_head(pool, &identity_id)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to load fanout stream head")
            })?
    } else {
        let delivery_log = state
            .dm_fanout_delivery_log
            .read()
            .expect("acquire dm fanout delivery log read lock");
        delivery_log
            .get(&identity_id)
            .and_then(|entries| entries.last())
            .map(|entry| entry.cursor)
            .unwrap_or(0)
    };
    if user_cursor > tail_cursor {
        return Err(bad_request(
            "cursor_out_of_range",
            "cursor exceeds available fanout history",
        ));
    }

    let entries = state
        .dm_fanout_delivery_log
        .read()
        .expect("acquire dm fanout delivery log read lock")
        .get(&identity_id)
        .cloned()
        .unwrap_or_default();

    let effective_cursor = user_cursor.max(last_cursor);

    let mut items = Vec::new();
    let mut deduped_message_ids = Vec::new();
    let mut seen_delivery_keys = HashSet::new();
    let mut scanned_cursor = last_cursor;
    for entry in &entries {
        if entry.cursor <= effective_cursor {
            continue;
        }

        scanned_cursor = entry.cursor;

        if entry.delivered_device_ids.iter().any(|id| id == &device_id) {
            continue;
        }

        let dedupe_key = (
            entry.message_id.clone(),
            entry.sender_identity_id.clone(),
            entry.source_device_id.clone(),
            ciphertext_fingerprint(&entry.ciphertext),
        );
        if !seen_delivery_keys.insert(dedupe_key) {
            deduped_message_ids.push(entry.message_id.clone());
            continue;
        }

        items.push(DmFanoutCatchUpItem {
            cursor: entry.cursor.to_string(),
            message_id: entry.message_id.clone(),
            ciphertext: entry.ciphertext.clone(),
            source_device_id: entry.source_device_id.clone(),
        });

        if items.len() >= limit as usize {
            break;
        }
    }

    deduped_message_ids.sort();
    deduped_message_ids.dedup();

    let mut committed_cursor = last_cursor;
    if scanned_cursor > last_cursor {
        if let Some(pool) = state.db_pool.as_ref() {
            committed_cursor = dm_repo::upsert_dm_fanout_device_cursor(
                pool,
                &identity_id,
                &device_id,
                scanned_cursor,
            )
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to persist fanout cursor")
            })?;
        } else {
            let mut fanout_cursors = state
                .dm_fanout_device_cursors
                .write()
                .expect("acquire dm fanout cursor write lock");
            let device_cursors = fanout_cursors.entry(identity_id.clone()).or_default();
            let current = device_cursors.get(&device_id).copied().unwrap_or(0);
            committed_cursor = current.max(scanned_cursor);
            device_cursors.insert(device_id.clone(), committed_cursor);
        }
    }

    let reason_code = if items.is_empty() {
        "fanout_catch_up_no_missed"
    } else {
        "fanout_catch_up_ok"
    };

    Ok(Json(DmFanoutCatchUpResponse {
        status: "ready".to_string(),
        reason_code: reason_code.to_string(),
        transport_profile: "direct_only".to_string(),
        device_id,
        replay_count: items.len() as u32,
        next_cursor: committed_cursor.to_string(),
        deduped_message_ids,
        items,
    }))
}

pub async fn list_dm_threads(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(query): Query<DmThreadListQuery>,
) -> ApiResult<Json<DmThreadPage>> {
    let limit = parse_limit(query.limit)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;
    let unread_only = query.unread_only.unwrap_or(false);
    let mut items = dm_history_repo::list_dm_threads_for_identity(
        pool,
        &auth.identity_id,
        query.cursor.as_deref(),
        limit,
        unread_only,
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => bad_request("cursor_invalid", "unknown dm thread cursor"),
        _ => internal_error("storage_unavailable", "failed to list dm threads"),
    })?;

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let next_cursor = if has_more {
        items.last().map(|item| item.thread_id.clone())
    } else {
        None
    };

    Ok(Json(DmThreadPage { items, next_cursor }))
}

pub async fn list_dm_thread_messages(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(thread_id): Path<String>,
    Query(query): Query<DmThreadMessageListQuery>,
) -> ApiResult<Json<DmMessagePage>> {
    let limit = parse_limit(query.limit)?;
    let cursor = parse_message_cursor(query.cursor)?;
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;

    let query_cursor = cursor.filter(|value| *value <= i64::MAX as u64);
    let mut items = dm_history_repo::list_dm_thread_messages_for_identity(
        pool,
        &auth.identity_id,
        &thread_id,
        query_cursor,
        limit,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to list dm thread messages"))?
    .ok_or({
        (
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "thread_not_found",
                message: "dm thread was not found",
            }),
        )
    })?;

    let has_more = items.len() > limit;
    if has_more {
        items.truncate(limit);
    }

    let page_items = items;
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

pub async fn mark_dm_thread_read(
    State(state): State<AppState>,
    auth: AuthSession,
    headers: HeaderMap,
    Path(thread_id): Path<String>,
    Json(body): Json<DmThreadMarkReadRequest>,
) -> ApiResult<Json<DmThreadMarkReadResponse>> {
    enforce_csrf_for_cookie_auth(&auth, &headers)?;

    if body.last_read_seq > i64::MAX as u64 {
        return Err(bad_request(
            "last_read_seq_invalid",
            "last_read_seq is out of range",
        ));
    }

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dm history requires configured database pool",
        )
    })?;

    let (new_seq, unread) = dm_history_repo::mark_dm_thread_read(
        pool,
        &auth.identity_id,
        &thread_id,
        body.last_read_seq,
    )
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to mark dm thread as read"))?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ApiError {
            code: "thread_not_found",
            message: "dm thread was not found or identity is not a participant",
        }),
    ))?;

    Ok(Json(DmThreadMarkReadResponse {
        thread_id,
        last_read_seq: new_seq,
        unread,
    }))
}

async fn consume_pairing_nonce(
    state: &AppState,
    nonce: &str,
    expires_at_epoch: i64,
    expires_at: chrono::DateTime<Utc>,
) -> ApiResult<bool> {
    if let Some(pool) = &state.db_pool {
        return dm_repo::consume_dm_pairing_nonce(pool, nonce, expires_at)
            .await
            .map_err(|_| {
                internal_error(
                    "pairing_store_unavailable",
                    "failed to persist pairing nonce replay state",
                )
            });
    }

    let now = Utc::now().timestamp();
    let mut nonce_guard = state
        .dm_pairing_nonces
        .write()
        .expect("acquire dm pairing nonce write lock");
    nonce_guard.retain(|_, expiry| *expiry >= now);
    if nonce_guard.contains_key(nonce) {
        return Ok(false);
    }
    nonce_guard.insert(nonce.to_string(), expires_at_epoch);
    Ok(true)
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

fn default_dm_policy() -> DmPolicy {
    DmPolicy {
        inbound_policy: "friends_only".to_string(),
        offline_delivery_mode: DM_OFFLINE_DELIVERY_MODE.to_string(),
    }
}

async fn current_dm_policy(state: &AppState, identity_id: &str) -> ApiResult<DmPolicy> {
    if let Some(pool) = state.db_pool.as_ref() {
        return dm_repo::get_dm_policy(pool, identity_id)
            .await
            .map_err(|_| internal_error("storage_unavailable", "failed to load dm policy"))
            .map(|policy| policy.unwrap_or_else(default_dm_policy));
    }

    Ok(state
        .dm_policies
        .read()
        .expect("acquire dm policy read lock")
        .get(identity_id)
        .cloned()
        .unwrap_or_else(default_dm_policy))
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

async fn is_friend(state: &AppState, a: &str, b: &str) -> ApiResult<bool> {
    if let Some(pool) = state.db_pool.as_ref() {
        return friends_repo::are_friends(pool, a, b).await.map_err(|_| {
            internal_error(
                "friendship_lookup_failed",
                "failed to evaluate friendship state for DM policy",
            )
        });
    }

    Ok(state
        .friend_requests
        .read()
        .expect("acquire friend request read lock")
        .values()
        .any(|record| {
            record.status == "accepted"
                && ((record.requester_identity_id == a && record.target_identity_id == b)
                    || (record.requester_identity_id == b && record.target_identity_id == a))
        }))
}

enum DmInteractionPolicyDecision {
    Allowed,
    BlockedFriendsOnly,
    BlockedSameServer,
    BlockedUnknown,
}

async fn dm_interaction_policy_decision(
    state: &AppState,
    sender_identity_id: &str,
    recipient_identity_id: &str,
) -> ApiResult<DmInteractionPolicyDecision> {
    let policy = current_dm_policy(state, recipient_identity_id).await?;

    match policy.inbound_policy.as_str() {
        "anyone" => Ok(DmInteractionPolicyDecision::Allowed),
        "friends_only" => {
            if is_friend(state, sender_identity_id, recipient_identity_id).await? {
                Ok(DmInteractionPolicyDecision::Allowed)
            } else {
                Ok(DmInteractionPolicyDecision::BlockedFriendsOnly)
            }
        }
        "same_server" => {
            if let Some(pool) = state.db_pool.as_ref() {
                if servers_repo::identities_share_server(
                    pool,
                    sender_identity_id,
                    recipient_identity_id,
                )
                .await
                .map_err(|_| {
                    internal_error(
                        "storage_unavailable",
                        "failed to evaluate shared-server DM policy",
                    )
                })? {
                    Ok(DmInteractionPolicyDecision::Allowed)
                } else {
                    Ok(DmInteractionPolicyDecision::BlockedSameServer)
                }
            } else {
                Ok(DmInteractionPolicyDecision::BlockedSameServer)
            }
        }
        _ => Ok(DmInteractionPolicyDecision::BlockedUnknown),
    }
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

    let claims_json = serde_json::to_vec(&envelope.claims)
        .map_err(|_| bad_request("pairing_invalid", "failed to encode pairing claims"))?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, key_secret.as_bytes());
    let signature_bytes = hex::decode(&envelope.signature)
        .map_err(|_| bad_request("pairing_invalid", "signature is not valid hex"))?;
    hmac::verify(&key, &claims_json, &signature_bytes).map_err(|_| {
        bad_request(
            "pairing_invalid",
            "pairing envelope signature verification failed",
        )
    })?;

    Ok(())
}

fn ciphertext_fingerprint(value: &str) -> [u8; 32] {
    let digest = digest::digest(&digest::SHA256, value.as_bytes());
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(digest.as_ref());
    bytes
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
