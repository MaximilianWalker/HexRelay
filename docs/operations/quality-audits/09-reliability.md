# Reliability Quality Audit

## Metadata

- topic_id: 09-reliability
- topic: Reliability
- last_audited: 2026-05-13T12:37:27Z
- source_of_truth: `docs/operations/quality-audits/09-reliability.md`

## Investigation Focus

- Inspect partial failure handling, timeout/retry behavior, idempotency, startup/shutdown resilience, and persistence guarantees.
- Prioritize findings that can cause lost messages, stuck delivery, inconsistent runtime state, or fragile operations.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-09-20260513-dm-outbound-retry-not-driven | P1 | found | DM outbound forwarding retries are scheduled but never driven by production runtime. | `docs/architecture/02-data-lifecycle-retention-replication.md:28` and `:41` define outbound forwarding retry bookkeeping, next-attempt scheduling, bounded attempts, and backoff as origin-node state. `services/api-rs/src/transport/http/handlers/dm.rs:409` computes `next_attempt_at` after forwarding failure, `services/api-rs/src/infra/db/repos/dm_repo.rs:544` persists failed records with that timestamp, and `services/api-rs/src/infra/db/repos/dm_repo.rs:627` can list due records. The retry worker exists at `services/api-rs/src/domain/dm/outbound_forwarding.rs:49`, but `rg -n "retry_due_dm_outbound_forwards\(" services/api-rs/src docs -g '!services/api-rs/src/tests/**'` returned only that definition, and `services/api-rs/src/main.rs:68` to `:80` builds and serves the app without spawning or invoking a retry driver. | Wire a bounded background worker, supervised scheduled command, or documented operator job that calls `retry_due_dm_outbound_forwards`, then add integration coverage proving a due failed outbound forward is retried without direct test-only invocation. | 2026-05-13T12:37:27Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T12:37:27Z | Codex | Added 1 P1 found finding about undriven DM outbound forwarding retries. |
