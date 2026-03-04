use std::{env, net::SocketAddr};

pub struct ApiConfig {
    pub bind_addr: SocketAddr,
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

        Self { bind_addr }
    }
}
