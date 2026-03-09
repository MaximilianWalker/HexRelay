use api_rs::{
    app::{build_app, ApiConfig, AppState},
    db::connect_and_prepare,
};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() {
    let config = ApiConfig::from_env().expect("load API configuration from environment");

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "api_rs=info,tower_http=info".to_string()),
        )
        .init();

    let db_pool = connect_and_prepare(&config.database_url)
        .await
        .expect("connect and prepare API database");

    let app = build_app(
        AppState::new(
            config.node_fingerprint.clone(),
            config.allowed_origins.clone(),
            config.active_signing_key_id.clone(),
            config.session_signing_keys.clone().into_iter().collect(),
            config.session_cookie_domain.clone(),
            config.session_cookie_secure,
            config.session_cookie_same_site.clone(),
            config.rate_limits,
        )
        .with_db_pool(db_pool),
    );
    let addr = config.bind_addr;
    info!(%addr, "starting api service");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind api listener");
    axum::serve(listener, app)
        .await
        .expect("serve api application");
}
