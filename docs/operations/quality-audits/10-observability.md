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
| QA-10-20260513-shallow-health-probes | P2 | found | API and realtime `/health` probes are used as readiness gates but do not check backing dependencies or critical internal surfaces. | `services/api-rs/src/transport/http/handlers/health.rs:5` returns static `api-rs/ok`; `services/realtime-rs/src/transport/ws/handlers/gateway.rs:26` returns static `ok`; `docs/operations/01-mvp-runbook.md:46` and `.github/workflows/ci.yml:464` use `/health` as startup evidence for API and realtime. | Add dependency-aware readiness checks or separate shallow liveness vs readiness probes covering DB, Redis-backed presence/replay when enabled, and realtime upstream validation. | 2026-05-13T15:37:44Z |
| QA-10-20260513-missing-metrics-alerts | P2 | found | SLO/dashboard evidence docs require metrics and alert observations, but the services have no metrics exporter or alertable signal pipeline. | `docs/testing/observability-evidence-template.md:29` requires dashboard evidence and `docs/testing/observability-evidence-template.md:35` requires alert/fault-injection evidence; `docs/operations/01-mvp-runbook.md:192` defines SLO breach response; `rg -n 'prometheus\|opentelemetry\|metrics\|/metrics\|counter!\|histogram!\|gauge!' Cargo.toml services/api-rs/Cargo.toml services/realtime-rs/Cargo.toml services/api-rs/src services/realtime-rs/src docs/testing/observability-evidence-template.md docs/operations/01-mvp-runbook.md docs/planning/kpi-slo-test-profile.md` found only documentation mentions, not service instrumentation. | Introduce a minimal metrics/alert contract for auth/session validation, websocket upgrades, DM/server-channel dispatch outcomes, and SLO benchmark evidence before treating dashboard/alert artifacts as actionable. | 2026-05-13T15:37:44Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T15:37:44Z | Codex automation | Added 2 P2 found findings about shallow readiness probes and missing metrics/alert instrumentation. |
