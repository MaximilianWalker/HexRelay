use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::models::RegisteredIdentityKey;

#[derive(Clone, Default)]
pub struct AppState {
    pub identity_keys: Arc<RwLock<HashMap<String, RegisteredIdentityKey>>>,
}
