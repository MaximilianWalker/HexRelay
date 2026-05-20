# HexRelay Navigation Implementation Plan

## Document Metadata

- Doc ID: navigation-implementation-plan
- Owner: Web and delivery maintainers
- Status: approval_in_progress
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/planning/navigation-implementation-plan.md`

## Quick Context

- Purpose: sequence `T4.6.1` through `T4.6.4` without implementing product UI before explicit approval.
- Primary edit location: update this file when navigation implementation sequencing, task slicing, approval package, or validation evidence changes.
- Latest meaningful change: 2026-05-20 recorded in-progress user decisions for the Iteration 2 navigation/UX approval package and locked the server-node authority clarification.

## Approval Boundary

This document is a plan-only artifact. It does not approve product UI implementation.

Implementation of `T4.6.1` through `T4.6.4` must wait until the user explicitly approves the final proposed flow, copy, controls, and behavior for:

- global `Servers Hub`;
- global `Contacts Hub`;
- desktop server workspace navigation with sidebar and topbar modes;
- explicit navigation collapse and sidebar/topbar switching without a burger control;
- mobile top-level tabs and workspace drawers.

Until that approval exists, allowed work is limited to planning, test/evidence design, and non-runtime documentation.

## In-Progress Decision Record

Status: in progress. These decisions were made during the 2026-05-20 UX planning discussion and must be carried into the final approval package. They are not runtime implementation approval by themselves.

### Locked UX Decisions

- Servers Hub and Contacts Hub must use the same interaction model. Both hubs support card and list layouts; one hub shows servers and the other shows users/contacts.
- The old burger-control proposal is rejected. Desktop navigation should use the existing collapse control and sidebar/topbar mode switch.
- Row/card primary action opens the server or DM. Secondary actions live in a menu; long click/long press enters selection.
- Multi-select is required in both hubs.
- Bulk actions may include destructive actions.
- Destructive actions map to `Leave server` for servers and `Block + Remove` for contacts.
- Destructive confirmations use a modal that shows the selected count and selected item names.
- Servers and Contacts remember their card/list layout independently.
- First-time layout defaults are cards on desktop and list on mobile, with the toggle available in both places.
- Shared top-level filters are `Favorites`, `Unread`, and `Muted`. Contact online/away/offline state is metadata, not a top-level filter.
- Empty, error, and search-no-results copy should use shared templates with only the noun changing between servers and contacts.
- Topbar tab reorder must support both drag-and-drop and menu/button controls.
- Mobile workspace drawer state resets on navigation.
- Hub actions that change state must be API-backed so they persist across devices/sessions.
- `Block + Remove` blocks the user, deletes the accepted/pending contact relationship, and hides the user from Contacts.
- `Block + Remove` keeps existing 1:1 DM history accessible.
- `Leave server` is allowed for owner/admin users only after a warning in the confirmation modal.
- Servers Hub empty state offers both `Join` and `Create`.
- `Create Server` is a real flow, not a placeholder.
- Create Server requires a name and supports optional icon and description fields.
- Server icons use generated color/initials, not image upload/media storage.
- Create Server defaults include one text channel and one voice channel.
- The default voice channel is metadata-only in Iteration 2; no join/call runtime is added in this slice.

### Locked Architecture Clarification

- The user-facing model should treat a HexRelay server as a separate server runtime/node, not as a user-owned client feature.
- The user app may spawn local server instances for convenience, but each spawned server acts as its own server runtime/node with separate identity and state.
- Two servers hosted on the same machine should still behave as distinct server instances/nodes.
- A server invite should feel like a server join: redeeming it should make the user belong to that server and make the server appear in the Servers Hub.
- Current API storage uses one `local_server` singleton and node-local membership/channel/role/message tables; it is not a many-servers-in-one-runtime model.
- Canonical authority: `docs/architecture/adr-0004-server-node-authority.md`.

### Architecture Reconfirmation Result

Status: accepted as of 2026-05-20. The architecture now explicitly matches the server-instance model, with a code guardrail keeping API-facing server membership scoped to the connected node/server identity.

- Matches user direction: `docs/architecture/04-communication-networking-layer-plan.md` says servers are runtime nodes in the server-node P2P network.
- Matches user direction: `docs/architecture/adr-0002-runtime-deployment-modes.md` keeps service boundaries explicit and treats dedicated server runtime as a separate headless service/package managed through the normal app.
- Matches user direction: `docs/architecture/01-system-overview.md` makes node-authoritative state live behind API/realtime services and says clients attach to nodes.
- Matches user direction: `docs/reference/glossary.md` now defines `Server` as a user-facing community backed by one node authority.
- Schema status: the current API schema has no multi-server `server_id` partition. Runtime authorization and directory listing are scoped to the local node fingerprint and singleton local-server storage.
- Implementation implication: Create/Join Server runtime work must provision/connect server runtimes and persist app-level connection state; it must not add a user-facing flow that creates many independent servers in one API database.

## Source Authorities

| Authority | Role |
|---|---|
| `docs/product/07-ui-navigation-spec.md` | Product/design authority for navigation hierarchy, hub behavior, desktop modes, and mobile behavior |
| `docs/product/08-screen-state-spec.md` | Required screen states and UX approval gate |
| `docs/product/09-configuration-defaults-register.md` | Default device preferences for `ui.server_nav_mode` and `ui.server_nav_visibility` |
| `docs/planning/iterations/02-sprint-board.md` | Task IDs, dependencies, and acceptance criteria |
| `docs/testing/01-mvp-verification-matrix.md` | Navigation evidence path and screenshot/checklist expectations |

## Selected Cluster

| Task ID | Delivery slice | Acceptance target |
|---|---|---|
| `T4.6.1` | Servers Hub | Search/filter/pin actions work and deep-link into server workspace |
| `T4.6.2` | Contacts Hub | Search/filter/open-DM actions work and state persists per user |
| `T4.6.3` | Desktop server navigation | Topbar supports open/close/reorder/pin tabs and folder assignment; sidebar/topbar mode plus sidebar collapse preferences persist per device |
| `T4.6.4` | Mobile navigation | Mobile app shows `Home` / `Servers` / `Contacts` / `Settings` tabs and slide-in workspace drawers per spec |

## Sprint-Board Status Contract

Until an approval reference with non-pending `NAV-APP-*` values exists, `T4.6.1` through `T4.6.4` must remain blocked follow-ups in `docs/planning/iterations/02-sprint-board.md`.

Move only the approved implementation slice back into active sprint-board execution when its approval exists, using the canonical `Approval-To-Slice Map` below for the decision-to-task mapping.

Do not mark any `T4.6.x` task done until the associated runtime behavior, validation, screenshots/checklists, and evidence pack exist.

## Current Plan-Only Split Rationale

The full `T4.6.1` through `T4.6.4` cluster spans four user-facing navigation surfaces and requires explicit approval before any runtime UI implementation. The smallest mergeable prerequisite is therefore this plan-only approval package:

- keep the proposed flow, controls, copy, and behavior in one reviewable authority;
- define the exact user approvals needed before Web changes begin;
- split implementation into independently mergeable slices after approval;
- define validation and evidence expectations before implementation starts.

This PR must not claim any `T4.6.x` runtime acceptance criteria as complete. After approval, the first implementation PR should start with `NAV-01` only unless the approved scope explicitly allows a larger slice.

## Plan-Only Change Scope

Plan-only navigation PRs may refine only approval, sequencing, validation, evidence, and documentation freshness for `T4.6.1` through `T4.6.4`.

Allowed plan-only changes:

- clarify proposed flows, controls, copy, persistence choices, and approval questions;
- map pending approval decisions to the first implementation slice that needs them;
- refine validation commands, screenshot/checklist evidence, and PR-body requirements;
- update docs indexes and planning references that point at this approval package.

Disallowed without explicit UX approval:

- editing `apps/web` runtime UI, route behavior, storage helpers, fixtures consumed by visible UI, or browser tests that encode new UX behavior;
- changing API/realtime contracts to support navigation behavior;
- adding evidence that claims `T4.6.1` through `T4.6.4` runtime acceptance criteria are met;
- treating this plan, sprint-board task selection, or an automated PR merge as approval for any user-visible flow, copy, control, or behavior.

Plan-only PRs may merge before UX approval because they do not change product behavior. The next runtime PR must cite the exact approval reference and approved `NAV-APP-*` values it implements.

## Implementation Principles

- Preserve the existing web data boundaries; prefer existing API client functions and local web state helpers before adding new backend routes.
- Keep hub and workspace navigation state explicit and serializable so it can be tested without browser-only assumptions.
- Persist device-scoped navigation preferences separately from user-scoped hub filters.
- Keep DM transport, endpoint-card, preflight, WAN wizard, and node-bypassing terminology out of navigation UI and tests.
- Treat server-channel and DM delivery summaries as backend state, not navigation labels.
- Keep desktop and mobile behavior aligned through shared state helpers, with layout-specific rendering only at the component boundary.

## Proposed Flow Package

Status: approval_in_progress. The following flow package must be finalized before runtime UI work begins.

### Servers Hub

Flow:

1. User opens `Servers` from top-level navigation.
2. Hub loads joined servers in searchable card and list layouts.
3. User can filter to favorites, unread, or muted servers.
4. User can switch between card and list layouts; Servers and Contacts remember layout independently.
5. User can pin/unpin, mute/unmute, open settings when available, or leave from the row/card menu.
6. Opening a server deep-links to its server workspace and records it as the active server context.

Controls:

| Control | Proposed behavior |
|---|---|
| Search input | Case-insensitive server name filtering |
| Filter control | Favorites, unread, muted |
| Row/card primary action | Navigate to server workspace |
| Row/card menu | Pin/unpin, mute/unmute, settings when available, leave |
| Multi-select | Select multiple servers for bulk actions |
| Destructive confirmation | Show selected count and server names before leave |

Copy baseline:

| State | Proposed copy |
|---|---|
| Empty | `No servers yet. Join or create a server to get started` |
| Search no results | `No servers match your search` |
| Permission denied | `You do not have access to this server` |
| Error | `Servers could not load` |

### Contacts Hub

Flow:

1. User opens `Contacts` from top-level navigation.
2. Hub loads contacts and DM threads in searchable card and list layouts.
3. User can switch between card and list layouts; Servers and Contacts remember layout independently.
4. User can filter to favorites, unread, or muted contacts.
5. Opening a contact deep-links to the DM workspace.
6. Mute and block/remove actions use API-backed relationship policy surfaces and must preserve deterministic success/error states.

Controls:

| Control | Proposed behavior |
|---|---|
| Search input | Case-insensitive contact/thread filtering |
| Filter control | Favorites, unread, muted |
| Row/card primary action | Navigate to the selected DM workspace |
| Row/card menu | Mute/unmute, block/remove |
| Multi-select | Select multiple contacts for bulk actions |
| Destructive confirmation | Show selected count and contact names before block/remove |

Copy baseline:

| State | Proposed copy |
|---|---|
| Empty | `No contacts yet. Add a friend or redeem a contact invite` |
| Search no results | `No contacts match your search` |
| Pending request | `Request pending` |
| Inbound request | `Respond to request` |
| Error | `Contacts could not load` |

### Desktop Server Workspace

Flow:

1. User opens a server workspace from the Servers Hub, saved tab, direct link, or prior session.
2. Sidebar mode shows server/channel navigation as the default desktop mode.
3. Topbar tab mode shows active server contexts as reorderable tabs.
4. User can open, close, reorder, pin/save, and folder saved tabs.
5. No burger control is used in Iteration 2.
6. Explicit controls switch between sidebar and topbar modes.
7. Explicit collapse control expands/collapses sidebar navigation.
8. Navigation mode and collapse preferences persist per device.

Controls:

| Control | Proposed behavior |
|---|---|
| Navigation mode control | Switch `sidebar` and `topbar` modes |
| Tab close action | Remove unsaved active tab |
| Tab pin action | Save or unsave tab |
| Tab reorder gesture/control | Persist manual tab order |
| Folder assignment | Move saved tab into a named folder |
| Mode switch control | Persist sidebar/topbar mode |
| Collapse control | Persist sidebar expanded/collapsed state |

Copy baseline:

| State | Proposed copy |
|---|---|
| Channel empty | `No messages yet` |
| Permission denied | `You cannot view this channel` |
| Reconnecting | `Reconnecting` |
| Error | `Server workspace could not load` |

### Mobile Navigation

Flow:

1. Mobile shows top-level tabs for `Home`, `Servers`, `Contacts`, and `Settings`.
2. Servers and Contacts hubs default to dense list layouts.
3. Server workspace opens channel/server navigation in slide-in drawers.
4. Drawer state is transient unless a future approved preference says otherwise.

Controls:

| Control | Proposed behavior |
|---|---|
| Top-level tabs | Switch between `Home`, `Servers`, `Contacts`, and `Settings` |
| Workspace drawer button | Open server/channel navigation drawer |
| Drawer close action | Return focus to active workspace |
| Card/list toggle | Available in Servers and Contacts hubs |

Copy baseline:

| State | Proposed copy |
|---|---|
| Drawer label | `Server navigation` |
| Drawer close | `Close navigation` |
| Search no results | Reuse hub-specific no-results copy |
| Error | Reuse hub/workspace error copy |

## Work Packages

| Package | Scope | Dependencies | Suggested validation |
|---|---|---|---|
| NAV-01 | Navigation state model, preference helpers, route constants, and fixture shape for hub/workspace tests | Approval package accepted | Web unit tests for preference serialization and route construction |
| NAV-02 | `Servers Hub` data loading, search/filter state, pin action wiring, desktop/mobile responsive layout | NAV-01, existing server/channel APIs | Web render tests, lint, coverage, build |
| NAV-03 | `Contacts Hub` search/filter/open-DM behavior and persisted per-user hub state | NAV-01, existing contacts/DM APIs | Web render tests, existing contacts helper tests, lint, coverage, build |
| NAV-04 | Desktop server workspace mode switch, topbar tab state, saved tabs/folders, and sidebar collapse persistence | NAV-01, NAV-02 | Web render tests for mode switching, persisted preferences, and deep links |
| NAV-05 | Mobile top-level tabs and workspace drawer behavior | NAV-01, NAV-02, NAV-04 | Browser screenshot checklist for mobile widths plus render tests |
| NAV-06 | Navigation evidence pack and iteration closeout docs | NAV-02 through NAV-05 | `evidence/iteration-02/navigation/<YYYY-MM-DD>/` with checklist, screenshots, validators, and provenance |

## Smallest Mergeable Slices After Approval

The full `T4.6.1` through `T4.6.4` cluster is large enough that implementation should be split after approval:

1. `NAV-01` plus the shared test fixture/state foundation.
2. `NAV-02` Servers Hub.
3. `NAV-03` Contacts Hub.
4. `NAV-04` desktop dual-mode navigation and persistence.
5. `NAV-05` mobile navigation and drawer behavior.
6. `NAV-06` evidence closeout and sprint-board status update.

Each slice should be independently mergeable and should not claim task completion until the associated acceptance criteria and evidence are present.

## Slice Exit Criteria

| Slice | May merge when | Must not claim |
|---|---|---|
| `NAV-01` | Route constants, serializable navigation state helpers, and preference persistence helpers have focused web tests | Any visible hub/workspace UI behavior |
| `NAV-02` | Servers Hub search, filters, pin action wiring, deep-link behavior, responsive layout, and required states pass web validation | Contacts Hub or desktop/mobile navigation completion |
| `NAV-03` | Contacts Hub search, filters, open-DM behavior, persisted user hub state, and required states pass web validation | Servers Hub, desktop mode, or mobile drawer completion |
| `NAV-04` | Desktop sidebar/topbar mode switch, tab open/close/reorder/pin/folder behavior, and sidebar collapse persistence pass web validation | Mobile top-level tabs or mobile drawer completion |
| `NAV-05` | Mobile `Home` / `Servers` / `Contacts` / `Settings` tabs and workspace drawer behavior pass browser screenshot review plus render coverage | Full navigation closeout without evidence artifacts |
| `NAV-06` | Navigation evidence pack exists with required screenshots, checklist, validators, and provenance, and the sprint board is updated to reflect completed acceptance evidence | New runtime behavior beyond evidence/docs closeout |

Each implementation slice must cite the approved `NAV-APP-*` decisions it uses. Missing approval is a hard stop for that slice.

## Validation Plan

Before opening each implementation PR after approval:

- run `npm --prefix apps/web run lint`;
- run `npm --prefix apps/web run test:coverage`;
- run `npm --prefix apps/web run build`;
- cite the explicit UX approval in the PR body, as required by `AGENTS.md` and `docs/product/08-screen-state-spec.md`;
- collect desktop and mobile screenshots/checklist evidence under `evidence/iteration-02/navigation/<YYYY-MM-DD>/` once the UI behavior exists.

For this plan-only PR, validation is limited to docs freshness and plan-only diff scope checks.

## Evidence Checklist

When implementation is approved and complete, the navigation evidence pack must include:

- `summary.md` with covered task IDs, approved UX reference, scope, and outcome;
- `validators.txt` with exact commands and manual checks;
- `provenance.json` with commit SHA, PR number or run ID, and generation timestamp;
- desktop screenshots for Servers Hub, Contacts Hub, sidebar mode, topbar mode, and collapsed sidebar state;
- mobile screenshots for top-level tabs and workspace drawer;
- notes for any missing artifact with explicit rationale.

## Open Approval Questions

These questions must be answered or explicitly accepted as written before UI implementation starts:

| Area | Approval question |
|---|---|
| Servers Hub | Final copy and exact create/join controls still need final approval after architecture validation. |
| Contacts Hub | Final copy and exact add/invite controls still need final approval. |
| Topbar tabs | Reorder model is in-progress locked as both drag-and-drop and menu/button controls. |
| Navigation visibility | Burger proposal is rejected; final implementation should use sidebar/topbar switch plus collapse control. |
| Mobile drawer | Drawer state is in-progress locked to reset on navigation. |
| Card/list toggle | Both Servers and Contacts hubs expose card/list toggle. |

## Approval Decision Record

Record approval against each decision below before runtime UI work begins. Approval may live in a PR comment, issue comment, or project note, but the first implementation PR must cite the exact approval reference.

| Decision ID | Required approval | Approved value |
|---|---|---|
| `NAV-APP-01` | Servers Hub filters, card actions, and state copy | in progress: shared hub model, Favorites/Unread/Muted filters, card/list layouts, primary open action, menu actions, multi-select, destructive leave confirmation |
| `NAV-APP-02` | Contacts Hub filters, open-DM action, and whether mute/block are available from the hub | in progress: shared hub model, Favorites/Unread/Muted filters, card/list layouts, primary open-DM action, menu actions, multi-select, destructive block/remove confirmation, DM history retained |
| `NAV-APP-03` | Topbar tab reorder control model: drag-and-drop, menu/buttons, or both | in progress: both drag-and-drop and menu/button controls |
| `NAV-APP-04` | Navigation visibility control model | in progress: no burger; sidebar/topbar switch plus explicit sidebar collapse |
| `NAV-APP-05` | Mobile workspace drawer persistence: reset on navigation or persist for the mobile session | in progress: reset on navigation |
| `NAV-APP-06` | Mobile hub presentation and card/list layout availability | in progress: card/list toggle available in both hubs; first-time defaults are cards on desktop and list on mobile |

If any decision is explicitly deferred, implementation for the affected surface must either avoid that behavior or stay plan-only for that slice.

## Approval-To-Slice Map

| Approval decision | Blocks task/slice | First eligible implementation PR after approval | Required approval evidence |
|---|---|---|---|
| `NAV-APP-01` | `T4.6.1` / `NAV-02` Servers Hub | Servers Hub data/render/action implementation | Approved filters, card actions, and state copy |
| `NAV-APP-02` | `T4.6.2` / `NAV-03` Contacts Hub | Contacts Hub data/render/action implementation | Approved filters, open-DM action, copy, and mute/block availability |
| `NAV-APP-03` | `T4.6.3` / `NAV-04` topbar tabs | Desktop topbar tab state and controls | Approved reorder interaction model |
| `NAV-APP-04` | `T4.6.3` / `NAV-04` navigation visibility persistence | Desktop navigation visibility control | Approved no-burger sidebar/topbar switch and collapse behavior |
| `NAV-APP-05` | `T4.6.4` / `NAV-05` mobile drawer | Mobile workspace drawer behavior | Approved drawer reset behavior |
| `NAV-APP-06` | `T4.6.4` / `NAV-05` mobile hub presentation | Mobile hub presentation and card/list toggle | Approved card/list toggle behavior |

`NAV-01` may be approved separately as a no-visible-UI foundation slice. If only `NAV-01` is approved, the implementation PR must stay limited to route constants, serializable state helpers, preference helpers, shared fixture foundations that do not encode visible UI behavior, and tests for those helpers/fixtures.

## Approval Response Template

Use this template in the approving PR comment, issue comment, or project note. The first runtime UI implementation PR must link to the exact approval reference and copy the approved decision values into its PR body.

```text
Navigation approval reference:
- Scope approved: NAV-01 only | NAV-02 | NAV-03 | NAV-04 | NAV-05 | NAV-06 evidence closeout | full T4.6.1-T4.6.4 sequence
- NAV-APP-01 Servers Hub filters/actions/copy: approved as written | approved with changes: <changes> | deferred
- NAV-APP-02 Contacts Hub filters/actions/copy and hub mute/block availability: approved as written | approved with changes: <changes> | deferred
- NAV-APP-03 Topbar tab reorder model: drag-and-drop | menu/buttons | both | deferred
- NAV-APP-04 Navigation visibility control model: no burger, sidebar/topbar switch plus collapse | deferred
- NAV-APP-05 Mobile drawer persistence: reset on navigation | persist for session | deferred
- NAV-APP-06 Mobile hub presentation: card/list toggle with list mobile default | deferred
- Additional constraints: <any limits, exclusions, or required validation>
```

Approval can cover one slice at a time. If approval covers only `NAV-01`, runtime work remains limited to shared state, route, preference, and fixture foundations with no visible hub or workspace behavior.

## Pre-Implementation Gate Checklist

Before any runtime UI implementation branch starts, verify all of the following:

| Gate | Required evidence |
|---|---|
| Approval reference exists | PR/issue/project note link with non-pending `NAV-APP-*` values for the slice |
| Scope is slice-bounded | PR body names the exact `NAV-*` slice and the related `T4.6.x` acceptance target |
| UX copy/control deltas are frozen for the slice | Any changed copy, filters, controls, persistence behavior, or mobile behavior appears in the approval reference |
| Validation commands are selected | PR body lists the web lint, coverage, build, render/browser, and screenshot checks that apply to the slice |
| Evidence path is reserved when UI exists | `evidence/iteration-02/navigation/<YYYY-MM-DD>/` path and artifact list are named before screenshot capture |
| No unapproved adjacent behavior is bundled | PR body explicitly excludes deferred `NAV-APP-*` decisions and unrelated DM/server-channel UX changes |

Missing approval evidence is a hard stop. Do not use a plan-only PR, an implementation-ready task row, or existing partially built web components as implied approval for product UI behavior.

## Runtime Implementation Hard Stops

Stop and keep the work plan-only when any of these conditions apply:

- the slice has no approval reference with non-pending `NAV-APP-*` values;
- the approval reference changes copy, controls, or behavior but the PR body does not quote the approved value set;
- the implementation attempts to include a later `NAV-*` slice whose approval is missing or deferred;
- the slice needs a new backend/API field or contract that is not already approved and scoped in the implementation PR;
- validation evidence would require claiming a `T4.6.x` acceptance criterion before the corresponding runtime behavior and screenshots/checklists exist.

## Known Limits

- This plan does not introduce backend API contracts.
- This plan does not approve any runtime UI copy, control, or behavior.
- Navigation implementation may discover missing server/contact API fields; those must be handled in the future implementation slice with contract updates if needed.
- Manual screenshot evidence remains required until deterministic browser/render coverage is added for every navigation acceptance state.

## Related Documents

- `docs/product/07-ui-navigation-spec.md`
- `docs/product/08-screen-state-spec.md`
- `docs/product/09-configuration-defaults-register.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/testing/01-mvp-verification-matrix.md`
