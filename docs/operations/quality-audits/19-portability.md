# Portability Quality Audit

## Metadata

- topic_id: 19-portability
- topic: Portability
- last_audited: 2026-05-17T21:40:50Z
- source_of_truth: `docs/operations/quality-audits/19-portability.md`

## Investigation Focus

- Check Windows and Linux parity, environment assumptions, path handling, runtime profiles, packaging assumptions, and browser/runtime compatibility.
- Flag Windows-only or Linux-deferred plans unless explicitly approved by project rules.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| _none_ | | | | | | |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-19-20260514-ci-linux-only | P1 | resolved | CI gates did not exercise any Windows path despite Windows/Linux first-class requirements. | `.github/workflows/ci.yml` defines `windows-parity-check` on `windows-latest`, runs `npm run setup`, validates runtime/network profiles, and runs `npm run test -- --skip-service-backed-tests`; `integration-smoke` depends on the Windows gate. | 2026-05-17T21:40:50Z |
| QA-19-20260514-security-script-bash-only | P2 | resolved | The root security gate was shell-only. | `npm run security` now runs `scripts/security.mjs`, which installs/runs `cargo-audit` through Node helpers on Windows and Linux. | 2026-05-20T17:30:00Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T18:52:25Z | Codex | Added 1 P1 confirmed finding about Linux-only CI coverage and 1 P2 confirmed finding about the shell-only root security gate. |
| 2026-05-17T21:40:50Z | Codex automation | Resolved `QA-19-20260514-ci-linux-only` by adding a Windows parity CI job for the Node setup/test path and portable profile validators. |
| 2026-05-20T17:30:00Z | Codex | Resolved `QA-19-20260514-security-script-bash-only` by routing the security gate through Node. |
