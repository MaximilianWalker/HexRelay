use std::{
    collections::BTreeMap,
    collections::HashMap,
    collections::HashSet,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use crate::{
    config::ApiRateLimitConfig,
    models::{
        AuthChallengeRecord, DmEndpointCardRecord, DmFanoutDeliveryRecord, DmLanPresenceRecord,
        DmPolicy, DmProfileDeviceRecord, FriendRequestRecord, InviteRecord, RegisteredIdentityKey,
        SessionRecord,
    },
    transport::http::middleware::rate_limit::RateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub active_signing_key_id: String,
    pub allow_public_identity_registration: bool,
    pub allowed_origins: Vec<String>,
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub blocked_users: Arc<RwLock<HashMap<String, HashMap<String, i64>>>>,
    pub db_pool: Option<PgPool>,
    pub discovery_denylist: Arc<HashSet<String>>,
    pub http_client: reqwest::Client,
    pub presence_internal_token: String,
    pub presence_redis_client: Option<redis::Client>,
    pub realtime_base_url: String,
    pub dm_endpoint_cards: Arc<RwLock<HashMap<String, HashMap<String, DmEndpointCardRecord>>>>,
    pub dm_fanout_delivery_log: Arc<RwLock<HashMap<String, Vec<DmFanoutDeliveryRecord>>>>,
    pub dm_fanout_device_cursors: Arc<RwLock<HashMap<String, HashMap<String, u64>>>>,
    pub dm_lan_presence: Arc<RwLock<HashMap<String, DmLanPresenceRecord>>>,
    pub dm_profile_devices: Arc<RwLock<HashMap<String, HashMap<String, DmProfileDeviceRecord>>>>,
    pub dm_pairing_nonces: Arc<RwLock<HashMap<String, i64>>>,
    pub dm_policies: Arc<RwLock<HashMap<String, DmPolicy>>>,
    pub friend_requests: Arc<RwLock<HashMap<String, FriendRequestRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
    pub muted_users: Arc<RwLock<HashMap<String, HashMap<String, i64>>>>,
    pub node_fingerprint: String,
    pub rate_limiter: RateLimiter,
    pub rate_limits: ApiRateLimitConfig,
    pub session_cookie_domain: Option<String>,
    pub session_cookie_same_site: String,
    pub session_cookie_secure: bool,
    pub session_signing_keys: Arc<BTreeMap<String, String>>,
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
        presence_internal_token: String,
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
            db_pool: None,
            discovery_denylist: Arc::new(discovery_denylist.into_iter().collect()),
            http_client,
            presence_internal_token,
            presence_redis_client,
            realtime_base_url,
            dm_endpoint_cards: Arc::default(),
            dm_fanout_delivery_log: Arc::default(),
            dm_fanout_device_cursors: Arc::default(),
            dm_lan_presence: Arc::default(),
            dm_profile_devices: Arc::default(),
            dm_pairing_nonces: Arc::default(),
            dm_policies: Arc::default(),
            friend_requests: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
            muted_users: Arc::default(),
            node_fingerprint,
            rate_limiter: RateLimiter::default(),
            rate_limits,
            session_cookie_domain,
            session_cookie_same_site,
            session_cookie_secure,
            session_signing_keys: Arc::new(session_signing_keys),
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            "hexrelay-local-fingerprint".to_string(),
            vec!["http://localhost:3002".to_string()],
            "v1".to_string(),
            Vec::new(),
            "hexrelay-dev-presence-token-change-me".to_string(),
            None,
            "http://127.0.0.1:8081".to_string(),
            BTreeMap::from([(
                "v1".to_string(),
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
                window_seconds: 60,
            },
            false,
        )
        .with_public_identity_registration(true)
    }
}
