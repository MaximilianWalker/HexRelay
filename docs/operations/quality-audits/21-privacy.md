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
| QA-21-20260515-dm-metadata-logs-unbounded | P2 | confirmed | DM delivery logs emit stable message, thread, recipient, and device metadata outside the documented bounded delivery-metadata retention controls. | `services/api-rs/src/domain/dm/realtime.rs:112-125` logs DM dispatch success with message/thread/recipient identifiers and device outcome counts, `services/realtime-rs/src/domain/dms.rs:141-152` logs realtime dispatch summaries with message and recipient identifiers, and `services/realtime-rs/src/domain/dms.rs:204-223` logs ack failures with message, recipient, and device identifiers. The retention authority bounds database delivery metadata in `docs/architecture/02-data-lifecycle-retention-replication.md:48-53` but does not cover equivalent tracing/log retention or redaction. | Redact or hash DM identifiers in routine logs and document/enforce a retention policy for operational logs that carry DM delivery metadata. | 2026-05-15T00:54:28Z |
| QA-21-20260515-export-import-surface-missing | P2 | confirmed | The runtime has no user/node data export-import surface even though data ownership is an explicit product baseline. | `docs/product/01-mvp-plan.md:139` requires full export/import, `docs/product/01-mvp-plan.md:448` defines operator export/import of full node user data, and `README.md:28` keeps export-import guarantees as a repository goal. The API router in `services/api-rs/src/app/router.rs:60-141` exposes auth, contacts, servers, DMs, friend requests, block/mute, and message delete routes, but no export/import or account-data portability route; a targeted `rg` search for export/import/erasure/account-delete terms found product requirements but no runtime or contract implementation. | Add a scoped export/import contract and implementation plan for identity/profile/session/message-owned data, including deletion/retention behavior and evidence tests. | 2026-05-15T00:54:28Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-21-20260515-private-key-secure-store-fallback | P1 | fixed | Client private-key encryption fell back to browser session storage with the wrapping master key stored beside the encrypted private key. | `apps/web/lib/secure-store.ts` now reads and writes only through `window.__HEXRELAY_SECURE_STORE__`; writes throw when the provider is missing or fails instead of storing fallback values. `apps/web/lib/secure-store.test.ts` covers provider-only read/write/remove, unavailable-provider rejection, and provider write-failure rejection without browser fallback storage. `apps/web/lib/sessions.test.ts` covers private-key persistence failing closed without writing master-key or private-key material into browser fallback storage. Temporary remediation check `Select-String -Path apps\web\lib\secure-store.ts -Pattern 'sessionStorage\|fallback'` failed before the fix and passed after. | 2026-05-17 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-15T00:54:28Z | Codex | Added 1 P1 confirmed finding and 2 P2 confirmed findings about private-key storage fallback, unbounded DM metadata logging, and missing export-import runtime surface. |
