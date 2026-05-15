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
| QA-14-20260514-posix-only-local-parity | P2 | found | Required local CI parity checks are POSIX-only even though Windows is a first-class development target. | `docs/operations/contributor-guide.md:95-103` marks pre-PR checks required and lists `./scripts/validate-migration-evidence.sh`, `./scripts/validate-evidence-provenance.sh`, `./scripts/validate-contract-parity.sh`, `bash scripts/test-contract-parity.sh`, `./scripts/validate-dm-transport-policy.sh`, and `./scripts/validate-docs-index-freshness.sh`; `docs/operations/contributor-guide.md:111-128` provides only a Bash command block for the same sequence; `docs/operations/dev-prerequisites.md:36` says Windows normal development should use PowerShell-backed npm paths and Git Bash/WSL only for direct `.sh` scripts; `scripts/` has no `.ps1` or `.mjs` counterparts for those required parity validators, only the `.sh` files. | Add a cross-platform npm/Node wrapper for required local parity, including base/head SHA resolution and security/documentation validators, then route the contributor guide and PR template to that wrapper. | 2026-05-14T03:44:59Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T03:44:59Z | Codex | Added 1 P2 found finding about POSIX-only local parity checks blocking reliable Windows pre-PR validation. |
