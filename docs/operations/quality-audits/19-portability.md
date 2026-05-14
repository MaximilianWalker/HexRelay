# Portability Quality Audit

## Metadata

- topic_id: 19-portability
- topic: Portability
- last_audited: 2026-05-14T18:52:25Z
- source_of_truth: `docs/operations/quality-audits/19-portability.md`

## Investigation Focus

- Check Windows and Linux parity, environment assumptions, path handling, runtime profiles, packaging assumptions, and browser/runtime compatibility.
- Flag Windows-only or Linux-deferred plans unless explicitly approved by project rules.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-19-20260514-ci-linux-only | P1 | confirmed | CI gates do not exercise any Windows path despite Windows/Linux first-class requirements. | `AGENTS.md:12` requires Windows and Linux as mandatory first-class development/testing targets; `.github/workflows/ci.yml` has 11 `runs-on: ubuntu-latest` entries and `rg -n "windows-latest|runs-on:.*windows|matrix:.*os|os:" .github/workflows/ci.yml` returned no matches. | Add a minimal Windows CI job or OS matrix that runs the PowerShell-backed setup/test path and any portable path checks needed before release readiness. | 2026-05-14T18:52:25Z |
| QA-19-20260514-security-script-bash-only | P2 | confirmed | The root security gate is Bash-only even though a PowerShell cargo-audit helper exists. | `package.json:37` defines `npm run security` as `bash scripts/ensure-cargo-audit.sh && cargo audit ...`; `scripts/ensure-cargo-audit.ps1` exists, and `docs/operations/dev-prerequisites.md:36` directs Windows developers to PowerShell-backed npm paths for normal local development. | Route `npm run security` through a Node platform dispatcher or add/document a Windows security alias that uses `scripts/ensure-cargo-audit.ps1`. | 2026-05-14T18:52:25Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T18:52:25Z | Codex | Added 1 P1 confirmed finding about Linux-only CI coverage and 1 P2 confirmed finding about the Bash-only root security gate. |
