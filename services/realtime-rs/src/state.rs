use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::transport::ws::middleware::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub allowed_origins: Vec<String>,
    pub trust_proxy_headers: bool,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub ws_connect_rate_limit: usize,
    pub ws_rate_limit_window_seconds: u64,
    pub ws_max_inbound_message_bytes: usize,
    pub ws_message_rate_limit: usize,
    pub ws_message_rate_window_seconds: u64,
    pub ws_max_connections_per_identity: usize,
    pub ws_auth_grace_seconds: u64,
    pub ws_auth_cache_max_entries: usize,
    pub active_connections: Arc<Mutex<HashMap<String, usize>>>,
    pub validated_session_cache: Arc<Mutex<HashMap<String, CachedSession>>>,
}

#[derive(Clone)]
pub struct CachedSession {
    pub identity_id: String,
    pub expires_at: DateTime<Utc>,
    pub validated_at: tokio::time::Instant,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_base_url: String,
        allowed_origins: Vec<String>,
        trust_proxy_headers: bool,
        ws_connect_rate_limit: usize,
        ws_rate_limit_window_seconds: u64,
        ws_max_inbound_message_bytes: usize,
        ws_message_rate_limit: usize,
        ws_message_rate_window_seconds: u64,
        ws_max_connections_per_identity: usize,
        ws_auth_grace_seconds: u64,
        ws_auth_cache_max_entries: usize,
    ) -> Result<Self, String> {
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|error| format!("build realtime HTTP client: {error}"))?;

        Ok(Self {
            api_base_url,
            allowed_origins,
            trust_proxy_headers,
            http_client,
            rate_limiter: RateLimiter::default(),
            ws_connect_rate_limit,
            ws_rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
            ws_auth_grace_seconds,
            ws_auth_cache_max_entries,
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            validated_session_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
