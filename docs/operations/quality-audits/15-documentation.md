# Documentation Quality Audit

## Metadata

- topic_id: 15-documentation
- topic: Documentation
- last_audited: 2026-05-14T06:45:31Z
- source_of_truth: `docs/operations/quality-audits/15-documentation.md`

## Investigation Focus

- Check canonical docs for drift, missing update triggers, stale caveats, duplicated authorities, and unclear operational instructions.
- Prefer findings that affect planning, implementation safety, or release confidence.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-15-20260514-stale-signaling-doc-caveats | P2 | found | Entry and contributor docs still describe recipient-targeted realtime signaling as deferred or self-targeted-only after the readiness log closed that gap. | `README.md:50` still lists recipient-targeted realtime signaling delivery as a current deferred gap, and `docs/operations/contributor-guide.md:80` still says signaling parity covers self-targeted-only delivery support; `docs/operations/readiness-corrections-log.md:51` marks recipient-targeted websocket signaling delivery closed, while `docs/contracts/README.md:30` and `docs/planning/iterations/02-sprint-board.md:96` describe accepted-contact recipient-targeted live delivery. | Refresh entry/contributor caveats and parity guidance so they point at the current Active Watch Summary and current recipient-targeted signaling semantics. | 2026-05-14T06:45:31Z |
| QA-15-20260514-quality-ledger-freshness-guidance-drift | P2 | found | Contributor docs overstate docs-index refresh requirements for recurring quality-audit ledger edits. | `docs/operations/contributor-guide.md:53` says any docs file other than `docs/README.md` must refresh the docs index, but `docs/README.md:83` and `scripts/validators/docs-index-freshness.sh:19` explicitly exempt recurring numbered ledgers under `docs/operations/quality-audits/[0-9][0-9]-*.md`; the scheduled audit rules also require editing only the selected ledger. | Align contributor docs with the docs index and validator exemption for numbered quality-audit ledgers, while keeping index updates required for quality-audit protocol/index changes. | 2026-05-14T06:45:31Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T06:45:31Z | Codex automation | Added 2 P2 found findings about stale recipient-targeted signaling documentation and quality-audit ledger freshness guidance drift. |
