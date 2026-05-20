# Data Design Quality Audit

## Metadata

- topic_id: 12-data-design
- topic: Data Design
- last_audited: 2026-05-13T21:41:24Z
- source_of_truth: `docs/operations/quality-audits/12-data-design.md`

## Investigation Focus

- Review schema ownership, migrations, retention, consistency rules, export/import, and backup implications.
- Prioritize issues that threaten data ownership, portability, encrypted DM boundaries, or migration safety.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-12-20260513-message-retention-policy-unimplemented | P2 | found | Per-server message retention is documented but not represented in the storage model or runtime retention path. | `docs/product/01-mvp-plan.md:38` says message retention defaults to forever and is configurable per server; `docs/architecture/02-data-lifecycle-retention-replication.md:30` says server channel messages are subject to server retention policy and `:37` names `retention.message_days`; `docs/product/09-configuration-defaults-register.md:53` registers `retention.message_days`, but `services/api-rs/migrations/0012_servers_and_memberships.sql:1-4` stores only `server_id`, `name`, and `created_at`, and `rg -n -e "retention.message_days" -e "message_days" -e "retention_days" -e "purge.*message" services apps docs/architecture/02-data-lifecycle-retention-replication.md docs/product/01-mvp-plan.md docs/product/09-configuration-defaults-register.md` found only docs plus DM delivery-metadata retention code. | Add an explicit server retention policy data model and deterministic purge/tombstone enforcement path, or move configurable message retention out of MVP scope until it is represented end to end. | 2026-05-13T21:41:24Z |
| QA-12-20260513-migration-evidence-content-not-validated | P2 | found | The migration evidence gate can pass placeholder or stale evidence because it only checks that a changed migration's evidence file changed. | `docs/operations/migration-validation-template.md:30-55` requires concrete forward, rerun, rollback, and data-integrity evidence; `scripts/validators/migration-evidence.mjs:11-27` checks only that the matching `evidence/migrations/<migration>.md` path appears in the diff; existing starter artifacts such as `evidence/migrations/0001_friend_requests.md:13`, `:22`, `:29`, `:35`, and `:40-41` contain no retained command transcript, pending revalidation, `Reviewer: TBD`, and `Decision: pass (historical baseline starter)`. | Tighten the migration evidence validator to reject copied placeholders and require filled provenance, commands, outcomes, and integrity fields for any changed migration evidence artifact. | 2026-05-13T21:41:24Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T21:41:24Z | Codex | Added 2 P2 found findings about missing server message retention representation and weak migration evidence content validation. |
