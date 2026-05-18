use std::{
    collections::BTreeMap,
    collections::HashMap,
    collections::HashSet,
    sync::{Arc, RwLock},
};

use communication_core::StaticPeerRegistry;
use sqlx::PgPool;

use crate::{
    config::{ApiDmRetentionConfig, ApiRateLimitConfig},
    domain::node_identity::LocalNodeIdentity,
    models::{
        AuthChallengeRecord, DmFanoutDeliveryRecord, DmPolicy, DmProfileDeviceRecord,
        FriendRequestRecord, InviteRecord, RegisteredIdentityKey, SessionRecord,
    },
    transport::http::adapters::server_channels_realtime::ServerChannelDispatchQueue,
    transport::http::middleware::rate_limit::RateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub active_signing_key_id: String,
    pub allow_public_identity_registration: bool,
    pub allowed_origins: Vec<String>,
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub blocked_users: Arc<RwLock<HashMap<String, HashMap<String, i64>>>>,
    pub channel_dispatch_internal_token: String,
    pub db_pool: Option<PgPool>,
    pub enable_dev_testing: bool,
    pub discovery_denylist: Arc<HashSet<String>>,
    pub static_peer_registry: StaticPeerRegistry,
    pub http_client: reqwest::Client,
    pub presence_watcher_internal_token: String,
    pub presence_redis_client: Option<redis::Client>,
    pub realtime_base_url: String,
    pub dm_fanout_delivery_log: Arc<RwLock<HashMap<String, Vec<DmFanoutDeliveryRecord>>>>,
    pub dm_fanout_device_cursors: Arc<RwLock<HashMap<String, HashMap<String, u64>>>>,
    pub dm_profile_devices: Arc<RwLock<HashMap<String, HashMap<String, DmProfileDeviceRecord>>>>,
    pub dm_policies: Arc<RwLock<HashMap<String, DmPolicy>>>,
    pub friend_requests: Arc<RwLock<HashMap<String, FriendRequestRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
    pub muted_users: Arc<RwLock<HashMap<String, HashMap<String, i64>>>>,
    pub node_admin_identity_ids: Arc<HashSet<String>>,
    pub node_fingerprint: String,
    pub node_forwarding_nonces: Arc<RwLock<HashMap<String, i64>>>,
    pub node_owner_identity_ids: Arc<HashSet<String>>,
    pub local_node_identity: Option<LocalNodeIdentity>,
    pub rate_limiter: RateLimiter,
    pub rate_limits: ApiRateLimitConfig,
    pub dm_retention: ApiDmRetentionConfig,
    pub session_cookie_domain: Option<String>,
    pub session_cookie_same_site: String,
    pub session_cookie_secure: bool,
    pub session_signing_keys: Arc<BTreeMap<String, String>>,
    pub server_channel_dispatch_queue: ServerChannelDispatchQueue,
    pub sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
    pub trust_proxy_headers: bool,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        node_fingerprint: String,
        allowed_origins: Vec<String>,
        active_signing_key_id: String,
        discovery_denylist: Vec<String>,
        channel_dispatch_internal_token: String,
        presence_watcher_internal_token: String,
        presence_redis_client: Option<redis::Client>,
        realtime_base_url: String,
        session_signing_keys: BTreeMap<String, String>,
        session_cookie_domain: Option<String>,
        session_cookie_secure: bool,
        session_cookie_same_site: String,
        rate_limits: ApiRateLimitConfig,
        trust_proxy_headers: bool,
    ) -> Self {
        let http_client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(2))
            .timeout(std::time::Duration::from_secs(3))
            .build()
            .expect("build API HTTP client");

        Self {
            active_signing_key_id,
            allow_public_identity_registration: false,
            allowed_origins,
            auth_challenges: Arc::default(),
            blocked_users: Arc::default(),
            channel_dispatch_internal_token,
            db_pool: None,
            enable_dev_testing: false,
            discovery_denylist: Arc::new(discovery_denylist.into_iter().collect()),
            static_peer_registry: StaticPeerRegistry::default(),
            http_client,
            presence_watcher_internal_token,
            presence_redis_client,
            realtime_base_url,
            dm_fanout_delivery_log: Arc::default(),
            dm_fanout_device_cursors: Arc::default(),
            dm_profile_devices: Arc::default(),
            dm_policies: Arc::default(),
            friend_requests: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
            muted_users: Arc::default(),
            node_admin_identity_ids: Arc::default(),
            node_fingerprint,
            node_forwarding_nonces: Arc::default(),
            node_owner_identity_ids: Arc::default(),
            local_node_identity: None,
            rate_limiter: RateLimiter::default(),
            rate_limits,
            dm_retention: ApiDmRetentionConfig::default(),
            session_cookie_domain,
            session_cookie_same_site,
            session_cookie_secure,
            session_signing_keys: Arc::new(session_signing_keys),
            server_channel_dispatch_queue: ServerChannelDispatchQueue::default(),
            sessions: Arc::default(),
            trust_proxy_headers,
        }
    }

    pub fn with_db_pool(mut self, db_pool: PgPool) -> Self {
        self.db_pool = Some(db_pool);
        self
    }

    pub fn with_public_identity_registration(mut self, allow: bool) -> Self {
        self.allow_public_identity_registration = allow;
        self
    }

    pub fn with_dev_testing(mut self, enable: bool) -> Self {
        self.enable_dev_testing = enable;
        self
    }

    pub fn with_static_peer_registry(mut self, registry: StaticPeerRegistry) -> Self {
        self.static_peer_registry = registry;
        self
    }

    pub fn with_local_node_identity(mut self, identity: Option<LocalNodeIdentity>) -> Self {
        self.local_node_identity = identity;
        self
    }

    pub fn with_node_owner_identity_ids(mut self, identity_ids: Vec<String>) -> Self {
        self.node_owner_identity_ids = Arc::new(identity_ids.into_iter().collect());
        self
    }

    pub fn with_node_admin_identity_ids(mut self, identity_ids: Vec<String>) -> Self {
        self.node_admin_identity_ids = Arc::new(identity_ids.into_iter().collect());
        self
    }

    pub fn with_dm_retention(mut self, retention: ApiDmRetentionConfig) -> Self {
        self.dm_retention = retention;
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            "hexrelay-local-fingerprint".to_string(),
            vec!["http://localhost:3002".to_string()],
            "primary".to_string(),
            Vec::new(),
            "hexrelay-dev-channel-dispatch-token-change-me".to_string(),
            "hexrelay-dev-presence-watcher-token-change-me".to_string(),
            None,
            "http://127.0.0.1:8081".to_string(),
            BTreeMap::from([(
                "primary".to_string(),
                "hexrelay-dev-signing-key-change-me".to_string(),
            )]),
            None,
            false,
            "Lax".to_string(),
            ApiRateLimitConfig {
                auth_challenge_per_window: 30,
                auth_verify_per_window: 30,
                discovery_query_per_window: 30,
                invite_create_per_window: 20,
                invite_redeem_per_window: 40,
                dm_dispatch_per_window: 120,
                dm_catch_up_per_window: 120,
                dm_ack_per_window: 600,
                dm_internal_forward_per_window: 240,
                window_seconds: 60,
            },
            false,
        )
    }
}
