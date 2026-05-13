# Local Validation

Generated at: 2026-05-13T10:11Z

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo test -p communication-core --all-features` | pass, 62 tests |
| `cargo test -p realtime-rs --all-features websocket_presence` | pass, 4 tests |
| `cargo test -p realtime-rs --all-features websocket_channel_message` | pass, 3 tests |
| `cargo test -p realtime-rs --all-features websocket_channel_events_hydrate_late_device_without_prior_active_connection` | pass, 1 test |
| `cargo test -p api-rs --all-features api_server_channel_mutations_fan_out_over_realtime_websocket` | pass, 1 test |

Redis and Postgres were reachable on `127.0.0.1:6379` and `127.0.0.1:5432` for the targeted realtime/API integration validations.
