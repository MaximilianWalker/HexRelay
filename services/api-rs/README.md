# API Service

Rust HTTP API service scaffold for HexRelay.

## Runtime Position

- Runs as server-side logic in both supported modes:
  - desktop local-first mode (local sidecar/runtime component)
  - dedicated server mode (headless node deployment)
- Never shipped as client-bundled browser code.

- Run: `cargo run -p api-rs`
- Health: `http://127.0.0.1:8080/health`
- Test: `cargo test -p api-rs`
