# HexRelay Dedicated Server Deployment Guide

## Document Metadata

- Doc ID: dedicated-server-deployment
- Owner: Platform maintainers
- Status: needs-detail
- Scope: repository
- last_updated: 2026-04-10
- Source of truth: `docs/operations/02-dedicated-server-deployment.md`

## Quick Context

- Purpose: provide the canonical operator guide for single-node dedicated deployments of `api-rs` and `realtime-rs`.
- Primary edit location: update this file when dedicated-server bring-up, secrets, ingress, smoke validation, or rollback assumptions change.
- Latest meaningful change: 2026-04-10 clarified that only the single-node dedicated topology is currently validated and made the process-local websocket abuse-control constraints explicit for operators.

## Purpose

- Give operators one place to start for headless dedicated-server deployment.
- Separate operator deployment steps from local-dev bootstrap and general runbook guidance.
- Make unresolved deployment gaps explicit instead of implying full deployment readiness.

## Status and Scope

- Current status is `needs-detail`.
- This guide is execution-oriented for single-node dedicated deployment, but it is not yet fully complete.
- Known blockers/uncertainties remain around:
  - exact minimum required dependency set for all supported features
  - multi-instance realtime websocket abuse-control equivalence

## Supported Deployment Shape

- Target shape covered here:
  - one `api-rs` instance
  - one `realtime-rs` instance
  - shared Postgres
  - operator-managed ingress/TLS
- This guide does not cover:
  - horizontal realtime scaling as fully production-ready guidance
  - project-operated DM relay infrastructure
  - voice/media rollout beyond noting optional coturn scope

## Current Deployment Validation Boundary

- The currently validated dedicated topology is one `api-rs` instance plus one `realtime-rs` instance behind operator-managed ingress.
- Multi-instance `realtime-rs` deployments remain outside the validated baseline because websocket abuse controls are still process-local.
- Specifically, websocket rate limits and per-identity concurrent connection caps are enforced per `realtime-rs` process, not as shared global state.
- If an operator experiments with multi-instance realtime anyway, they must treat it as deployment-specific validation work and verify sticky routing/session affinity plus edge/global limiting before claiming equivalent behavior.

## Runtime Authorities

- System overview: `docs/architecture/01-system-overview.md`
- Runtime mode authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`
- Runtime config authority: `docs/reference/runtime-config-reference.md`
- Incident/recovery authority: `docs/operations/01-mvp-runbook.md`

## Dependency Minimums

- Required for the documented single-node baseline:
  - Postgres
  - `api-rs`
  - `realtime-rs`
  - ingress/reverse proxy with TLS termination
- Required for the currently validated dedicated baseline:
  - Redis
- Optional depending on enabled scope:
  - object storage for blob/media features
  - coturn for voice/TURN validation or constrained-network media scope

## Schema Bootstrap and Migration Authority

- The authoritative schema bootstrap/migration path for dedicated deployments is `api-rs` startup.
- `services/api-rs/src/main.rs` calls `connect_and_prepare(&config.database_url)` before the HTTP listener starts.
- `services/api-rs/src/db.rs` then:
  - connects to Postgres,
  - acquires the migration advisory lock,
  - creates `schema_migrations` if needed,
  - applies any unapplied embedded migrations in order,
  - fails startup on migration checksum mismatch or migration apply failure,
  - backfills legacy invite-token hashes after migrations,
  - returns only after the database is prepared for runtime traffic.
- There is no separate reviewed operator migration command in this repository yet; the supported single-node procedure is to let `api-rs` own schema preparation and to treat startup failure as the migration failure signal.

### Operator Procedure

1. Provision Postgres and ensure the target database from `API_DATABASE_URL` exists and is writable by the `api-rs` service user.
2. Provision Redis before starting either service; the validated dedicated baseline depends on Redis for realtime replay/presence convergence.
3. Start `api-rs` with the production `API_DATABASE_URL` and required auth/internal-token settings.
4. Watch `api-rs` startup logs and do not continue until startup succeeds; a migration checksum mismatch or SQL apply error is a hard stop, not a warning.
5. Verify `GET /health` on the API endpoint only after `api-rs` has completed startup.
6. Start `realtime-rs` only after the API health probe is green.
7. Verify `GET /health` on the realtime endpoint.
8. Run remote smoke with explicit dedicated-deployment URLs/origin.
9. Record operator evidence for API health, realtime health, smoke, and the migration state snapshot referenced by the runbook restore contract.

### Failure Handling

- If `api-rs` exits during startup because database initialization failed, stop the rollout and treat it as a migration/bootstrap failure.
- Do not start `realtime-rs` against an API instance that has not completed startup successfully.
- Use the runbook rollback procedure if a previously known-good deployment must be restored after migration/bootstrap failure.

## Secrets and Config Inputs

- Canonical variable inventory and production validation rules live in:
  - `docs/reference/runtime-config-reference.md`
- Minimum high-sensitivity values to set explicitly:
  - `API_DATABASE_URL`
  - `API_SESSION_SIGNING_KEYS`
  - `API_SESSION_SIGNING_KEY_ID`
  - `API_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `API_PRESENCE_WATCHER_INTERNAL_TOKEN`
  - `REALTIME_CHANNEL_DISPATCH_INTERNAL_TOKEN`
  - `REALTIME_PRESENCE_WATCHER_INTERNAL_TOKEN`
