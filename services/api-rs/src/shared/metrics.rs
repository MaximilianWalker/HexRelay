use std::sync::atomic::{AtomicU64, Ordering};

use communication_core::observability::{render_prometheus_counters, MetricLabel, MetricSample};

#[derive(Debug, Default)]
pub struct ApiMetrics {
    auth_verify_issued: AtomicU64,
    auth_verify_rejected: AtomicU64,
    auth_session_validation_accepted: AtomicU64,
    auth_session_validation_rejected: AtomicU64,
    dm_dispatch_accepted: AtomicU64,
    dm_dispatch_blocked: AtomicU64,
    dm_dispatch_forwarded: AtomicU64,
    dm_dispatch_failed: AtomicU64,
    server_channel_dispatch_enqueued: AtomicU64,
    server_channel_dispatch_failed: AtomicU64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthVerifyOutcome {
    Issued,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthSessionValidationOutcome {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmDispatchOutcome {
    Accepted,
    Blocked,
    Forwarded,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerChannelDispatchOutcome {
    Enqueued,
    Failed,
}

impl ApiMetrics {
    pub fn record_auth_verify(&self, outcome: AuthVerifyOutcome) {
        match outcome {
            AuthVerifyOutcome::Issued => self.auth_verify_issued.fetch_add(1, Ordering::Relaxed),
            AuthVerifyOutcome::Rejected => {
                self.auth_verify_rejected.fetch_add(1, Ordering::Relaxed)
            }
        };
    }

    pub fn record_auth_session_validation(&self, outcome: AuthSessionValidationOutcome) {
        match outcome {
            AuthSessionValidationOutcome::Accepted => self
                .auth_session_validation_accepted
                .fetch_add(1, Ordering::Relaxed),
            AuthSessionValidationOutcome::Rejected => self
                .auth_session_validation_rejected
                .fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn record_dm_dispatch(&self, outcome: DmDispatchOutcome) {
        match outcome {
            DmDispatchOutcome::Accepted => {
                self.dm_dispatch_accepted.fetch_add(1, Ordering::Relaxed)
            }
            DmDispatchOutcome::Blocked => self.dm_dispatch_blocked.fetch_add(1, Ordering::Relaxed),
            DmDispatchOutcome::Forwarded => {
                self.dm_dispatch_forwarded.fetch_add(1, Ordering::Relaxed)
            }
            DmDispatchOutcome::Failed => self.dm_dispatch_failed.fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn record_server_channel_dispatch(&self, outcome: ServerChannelDispatchOutcome) {
        match outcome {
            ServerChannelDispatchOutcome::Enqueued => self
                .server_channel_dispatch_enqueued
                .fetch_add(1, Ordering::Relaxed),
            ServerChannelDispatchOutcome::Failed => self
                .server_channel_dispatch_failed
                .fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn render_prometheus(&self) -> String {
        render_prometheus_counters(&[
            sample(
                "hexrelay_api_auth_verify_total",
                "API auth verification attempts by outcome.",
                "outcome",
                "issued",
                self.auth_verify_issued.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_auth_verify_total",
                "API auth verification attempts by outcome.",
                "outcome",
                "rejected",
                self.auth_verify_rejected.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_auth_session_validation_total",
                "API session validations by outcome.",
                "outcome",
                "accepted",
                self.auth_session_validation_accepted
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_auth_session_validation_total",
                "API session validations by outcome.",
                "outcome",
                "rejected",
                self.auth_session_validation_rejected
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_dm_dispatch_total",
                "API encrypted DM dispatch attempts by outcome.",
                "outcome",
                "accepted",
                self.dm_dispatch_accepted.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_dm_dispatch_total",
                "API encrypted DM dispatch attempts by outcome.",
                "outcome",
                "blocked",
                self.dm_dispatch_blocked.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_dm_dispatch_total",
                "API encrypted DM dispatch attempts by outcome.",
                "outcome",
                "forwarded",
                self.dm_dispatch_forwarded.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_dm_dispatch_total",
                "API encrypted DM dispatch attempts by outcome.",
                "outcome",
                "failed",
                self.dm_dispatch_failed.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_server_channel_dispatch_enqueue_total",
                "API server-channel realtime dispatch enqueue attempts by outcome.",
                "outcome",
                "enqueued",
                self.server_channel_dispatch_enqueued
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_api_server_channel_dispatch_enqueue_total",
                "API server-channel realtime dispatch enqueue attempts by outcome.",
                "outcome",
                "failed",
                self.server_channel_dispatch_failed.load(Ordering::Relaxed),
            ),
        ])
    }
}

fn sample(
    name: &'static str,
    help: &'static str,
    label_name: &'static str,
    label_value: &'static str,
    value: u64,
) -> MetricSample<'static> {
    MetricSample::counter(
        name,
        help,
        vec![MetricLabel::new(label_name, label_value)],
        value,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_api_metrics_for_alertable_outcomes() {
        let metrics = ApiMetrics::default();
        metrics.record_auth_verify(AuthVerifyOutcome::Issued);
        metrics.record_auth_session_validation(AuthSessionValidationOutcome::Rejected);
        metrics.record_dm_dispatch(DmDispatchOutcome::Forwarded);
        metrics.record_server_channel_dispatch(ServerChannelDispatchOutcome::Failed);

        let rendered = metrics.render_prometheus();

        assert!(rendered.contains("hexrelay_api_auth_verify_total{outcome=\"issued\"} 1"));
        assert!(
            rendered.contains("hexrelay_api_auth_session_validation_total{outcome=\"rejected\"} 1")
        );
        assert!(rendered.contains("hexrelay_api_dm_dispatch_total{outcome=\"forwarded\"} 1"));
        assert!(rendered
            .contains("hexrelay_api_server_channel_dispatch_enqueue_total{outcome=\"failed\"} 1"));
    }
}
