# HexRelay Clarifications Log

## Document Metadata

- Doc ID: clarifications-log
- Owner: Product maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/03-clarifications.md`

## Quick Context

- Primary edit location for unresolved product and architecture questions.
- Move resolved items to the resolved section and update linked canonical docs in the same PR.
- Latest meaningful change: 2026-03-04 locked DM offline behavior to best-effort online with encrypted local outbox retries.

## Purpose

- Track open product and architecture clarifications without polluting canonical requirement text.
- Provide a single queue for assumptions that need explicit resolution.

## Clarification Entry Format

- ID
- Status (`open`, `resolved`, `deferred`)
- Area
- Question
- Current assumption
- Resolution (if resolved)
- Linked docs

## Active Clarifications

- None currently.

## Resolved Clarifications

- C-005 (resolved 2026-03-04): Invite scope semantics in MVP are limited to join eligibility only.
- C-006 (resolved 2026-03-04): MVP Crypto Profile v1 is locked for identity/auth and baseline E2EE execution.
- C-007 (resolved 2026-03-04): Iteration 1 identity/invite/auth OpenAPI endpoint baseline is locked before parallel API/web work.
- C-001 (resolved 2026-03-04): Group DM E2EE is required in MVP.
- C-002 (resolved 2026-03-04): Discovery abuse baseline is signed registry plus rate limits and denylist support.
- C-003 (resolved 2026-03-04): Join/auth onboarding UX uses dedicated per-case states with guided recovery actions.
- C-004 (resolved 2026-03-04): Recovery phrase setup is mandatory during onboarding.
- C-008 (resolved 2026-03-04): UI direction is heavily Discord-inspired but server navigation cannot use small icon-circle rails as the primary pattern.
- C-009 (resolved 2026-03-04): MVP includes dedicated global `Servers` and `Contacts` hub pages with searchable card-oriented browsing.
- C-010 (resolved 2026-03-04): Server navigation supports both sidebar mode and browser-like topbar tab mode with saved tabs and folders.
- C-011 (resolved 2026-03-04): Server workspace includes a burger control to shrink/hide server navigation chrome.
- C-012 (resolved 2026-03-04): Realtime event/signaling contracts are formalized in a versioned AsyncAPI artifact for Iterations 2-3.
- C-013 (resolved 2026-03-04): KPI/SLO validation uses a fixed test profile (`200 users`, `70/30 WiFi/Fast4G`, latest stable Chrome/Firefox, single-region staging).
- C-014 (resolved 2026-03-04): Migration conflict precedence uses user-signed profile data as canonical; server-owned security/membership fields remain server-authoritative.
- C-015 (resolved 2026-03-04): Post-MVP discovery follows a hybrid roadmap: federation remains supported, trusted registries are added, and full P2P discovery becomes an optional mode.
- C-016 (resolved 2026-03-04): MVP supports direct user add through expiring contact invite link and QR redeem flow.
- C-017 (resolved 2026-03-04): Server invites support optional expiration/max-uses, including non-expiring multi-use links for open-access behavior.
- C-018 (resolved 2026-03-04): Server-mediated friend requests are intent-based, raw key/profile-identifying data is not exposed by default, and DM inbound policy defaults to friends-only with user opt-in overrides.
- C-019 (resolved 2026-03-04): DMs use direct user-to-user transport and are not relayed or stored by guild/community servers.
- C-020 (resolved 2026-03-04): MVP DM offline behavior is best-effort online delivery with encrypted local outbox retries and no guaranteed offline queue.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/product/04-dependencies-risks.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/contracts/realtime-events-v1.asyncapi.yaml`
- `docs/planning/kpi-slo-test-profile.md`
