use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::PgPool;

const MAX_BUCKETS: usize = 10_000;
static LAST_DISTRIBUTED_CLEANUP_WINDOW: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct Bucket {
    last_seen: u64,
    timestamps: Vec<u64>,
}

#[derive(Clone, Default)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, Bucket>>>,
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

        guard.retain(|_, bucket| {
            bucket.timestamps.retain(|timestamp| *timestamp >= oldest);
            !bucket.timestamps.is_empty()
        });

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

pub async fn allow_distributed(
    pool: &PgPool,
    scope: &str,
    key: &str,
    limit: usize,
    window_seconds: u64,
) -> Result<bool, sqlx::Error> {
    if limit == 0 {
        return Ok(true);
    }

    if window_seconds == 0 {
        return Ok(false);
    }

    let now = now_unix_seconds();
    let window_start = now / window_seconds;
    let cleanup_before = window_start.saturating_sub(2);

    let last_cleanup_window = LAST_DISTRIBUTED_CLEANUP_WINDOW.load(Ordering::Acquire);
    if should_run_distributed_cleanup(window_start, last_cleanup_window)
        && LAST_DISTRIBUTED_CLEANUP_WINDOW
            .compare_exchange(
                last_cleanup_window,
                window_start,
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok()
    {
        sqlx::query("DELETE FROM rate_limit_counters WHERE window_start < $1")
            .bind(cleanup_before as i64)
            .execute(pool)
            .await?;
    }

    let count = sqlx::query_scalar::<_, i64>(
        "
        INSERT INTO rate_limit_counters (scope, limiter_key, window_start, count, updated_at)
        VALUES ($1, $2, $3, 1, NOW())
        ON CONFLICT (scope, limiter_key, window_start)
        DO UPDATE SET
          count = rate_limit_counters.count + 1,
          updated_at = NOW()
        RETURNING count::BIGINT
        ",
    )
    .bind(scope)
    .bind(key)
    .bind(window_start as i64)
    .fetch_one(pool)
    .await?;

    Ok((count as usize) <= limit)
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs()
}

fn should_run_distributed_cleanup(window_start: u64, last_cleanup_window: u64) -> bool {
    window_start >= last_cleanup_window.saturating_add(16)
}

#[cfg(test)]
mod tests {
    use super::{should_run_distributed_cleanup, RateLimiter};

    #[test]
    fn in_memory_limiter_rejects_after_limit_within_window() {
        let limiter = RateLimiter::default();

        assert!(limiter.allow("auth", "user-a", 1, 60));
        assert!(!limiter.allow("auth", "user-a", 1, 60));
    }

    #[test]
    fn in_memory_limiter_allows_unlimited_when_limit_zero() {
        let limiter = RateLimiter::default();

        assert!(limiter.allow("auth", "user-b", 0, 60));
        assert!(limiter.allow("auth", "user-b", 0, 60));
    }

    #[test]
    fn distributed_cleanup_runs_on_elapsed_window_interval() {
        assert!(!should_run_distributed_cleanup(31, 16));
        assert!(should_run_distributed_cleanup(32, 16));
        assert!(should_run_distributed_cleanup(48, 16));
    }
}
