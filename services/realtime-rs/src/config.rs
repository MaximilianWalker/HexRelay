use std::net::IpAddr;
use std::{env, net::SocketAddr};

use reqwest::Url;

pub struct RealtimeConfig {
    pub api_base_url: String,
    pub allowed_origins: Vec<String>,
    pub bind_addr: SocketAddr,
    pub ws_connect_rate_limit: usize,
    pub rate_limit_window_seconds: u64,
    pub ws_max_inbound_message_bytes: usize,
    pub ws_message_rate_limit: usize,
    pub ws_message_rate_window_seconds: u64,
    pub ws_max_connections_per_identity: usize,
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

        let api_base_url = env::var("REALTIME_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let allowed_origins_raw = env::var("REALTIME_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let allowed_origins = allowed_origins_raw
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        let ws_connect_rate_limit = parse_usize_env("REALTIME_WS_CONNECT_RATE_LIMIT", 60);
        let rate_limit_window_seconds = parse_u64_env("REALTIME_RATE_LIMIT_WINDOW_SECONDS", 60);
        let ws_max_inbound_message_bytes =
            parse_usize_env("REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES", 16384);
        let ws_message_rate_limit = parse_usize_env("REALTIME_WS_MESSAGE_RATE_LIMIT", 120);
        let ws_message_rate_window_seconds =
            parse_u64_env("REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS", 60);
        let ws_max_connections_per_identity =
            parse_usize_env("REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY", 3);

        if api_base_url.trim().is_empty() {
            panic!("Invalid REALTIME_API_BASE_URL. Value must not be empty");
        }

        if allowed_origins.is_empty() {
            panic!("Invalid REALTIME_ALLOWED_ORIGINS. Must contain at least one origin");
        }

        let parsed_api_url = Url::parse(&api_base_url).unwrap_or_else(|_| {
            panic!(
                "Invalid REALTIME_API_BASE_URL='{}'. Expected absolute URL like http://127.0.0.1:8080",
                api_base_url
            )
        });

        let scheme = parsed_api_url.scheme();
        if scheme != "http" && scheme != "https" {
            panic!(
                "Invalid REALTIME_API_BASE_URL='{}'. Scheme must be http or https",
                api_base_url
            );
        }

        if scheme == "http" && !is_loopback_host(parsed_api_url.host_str()) {
            panic!(
                "Invalid REALTIME_API_BASE_URL='{}'. Non-loopback hosts must use https",
                api_base_url
            );
        }

        Self {
            api_base_url,
            allowed_origins,
            bind_addr,
            ws_connect_rate_limit,
            rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
        }
    }
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    match env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<usize>()
            .unwrap_or_else(|_| panic!("Invalid {}='{}'. Expected positive integer", key, value)),
        Err(_) => default,
    }
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    match env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid {}='{}'. Expected positive integer", key, value)),
        Err(_) => default,
    }
}

fn is_loopback_host(host: Option<&str>) -> bool {
    let Some(host) = host else {
        return false;
    };

    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    match host.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_loopback_host;

    #[test]
    fn detects_loopback_hosts() {
        assert!(is_loopback_host(Some("127.0.0.1")));
        assert!(is_loopback_host(Some("::1")));
        assert!(is_loopback_host(Some("localhost")));
    }

    #[test]
    fn rejects_non_loopback_hosts() {
        assert!(!is_loopback_host(Some("example.com")));
        assert!(!is_loopback_host(Some("10.0.0.5")));
        assert!(!is_loopback_host(None));
    }
}
