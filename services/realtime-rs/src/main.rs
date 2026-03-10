use realtime_rs::app::{build_app, AppState, RealtimeConfig};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    let config = RealtimeConfig::from_env().expect("load realtime configuration from environment");

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "realtime_rs=info,tower_http=info".to_string()),
        )
        .init();

    if config.require_api_health_on_start {
        if let Err(err) = wait_for_api_health(&config.api_base_url).await {
            error!(error = %err, "realtime startup aborted due to unreachable API upstream");
            std::process::exit(1);
        }
    }

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

async fn wait_for_api_health(api_base_url: &str) -> Result<(), String> {
    let url = format!("{}/health", api_base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|err| format!("failed to build health preflight client: {err}"))?;

    for _ in 0..30 {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(_) | Err(_) => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }

    Err(format!("api health check failed at {url} after 15s"))
}
