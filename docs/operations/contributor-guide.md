# HexRelay Contributor Guide

## Document Metadata

- Doc ID: contributor-guide
- Owner: Maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-11
- Source of truth: `docs/operations/contributor-guide.md`

## Quick Context

- Primary edit location for contribution workflow, docs QA checks, and PR hygiene.
- Keep this aligned with `docs/README.md` source-of-truth ownership rules.
- Latest meaningful change: 2026-03-11 added contract/docs-index parity checks and updated local parity commands to include them.

## Purpose

- Define the default contribution workflow for MVP-phase development.
- Keep quality gates deterministic without slowing delivery.

## Repository State

- Current state includes active implementation across web, API, and realtime services.
- Primary product runtime target is bundled desktop local-first operation.
- Dedicated server mode remains a supported path and should be preserved in architecture/API decisions.

## Local Development Prerequisites

- Before first setup, verify required local tooling versions in `docs/operations/dev-prerequisites.md`.
- Rust toolchain follows latest stable via `rust-toolchain.toml`; run `rustup toolchain install stable` if local toolchain is missing.

## Branch and PR Workflow

- Use short-lived branches from `master`.
- Suggested branch naming: `feat/<scope>`, `fix/<scope>`, `docs/<scope>`, `chore/<scope>`.
- Keep each PR scoped to one main task or one coherent doc update.
- Reference the task ID as defined in the active sprint board in PR title/body when applicable.

## Commit Policy

- Keep commits focused and reviewable.
- Include DCO sign-off on each commit (`Signed-off-by:` trailer).
- Follow the repository license/contribution baseline: AGPL-3.0 and DCO, no CLA for MVP.

## Validation Expectations

- For docs-only changes:
  - Verify links and paths resolve.
  - Keep metadata and `last_updated` fields accurate.
  - Confirm canonical source-of-truth boundaries are still respected (no duplicate authority across docs).
- For code changes:
  - Run lint, tests, and build for touched projects.
  - Run `npm run security` before opening a PR.
  - Keep security-sensitive data out of logs and fixtures.

## Security Tooling Baseline

- `cargo-audit` is pinned to `0.22.0` via `scripts/ensure-cargo-audit.sh` and CI uses the same version.
- If `npm run setup` fails installing `cargo-audit` because Rust is too old, run `rustup update stable` and retry setup.

## CI Expectations

- GitHub Actions workflow `/.github/workflows/ci.yml` is the canonical MVP gate for Rust and web checks.
- Required jobs include `security-audit`, `rust-check`, `web-check`, `migration-evidence-check`, `evidence-provenance-check`, `contract-parity-check`, `docs-index-freshness-check`, `rust-coverage-gate`, and `integration-smoke`.
- Current enforced backend coverage threshold is 80% and must remain paired with meaningful test additions when enforcement changes.
- Rust gate runs `fmt`, `clippy`, and `test` for `services/api-rs` and `services/realtime-rs`.
- Web gate runs `lint`, `test:coverage`, and `build` for `apps/web`.
- Integration smoke always uploads CI evidence artifacts at `evidence/ci/<run_id>/`.
- Missing required lockfiles or missing `lint`/`test:coverage`/`build` scripts fail CI with actionable errors.

Non-localizable CI checks:
- `migration-evidence-check` requires PR base/head SHAs from CI context.
- `integration-smoke` artifact upload path is CI-owned (`evidence/ci/<run_id>/`).

## Local CI Parity (Pre-PR)

Required local checks (run before opening PR):
- `npm run security`
- `npm run test`
- `./scripts/validate-migration-evidence.sh "$BASE_SHA" "$HEAD_SHA"`
- `./scripts/validate-evidence-provenance.sh "$BASE_SHA" "$HEAD_SHA"`
- `./scripts/validate-contract-parity.sh "$BASE_SHA" "$HEAD_SHA"`
- `./scripts/validate-docs-index-freshness.sh "$BASE_SHA" "$HEAD_SHA"`
- Rust `fmt`/`clippy`/tests and coverage gate command
- Web `lint`/`test:coverage`/`build`

