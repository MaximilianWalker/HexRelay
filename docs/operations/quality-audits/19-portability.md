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

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-19-20260514-security-script-bash-only | P2 | fixed | The root security gate is Bash-only even though a PowerShell cargo-audit helper exists. | `package.json` now routes `npm run security` through `node scripts/security.mjs`; the dispatcher runs `scripts/ensure-cargo-audit.ps1` on Windows and `scripts/ensure-cargo-audit.sh` elsewhere before invoking the unchanged local `cargo audit --deny warnings --ignore RUSTSEC-2023-0071` command. Temporary validation `node -e "..."` now confirms the package script is not Bash-only, and `node --check scripts/security.mjs` validates the dispatcher syntax. The remaining local-vs-CI advisory-ignore drift is tracked separately by `QA-13-20260514-cargo-audit-ignore-parity-drift`. | 2026-05-18T03:57:42Z |
| QA-19-20260514-ci-linux-only | P1 | resolved | CI gates do not exercise any Windows path despite Windows/Linux first-class requirements. | `.github/workflows/ci.yml` now defines `windows-parity-check` on `windows-latest`, runs `npm run setup:windows`, validates runtime/network profiles, and runs `npm run test:windows -- -SkipServiceBackedTests`; `integration-smoke` depends on the new Windows gate. | 2026-05-17T21:40:50Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T03:57:42Z | Codex automation | Resolved `QA-19-20260514-security-script-bash-only` by routing `npm run security` through a Node platform dispatcher that uses the existing PowerShell cargo-audit helper on Windows and the existing Bash helper elsewhere. |
| 2026-05-14T18:52:25Z | Codex | Added 1 P1 confirmed finding about Linux-only CI coverage and 1 P2 confirmed finding about the Bash-only root security gate. |
| 2026-05-17T21:40:50Z | Codex automation | Resolved `QA-19-20260514-ci-linux-only` by adding a Windows parity CI job for the PowerShell setup/test path and portable profile validators. |
