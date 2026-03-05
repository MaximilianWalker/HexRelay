# HexRelay MVP Operations Runbook

## Document Metadata

- Doc ID: mvp-runbook
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-05
- Source of truth: `docs/operations/01-mvp-runbook.md`

## Quick Context

- Purpose: provide minimum operational procedures for MVP reliability and recovery.
- Primary edit location: update when deployment/recovery/incident steps change.
- Latest meaningful change: 2026-03-04 expanded dedicated-server procedures with concrete startup, health, restart, and TLS boundary assumptions.
  - 2026-03-05 security automation and CI evidence artifact collection baseline added.

## Core Procedures

- Startup verification: `docker compose up -d` + health checks for Postgres/Redis/storage/coturn.
- Mode selection:
  - Desktop local-first: user runs bundled app with local API/realtime services.
    - UI launch options: embedded desktop window or local browser against localhost.
  - Dedicated server: operator runs headless API/realtime services for remote clients.
- Incident triage:
  - auth failure spike
  - message delivery degradation
  - voice join degradation
- Recovery paths:
  - restart service scope (single service, full stack)
  - rotate leaked invite tokens
  - revoke compromised sessions

## Dedicated Server Baseline

- Scope: single-node headless deployment running `services/api-rs` + `services/realtime-rs` with shared Postgres.
- Runtime authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`.
- Abuse controls: API rate limits are enforced via shared Postgres counters to preserve limits across horizontally scaled API instances.
- Minimum environment:
  - API: `API_BIND`, `API_DATABASE_URL`, `API_SESSION_SIGNING_KEYS`, `API_SESSION_SIGNING_KEY_ID`, `API_ALLOWED_ORIGINS`, `API_SESSION_COOKIE_SECURE`, `API_SESSION_COOKIE_SAME_SITE`.
  - Realtime: `REALTIME_BIND`, `REALTIME_API_BASE_URL`, `REALTIME_ALLOWED_ORIGINS`, `REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES`, `REALTIME_WS_MESSAGE_RATE_LIMIT`, `REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS`, `REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY`.
- Startup sequence:
  1. Start database dependencies.
  2. Start API service and verify `GET /health` returns 200.
  3. Start realtime service and verify `GET /health` returns 200.
  4. Execute smoke path (`apps/web/scripts/e2e-smoke.mjs` or CI equivalent) before exposing service.

### Dedicated Server Bring-Up (Command Baseline)

1. Load environment values from `.env` (or export directly):

```bash
set -a; source .env; set +a
```

2. Start API service:

```bash
cargo run --manifest-path services/api-rs/Cargo.toml
```

3. In a second shell, verify API health:

```bash
curl -fsS "http://$API_BIND/health"
```

4. Start realtime service:

```bash
cargo run --manifest-path services/realtime-rs/Cargo.toml
```

5. In a third shell, verify realtime health:

```bash
curl -fsS "http://$REALTIME_BIND/health"
```

6. Run smoke validation against running services:

```bash
npm --prefix apps/web run e2e:smoke
```

## TLS and Network Boundary Assumptions

- External dedicated deployments terminate TLS at ingress/reverse proxy.
- API and realtime processes should not be exposed directly without TLS termination.
- For local desktop mode, loopback (`127.0.0.1`) bindings are the default trust boundary.

## Restart and Recovery Procedures

- Single-service restart:
  - Restart `api-rs` when auth/session or invite endpoints degrade.
  - Restart `realtime-rs` when websocket fanout/signaling degrades.
- Full runtime restart:
  - Restart both services if cross-service auth validation fails repeatedly.
- Post-restart validation:
  - API and realtime `/health` probes return 200.
  - Session validate endpoint works with existing active `hexrelay_session` cookie.
  - Realtime websocket auth handshake passes with valid session cookie (`hexrelay_session`).

### Rollback Procedure (Single Node)

1. Stop realtime service.
2. Stop API service.
3. Deploy previous known-good build artifacts.
4. Start API then realtime using the bring-up command baseline.
5. Re-run smoke validation and archive logs for incident evidence.

## Backup and Restore

- Back up database snapshots and object storage indexes.
- Verify restore quarterly in staging.
- Migration restore validation must include signature verification and reconcile logs.

### Restore Evidence Contract

- Store restore drill artifacts under `evidence/operations/restore-drills/<YYYY-MM-DD>/`.
- Minimum required files:
  - `restore-commands.txt` (executed commands in order)
  - `health-checks.txt` (`/health` and smoke outputs)
  - `migration-state.txt` (`schema_migrations` checksum snapshot)
  - `incident-notes.md` (what failed, what was fixed, final status)

## SLO Breach Response

- Trigger: KPI/SLO thresholds violated in benchmark profile.
- Action: open remediation task in active iteration board before sign-off.

## CI Security and Evidence Baseline

- Security gates run in CI:
  - `cargo audit --deny warnings`
  - `npm audit --omit=dev --audit-level=high`
  - `semgrep scan --config p/security-audit --error`
- Integration-smoke run collects evidence artifacts to `evidence/ci/<run_id>/` and uploads as CI artifact.

## Related Documents

- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/iterations/04-sprint-board.md`
