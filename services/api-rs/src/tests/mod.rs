const TEST_ALLOWED_ORIGIN: &str = "http://localhost:3002";
const TEST_NODE_FINGERPRINT: &str = "hexrelay-local-fingerprint";

pub(super) use std::{
    collections::{BTreeMap, HashMap},
    env,
};

pub(super) use axum::{
    body::{to_bytes, Body},
    http::{header::SET_COOKIE, Request, StatusCode},
};
pub(super) use chrono::{Duration, Utc};
pub(super) use ring::{
    digest::{digest, SHA256},
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};
pub(super) use serde::Deserialize;
pub(super) use tower::util::ServiceExt;

pub(super) use crate::{
    app::build_app,
    config::ApiRateLimitConfig,
    db::connect_and_prepare,
    models::{AuthChallengeRecord, RegisteredIdentityKey, SessionRecord},
    session_token::issue_session_token,
    state::AppState,
};

#[derive(Deserialize)]
struct AuthChallengeResponse {
    challenge_id: String,
    nonce: String,
}

#[derive(Deserialize)]
struct AuthVerifyResponse {
    session_id: String,
}

#[derive(Deserialize)]
struct InviteCreateResponse {
    token: String,
}

#[derive(Deserialize)]
struct ServerListResponse {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ContactListResponse {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct FriendRequestRecord {
    request_id: String,
}

#[derive(Deserialize)]
struct FriendRequestPage {
    items: Vec<serde_json::Value>,
}

async fn register_identity(
    app: axum::Router,
    identity_id: &str,
    public_key: &str,
) -> (StatusCode, axum::Router) {
    let request = Request::builder()
        .method("POST")
        .uri("/v1/identity/keys/register")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{identity_id}","public_key":"{public_key}","algorithm":"ed25519"}}"#
        )))
        .expect("build register request");

    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("register response");

    (response.status(), app)
}

async fn authenticate_identity(app: axum::Router, identity_id: &str) -> (String, axum::Router) {
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let (register_status, app) = register_identity(app, identity_id, &public_key).await;
    assert_eq!(register_status, StatusCode::CREATED);

    let challenge_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/challenge")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"identity_id":"{identity_id}"}}"#)))
        .expect("build challenge request");

    let challenge_response = app
        .clone()
        .oneshot(challenge_request)
        .await
        .expect("challenge response");
    assert_eq!(challenge_response.status(), StatusCode::OK);

    let challenge_bytes = to_bytes(challenge_response.into_body(), usize::MAX)
        .await
        .expect("read challenge response body");
    let challenge: AuthChallengeResponse =
        serde_json::from_slice(&challenge_bytes).expect("decode challenge response");

    let signature = signing_key.sign(challenge.nonce.as_bytes());
    let signature_hex = hex::encode(signature.as_ref());

    let verify_request = Request::builder()
        .method("POST")
        .uri("/v1/auth/verify")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"identity_id":"{identity_id}","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id, signature_hex
        )))
        .expect("build verify request");

    let verify_response = app
        .clone()
        .oneshot(verify_request)
        .await
        .expect("verify response");
    assert_eq!(verify_response.status(), StatusCode::OK);

    let session_cookie =
        extract_cookie_from_set_cookie_headers(&verify_response, "hexrelay_session")
            .expect("verify response includes session cookie");

    let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
        .await
        .expect("read verify response body");
    let verify: AuthVerifyResponse =
        serde_json::from_slice(&verify_bytes).expect("decode verify response");

    assert!(!verify.session_id.is_empty());

    (session_cookie, app)
}

fn extract_cookie_from_set_cookie_headers(
    response: &axum::response::Response,
    cookie_name: &str,
) -> Option<String> {
    for value in response.headers().get_all(SET_COOKIE) {
        let raw = value.to_str().ok()?;
        let first_part = raw.split(';').next()?;
        if let Some((name, cookie_value)) = first_part.split_once('=') {
            if name == cookie_name {
                return Some(cookie_value.to_string());
            }
        }
    }

    None
}

async fn app_with_database() -> Option<axum::Router> {
    let database_url = match env::var("API_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            assert!(
                env::var("CI").is_err(),
                "API_DATABASE_URL must be set in CI"
            );
            return None;
        }
    };

    let pool = match connect_and_prepare(&database_url).await {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("CI").is_err(),
                "failed to prepare DB in CI: {error}"
            );
            return None;
        }
    };
    Some(build_app(AppState::default().with_db_pool(pool)))
}

fn app_with_sessions(identity_ids: &[&str]) -> (axum::Router, HashMap<String, String>) {
    let state = AppState::default();
    let mut bearer_tokens = HashMap::new();

    {
        let mut sessions = state
            .sessions
            .write()
            .expect("acquire session write lock for tests");

        for identity_id in identity_ids {
            let expires_at = Utc::now() + Duration::hours(1);
            let session_id = format!("sess-{identity_id}");

            sessions.insert(
                session_id.clone(),
                SessionRecord {
                    identity_id: (*identity_id).to_string(),
                    expires_at,
                },
            );

            bearer_tokens.insert(
                (*identity_id).to_string(),
                issue_session_token(
                    &session_id,
                    identity_id,
                    expires_at.timestamp(),
                    &state.active_signing_key_id,
                    state
                        .session_signing_keys
                        .get(&state.active_signing_key_id)
                        .expect("active signing key for tests"),
                ),
            );
        }
    }

    (build_app(state), bearer_tokens)
}
mod auth_tests;
mod db_persistence_tests;
mod directory_tests;
mod friends_tests;
mod invites_tests;
