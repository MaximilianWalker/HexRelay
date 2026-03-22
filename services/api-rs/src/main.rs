use api_rs::{
    app::{build_app, ApiConfig, AppState},
    db::connect_and_prepare,
};
use std::env;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "api_rs=info,tower_http=info".to_string()),
        )
        .init();

    let config = match ApiConfig::from_env() {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "api startup aborted due to invalid configuration");
            std::process::exit(1);
        }
    };

    let db_pool = match connect_and_prepare(&config.database_url).await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "api startup aborted due to database initialization failure");
            std::process::exit(1);
        }
    };

    let app = build_app(
        AppState::new(
            config.node_fingerprint.clone(),
            config.allowed_origins.clone(),
            config.active_signing_key_id.clone(),
            config.discovery_denylist.clone(),
            config.session_signing_keys.clone().into_iter().collect(),
            config.session_cookie_domain.clone(),
            config.session_cookie_secure,
            config.session_cookie_same_site.clone(),
            config.rate_limits,
            config.trust_proxy_headers,
        )
        .with_db_pool(db_pool),
    );
    let addr = config.bind_addr;
    info!(%addr, "starting api service");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "api startup aborted due to bind failure");
            std::process::exit(1);
        }
    };

    if let Err(err) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    {
        error!(error = %err, "api runtime exited with server error");
        std::process::exit(1);
    }
}
