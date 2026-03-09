use realtime_rs::app::{build_app, AppState, RealtimeConfig};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    let config =
        RealtimeConfig::from_env().expect("load realtime configuration from environment");

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "realtime_rs=info,tower_http=info".to_string()),
        )
        .init();

    let app = build_app(AppState::new(
        config.api_base_url.clone(),
        config.allowed_origins.clone(),
        config.ws_connect_rate_limit,
        config.rate_limit_window_seconds,
        config.ws_max_inbound_message_bytes,
        config.ws_message_rate_limit,
        config.ws_message_rate_window_seconds,
        config.ws_max_connections_per_identity,
    ));

    let addr = config.bind_addr;
    info!(%addr, "starting realtime service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind realtime listener");
    axum::serve(listener, app)
        .await
        .expect("serve realtime application");
}
