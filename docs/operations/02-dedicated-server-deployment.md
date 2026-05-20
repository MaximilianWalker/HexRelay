# HexRelay Dedicated Server Deployment Guide

## Document Metadata

- Doc ID: dedicated-server-deployment
- Owner: Platform maintainers
- Status: needs-detail
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/operations/02-dedicated-server-deployment.md`

## Quick Context

- Purpose: provide the canonical operator guide for single-server dedicated deployments of `api-rs` and `realtime-rs`.
- Primary edit location: update this file when dedicated-server bring-up, secrets, ingress, administration surface, smoke validation, or rollback assumptions change.
- Latest meaningful change: 2026-05-11 clarified that dedicated server delivery is a separate headless service/package path and that administration is performed through the normal HexRelay app for authorized admins using the app-to-server capability contract.

## Purpose

- Give operators one place to start for headless dedicated-server deployment.
- Separate operator deployment steps from local-dev bootstrap and general runbook guidance.
- Make unresolved deployment gaps explicit instead of implying full deployment readiness.

## Status and Scope

- Current status is `needs-detail`.
- This guide is execution-oriented for single-server dedicated deployment, but it is not yet fully complete.
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
- Release packaging authority: `docs/operations/03-release-packaging.md`
- Runtime config authority: `docs/reference/runtime-config-reference.md`
- Incident/recovery authority: `docs/operations/01-mvp-runbook.md`

## Dedicated Server Package Boundary

- Dedicated server mode is a headless service/package path, not a separate desktop app.
- Dedicated server artifacts are not installed by default by the desktop installer.
- The server package may reuse the same Rust service code as desktop local runtime where practical, but it has separate operational responsibilities.
- Windows dedicated server delivery may run as a Windows Service or console binary package.
- Linux dedicated server delivery should support native binaries, a `.deb` package with a `systemd` unit, and container images.
- Server deployments own different config, ports, persistence paths, logs, backups, ingress, firewall exposure, and security posture than a normal desktop install.
- The normal HexRelay app may offer dedicated-server management for authorized server owners/admins, but it should connect to an operator-installed dedicated server endpoint rather than silently bundling a long-running public service into every client install.

## Dedicated Server Administration Surface

- Dedicated server mode remains headless. The dedicated server artifact does not include a separate full UI or standalone admin website by default.
- The normal HexRelay app is the intended administration surface for local servers, LAN servers, private online servers, and public dedicated servers when the signed-in identity has server-owner/admin permissions.
- The app connects to the configured server endpoint and uses authenticated operator/admin APIs exposed by `api-rs`; `realtime-rs` remains the live event plane and must not become an authorization shortcut.
- Admin/operator access must be permission-gated by server-local roles/scopes and protected by the same ingress, TLS, origin, session, and CSRF expectations as other authenticated app traffic.
- The initial app-to-server contract is `GET /server/connection` for endpoint/auth metadata and authenticated `GET /server/capabilities` for per-identity capabilities.
- Until durable server-local role management exists, bootstrap owner/admin authority comes from `API_SERVER_OWNER_IDENTITY_IDS` and `API_SERVER_ADMIN_IDENTITY_IDS`.
- Server discoverability, LAN proximity, invite-only status, or private online hosting must not grant administration access by itself.
- A separate self-hosted admin web console is out of scope unless a later explicit architecture/product decision approves it.
- This section defines the runtime/operations boundary only. Specific admin pages, copy, controls, and flows still require explicit UX approval before implementation.

## Dependency Minimums

- Required for the reviewed single-server dedicated baseline:
  - Postgres for durable API state and embedded migration/bootstrap.
  - Redis for the currently reviewed realtime replay/presence convergence path.
  - one `api-rs` instance.
  - one `realtime-rs` instance.
  - ingress/reverse proxy with TLS termination.
- Required only when the corresponding feature scope is enabled:
  - object storage for blob/media features.
  - coturn for voice/TURN validation or constrained-network media scope.
- Not part of the reviewed dedicated baseline:
  - horizontally scaled `realtime-rs` topologies.
  - project-operated DM relay infrastructure.

## Schema Bootstrap and Migration Authority

- The authoritative schema bootstrap/migration path for dedicated deployments is `api-rs` startup.
- `services/api-rs/src/main.rs` calls `connect_and_prepare(&config.database_url)` before the HTTP listener starts.
- `services/api-rs/src/db.rs` then:
  - connects to Postgres,
  - acquires the migration advisory lock,
  - creates `schema_migrations` if needed,
  - applies any unapplied embedded migrations in order,
  - fails startup on migration checksum mismatch or migration apply failure,
  - returns only after the database is prepared for runtime traffic.
- There is no separate reviewed operator migration command in this repository yet; the supported single-server procedure is to let `api-rs` own schema preparation and to treat startup failure as the migration failure signal.

### Operator Procedure

1. Provision Postgres and ensure the target database from `API_DATABASE_URL` exists and is writable by the `api-rs` service user.
2. Provision Redis before starting either service; the reviewed dedicated baseline depends on it for realtime replay/presence convergence.
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
- If deployment is single-server, record that process-local realtime websocket abuse controls are accepted as-is for this rollout.
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
- `docs/operations/03-release-packaging.md`
- `docs/reference/runtime-config-reference.md`
- `docs/operations/01-mvp-runbook.md`
- `infra/README.md`
