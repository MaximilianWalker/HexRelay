use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::models::{AuthChallengeRecord, InviteRecord, RegisteredIdentityKey, SessionRecord};

#[derive(Clone)]
pub struct AppState {
    pub auth_challenges: Arc<RwLock<HashMap<String, AuthChallengeRecord>>>,
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
    pub invites: Arc<RwLock<HashMap<String, InviteRecord>>>,
    pub node_fingerprint: String,
    pub sessions: Arc<RwLock<HashMap<String, SessionRecord>>>,
}

impl AppState {
    pub fn new(node_fingerprint: String) -> Self {
        Self {
            auth_challenges: Arc::default(),
            identity_keys: Arc::default(),
            invites: Arc::default(),
            node_fingerprint,
            sessions: Arc::default(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new("hexrelay-local-fingerprint".to_string())
    }
}
