# Readiness Corrections Log

## Document Metadata

- Doc ID: readiness-corrections-log
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `docs/operations/readiness-corrections-log.md`

## Purpose

- Track recurring readiness findings, the concrete correction applied, and the rule/doc that now prevents recurrence.

## Entry Format

- Date (UTC)
- Area (`docs` | `api-rs` | `realtime-rs` | `ci` | `workflow`)
- Finding summary
- Correction applied (path references)
- Preventive rule/document update
- Status (`closed` | `watch`)

## Entries

- 2026-03-10 | `ci` | Rust toolchain policy drift between pinned version and stable preference | switched toolchain policy to `stable` in `rust-toolchain.toml` and `.github/workflows/ci.yml` | documented stable-first standard in `docs/operations/dev-prerequisites.md` and `docs/operations/contributor-guide.md` | `closed`
- 2026-03-10 | `workflow` | Integration smoke repeatedly failed with low-signal timeout output | added fail-fast health wait with process-liveness/log-tail behavior in `.github/workflows/ci.yml` and timeout control in `apps/web/scripts/e2e-smoke.mjs` | CI troubleshooting now tied to explicit logs and deterministic startup checks in workflow | `closed`
- 2026-03-10 | `api-rs` | Invite backfill test flake under parallel execution | removed marker short-circuit race in `services/api-rs/src/db.rs` and stabilized env-based config tests in `services/api-rs/src/config.rs` | added this log policy and AGENTS rule to prevent recurring rediscovery loops | `closed`
- 2026-03-10 | `docs` | Repeated findings around reproducibility and CI parity wording | aligned startup env contract and parity wording in `README.md`, `docs/operations/dev-prerequisites.md`, and `docs/operations/contributor-guide.md` | maintainers must update this log whenever readiness corrections land | `watch`
- 2026-03-10 | `docs` | Local CI parity wording omitted some required CI jobs | updated `docs/operations/contributor-guide.md` local parity section with migration evidence validation, Semgrep, and npm audit commands plus CI-only caveats | contributor guide now distinguishes local parity from CI-owned checks | `closed`
- 2026-03-10 | `docs` | Iteration log and evidence provenance requirements repeatedly drifted | updated `docs/planning/05-iteration-log.md`, `docs/testing/01-mvp-verification-matrix.md`, `docs/testing/observability-evidence-template.md`, and `evidence/README.md` with explicit provenance fields (`commit_sha`, `pr_number/run_id`, `generated_at_utc`) | provenance is now a required evidence contract fieldset | `closed`
- 2026-03-10 | `realtime-rs` | WS ingress allowed binary-frame message-rate bypass and opaque handshake failures | applied binary-frame rate limiting and machine-readable handshake rejection JSON in `services/realtime-rs/src/transport/ws/handlers/gateway.rs`; added connect peer fallback keying | realtime ingress policy now enforces consistent abuse signaling and avoids `src:unknown` collapse for direct clients | `closed`
- 2026-03-10 | `api-rs` | Startup/config and DB-test readiness findings were rediscovered | moved tracing init before config parse in `services/api-rs/src/main.rs`; improved DB test setup behavior in `services/api-rs/src/tests/mod.rs` | startup failures are logged deterministically and local DB test skips include explicit reason | `closed`
- 2026-03-10 | `realtime-rs` | Realtime config accepted zero/degenerate limits | added strict numeric guardrails and tests in `services/realtime-rs/src/config.rs` | runtime now fails fast on unsafe realtime limiter configuration | `closed`
- 2026-03-10 | `docs` | Runtime REST contract scope drifted from implemented endpoints | expanded `docs/contracts/runtime-rest-v1.openapi.yaml` to include `/health`, `/v1/servers`, `/v1/contacts`, and friends routes | runtime contract now reflects implemented router surface | `closed`
- 2026-03-10 | `workflow` | Realtime abuse controls across multi-instance deployments remain easy to misinterpret | added explicit process-local limiter scope note in `docs/operations/01-mvp-runbook.md` | deployment docs now require sticky routing or edge/global limiting for equivalent behavior | `watch`
- 2026-03-10 | `ci` | Coverage target repeatedly raised in audits without implementation context | documented current enforced threshold and increase policy in `docs/operations/contributor-guide.md` | threshold changes must ship with accompanying tests to avoid noisy regressions | `watch`
- 2026-03-10 | `docs` | Local migration-evidence parity command assumed `origin/master` and failed on non-master default branches | changed local parity command in `docs/operations/contributor-guide.md` to resolve default branch dynamically with safe `master` fallback | parity instructions are now branch-name agnostic | `closed`
- 2026-03-10 | `ci` | Evidence provenance contract existed in docs but lacked automated enforcement | added `scripts/validate-evidence-provenance.sh` and wired `evidence-provenance-check` in `.github/workflows/ci.yml`; updated parity docs | provenance drift now fails CI for changed iteration/operations evidence artifacts | `closed`
- 2026-03-10 | `api-rs` | Auth challenge endpoint leaked identity existence via explicit unknown-identity error path | changed challenge issuance flow in `services/api-rs/src/transport/http/handlers/auth.rs` to return challenge envelopes for unknown identities without persistence; updated integration test in `services/api-rs/src/tests/integration/auth_tests.rs` | unknown identity no longer returns direct existence signal in challenge endpoint | `closed`
- 2026-03-10 | `realtime-rs` | Session-validation upstream failures lacked actionable diagnostics | added structured warning logs for request/network, status, and decode failures in `services/realtime-rs/src/transport/ws/handlers/gateway.rs` | websocket auth triage now has concrete upstream failure reason categories | `closed`
- 2026-03-10 | `docs` | REST contract drift on friends direction enum and decline/cancel response semantics | aligned `docs/contracts/runtime-rest-v1.openapi.yaml` with implemented API behavior (`inbound/outbound`, `204` for decline/cancel) | runtime contract now reflects handler validation and status behavior | `closed`
- 2026-03-10 | `docs` | Tooling prerequisites omitted Python/pip despite local Semgrep parity path | added Python and pip minimums plus verification commands in `docs/operations/dev-prerequisites.md` | prerequisites now cover all documented local parity commands | `closed`
- 2026-03-10 | `docs` | Runbook startup verification claimed coturn health checks without deterministic procedure | narrowed baseline startup verification to Postgres/Redis/storage and added explicit conditional coturn validation note in `docs/operations/01-mvp-runbook.md` | runbook startup checks now match executable baseline steps | `closed`
- 2026-03-10 | `docs` | Onboarding lacked a compact deterministic pre-dev pass/fail path | added `Pre-Dev Gate (Deterministic)` checklist to `README.md` and clarified required vs CI-owned parity checks in `docs/operations/contributor-guide.md` | first-run contributor flow now has one compact success path with explicit expectations | `closed`
- 2026-03-10 | `api-rs` | DB persistence integration tests reused fixed identities and flaked across reruns | generated unique test identities in `services/api-rs/src/tests/integration/db_persistence_tests.rs` | DB-backed test flow now avoids duplicate-key collisions from prior runs | `closed`
- 2026-03-10 | `realtime-rs` | API preflight timeout messaging did not reflect actual wait behavior | made startup API health wait deadline deterministic in `services/realtime-rs/src/main.rs` with bounded elapsed-time reporting | startup timeout behavior and error messaging now stay aligned | `closed`
- 2026-03-10 | `api-rs` | Auth challenge distributed rate-limit keying weakened when proxy headers were disabled | included socket `ConnectInfo` fallback keying and API serve connect-info wiring in `services/api-rs/src/main.rs` and `services/api-rs/src/transport/http/handlers/auth.rs` | rate-limit source dimension now remains informative without trusted proxy headers | `closed`
- 2026-03-10 | `realtime-rs` | Realtime error-code contract lacked explicit code taxonomy | constrained AsyncAPI `ErrorDataV1.code` to implemented runtime error codes in `docs/contracts/realtime-events-runtime-v1.asyncapi.yaml` | websocket error taxonomy is now explicit and versioned in contract docs | `closed`
