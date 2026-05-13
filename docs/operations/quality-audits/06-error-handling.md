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
| QA-06-20260513-unmapped-extractor-rejections | P2 | found | REST extractor failures can bypass the shared `ApiError` response shape. | `services/api-rs/src/shared/errors.rs:5` defines handler-local `ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>`, but handlers consume Axum extractors directly, for example `Json(payload): Json<AuthVerifyRequest>` in `services/api-rs/src/transport/http/handlers/auth.rs:257` and `Query(query): Query<DmThreadListQuery>` in `services/api-rs/src/transport/http/handlers/dm.rs:941`; `rg -n "JsonRejection\|DefaultBodyLimit\|handle_error\|HandleErrorLayer\|fallback\(" services/api-rs/src` returned no custom rejection/fallback mapping; `docs/contracts/runtime-rest.openapi.yaml:4098` documents shared BadRequest responses as JSON `ApiError`. Malformed JSON, bad query values, and unsupported content types therefore use framework-level rejection behavior instead of the documented error envelope. | Add a shared extractor/rejection layer or typed extractors that translate JSON/query/path/body-limit failures into stable `ApiError` codes, then add representative malformed-body/query regression tests and extend parity if needed. | 2026-05-13T03:33:40Z |
| QA-06-20260513-storage-errors-discard-causes | P2 | found | Many storage failure paths drop the underlying error before it can be logged or classified. | High-impact auth/session/invite/DM paths map database failures with `map_err(|_| internal_error(...))`, for example `services/api-rs/src/transport/http/middleware/auth.rs:197`, `services/api-rs/src/transport/http/handlers/auth.rs:67`, `services/api-rs/src/transport/http/handlers/auth.rs:396`, `services/api-rs/src/transport/http/handlers/invites.rs:200`, and `services/api-rs/src/transport/http/handlers/dm.rs:123`; `services/api-rs/src/shared/errors.rs:33` only constructs the client `ApiError`, and `rg -n "tracing::\|warn!\|error!\|info!\|debug!" services/api-rs/src/transport/http/handlers services/api-rs/src/transport/http/middleware services/api-rs/src/shared/errors.rs` shows most of these `|_| internal_error` call sites have no adjacent source-error logging. Operators get only `storage_unavailable` and lose whether the failure was timeout, constraint drift, migration mismatch, or connectivity. | Introduce a shared storage-error mapper that logs the source error with route/domain context while preserving client-safe `ApiError` messages, then replace the lossy auth/session/invite/DM mappings. | 2026-05-13T03:33:40Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T03:33:40Z | Codex | Added 2 P2 found findings about REST extractor rejection normalization and storage error cause preservation. |
