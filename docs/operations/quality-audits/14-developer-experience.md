# Developer Experience Quality Audit

## Metadata

- topic_id: 14-developer-experience
- topic: Developer Experience
- last_audited: 2026-05-14T03:44:59Z
- source_of_truth: `docs/operations/quality-audits/14-developer-experience.md`

## Investigation Focus

- Review setup, local runtime commands, validation feedback, scripts, CI parity, and troubleshooting paths.
- Prioritize problems that slow reliable Windows or Linux development.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| _none_ | | | | | | |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-14-20260514-posix-only-local-parity | P2 | resolved | Required local CI parity checks were POSIX-only even though Windows is a first-class development target. | `npm run check -- --skip-service-backed-tests` now provides one cross-platform local gate, and contributor guidance points to Node validators plus `npm run test:contract-parity`. | 2026-05-20T17:30:00Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T03:44:59Z | Codex | Added 1 P2 found finding about POSIX-only local parity checks blocking reliable Windows pre-PR validation. |
| 2026-05-20T17:30:00Z | Codex | Resolved the POSIX-only local parity finding by adding the cross-platform `npm run check` gate and removing shell-only validator guidance. |
