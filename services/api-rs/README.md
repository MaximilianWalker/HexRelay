# API Service

Rust HTTP API service scaffold for HexRelay.

## Runtime Position

- Runs as server-side logic in both supported modes:
  - desktop local-first mode (local sidecar/runtime component)
  - dedicated server mode (headless node deployment)
- Never shipped as client-bundled browser code.

## Authority

- Runtime/deployment authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`.
- Operational procedure authority: `docs/operations/01-mvp-runbook.md`.
- This file is service-local quickstart context only.

- Run: `cargo run -p api-rs`
- Health: `http://127.0.0.1:8080/health`
- Test: `cargo test -p api-rs`
