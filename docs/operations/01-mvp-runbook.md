# HexRelay MVP Operations Runbook

## Document Metadata

- Doc ID: mvp-runbook
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/operations/01-mvp-runbook.md`

## Quick Context

- Purpose: provide minimum operational procedures for MVP reliability and recovery.
- Primary edit location: update when deployment/recovery/incident steps change.
- Latest meaningful change: 2026-03-04 added operating modes for desktop local-first runtime and dedicated server deployments.

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

## Backup and Restore

- Back up database snapshots and object storage indexes.
- Verify restore quarterly in staging.
- Migration restore validation must include signature verification and reconcile logs.

## SLO Breach Response

- Trigger: KPI/SLO thresholds violated in benchmark profile.
- Action: open remediation task in active iteration board before sign-off.

## Related Documents

- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/iterations/04-sprint-board.md`
