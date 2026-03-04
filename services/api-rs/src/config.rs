use std::{env, net::SocketAddr};

pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub allowed_origins: Vec<String>,
    pub database_url: String,
    pub node_fingerprint: String,
    pub session_signing_key: String,
}

impl ApiConfig {
    pub fn from_env() -> Self {
        let bind_raw = env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let bind_addr = bind_raw.parse::<SocketAddr>().unwrap_or_else(|_| {
            panic!(
                "Invalid API_BIND='{}'. Expected host:port like 127.0.0.1:8080",
                bind_raw
            )
        });

        let node_fingerprint = env::var("API_NODE_FINGERPRINT")
            .unwrap_or_else(|_| "hexrelay-local-fingerprint".to_string());
        let allowed_origins_raw = env::var("API_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let database_url = env::var("API_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay".to_string()
        });
        let session_signing_key = env::var("API_SESSION_SIGNING_KEY")
            .unwrap_or_else(|_| "hexrelay-dev-signing-key-change-me".to_string());

        let allowed_origins = allowed_origins_raw
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if node_fingerprint.trim().is_empty() {
            panic!("Invalid API_NODE_FINGERPRINT. Value must not be empty");
        }

        if database_url.trim().is_empty() {
            panic!("Invalid API_DATABASE_URL. Value must not be empty");
        }

        if allowed_origins.is_empty() {
            panic!("Invalid API_ALLOWED_ORIGINS. Must contain at least one origin");
        }

        if session_signing_key.trim().len() < 16 {
            panic!("Invalid API_SESSION_SIGNING_KEY. Must be at least 16 chars");
        }

        Self {
            bind_addr,
            allowed_origins,
            database_url,
            node_fingerprint,
            session_signing_key,
        }
    }
}
