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
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub db_pool: Option<PgPool>,
    pub friend_requests: Arc<RwLock<HashMap<String, FriendRequestRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
    pub node_fingerprint: String,
    pub sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
}

impl AppState {
    pub fn new(node_fingerprint: String) -> Self {
        Self {
            auth_challenges: Arc::default(),
            db_pool: None,
            friend_requests: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
            node_fingerprint,
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
        Self::new("hexrelay-local-fingerprint".to_string())
    }
}
