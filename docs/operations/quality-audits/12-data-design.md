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
| QA-12-20260513-migration-evidence-content-not-validated | P2 | found | The migration evidence gate can pass placeholder or stale evidence because it only checks that a changed migration's evidence file changed. | `docs/operations/migration-validation-template.md:30-55` requires concrete forward, rerun, rollback, and data-integrity evidence; `scripts/validate-migration-evidence.sh:11-27` checks only that the matching `evidence/migrations/<migration>.md` path appears in the diff; existing starter artifacts such as `evidence/migrations/0001_friend_requests.md:13`, `:22`, `:29`, `:35`, and `:40-41` contain no retained command transcript, pending revalidation, `Reviewer: TBD`, and `Decision: pass (historical baseline starter)`. | Tighten the migration evidence validator to reject copied placeholders and require filled provenance, commands, outcomes, and integrity fields for any changed migration evidence artifact. | 2026-05-13T21:41:24Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-12-20260513-message-retention-policy-unimplemented | P2 | fixed | Per-server message retention is documented but not represented in the storage model or runtime retention path. | Added `services/api-rs/migrations/0026_server_message_retention_policy.sql` with `servers.retention_message_days` and a retention lookup index, wired it in `services/api-rs/src/db.rs`, added server retention policy update support in `services/api-rs/src/infra/db/repos/servers_repo.rs`, added deterministic channel-message retention tombstoning in `services/api-rs/src/infra/db/repos/server_channels_repo.rs` and `services/api-rs/src/transport/http/handlers/server_channels.rs`, and added regression coverage in `services/api-rs/src/tests/integration/server_channel_messages_tests.rs`. Documentation now records the storage/runtime behavior in `docs/architecture/02-data-lifecycle-retention-replication.md`, `docs/product/09-configuration-defaults-register.md`, and `docs/contracts/runtime-rest.openapi.yaml`. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T21:41:24Z | Codex | Added 2 P2 found findings about missing server message retention representation and weak migration evidence content validation. |
| 2026-05-19T12:30:59Z | Codex | Fixed `QA-12-20260513-message-retention-policy-unimplemented` with per-server retention storage, deterministic tombstone enforcement, migration evidence, and focused API integration coverage. |
