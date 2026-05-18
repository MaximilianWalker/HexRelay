# Privacy Quality Audit

## Metadata

- topic_id: 21-privacy
- topic: Privacy
- last_audited: 2026-05-15T00:54:28Z
- source_of_truth: `docs/operations/quality-audits/21-privacy.md`

## Investigation Focus

- Review data minimization, retention, export/delete, logging, trust boundaries, and DM encryption constraints.
- Treat server-readable DM content, private-key upload, or excessive delivery metadata as severe findings.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-21-20260515-private-key-secure-store-fallback | P1 | confirmed | Client private-key encryption falls back to browser session storage with the wrapping master key stored beside the encrypted private key. | `apps/web/lib/secure-store.ts:13-20` defines the secure-store fallback as `window.sessionStorage`, `apps/web/lib/secure-store.ts:76-95` writes to that fallback when no provider exists or provider writes fail, and `apps/web/lib/sessions.ts:25-34` persists the master key material through that same abstraction before `apps/web/lib/sessions.ts:165-174` encrypts and stores persona private keys. This weakens the client-only private-key at-rest guarantee in `docs/product/01-mvp-plan.md:254-256` and `docs/architecture/02-data-lifecycle-retention-replication.md:22-26`. | Make desktop/platform secure storage or a user-held passphrase-backed keystore mandatory for private-key writes; if unavailable, fail closed instead of storing both key material and ciphertext in the same browser storage tier. | 2026-05-15T00:54:28Z |
| QA-21-20260515-dm-metadata-logs-unbounded | P2 | confirmed | DM delivery logs emit stable message, thread, recipient, and device metadata outside the documented bounded delivery-metadata retention controls. | `services/api-rs/src/domain/dm/realtime.rs:112-125` logs DM dispatch success with message/thread/recipient identifiers and device outcome counts, `services/realtime-rs/src/domain/dms.rs:141-152` logs realtime dispatch summaries with message and recipient identifiers, and `services/realtime-rs/src/domain/dms.rs:204-223` logs ack failures with message, recipient, and device identifiers. The retention authority bounds database delivery metadata in `docs/architecture/02-data-lifecycle-retention-replication.md:48-53` but does not cover equivalent tracing/log retention or redaction. | Redact or hash DM identifiers in routine logs and document/enforce a retention policy for operational logs that carry DM delivery metadata. | 2026-05-15T00:54:28Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-21-20260515-export-import-surface-missing | P2 | fixed | Runtime account-data export/import surface was absent despite data ownership requirements. | Added authenticated `GET /account/export` and dry-run-only `POST /account/import` in `services/api-rs/src/app/router.rs`, `services/api-rs/src/transport/http/handlers/account_data.rs`, and `services/api-rs/src/infra/db/repos/account_data_repo.rs`; documented the runtime contract in `docs/contracts/runtime-rest.openapi.yaml` and secret/metadata exclusions plus dry-run import behavior in `docs/architecture/02-data-lifecycle-retention-replication.md`; added regression coverage in `services/api-rs/src/tests/integration/account_data_tests.rs`. Mutating import remains future migration work and fails closed. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T19:12:06Z | Codex issue remediator | Fixed `QA-21-20260515-export-import-surface-missing` with an authenticated account-data export surface, dry-run import validation, runtime contract/docs coverage, and focused regression tests. |
| 2026-05-15T00:54:28Z | Codex | Added 1 P1 confirmed finding and 2 P2 confirmed findings about private-key storage fallback, unbounded DM metadata logging, and missing export-import runtime surface. |
