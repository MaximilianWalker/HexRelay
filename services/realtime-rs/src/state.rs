use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(api_base_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .expect("build realtime HTTP client");

        Self {
            api_base_url,
            http_client,
        }
    }
}
