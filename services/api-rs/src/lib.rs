pub mod app;
pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod invite_handlers;
pub mod models;
pub mod session_token;
pub mod state;
pub mod validation;

#[cfg(test)]
mod tests {
    const TEST_NODE_FINGERPRINT: &str = "hexrelay-local-fingerprint";

    use std::{collections::HashMap, env};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use chrono::{Duration, Utc};
    use ring::{
        rand::SystemRandom,
        signature::{Ed25519KeyPair, KeyPair},
    };
    use serde::Deserialize;
    use tower::util::ServiceExt;

    use crate::{
        app::build_app,
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
        access_token: String,
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

        let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
            .await
            .expect("read verify response body");
        let verify: AuthVerifyResponse =
            serde_json::from_slice(&verify_bytes).expect("decode verify response");

        (verify.access_token, app)
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
                        &state.session_signing_key,
                    ),
                );
            }
        }

        (build_app(state), bearer_tokens)
    }

    #[tokio::test]
    async fn registers_identity_key_with_hex_key() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn rejects_duplicate_identity_registration() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-dup","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"ed25519"}"#,
            ))
            .expect("build first request");
        let first_response = app.clone().oneshot(request).await.expect("first response");
        assert_eq!(first_response.status(), StatusCode::CREATED);

        let duplicate_request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-dup","public_key":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","algorithm":"ed25519"}"#,
            ))
            .expect("build duplicate request");
        let duplicate_response = app
            .oneshot(duplicate_request)
            .await
            .expect("duplicate response");
        assert_eq!(duplicate_response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn rejects_invalid_algorithm() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","algorithm":"rsa"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_invalid_public_key_format() {
        let app = build_app(AppState::default());
        let request = Request::builder()
            .method("POST")
            .uri("/v1/identity/keys/register")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-1","public_key":"not-a-real-key","algorithm":"ed25519"}"#,
            ))
            .expect("build request");

        let response = app.oneshot(request).await.expect("get response");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn issues_auth_challenge_for_registered_identity() {
        let app = build_app(AppState::default());
        let (register_status, app) = register_identity(
            app,
            "user-1",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"user-1"}"#))
            .expect("build challenge request");

        let challenge_response = app
            .oneshot(challenge_request)
            .await
            .expect("challenge response");
        assert_eq!(challenge_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_auth_challenge_for_unknown_identity() {
        let app = build_app(AppState::default());
        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"missing-user"}"#))
            .expect("build challenge request");

        let challenge_response = app
            .oneshot(challenge_request)
            .await
            .expect("challenge response");
        assert_eq!(challenge_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn verifies_auth_challenge_and_revokes_session() {
        let app = build_app(AppState::default());
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
        let public_key = hex::encode(signing_key.public_key().as_ref());

        let (register_status, app) = register_identity(app, "user-verify", &public_key).await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"user-verify"}"#))
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
                r#"{{"identity_id":"user-verify","challenge_id":"{}","signature":"{}"}}"#,
                challenge.challenge_id, signature_hex
            )))
            .expect("build verify request");

        let verify_response = app
            .clone()
            .oneshot(verify_request)
            .await
            .expect("verify response");
        assert_eq!(verify_response.status(), StatusCode::OK);

        let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
            .await
            .expect("read verify response body");
        let verify: AuthVerifyResponse =
            serde_json::from_slice(&verify_bytes).expect("decode verify response");
        assert!(!verify.session_id.is_empty());

        let revoke_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/sessions/revoke")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", verify.access_token))
            .body(Body::from(format!(
                r#"{{"session_id":"{}"}}"#,
                verify.session_id
            )))
            .expect("build revoke request");

        let revoke_response = app.oneshot(revoke_request).await.expect("revoke response");
        assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn rejects_invalid_signature_on_verify() {
        let app = build_app(AppState::default());
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
        let public_key = hex::encode(signing_key.public_key().as_ref());

        let (register_status, app) =
            register_identity(app, "user-invalid-signature", &public_key).await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"user-invalid-signature"}"#))
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

        let verify_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"identity_id":"user-invalid-signature","challenge_id":"{}","signature":"{}"}}"#,
                challenge.challenge_id,
                hex::encode([0_u8; 64])
            )))
            .expect("build verify request");

        let verify_response = app.oneshot(verify_request).await.expect("verify response");
        assert_eq!(verify_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn creates_and_redeems_multi_use_invite() {
        let (app, tokens) = app_with_sessions(&["usr-invite"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/invites")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
            .body(Body::from(r#"{"mode":"multi_use","max_uses":2}"#))
            .expect("build create invite request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create invite response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create response body");
        let created: InviteCreateResponse =
            serde_json::from_slice(&create_bytes).expect("decode invite create response");

        let redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
                created.token, TEST_NODE_FINGERPRINT
            )))
            .expect("build redeem invite request");

        let redeem_response = app
            .clone()
            .oneshot(redeem_request)
            .await
            .expect("redeem invite response");
        assert_eq!(redeem_response.status(), StatusCode::OK);

        let second_redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
                created.token, TEST_NODE_FINGERPRINT
            )))
            .expect("build second redeem invite request");

        let second_redeem_response = app
            .oneshot(second_redeem_request)
            .await
            .expect("second redeem invite response");
        assert_eq!(second_redeem_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn rejects_exhausted_one_time_invite() {
        let (app, tokens) = app_with_sessions(&["usr-invite"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/invites")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
            .body(Body::from(r#"{"mode":"one_time"}"#))
            .expect("build create invite request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create invite response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create response body");
        let created: InviteCreateResponse =
            serde_json::from_slice(&create_bytes).expect("decode invite create response");

        let first_redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
                created.token, TEST_NODE_FINGERPRINT
            )))
            .expect("build first redeem request");
        let first_redeem_response = app
            .clone()
            .oneshot(first_redeem_request)
            .await
            .expect("first redeem response");
        assert_eq!(first_redeem_response.status(), StatusCode::OK);

        let second_redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
                created.token, TEST_NODE_FINGERPRINT
            )))
            .expect("build second redeem request");
        let second_redeem_response = app
            .oneshot(second_redeem_request)
            .await
            .expect("second redeem response");
        assert_eq!(second_redeem_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_expired_invite() {
        let (app, tokens) = app_with_sessions(&["usr-invite"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/invites")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
            .body(Body::from(
                r#"{"mode":"multi_use","expires_at":"2000-01-01T00:00:00Z"}"#,
            ))
            .expect("build create invite request");

        let create_response = app
            .oneshot(create_request)
            .await
            .expect("create invite response");
        assert_eq!(create_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_fingerprint_mismatch_on_redeem() {
        let (app, tokens) = app_with_sessions(&["usr-invite"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/invites")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-invite"]))
            .body(Body::from(r#"{"mode":"multi_use","max_uses":2}"#))
            .expect("build create invite request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create invite response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create response body");
        let created: InviteCreateResponse =
            serde_json::from_slice(&create_bytes).expect("decode invite create response");

        let redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"mismatch-node"}}"#,
                created.token
            )))
            .expect("build redeem request");

        let redeem_response = app.oneshot(redeem_request).await.expect("redeem response");
        assert_eq!(redeem_response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn lists_servers_with_filters() {
        let (app, tokens) = app_with_sessions(&["usr-nora-k"]);
        let request = Request::builder()
            .method("GET")
            .uri("/v1/servers?favorites_only=true&unread_only=true")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .body(Body::empty())
            .expect("build servers list request");

        let response = app.oneshot(request).await.expect("servers response");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read servers response body");
        let payload: ServerListResponse =
            serde_json::from_slice(&body).expect("decode server list response");
        assert!(!payload.items.is_empty());
    }

    #[tokio::test]
    async fn lists_contacts_with_search_filter() {
        let (app, tokens) = app_with_sessions(&["usr-nora-k"]);
        let request = Request::builder()
            .method("GET")
            .uri("/v1/contacts?search=nora")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .body(Body::empty())
            .expect("build contacts list request");

        let response = app.oneshot(request).await.expect("contacts response");
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read contacts response body");
        let payload: ContactListResponse =
            serde_json::from_slice(&body).expect("decode contacts list response");
        assert_eq!(payload.items.len(), 1);
    }

    #[tokio::test]
    async fn creates_and_lists_friend_requests() {
        let (app, tokens) = app_with_sessions(&["usr-nora-k", "usr-jules-p"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-nora-k","target_identity_id":"usr-jules-p"}"#,
            ))
            .expect("build friend request create request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create friend request response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let list_request = Request::builder()
            .method("GET")
            .uri("/v1/friends/requests?identity_id=usr-jules-p&direction=inbound")
            .header("authorization", format!("Bearer {}", tokens["usr-jules-p"]))
            .body(Body::empty())
            .expect("build friend request list request");

        let list_response = app
            .oneshot(list_request)
            .await
            .expect("list friend request response");
        assert_eq!(list_response.status(), StatusCode::OK);

        let list_body = to_bytes(list_response.into_body(), usize::MAX)
            .await
            .expect("read friend request list body");
        let list_payload: FriendRequestPage =
            serde_json::from_slice(&list_body).expect("decode friend request page");
        assert_eq!(list_payload.items.len(), 1);
    }

    #[tokio::test]
    async fn accepts_and_declines_friend_requests() {
        let (app, tokens) = app_with_sessions(&["usr-mina-s", "usr-alex-r", "usr-nora-k"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-mina-s"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-mina-s","target_identity_id":"usr-alex-r"}"#,
            ))
            .expect("build create request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create body");
        let created: FriendRequestRecord =
            serde_json::from_slice(&create_body).expect("decode create response");

        let accept_request = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/accept",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-alex-r"]))
            .body(Body::empty())
            .expect("build accept request");
        let accept_response = app
            .clone()
            .oneshot(accept_request)
            .await
            .expect("accept response");
        assert_eq!(accept_response.status(), StatusCode::OK);

        let create_decline_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-nora-k"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-nora-k","target_identity_id":"usr-alex-r"}"#,
            ))
            .expect("build create decline request");

        let create_decline_response = app
            .clone()
            .oneshot(create_decline_request)
            .await
            .expect("create decline response");
        assert_eq!(create_decline_response.status(), StatusCode::CREATED);

        let decline_body = to_bytes(create_decline_response.into_body(), usize::MAX)
            .await
            .expect("read decline create body");
        let decline_created: FriendRequestRecord =
            serde_json::from_slice(&decline_body).expect("decode decline create");

        let decline_request = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/decline",
                decline_created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-alex-r"]))
            .body(Body::empty())
            .expect("build decline request");
        let decline_response = app
            .oneshot(decline_request)
            .await
            .expect("decline response");
        assert_eq!(decline_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn rejects_conflicting_transition_when_not_pending() {
        let (app, tokens) = app_with_sessions(&["usr-a", "usr-b"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-a"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-a","target_identity_id":"usr-b"}"#,
            ))
            .expect("build create request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create body");
        let created: FriendRequestRecord =
            serde_json::from_slice(&create_body).expect("decode create response");

        let first_accept = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/accept",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-b"]))
            .body(Body::empty())
            .expect("build first accept");
        let first_accept_response = app
            .clone()
            .oneshot(first_accept)
            .await
            .expect("first accept response");
        assert_eq!(first_accept_response.status(), StatusCode::OK);

        let idempotent_accept = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/accept",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-b"]))
            .body(Body::empty())
            .expect("build idempotent accept");
        let idempotent_accept_response = app
            .clone()
            .oneshot(idempotent_accept)
            .await
            .expect("idempotent accept response");
        assert_eq!(idempotent_accept_response.status(), StatusCode::OK);

        let conflicting_decline = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/decline",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-b"]))
            .body(Body::empty())
            .expect("build conflicting decline");
        let conflicting_decline_response = app
            .oneshot(conflicting_decline)
            .await
            .expect("conflicting decline response");
        assert_eq!(conflicting_decline_response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn requester_can_cancel_pending_friend_request() {
        let (app, tokens) = app_with_sessions(&["usr-c", "usr-d"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-c"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-c","target_identity_id":"usr-d"}"#,
            ))
            .expect("build create request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create body");
        let created: FriendRequestRecord =
            serde_json::from_slice(&create_body).expect("decode create response");

        let cancel_request = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/cancel",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-c"]))
            .body(Body::empty())
            .expect("build cancel request");

        let cancel_response = app.oneshot(cancel_request).await.expect("cancel response");
        assert_eq!(cancel_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn rejects_transition_by_wrong_actor() {
        let (app, tokens) = app_with_sessions(&["usr-e", "usr-f", "usr-g"]);

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/friends/requests")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", tokens["usr-e"]))
            .body(Body::from(
                r#"{"requester_identity_id":"usr-e","target_identity_id":"usr-f"}"#,
            ))
            .expect("build create request");

        let create_response = app
            .clone()
            .oneshot(create_request)
            .await
            .expect("create response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create body");
        let created: FriendRequestRecord =
            serde_json::from_slice(&create_body).expect("decode create response");

        let wrong_actor_accept = Request::builder()
            .method("POST")
            .uri(format!(
                "/v1/friends/requests/{}/accept",
                created.request_id
            ))
            .header("authorization", format!("Bearer {}", tokens["usr-g"]))
            .body(Body::empty())
            .expect("build wrong actor accept request");

        let wrong_actor_response = app
            .oneshot(wrong_actor_accept)
            .await
            .expect("wrong actor response");
        assert_eq!(wrong_actor_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_friend_request_without_authorization_header() {
        let app = build_app(AppState::default());

        let request = Request::builder()
            .method("GET")
            .uri("/v1/friends/requests?identity_id=usr-a")
            .body(Body::empty())
            .expect("build friend request list request");

        let response = app
            .oneshot(request)
            .await
            .expect("friend request list response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn validates_session_with_header() {
        let (app, tokens) = app_with_sessions(&["usr-session"]);

        let request = Request::builder()
            .method("GET")
            .uri("/v1/auth/sessions/validate")
            .header("authorization", format!("Bearer {}", tokens["usr-session"]))
            .body(Body::empty())
            .expect("build session validate request");

        let response = app
            .oneshot(request)
            .await
            .expect("session validate response");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn validates_and_revokes_db_backed_session() {
        let Some(app) = app_with_database().await else {
            return;
        };

        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
        let public_key = hex::encode(signing_key.public_key().as_ref());

        let (register_status, app) = register_identity(app, "db-user-verify", &public_key).await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"db-user-verify"}"#))
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
                r#"{{"identity_id":"db-user-verify","challenge_id":"{}","signature":"{}"}}"#,
                challenge.challenge_id, signature_hex
            )))
            .expect("build verify request");

        let verify_response = app
            .clone()
            .oneshot(verify_request)
            .await
            .expect("verify response");
        assert_eq!(verify_response.status(), StatusCode::OK);

        let verify_bytes = to_bytes(verify_response.into_body(), usize::MAX)
            .await
            .expect("read verify response body");
        let verify: AuthVerifyResponse =
            serde_json::from_slice(&verify_bytes).expect("decode verify response");

        let validate_request = Request::builder()
            .method("GET")
            .uri("/v1/auth/sessions/validate")
            .header("authorization", format!("Bearer {}", verify.access_token))
            .body(Body::empty())
            .expect("build validate request");

        let validate_response = app
            .clone()
            .oneshot(validate_request)
            .await
            .expect("validate response");
        assert_eq!(validate_response.status(), StatusCode::OK);

        let revoke_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/sessions/revoke")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", verify.access_token))
            .body(Body::from(format!(
                r#"{{"session_id":"{}"}}"#,
                verify.session_id
            )))
            .expect("build revoke request");

        let revoke_response = app
            .clone()
            .oneshot(revoke_request)
            .await
            .expect("revoke response");
        assert_eq!(revoke_response.status(), StatusCode::NO_CONTENT);

        let validate_after_revoke = Request::builder()
            .method("GET")
            .uri("/v1/auth/sessions/validate")
            .header("authorization", format!("Bearer {}", verify.access_token))
            .body(Body::empty())
            .expect("build post-revoke validate request");

        let validate_after_revoke_response = app
            .oneshot(validate_after_revoke)
            .await
            .expect("post-revoke validate response");
        assert_eq!(
            validate_after_revoke_response.status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn rejects_replayed_challenge_verification() {
        let app = build_app(AppState::default());
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
        let public_key = hex::encode(signing_key.public_key().as_ref());

        let (register_status, app) = register_identity(app, "user-replay", &public_key).await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"user-replay"}"#))
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

        let verify_body = format!(
            r#"{{"identity_id":"user-replay","challenge_id":"{}","signature":"{}"}}"#,
            challenge.challenge_id, signature_hex
        );

        let request_a = Request::builder()
            .method("POST")
            .uri("/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(verify_body.clone()))
            .expect("build verify request a");

        let request_b = Request::builder()
            .method("POST")
            .uri("/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(verify_body))
            .expect("build verify request b");

        let (response_a, response_b) = tokio::join!(
            app.clone().oneshot(request_a),
            app.clone().oneshot(request_b)
        );

        let status_a = response_a.expect("verify response a").status();
        let status_b = response_b.expect("verify response b").status();

        let success_count =
            usize::from(status_a == StatusCode::OK) + usize::from(status_b == StatusCode::OK);
        let unauthorized_count = usize::from(status_a == StatusCode::UNAUTHORIZED)
            + usize::from(status_b == StatusCode::UNAUTHORIZED);

        assert_eq!(success_count, 1);
        assert_eq!(unauthorized_count, 1);
    }

    #[tokio::test]
    async fn rejects_expired_challenge_verification() {
        let state = AppState::default();

        state
            .identity_keys
            .write()
            .expect("acquire identity key write lock")
            .insert(
                "user-expired".to_string(),
                RegisteredIdentityKey {
                    public_key: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .to_string(),
                    algorithm: "ed25519".to_string(),
                },
            );

        state
            .auth_challenges
            .write()
            .expect("acquire challenge write lock")
            .insert(
                "challenge-expired".to_string(),
                AuthChallengeRecord {
                    identity_id: "user-expired".to_string(),
                    nonce: "deadbeef".to_string(),
                    expires_at: Utc::now() - Duration::seconds(1),
                },
            );

        let app = build_app(state);

        let verify_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"identity_id":"user-expired","challenge_id":"challenge-expired","signature":"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}"#,
            ))
            .expect("build verify request");

        let response = app.oneshot(verify_request).await.expect("verify response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn redeems_db_invite_after_app_restart() {
        let Some(app) = app_with_database().await else {
            return;
        };

        let (access_token, app) = authenticate_identity(app, "db-user-invite").await;

        let create_request = Request::builder()
            .method("POST")
            .uri("/v1/invites")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {access_token}"))
            .body(Body::from(r#"{"mode":"one_time"}"#))
            .expect("build create invite request");

        let create_response = app
            .oneshot(create_request)
            .await
            .expect("create invite response");
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let create_bytes = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .expect("read create invite body");
        let created: InviteCreateResponse =
            serde_json::from_slice(&create_bytes).expect("decode invite create response");

        let Some(app_after_restart) = app_with_database().await else {
            return;
        };

        let redeem_request = Request::builder()
            .method("POST")
            .uri("/v1/invites/redeem")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"token":"{}","node_fingerprint":"{}"}}"#,
                created.token, TEST_NODE_FINGERPRINT
            )))
            .expect("build redeem invite request");

        let redeem_response = app_after_restart
            .oneshot(redeem_request)
            .await
            .expect("redeem invite response");
        assert_eq!(redeem_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn verifies_db_challenge_after_app_restart() {
        let Some(app) = app_with_database().await else {
            return;
        };

        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("generate keypair");
        let signing_key = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("decode keypair");
        let public_key = hex::encode(signing_key.public_key().as_ref());

        let (register_status, app) = register_identity(app, "db-user-restart", &public_key).await;
        assert_eq!(register_status, StatusCode::CREATED);

        let challenge_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/challenge")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"identity_id":"db-user-restart"}"#))
            .expect("build challenge request");

        let challenge_response = app
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

        let Some(app_after_restart) = app_with_database().await else {
            return;
        };

        let verify_request = Request::builder()
            .method("POST")
            .uri("/v1/auth/verify")
            .header("content-type", "application/json")
            .body(Body::from(format!(
                r#"{{"identity_id":"db-user-restart","challenge_id":"{}","signature":"{}"}}"#,
                challenge.challenge_id, signature_hex
            )))
            .expect("build verify request");

        let verify_response = app_after_restart
            .oneshot(verify_request)
            .await
            .expect("verify response");
        assert_eq!(verify_response.status(), StatusCode::OK);
    }
}
