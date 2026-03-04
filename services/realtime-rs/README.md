# Realtime Service

Rust realtime/signaling service scaffold for HexRelay.

## Runtime Position

- Runs as server-side realtime logic in both supported modes:
  - desktop local-first mode (local sidecar/runtime component)
  - dedicated server mode (headless node deployment)
- Never shipped as client-bundled browser code.

## Authority

- Runtime/deployment authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`.
- Operational procedure authority: `docs/operations/01-mvp-runbook.md`.
- This file is service-local quickstart context only.

- Run: `cargo run -p realtime-rs`
- Health: `http://127.0.0.1:8081/health`
- WebSocket: `ws://127.0.0.1:8081/ws`
- Test: `cargo test -p realtime-rs`
