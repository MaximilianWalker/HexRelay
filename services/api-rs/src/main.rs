use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tracing::info;

struct ApiConfig {
    bind_addr: SocketAddr,
}

fn load_config() -> ApiConfig {
    let bind_raw = env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let bind_addr = bind_raw.parse::<SocketAddr>().unwrap_or_else(|_| {
        panic!(
            "Invalid API_BIND='{}'. Expected host:port like 127.0.0.1:8080",
            bind_raw
        )
    });

    ApiConfig { bind_addr }
}

#[derive(Serialize)]
struct HealthResponse {
    service: &'static str,
    status: &'static str,
}

#[derive(Clone, Default)]
struct AppState {
    identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
}

#[derive(Clone)]
struct RegisteredIdentityKey {
    public_key: String,
    algorithm: String,
}

#[derive(Deserialize)]
struct IdentityKeyRegistrationRequest {
    identity_id: String,
    public_key: String,
    algorithm: String,
}

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

async fn register_identity_key(
    State(state): State<AppState>,
    Json(payload): Json<IdentityKeyRegistrationRequest>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    if payload.algorithm != "ed25519" {
        return Err(bad_request(
            "algorithm_invalid",
            "algorithm must be ed25519",
        ));
    }

    if payload.identity_id.trim().is_empty() {
        return Err(bad_request(
            "identity_invalid",
            "identity_id must not be empty",
        ));
    }

    if !is_valid_public_key(&payload.public_key) {
        return Err(bad_request(
            "public_key_invalid",
            "public_key must be 32-byte ed25519 key in hex or base64",
        ));
    }

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

fn bad_request(code: &'static str, message: &'static str) -> (StatusCode, Json<ApiError>) {
    (StatusCode::BAD_REQUEST, Json(ApiError { code, message }))
}

fn is_valid_public_key(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    BASE64
        .decode(trimmed)
        .map(|decoded| decoded.len() == 32)
        .unwrap_or(false)
}

fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/identity/keys/register", post(register_identity_key))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let config = load_config();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "api_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = build_app(AppState::default());
    let addr = config.bind_addr;
    info!(%addr, "starting api service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind api listener");
    axum::serve(listener, app)
        .await
        .expect("serve api application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::util::ServiceExt;

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
}
