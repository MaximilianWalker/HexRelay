use std::{
    collections::HashMap,
    env,
    net::{IpAddr, SocketAddr},
};

use reqwest::Url;

#[derive(Clone)]
pub struct ApiRateLimitConfig {
    pub auth_challenge_per_window: usize,
    pub auth_verify_per_window: usize,
    pub discovery_query_per_window: usize,
    pub invite_create_per_window: usize,
    pub invite_redeem_per_window: usize,
    pub window_seconds: u64,
}

pub struct ApiConfig {
    pub allow_public_identity_registration: bool,
    pub bind_addr: SocketAddr,
    pub channel_dispatch_internal_token: String,
    pub allowed_origins: Vec<String>,
    pub database_url: String,
    pub node_fingerprint: String,
    pub discovery_denylist: Vec<String>,
    pub presence_watcher_internal_token: String,
    pub presence_redis_url: Option<String>,
    pub realtime_base_url: String,
    pub session_signing_keys: HashMap<String, String>,
    pub active_signing_key_id: String,
    pub session_cookie_domain: Option<String>,
    pub session_cookie_secure: bool,
    pub session_cookie_same_site: String,
    pub trust_proxy_headers: bool,
    pub rate_limits: ApiRateLimitConfig,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, String> {
        const DEFAULT_NODE_FINGERPRINT: &str = "hexrelay-local-fingerprint";
        const DEFAULT_DATABASE_URL: &str =
            "postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay";
        const DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN: &str =
            "hexrelay-dev-channel-dispatch-token-change-me";
        const DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN: &str =
            "hexrelay-dev-presence-watcher-token-change-me";

        let bind_raw = env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let bind_addr = bind_raw.parse::<SocketAddr>().map_err(|_| {
            format!(
                "Invalid API_BIND='{}'. Expected host:port like 127.0.0.1:8080",
                bind_raw
            )
        })?;

        let environment = env::var("API_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .trim()
            .to_ascii_lowercase();
        if environment != "development" && environment != "production" {
            return Err(
                "Invalid API_ENVIRONMENT. Expected 'development' or 'production'".to_string(),
            );
        }

        let allow_public_identity_registration =
            parse_bool_env("API_ALLOW_PUBLIC_IDENTITY_REGISTRATION", false)?;
        let node_fingerprint = env::var("API_NODE_FINGERPRINT")
            .unwrap_or_else(|_| DEFAULT_NODE_FINGERPRINT.to_string());
        let allowed_origins_raw = env::var("API_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let database_url =
            env::var("API_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let discovery_denylist = parse_csv_env("API_DISCOVERY_DENYLIST");
        let channel_dispatch_internal_token = env::var("API_CHANNEL_DISPATCH_INTERNAL_TOKEN")
            .unwrap_or_else(|_| DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN.to_string());
        let presence_watcher_internal_token = env::var("API_PRESENCE_WATCHER_INTERNAL_TOKEN")
            .unwrap_or_else(|_| DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN.to_string());
        let presence_redis_url = env::var("API_PRESENCE_REDIS_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let realtime_base_url = env::var("API_REALTIME_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
        let (active_signing_key_id, session_signing_keys) = parse_session_signing_keys()?;
        let session_cookie_domain = env::var("API_SESSION_COOKIE_DOMAIN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let session_cookie_secure = parse_bool_env("API_SESSION_COOKIE_SECURE", false)?;
        let trust_proxy_headers = parse_bool_env("API_TRUST_PROXY_HEADERS", false)?;
        let session_cookie_same_site =
            env::var("API_SESSION_COOKIE_SAME_SITE").unwrap_or_else(|_| "Lax".to_string());
        let rate_limits = ApiRateLimitConfig {
            auth_challenge_per_window: parse_positive_usize_env(
                "API_AUTH_CHALLENGE_RATE_LIMIT",
                30,
            )?,
            auth_verify_per_window: parse_positive_usize_env("API_AUTH_VERIFY_RATE_LIMIT", 30)?,
            discovery_query_per_window: parse_positive_usize_env(
                "API_DISCOVERY_QUERY_RATE_LIMIT",
                30,
            )?,
            invite_create_per_window: parse_positive_usize_env("API_INVITE_CREATE_RATE_LIMIT", 20)?,
            invite_redeem_per_window: parse_positive_usize_env("API_INVITE_REDEEM_RATE_LIMIT", 40)?,
            window_seconds: parse_u64_env("API_RATE_LIMIT_WINDOW_SECONDS", 60)?,
        };

        let allowed_origins = allowed_origins_raw
            .split(',')
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();

        if node_fingerprint.trim().is_empty() {
            return Err("Invalid API_NODE_FINGERPRINT. Value must not be empty".to_string());
        }

        if database_url.trim().is_empty() {
            return Err("Invalid API_DATABASE_URL. Value must not be empty".to_string());
        }

        if channel_dispatch_internal_token.trim().len() < 16 {
            return Err(
                "Invalid API_CHANNEL_DISPATCH_INTERNAL_TOKEN. Expected at least 16 characters"
                    .to_string(),
            );
        }

        if presence_watcher_internal_token.trim().len() < 16 {
            return Err(
                "Invalid API_PRESENCE_WATCHER_INTERNAL_TOKEN. Expected at least 16 characters"
                    .to_string(),
            );
        }

        if realtime_base_url.trim().is_empty() {
            return Err("Invalid API_REALTIME_BASE_URL. Value must not be empty".to_string());
        }

        if allowed_origins.is_empty() {
            return Err(
                "Invalid API_ALLOWED_ORIGINS. Must contain at least one origin".to_string(),
            );
        }

        if rate_limits.window_seconds == 0 {
            return Err(
                "Invalid API_RATE_LIMIT_WINDOW_SECONDS. Must be greater than zero".to_string(),
            );
        }

        if session_cookie_same_site != "Strict"
            && session_cookie_same_site != "Lax"
            && session_cookie_same_site != "None"
        {
            return Err(
                "Invalid API_SESSION_COOKIE_SAME_SITE. Expected Strict, Lax, or None".to_string(),
            );
        }

        if session_cookie_same_site == "None" && !session_cookie_secure {
            return Err(
                "Invalid cookie config. SameSite=None requires API_SESSION_COOKIE_SECURE=true"
                    .to_string(),
            );
        }

        if session_cookie_domain.is_some() && !session_cookie_secure {
            return Err(
                "Invalid cookie config. API_SESSION_COOKIE_DOMAIN requires API_SESSION_COOKIE_SECURE=true"
                    .to_string(),
            );
        }

        if environment == "production" {
            if database_url == DEFAULT_DATABASE_URL {
                return Err(
                    "Invalid API_DATABASE_URL for production. Configure a non-default database URL"
                        .to_string(),
                );
            }

            if node_fingerprint == DEFAULT_NODE_FINGERPRINT {
                return Err(
                    "Invalid API_NODE_FINGERPRINT for production. Configure a deployment-specific value"
                        .to_string(),
                );
            }

            if !session_cookie_secure {
                return Err(
                    "Invalid cookie config for production. Set API_SESSION_COOKIE_SECURE=true"
                        .to_string(),
                );
            }

            let has_keyring = env::var("API_SESSION_SIGNING_KEYS")
                .ok()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false);
            if !has_keyring {
                return Err(
                    "Invalid signing key config for production. Set API_SESSION_SIGNING_KEYS and API_SESSION_SIGNING_KEY_ID"
                        .to_string(),
                );
            }

            if channel_dispatch_internal_token == DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN {
                return Err(
                    "Invalid API_CHANNEL_DISPATCH_INTERNAL_TOKEN for production. Configure a non-default internal token"
                        .to_string(),
                );
            }

            if presence_watcher_internal_token == DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN {
                return Err(
                    "Invalid API_PRESENCE_WATCHER_INTERNAL_TOKEN for production. Configure a non-default internal token"
                        .to_string(),
                );
            }
        }

        let parsed_realtime_url = Url::parse(&realtime_base_url).map_err(|_| {
            format!(
                "Invalid API_REALTIME_BASE_URL='{}'. Expected absolute URL like http://127.0.0.1:8081",
                realtime_base_url
            )
        })?;
        let scheme = parsed_realtime_url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(format!(
                "Invalid API_REALTIME_BASE_URL='{}'. Scheme must be http or https",
                realtime_base_url
            ));
        }
        if scheme == "http" && !is_loopback_host(parsed_realtime_url.host_str()) {
            return Err(format!(
                "Invalid API_REALTIME_BASE_URL='{}'. Non-loopback hosts must use https",
                realtime_base_url
            ));
        }

        Ok(Self {
            allow_public_identity_registration,
            bind_addr,
            channel_dispatch_internal_token,
            allowed_origins,
            database_url,
            node_fingerprint,
            discovery_denylist,
            presence_watcher_internal_token,
            presence_redis_url,
            realtime_base_url,
            session_signing_keys,
            active_signing_key_id,
            session_cookie_domain,
            session_cookie_secure,
            session_cookie_same_site,
            trust_proxy_headers,
            rate_limits,
        })
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

fn parse_csv_env(key: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn parse_session_signing_keys() -> Result<(String, HashMap<String, String>), String> {
    let active_key_id = env::var("API_SESSION_SIGNING_KEY_ID").unwrap_or_else(|_| "v1".to_string());
    let keyring_raw = env::var("API_SESSION_SIGNING_KEYS").ok();

    let mut keys = HashMap::new();

    if let Some(raw) = keyring_raw {
        for segment in raw
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            let mut pair = segment.splitn(2, ':');
            let Some(key_id) = pair.next() else {
                continue;
            };
            let Some(secret) = pair.next() else {
                return Err(format!(
                    "Invalid API_SESSION_SIGNING_KEYS entry '{}'. Expected key_id:secret",
                    segment
                ));
            };

            if key_id.trim().is_empty() {
                return Err("Invalid API_SESSION_SIGNING_KEYS entry with empty key_id".to_string());
            }

            if secret.trim().len() < 16 {
                return Err(format!(
                    "Invalid API_SESSION_SIGNING_KEYS secret for key_id '{}'. Must be at least 16 chars",
                    key_id
                ));
            }

            keys.insert(key_id.trim().to_string(), secret.trim().to_string());
        }
    }

    if keys.is_empty() {
        let legacy_key = env::var("API_SESSION_SIGNING_KEY").map_err(|_| {
            "Missing API_SESSION_SIGNING_KEY or API_SESSION_SIGNING_KEYS environment variable"
                .to_string()
        })?;

        if legacy_key.trim().len() < 16 {
            return Err("Invalid API_SESSION_SIGNING_KEY. Must be at least 16 chars".to_string());
        }

        keys.insert(active_key_id.clone(), legacy_key);
    }

    if !keys.contains_key(&active_key_id) {
        return Err(format!(
            "Invalid API_SESSION_SIGNING_KEY_ID='{}'. No matching key in API_SESSION_SIGNING_KEYS",
            active_key_id
        ));
    }

    Ok((active_key_id, keys))
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

fn parse_positive_usize_env(key: &str, default: usize) -> Result<usize, String> {
    let value = parse_usize_env(key, default)?;
    if value == 0 {
        return Err(format!(
            "Invalid {}='0'. Expected integer greater than zero",
            key
        ));
    }

    Ok(value)
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

#[cfg(test)]
mod tests {
    use super::ApiConfig;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_api_env<F>(pairs: &[(&str, Option<&str>)], f: F)
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
    fn rejects_invalid_environment_value() {
        with_api_env(
            &[
                ("API_ENVIRONMENT", Some("staging")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("invalid env should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("Invalid API_ENVIRONMENT"));
            },
        );
    }

    #[test]
    fn parses_public_identity_registration_flag() {
        with_api_env(
            &[
                ("API_ALLOW_PUBLIC_IDENTITY_REGISTRATION", Some("true")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert!(config.allow_public_identity_registration);
            },
        );
    }

    #[test]
    fn parses_split_internal_tokens() {
        with_api_env(
            &[
                (
                    "API_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("hexrelay-dev-channel-dispatch-token-1234"),
                ),
                (
                    "API_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("hexrelay-dev-presence-watcher-token-1234"),
                ),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert_eq!(
                    config.channel_dispatch_internal_token,
                    "hexrelay-dev-channel-dispatch-token-1234"
                );
                assert_eq!(
                    config.presence_watcher_internal_token,
                    "hexrelay-dev-presence-watcher-token-1234"
                );
            },
        );
    }

    #[test]
    fn production_requires_secure_cookie_and_non_default_db() {
        with_api_env(
            &[
                ("API_ENVIRONMENT", Some("production")),
                (
                    "API_SESSION_SIGNING_KEYS",
                    Some("v1:production-secret-key-1234567890"),
                ),
                ("API_SESSION_SIGNING_KEY_ID", Some("v1")),
                ("API_SESSION_COOKIE_SECURE", Some("false")),
                ("API_NODE_FINGERPRINT", Some("prod-fingerprint")),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("insecure production config should fail"),
                    Err(err) => err,
                };
                assert!(
                    err.contains("API_DATABASE_URL") || err.contains("API_SESSION_COOKIE_SECURE")
                );
            },
        );
    }

    #[test]
    fn parses_proxy_header_trust_flag() {
        with_api_env(
            &[
                ("API_TRUST_PROXY_HEADERS", Some("true")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert!(config.trust_proxy_headers);
            },
        );
    }

    #[test]
    fn rejects_zero_rate_limits() {
        with_api_env(
            &[
                ("API_AUTH_CHALLENGE_RATE_LIMIT", Some("0")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("zero auth challenge limit should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_AUTH_CHALLENGE_RATE_LIMIT"));
            },
        );
    }
}
