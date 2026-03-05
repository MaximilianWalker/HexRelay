use std::time::Duration;

use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub http_client: reqwest::Client,
    pub rate_limiter: RateLimiter,
    pub ws_connect_rate_limit: usize,
    pub ws_rate_limit_window_seconds: u64,
}

impl AppState {
    pub fn new(
        api_base_url: String,
        ws_connect_rate_limit: usize,
        ws_rate_limit_window_seconds: u64,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .expect("build realtime HTTP client");

        Self {
            api_base_url,
            http_client,
            rate_limiter: RateLimiter::default(),
            ws_connect_rate_limit,
            ws_rate_limit_window_seconds,
        }
    }
}
