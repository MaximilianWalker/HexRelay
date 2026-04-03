# HexRelay Dedicated Server Deployment Guide

## Document Metadata

- Doc ID: dedicated-server-deployment
- Owner: Platform maintainers
- Status: needs-detail
- Scope: repository
- last_updated: 2026-04-03
- Source of truth: `docs/operations/02-dedicated-server-deployment.md`

## Quick Context

- Purpose: provide the canonical operator guide for single-node dedicated deployments of `api-rs` and `realtime-rs`.
- Primary edit location: update this file when dedicated-server bring-up, secrets, ingress, smoke validation, or rollback assumptions change.
- Latest meaningful change: 2026-04-03 added the first operator-focused dedicated-server deployment baseline and explicitly marked unresolved bootstrap/dependency gaps.

## Purpose

- Give operators one place to start for headless dedicated-server deployment.
- Separate operator deployment steps from local-dev bootstrap and general runbook guidance.
- Make unresolved deployment gaps explicit instead of implying full deployment readiness.

## Status and Scope

- Current status is `needs-detail`.
- This guide is execution-oriented for single-node dedicated deployment, but it is not yet fully complete.
- Known blockers/uncertainties remain around:
  - authoritative schema bootstrap/migration procedure
  - exact minimum required dependency set for all supported features
  - multi-instance realtime abuse-control equivalence

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
- Required for current realtime replay/presence convergence behavior:
  - Redis
- Optional depending on enabled scope:
  - object storage for blob/media features
  - coturn for voice/TURN validation or constrained-network media scope

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
3. Start `api-rs`.
4. Verify `GET /health` on the API endpoint.
5. Start `realtime-rs`.
6. Verify `GET /health` on the realtime endpoint.
7. Run remote smoke with explicit dedicated-deployment URLs/origin.
8. Record operator evidence for health, smoke, and config review.

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

- `blocked`: authoritative schema bootstrap/migration command flow is not yet documented in reviewed operator docs.
- `needs-detail`: exact minimum dependency matrix for all optional scopes still needs sharper separation between required and optional services.
- `watch`: multi-instance realtime limiter equivalence remains an open readiness item in `docs/operations/readiness-corrections-log.md`.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/architecture/01-system-overview.md`
- `docs/reference/runtime-config-reference.md`
- `docs/operations/01-mvp-runbook.md`
- `infra/README.md`
