# Realtime Service

Rust realtime/signaling service scaffold for HexRelay.

## Runtime Position

- Runs as server-side realtime logic in both supported modes:
  - desktop local-first mode (local sidecar/runtime component)
  - dedicated server mode (headless node deployment)
- Never shipped as client-bundled browser code.

- Run: `cargo run -p realtime-rs`
- Health: `http://127.0.0.1:8081/health`
- WebSocket: `ws://127.0.0.1:8081/ws`
- Test: `cargo test -p realtime-rs`
