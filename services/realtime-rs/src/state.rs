use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use chrono::{DateTime, Utc};
use communication_core::StaticPeerRegistry;
use serde::Serialize;
use tokio::sync::{mpsc::Sender, Mutex};

use crate::transport::ws::middleware::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct ConnectionSenderEntry {
    pub sender: Sender<String>,
    pub device_id: Option<String>,
    pub dm_device_verified: bool,
}

pub type ConnectionSenderMap = HashMap<String, HashMap<String, ConnectionSenderEntry>>;

#[derive(Clone)]
pub struct AppState {
    pub api_base_url: String,
    pub allowed_origins: Vec<String>,
    pub trust_proxy_headers: bool,
    pub http_client: reqwest::Client,
    pub channel_dispatch_internal_token: String,
    pub presence_watcher_internal_token: String,
    pub presence_redis_client: Option<redis::Client>,
    pub static_peer_registry: StaticPeerRegistry,
    pub rate_limiter: RateLimiter,
    pub ws_connect_rate_limit: usize,
    pub ws_rate_limit_window_seconds: u64,
    pub ws_max_inbound_message_bytes: usize,
    pub ws_message_rate_limit: usize,
    pub ws_message_rate_window_seconds: u64,
    pub ws_max_connections_per_identity: usize,
    pub ws_auth_grace_seconds: u64,
    pub ws_auth_cache_max_entries: usize,
    pub enable_dev_faults: bool,
    pub dev_faults: Arc<Mutex<DevFaultState>>,
    pub active_connections: Arc<Mutex<HashMap<String, usize>>>,
    pub connection_senders: Arc<Mutex<ConnectionSenderMap>>,
    pub locally_dispatched_channel_event_ids: Arc<Mutex<VecDeque<String>>>,
    pub validated_session_cache: Arc<Mutex<HashMap<String, CachedSession>>>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DevFaultConfig {
    pub delay_ms: u64,
    pub drop_rate: f64,
    pub disconnect_after_seconds: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct DevFaultState {
    pub config: DevFaultConfig,
    pub drop_debt: f64,
}

#[derive(Clone)]
pub struct CachedSession {
    pub identity_id: String,
    pub expires_at: DateTime<Utc>,
    pub validated_at: tokio::time::Instant,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        api_base_url: String,
        allowed_origins: Vec<String>,
        channel_dispatch_internal_token: String,
        presence_watcher_internal_token: String,
        presence_redis_client: Option<redis::Client>,
        trust_proxy_headers: bool,
        ws_connect_rate_limit: usize,
        ws_rate_limit_window_seconds: u64,
        ws_max_inbound_message_bytes: usize,
        ws_message_rate_limit: usize,
        ws_message_rate_window_seconds: u64,
        ws_max_connections_per_identity: usize,
        ws_auth_grace_seconds: u64,
        ws_auth_cache_max_entries: usize,
    ) -> Result<Self, String> {
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|error| format!("build realtime HTTP client: {error}"))?;

        Ok(Self {
            api_base_url,
            allowed_origins,
            trust_proxy_headers,
            http_client,
            channel_dispatch_internal_token,
            presence_watcher_internal_token,
            presence_redis_client,
            static_peer_registry: StaticPeerRegistry::default(),
            rate_limiter: RateLimiter::default(),
            ws_connect_rate_limit,
            ws_rate_limit_window_seconds,
            ws_max_inbound_message_bytes,
            ws_message_rate_limit,
            ws_message_rate_window_seconds,
            ws_max_connections_per_identity,
            ws_auth_grace_seconds,
            ws_auth_cache_max_entries,
            enable_dev_faults: false,
            dev_faults: Arc::default(),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            connection_senders: Arc::new(Mutex::new(HashMap::new())),
            locally_dispatched_channel_event_ids: Arc::default(),
            validated_session_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn with_dev_faults_enabled(mut self, enable: bool) -> Self {
        self.enable_dev_faults = enable;
        self
    }

    pub fn with_static_peer_registry(mut self, registry: StaticPeerRegistry) -> Self {
        self.static_peer_registry = registry;
        self
    }
}
