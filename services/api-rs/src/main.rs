use api_rs::{app::build_app, config::ApiConfig, state::AppState};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    let config = ApiConfig::from_env();

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
