use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, TimeZone, Utc};
use ring::hmac;
use serde::{Deserialize, Serialize};

use crate::{
    domain::dm::validation::{
        validate_connectivity_preflight, validate_dm_policy_update,
        validate_pairing_envelope_create, validate_pairing_envelope_import,
        DM_OFFLINE_DELIVERY_MODE, DM_PAIRING_ENVELOPE_VERSION,
    },
    models::{
        ApiError, DmConnectivityPreflightRequest, DmConnectivityPreflightResponse, DmMessagePage,
        DmMessageRecord, DmPairingEnvelopeCreateRequest, DmPairingEnvelopeImportRequest,
        DmPairingEnvelopeResponse, DmPairingImportResponse, DmPolicy, DmPolicyUpdate,
        DmThreadListQuery, DmThreadMessageListQuery, DmThreadPage, DmThreadSummary,
    },
    shared::errors::{bad_request, ApiResult},
    state::AppState,
    transport::http::middleware::auth::{enforce_csrf_for_cookie_auth, AuthSession},
};

const DEFAULT_PAGE_LIMIT: usize = 20;
const MAX_PAGE_LIMIT: usize = 100;

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

    Ok(Json(DmConnectivityPreflightResponse {
        status: "ready".to_string(),
        reason_code: "preflight_ok".to_string(),
        transport_profile: "direct_only".to_string(),
        remediation: vec!["Start direct DM connection.".to_string()],
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
