use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_BUCKETS: usize = 10_000;

#[derive(Default)]
struct Bucket {
    last_seen: u64,
    timestamps: Vec<u64>,
}

#[derive(Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, Bucket>>>,
    last_cleanup_epoch: Arc<AtomicU64>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            entries: Arc::default(),
            last_cleanup_epoch: Arc::new(AtomicU64::new(u64::MAX)),
        }
    }
}

impl RateLimiter {
    pub fn allow(&self, scope: &str, key: &str, limit: usize, window_seconds: u64) -> bool {
        if limit == 0 {
            return true;
        }

        if window_seconds == 0 {
            return false;
        }

        let now = now_unix_seconds();
        let oldest = now.saturating_sub(window_seconds);
        let cleanup_epoch = now / window_seconds;
        let bucket_key = format!("{scope}:{key}");

        let mut guard = self
            .entries
            .write()
            .expect("acquire rate limiter write lock");

        if self
            .last_cleanup_epoch
            .swap(cleanup_epoch, Ordering::AcqRel)
            != cleanup_epoch
        {
            guard.retain(|_, bucket| {
                bucket.timestamps.retain(|timestamp| *timestamp >= oldest);
                !bucket.timestamps.is_empty()
            });
        }

        if guard.len() >= MAX_BUCKETS && !guard.contains_key(&bucket_key) {
            if let Some(candidate) = guard
                .iter()
                .min_by_key(|(_, bucket)| bucket.last_seen)
                .map(|(existing_key, _)| existing_key.clone())
            {
                guard.remove(&candidate);
            }
        }

        let bucket = guard.entry(bucket_key).or_default();
        bucket.timestamps.retain(|timestamp| *timestamp >= oldest);
        bucket.last_seen = now;

        if bucket.timestamps.len() >= limit {
            return false;
        }

        bucket.timestamps.push(now);
        true
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::RateLimiter;

    #[test]
    fn limiter_rejects_after_limit_within_window() {
        let limiter = RateLimiter::default();
        assert!(limiter.allow("ws", "key-a", 1, 60));
        assert!(!limiter.allow("ws", "key-a", 1, 60));
    }

    #[test]
    fn limiter_allows_unlimited_when_limit_zero() {
        let limiter = RateLimiter::default();
        assert!(limiter.allow("ws", "key-b", 0, 60));
        assert!(limiter.allow("ws", "key-b", 0, 60));
    }
}
