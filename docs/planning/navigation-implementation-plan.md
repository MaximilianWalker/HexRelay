# HexRelay Navigation Implementation Plan

## Document Metadata

- Doc ID: navigation-implementation-plan
- Owner: Web and delivery maintainers
- Status: approval_pending
- Scope: repository
- last_updated: 2026-05-13
- Source of truth: `docs/planning/navigation-implementation-plan.md`

## Quick Context

- Purpose: sequence `T4.6.1` through `T4.6.4` without implementing product UI before explicit approval.
- Primary edit location: update this file when navigation implementation sequencing, task slicing, approval package, or validation evidence changes.
- Latest meaningful change: 2026-05-13 created the plan-only implementation path for Servers Hub, Contacts Hub, dual server navigation, and mobile navigation.

## Approval Boundary

This document is a plan-only artifact. It does not approve product UI implementation.

Implementation of `T4.6.1` through `T4.6.4` must wait until the user explicitly approves the proposed flow, copy, controls, and behavior for:

- global `Servers Hub`;
- global `Contacts Hub`;
- desktop server workspace navigation with sidebar and topbar modes;
- burger `expanded` / `collapsed` / `hidden` persistence;
- mobile top-level tabs and workspace drawers.

Until that approval exists, allowed work is limited to planning, test/evidence design, and non-runtime documentation.

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
| `T4.6.3` | Desktop server navigation | Topbar supports open/close/reorder/pin tabs and folder assignment; burger preference persists per device |
| `T4.6.4` | Mobile navigation | Mobile app shows `Home` / `Servers` / `Contacts` / `Settings` tabs and slide-in workspace drawers per spec |

## Implementation Principles

- Preserve the existing web data boundaries; prefer existing API client functions and local web state helpers before adding new backend routes.
- Keep hub and workspace navigation state explicit and serializable so it can be tested without browser-only assumptions.
- Persist device-scoped navigation preferences separately from user-scoped hub filters.
- Keep DM transport, endpoint-card, preflight, WAN wizard, and node-bypassing terminology out of navigation UI and tests.
- Treat server-channel and DM delivery summaries as backend state, not navigation labels.
- Keep desktop and mobile behavior aligned through shared state helpers, with layout-specific rendering only at the component boundary.

## Proposed Flow Package

Status: approval_pending. The following flow package must be approved before runtime UI work begins.

### Servers Hub

Flow:

1. User opens `Servers` from top-level navigation.
2. Hub loads joined servers in a searchable card grid on desktop and list-first layout on mobile.
3. User can filter to favorites, unread, or muted servers.
4. User can pin or unpin a server from the card action.
5. Opening a server deep-links to its server workspace and records it as the active server context.

Controls:

| Control | Proposed behavior |
|---|---|
| Search input | Case-insensitive server name filtering |
| Filter control | Favorites, unread, muted |
| Open action | Navigate to server workspace |
| Pin action | Toggle persistent favorite/pin state |
| Settings action | Navigate to server-scoped settings when available |

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
2. Hub loads contacts and DM threads in a searchable list, with optional card presentation where the layout supports it.
3. User can filter to online, unread, or favorite contacts.
4. Opening a contact deep-links to the DM workspace.
5. Mute/block actions use existing relationship policy surfaces and must preserve deterministic success/error states.

Controls:

| Control | Proposed behavior |
|---|---|
| Search input | Case-insensitive contact/thread filtering |
| Filter control | Online, unread, favorites |
| Open DM action | Navigate to the selected DM workspace |
| Mute action | Toggle mute state when supported |
| Block action | Trigger existing block policy action when supported |

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
5. Burger control cycles or selects `expanded`, `collapsed`, and `hidden` navigation visibility.
6. `ui.server_nav_mode` and `ui.server_nav_visibility` persist per device.

Controls:

| Control | Proposed behavior |
|---|---|
| Navigation mode control | Switch `sidebar` and `topbar` modes |
| Tab close action | Remove unsaved active tab |
| Tab pin action | Save or unsave tab |
| Tab reorder gesture/control | Persist manual tab order |
| Folder assignment | Move saved tab into a named folder |
| Burger control | Persist `expanded`, `collapsed`, or `hidden` visibility |

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
| Optional card/list toggle | Use only if approved with the hub flow |

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
| NAV-04 | Desktop server workspace mode switch, topbar tab state, saved tabs/folders, and burger visibility persistence | NAV-01, NAV-02 | Web render tests for mode switching, persisted preferences, and deep links |
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
- desktop screenshots for Servers Hub, Contacts Hub, sidebar mode, topbar mode, and burger visibility states;
- mobile screenshots for top-level tabs and workspace drawer;
- notes for any missing artifact with explicit rationale.

## Open Approval Questions

These questions must be answered or explicitly accepted as written before UI implementation starts:

| Area | Approval question |
|---|---|
| Servers Hub | Are the proposed filters, card actions, and copy baseline approved? |
| Contacts Hub | Are mute/block actions allowed directly from the hub, or should they stay inside contact detail surfaces first? |
| Topbar tabs | Should tab reordering be drag-and-drop only, button/menu driven, or both? |
| Burger visibility | Should the burger control cycle states in order or open a menu with explicit `expanded`, `collapsed`, and `hidden` choices? |
| Mobile drawer | Should drawer state reset on navigation, or persist for the current mobile session? |
| Card/list toggle | Should mobile hubs expose a card/list toggle in MVP, or stay list-first only? |

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
