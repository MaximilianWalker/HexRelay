#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(api_base_url: String) -> Self {
        Self {
            api_base_url,
            http_client: reqwest::Client::new(),
        }
    }
}
