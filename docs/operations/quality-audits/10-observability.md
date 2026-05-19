# Observability Quality Audit

## Metadata

- topic_id: 10-observability
- topic: Observability
- last_audited: 2026-05-13T15:37:44Z
- source_of_truth: `docs/operations/quality-audits/10-observability.md`

## Investigation Focus

- Review logs, metrics, traces, health checks, evidence templates, and debuggability for production-like incidents.
- Flag missing signals for critical auth, DM delivery, realtime, persistence, and deployment workflows.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-10-20260513-missing-metrics-alerts | P2 | found | SLO/dashboard evidence docs require metrics and alert observations, but the services have no metrics exporter or alertable signal pipeline. | `docs/testing/observability-evidence-template.md:29` requires dashboard evidence and `docs/testing/observability-evidence-template.md:35` requires alert/fault-injection evidence; `docs/operations/01-mvp-runbook.md:192` defines SLO breach response; `rg -n -e "prometheus" -e "opentelemetry" -e "metrics" -e "/metrics" -e "counter!" -e "histogram!" -e "gauge!" Cargo.toml services/api-rs/Cargo.toml services/realtime-rs/Cargo.toml services/api-rs/src services/realtime-rs/src docs/testing/observability-evidence-template.md docs/operations/01-mvp-runbook.md docs/planning/kpi-slo-test-profile.md` found only documentation mentions, not service instrumentation. | Introduce a minimal metrics/alert contract for auth/session validation, websocket upgrades, DM/server-channel dispatch outcomes, and SLO benchmark evidence before treating dashboard/alert artifacts as actionable. | 2026-05-13T15:37:44Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-10-20260513-shallow-health-probes | P2 | fixed | API and realtime `/health` probes were used as readiness gates but did not check backing dependencies or critical internal surfaces. | Added `/ready` readiness routes for API and realtime while preserving `/health` as shallow liveness; API readiness checks Postgres plus configured Redis, realtime readiness checks configured Redis plus API `/ready` and `/auth/sessions/validate`; CI, run scripts, and operations docs now use `/ready` for startup evidence. Temporary scanner failed before the fix and passed after the fix. | 2026-05-19T08:39:36Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-19T08:39:36Z | Codex issue remediator | Marked `QA-10-20260513-shallow-health-probes` fixed after adding dependency-aware `/ready` startup readiness routes and moving startup evidence away from shallow `/health` liveness. |
| 2026-05-13T15:37:44Z | Codex automation | Added 2 P2 found findings about shallow readiness probes and missing metrics/alert instrumentation. |
