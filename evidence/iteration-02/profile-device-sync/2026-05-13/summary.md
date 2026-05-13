# Profile-Device Sync Evidence

## Scope

- Requirement IDs: `T3.3.2`, `T4.3.3`, `T4.3.4`
- Owner: Realtime/Core
- Evidence date: 2026-05-13
- Outcome: local validation passed, pending PR CI revalidation

## Covered Behavior

- Presence online/offline edges are routed through the shared `NodeClientTransport` adapter path before publishing to the Redis-backed realtime replay stream.
- Server-channel create/update/delete dispatch is routed through the shared `NodeClientTransport` adapter path from API persistence to realtime fanout.
- Presence events converge across multiple active profile devices and hydrate late or reconnecting devices by per-device cursor.
- Server-channel events converge across active profile devices, hydrate late devices by replay cursor, avoid duplicate replay on reconnect, and exclude read-denied members.

## Runtime Evidence

- `services/realtime-rs/src/domain/presence.rs` publishes presence edges through `send_via_node_dispatch_with_provenance(...)` with `CommunicationMode::Presence`.
- `services/api-rs/src/domain/server_channels/realtime.rs` dispatches server-channel events through `send_via_node_dispatch_with_provenance(...)` with `CommunicationMode::ServerChannel`.
- `services/realtime-rs/src/domain/presence.rs` and `services/realtime-rs/src/domain/channels.rs` persist replay entries and per-device cursors through the shared private replay-store helpers.
- `services/realtime-rs/src/transport/ws/handlers/gateway.rs` hydrates presence and channel backlogs before publishing the websocket connection's online presence edge.

## Test Evidence

- `websocket_presence_hydrates_late_profile_device_and_converges_live`
- `websocket_presence_hydrates_late_profile_device_without_existing_viewer_connection`
- `websocket_presence_rehydrates_missed_offline_transition_for_reconnecting_device`
- `websocket_channel_message_created_hydrates_late_profile_device`
- `websocket_channel_events_hydrate_late_device_without_prior_active_connection`
- `websocket_channel_message_updated_hydrates_late_profile_device`
- `websocket_channel_message_deleted_hydrates_late_profile_device`
- `api_server_channel_mutations_fan_out_over_realtime_websocket`
- `server_channel_dispatch_queue_sends_events_fifo`
- `presence_edge_dispatch_uses_node_adapter_without_redis_on_current_thread`

Local validation results are recorded in `outputs/local-validation.md`.

## Split Rationale

The deterministic selector chose the broader `T4.2-T4.3-server-channel-realtime` cluster. The smallest mergeable slice for this run is a closeout/evidence update because the runtime implementation and regression coverage already exist, while this board and evidence ledger still treated `T3.3.2`, `T4.3.3`, and `T4.3.4` as open.
