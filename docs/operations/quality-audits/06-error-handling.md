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
| QA-06-20260513-unmapped-extractor-rejections | P2 | found | REST extractor failures can bypass the shared `ApiError` response shape. | `services/api-rs/src/shared/errors.rs:5` defines handler-local `ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>`, but handlers consume Axum extractors directly, for example `Json(payload): Json<AuthVerifyRequest>` in `services/api-rs/src/transport/http/handlers/auth.rs:257` and `Query(query): Query<DmThreadListQuery>` in `services/api-rs/src/transport/http/handlers/dm.rs:941`; `rg -n -e "JsonRejection" -e "DefaultBodyLimit" -e "handle_error" -e "HandleErrorLayer" -e "fallback\\(" services/api-rs/src` returned no custom rejection/fallback mapping; `docs/contracts/runtime-rest.openapi.yaml:4098` documents shared BadRequest responses as JSON `ApiError`. Malformed JSON, bad query values, and unsupported content types therefore use framework-level rejection behavior instead of the documented error envelope. | Add a shared extractor/rejection layer or typed extractors that translate JSON/query/path/body-limit failures into stable `ApiError` codes, then add representative malformed-body/query regression tests and extend parity if needed. | 2026-05-13T03:33:40Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-06-20260513-storage-errors-discard-causes | P2 | fixed | Many storage failure paths dropped the underlying error before it could be logged or classified. | Added shared `storage_error` mapping in `services/api-rs/src/shared/errors.rs` and replaced lossy auth/session/invite/DM storage mappings with source-error logging contexts while preserving client-safe response codes/messages; temporary harness `rg -n 'map_err\\(\\|_\\|' ... | Select-String 'storage_unavailable|storage_failure|friendship_lookup_failed'` returned no source-discarding storage mappings in the selected surfaces; `cargo test -p api-rs storage_error_preserves_client_safe_response --all-features` passed. | 2026-05-19 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-13T03:33:40Z | Codex | Added 2 P2 found findings about REST extractor rejection normalization and storage error cause preservation. |
