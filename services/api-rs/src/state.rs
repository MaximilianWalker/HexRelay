use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::PgPool;

use crate::models::{
    AuthChallengeRecord, FriendRequestRecord, InviteRecord, RegisteredIdentityKey, SessionRecord,
};

#[derive(Clone)]
pub struct AppState {
    pub allowed_origins: Vec<String>,
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub db_pool: Option<PgPool>,
    pub friend_requests: Arc<RwLock<HashMap<String, FriendRequestRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
    pub node_fingerprint: String,
    pub session_signing_key: String,
    pub sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
}

impl AppState {
    pub fn new(
        node_fingerprint: String,
        allowed_origins: Vec<String>,
        session_signing_key: String,
    ) -> Self {
        Self {
            allowed_origins,
            auth_challenges: Arc::default(),
            db_pool: None,
            friend_requests: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
            node_fingerprint,
            session_signing_key,
            sessions: Arc::default(),
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
            "hexrelay-dev-signing-key-change-me".to_string(),
        )
    }
}
