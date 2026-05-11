use std::{
    collections::HashMap,
    env,
    net::{IpAddr, SocketAddr},
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use communication_core::{
    ed25519_public_key_hex, verify_peer_invite_ed25519, DescriptorValidationContext,
    Ed25519DescriptorVerifier, NodeDescriptor, PeerInviteEnvelope, PeerInviteValidationContext,
    StaticPeerRegistry,
};
use reqwest::Url;

use crate::domain::node_identity::LocalNodeIdentity;

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
    pub enable_dev_testing: bool,
    pub bind_addr: SocketAddr,
    pub channel_dispatch_internal_token: String,
    pub allowed_origins: Vec<String>,
    pub database_url: String,
    pub node_fingerprint: String,
    pub local_node_identity: Option<LocalNodeIdentity>,
    pub discovery_denylist: Vec<String>,
    pub static_peer_registry: StaticPeerRegistry,
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
        let enable_dev_testing = parse_bool_env("API_ENABLE_DEV_TESTING", false)?;
        let node_fingerprint = env::var("API_NODE_FINGERPRINT")
            .unwrap_or_else(|_| DEFAULT_NODE_FINGERPRINT.to_string());
        let allowed_origins_raw = env::var("API_ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3002,http://127.0.0.1:3002".to_string());
        let database_url =
            env::var("API_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string());
        let discovery_denylist = parse_csv_env("API_DISCOVERY_DENYLIST");
        if node_fingerprint.trim().is_empty() {
            return Err("Invalid API_NODE_FINGERPRINT. Value must not be empty".to_string());
        }

        let static_peer_descriptor_max_ttl_seconds =
            parse_i64_env("API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS", 86_400)?;
        let revoked_static_peer_invite_ids = parse_csv_env("API_REVOKED_STATIC_PEER_INVITE_IDS");
        let local_node_identity = parse_local_node_identity(
            "API_LOCAL_NODE_DESCRIPTOR_JSON",
            "API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64",
            static_peer_descriptor_max_ttl_seconds,
            &node_fingerprint,
        )?;
        let static_peer_registry = parse_static_peer_registry(
            "API_STATIC_PEER_DESCRIPTORS_JSON",
            "API_STATIC_PEER_INVITES_JSON",
            static_peer_descriptor_max_ttl_seconds,
            node_fingerprint.trim(),
            &revoked_static_peer_invite_ids,
        )?;
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

        if enable_dev_testing {
            if !bind_addr.ip().is_loopback() {
                return Err(
                    "Invalid API_ENABLE_DEV_TESTING. API_BIND must be loopback when dev testing is enabled"
                        .to_string(),
                );
            }

            let parsed_database_url = Url::parse(&database_url).map_err(|_| {
                format!(
                    "Invalid API_DATABASE_URL='{}'. Expected absolute Postgres URL",
                    database_url
                )
            })?;
            if !is_loopback_host(parsed_database_url.host_str()) {
                return Err(
                    "Invalid API_ENABLE_DEV_TESTING. API_DATABASE_URL must use a loopback host when dev testing is enabled"
                        .to_string(),
                );
            }

            for origin in &allowed_origins {
                let parsed_origin = Url::parse(origin).map_err(|_| {
                    format!(
                        "Invalid API_ALLOWED_ORIGINS entry '{}'. Expected absolute URL",
                        origin
                    )
                })?;
                if !is_loopback_host(parsed_origin.host_str()) {
                    return Err(
                        "Invalid API_ENABLE_DEV_TESTING. API_ALLOWED_ORIGINS must be loopback-only when dev testing is enabled"
                            .to_string(),
                    );
                }
            }
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
            if enable_dev_testing {
                return Err(
                    "Invalid API_ENABLE_DEV_TESTING for production. Dev testing endpoints must be disabled"
                        .to_string(),
                );
            }

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
            enable_dev_testing,
            bind_addr,
            channel_dispatch_internal_token,
            allowed_origins,
            database_url,
            node_fingerprint,
            local_node_identity,
            discovery_denylist,
            static_peer_registry,
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

pub fn parse_i64_env(key: &str, default: i64) -> Result<i64, String> {
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

fn parse_static_peer_registry(
    descriptor_env_key: &str,
    invite_env_key: &str,
    max_ttl_seconds: i64,
    local_node_id: &str,
    revoked_invite_ids: &[String],
) -> Result<StaticPeerRegistry, String> {
    let context = DescriptorValidationContext {
        now_epoch_seconds: current_epoch_seconds()?,
        max_ttl_seconds,
        revoked_descriptor_ids: Vec::new(),
    };
    let verifier = Ed25519DescriptorVerifier;

    let mut descriptors = parse_static_peer_descriptors(descriptor_env_key, &context, &verifier)?;
    descriptors.extend(parse_static_peer_invites(
        invite_env_key,
        &context,
        &verifier,
        max_ttl_seconds,
        local_node_id,
        revoked_invite_ids,
    )?);

    if descriptors.is_empty() {
        return Ok(StaticPeerRegistry::default());
    }

    StaticPeerRegistry::try_new(descriptors)
        .map_err(|error| format!("Invalid static peer configuration: {error:?}"))
}

fn parse_static_peer_descriptors(
    env_key: &str,
    context: &DescriptorValidationContext,
    verifier: &Ed25519DescriptorVerifier,
) -> Result<Vec<NodeDescriptor>, String> {
    let raw = env::var(env_key).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let descriptors = serde_json::from_str::<Vec<NodeDescriptor>>(&raw)
        .map_err(|error| format!("Invalid {env_key}. Expected JSON array: {error}"))?;

    for descriptor in &descriptors {
        descriptor
            .validate_with_signature(context, verifier)
            .map_err(|error| {
                format!(
                    "Invalid {env_key} descriptor '{}': {error:?}",
                    descriptor.descriptor_id
                )
            })?;
    }

    Ok(descriptors)
}

fn parse_static_peer_invites(
    env_key: &str,
    descriptor_context: &DescriptorValidationContext,
    verifier: &Ed25519DescriptorVerifier,
    max_ttl_seconds: i64,
    local_node_id: &str,
    revoked_invite_ids: &[String],
) -> Result<Vec<NodeDescriptor>, String> {
    let raw = env::var(env_key).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    let envelopes = serde_json::from_str::<Vec<PeerInviteEnvelope>>(&raw)
        .map_err(|error| format!("Invalid {env_key}. Expected JSON array: {error}"))?;
    let invite_context = PeerInviteValidationContext {
        now_epoch_seconds: descriptor_context.now_epoch_seconds,
        max_ttl_seconds,
        revoked_invite_ids: revoked_invite_ids.to_vec(),
        expected_subject_node_id: Some(local_node_id.to_string()),
    };
    let mut descriptors = Vec::with_capacity(envelopes.len());

    for envelope in envelopes {
        envelope
            .issuer_descriptor
            .validate_with_signature(descriptor_context, verifier)
            .map_err(|error| {
                format!(
                    "Invalid {env_key} invite '{}' issuer descriptor '{}': {error:?}",
                    envelope.invite.invite_id, envelope.issuer_descriptor.descriptor_id
                )
            })?;
        envelope
            .invite
            .validate(&envelope.issuer_descriptor, &invite_context)
            .map_err(|error| {
                format!(
                    "Invalid {env_key} invite '{}': {error:?}",
                    envelope.invite.invite_id
                )
            })?;
        verify_peer_invite_ed25519(&envelope.invite, &envelope.issuer_descriptor).map_err(
            |error| {
                format!(
                    "Invalid {env_key} invite '{}' signature: {error:?}",
                    envelope.invite.invite_id
                )
            },
        )?;
        descriptors.push(envelope.issuer_descriptor);
    }

    Ok(descriptors)
}

fn parse_local_node_identity(
    descriptor_env_key: &str,
    private_key_env_key: &str,
    max_ttl_seconds: i64,
    expected_node_id: &str,
) -> Result<Option<LocalNodeIdentity>, String> {
    let descriptor_raw = env::var(descriptor_env_key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let private_key_raw = env::var(private_key_env_key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    match (descriptor_raw, private_key_raw) {
        (None, None) => Ok(None),
        (Some(_), None) => Err(format!(
            "Invalid {descriptor_env_key}. {private_key_env_key} is required when local node descriptor is configured"
        )),
        (None, Some(_)) => Err(format!(
            "Invalid {private_key_env_key}. {descriptor_env_key} is required when local node private key is configured"
        )),
        (Some(_), Some(_)) => Ok(Some(parse_required_local_node_identity(
            descriptor_env_key,
            private_key_env_key,
            max_ttl_seconds,
            Some(expected_node_id),
        )?)),
    }
}

pub fn parse_required_local_node_identity(
    descriptor_env_key: &str,
    private_key_env_key: &str,
    max_ttl_seconds: i64,
    expected_node_id: Option<&str>,
) -> Result<LocalNodeIdentity, String> {
    let descriptor_raw = env::var(descriptor_env_key)
        .map_err(|_| format!("Missing {descriptor_env_key}"))?
        .trim()
        .to_string();
    let private_key_raw = env::var(private_key_env_key)
        .map_err(|_| format!("Missing {private_key_env_key}"))?
        .trim()
        .to_string();

    if descriptor_raw.is_empty() {
        return Err(format!(
            "Invalid {descriptor_env_key}. Value must not be empty"
        ));
    }

    if private_key_raw.is_empty() {
        return Err(format!(
            "Invalid {private_key_env_key}. Value must not be empty"
        ));
    }

    let descriptor = serde_json::from_str::<NodeDescriptor>(&descriptor_raw)
        .map_err(|error| format!("Invalid {descriptor_env_key}. Expected JSON object: {error}"))?;
    let private_key_pkcs8 = BASE64.decode(private_key_raw.as_bytes()).map_err(|_| {
        format!("Invalid {private_key_env_key}. Expected base64-encoded Ed25519 PKCS#8 bytes")
    })?;
    let public_key = ed25519_public_key_hex(&private_key_pkcs8).map_err(|error| {
        format!("Invalid {private_key_env_key}. Could not derive Ed25519 public key: {error:?}")
    })?;

    if let Some(expected_node_id) = expected_node_id {
        if descriptor.node_id != expected_node_id.trim() {
            return Err(format!(
                "Invalid {descriptor_env_key}. descriptor node_id must match API_NODE_FINGERPRINT"
            ));
        }
    }

    if descriptor.node_public_key != public_key {
        return Err(format!(
            "Invalid {private_key_env_key}. Private key does not match local node descriptor public key"
        ));
    }

    let context = DescriptorValidationContext {
        now_epoch_seconds: current_epoch_seconds()?,
        max_ttl_seconds,
        revoked_descriptor_ids: Vec::new(),
    };
    descriptor
        .validate_with_signature(&context, &Ed25519DescriptorVerifier)
        .map_err(|error| {
            format!(
                "Invalid {descriptor_env_key} descriptor '{}': {error:?}",
                descriptor.descriptor_id
            )
        })?;

    Ok(LocalNodeIdentity {
        descriptor,
        private_key_pkcs8,
    })
}

pub fn current_epoch_seconds() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System clock is before UNIX epoch".to_string())?
        .as_secs();

    i64::try_from(seconds).map_err(|_| "System clock value is too large".to_string())
}

#[cfg(test)]
mod tests {
    use super::{current_epoch_seconds, ApiConfig};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use communication_core::{
        ed25519_public_key_hex, sign_descriptor_ed25519_pkcs8, DiscoveryPath, DiscoveryPolicy,
        DmForwardingPolicy, NetworkMode, NodeDescriptor, NodeSignature, NodeSignatureAlgorithm,
        PeeringPolicy, RelayPolicy, StoragePolicy,
    };
    use ring::rand::SystemRandom;
    use ring::signature::Ed25519KeyPair;
    use std::sync::{Mutex, OnceLock};

    use crate::domain::{
        node_identity::{generate_node_identity, LocalNodeIdentity, NodeIdentityGenerateOptions},
        peer_invites::{issue_peer_invite, PeerInviteIssueOptions},
    };

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    const API_ENV_KEYS: &[&str] = &[
        "API_BIND",
        "API_ENVIRONMENT",
        "API_ALLOW_PUBLIC_IDENTITY_REGISTRATION",
        "API_ENABLE_DEV_TESTING",
        "API_NODE_FINGERPRINT",
        "API_DATABASE_URL",
        "API_ALLOWED_ORIGINS",
        "API_DISCOVERY_DENYLIST",
        "API_LOCAL_NODE_DESCRIPTOR_JSON",
        "API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64",
        "API_STATIC_PEER_DESCRIPTORS_JSON",
        "API_STATIC_PEER_INVITES_JSON",
        "API_REVOKED_STATIC_PEER_INVITE_IDS",
        "API_STATIC_PEER_DESCRIPTOR_MAX_TTL_SECONDS",
        "API_CHANNEL_DISPATCH_INTERNAL_TOKEN",
        "API_PRESENCE_WATCHER_INTERNAL_TOKEN",
        "API_PRESENCE_REDIS_URL",
        "API_REALTIME_BASE_URL",
        "API_SESSION_SIGNING_KEYS",
        "API_SESSION_SIGNING_KEY_ID",
        "API_SESSION_SIGNING_KEY",
        "API_SESSION_COOKIE_DOMAIN",
        "API_SESSION_COOKIE_SECURE",
        "API_TRUST_PROXY_HEADERS",
        "API_SESSION_COOKIE_SAME_SITE",
        "API_AUTH_CHALLENGE_RATE_LIMIT",
        "API_AUTH_VERIFY_RATE_LIMIT",
        "API_DISCOVERY_QUERY_RATE_LIMIT",
        "API_INVITE_CREATE_RATE_LIMIT",
        "API_INVITE_REDEEM_RATE_LIMIT",
        "API_RATE_LIMIT_WINDOW_SECONDS",
    ];

    fn with_api_env<F>(pairs: &[(&str, Option<&str>)], f: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().expect("acquire env test lock");
        let previous = API_ENV_KEYS
            .iter()
            .map(|key| ((*key).to_string(), std::env::var(key).ok()))
            .collect::<Vec<_>>();

        for key in API_ENV_KEYS {
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

    fn signed_descriptor(node_id: &str, descriptor_id: &str) -> (NodeDescriptor, Vec<u8>) {
        signed_descriptor_with_peering(node_id, descriptor_id, PeeringPolicy::StaticAllowlist)
    }

    fn signed_descriptor_with_peering(
        node_id: &str,
        descriptor_id: &str,
        peering_policy: PeeringPolicy,
    ) -> (NodeDescriptor, Vec<u8>) {
        let pkcs8 =
            Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).expect("generate ed25519 key");
        let public_key = ed25519_public_key_hex(pkcs8.as_ref()).expect("derive public key");
        let now = current_epoch_seconds().expect("read current epoch");
        let mut descriptor = NodeDescriptor {
            node_id: node_id.to_string(),
            node_public_key: public_key,
            descriptor_id: descriptor_id.to_string(),
            issued_at_epoch_seconds: now - 1,
            expires_at_epoch_seconds: now + 300,
            network_mode: NetworkMode::PrivatePeers,
            discovery_policy: DiscoveryPolicy::PrivateAllowlist,
            peering_policy,
            relay_policy: RelayPolicy::None,
            dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
            storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
            addresses: vec![format!("https://{node_id}.example")],
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

        (descriptor, pkcs8.as_ref().to_vec())
    }

    fn signed_static_peer_json() -> String {
        let (descriptor, _) = signed_descriptor("node-a", "descriptor-a");
        serde_json::to_string(&vec![descriptor]).expect("serialize static peer descriptor")
    }

    fn signed_static_peer_invite_json(subject_node_id: &str) -> String {
        let (issuer_descriptor, issuer_private_key) = signed_descriptor_with_peering(
            "node-inviter",
            "descriptor-inviter",
            PeeringPolicy::InviteToken,
        );
        let now = current_epoch_seconds().expect("read current epoch");
        let envelope = issue_peer_invite(
            &LocalNodeIdentity {
                descriptor: issuer_descriptor,
                private_key_pkcs8: issuer_private_key,
            },
            &PeerInviteIssueOptions {
                invite_id: Some("peer-invite-node-inviter".to_string()),
                subject_node_id: Some(subject_node_id.to_string()),
                allow_unbound: false,
                ttl_seconds: 300,
                max_ttl_seconds: 86_400,
                discovery_path: DiscoveryPath::PrivateAllowlist,
                max_uses: Some(1),
            },
            now - 1,
        )
        .expect("issue static peer invite");

        serde_json::to_string(&vec![envelope]).expect("serialize static peer invite")
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
    fn parses_and_validates_static_peer_descriptors() {
        let peer_json = signed_static_peer_json();

        with_api_env(
            &[
                ("API_STATIC_PEER_DESCRIPTORS_JSON", Some(peer_json.as_str())),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert_eq!(config.static_peer_registry.descriptors().len(), 1);
                assert_eq!(
                    config.static_peer_registry.descriptors()[0].node_id,
                    "node-a"
                );
            },
        );
    }

    #[test]
    fn parses_and_validates_static_peer_invites() {
        let invite_json = signed_static_peer_invite_json("node-local");

        with_api_env(
            &[
                ("API_NODE_FINGERPRINT", Some("node-local")),
                ("API_STATIC_PEER_INVITES_JSON", Some(invite_json.as_str())),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert_eq!(config.static_peer_registry.descriptors().len(), 1);
                assert_eq!(
                    config.static_peer_registry.descriptors()[0].node_id,
                    "node-inviter"
                );
                assert_eq!(
                    config.static_peer_registry.descriptors()[0].peering_policy,
                    PeeringPolicy::InviteToken
                );
            },
        );
    }

    #[test]
    fn rejects_static_peer_invite_for_different_subject_node() {
        let invite_json = signed_static_peer_invite_json("node-other");

        with_api_env(
            &[
                ("API_NODE_FINGERPRINT", Some("node-local")),
                ("API_STATIC_PEER_INVITES_JSON", Some(invite_json.as_str())),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("subject-bound static peer invite should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_STATIC_PEER_INVITES_JSON"));
                assert!(err.contains("SubjectNodeMismatch"));
            },
        );
    }

    #[test]
    fn rejects_revoked_static_peer_invite() {
        let invite_json = signed_static_peer_invite_json("node-local");

        with_api_env(
            &[
                ("API_NODE_FINGERPRINT", Some("node-local")),
                ("API_STATIC_PEER_INVITES_JSON", Some(invite_json.as_str())),
                (
                    "API_REVOKED_STATIC_PEER_INVITE_IDS",
                    Some("peer-invite-node-inviter"),
                ),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("revoked static peer invite should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_STATIC_PEER_INVITES_JSON"));
                assert!(err.contains("InviteRevoked"));
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

        with_api_env(
            &[
                ("API_STATIC_PEER_DESCRIPTORS_JSON", Some(peer_json.as_str())),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("invalid static peer descriptor should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_STATIC_PEER_DESCRIPTORS_JSON"));
            },
        );
    }

    #[test]
    fn parses_and_validates_local_node_identity() {
        let (generated, generated_identity) = generate_node_identity(
            &NodeIdentityGenerateOptions {
                node_id: "node-local".to_string(),
                descriptor_id: Some("descriptor-local".to_string()),
                ttl_seconds: 300,
                max_ttl_seconds: 86_400,
                network_mode: NetworkMode::PrivatePeers,
                discovery_policy: DiscoveryPolicy::PrivateAllowlist,
                peering_policy: PeeringPolicy::InviteToken,
                relay_policy: RelayPolicy::None,
                dm_forwarding_policy: DmForwardingPolicy::LocalRecipientsOnly,
                storage_policy: StoragePolicy::DurableEncryptedEnvelopes,
                addresses: vec!["https://node-local.example".to_string()],
                supported_protocols: vec!["hexrelay-node-http".to_string()],
                trust_labels: Vec::new(),
                revocation_pointer: None,
            },
            current_epoch_seconds().expect("read current epoch") - 1,
        )
        .expect("generate local node identity");
        let descriptor_json = serde_json::to_string(&generated.api_local_node_descriptor_json)
            .expect("serialize local node descriptor");

        with_api_env(
            &[
                ("API_NODE_FINGERPRINT", Some("node-local")),
                (
                    "API_LOCAL_NODE_DESCRIPTOR_JSON",
                    Some(descriptor_json.as_str()),
                ),
                (
                    "API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64",
                    Some(generated.api_local_node_private_key_pkcs8_base64.as_str()),
                ),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                let identity = config
                    .local_node_identity
                    .expect("local node identity should parse");
                assert_eq!(identity.descriptor.node_id, "node-local");
                assert_eq!(
                    identity.private_key_pkcs8,
                    generated_identity.private_key_pkcs8
                );
            },
        );
    }

    #[test]
    fn rejects_local_node_identity_when_private_key_mismatches_descriptor() {
        let (descriptor, _) = signed_descriptor("node-local", "descriptor-local");
        let (_, wrong_private_key_pkcs8) = signed_descriptor("node-other", "descriptor-other");
        let descriptor_json =
            serde_json::to_string(&descriptor).expect("serialize local node descriptor");
        let private_key = BASE64.encode(wrong_private_key_pkcs8);

        with_api_env(
            &[
                ("API_NODE_FINGERPRINT", Some("node-local")),
                (
                    "API_LOCAL_NODE_DESCRIPTOR_JSON",
                    Some(descriptor_json.as_str()),
                ),
                (
                    "API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64",
                    Some(private_key.as_str()),
                ),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("mismatched local node key should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_LOCAL_NODE_PRIVATE_KEY_PKCS8_BASE64"));
            },
        );
    }

    #[test]
    fn parses_dev_testing_flag() {
        with_api_env(
            &[
                ("API_ENABLE_DEV_TESTING", Some("true")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let config = ApiConfig::from_env().expect("config should load");
                assert!(config.enable_dev_testing);
            },
        );
    }

    #[test]
    fn production_rejects_dev_testing_flag() {
        with_api_env(
            &[
                ("API_ENVIRONMENT", Some("production")),
                ("API_ENABLE_DEV_TESTING", Some("true")),
                (
                    "API_DATABASE_URL",
                    Some("postgres://hexrelay:pw@db.example.com:5432/hexrelay_prod"),
                ),
                ("API_NODE_FINGERPRINT", Some("prod-fingerprint")),
                ("API_SESSION_COOKIE_SECURE", Some("true")),
                (
                    "API_SESSION_SIGNING_KEYS",
                    Some("v1:production-secret-key-1234567890"),
                ),
                ("API_SESSION_SIGNING_KEY_ID", Some("v1")),
                (
                    "API_CHANNEL_DISPATCH_INTERNAL_TOKEN",
                    Some("prod-channel-dispatch-token-1234"),
                ),
                (
                    "API_PRESENCE_WATCHER_INTERNAL_TOKEN",
                    Some("prod-presence-watcher-token-1234"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("production dev testing flag should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_ENABLE_DEV_TESTING"));
            },
        );
    }

    #[test]
    fn dev_testing_requires_loopback_bind() {
        with_api_env(
            &[
                ("API_ENABLE_DEV_TESTING", Some("true")),
                ("API_BIND", Some("0.0.0.0:8080")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("public bind with dev testing should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_BIND"));
            },
        );
    }

    #[test]
    fn dev_testing_requires_loopback_database() {
        with_api_env(
            &[
                ("API_ENABLE_DEV_TESTING", Some("true")),
                (
                    "API_DATABASE_URL",
                    Some("postgres://hexrelay:pw@db.example.com:5432/hexrelay"),
                ),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("remote DB with dev testing should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_DATABASE_URL"));
            },
        );
    }

    #[test]
    fn dev_testing_requires_loopback_origins() {
        with_api_env(
            &[
                ("API_ENABLE_DEV_TESTING", Some("true")),
                ("API_ALLOWED_ORIGINS", Some("https://app.example.com")),
                (
                    "API_SESSION_SIGNING_KEY",
                    Some("hexrelay-dev-signing-key-change-me"),
                ),
            ],
            || {
                let err = match ApiConfig::from_env() {
                    Ok(_) => panic!("remote origin with dev testing should fail"),
                    Err(err) => err,
                };
                assert!(err.contains("API_ALLOWED_ORIGINS"));
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
