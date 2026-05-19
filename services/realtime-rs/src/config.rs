use std::{
    env,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use communication_core::{
    config::{is_loopback_host, parse_allowed_browser_origins, BrowserOriginPolicy},
    DescriptorValidationContext, Ed25519DescriptorVerifier, NodeDescriptor, StaticPeerRegistry,
};
use reqwest::Url;

pub struct RealtimeConfig {
    pub api_base_url: String,
    pub require_api_health_on_start: bool,
    pub channel_dispatch_internal_token: String,
    pub presence_watcher_internal_token: String,
    pub presence_redis_url: Option<String>,
    pub static_peer_registry: StaticPeerRegistry,
    pub trust_proxy_headers: bool,
    pub allowed_origins: Vec<String>,
    pub bind_addr: SocketAddr,
    pub ws_connect_rate_limit: usize,
    pub rate_limit_window_seconds: u64,
    pub ws_max_inbound_message_bytes: usize,
    pub ws_message_rate_limit: usize,
    pub ws_message_rate_window_seconds: u64,
    pub ws_max_connections_per_identity: usize,
    pub ws_auth_grace_seconds: u64,
    pub ws_auth_cache_max_entries: usize,
    pub enable_dev_faults: bool,
}

impl RealtimeConfig {
    pub fn from_env() -> Result<Self, String> {
        const DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN: &str =
            "hexrelay-dev-channel-dispatch-token-change-me";
        const DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN: &str =
            "hexrelay-dev-presence-watcher-token-change-me";
        let environment = env::var("REALTIME_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string())
            .trim()
            .to_ascii_lowercase();
        if environment != "development" && environment != "production" {
            return Err(
                "Invalid REALTIME_ENVIRONMENT. Expected 'development' or 'production'".to_string(),
            );
        }
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
        let origin_policy = if environment == "production" {
            BrowserOriginPolicy::Production
        } else {
            BrowserOriginPolicy::Development
        };
        let allowed_origins = parse_allowed_browser_origins(
            "REALTIME_ALLOWED_ORIGINS",
            &allowed_origins_raw,
            origin_policy,
        )?;
        let require_api_health_on_start =
            parse_bool_env("REALTIME_REQUIRE_API_HEALTH_ON_START", true)?;
        let channel_dispatch_internal_token = env::var("REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN")
            .unwrap_or_else(|_| DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN.to_string());
        let presence_watcher_internal_token = env::var("REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN")
            .unwrap_or_else(|_| DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN.to_string());
        let presence_redis_url = env::var("REALTIME_PRESENCE_REDIS_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let static_peer_descriptor_max_ttl_seconds =
            parse_i64_env("REALTIME_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS", 86_400)?;
        let static_peer_registry = parse_static_peer_registry(
            "REALTIME_STATIC_PEER_DESCRIPTORS_JSON",
            static_peer_descriptor_max_ttl_seconds,
        )?;
        let trust_proxy_headers = parse_bool_env("REALTIME_TRUST_PROXY_HEADERS", false)?;
        let enable_dev_faults = parse_bool_env("REALTIME_ENABLE_DEV_FAULTS", false)?;
        let ws_connect_rate_limit = parse_usize_env("REALTIME_WS_CONNECT_RATE_LIMIT", 60)?;
        let rate_limit_window_seconds = parse_u64_env("REALTIME_RATE_LIMIT_WINDOW_SECONDS", 60)?;
        let ws_max_inbound_message_bytes =
            parse_usize_env("REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES", 16384)?;
        let ws_message_rate_limit = parse_usize_env("REALTIME_WS_MESSAGE_RATE_LIMIT", 120)?;
        let ws_message_rate_window_seconds =
            parse_u64_env("REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS", 60)?;
        let ws_max_connections_per_identity =
            parse_usize_env("REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY", 3)?;
        let ws_auth_grace_seconds = parse_u64_env("REALTIME_WS_AUTH_GRACE_SECONDS", 0)?;
        let ws_auth_cache_max_entries =
            parse_usize_env("REALTIME_WS_AUTH_CACHE_MAX_ENTRIES", 10000)?;

        if api_base_url.trim().is_empty() {
            return Err("Invalid REALTIME_API_BASE_URL. Value must not be empty".to_string());
        }

        if channel_dispatch_internal_token.trim().len() < 16 {
            return Err(
                "Invalid REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN. Expected at least 16 characters"
                    .to_string(),
            );
        }

        if presence_watcher_internal_token.trim().len() < 16 {
            return Err(
                "Invalid REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN. Expected at least 16 characters"
                    .to_string(),
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

        if ws_auth_cache_max_entries == 0 {
            return Err(
                "Invalid REALTIME_WS_AUTH_CACHE_MAX_ENTRIES. Expected integer greater than 0"
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

        if environment == "production"
            && channel_dispatch_internal_token == DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN
        {
            return Err(
                "Invalid REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN for production. Configure a non-default internal token"
                    .to_string(),
            );
        }

        if environment == "production"
            && presence_watcher_internal_token == DEFAULT_PRESENCE_WATCHER_INTERNAL_TOKEN
        {
            return Err(
                "Invalid REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN for production. Configure a non-default internal token"
                    .to_string(),
            );
        }

        if environment == "production" && enable_dev_faults {
            return Err(
                "Invalid REALTIME_ENABLE_DEV_FAULTS for production. Dev fault hooks must be disabled"
                    .to_string(),
            );
        }

        if enable_dev_faults
            && !bind_addr.ip().is_loopback()
            && channel_dispatch_internal_token == DEFAULT_CHANNEL_DISPATCH_INTERNAL_TOKEN
        {
            return Err(
                "Invalid REALTIME_ENABLE_DEV_FAULTS. Non-loopback dev fault binds require a non-default REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN"
                    .to_string(),
            );
        }

        Ok(Self {
            api_base_url,
            require_api_health_on_start,
            channel_dispatch_internal_token,
            presence_watcher_internal_token,
            presence_redis_url,
            static_peer_registry,
            trust_proxy_headers,
            allowed_origins,
            bind_addr,
            ws_connect_rate_limit,
            rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
            ws_auth_grace_seconds,
            ws_auth_cache_max_entries,
            enable_dev_faults,
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

fn parse_i64_env(key: &str, default: i64) -> Result<i64, String> {
    match env::var(key) {
        Ok(value) => value
            .trim()
            .parse::<i64>()
            .map_err(|_| format!("Invalid {}='{}'. Expected positive integer", key, value))
            .and_then(|parsed| {
                if parsed > 0 {
                    Ok(parsed)
                } else {
                    Err(format!(
                        "Invalid {}='{}'. Expected integer greater than zero",
                        key, value
                    ))
                }
            }),
        Err(_) => Ok(default),
    }
}

fn parse_static_peer_registry(
    env_key: &str,
    max_ttl_seconds: i64,
) -> Result<StaticPeerRegistry, String> {
    let raw = env::var(env_key).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(StaticPeerRegistry::default());
    }

    let descriptors = serde_json::from_str::<Vec<NodeDescriptor>>(&raw)
        .map_err(|error| format!("Invalid {env_key}. Expected JSON array: {error}"))?;
    let context = DescriptorValidationContext {
        now_epoch_seconds: current_epoch_seconds()?,
        max_ttl_seconds,
        revoked_descriptor_ids: Vec::new(),
    };
    let verifier = Ed25519DescriptorVerifier;

    for descriptor in &descriptors {
        descriptor
            .validate_with_signature(&context, &verifier)
            .map_err(|error| {
                format!(
                    "Invalid {env_key} descriptor '{}': {error:?}",
                    descriptor.descriptor_id
                )
            })?;
    }

    StaticPeerRegistry::try_new(descriptors)
        .map_err(|error| format!("Invalid {env_key}: {error:?}"))
}

fn current_epoch_seconds() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is before UNIX epoch".to_string())?
        .as_secs();

    i64::try_from(seconds).map_err(|_| "System clock value is too large".to_string())
}

#[cfg(test)]
mod tests {
    use super::{current_epoch_seconds, is_loopback_host, RealtimeConfig};
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPolicy, DmForwardingPolicy,
        NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm, PeeringPolicy,
        RelayPolicy, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::Ed25519KeyPair;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    const REALTIME_ENV_KEYS: &[&str] = &[
        "REALTIME_ENVIRONMENT",
        "REALTIME_BIND",
        "REALTIME_API_BASE_URL",
        "REALTIME_ALLOWED_ORIGINS",
        "REALTIME_REQUIRE_API_HEALTH_ON_START",
        "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
        "REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN",
        "REALTIME_PRESENCE_REDIS_URL",
        "REALTIME_STATIC_PEER_DESCRIPTORS_JSON",
        "REALTIME_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS",
        "REALTIME_TRUST_PROXY_HEADERS",
        "REALTIME_ENABLE_DEV_FAULTS",
        "REALTIME_WS_CONNECT_RATE_LIMIT",
        "REALTIME_RATE_LIMIT_WINDOW_SECONDS",
        "REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES",
        "REALTIME_WS_MESSAGE_RATE_LIMIT",
        "REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS",
        "REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY",
        "REALTIME_WS_AUTH_GRACE_SECONDS",
        "REALTIME_WS_AUTH_CACHE_MAX_ENTRIES",
    ];

    fn with_realtime_env<F>(pairs: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        let guard = env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = REALTIME_ENV_KEYS
            .iter()
            .map(|key| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for key in REALTIME_ENV_KEYS {
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

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

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

        drop(guard);

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    fn signed_static_peer_json() -> String {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = current_epoch_seconds().expect("read current epoch");
        let mut descriptor = NodeDescriptor {
            node_id: "node-a".to_string(),
            node_public_key: public_key,
            descriptor_id: "descriptor-a".to_string(),
            issued_at_epoch_seconds: now - 1,
            expires_at_epoch_seconds: now + 300,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy: PeeringPolicy::StaticAllowlist,
            relay_policy: RelayPolicy::None,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec!["https://node-a.example".to_string()],
            supported_protocols: vec!["hexrelay-node-http".to_string()],
            rate_limits: Vec::new(),
            trust_labels: Vec::new(),
            revocation_pointer: None,
            signature: NodeSignature {
                algorithm: NodeSignatureAlgorithm::Ed25519,
                value: String::new(),
            },
        };
        descriptor.signature.value =
            sign_descriptor_ed25519_pkcs8(&descriptor, pkcs8.as_ref()).expect("sign descriptor");

        serde_json::to_string(&vec![descriptor]).expect("serialize static peer descriptor")
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
    fn parses_and_validates_static_peer_descriptors() {
        let peer_json = signed_static_peer_json();

        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                (
                    "REALTIME_STATIC_PEER_DESCRIPTORS_JSON",
                    Some(peer_json.as_str()),
                ),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
                assert_eq!(config.static_peer_registry.descriptors().len(), 1);
                assert_eq!(
                    config.static_peer_registry.descriptors()[0].node_id,
                    "node-a"
                );
            },
        );
    }

    #[test]
    fn rejects_invalid_static_peer_descriptor_signature() {
        let mut descriptors =
            serde_json::from_str::<Vec<NodeDescriptor>>(&signed_static_peer_json())
                .expect("parse generated peer json");
        descriptors[0].signature.value = "00".repeat(64);
        let peer_json = serde_json::to_string(&descriptors).expect("serialize tampered peer json");

        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                (
                    "REALTIME_STATIC_PEER_DESCRIPTORS_JSON",
                    Some(peer_json.as_str()),
                ),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("invalid static peer descriptor should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_STATIC_PEER_DESCRIPTORS_JSON"));
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

        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_WS_AUTH_CACHE_MAX_ENTRIES", Some("0")),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("zero auth cache entries must fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_WS_AUTH_CACHE_MAX_ENTRIES"));
            },
        );
    }

    #[test]
    fn parses_auth_grace_configuration() {
        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_WS_AUTH_GRACE_SECONDS", Some("30")),
                ("REALTIME_WS_AUTH_CACHE_MAX_ENTRIES", Some("200")),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
                assert_eq!(config.ws_auth_grace_seconds, 30);
                assert_eq!(config.ws_auth_cache_max_entries, 200);
            },
        );
    }

    #[test]
    fn parses_dev_fault_flag_only_outside_production() {
        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_ENABLE_DEV_FAULTS", Some("true")),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
                assert!(config.enable_dev_faults);
            },
        );

        with_realtime_env(
            &[
                ("REALTIME_BIND", Some("0.0.0.0:8081")),
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                ("REALTIME_ENABLE_DEV_FAULTS", Some("true")),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("non-loopback dev faults with default token must fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN"));
            },
        );

        with_realtime_env(
            &[
                ("REALTIME_BIND", Some("0.0.0.0:8081")),
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("development-dev-fault-token-12345"),
                ),
                ("REALTIME_ENABLE_DEV_FAULTS", Some("true")),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
                assert!(config.enable_dev_faults);
            },
        );

        with_realtime_env(
            &[
                ("REALTIME_ENVIRONMENT", Some("production")),
                (
                    "REALTIME_API_BASE_URL",
                    Some("https://realtime.example.com"),
                ),
                ("REALTIME_ALLOWED_ORIGINS", Some("https://app.example.com")),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("production-channel-dispatch-token-12345"),
                ),
                (
                    "REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("production-presence-watcher-token-12345"),
                ),
                ("REALTIME_ENABLE_DEV_FAULTS", Some("true")),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("production dev faults must fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_ENABLE_DEV_FAULTS"));
            },
        );
    }

    #[test]
    fn production_requires_non_default_internal_token() {
        with_realtime_env(
            &[
                ("REALTIME_ENVIRONMENT", Some("production")),
                (
                    "REALTIME_API_BASE_URL",
                    Some("https://realtime.example.com"),
                ),
                ("REALTIME_ALLOWED_ORIGINS", Some("https://app.example.com")),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("hexrelay-dev-channel-dispatch-token-change-me"),
                ),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("default production token should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN"));
            },
        );
    }

    #[test]
    fn production_rejects_non_loopback_http_allowed_origin() {
        with_realtime_env(
            &[
                ("REALTIME_ENVIRONMENT", Some("production")),
                (
                    "REALTIME_API_BASE_URL",
                    Some("https://realtime.example.com"),
                ),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://app.example.com")),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("production-channel-dispatch-token-12345"),
                ),
                (
                    "REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("production-presence-watcher-token-12345"),
                ),
            ],
            || {
                let err = match RealtimeConfig::from_env() {
                    Ok(_) => panic!("production non-loopback http origin should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("REALTIME_ALLOWED_ORIGINS"));
                assert!(err.contains("Non-loopback browser origins must use https"));
            },
        );
    }

    #[test]
    fn production_accepts_https_and_loopback_http_allowed_origins() {
        with_realtime_env(
            &[
                ("REALTIME_ENVIRONMENT", Some("production")),
                (
                    "REALTIME_API_BASE_URL",
                    Some("https://realtime.example.com"),
                ),
                (
                    "REALTIME_ALLOWED_ORIGINS",
                    Some("https://app.example.com,http://127.0.0.1:3002"),
                ),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("production-channel-dispatch-token-12345"),
                ),
                (
                    "REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("production-presence-watcher-token-12345"),
                ),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("production config should parse");
                assert_eq!(
                    config.allowed_origins,
                    vec![
                        "https://app.example.com".to_string(),
                        "http://127.0.0.1:3002".to_string()
                    ]
                );
            },
        );
    }

    #[test]
    fn parses_split_internal_tokens() {
        with_realtime_env(
            &[
                ("REALTIME_API_BASE_URL", Some("http://127.0.0.1:8080")),
                ("REALTIME_ALLOWED_ORIGINS", Some("http://127.0.0.1:3002")),
                (
                    "REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("hexrelay-dev-channel-dispatch-token-1234"),
                ),
                (
                    "REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("hexrelay-dev-presence-watcher-token-1234"),
                ),
            ],
            || {
                let config = RealtimeConfig::from_env().expect("config should parse");
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
}
