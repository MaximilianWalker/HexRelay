# HexRelay Development Prerequisites

## Document Metadata

- Doc ID: dev-prerequisites
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `docs/operations/dev-prerequisites.md`

## Quick Context

- Primary edit location for local development toolchain minimums and setup verification steps.
- Keep this aligned with CI runtime assumptions in `.github/workflows/ci.yml`.
- Latest meaningful change: 2026-03-10 pinned Rust version and added curl to required local tooling for startup checks.

## Purpose

- Define the minimum local toolchain required to run setup, start services, and execute repository checks.

## Required Tooling

- Node.js: 20.x (matches CI baseline).
- npm: 10.x (or newer npm compatible with Node.js 20).
- Rust toolchain: 1.94.0 (pinned by `rust-toolchain.toml` with `rustfmt` and `clippy`).
- Docker Engine: 24.x or newer.
- Docker Compose CLI plugin: 2.x (`docker compose`, not legacy `docker-compose`).
- Bash: required for repository scripts in `scripts/*.sh`.
- curl: required for local health checks used by `scripts/run.sh`.

## Quick Verification

Run from repository root:

```bash
node --version
npm --version
rustc --version
cargo --version
docker --version
docker compose version
bash --version
curl --version
```

Expected: commands resolve without errors and versions satisfy the required tooling section.

## Recommended Setup Flow

1. Install required tooling.
2. Run `npm run setup`.
3. Run `npm run run` and confirm service startup succeeds.
4. Run `npm run test` before opening a PR.

## Related Documents

- `README.md`
- `docs/operations/contributor-guide.md`
- `.github/workflows/ci.yml`
