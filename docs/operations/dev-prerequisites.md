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
- Latest meaningful change: 2026-03-10 standardized Rust policy and made local startup env loading deterministic via service `.env` files.

## Purpose

- Define the minimum local toolchain required to run setup, start services, and execute repository checks.

## Required Tooling

- Node.js: 20.x (matches CI baseline).
- npm: 10.x (or newer npm compatible with Node.js 20).
- Rust toolchain: latest stable channel (enforced by `rust-toolchain.toml` with `rustfmt` and `clippy`).
- Docker Engine: 24.x or newer.
- Docker Compose CLI plugin: 2.x (`docker compose`, not legacy `docker-compose`).
- Bash: required for repository scripts in `scripts/*.sh`.
- curl: required for local health checks used by `scripts/run.sh`.

### Platform Notes

- Windows: use Git Bash or WSL for `scripts/*.sh` commands.
- macOS/Linux: default system shell is sufficient.

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
3. Confirm `services/api-rs/.env` and `services/realtime-rs/.env` exist (created automatically from `*.env.example` on first `npm run run`).
4. Set local signing keys in `services/api-rs/.env` (`API_SESSION_SIGNING_KEYS` + `API_SESSION_SIGNING_KEY_ID`).
5. Run `npm run run` and confirm service startup succeeds.
6. Run `npm run test` before opening a PR.

## Canonical Local Runtime Env Contract

- `API_SESSION_SIGNING_KEYS` (required): keyring in `key_id:secret` format (set in `services/api-rs/.env`).
- `API_SESSION_SIGNING_KEY_ID` (required when keyring is used): active key ID present in `API_SESSION_SIGNING_KEYS`.
- `API_ENVIRONMENT` defaults to `development`; set `production` for dedicated deployments to enforce stricter config checks.
- `API_TRUST_PROXY_HEADERS` and `REALTIME_TRUST_PROXY_HEADERS` default to `false`; set to `true` only behind a trusted reverse proxy that sanitizes forwarded headers.
- Legacy fallback `API_SESSION_SIGNING_KEY` is supported for local compatibility only; prefer keyring form.

## Related Documents

- `README.md`
- `docs/operations/contributor-guide.md`
- `.github/workflows/ci.yml`