- Deployment-facing values that must be coordinated with ingress/client config:
  - `API_ALLOWED_ORIGINS`
  - `REALTIME_ALLOWED_ORIGINS`
  - `API_REALTIME_BASE_URL`
  - `REALTIME_API_BASE_URL`
  - cookie security/domain settings
  - proxy-header trust flags

## Remote Web Client and Smoke Inputs

- Remote clients must point at the dedicated deployment using:
  - `NEXT_PUBLIC_API_BASE_URL`
  - `NEXT_PUBLIC_REALTIME_WS_URL`
- Smoke validation additionally requires:
  - `SMOKE_WEB_ORIGIN`
- Current smoke script defaults to localhost, so remote operator validation must override those values.
- If the smoke flow needs fresh public identity bootstrap, also set `API_ALLOW_PUBLIC_IDENTITY_REGISTRATION=true` only for that smoke/bootstrap environment and revert to the default fail-closed value afterward.

Example remote smoke invocation:

```bash
NEXT_PUBLIC_API_BASE_URL="https://api.example.com" \
NEXT_PUBLIC_REALTIME_WS_URL="wss://rt.example.com/ws" \
SMOKE_WEB_ORIGIN="https://app.example.com" \
npm --prefix apps/web run e2e:smoke
```

## Bring-Up Baseline

1. Provision Postgres and Redis.
2. Prepare environment values for `api-rs` and `realtime-rs` using `docs/reference/runtime-config-reference.md`.
3. Start `api-rs` and wait for database initialization to complete successfully.
4. Verify `GET /health` on the API endpoint.
5. Start `realtime-rs`.
6. Verify `GET /health` on the realtime endpoint.
7. Run remote smoke with explicit dedicated-deployment URLs/origin.
8. Record operator evidence for health, smoke, migration state, and config review.

### Deployment Checklist Sign-Off

- Confirm the rollout matches the validated baseline shape: one `api-rs` instance and one `realtime-rs` instance.
- Confirm `api-rs` completed database initialization successfully before starting `realtime-rs`.
- Confirm `GET /health` succeeds for both services before smoke.
- Confirm remote smoke passes with the dedicated deployment URLs and origin.
- Confirm ingress/origin/TLS settings match the deployed browser-facing hosts.
- Confirm the deployment evidence includes the migration state snapshot required by `docs/operations/01-mvp-runbook.md`.
- If deployment is single-node, record that process-local realtime websocket abuse controls are accepted as-is for this rollout.
- If deployment uses more than one `realtime-rs` instance, do not sign off until sticky routing/session affinity and edge/global websocket limiting have both been validated and recorded in deployment evidence.

## Ingress and Trust Boundary Expectations

- TLS must terminate at ingress/reverse proxy.
- API and realtime processes should not be exposed directly without TLS termination.
- `API_TRUST_PROXY_HEADERS` and `REALTIME_TRUST_PROXY_HEADERS` must remain `false` unless the proxy sanitizes forwarded headers.
- Allowed origins must match the actual browser-facing app origin(s).
- Realtime websocket origin validation remains enforced in runtime mode.

## Rollback and Recovery

- Use `docs/operations/01-mvp-runbook.md` for:
  - restart sequencing
  - rollback triggers
  - restore evidence contract
  - grace-mode handling

## Known Gaps

- `needs-detail`: exact minimum dependency matrix for all optional scopes still needs sharper separation between required and optional services.
- `watch`: multi-instance realtime websocket abuse-control equivalence remains an open readiness item in `docs/operations/readiness-corrections-log.md`.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/architecture/01-system-overview.md`
- `docs/reference/runtime-config-reference.md`
- `docs/operations/01-mvp-runbook.md`
- `infra/README.md`
