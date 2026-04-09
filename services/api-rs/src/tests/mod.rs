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
pub(super) use uuid::Uuid;

pub(super) use crate::infra::db::repos::dm_history_repo;
pub(super) use crate::infra::db::repos::server_channels_repo;
pub(super) use crate::infra::db::repos::servers_repo;
pub(super) use crate::{
    app::build_app,
    config::ApiRateLimitConfig,
    db::connect_and_prepare,
    infra::{crypto::session_token::issue_session_token, db::repos::auth_repo},
    models::{AuthChallengeRecord, RegisteredIdentityKey, SessionRecord},
    state::AppState,
};

type SeedDmMessage<'a> = (&'a str, &'a str, u64, &'a str, &'a str, Option<&'a str>);
type SeedServerChannelMessage<'a> = (
    &'a str,
    &'a str,
    u64,
    &'a str,
    Option<&'a str>,
    &'a [&'a str],
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
);

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
    invite_id: String,
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
    requester_identity_id: String,
    target_identity_id: String,
    status: String,
}

#[derive(Deserialize)]
struct FriendRequestPage {
    items: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct IdentityBootstrapBundle {
    identity_id: String,
    public_key: String,
    algorithm: String,
    endpoint_cards: Vec<serde_json::Value>,
    devices: Vec<serde_json::Value>,
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

async fn register_identity_expect_success(
    app: axum::Router,
    identity_id: &str,
    public_key: &str,
) -> axum::Router {
    let (status, app) = register_identity(app, identity_id, public_key).await;
    assert_eq!(status, StatusCode::CREATED);
    app
}

fn test_state_with_public_identity_registration() -> AppState {
    AppState::default().with_public_identity_registration(true)
}

async fn authenticate_identity(app: axum::Router, identity_id: &str) -> (String, axum::Router) {
    let rng = SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
    let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
    let public_key = hex::encode(signing_key.public_key().as_ref());

    let app = register_identity_expect_success(app, identity_id, &public_key).await;

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

async fn prepared_database_pool() -> Option<sqlx::PgPool> {
    let database_url = match env::var("API_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            assert!(
                env::var("CI").is_err(),
                "API_DATABASE_URL must be set in CI"
            );
            eprintln!(
                "[api-rs test] skipping DB-backed tests because API_DATABASE_URL is not configured"
            );
            return None;
        }
    };

    Some(match connect_and_prepare(&database_url).await {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("CI").is_err(),
                "failed to prepare DB in CI: {error}"
            );
            eprintln!(
                "[api-rs test] skipping DB-backed tests because database is unavailable at {database_url}: {error}"
            );
            return None;
        }
    })
}

async fn prepared_presence_redis_client() -> Option<redis::Client> {
    let redis_url = match env::var("API_PRESENCE_REDIS_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "API_PRESENCE_REDIS_URL must be set in GitHub Actions"
            );
            eprintln!(
                "[api-rs test] skipping Redis-backed presence test because API_PRESENCE_REDIS_URL is not configured"
            );
            return None;
        }
    };

    let client = match redis::Client::open(redis_url.as_str()) {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "invalid Redis URL in GitHub Actions: {error}"
            );
            eprintln!(
                "[api-rs test] skipping Redis-backed presence test because Redis URL is invalid: {error}"
            );
            return None;
        }
    };

    let mut connection = match client.get_multiplexed_tokio_connection().await {
        Ok(value) => value,
        Err(error) => {
            assert!(
                env::var("GITHUB_ACTIONS").is_err(),
                "failed to connect to Redis in GitHub Actions: {error}"
            );
            eprintln!(
                "[api-rs test] skipping Redis-backed presence test because Redis is unavailable: {error}"
            );
            return None;
        }
    };

    let _: String = redis::cmd("PING")
        .query_async(&mut connection)
        .await
        .expect("ping Redis");

    Some(client)
}

async fn app_with_database() -> Option<axum::Router> {
    let pool = prepared_database_pool().await?;
    Some(build_app(
        test_state_with_public_identity_registration().with_db_pool(pool),
    ))
}

async fn app_with_database_and_sessions(
    identity_ids: &[&str],
) -> Option<(axum::Router, HashMap<String, String>, sqlx::PgPool)> {
    let pool = prepared_database_pool().await?;
    let state = test_state_with_public_identity_registration().with_db_pool(pool.clone());
    let mut tokens = HashMap::new();

    for identity_id in identity_ids {
        tokens.insert(
            (*identity_id).to_string(),
            issue_db_session_cookie(&pool, &state, identity_id).await,
        );
    }

    Some((build_app(state), tokens, pool))
}

fn unique_identity(prefix: &str) -> String {
    format!("{}-{}", prefix, Uuid::new_v4().simple())
}

fn test_identity_public_key(identity_id: &str) -> String {
    hex::encode(digest(&SHA256, identity_id.as_bytes()).as_ref())
}

