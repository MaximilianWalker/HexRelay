use std::{
    collections::BTreeMap,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use crate::{
    config::ApiRateLimitConfig,
    models::{
        AuthChallengeRecord, DmPolicy, FriendRequestRecord, InviteRecord, RegisteredIdentityKey,
        SessionRecord,
    },
    transport::http::middleware::rate_limit::RateLimiter,
};

#[derive(Clone)]
pub struct AppState {
    pub active_signing_key_id: String,
    pub allowed_origins: Vec<String>,
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub db_pool: Option<PgPool>,
    pub dm_policies: Arc<RwLock<HashMap<String, DmPolicy>>>,
    pub friend_requests: Arc<RwLock<HashMap<String, FriendRequestRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
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
        session_signing_keys: BTreeMap<String, String>,
        session_cookie_domain: Option<String>,
        session_cookie_secure: bool,
        session_cookie_same_site: String,
        rate_limits: ApiRateLimitConfig,
        trust_proxy_headers: bool,
    ) -> Self {
        Self {
            active_signing_key_id,
            allowed_origins,
            auth_challenges: Arc::default(),
            db_pool: None,
            dm_policies: Arc::default(),
            friend_requests: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
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
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            "hexrelay-local-fingerprint".to_string(),
            vec!["http://localhost:3002".to_string()],
            "v1".to_string(),
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
                invite_create_per_window: 20,
                invite_redeem_per_window: 40,
                window_seconds: 60,
            },
            false,
        )
    }
}
