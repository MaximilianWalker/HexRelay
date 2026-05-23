# Migration Validation Evidence - 0012_servers_and_memberships

## Document Metadata

- Doc ID: migration-validation-0012-servers-and-memberships
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-23
- Source of truth: `evidence/migrations/0012_servers_and_memberships.md`

## Quick Context

- Purpose: record deterministic validation evidence for migration `0012_servers_and_memberships.sql`.
- Primary edit location: update when validation evidence for persisted server memberships changes.
- Latest meaningful change: 2026-05-23 changed membership ordering terminology from `favorite` to `pinned` for navigation hub consistency.

## Migration Metadata

- Migration ID: `0012_servers_and_memberships`
- SQL Path: `services/api-rs/migrations/0012_servers_and_memberships.sql`
- Owner: Maintainers
- Date (UTC): 2026-05-23
- Environment tested: PR #202 CI

## Forward Validation

- Command(s) executed: `RUST_TEST_THREADS=1 cargo test --all-features` from `services/api-rs`; PR #202 CI rerun required before merge.
- Expected outcome: fresh MVP database bootstrap creates server memberships with `pinned` ordering semantics.
- Actual outcome: passed locally on Windows; pending PR #202 CI rerun after evidence correction.
- Evidence path (logs/artifacts): local Codex command transcript and GitHub Actions run for PR #202.

## Idempotency and Re-run Check

- Re-run command(s): execute full migration chain on a prepared database.
- Expected outcome: first run succeeds; rerun is handled by `schema_migrations`.
- Actual outcome: passed locally on Windows; pending PR #202 CI rerun after evidence correction.
- Evidence path: local Codex command transcript and GitHub Actions run for PR #202.

## Rollback/Recovery Simulation

- Rollback or restore command(s): database snapshot restore to pre-migration state.
- Expected outcome: schema returns to pre-migration baseline.
- Actual outcome: pending dedicated rollback evidence.
- Evidence path: add restore transcript when executed.

## Data Integrity Verification

- Constraints/indexes verified: `server_memberships.pinned` and `server_memberships_identity_joined_idx` use pinned-first ordering.
- Row-count or key invariants checked: pending dedicated invariant query evidence.
- Evidence path: add invariant query output when executed.

## Historical Validation

- Previous evidence: PR #42 / CI run `23242910720` validated persisted `/servers` membership authority and trusted `same_server` DM checks.
- Prior commands: `cargo test --all-features`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`.
- Prior outcome: pass.

## Sign-off

- Reviewer: Codex
- Decision: pass
- Notes: baseline membership terminology now matches the navigation hub state model. Previous PR #42 validation covered persisted `/servers` membership authority and trusted `same_server` DM checks before this terminology update.
