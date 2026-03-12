# HexRelay MVP Operations Runbook

## Document Metadata

- Doc ID: mvp-runbook
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-11
- Source of truth: `docs/operations/01-mvp-runbook.md`

## Quick Context

- Purpose: provide minimum operational procedures for MVP reliability and recovery.
- Primary edit location: update when deployment/recovery/incident steps change.
- Latest meaningful change: 2026-03-11 added bounded realtime auth-upstream outage grace controls and rollback guidance.
  - 2026-03-05 security automation and CI evidence artifact collection baseline added.

## Core Procedures

- Startup verification: `docker compose --env-file infra/.env -f infra/docker-compose.yml up -d` + health checks for Postgres/Redis/storage.
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
- Realtime limiter scope: websocket connect/message limits are process-local; multi-instance deployments must apply sticky routing or edge/global limiting to preserve equivalent abuse controls.
- Minimum environment:
  - API: `API_BIND`, `API_ENVIRONMENT`, `API_DATABASE_URL`, `API_SESSION_SIGNING_KEYS`, `API_SESSION_SIGNING_KEY_ID`, `API_ALLOWED_ORIGINS`, `API_TRUST_PROXY_HEADERS`, `API_SESSION_COOKIE_SECURE`, `API_SESSION_COOKIE_SAME_SITE`.
  - Realtime: `REALTIME_BIND`, `REALTIME_API_BASE_URL`, `REALTIME_REQUIRE_API_HEALTH_ON_START`, `REALTIME_TRUST_PROXY_HEADERS`, `REALTIME_ALLOWED_ORIGINS`, `REALTIME_WS_MAX_INBOUND_MESSAGE_BYTES`, `REALTIME_WS_MESSAGE_RATE_LIMIT`, `REALTIME_WS_MESSAGE_RATE_WINDOW_SECONDS`, `REALTIME_WS_MAX_CONNECTIONS_PER_IDENTITY`, `REALTIME_WS_AUTH_GRACE_SECONDS`, `REALTIME_WS_AUTH_CACHE_MAX_ENTRIES`.
- Startup sequence:
  1. Start database dependencies.
  2. Start API service and verify `GET /health` returns 200.
  3. Start realtime service and verify `GET /health` returns 200.
  4. Execute smoke path (`apps/web/scripts/e2e-smoke.mjs` or CI equivalent) before exposing service.
  5. If voice/TURN scenarios are in scope, validate coturn reachability with the constrained-network profile (`docs/planning/turn-nat-test-profile.md`).

## Desktop Local-First Baseline

- Scope: default user runtime with local API/realtime services started through repository scripts.
- Startup sequence:
  1. `npm run setup`
  2. `npm run run`
  3. Verify `curl -fsS "http://127.0.0.1:8080/health"`
  4. Verify `curl -fsS "http://127.0.0.1:8081/health"`
  5. Run `npm --prefix apps/web run e2e:smoke`
- Triage baseline:
  - If API health fails, inspect local API service output first.
  - If realtime health fails, inspect local realtime output and API `/v1/auth/sessions/validate` path.
  - If smoke fails, capture command output and compare with CI artifacts under `evidence/ci/<run_id>/`.

### Dedicated Server Bring-Up (Command Baseline)

1. Load environment values from service env files (or export directly):

```bash
set -a; source services/api-rs/.env; source services/realtime-rs/.env; set +a
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

### Dedicated Server Env and Secret Checklist

- Config source contract:
  - non-secret settings come from environment variables declared in `.env`.
  - secret values must come from a secret manager or host secret store (not committed files).
- Required secret mappings:
  - `API_DATABASE_URL`: secret source must be documented per environment.
  - `API_SESSION_SIGNING_KEYS`: secret source must be documented per environment.
- Required non-secret mappings:
  - `API_ALLOWED_ORIGINS`, `REALTIME_ALLOWED_ORIGINS`, `API_BIND`, `REALTIME_BIND`.
- Verification before rollout:
  - checklist artifact saved under `evidence/operations/deploy-checks/<YYYY-MM-DD>/secrets-checklist.md`.

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
- Realtime auth-upstream outage mode:
  - Default is disabled (`REALTIME_WS_AUTH_GRACE_SECONDS=0`) and websocket auth remains strict fail-closed.
  - If temporary API auth availability issues are confirmed, operators may enable a short grace window to allow only recently validated websocket sessions while upstream auth recovers.
  - Keep grace windows short and disable immediately after upstream auth stabilizes.
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
6. If auth grace mode was enabled during incident response, reset `REALTIME_WS_AUTH_GRACE_SECONDS=0` and verify websocket upgrades still pass under normal upstream validation.

## Release Decision and Abort Thresholds

- Release decision owner: current sprint technical owner (record explicit primary and backup names in deployment PR).
- Abort conditions (no rollout/continue rollout):
  - any required CI job failure on candidate commit,
  - health check failure after startup retries,
  - smoke e2e failure,
  - migration checksum mismatch or migration apply failure.
- Immediate rollback triggers (after rollout begins):
  - sustained auth/session validation failures > 5 minutes,
  - realtime websocket upgrade failure rate > 10% for 5-minute window,
  - message send/redeem critical path failure on smoke replay.

## Incident Ownership and Escalation

- Required ownership fields (record in deployment PR and incident evidence):
  - Primary responder (on-call engineer)
  - Secondary responder (backup engineer)
  - Incident decision owner (rollback/go-no-go authority)
- Escalation thresholds:
  - If no mitigation path is identified within 15 minutes, escalate from primary to secondary responder.
  - If outage-impacting symptoms persist beyond 30 minutes, decision owner must choose rollback vs constrained continuation and record rationale.
  - If rollback is initiated, attach rollback timestamp and post-rollback verification output to incident evidence artifact.

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
  - `cargo audit --deny warnings --ignore RUSTSEC-2023-0071`
  - `npm audit --omit=dev --audit-level=high`
  - `semgrep scan --config p/security-audit --error`
- Rust coverage gate:
  - `cargo llvm-cov --workspace --all-features --fail-under-lines 80`
- Integration-smoke run collects evidence artifacts to `evidence/ci/<run_id>/` and uploads as CI artifact.

## Realtime Upgrade Policy

- WebSocket upgrades require an allowed `Origin` header (`REALTIME_ALLOWED_ORIGINS`).
- Requests without `Origin` are rejected (`403`) in runtime mode.

## Related Documents

- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/iterations/04-sprint-board.md`
