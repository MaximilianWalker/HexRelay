# Contract Parity Backlog

## Document Metadata

- Doc ID: contract-parity-backlog
- Owner: API and realtime maintainers
- Status: draft
- Scope: repository
- last_updated: 2026-04-06
- Source of truth: `docs/contracts/contract-parity-backlog.md`

## Quick Context

- Primary working backlog for remaining runtime-vs-contract parity hardening.
- Use this file to decide the next meaningful `scripts/validate-contract-parity.sh` slices.
- Latest meaningful change: 2026-04-06 broadened route-scoped parity across auth/social surfaces, and tightened success-body documentation across auth/invite, DM control, server-channel lifecycle, social-graph handshake, and DM connectivity setup/control routes.

## Purpose

- Track the major parity categories still not enforced by CI.
- Keep the next slices grouped into medium-sized, coherent validator changes.
- Avoid losing context after the recent run of small response-path parity passes.

## Current Coverage Snapshot

- Covered well: route inventory, realtime inventory, global error-code inventory, exact session-auth security-scheme parity, internal-auth/header/security parity for the internal presence watcher route, CSRF parameter parity, request-body presence, request/response schema-ref parity including request/response alias normalization and direct mismatch regressions, success-status presence, selected error-status presence including extractor-backed `403`/`404` paths and helper/delegate `400`/`500` flows, path/query parameter presence, response-header parity, auth cookie semantics, route-scoped `ApiError.code` parity for high-signal routes, broad route-scoped error-example parity including status-specific server-channel mutation checks, deterministic regression fixtures for missing auth/status/schema/content/header branches, and tracked query semantics for requiredness/type/enum/reject-backed bounds plus first-pass discovery normalization/default semantics.
- Still weak: full API-wide breadth closeout for schema/error-example coverage, broader query semantics beyond the currently safe rule set, and exhaustive success-content parity closeout.

## Prioritized Todo List

