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
    pub rate_limits: ApiRateLimitConfig,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, String> {
        let bind_raw = env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let bind_addr = bind_raw.parse::<SocketAddr>().map_err(|_| {
            format!(
                "Invalid API_BIND='{}'. Expected host:port like 127.0.0.1:8080",
                bind_raw
            )
        })?;

        let node_fingerprint = env::var("API_NODE_FINGERPRINT")
            .unwrap_or_else(|_| "hexrelay-local-fingerprint".to_string());
        let allowed_origins_raw = env::var("API_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let database_url = env::var("API_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay".to_string()
        });
        let (active_signing_key_id, session_signing_keys) = parse_session_signing_keys()?;
        let session_cookie_domain = env::var("API_SESSION_COOKIE_DOMAIN")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let session_cookie_secure = parse_bool_env("API_SESSION_COOKIE_SECURE", false)?;
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
