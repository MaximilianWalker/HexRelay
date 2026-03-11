use realtime_rs::app::{build_app, AppState, RealtimeConfig};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "realtime_rs=info,tower_http=info".to_string()),
        )
        .init();

    let config = match RealtimeConfig::from_env() {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "realtime startup aborted due to invalid configuration");
            std::process::exit(1);
        }
    };

    if config.require_api_health_on_start {
        if let Err(err) = wait_for_api_health(&config.api_base_url).await {
            error!(error = %err, "realtime startup aborted due to unreachable API upstream");
            std::process::exit(1);
        }
    }

    let state = match AppState::new(
        config.api_base_url.clone(),
        config.allowed_origins.clone(),
        config.trust_proxy_headers,
        config.ws_connect_rate_limit,
        config.rate_limit_window_seconds,
        config.ws_max_inbound_message_bytes,
        config.ws_message_rate_limit,
        config.ws_message_rate_window_seconds,
        config.ws_max_connections_per_identity,
    ) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "realtime startup aborted due to state initialization failure");
            std::process::exit(1);
        }
    };

    let app = build_app(state);

    let addr = config.bind_addr;
    info!(%addr, "starting realtime service");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "realtime startup aborted due to bind failure");
            std::process::exit(1);
        }
    };

    if let Err(err) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    {
        error!(error = %err, "realtime runtime exited with server error");
        std::process::exit(1);
    }
}

async fn wait_for_api_health(api_base_url: &str) -> Result<(), String> {
    const MAX_WAIT: std::time::Duration = std::time::Duration::from_secs(15);
    const RETRY_SLEEP: std::time::Duration = std::time::Duration::from_millis(500);

    let url = format!("{}/health", api_base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(1))
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .map_err(|err| format!("failed to build health preflight client: {err}"))?;

    let start = std::time::Instant::now();
    while start.elapsed() < MAX_WAIT {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(_) | Err(_) => tokio::time::sleep(RETRY_SLEEP).await,
        }
    }

    Err(format!(
        "api health check failed at {url} after {:?}",
        start.elapsed()
    ))
}