1. Close out request schema parity breadth
- Verify each routed `Json<T>` request body maps to the correct OpenAPI schema ref, not just that `requestBody` exists.
- Start with high-churn surfaces in `services/api-rs/src/transport/http/handlers/dm.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: validator support is in place, regression fixtures cover alias and direct mismatch paths, and the main DM/friends/block-mute surfaces now appear aligned; remaining work is breadth closeout rather than core capability.

2. Close out response schema parity breadth
- Verify routed `200`/`201` JSON responses point at the correct OpenAPI schema ref for the returned DTO.
- Start with `services/api-rs/src/transport/http/handlers/auth.rs`, `services/api-rs/src/transport/http/handlers/presence.rs`, `services/api-rs/src/transport/http/handlers/server_channels.rs`, and `services/api-rs/src/transport/http/handlers/dm.rs`.
- Status: validator support is in place, regression fixtures cover alias and direct mismatch paths, and the current routed JSON success families now appear top-level aligned after the server-channel naming cleanup; remaining work is narrow follow-up only if nested/schema-shape drift or a new routed alias family appears.

3. Add internal header parity
- Enforce non-CSRF request-header documentation for runtime-required headers.
- First target: `x-hexrelay-internal-token` on `/v1/internal/presence/watchers/{identity_id}` from `services/api-rs/src/transport/http/handlers/presence.rs`.
- Status: completed for the currently known internal-token route surface.

4. Add auth response-header and cookie parity
- Enforce documented `Set-Cookie` and cookie-clearing behavior for auth session routes.
- First target: `/v1/auth/verify` and `/v1/auth/sessions/revoke` in `services/api-rs/src/transport/http/handlers/auth.rs`.
- Status: completed for auth verify/revoke cookie issue-vs-clear semantics on `hexrelay_session` and `hexrelay_csrf`.

5. Add route-scoped `ApiError.code` parity
- Check the concrete error codes each route can emit instead of validating only the global `ApiError.code` enum inventory.
- First target routes: friend-request transitions in `services/api-rs/src/transport/http/handlers/friends.rs` and message mutation routes in `services/api-rs/src/transport/http/handlers/server_channels.rs`.
- Status: strong grouped pass completed for initial high-signal routes; remaining work is breadth expansion only where additional value is found.

6. Close out route-scoped error example breadth
- Validate that documented route-level `401`/`403`/`404` examples and descriptions match runtime failure meaning, not just status presence.
- First cleanup target: `/v1/friends/requests/{request_id}/accept` in `docs/contracts/runtime-rest-v1.openapi.yaml`.
- Status: broad grouped pass now covers auth register/challenge/verify/revoke routes, friend-request list/create/mutation/bootstrap routes, block/mute create routes, invite lifecycle routes, discovery validation routes, server membership read routes, DM thread/history read routes, DM control-plane bad-request examples, internal-auth `401` examples, and server-channel mutation status-specific examples; remaining work is breadth closeout on any still-underdocumented simple routes only.

7. Expand query semantics beyond the tracked rule table
- Cover more query/filter semantics where runtime behavior is stable enough to assert mechanically.
- First targets: `services/api-rs/src/transport/http/handlers/directory.rs` and remaining safe rules in `services/api-rs/src/transport/http/handlers/discovery.rs`.
- Status: in progress; `ServerListQuery` and `ContactListQuery` now cover blank-search normalization plus case-insensitive matching, and `DiscoveryUserListQuery` covers default, trim-before-enum, blank-query normalization, case-insensitive matching, and limit clamp semantics. Remaining work is limited to additional safe query rules where runtime behavior is stable enough to assert.

8. Close out success content parity breadth
- Enforce that JSON success routes document response content and true no-content routes stay `204` without body docs.
- First target files: `services/api-rs/src/transport/http/handlers/health.rs`, `services/api-rs/src/transport/http/handlers/friends.rs`, and `services/api-rs/src/transport/http/handlers/block_mute.rs`.
- Status: validator support is in place and first-pass cleanup landed; auth/invite, DM fanout/profile-device control, server-channel lifecycle, social-graph handshake, and DM connectivity setup/control routes now have branch-specific success descriptions/examples, with remaining work limited to the last simpler residual success payload families rather than missing core capability.

9. Separate internal-auth parity from session-auth parity
- Add a dedicated validator path for internal-token-protected endpoints rather than treating them as a one-off documented header.
- Start with `services/api-rs/src/transport/http/handlers/presence.rs` and the matching OpenAPI route block.
- Status: completed for the current internal presence watcher route family, including dedicated header/security handling and required `internal_token_invalid` example coverage.

10. Close out validator regression fixture coverage
- Add a small deterministic test layer around the validator so future parity expansions do not regress silently.
- Cover tricky handlers first: `auth.rs`, `friends.rs`, `dm.rs`, `server_channels.rs`, and `presence.rs`.
- Status: in progress; fixtures now cover missing route examples, auth cookie semantics, discovery query semantics, DM control-plane examples, invite create examples, DM fanout validation, internal-auth header/security/example gaps, session-auth security/status gaps, missing 401/500 branches including non-auth helper/delegate `500` flows, request/response schema alias handling, direct request-schema mismatch, response-header parity, requestBody/CSRF gaps, no-content success-schema regressions, and server-channel status-specific example checks with passing/failing counterparts. Remaining work is narrow targeted closeout only if another unprotected validator branch with real regression value appears.

## Recommended Order

- First wave: items 1, 2, and 8 breadth-closeout on real routes.
- Second wave: items 6 and 7 breadth expansion where runtime semantics remain stable enough to assert.
- Third wave: item 10 only if a new validator branch appears or a missing regression target is discovered.

## Working Rule

- Prefer grouped slices that close one coherent parity class across multiple routes.
- Avoid returning to one-status-at-a-time PRs unless a review finding forces a tiny follow-up.
