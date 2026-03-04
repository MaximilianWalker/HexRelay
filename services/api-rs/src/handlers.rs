use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use rand::RngCore;
use ring::signature::{UnparsedPublicKey, ED25519};
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::{bad_request, unauthorized, ApiResult},
    models::{
        AuthChallengeRecord, AuthChallengeRequest, AuthChallengeResponse, AuthVerifyRequest,
        AuthVerifyResponse, HealthResponse, IdentityKeyRegistrationRequest, InviteCreateRequest,
        InviteCreateResponse, InviteRecord, InviteRedeemRequest, InviteRedeemResponse,
        RegisteredIdentityKey, SessionRecord, SessionRevokeRequest,
    },
    state::AppState,
    validation::{
        decode_32_bytes, decode_64_bytes, validate_auth_challenge_request,
        validate_auth_verify_request, validate_identity_registration,
        validate_invite_create_request, validate_invite_redeem_request,
        validate_session_revoke_request,
    },
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

pub async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> ApiResult<StatusCode> {
    validate_identity_registration(&payload)?;

    let mut guard = state
        .identity_keys
        .write()
        .expect("acquire identity key write lock");

    let previous = guard.insert(
        payload.identity_id,
        RegisteredIdentityKey {
            public_key: payload.public_key,
            algorithm: payload.algorithm,
        },
    );

    if let Some(existing) = previous {
        info!(
            previous_algorithm = %existing.algorithm,
            previous_public_key_len = existing.public_key.len(),
            "replaced existing identity key registration"
        );
    }

    Ok(StatusCode::CREATED)
}

pub async fn issue_auth_challenge(
    State(state): State<AppState>,
    Json(payload): Json<AuthChallengeRequest>,
) -> ApiResult<Json<AuthChallengeResponse>> {
    validate_auth_challenge_request(&payload)?;

    let identity_exists = state
        .identity_keys
        .read()
        .expect("acquire identity key read lock")
        .contains_key(&payload.identity_id);

    if !identity_exists {
        return Err(bad_request(
            "identity_invalid",
            "identity_id is not registered",
        ));
    }

    let challenge_id = Uuid::new_v4().to_string();
    let nonce = random_hex(32);
    let expires_at = (Utc::now() + Duration::minutes(5)).to_rfc3339();

    state
        .auth_challenges
        .write()
        .expect("acquire challenge write lock")
        .insert(
            challenge_id.clone(),
            AuthChallengeRecord {
                identity_id: payload.identity_id,
                nonce: nonce.clone(),
                expires_at: Utc::now() + Duration::minutes(5),
            },
        );

    Ok(Json(AuthChallengeResponse {
        challenge_id,
        nonce,
        expires_at,
    }))
}

