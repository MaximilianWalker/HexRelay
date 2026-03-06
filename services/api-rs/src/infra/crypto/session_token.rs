use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use ring::hmac;
use std::collections::BTreeMap;

const TOKEN_VERSION: &str = "v1";

pub struct SessionTokenClaims {
    pub expires_at: i64,
    pub identity_id: String,
    pub key_id: String,
    pub session_id: String,
}

pub fn issue_session_token(
    session_id: &str,
    identity_id: &str,
    expires_at_epoch: i64,
    key_id: &str,
    signing_key: &str,
) -> String {
    let payload = format!("{session_id}:{identity_id}:{expires_at_epoch}");
    let payload_b64 = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let signing_input = build_signing_input(TOKEN_VERSION, key_id, &payload_b64);
    let signature = sign_payload(signing_input.as_bytes(), signing_key);
    format!(
        "{}.{}.{}.{}",
        TOKEN_VERSION,
        key_id,
        payload_b64,
        URL_SAFE_NO_PAD.encode(signature)
    )
}

pub fn validate_session_token(
    token: &str,
    signing_keys: &BTreeMap<String, String>,
) -> Option<SessionTokenClaims> {
    let mut parts = token.split('.');
    let token_version = parts.next()?;
    let key_id = parts.next()?;
    let payload_b64 = parts.next()?;
    let signature_b64 = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    if token_version != TOKEN_VERSION {
        return None;
    }

    let signing_key = signing_keys.get(key_id)?;

    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let signature = URL_SAFE_NO_PAD.decode(signature_b64).ok()?;

    let signing_input = build_signing_input(token_version, key_id, payload_b64);
    if verify_payload(signing_input.as_bytes(), &signature, signing_key).is_err() {
        return None;
    }

    let payload = String::from_utf8(payload_bytes).ok()?;
    let mut payload_parts = payload.split(':');
    let session_id = payload_parts.next()?.to_string();
    let identity_id = payload_parts.next()?.to_string();
    let expires_at = payload_parts.next()?.parse::<i64>().ok()?;

    if payload_parts.next().is_some() {
        return None;
    }

    if Utc::now().timestamp() > expires_at {
        return None;
    }

    Some(SessionTokenClaims {
        expires_at,
        identity_id,
        key_id: key_id.to_string(),
        session_id,
    })
}

fn build_signing_input(token_version: &str, key_id: &str, payload_b64: &str) -> String {
    format!("{token_version}.{key_id}.{payload_b64}")
}

fn sign_payload(payload: &[u8], signing_key: &str) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, signing_key.as_bytes());
    hmac::sign(&key, payload).as_ref().to_vec()
}

fn verify_payload(payload: &[u8], signature: &[u8], signing_key: &str) -> Result<(), ()> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, signing_key.as_bytes());
    hmac::verify(&key, payload, signature).map_err(|_| ())
}
