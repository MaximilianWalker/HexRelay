# HexRelay Navigation Implementation Plan

## Document Metadata

- Doc ID: navigation-implementation-plan
- Owner: Web and delivery maintainers
- Status: implementation_in_progress
- Scope: repository
- last_updated: 2026-05-26
- Source of truth: `docs/planning/navigation-implementation-plan.md`

## Quick Context

- Purpose: sequence the approved Iteration 2 navigation work for `T4.6.1` through `T4.6.4`.
- Primary edit location: update this file when navigation implementation sequencing, task slicing, runtime/API scope, or evidence changes.
- Latest meaningful change: 2026-05-26 added the shared `apps/web` UI framework path for navigation work: tokenized globals, UI primitives, hub feature components, chat components, settings rows, and onboarding shell extraction.

## Approved Baseline

- Keep the existing `WorkspaceShell`, sidebar/topbar switching, explicit collapse persistence, mobile top-level tabs, and workspace tab pin/close behavior.
- Treat `NAV-01` as a gap-fill foundation slice, not a rebuild.
- Servers and Contacts use the same hub interaction model: card/list layouts, search, Pinned/Unread/Muted filters, selection, shared bulk action patterns, and API-backed mutations.
- Servers Hub shows joined servers. Contacts Hub shows users/contacts.
- `Pinned` is the user-facing term for hub pin behavior.
- Desktop keeps sidebar/topbar switching and explicit collapse controls.
- Topbar tabs support pin/save through the tab context menu and manual reorder through drag-and-drop.
- Collapsed sidebar surface width and collapsed topbar surface height must share the same visual size.
- Collapsed mode hides labels/actions without changing the app edge spacing, panel padding, or gaps between chrome surfaces.
- Collapsed desktop workspace tabs must render as image-only affordances: square server images and round user/contact images, with no empty card shell after labels/actions are hidden.
- Mobile has only `Home`, `Servers`, `Contacts`, and `Settings` top-level tabs plus workspace drawers.
- Mobile does not expose a sidebar/topbar layout switch.
- Visible Create, Join, Leave, Pin, Mute, and Block + Remove controls must be functional when merged.

## Architecture Clarification

- A HexRelay server is a separately runnable server runtime authority with its own server identity and state boundary.
- The user app may spawn or supervise local server runtimes for convenience, and may connect to remote/dedicated servers, but the app is not the authority for many unrelated servers inside one shared API database.
- Current API storage uses one `local_server` singleton and server-local membership/channel/role/message tables. API paths using `server_id` are scoped to the connected server identity.
- App-level multi-server views aggregate joined server connections across server endpoints.
- Server runtime identity and authority remain canonical in `docs/architecture/adr-0004-server-authority.md`.

## Work Packages

| Package | Scope | Validation |
|---|---|---|
| `NAV-01` | Route constants/helpers for Home, Servers, Contacts, Settings, server workspace, and DM workspace; shared hub state model; per-device Servers/Contacts layout preference helpers | Web unit tests for route construction, serialization, defaults, and storage failure fallback |
| `NAV-02` | Servers Hub API-backed list/search/filter/card-list layout, pin/mute, Create, Join, Leave, selection, and destructive confirmation | Web tests, API integration tests, lint, coverage, build |
| `NAV-03` | Contacts Hub API-backed list/search/filter/card-list layout, pin/mute, Add contact via friend request, Block + Remove, selection, and destructive confirmation | Web tests, API integration tests, lint, coverage, build |
| `NAV-04` | Desktop workspace navigation gap-fill: tab reorder, pinned/saved tabs, sidebar/topbar switch, explicit collapse persistence | Web unit/render tests and desktop screenshot evidence |
| `NAV-05` | Mobile navigation gap-fill: fixed top-level tabs and workspace drawers with hub card/list toggle | Mobile screenshot evidence and browser checks |
| `NAV-06` | Evidence pack and Iteration 2 navigation closeout docs | `evidence/iteration-02/navigation/<YYYY-MM-DD>/` with screenshots, validators, summary, and provenance |

