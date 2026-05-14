use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::{
    domain::dm::realtime::{dispatch_dm_envelope, DispatchDmEnvelopeInput},
    infra::db::repos::dm_repo,
    state::AppState,
};

pub const DM_OUTBOUND_FORWARD_MAX_ATTEMPTS: u32 = 5;
pub const DM_OUTBOUND_FORWARD_RETRY_LIMIT: u32 = 25;
pub const DM_OUTBOUND_FORWARD_STALE_ATTEMPT_SECONDS: i64 = 30;
pub const DM_OUTBOUND_FORWARD_RETRY_WORKER_INTERVAL_SECONDS: u64 = 5;
const DM_OUTBOUND_FORWARD_BASE_BACKOFF_SECONDS: i64 = 5;
const DM_OUTBOUND_FORWARD_MAX_BACKOFF_SECONDS: i64 = 300;
const MAX_FORWARDING_ERROR_LENGTH: usize = 512;

#[derive(Debug, Clone, Copy)]
pub struct DmOutboundForwardRetryConfig {
    pub limit: u32,
    pub max_attempts: u32,
    pub stale_attempt_seconds: i64,
}

impl Default for DmOutboundForwardRetryConfig {
    fn default() -> Self {
        Self {
            limit: DM_OUTBOUND_FORWARD_RETRY_LIMIT,
            max_attempts: DM_OUTBOUND_FORWARD_MAX_ATTEMPTS,
            stale_attempt_seconds: DM_OUTBOUND_FORWARD_STALE_ATTEMPT_SECONDS,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DmOutboundForwardRetryWorkerConfig {
    pub retry: DmOutboundForwardRetryConfig,
    pub interval: StdDuration,
}

impl Default for DmOutboundForwardRetryWorkerConfig {
    fn default() -> Self {
        Self {
            retry: DmOutboundForwardRetryConfig::default(),
            interval: StdDuration::from_secs(DM_OUTBOUND_FORWARD_RETRY_WORKER_INTERVAL_SECONDS),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DmOutboundForwardRetrySummary {
    pub scanned: u32,
    pub attempted: u32,
    pub forwarded: u32,
    pub retryable_failed: u32,
    pub terminal_failed: u32,
    pub skipped: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ForwardFailureClass {
    Retryable,
    Terminal,
}

pub fn spawn_dm_outbound_forward_retry_worker(
    state: AppState,
    config: DmOutboundForwardRetryWorkerConfig,
) -> JoinHandle<()> {
    tokio::spawn(run_dm_outbound_forward_retry_worker(state, config))
}

async fn run_dm_outbound_forward_retry_worker(
    state: AppState,
    config: DmOutboundForwardRetryWorkerConfig,
) {
    let interval_duration = config.interval.max(StdDuration::from_millis(1));
    info!(
        interval_ms = interval_duration.as_millis(),
        limit = config.retry.limit,
        max_attempts = config.retry.max_attempts,
        stale_attempt_seconds = config.retry.stale_attempt_seconds,
        "starting dm outbound forward retry worker"
    );

    let mut tick = tokio::time::interval(interval_duration);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tick.tick().await;

        if !static_peer_forwarding_ready(&state) {
            debug!(
                has_local_node_identity = state.local_node_identity.is_some(),
                static_peer_count = state.static_peer_registry.descriptors().len(),
                "skipping dm outbound forward retries until static-peer forwarding is configured"
            );
            continue;
        }

        match retry_due_dm_outbound_forwards(&state, config.retry).await {
            Ok(summary) if summary.scanned > 0 => {
                info!(
                    scanned = summary.scanned,
                    attempted = summary.attempted,
                    forwarded = summary.forwarded,
                    retryable_failed = summary.retryable_failed,
                    terminal_failed = summary.terminal_failed,
                    skipped = summary.skipped,
                    "processed due dm outbound forward retries"
                );
            }
            Ok(_) => {
                debug!("no due dm outbound forward retries found");
            }
            Err(error) => {
                warn!(
                    error = %error,
                    "failed to process dm outbound forward retries"
                );
            }
        }
    }
}

fn static_peer_forwarding_ready(state: &AppState) -> bool {
    state.local_node_identity.is_some() && !state.static_peer_registry.descriptors().is_empty()
}

pub async fn retry_due_dm_outbound_forwards(
    state: &AppState,
    config: DmOutboundForwardRetryConfig,
) -> Result<DmOutboundForwardRetrySummary, String> {
    let pool = state.db_pool.as_ref().ok_or_else(|| {
        "durable dm outbound forwarding requires configured database storage".to_string()
    })?;
    let max_attempts = config.max_attempts.max(1);
    let records = dm_repo::list_due_dm_outbound_forward_records(
        pool,
        config.limit.max(1),
        max_attempts,
        config.stale_attempt_seconds,
    )
    .await
    .map_err(|error| format!("load due DM outbound forwards: {error}"))?;
    let mut summary = DmOutboundForwardRetrySummary {
        scanned: records.len() as u32,
        ..Default::default()
    };

    for record in records {
        let Some(attempt_count) = dm_repo::mark_dm_outbound_forward_retry_started(
            pool,
            &record.sender_identity_id,
            &record.destination_node_id,
            &record.message_id,
            max_attempts,
            config.stale_attempt_seconds,
        )
        .await
        .map_err(|error| format!("claim DM outbound forward retry: {error}"))?
        else {
            summary.skipped += 1;
            continue;
        };

        summary.attempted += 1;
        let accepted_at = Utc::now().to_rfc3339();
        let target_device_ids = Vec::new();
        let dispatch_result = dispatch_dm_envelope(
            state,
            DispatchDmEnvelopeInput {
                destination_node_id: Some(&record.destination_node_id),
                message_id: &record.message_id,
                thread_id: &record.thread_id,
                sender_identity_id: &record.sender_identity_id,
                recipient_identity_id: &record.recipient_identity_id,
                ciphertext: &record.ciphertext,
                source_device_id: record.source_device_id.as_deref(),
                accepted_at: &accepted_at,
                delivery_cursor: record.delivery_cursor,
                target_device_ids: &target_device_ids,
            },
        )
        .await;

        match dispatch_result {
            Ok(()) => {
                let updated = dm_repo::mark_dm_outbound_forward_succeeded(
                    pool,
                    &record.sender_identity_id,
                    &record.destination_node_id,
                    &record.message_id,
                )
                .await
                .map_err(|error| format!("persist DM outbound forward retry success: {error}"))?;
                if updated {
                    summary.forwarded += 1;
                } else {
                    summary.skipped += 1;
                }
            }
            Err(error) => {
                let error_summary = forwarding_error_summary(&error);
                let failure_class = classify_forward_failure(&error_summary);
                let next_attempt_at = match failure_class {
                    ForwardFailureClass::Retryable => next_retry_attempt_at(
                        Utc::now(),
                        &record.destination_node_id,
                        &record.message_id,
                        attempt_count,
                        max_attempts,
                    ),
                    ForwardFailureClass::Terminal => None,
                };
                dm_repo::mark_dm_outbound_forward_failed(
                    pool,
                    &record.sender_identity_id,
                    &record.destination_node_id,
                    &record.message_id,
                    &error_summary,
                    next_attempt_at,
                )
                .await
                .map_err(|error| format!("persist DM outbound forward retry failure: {error}"))?;

                match failure_class {
                    ForwardFailureClass::Retryable if next_attempt_at.is_some() => {
                        summary.retryable_failed += 1;
                    }
                    _ => {
                        summary.terminal_failed += 1;
                    }
                }
            }
        }
    }

    Ok(summary)
}

pub fn next_retry_attempt_at(
    now: DateTime<Utc>,
    destination_node_id: &str,
    message_id: &str,
    attempt_count: u32,
    max_attempts: u32,
) -> Option<DateTime<Utc>> {
    if attempt_count >= max_attempts.max(1) {
        return None;
    }

    let exponent = attempt_count.saturating_sub(1).min(10);
    let multiplier = 1_i64.checked_shl(exponent).unwrap_or(i64::MAX);
    let base_delay = DM_OUTBOUND_FORWARD_BASE_BACKOFF_SECONDS
        .saturating_mul(multiplier)
        .min(DM_OUTBOUND_FORWARD_MAX_BACKOFF_SECONDS);
    let jitter_window = (base_delay / 5).max(1);
    let jitter = (seahash::hash(format!("{destination_node_id}:{message_id}").as_bytes())
        % u64::try_from(jitter_window + 1).unwrap_or(1)) as i64;

    Some(now + ChronoDuration::seconds(base_delay + jitter))
}

pub fn next_retry_attempt_after_failure(
    now: DateTime<Utc>,
    destination_node_id: &str,
    message_id: &str,
    attempt_count: u32,
    max_attempts: u32,
    error: &str,
) -> Option<DateTime<Utc>> {
    match classify_forward_failure(error) {
        ForwardFailureClass::Retryable => next_retry_attempt_at(
            now,
            destination_node_id,
            message_id,
            attempt_count,
            max_attempts,
        ),
        ForwardFailureClass::Terminal => None,
    }
}

pub fn forwarding_error_summary(error: &str) -> String {
    let trimmed = error.trim();
    if trimmed.len() <= MAX_FORWARDING_ERROR_LENGTH {
        return trimmed.to_string();
    }

    trimmed.chars().take(MAX_FORWARDING_ERROR_LENGTH).collect()
}

fn classify_forward_failure(error: &str) -> ForwardFailureClass {
    let normalized = error.to_ascii_lowercase();
    if normalized.starts_with("plan dm envelope route:")
        || normalized.contains("local node identity")
        || normalized.contains("local node private key")
        || normalized.contains("destination node descriptor has no forwarding address")
        || normalized.contains("destination node descriptor address must")
        || normalized.contains("rejected with status 400")
        || normalized.contains("rejected with status 401")
        || normalized.contains("rejected with status 403")
        || normalized.contains("rejected with status 404")
        || normalized.contains("rejected with status 409")
        || normalized.contains("rejected with status 422")
    {
        ForwardFailureClass::Terminal
    } else {
        ForwardFailureClass::Retryable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_stops_at_max_attempts() {
        let now = Utc::now();

        assert!(next_retry_attempt_at(now, "node-a", "msg-a", 5, 5).is_none());
    }

    #[test]
    fn retry_backoff_uses_stable_bounded_jitter() {
        let now = Utc::now();

        let first = next_retry_attempt_at(now, "node-a", "msg-a", 2, 5).expect("next attempt");
        let second = next_retry_attempt_at(now, "node-a", "msg-a", 2, 5).expect("next attempt");

        assert_eq!(first, second);
        let delay = first - now;
        assert!(delay >= ChronoDuration::seconds(10));
        assert!(delay <= ChronoDuration::seconds(12));
    }

    #[test]
    fn route_policy_errors_are_terminal() {
        assert_eq!(
            classify_forward_failure("plan DM envelope route: static peer route unavailable"),
            ForwardFailureClass::Terminal
        );
        assert!(next_retry_attempt_after_failure(
            Utc::now(),
            "node-a",
            "msg-a",
            1,
            5,
            "plan DM envelope route: static peer route unavailable"
        )
        .is_none());
        assert_eq!(
            classify_forward_failure(
                "node-forwarded DM envelope rejected with status 500 Internal Server Error"
            ),
            ForwardFailureClass::Retryable
        );
    }
}
