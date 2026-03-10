use std::net::IpAddr;
use std::{env, net::SocketAddr};

use reqwest::Url;

pub struct RealtimeConfig {
    pub api_base_url: String,
    pub require_api_health_on_start: bool,
    pub trust_proxy_headers: bool,
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
    pub fn from_env() -> Result<Self, String> {
        let bind_raw = env::var("REALTIME_BIND").unwrap_or_else(|_| "127.0.0.1:8081".to_string());
        let bind_addr = bind_raw.parse::<SocketAddr>().map_err(|_| {
            format!(
                "Invalid REALTIME_BIND='{}'. Expected host:port like 127.0.0.1:8081",
                bind_raw
            )
        })?;

        let api_base_url = env::var("REALTIME_API_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
        let allowed_origins_raw = env::var("REALTIME_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let allowed_origins = allowed_origins_raw
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        let require_api_health_on_start =
            parse_bool_env("REALTIME_REQUIRE_API_HEALTH_ON_START", true)?;
        let trust_proxy_headers = parse_bool_env("REALTIME_TRUST_PROXY_HEADERS", false)?;
        let ws_connect_rate_limit = parse_usize_env("REALTIME_WS_CONNECT_RATE_LIMIT", 60)?;
        let rate_limit_window_seconds = parse_u64_env("REALTIME_RATE_LIMIT_WINDOW_SECONDS", 60)?;
        let ws_max_inbound_message_bytes =
            parse_usize_env("REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES", 16384)?;
        let ws_message_rate_limit = parse_usize_env("REALTIME_WS_MESSAGE_RATE_LIMIT", 120)?;
        let ws_message_rate_window_seconds =
            parse_u64_env("REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS", 60)?;
        let ws_max_connections_per_identity =
            parse_usize_env("REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY", 3)?;

        if api_base_url.trim().is_empty() {
            return Err("Invalid REALTIME_API_BASE_URL. Value must not be empty".to_string());
        }

        if allowed_origins.is_empty() {
            return Err(
                "Invalid REALTIME_ALLOWED_ORIGINS. Must contain at least one origin".to_string(),
            );
        }

        if ws_connect_rate_limit == 0 {
            return Err(
                "Invalid REALTIME_WS_CONNECT_RATE_LIMIT. Expected integer greater than 0"
                    .to_string(),
            );
        }

        if rate_limit_window_seconds == 0 {
            return Err(
                "Invalid REALTIME_RATE_LIMIT_WINDOW_SECONDS. Expected integer greater than 0"
                    .to_string(),
            );
        }

        if ws_message_rate_limit == 0 {
            return Err(
                "Invalid REALTIME_WS_MESSAGE_RATE_LIMIT. Expected integer greater than 0"
                    .to_string(),
            );
        }

        if ws_message_rate_window_seconds == 0 {
            return Err(
                "Invalid REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS. Expected integer greater than 0"
                    .to_string(),
            );
        }

        if ws_max_connections_per_identity == 0 {
            return Err(
                "Invalid REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY. Expected integer greater than 0"
                    .to_string(),
            );
        }

        if ws_max_inbound_message_bytes < 256 {
            return Err(
                "Invalid REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES. Expected integer >= 256"
                    .to_string(),
            );
        }

        let parsed_api_url = Url::parse(&api_base_url).map_err(|_| {
            format!(
                "Invalid REALTIME_API_BASE_URL='{}'. Expected absolute URL like http://127.0.0.1:8080",
                api_base_url
            )
        })?;

        let scheme = parsed_api_url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(format!(
                "Invalid REALTIME_API_BASE_URL='{}'. Scheme must be http or https",
                api_base_url
            ));
        }

        if scheme == "http" && !is_loopback_host(parsed_api_url.host_str()) {
            return Err(format!(
                "Invalid REALTIME_API_BASE_URL='{}'. Non-loopback hosts must use https",
                api_base_url
            ));
        }

        Ok(Self {
            api_base_url,
            require_api_health_on_start,
            trust_proxy_headers,
            allowed_origins,
            bind_addr,
            ws_connect_rate_limit,
            rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
        })
    }
}

fn parse_bool_env(key: &str, default: bool) -> Result<bool, String> {
    match env::var(key) {
        Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Ok(true),
            "0" | "false" | "no" | "off" => Ok(false),
            _ => Err(format!("Invalid {}='{}'. Expected boolean", key, value)),
        },
        Err(_) => Ok(default),
    }
}

fn parse_usize_env(key: &str, default: usize) -> Result<usize, String> {
    match env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<usize>()
            .map_err(|_| format!("Invalid {}='{}'. Expected positive integer", key, value)),
        Err(_) => Ok(default),
    }
}

fn parse_u64_env(key: &str, default: u64) -> Result<u64, String> {
    match env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .map_err(|_| format!("Invalid {}='{}'. Expected positive integer", key, value)),
        Err(_) => Ok(default),
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
    use super::{is_loopback_host, RealtimeConfig};
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_realtime_env<F>(pairs: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().expect("acquire env test lock");
        let previous = pairs
            .iter()
            .map(|(key, _)| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for (key, value) in pairs {
            match value {
                Some(value) => unsafe {
                    std::env::set_var(key, value);
                },
                None => unsafe {
                    std::env::remove_var(key);
                },
            }
        }

        f();

        for (key, value) in previous {
            if let Some(value) = value {
                unsafe {
                    std::env::set_var(key, value);
                }
            } else {
                unsafe {
                    std::env::remove_var(key);
                }
            }
        }
    }

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

    #[test]
    fn parses_proxy_header_trust_flag() {
        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_TRUST_PROXY_HEADERS", Some("true")),
                ("REALTIME_REQUIRE_API_HEALTH_ON_START", Some("false")),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
                assert!(config.trust_proxy_headers);
                assert!(!config.require_api_health_on_start);
            },
        );
    }

    #[test]
    fn rejects_zero_or_too_small_limits() {
        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_WS_MESSAGE_RATE_LIMIT", Some("0")),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("zero message rate must fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_WS_MESSAGE_RATE_LIMIT"));
            },
        );

        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES", Some("128")),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("small message payload limit must fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES"));
            },
        );
    }
}
