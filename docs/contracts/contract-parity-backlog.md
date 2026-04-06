# Contract Parity Backlog

## Document Metadata

- Doc ID: contract-parity-backlog
- Owner: API and realtime maintainers
- Status: draft
- Scope: repository
- last_updated: 2026-04-04
- Source of truth: `docs/contracts/contract-parity-backlog.md`

## Quick Context

- Primary working backlog for remaining runtime-vs-contract parity hardening.
- Use this file to decide the next meaningful `scripts/validate-contract-parity.sh` slices.
- Latest meaningful change: 2026-04-04 expanded success-content parity and cleaned unrelated contract drift exposed by the broader parity passes.

## Purpose

- Track the major parity categories still not enforced by CI.
- Keep the next slices grouped into medium-sized, coherent validator changes.
- Avoid losing context after the recent run of small response-path parity passes.

## Current Coverage Snapshot

- Covered well: route inventory, realtime inventory, global error-code inventory, session auth/security parity, CSRF parameter parity, request-body presence, success-status presence, selected error-status presence, path/query parameter presence, and tracked query semantics for requiredness/type/enum/reject-backed bounds.
- Still weak: exact schema refs and body shape, header/cookie semantics, route-scoped `ApiError.code` parity, broader query semantics, and success-content parity.

## Prioritized Todo List

1. Add request schema parity
- Verify each routed `Json<T>` request body maps to the correct OpenAPI schema ref, not just that `requestBody` exists.
- Start with high-churn surfaces in `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: in progress.

2. Add response schema parity
- Verify routed `200`/`201` JSON responses point at the correct OpenAPI schema ref for the returned DTO.
- Start with `services/api-rs/src/transport/http/handlers/auth.rs`, `services/api-rs/src/transport/http/handlers/presence.rs`, `services/api-rs/src/transport/http/handlers/server_channels.rs`, and `services/api-rs/src/transport/http/handlers/dm.rs`.
- Status: in progress.

3. Add internal header parity
- Enforce non-CSRF request-header documentation for runtime-required headers.
- First target: `x-hexrelay-internal-token` on `/v1/internal/presence/watchers/{identity_id}` from `services/api-rs/src/transport/http/handlers/presence.rs`.
- Status: in progress.

4. Add auth response-header and cookie parity
- Enforce documented `Set-Cookie` and cookie-clearing behavior for auth session routes.
- First target: `/v1/auth/verify` and `/v1/auth/sessions/revoke` in `services/api-rs/src/transport/http/handlers/auth.rs`.
- Status: in progress; first pass now checks issue-vs-clear `Set-Cookie` semantics for `hexrelay_session` and `hexrelay_csrf` on auth verify/revoke responses.

5. Add route-scoped `ApiError.code` parity
- Check the concrete error codes each route can emit instead of validating only the global `ApiError.code` enum inventory.
- First target routes: friend-request transitions in `services/api-rs/src/transport/http/handlers/friends.rs` and message mutation routes in `services/api-rs/src/transport/http/handlers/server_channels.rs`.
- Status: first pass completed for the initial high-signal route set.

6. Add route-scoped error example parity
- Validate that documented route-level `401`/`403`/`404` examples and descriptions match runtime failure meaning, not just status presence.
- First cleanup target: `/v1/friends/requests/{request_id}/accept` in `docs/contracts/runtime-rest-v1.openapi.yaml`.
- Status: grouped pass now covers friend-request mutation/bootstrap routes, invite lifecycle routes, auth verify/discovery validation routes, server membership read routes, DM thread/history read routes, DM control-plane bad-request examples, and DM fanout validation examples.

7. Expand query semantics beyond the tracked rule table
- Cover more query/filter semantics where runtime behavior is stable enough to assert mechanically.
- First targets: `services/api-rs/src/transport/http/handlers/directory.rs` and remaining safe rules in `services/api-rs/src/transport/http/handlers/discovery.rs`.
- Status: in progress; `ServerListQuery` and `ContactListQuery` are now covered, and the next pass moves into `DiscoveryUserListQuery` in `services/api-rs/src/transport/http/handlers/discovery.rs`.

8. Add success content parity
- Enforce that JSON success routes document response content and true no-content routes stay `204` without body docs.
- First target files: `services/api-rs/src/transport/http/handlers/health.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: in progress; first pass now checks both runtime no-body success paths with documented schemas and runtime JSON success paths with missing documented content.

9. Separate internal-auth parity from session-auth parity
- Add a dedicated validator path for internal-token-protected endpoints rather than treating them as a one-off documented header.
- Start with `services/api-rs/src/transport/http/handlers/presence.rs` and the matching OpenAPI route block.
- Status: in progress; first pass now treats the internal presence watcher route as internal-token auth and requires a concrete `internal_token_invalid` route-level example.

10. Add validator regression fixtures or golden-route tests
- Add a small deterministic test layer around the validator so future parity expansions do not regress silently.
- Cover tricky handlers first: `auth.rs`, `friends.rs`, `dm.rs`, `server_channels.rs`, and `presence.rs`.

## Recommended Order

- First wave: items 1-4.
- Second wave: items 5-6.
- Third wave: items 7-10.

## Working Rule

- Prefer grouped slices that close one coherent parity class across multiple routes.
- Avoid returning to one-status-at-a-time PRs unless a review finding forces a tiny follow-up.
