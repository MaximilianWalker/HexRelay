# Observability Evidence Template

## Document Metadata

- Doc ID: observability-evidence-template
- Owner: Platform and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-19
- Source of truth: `docs/testing/observability-evidence-template.md`

## Quick Context

- Purpose: deterministic evidence template for observability and SLO verification tasks.
- Primary edit location: update when dashboard/alert evidence contract changes.
- Latest meaningful change: 2026-05-19 anchored dashboard and alert evidence to runtime `/metrics` counters.

Use this template when closing observability/SLO verification tasks.

## Metadata

- Date (UTC):
- Owner:
- Environment:
- Related task IDs:
- Commit SHA:
- PR number (or CI run ID):

## Dashboard Evidence

- Dashboard export path:
- API `/metrics` scrape path:
- Realtime `/metrics` scrape path:
- Key metrics shown:
- Time window covered:

## Alert/Fault Injection Evidence

- Fault injected:
- Alert expected:
- Alert query or rule path:
- Triggering metric:
- Alert observed (timestamp):
- Recovery observed (timestamp):
- Evidence path:

## Conclusion

- Result: pass / fail
- Follow-ups:

## Provenance

- Validator command list hash (optional but recommended):
- Raw outputs folder:
- Generated at (UTC):
