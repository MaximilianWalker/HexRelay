# API Service

Rust HTTP API service scaffold for HexRelay.

## Runtime Position

- Runs as server-side logic in both supported modes:
  - desktop local-first mode (local sidecar/runtime component)
  - dedicated server mode (headless node deployment)
- Never shipped as client-bundled browser code.

## Authority

- Runtime/deployment authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`.
- Whole-system overview authority: `docs/architecture/01-system-overview.md`.
- Runtime config authority: `docs/reference/runtime-config-reference.md`.
- Operational procedure authority: `docs/operations/01-mvp-runbook.md`.
- This file is service-local quickstart context only.

- Run: `cargo run -p api-rs`
- Health: `http://127.0.0.1:8080/health`
- Test: `cargo test -p api-rs`

## Security Defaults

- Runtime auth transport: HttpOnly `hexrelay_session` cookie + CSRF double-submit checks on authenticated mutation routes.
- API abuse controls use DB-backed fixed-window counters (`rate_limit_counters`) when Postgres is available.

## Current Module Layout

- `src/app/`: router/state/config composition entrypoints
- `src/transport/http/handlers/`: HTTP route handlers by feature
- `src/transport/http/middleware/`: auth and rate-limit middleware helpers
- `src/domain/`: feature validation/business transition logic
- `src/infra/db/repos/`: SQL and persistence adapters
- `src/infra/crypto/`: session token signing/validation
- `src/shared/`: shared error primitives
- `src/tests/integration/`: API integration-focused unit tests
