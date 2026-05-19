use std::sync::atomic::{AtomicU64, Ordering};

use communication_core::observability::{render_prometheus_counters, MetricLabel, MetricSample};

#[derive(Debug, Default)]
pub struct RealtimeMetrics {
    websocket_upgrade_accepted: AtomicU64,
    websocket_upgrade_origin_disallowed: AtomicU64,
    websocket_upgrade_rate_limited: AtomicU64,
    websocket_upgrade_dev_fault_drop: AtomicU64,
    websocket_upgrade_session_invalid: AtomicU64,
    websocket_upgrade_connection_cap_reached: AtomicU64,
    websocket_upgrade_failed_after_slot: AtomicU64,
    dm_envelope_dispatch_accepted: AtomicU64,
    dm_envelope_dispatch_invalid: AtomicU64,
    dm_envelope_dispatch_auth_failed: AtomicU64,
    channel_created_accepted: AtomicU64,
    channel_created_failed: AtomicU64,
    channel_created_auth_failed: AtomicU64,
    channel_updated_accepted: AtomicU64,
    channel_updated_failed: AtomicU64,
    channel_updated_auth_failed: AtomicU64,
    channel_deleted_accepted: AtomicU64,
    channel_deleted_failed: AtomicU64,
    channel_deleted_auth_failed: AtomicU64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebsocketUpgradeOutcome {
    Accepted,
    OriginDisallowed,
    RateLimited,
    DevFaultDrop,
    SessionInvalid,
    ConnectionCapReached,
    FailedAfterSlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmEnvelopeDispatchOutcome {
    Accepted,
    Invalid,
    AuthFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerChannelDispatchEvent {
    Created,
    Updated,
    Deleted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerChannelDispatchOutcome {
    Accepted,
    Failed,
    AuthFailed,
}

impl RealtimeMetrics {
    pub fn record_websocket_upgrade(&self, outcome: WebsocketUpgradeOutcome) {
        match outcome {
            WebsocketUpgradeOutcome::Accepted => self
                .websocket_upgrade_accepted
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::OriginDisallowed => self
                .websocket_upgrade_origin_disallowed
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::RateLimited => self
                .websocket_upgrade_rate_limited
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::DevFaultDrop => self
                .websocket_upgrade_dev_fault_drop
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::SessionInvalid => self
                .websocket_upgrade_session_invalid
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::ConnectionCapReached => self
                .websocket_upgrade_connection_cap_reached
                .fetch_add(1, Ordering::Relaxed),
            WebsocketUpgradeOutcome::FailedAfterSlot => self
                .websocket_upgrade_failed_after_slot
                .fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn record_dm_envelope_dispatch(&self, outcome: DmEnvelopeDispatchOutcome) {
        match outcome {
            DmEnvelopeDispatchOutcome::Accepted => self
                .dm_envelope_dispatch_accepted
                .fetch_add(1, Ordering::Relaxed),
            DmEnvelopeDispatchOutcome::Invalid => self
                .dm_envelope_dispatch_invalid
                .fetch_add(1, Ordering::Relaxed),
            DmEnvelopeDispatchOutcome::AuthFailed => self
                .dm_envelope_dispatch_auth_failed
                .fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn record_server_channel_dispatch(
        &self,
        event: ServerChannelDispatchEvent,
        outcome: ServerChannelDispatchOutcome,
    ) {
        match (event, outcome) {
            (ServerChannelDispatchEvent::Created, ServerChannelDispatchOutcome::Accepted) => self
                .channel_created_accepted
                .fetch_add(1, Ordering::Relaxed),
            (ServerChannelDispatchEvent::Created, ServerChannelDispatchOutcome::Failed) => {
                self.channel_created_failed.fetch_add(1, Ordering::Relaxed)
            }
            (ServerChannelDispatchEvent::Created, ServerChannelDispatchOutcome::AuthFailed) => self
                .channel_created_auth_failed
                .fetch_add(1, Ordering::Relaxed),
            (ServerChannelDispatchEvent::Updated, ServerChannelDispatchOutcome::Accepted) => self
                .channel_updated_accepted
                .fetch_add(1, Ordering::Relaxed),
            (ServerChannelDispatchEvent::Updated, ServerChannelDispatchOutcome::Failed) => {
                self.channel_updated_failed.fetch_add(1, Ordering::Relaxed)
            }
            (ServerChannelDispatchEvent::Updated, ServerChannelDispatchOutcome::AuthFailed) => self
                .channel_updated_auth_failed
                .fetch_add(1, Ordering::Relaxed),
            (ServerChannelDispatchEvent::Deleted, ServerChannelDispatchOutcome::Accepted) => self
                .channel_deleted_accepted
                .fetch_add(1, Ordering::Relaxed),
            (ServerChannelDispatchEvent::Deleted, ServerChannelDispatchOutcome::Failed) => {
                self.channel_deleted_failed.fetch_add(1, Ordering::Relaxed)
            }
            (ServerChannelDispatchEvent::Deleted, ServerChannelDispatchOutcome::AuthFailed) => self
                .channel_deleted_auth_failed
                .fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn render_prometheus(&self) -> String {
        render_prometheus_counters(&[
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "accepted",
                self.websocket_upgrade_accepted.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "origin_disallowed",
                self.websocket_upgrade_origin_disallowed
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "rate_limited",
                self.websocket_upgrade_rate_limited.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "dev_fault_drop",
                self.websocket_upgrade_dev_fault_drop
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "session_invalid",
                self.websocket_upgrade_session_invalid
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "connection_cap_reached",
                self.websocket_upgrade_connection_cap_reached
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_websocket_upgrade_total",
                "Realtime websocket upgrade attempts by outcome.",
                "outcome",
                "failed_after_slot",
                self.websocket_upgrade_failed_after_slot
                    .load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_dm_envelope_dispatch_total",
                "Realtime encrypted DM envelope dispatch attempts by outcome.",
                "outcome",
                "accepted",
                self.dm_envelope_dispatch_accepted.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_dm_envelope_dispatch_total",
                "Realtime encrypted DM envelope dispatch attempts by outcome.",
                "outcome",
                "invalid",
                self.dm_envelope_dispatch_invalid.load(Ordering::Relaxed),
            ),
            sample(
                "hexrelay_realtime_dm_envelope_dispatch_total",
                "Realtime encrypted DM envelope dispatch attempts by outcome.",
                "outcome",
                "auth_failed",
                self.dm_envelope_dispatch_auth_failed
                    .load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "created",
                "accepted",
                self.channel_created_accepted.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "created",
                "failed",
                self.channel_created_failed.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "created",
                "auth_failed",
                self.channel_created_auth_failed.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "updated",
                "accepted",
                self.channel_updated_accepted.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "updated",
                "failed",
                self.channel_updated_failed.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "updated",
                "auth_failed",
                self.channel_updated_auth_failed.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "deleted",
                "accepted",
                self.channel_deleted_accepted.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "deleted",
                "failed",
                self.channel_deleted_failed.load(Ordering::Relaxed),
            ),
            server_channel_sample(
                "deleted",
                "auth_failed",
                self.channel_deleted_auth_failed.load(Ordering::Relaxed),
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

fn server_channel_sample(
    event: &'static str,
    outcome: &'static str,
    value: u64,
) -> MetricSample<'static> {
    MetricSample::counter(
        "hexrelay_realtime_server_channel_dispatch_total",
        "Realtime server-channel dispatch attempts by event and outcome.",
        vec![
            MetricLabel::new("event", event),
            MetricLabel::new("outcome", outcome),
        ],
        value,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_realtime_metrics_for_alertable_outcomes() {
        let metrics = RealtimeMetrics::default();
        metrics.record_websocket_upgrade(WebsocketUpgradeOutcome::Accepted);
        metrics.record_dm_envelope_dispatch(DmEnvelopeDispatchOutcome::Invalid);
        metrics.record_server_channel_dispatch(
            ServerChannelDispatchEvent::Created,
            ServerChannelDispatchOutcome::Failed,
        );

        let rendered = metrics.render_prometheus();

        assert!(
            rendered.contains("hexrelay_realtime_websocket_upgrade_total{outcome=\"accepted\"} 1")
        );
        assert!(rendered
            .contains("hexrelay_realtime_dm_envelope_dispatch_total{outcome=\"invalid\"} 1"));
        assert!(rendered.contains(
            "hexrelay_realtime_server_channel_dispatch_total{event=\"created\",outcome=\"failed\"} 1"
        ));
    }
}
