use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::{env, net::SocketAddr};
use tracing::info;

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
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "api_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = Router::new().route("/health", get(health));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!(%addr, "starting api service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind api listener");
    axum::serve(listener, app)
        .await
        .expect("serve api application");
}
