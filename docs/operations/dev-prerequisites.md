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
- Latest meaningful change: 2026-03-10 added Python/pip parity prerequisites and a single copy-paste first-run env bootstrap block.

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
- Python: 3.10+ (required for local Semgrep parity command in contributor guide).
- pip: bundled with Python installation and required to install Semgrep locally.

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
python --version
python -m pip --version
```

Expected: commands resolve without errors and versions satisfy the required tooling section.

## Recommended Setup Flow

1. Install required tooling.
2. Run `npm run setup`.
3. Confirm `services/api-rs/.env` and `services/realtime-rs/.env` exist (created automatically from `*.env.example` on first `npm run run`).
4. Set local signing keys in `services/api-rs/.env` (`API_SESSION_SIGNING_KEYS` + `API_SESSION_SIGNING_KEY_ID`).
5. Run `npm run run` and confirm service startup succeeds.
6. Run `npm run test` before opening a PR.

- Reproducibility policy: dependency installation is lockfile-first (`npm ci` in setup scripts and CI).

## First-Run Env Bootstrap (Copy/Paste)

```bash
cat > services/api-rs/.env <<'EOF'
API_BIND=127.0.0.1:8080
API_ENVIRONMENT=development
API_NODE_FINGERPRINT=hexrelay-local-fingerprint
API_DATABASE_URL=postgres://hexrelay:hexrelay_dev_password@127.0.0.1:5432/hexrelay
API_ALLOWED_ORIGINS=http://localhost:3002,http://127.0.0.1:3002
API_TRUST_PROXY_HEADERS=false
API_SESSION_SIGNING_KEYS=v1:hexrelay-dev-signing-key-change-me
API_SESSION_SIGNING_KEY_ID=v1
API_SESSION_COOKIE_SECURE=false
API_SESSION_COOKIE_SAME_SITE=Lax
EOF

cat > services/realtime-rs/.env <<'EOF'
REALTIME_BIND=127.0.0.1:8081
REALTIME_API_BASE_URL=http://127.0.0.1:8080
REALTIME_REQUIRE_API_HEALTH_ON_START=true
REALTIME_TRUST_PROXY_HEADERS=false
REALTIME_ALLOWED_ORIGINS=http://localhost:3002,http://127.0.0.1:3002
REALTIME_WS_CONNECT_RATE_LIMIT=60
REALTIME_RATE_LIMIT_WINDOW_SECONDS=60
REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES=16384
REALTIME_WS_MESSAGE_RATE_LIMIT=120
REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS=60
REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY=3
EOF
```

- This bootstrap is for local development only.
- For dedicated/server deployments, use managed secrets and production-safe values.

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
