use std::net::IpAddr;

use url::Url;

use crate::domain::{DmTransportPolicy, PolicyContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommunicationConfig {
    pub dm_transport_policy: DmTransportPolicy,
    pub enable_server_channel: bool,
    pub enable_presence: bool,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            dm_transport_policy: DmTransportPolicy::EncryptedEnvelopeNode,
            enable_server_channel: true,
            enable_presence: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserOriginPolicy {
    Development,
    Production,
}

pub fn parse_allowed_browser_origins(
    env_key: &str,
    raw: &str,
    policy: BrowserOriginPolicy,
) -> Result<Vec<String>, String> {
    let origins = raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|origin| parse_browser_origin(env_key, origin, policy))
        .collect::<Result<Vec<_>, _>>()?;

    if origins.is_empty() {
        return Err(format!(
            "Invalid {env_key}. Must contain at least one origin"
        ));
    }

    Ok(origins)
}

pub fn is_loopback_host(host: Option<&str>) -> bool {
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

fn parse_browser_origin(
    env_key: &str,
    origin: &str,
    policy: BrowserOriginPolicy,
) -> Result<String, String> {
    let parsed = Url::parse(origin).map_err(|_| {
        format!("Invalid {env_key} entry '{origin}'. Expected absolute browser origin URL")
    })?;

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "Invalid {env_key} entry '{origin}'. Scheme must be http or https"
        ));
    }

    if parsed.host_str().is_none() {
        return Err(format!(
            "Invalid {env_key} entry '{origin}'. Expected absolute browser origin URL"
        ));
    }

    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(format!(
            "Invalid {env_key} entry '{origin}'. Expected origin without path, query, or fragment"
        ));
    }

    if policy == BrowserOriginPolicy::Production
        && scheme == "http"
        && !is_loopback_host(parsed.host_str())
    {
        return Err(format!(
            "Invalid {env_key} entry '{origin}'. Non-loopback browser origins must use https in production"
        ));
    }

    Ok(parsed.origin().ascii_serialization())
}

impl CommunicationConfig {
    pub fn policy_context(&self) -> PolicyContext {
        PolicyContext {
            dm_transport_policy: self.dm_transport_policy,
            enable_server_channel: self.enable_server_channel,
            enable_presence: self.enable_presence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_allowed_browser_origins, BrowserOriginPolicy};

    #[test]
    fn production_rejects_non_loopback_http_origin() {
        let err = parse_allowed_browser_origins(
            "TEST_ALLOWED_ORIGINS",
            "http://app.example.com",
            BrowserOriginPolicy::Production,
        )
        .expect_err("non-loopback http origin should fail");

        assert!(err.contains("TEST_ALLOWED_ORIGINS"));
        assert!(err.contains("Non-loopback browser origins must use https"));
    }

    #[test]
    fn production_accepts_https_and_loopback_http_origins() {
        let origins = parse_allowed_browser_origins(
            "TEST_ALLOWED_ORIGINS",
            "https://app.example.com,http://127.0.0.1:3002",
            BrowserOriginPolicy::Production,
        )
        .expect("origins should parse");

        assert_eq!(
            origins,
            vec![
                "https://app.example.com".to_string(),
                "http://127.0.0.1:3002".to_string()
            ]
        );
    }

    #[test]
    fn rejects_origins_with_path_query_or_fragment() {
        for origin in [
            "https://app.example.com/app",
            "https://app.example.com?debug=true",
            "https://app.example.com#debug",
        ] {
            let err = parse_allowed_browser_origins(
                "TEST_ALLOWED_ORIGINS",
                origin,
                BrowserOriginPolicy::Development,
            )
            .expect_err("origin with non-origin components should fail");

            assert!(err.contains("without path, query, or fragment"));
        }
    }
}
