use std::{collections::HashMap, env, net::SocketAddr};

#[derive(Clone)]
pub struct ApiRateLimitConfig {
    pub auth_challenge_per_window: usize,
    pub auth_verify_per_window: usize,
    pub invite_create_per_window: usize,
    pub invite_redeem_per_window: usize,
    pub window_seconds: u64,
}

pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub allowed_origins: Vec<String>,
    pub database_url: String,
    pub node_fingerprint: String,
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

        let node_fingerprint = env::var("API_NODE_FINGERPRINT")
            .unwrap_or_else(|_| DEFAULT_NODE_FINGERPRINT.to_string());
        let allowed_origins_raw = env::var("API_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let database_url =
            env::var("API_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
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
            auth_challenge_per_window: parse_usize_env("API_AUTH_CHALLENGE_RATE_LIMIT", 30)?,
            auth_verify_per_window: parse_usize_env("API_AUTH_VERIFY_RATE_LIMIT", 30)?,
            invite_create_per_window: parse_usize_env("API_INVITE_CREATE_RATE_LIMIT", 20)?,
            invite_redeem_per_window: parse_usize_env("API_INVITE_REDEEM_RATE_LIMIT", 40)?,
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
        }

        Ok(Self {
            bind_addr,
            allowed_origins,
            database_url,
            node_fingerprint,
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
        let keys = [
            "API_ENVIRONMENT",
            "API_BIND",
            "API_NODE_FINGERPRINT",
            "API_DATABASE_URL",
            "API_ALLOWED_ORIGINS",
            "API_SESSION_SIGNING_KEYS",
            "API_SESSION_SIGNING_KEY_ID",
            "API_SESSION_SIGNING_KEY",
            "API_SESSION_COOKIE_SECURE",
            "API_TRUST_PROXY_HEADERS",
            "API_SESSION_COOKIE_SAME_SITE",
            "API_SESSION_COOKIE_DOMAIN",
        ];

        let previous = keys
            .iter()
            .map(|key| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for key in keys {
            unsafe {
                std::env::remove_var(key);
            }
        }

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
                (
                    "API_DATABASE_URL",
                    Some("postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay"),
                ),
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
}
