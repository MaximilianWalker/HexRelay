# Profile-Device Sync Test Inventory

## Presence

- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_presence_hydrates_late_profile_device_and_converges_live`
- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_presence_hydrates_late_profile_device_without_existing_viewer_connection`
- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_presence_rehydrates_missed_offline_transition_for_reconnecting_device`
- `services/realtime-rs/src/domain/presence.rs::tests::presence_edge_dispatch_uses_node_adapter_without_redis_on_current_thread`

## Server Channels

- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_channel_message_created_hydrates_late_profile_device`
- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_channel_events_hydrate_late_device_without_prior_active_connection`
- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_channel_message_updated_hydrates_late_profile_device`
- `services/realtime-rs/src/tests/ws_transport_tests.rs::websocket_channel_message_deleted_hydrates_late_profile_device`
- `services/api-rs/src/tests/integration/server_channel_messages_tests.rs::api_server_channel_mutations_fan_out_over_realtime_websocket`
- `services/api-rs/src/domain/server_channels/realtime.rs::tests::server_channel_dispatch_queue_sends_events_fifo`

## Adapter Boundary

- `crates/communication-core/src/tests/router_tests.rs`
- `crates/communication-core/src/tests/policy_tests.rs`
