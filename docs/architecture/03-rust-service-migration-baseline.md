# Rust Service Migration Baseline

## Document Metadata

- Doc ID: rust-service-migration-baseline
- Owner: Architecture maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-06
- Source of truth: `docs/architecture/03-rust-service-migration-baseline.md`

## Purpose

- Freeze a concrete migration baseline before full crate-structure migration of `services/api-rs` and `services/realtime-rs`.
- Capture the exact current state, verification evidence, and current-file to target-module mapping.

## Frozen Baseline Snapshot

### CI Baseline

- Baseline run: `22745777330`
- URL: `https://github.com/MaximilianWalker/HexRelay/actions/runs/22745777330`
- Conclusion: `success`
- Completed at: `2026-03-06T02:11:34Z`
- Required checks all green:
  - `security-audit`
  - `web-check (apps/web)`
  - `rust-check (services/api-rs)`
  - `rust-check (services/realtime-rs)`
  - `rust-coverage-gate`
  - `integration-smoke (web->api->realtime)`
  - `migration-evidence-check`

### Coverage Baseline (workspace)

- Command: `cargo llvm-cov --workspace --all-features --fail-under-lines 65`
- Threshold: `65` lines (gate)
- Result: `pass`
- Total line coverage snapshot: `69.47%`
- Total region coverage snapshot: `70.98%`

## Current -> Target Mapping Matrix

### `services/api-rs/src`

| Current file/module | Target module path | Notes |
|---|---|---|
| `app.rs` | `app/router.rs` (plus `app/mod.rs`) | Keep as top-level router composition only |
| `main.rs` | `main.rs` + `app/config.rs` | Keep bootstrap thin; move config plumbing to `app/config.rs` |
| `state.rs` | `app/state.rs` | Runtime state wiring lives under app layer |
| `config.rs` | `app/config.rs` | Service config and env decoding |
| `handlers.rs` | `transport/http/handlers/health.rs` + `transport/http/handlers/mod.rs` | Keep only health and handler exports; feature handlers move out |
| `auth_handlers.rs` | `transport/http/handlers/auth.rs` + `transport/http/dto/auth.rs` | Split wire DTO concerns from domain logic |
| `invite_handlers.rs` | `transport/http/handlers/invites.rs` + `transport/http/dto/invites.rs` | Preserve endpoint behavior |
| `friend_request_handlers.rs` | `transport/http/handlers/friends.rs` + `transport/http/dto/friends.rs` | Already isolated from `handlers.rs`; next step is transport/domain split |
| `directory_handlers.rs` | `transport/http/handlers/directory.rs` + `transport/http/dto/directory.rs` | Keep query parsing in transport layer |
| `models.rs` | `domain/*/model.rs` + `transport/http/dto/*` + `shared/types.rs` | Decompose by ownership |
| `validation.rs` | `domain/*/validation.rs` | Feature-owned validation rules |
| `db.rs` | `infra/db/pool.rs` + `infra/db/migrations.rs` + `infra/db/repos/*.rs` | Separate pool/migration/repo responsibilities |
| `auth.rs` | `transport/http/middleware/auth.rs` + `domain/auth/service.rs` (selected helpers) | Request auth extraction vs domain behavior split |
| `rate_limit.rs` | `transport/http/middleware/rate_limit.rs` + `shared/types.rs` | Keep request-level enforcement in middleware |
| `session_token.rs` | `infra/crypto/session_token.rs` | Infrastructure cryptographic helper |
| `errors.rs` | `transport/http/error.rs` + `shared/errors.rs` | Transport mapping vs shared error primitives |
| `tests/mod.rs`, `tests/*_tests.rs` | `tests/*_tests.rs` (retained) and optional `tests/helpers.rs` | Current domain test split is baseline for next phases |
| `lib.rs` | `lib.rs` (module wiring only) | Keep no business logic in crate root |

### `services/realtime-rs/src`

| Current file/module | Target module path | Notes |
|---|---|---|
| `app.rs` | `app/router.rs` + `app/mod.rs` | Router and composition only |
| `main.rs` | `main.rs` + `app/config.rs` | Thin startup and settings wiring |
| `state.rs` | `app/state.rs` | Connection and runtime state |
| `config.rs` | `app/config.rs` | Service configuration ownership |
| `handlers.rs` | `transport/ws/handlers/*.rs` + `transport/ws/dto/*.rs` + `transport/ws/middleware/*.rs` | Split websocket handler hotspot by concern |
| `rate_limit.rs` | `transport/ws/middleware/rate_limit.rs` | Keep request/session enforcement in middleware |
| `lib.rs` | `lib.rs` (module wiring only) | No endpoint logic in crate root |

## Migration Rules for Next Stages

- `api-rs` migrates first; `realtime-rs` mirrors only after `api-rs` structure stabilizes in CI.
- Every stage must be no-behavior-change unless explicitly scoped as feature work.
- Every stage must pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`
  - `cargo llvm-cov --workspace --all-features --fail-under-lines 65`
  - `npm run security`
  - `npm run test`

## Related Documents

- `docs/architecture/adr-0003-rust-service-module-architecture.md`
- `docs/operations/contributor-guide.md`
- `services/api-rs/README.md`
- `services/realtime-rs/README.md`
