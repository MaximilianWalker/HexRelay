use realtime_rs::app::{build_app, AppState, RealtimeConfig};
use realtime_rs::domain::channels::spawn_channel_subscriber;
use realtime_rs::domain::presence::spawn_presence_subscriber;
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
        // Despite the legacy name, this startup gate now performs a broader API
        // readiness preflight, not only a /health probe.
        if let Err(err) = wait_for_api_readiness(&config.api_base_url).await {
            error!(error = %err, "realtime startup aborted due to unreachable API upstream");
            std::process::exit(1);
        }
    }

    let presence_redis_client = match config.presence_redis_url.as_ref() {
        Some(url) => match redis::Client::open(url.as_str()) {
            Ok(value) => Some(value),
            Err(err) => {
                error!(error = %err, "realtime startup aborted due to invalid presence Redis configuration");
                std::process::exit(1);
            }
        },
        None => None,
    };

    let state = match AppState::new(
        config.api_base_url.clone(),
        config.allowed_origins.clone(),
        config.channel_dispatch_internal_token.clone(),
        config.presence_watcher_internal_token.clone(),
        presence_redis_client,
        config.trust_proxy_headers,
        config.ws_connect_rate_limit,
        config.rate_limit_window_seconds,
        config.ws_max_inbound_message_bytes,
        config.ws_message_rate_limit,
        config.ws_message_rate_window_seconds,
        config.ws_max_connections_per_identity,
        config.ws_auth_grace_seconds,
        config.ws_auth_cache_max_entries,
    ) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "realtime startup aborted due to state initialization failure");
            std::process::exit(1);
        }
    };

    spawn_presence_subscriber(state.clone());
    spawn_channel_subscriber(state.clone());

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

async fn wait_for_api_readiness(api_base_url: &str) -> Result<(), String> {
    const MAX_WAIT: std::time::Duration = std::time::Duration::from_secs(15);
    const RETRY_SLEEP: std::time::Duration = std::time::Duration::from_millis(500);

    let api_base_url = api_base_url.trim_end_matches('/');
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(1))
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .map_err(|err| format!("failed to build readiness preflight client: {err}"))?;

    wait_for_api_endpoint(
        &client,
        &format!("{api_base_url}/health"),
        "api health check",
        MAX_WAIT,
        RETRY_SLEEP,
        |status| status.is_success(),
    )
    .await?;

    wait_for_api_endpoint(
        &client,
        &format!("{api_base_url}/v1/auth/sessions/validate"),
        "api session validation readiness check",
        MAX_WAIT,
        RETRY_SLEEP,
        |status| status.is_success() || status == reqwest::StatusCode::UNAUTHORIZED,
    )
    .await
}

async fn wait_for_api_endpoint<F>(
    client: &reqwest::Client,
    url: &str,
    check_name: &str,
    max_wait: std::time::Duration,
    retry_sleep: std::time::Duration,
    ready: F,
) -> Result<(), String>
where
    F: Fn(reqwest::StatusCode) -> bool,
{
    let start = std::time::Instant::now();
    while start.elapsed() < max_wait {
        match client.get(url).send().await {
            Ok(response) if ready(response.status()) => return Ok(()),
            Ok(_) | Err(_) => tokio::time::sleep(retry_sleep).await,
        }
    }

    Err(format!(
        "{check_name} failed at {url} after {:?}",
        start.elapsed()
    ))
}

#[cfg(test)]
mod tests {
    use super::wait_for_api_readiness;
    use axum::{extract::State, http::StatusCode, routing::get, Router};
    use tokio::net::TcpListener;

    #[derive(Clone)]
    struct ApiReadinessStubState {
        health_status: StatusCode,
        validate_status: StatusCode,
    }

    async fn start_api_readiness_stub(
        health_status: StatusCode,
        validate_status: StatusCode,
    ) -> String {
        async fn health(State(state): State<ApiReadinessStubState>) -> StatusCode {
            state.health_status
        }

        async fn validate(State(state): State<ApiReadinessStubState>) -> StatusCode {
            state.validate_status
        }

        let app = Router::new()
            .route("/health", get(health))
            .route("/v1/auth/sessions/validate", get(validate))
            .with_state(ApiReadinessStubState {
                health_status,
                validate_status,
            });
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind readiness stub listener");
        let address = listener.local_addr().expect("read readiness stub address");
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve readiness stub API");
        });

        format!("http://{}", address)
    }

    #[tokio::test]
    async fn startup_readiness_accepts_reachable_session_validation_surface() {
        let api_base = start_api_readiness_stub(StatusCode::OK, StatusCode::UNAUTHORIZED).await;

        assert!(wait_for_api_readiness(&api_base).await.is_ok());
    }

    #[tokio::test]
    async fn startup_readiness_rejects_unavailable_session_validation_surface() {
        let api_base =
            start_api_readiness_stub(StatusCode::OK, StatusCode::SERVICE_UNAVAILABLE).await;

        let error = wait_for_api_readiness(&api_base)
            .await
            .expect_err("session validation readiness should fail");
        assert!(error.contains("session validation readiness check"));
    }

    #[tokio::test]
    async fn startup_readiness_rejects_forbidden_session_validation_surface() {
        let api_base = start_api_readiness_stub(StatusCode::OK, StatusCode::FORBIDDEN).await;

        let error = wait_for_api_readiness(&api_base)
            .await
            .expect_err("forbidden session validation readiness should fail");
        assert!(error.contains("session validation readiness check"));
    }
}
