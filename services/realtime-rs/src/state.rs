use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub allowed_origins: Vec<String>,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub ws_connect_rate_limit: usize,
    pub ws_rate_limit_window_seconds: u64,
    pub ws_max_inbound_message_bytes: usize,
    pub ws_message_rate_limit: usize,
    pub ws_message_rate_window_seconds: u64,
    pub ws_max_connections_per_identity: usize,
    pub active_connections: Arc<Mutex<HashMap<String, usize>>>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_base_url: String,
        allowed_origins: Vec<String>,
        ws_connect_rate_limit: usize,
        ws_rate_limit_window_seconds: u64,
        ws_max_inbound_message_bytes: usize,
        ws_message_rate_limit: usize,
        ws_message_rate_window_seconds: u64,
        ws_max_connections_per_identity: usize,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .expect("build realtime HTTP client");

        Self {
            api_base_url,
            allowed_origins,
            http_client,
            rate_limiter: RateLimiter::default(),
            ws_connect_rate_limit,
            ws_rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
            active_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
