use std::{env, net::SocketAddr};

pub struct RealtimeConfig {
    pub bind_addr: SocketAddr,
}

impl RealtimeConfig {
    pub fn from_env() -> Self {
        let bind_raw = env::var("REALTIME_BIND").unwrap_or_else(|_| "127.0.0.1:8081".to_string());
        let bind_addr = bind_raw.parse::<SocketAddr>().unwrap_or_else(|_| {
            panic!(
                "Invalid REALTIME_BIND='{}'. Expected host:port like 127.0.0.1:8081",
                bind_raw
            )
        });

        Self { bind_addr }
    }
}
