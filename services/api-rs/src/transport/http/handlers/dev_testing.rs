use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{
    app::state::AppState,
    infra::crypto::session_token::issue_session_token,
    models::ApiError,
    shared::errors::{bad_request, forbidden, internal_error, ApiResult},
};

const SESSION_COOKIE_NAME: &str = "hexrelay_session";
const CSRF_COOKIE_NAME: &str = "hexrelay_csrf";
const SESSION_DAYS: i64 = 30;

struct TestingProfile {
    profile_id: &'static str,
    identity_id: &'static str,
    public_key: &'static str,
    algorithm: &'static str,
    session_id: &'static str,
    purpose: &'static str,
}

const TESTING_PROFILES: &[TestingProfile] = &[
    TestingProfile {
        profile_id: "alice.primary",
        identity_id: "usr-test-alice",
        public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        algorithm: "ed25519",
        session_id: "sess-test-alice-primary",
        purpose: "Happy-path sender and primary manual-test persona",
    },
    TestingProfile {
        profile_id: "bob.primary",
        identity_id: "usr-test-bob",
        public_key: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        algorithm: "ed25519",
        session_id: "sess-test-bob-primary",
        purpose: "Happy-path peer and receiver",
    },
];

#[derive(Serialize)]
pub struct TestingProfileListResponse {
    items: Vec<TestingProfileSummary>,
}

#[derive(Serialize)]
pub struct TestingProfileSummary {
    profile_id: String,
    identity_id: String,
    purpose: String,
}

#[derive(Deserialize)]
pub struct TestingSessionCreateRequest {
    profile_id: String,
}

#[derive(Serialize)]
pub struct TestingSessionCreateResponse {
    profile_id: String,
    identity_id: String,
    session_id: String,
    expires_at: String,
    csrf_token: String,
}

pub async fn list_testing_profiles(
    State(state): State<AppState>,
) -> ApiResult<Json<TestingProfileListResponse>> {
    ensure_dev_testing_enabled(&state)?;

    Ok(Json(TestingProfileListResponse {
        items: TESTING_PROFILES
            .iter()
            .map(|profile| TestingProfileSummary {
                profile_id: profile.profile_id.to_string(),
                identity_id: profile.identity_id.to_string(),
                purpose: profile.purpose.to_string(),
            })
            .collect(),
    }))
}

pub async fn activate_testing_session(
    State(state): State<AppState>,
    Json(payload): Json<TestingSessionCreateRequest>,
) -> ApiResult<Response> {
    ensure_dev_testing_enabled(&state)?;
    let profile = TESTING_PROFILES
        .iter()
        .find(|profile| profile.profile_id == payload.profile_id)
        .ok_or_else(|| bad_request("testing_profile_unknown", "unknown testing profile"))?;

    let pool = state.db_pool.as_ref().ok_or_else(|| {
        internal_error(
            "storage_unavailable",
            "dev testing sessions require configured database pool",
        )
    })?;

    let identity_row =
        sqlx::query("SELECT public_key, algorithm FROM identity_keys WHERE identity_id = $1")
            .bind(profile.identity_id)
            .fetch_optional(pool)
            .await
            .map_err(|_| {
                internal_error("storage_unavailable", "failed to check testing identity")
            })?;
    let Some(identity_row) = identity_row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                code: "testing_identity_missing",
                message: "seed the requested testing profile before activation",
            }),
        ));
    };

    let public_key = identity_row
        .try_get::<String, _>("public_key")
        .map_err(|_| {
            internal_error("storage_unavailable", "failed to read testing identity key")
        })?;
    let algorithm = identity_row
        .try_get::<String, _>("algorithm")
        .map_err(|_| {
            internal_error(
                "storage_unavailable",
                "failed to read testing identity algorithm",
            )
        })?;
    if public_key != profile.public_key || algorithm != profile.algorithm {
        return Err((
            StatusCode::CONFLICT,
            Json(ApiError {
                code: "testing_identity_mismatch",
                message: "testing identity does not match the canonical fixture key",
            }),
        ));
    }

    let expires_at = Utc::now() + Duration::days(SESSION_DAYS);
    sqlx::query(
        "
        INSERT INTO sessions (session_id, identity_id, expires_at, revoked_at)
        VALUES ($1, $2, $3, NULL)
        ON CONFLICT (session_id) DO UPDATE
        SET identity_id = EXCLUDED.identity_id,
            expires_at = EXCLUDED.expires_at,
            revoked_at = NULL
        ",
    )
    .bind(profile.session_id)
    .bind(profile.identity_id)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| internal_error("storage_unavailable", "failed to create testing session"))?;

    let signing_key = state
        .session_signing_keys
        .get(&state.active_signing_key_id)
        .ok_or_else(|| {
            internal_error("session_key_missing", "active signing key is unavailable")
        })?;
    let session_token = issue_session_token(
        profile.session_id,
        profile.identity_id,
        expires_at.timestamp(),
        &state.active_signing_key_id,
        signing_key,
    );
    let csrf_token = random_hex(16);

    let body = TestingSessionCreateResponse {
        profile_id: profile.profile_id.to_string(),
        identity_id: profile.identity_id.to_string(),
        session_id: profile.session_id.to_string(),
        expires_at: expires_at.to_rfc3339(),
        csrf_token: csrf_token.clone(),
    };
    let mut response = Json(body).into_response();
    append_cookie(
        &mut response,
        build_cookie(
            SESSION_COOKIE_NAME,
            &session_token,
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            true,
        ),
    )?;
    append_cookie(
        &mut response,
        build_cookie(
            CSRF_COOKIE_NAME,
            &csrf_token,
            &state.session_cookie_same_site,
            state.session_cookie_secure,
            state.session_cookie_domain.as_deref(),
            false,
        ),
    )?;

    Ok(response)
}

