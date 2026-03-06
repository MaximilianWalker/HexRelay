# HexRelay Development Prerequisites

## Document Metadata

- Doc ID: dev-prerequisites
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-06
- Source of truth: `docs/operations/dev-prerequisites.md`

## Purpose

- Define the minimum local toolchain required to run setup, start services, and execute repository checks.

## Required Tooling

- Node.js: 20.x (matches CI baseline).
- npm: 10.x (or newer npm compatible with Node.js 20).
- Rust toolchain: stable (pinned by `rust-toolchain.toml` with `rustfmt` and `clippy`).
- Docker Engine: 24.x or newer.
- Docker Compose CLI plugin: 2.x (`docker compose`, not legacy `docker-compose`).
- Bash: required for repository scripts in `scripts/*.sh`.

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
