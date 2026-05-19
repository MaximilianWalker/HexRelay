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

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-10-20260513-missing-metrics-alerts | P2 | fixed | SLO/dashboard evidence docs require metrics and alert observations, but the services had no metrics exporter or alertable signal pipeline. | Added Prometheus text `/metrics` endpoints in `services/api-rs/src/transport/http/handlers/metrics.rs` and `services/realtime-rs/src/transport/http/metrics.rs`; added bounded counters for API auth/session, DM dispatch, and server-channel dispatch enqueue outcomes plus realtime websocket upgrade and protected DM/server-channel dispatch outcomes; documented the scrape surfaces in runtime contracts and observability evidence docs. Focused pre-fix `rg` for the selected metric names and `/metrics` routes returned no matches; post-fix tests and contract validation cover the new metrics surface. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-19T07:26:41Z | Codex issue remediator | Fixed `QA-10-20260513-missing-metrics-alerts` with runtime `/metrics` endpoints, bounded alertable counters, contract docs, and observability evidence guidance. |
| 2026-05-13T15:37:44Z | Codex automation | Added 2 P2 found findings about shallow readiness probes and missing metrics/alert instrumentation. |