## Runtime/API Requirements

### Server Runtime, Create, And Join

- `Create Server` provisions a dedicated managed local server runtime.
- Default create path: app generates bootstrap/admin credentials, claims the active user as owner, and saves the connection.
- Advanced create path: user supplies or generates bootstrap credentials manually.
- `Join Server` uses invite link as the primary input.
- Advanced join form supports endpoint, server id, and invite token.
- Server creation/join runtime work may be developed in parallel with hub UI work, but visible controls must call real runtime/API behavior.

### Servers Hub Actions

- Server summaries expose `pinned`, `muted`, and `unread`.
- Pin/unpin and mute/unmute are API-backed membership preference updates.
- `Leave server` removes membership, removes the saved local connection from normal app state, closes related tabs, and removes the server from the hub.
- Leave confirmation includes checked-by-default `Delete local data for this server`.
- Local data deletion means app cache only for Iteration 2.
- Sole owner/admin users may leave; dedicated servers may continue existing without users.

### Contacts Hub Actions

- Contact summaries expose `pinned`, `muted`, and `unread`.
- Filters are `Pinned`, `Unread`, and `Muted`.
- Pin/unpin and mute/unmute are API-backed contact preference updates.
- Add contact uses search or direct identity lookup to send a friend request.
- `Block + Remove` blocks the target, removes accepted or pending relationship state, hides the target from Contacts, and preserves existing DM history.

## UX Requirements

- Shared card/list component model is used for Servers and Contacts through `apps/web/components/hubs`.
- Shared primitives in `apps/web/components/ui` own repeated buttons, badges, avatars, dialogs, fields, panels, notices, toolbars, and segmented controls.
- Shared CSS tokens in `apps/web/app/styles` own spacing, sizing, radii, typography, semantic colors, focus, z-index, motion, base resets, and theme overrides.
- New navigation UI CSS must consume semantic token variables instead of raw colors or one-off spacing/radius values.
- First-time defaults are cards on desktop and list on mobile.
- Card/list preference is per device and separate for Servers vs Contacts.
- Hub selection starts from long press, with right-click select/deselect available on desktop item context menus.
- Bulk actions include pin/unpin, mute/unmute, and the entity-specific destructive action.
- Baseline copy uses `Pinned`.
- Empty, error, and search-no-results copy uses shared templates with only the entity noun changing.
- Contacts copy uses `friend request` for send, accept, decline, and cancel actions.
- `Invite` language is reserved for server join and server peering flows.

## Test Plan

- Web unit tests for route helpers, hub state serialization, per-device layout preferences, tab reorder state, and storage failure fallback.
- API integration tests for server pin/mute/leave, contact pin/mute, atomic Block + Remove, and Create/Join credential flows.
- Web tests for shared hub filters, card/list toggle, multi-select, bulk actions, destructive dialogs, and friend-request add/respond states.
- Web gates: `npm --prefix apps/web run lint`, `npm --prefix apps/web run test:coverage`, `npm --prefix apps/web run build`.
- Rust/API gates: `cargo fmt --all`, `cargo check --workspace`, `cargo test --workspace`, and contract parity checks.
- Evidence pack: `evidence/iteration-02/navigation/<YYYY-MM-DD>/` with desktop/mobile screenshots and validator output.

## Implementation Status

| Area | Status |
|---|---|
| `NAV-01` foundation | in progress |
| Server runtime/Create/Join API | in progress |
| Servers Hub UI/actions | in progress |
| Contacts Hub UI/actions | in progress |
| Desktop navigation gap-fill | in progress |
| Mobile navigation gap-fill | in progress |
| Contract/doc alignment | in progress |
| Evidence pack | pending |

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/architecture/adr-0004-server-authority.md`
- `docs/testing/01-mvp-verification-matrix.md`