fn ensure_dev_testing_enabled(state: &AppState) -> ApiResult<()> {
    if state.enable_dev_testing {
        return Ok(());
    }

    Err(forbidden(
        "dev_testing_disabled",
        "dev testing endpoints are disabled",
    ))
}

fn build_cookie(
    name: &str,
    value: &str,
    same_site: &str,
    secure: bool,
    domain: Option<&str>,
    http_only: bool,
) -> String {
    let mut parts = vec![
        format!("{name}={value}"),
        "Path=/".to_string(),
        format!("SameSite={same_site}"),
    ];

    if let Some(domain) = domain {
        parts.push(format!("Domain={domain}"));
    }

    if secure {
        parts.push("Secure".to_string());
    }

    if http_only {
        parts.push("HttpOnly".to_string());
    }

    parts.join("; ")
}

fn append_cookie(response: &mut Response, cookie: String) -> ApiResult<()> {
    let value = HeaderValue::from_str(&cookie).map_err(|_| {
        internal_error(
            "cookie_invalid",
            "failed to build testing session cookie header",
        )
    })?;
    response.headers_mut().append(SET_COOKIE, value);
    Ok(())
}

fn random_hex(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testing_profiles_match_dm_basic_fixture_identity_and_sessions() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/dev-seed/scenarios/dm-basic.json"
        )))
        .expect("parse dm-basic fixture");
        let identities = fixture["identities"]
            .as_array()
            .expect("fixture identities array");
        let sessions = fixture["sessions"]
            .as_array()
            .expect("fixture sessions array");

        assert_eq!(identities.len(), TESTING_PROFILES.len());
        assert_eq!(sessions.len(), TESTING_PROFILES.len());

        for profile in TESTING_PROFILES {
            let identity = identities
                .iter()
                .find(|identity| identity["profile_id"] == profile.profile_id)
                .expect("testing profile identity exists in fixture");
            assert_eq!(identity["identity_id"], profile.identity_id);
            assert_eq!(identity["public_key"], profile.public_key);
            assert_eq!(identity["algorithm"], profile.algorithm);

            let session = sessions
                .iter()
                .find(|session| session["profile_id"] == profile.profile_id)
                .expect("testing profile session exists in fixture");
            assert_eq!(session["identity_id"], profile.identity_id);
            assert_eq!(session["session_id"], profile.session_id);
        }
    }
}
