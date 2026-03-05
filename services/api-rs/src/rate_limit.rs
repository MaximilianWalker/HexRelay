use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Default)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, Vec<u64>>>>,
}

impl RateLimiter {
    pub fn allow(&self, scope: &str, key: &str, limit: usize, window_seconds: u64) -> bool {
        if limit == 0 {
            return true;
        }

        let now = now_unix_seconds();
        let oldest = now.saturating_sub(window_seconds);
        let bucket_key = format!("{scope}:{key}");

        let mut guard = self
            .entries
            .write()
            .expect("acquire rate limiter write lock");
        let bucket = guard.entry(bucket_key).or_default();
        bucket.retain(|timestamp| *timestamp >= oldest);

        if bucket.len() >= limit {
            return false;
        }

        bucket.push(now);
        true
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}
