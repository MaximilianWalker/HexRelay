use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use ring::hmac;

pub struct SessionTokenClaims {
    pub expires_at: i64,
    pub identity_id: String,
    pub session_id: String,
}

pub fn issue_session_token(
    session_id: &str,
    identity_id: &str,
    expires_at_epoch: i64,
    signing_key: &str,
) -> String {
    let payload = format!("{session_id}:{identity_id}:{expires_at_epoch}");
    let signature = sign_payload(payload.as_bytes(), signing_key);
    format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(payload.as_bytes()),
        URL_SAFE_NO_PAD.encode(signature)
    )
}

pub fn validate_session_token(token: &str, signing_key: &str) -> Option<SessionTokenClaims> {
    let mut parts = token.split('.');
    let payload_b64 = parts.next()?;
    let signature_b64 = parts.next()?;

    if parts.next().is_some() {
        return None;
    }

    let payload_bytes = URL_SAFE_NO_PAD.decode(payload_b64).ok()?;
    let signature = URL_SAFE_NO_PAD.decode(signature_b64).ok()?;

    let expected = sign_payload(&payload_bytes, signing_key);
    if signature != expected {
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
        session_id,
    })
}

fn sign_payload(payload: &[u8], signing_key: &str) -> Vec<u8> {
    let key = hmac::Key::new(hmac::HMAC_SHA256, signing_key.as_bytes());
    hmac::sign(&key, payload).as_ref().to_vec()
}