CI-owned checks (informational for local parity):
- CI artifact upload and retention under `evidence/ci/<run_id>/`
- PR-context dependent SHA resolution in workflow jobs

Run from repository root:

```bash
npm run security
npm run test
DEFAULT_BRANCH=$(git remote show origin | sed -n '/HEAD branch/s/.*: //p')
BASE_SHA=$(git merge-base "origin/${DEFAULT_BRANCH:-master}" HEAD 2>/dev/null || git rev-parse HEAD~1)
HEAD_SHA=$(git rev-parse HEAD)
./scripts/validate-migration-evidence.sh "$BASE_SHA" "$HEAD_SHA"
./scripts/validate-evidence-provenance.sh "$BASE_SHA" "$HEAD_SHA"
./scripts/validate-contract-parity.sh "$BASE_SHA" "$HEAD_SHA"
./scripts/validate-docs-index-freshness.sh "$BASE_SHA" "$HEAD_SHA"
python -m pip install semgrep
semgrep scan --config p/security-audit --error --exclude node_modules --exclude target
npm --prefix apps/web audit --omit=dev --audit-level=high
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p api-rs --all-features
cargo test -p realtime-rs --all-features
API_DATABASE_URL=postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay API_SESSION_SIGNING_KEYS=v1:ci-signing-key-hexrelay-12345 API_SESSION_SIGNING_KEY_ID=v1 cargo llvm-cov --workspace --all-features --fail-under-lines 80
npm --prefix apps/web run lint
npm --prefix apps/web run test:coverage
npm --prefix apps/web run build
```

- The `DEFAULT_BRANCH` fallback keeps local parity compatible with both `master` and `main` default-branch repositories.
- If no `origin` remote is available (fork/offline workflows), set `BASE_SHA=$(git rev-parse HEAD~1)` before running evidence validation scripts.

- `npm run test` is the fast local baseline; the explicit commands above mirror CI gates as closely as possible outside GitHub Actions context.
- If your change affects auth/realtime startup behavior, run `npm --prefix apps/web run e2e:smoke` after API and realtime are healthy.

## Local Happy Path and Triage

1. `npm run setup`
2. `npm run run`
3. Verify `curl -fsS "http://127.0.0.1:8080/health"` and `curl -fsS "http://127.0.0.1:8081/health"`
4. `npm --prefix apps/web run e2e:smoke`
5. If startup or smoke fails, follow `docs/operations/01-mvp-runbook.md` recovery and rollback sections.

## Docs QA Checklist

- Metadata block is present and complete (`Doc ID`, `Owner`, `Status`, `Scope`, `last_updated`, `Source of truth`).
- Canonical ownership is explicit in `docs/README.md` source-of-truth matrix.
- New links point to canonical indexes where possible (for example, iteration index over repeated board lists).
- Related documents section is updated when new canonical docs are introduced.
- Runtime/deployment wording matches `docs/architecture/adr-0002-runtime-deployment-modes.md` and does not introduce conflicting authority text.
- Recurring readiness findings are recorded and closed in `docs/operations/readiness-corrections-log.md`.

## PR Checklist

- Problem and intent are clear.
- Scope is minimal and matches the task.
- Related docs are updated in the same PR.
- Any architecture-impacting change includes an ADR in `docs/architecture/`.
- New terms are added to `docs/reference/glossary.md` when needed.
- Any `services/api-rs/migrations/*.sql` change includes an updated evidence artifact at `evidence/migrations/<migration>.md`.

## Release Hygiene (MVP)

- Merge only when required checks pass.
- Prefer merge cadence tied to iteration milestones.
- For risky changes, include rollback notes in PR description.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/product/01-mvp-plan.md`
- `docs/planning/05-iteration-log.md`
