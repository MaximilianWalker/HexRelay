use std::{env, net::SocketAddr};

pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub node_fingerprint: String,
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

        if node_fingerprint.trim().is_empty() {
            panic!("Invalid API_NODE_FINGERPRINT. Value must not be empty");
        }

        Self {
            bind_addr,
            node_fingerprint,
        }
    }
}
