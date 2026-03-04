use realtime_rs::{app::build_app, config::RealtimeConfig};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    let config = RealtimeConfig::from_env();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "realtime_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = build_app();

    let addr = config.bind_addr;
    info!(%addr, "starting realtime service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind realtime listener");
    axum::serve(listener, app)
        .await
        .expect("serve realtime application");
}
