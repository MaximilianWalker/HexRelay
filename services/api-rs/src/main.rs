use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::{env, net::SocketAddr};
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

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        service: "api-rs",
        status: "ok",
    })
}

#[tokio::main]
async fn main() {
    let config = load_config();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "api_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = Router::new().route("/health", get(health));
    let addr = config.bind_addr;
    info!(%addr, "starting api service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind api listener");
    axum::serve(listener, app)
        .await
        .expect("serve api application");
}