pub async fn verify_auth_challenge(
    State(state): State<AppState>,
    Json(payload): Json<AuthVerifyRequest>,
) -> ApiResult<Json<AuthVerifyResponse>> {
    validate_auth_verify_request(&payload)?;

    let challenge_record = state
        .auth_challenges
        .read()
        .expect("acquire challenge read lock")
        .get(&payload.challenge_id)
        .cloned()
        .ok_or_else(|| unauthorized("nonce_invalid", "challenge_id is invalid"))?;

    if challenge_record.identity_id != payload.identity_id {
        return Err(unauthorized(
            "nonce_invalid",
            "challenge does not match identity",
        ));
    }

    if Utc::now() > challenge_record.expires_at {
        return Err(unauthorized("nonce_invalid", "challenge has expired"));
    }

    let key_record = state
        .identity_keys
        .read()
        .expect("acquire identity key read lock")
        .get(&payload.identity_id)
        .cloned()
        .ok_or_else(|| unauthorized("identity_invalid", "identity_id is not registered"))?;

    if key_record.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "registered algorithm must be ed25519",
        ));
    }

    let public_key = decode_32_bytes(&key_record.public_key)
        .ok_or_else(|| bad_request("public_key_invalid", "registered public key is invalid"))?;
    let signature_bytes = decode_64_bytes(&payload.signature).ok_or_else(|| {
        bad_request(
            "signature_invalid",
            "signature must be 64-byte hex or base64",
        )
    })?;

    verify_signature(
        &public_key,
        challenge_record.nonce.as_bytes(),
        &signature_bytes,
    )
    .map_err(|_| unauthorized("signature_invalid", "signature verification failed"))?;

    state
        .auth_challenges
        .write()
        .expect("acquire challenge write lock")
        .remove(&payload.challenge_id);

    let session_id = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(12);

    state
        .sessions
        .write()
        .expect("acquire session write lock")
        .insert(
            session_id.clone(),
            SessionRecord {
                identity_id: payload.identity_id,
                expires_at,
            },
        );

    Ok(Json(AuthVerifyResponse {
        session_id,
        expires_at: expires_at.to_rfc3339(),
    }))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    Json(payload): Json<SessionRevokeRequest>,
) -> ApiResult<StatusCode> {
    validate_session_revoke_request(&payload)?;

    let removed = state
        .sessions
        .write()
        .expect("acquire session write lock")
        .remove(&payload.session_id);

    if removed.is_none() {
        return Err(bad_request("session_invalid", "session_id is invalid"));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_invite(
    State(state): State<AppState>,
    Json(payload): Json<InviteCreateRequest>,
) -> ApiResult<(StatusCode, Json<InviteCreateResponse>)> {
    validate_invite_create_request(&payload)?;

    let expires_at = if let Some(raw_expires_at) = payload.expires_at.as_ref() {
        let parsed = chrono::DateTime::parse_from_rfc3339(raw_expires_at)
            .map_err(|_| bad_request("invite_invalid", "expires_at must be RFC3339 date-time"))?
            .with_timezone(&Utc);

        if parsed <= Utc::now() {
            return Err(bad_request(
                "invite_invalid",
                "expires_at must be in the future",
            ));
        }

        Some(parsed)
    } else {
        None
    };

    let max_uses = if payload.mode == "one_time" {
        match payload.max_uses {
            None => Some(1),
            Some(1) => Some(1),
            Some(_) => {
                return Err(bad_request(
                    "invite_invalid",
                    "one_time invite max_uses must be 1 if provided",
                ));
            }
        }
    } else {
        payload.max_uses
    };

    let token = Uuid::new_v4().to_string();
    state
        .invites
        .write()
        .expect("acquire invite write lock")
        .insert(
            token.clone(),
            InviteRecord {
                mode: payload.mode.clone(),
                node_fingerprint: state.node_fingerprint.clone(),
                expires_at,
                max_uses,
                uses: 0,
            },
        );

    Ok((
        StatusCode::CREATED,
        Json(InviteCreateResponse {
            token,
            mode: payload.mode,
            expires_at: expires_at.map(|value| value.to_rfc3339()),
            max_uses,
        }),
    ))
}

pub async fn redeem_invite(
    State(state): State<AppState>,
    Json(payload): Json<InviteRedeemRequest>,
) -> ApiResult<Json<InviteRedeemResponse>> {
    validate_invite_redeem_request(&payload)?;

    let mut guard = state.invites.write().expect("acquire invite write lock");
    let invite = guard
        .get_mut(&payload.token)
        .ok_or_else(|| bad_request("invite_invalid", "invite token is invalid"))?;

    if invite.node_fingerprint != payload.node_fingerprint {
        return Err(bad_request(
            "fingerprint_mismatch",
            "invite node fingerprint mismatch",
        ));
    }

    if let Some(expires_at) = invite.expires_at {
        if Utc::now() > expires_at {
            return Err(bad_request("invite_expired", "invite token is expired"));
        }
    }

    if let Some(max_uses) = invite.max_uses {
        if invite.uses >= max_uses {
            return Err(bad_request("invite_exhausted", "invite token is exhausted"));
        }
    }

    invite.uses += 1;

    Ok(Json(InviteRedeemResponse { accepted: true }))
}

fn random_hex(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn verify_signature(public_key: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> Result<(), ()> {
    let key = UnparsedPublicKey::new(&ED25519, public_key);
    key.verify(message, signature).map_err(|_| ())
}
