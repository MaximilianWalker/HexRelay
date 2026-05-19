# Error Handling Quality Audit

## Metadata

- topic_id: 06-error-handling
- topic: Error Handling
- last_audited: 2026-05-13T03:33:40Z
- source_of_truth: `docs/operations/quality-audits/06-error-handling.md`

## Investigation Focus

- Look for swallowed failures, vague user/operator errors, unchecked fallible operations, and inconsistent error contracts.
- Prioritize paths where bad error handling can hide data, auth, delivery, or deployment failures.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-06-20260513-storage-errors-discard-causes | P2 | found | Many storage failure paths drop the underlying error before it can be logged or classified. | High-impact auth/session/invite/DM paths map database failures with `map_err(|_| internal_error(...))`, for example `services/api-rs/src/transport/http/middleware/auth.rs:197`, `services/api-rs/src/transport/http/handlers/auth.rs:67`, `services/api-rs/src/transport/http/handlers/auth.rs:396`, `services/api-rs/src/transport/http/handlers/invites.rs:200`, and `services/api-rs/src/transport/http/handlers/dm.rs:123`; `services/api-rs/src/shared/errors.rs:33` only constructs the client `ApiError`, and `rg -n -e "tracing::" -e "warn!" -e "error!" -e "info!" -e "debug!" services/api-rs/src/transport/http/handlers services/api-rs/src/transport/http/middleware services/api-rs/src/shared/errors.rs` shows most of these `|_| internal_error` call sites have no adjacent source-error logging. Operators get only `storage_unavailable` and lose whether the failure was timeout, constraint drift, migration mismatch, or connectivity. | Introduce a shared storage-error mapper that logs the source error with route/domain context while preserving client-safe `ApiError` messages, then replace the lossy auth/session/invite/DM mappings. | 2026-05-13T03:33:40Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-06-20260513-unmapped-extractor-rejections | P2 | fixed | REST extractor failures now use the shared `ApiError` response shape. | Added shared `Json`, `Query`, and `Path` wrappers in `services/api-rs/src/shared/extractors.rs` that map Axum JSON/query/path rejection failures into stable JSON `ApiError` codes, migrated REST handlers/middleware to those wrappers, documented the new error codes in `docs/contracts/runtime-rest.openapi.yaml`, and added `services/api-rs/src/tests/integration/extractor_rejection_tests.rs`; `cargo test -p api-rs --all-features extractor_rejection -- --nocapture` failed before the fix and passes after it. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T03:33:40Z | Codex | Added 2 P2 found findings about REST extractor rejection normalization and storage error cause preservation. |