async fn ensure_db_identity_key(pool: &sqlx::PgPool, identity_id: &str) {
    auth_repo::insert_identity_key(
        pool,
        identity_id,
        &test_identity_public_key(identity_id),
        "ed25519",
    )
    .await
    .expect("ensure test identity key");
}

async fn issue_db_session_cookie(
    pool: &sqlx::PgPool,
    state: &AppState,
    identity_id: &str,
) -> String {
    ensure_db_identity_key(pool, identity_id).await;

    let session_id = format!("sess-{}", Uuid::new_v4().simple());
    let expires_at = Utc::now() + Duration::hours(1);
    auth_repo::insert_session(pool, &session_id, identity_id, expires_at)
        .await
        .expect("insert test session");

    issue_session_token(
        &session_id,
        identity_id,
        expires_at.timestamp(),
        &state.active_signing_key_id,
        state
            .session_signing_keys
            .get(&state.active_signing_key_id)
            .expect("active signing key for test db session"),
    )
}

async fn seed_server_membership(
    pool: &sqlx::PgPool,
    server_id: &str,
    name: &str,
    identity_id: &str,
    favorite: bool,
    muted: bool,
    unread_count: i32,
) {
    ensure_db_identity_key(pool, identity_id).await;

    servers_repo::insert_server(pool, servers_repo::ServerInsertParams { server_id, name })
        .await
        .expect("insert server");

    servers_repo::insert_server_membership(
        pool,
        servers_repo::ServerMembershipInsertParams {
            server_id,
            identity_id,
            favorite,
            muted,
            unread_count,
        },
    )
    .await
    .expect("insert server membership");
}

async fn seed_dm_thread(
    pool: &sqlx::PgPool,
    thread_id: &str,
    kind: &str,
    title: &str,
    participants: &[(&str, u64)],
    messages: &[SeedDmMessage<'_>],
) {
    for (identity_id, _) in participants {
        ensure_db_identity_key(pool, identity_id).await;
    }

    for (_, author_id, _, _, _, _) in messages {
        ensure_db_identity_key(pool, author_id).await;
    }

    dm_history_repo::insert_dm_thread(
        pool,
        dm_history_repo::DmThreadInsertParams {
            thread_id,
            kind,
            title,
        },
    )
    .await
    .expect("insert dm thread");

    for (identity_id, last_read_seq) in participants {
        dm_history_repo::insert_dm_thread_participant(
            pool,
            dm_history_repo::DmThreadParticipantInsertParams {
                thread_id,
                identity_id,
                last_read_seq: *last_read_seq,
            },
        )
        .await
        .expect("insert dm thread participant");
    }

    for (message_id, author_id, seq, ciphertext, created_at, edited_at) in messages {
        dm_history_repo::insert_dm_message(
            pool,
            dm_history_repo::DmMessageInsertParams {
                message_id,
                thread_id,
                author_id,
                seq: *seq,
                ciphertext,
                created_at,
                edited_at: *edited_at,
            },
        )
        .await
        .expect("insert dm message");
    }
}

async fn seed_server_channel(
    pool: &sqlx::PgPool,
    server_id: &str,
    server_name: &str,
    channel_id: &str,
    channel_name: &str,
    member_identity_ids: &[&str],
    messages: &[SeedServerChannelMessage<'_>],
) {
    for identity_id in member_identity_ids {
        seed_server_membership(pool, server_id, server_name, identity_id, false, false, 0).await;
    }

    server_channels_repo::insert_server_channel(
        pool,
        server_channels_repo::ServerChannelInsertParams {
            channel_id,
            server_id,
            name: channel_name,
            kind: "text",
        },
    )
    .await
    .expect("insert server channel");

    for (
        message_id,
        author_id,
        channel_seq,
        content,
        reply_to_message_id,
        mention_identity_ids,
        created_at,
        edited_at,
        deleted_at,
    ) in messages
    {
        assert!(
            member_identity_ids.contains(author_id),
            "seed_server_channel requires message authors to be members of the seeded server"
        );
        ensure_db_identity_key(pool, author_id).await;
        for mentioned_identity_id in *mention_identity_ids {
            ensure_db_identity_key(pool, mentioned_identity_id).await;
        }

        server_channels_repo::insert_server_channel_message(
            pool,
            server_channels_repo::ServerChannelMessageInsertParams {
                message_id,
                channel_id,
                author_id,
                channel_seq: *channel_seq,
                content,
                reply_to_message_id: *reply_to_message_id,
                created_at,
                edited_at: *edited_at,
                deleted_at: *deleted_at,
            },
            mention_identity_ids,
        )
        .await
        .expect("insert server channel message");
    }
}

fn app_with_sessions(identity_ids: &[&str]) -> (axum::Router, HashMap<String, String>) {
    let state = test_state_with_public_identity_registration();
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
mod integration;
