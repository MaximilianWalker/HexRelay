# HexRelay UI Navigation Spec (MVP)

## Document Metadata

- Doc ID: ui-navigation-spec
- Owner: Product and design maintainers
- Status: revision_in_progress
- Scope: repository
- last_updated: 2026-06-10
- Source of truth: `docs/product/07-ui-navigation-spec.md`

## Quick Context

- Purpose: define the MVP navigation and layout model with Discord-like UX baseline and explicit deviations.
- Primary edit location: update this file when navigation patterns, primary surfaces, or interaction hierarchy changes.
- Latest meaningful change: 2026-06-10 split shared list/menu primitives so `List` owns customizable rows, icon color, and optional panel framing while `Menu` owns object-driven navigation/action lists, overall spacing, optional idle borders, and sidebar skin limited to cosmetic exceptions.

## Design Direction

- Baseline: heavily Discord-inspired interaction and layout conventions for familiarity.
- Explicit deviation: server navigation must not use the small circular icon rail as the primary pattern.
- Goal: preserve fast switching behavior while improving scalability for large server lists.
- Navigation style requirement: browser-like affordances are used for server switching through tabs, pinned/saved tabs, and manual reorder.
- UX approval gate: no UX flow, copy, control, or behavior change may be implemented until the user explicitly consents to it.
- Current approval status: Iteration 2 navigation UX is approved for implementation according to this document and `docs/planning/navigation-implementation-plan.md`.

## UI Framework Conventions

- Shared UI code for `apps/web` lives under `apps/web/components/ui`, with token-backed CSS Modules and no route-owned visual systems for repeated controls.
- The UI framework is layered as tokens/themes, headless behavior hooks, primitives, feature recipes, and route composition. Route files should orchestrate data and layout instead of owning repeated control semantics.
- Global CSS is split into `apps/web/app/styles/tokens.css`, `themes.css`, and `base.css`, imported only through `apps/web/app/globals.css`.
- Component CSS should use semantic variables such as `--color-bg-app`, `--color-surface`, `--color-border`, `--color-text`, `--color-text-muted`, `--color-accent`, `--color-danger`, `--color-warning`, and `--color-success`.
- Primitive palette values stay private to the theme files. Custom themes override semantic variables instead of editing component internals.
- Reusable primitives are `Avatar`, `Badge`, `Button`, `ButtonLink`, `Dialog`, `EmptyState`, `Field`, `IconButton`, `List`, `ListButton`, `ListLink`, `ListRow`, `Menu`, `Notice`, `Panel`, `SegmentedControl`, `ToggleButton`, `ToggleGroup`, `Toolbar`, and `VisuallyHidden`.
- `List` is the low-level customizable row primitive for popup rows, static rows, links, buttons, icons, icon color, end slots, and optional panel framing through `panel`. `Menu` composes `List` from item objects for sidebars, channel rails, action lists, and nested navigation, and exposes `panel`, `spacing`, plus `idleBorder` for borderless idle rows that still gain borders on hover, focus, or active state. Core row sizing and spacing stay in `Menu`; named skins are exceptions and must stay cosmetic. The app sidebar uses `panel`, `spacing="sm"`, `idleBorder={false}`, the explicit `sidebar` skin, accent icons, and a rail indicator without arrow decoration.
- Component and file names should use folder context instead of repeating it. For example, `components/hubs/toolbar.tsx` exports `Toolbar`; callers rely on the import path for hub context.
- Feature components own repeated behavior by surface:
  - `components/hubs`: toolbar, bulk actions, card/list surface, item rendering, and context menu behavior.
  - `components/chat`: channel rail, message timeline, message row, composer, and presence indicators.
  - `components/settings`: setting panel, row, and status alignment.
  - `components/onboarding`: onboarding shell and step layout.

### Spacing And Shape Rules

- Use the shared spacing scale from `tokens.css`: `--space-1` is 2px, `--space-4` is 8px, `--space-8` is 16px, `--space-12` is 24px, and `--space-20` is 40px.
- Use named gap tokens for repeated layouts: `--gap-icon`, `--gap-control`, `--gap-list`, `--gap-panel`, and `--gap-section`.
- Use named padding tokens for controls and containers: `--pad-control-sm`, `--pad-control-md`, `--pad-field`, `--pad-card`, `--pad-panel`, `--pad-dialog`, and `--app-edge`.
- Route CSS must not introduce one-off spacing, radii, or raw colors for repeated UI patterns. Local spacing is acceptable only when the physical layout requires it.
- Shared radii are `--radius-xs`, `--radius-sm`, `--radius-md`, `--radius-lg`, `--radius-xl`, `--radius-panel`, `--radius-dialog`, and `--radius-pill`.
- User avatars are round. Server avatars are rounded-square. This rule applies to expanded and collapsed navigation states.

### CSS Validation

- `npm --prefix apps/web run lint:styles` checks new framework CSS for raw color usage.
- `npm --prefix apps/web run lint` also runs naming and UI-framework validators that reject redundant path-context names, repeated local pressed controls, and route-owned copies of shared controls.
- Raw hex, rgb, rgba, hsl, hsla, one-off repeated radii, and one-off repeated spacing should be moved into tokens or semantic theme variables when promoting route CSS into shared components.

## Primary App Surfaces

- `Home`: landing surface for recent activity and quick resume.
- `Servers Hub`: global page showing all joined servers as cards or dense list.
- `Contacts Hub`: global page showing friends/DM threads as cards or dense list.
- `Server Workspace`: selected server with channel navigation and message area.
- `DM Workspace`: selected private DM/group DM with message area.

## Navigation Model (MVP)

### Top-Level

- Primary left navigation uses labeled sections instead of icon-only circles.
- Required top-level entries:
  - `Home`
  - `Servers`
  - `Contacts`
  - `Settings`

### Server Navigation

- Dual navigation modes are required:
  - `Sidebar Mode`: persistent list/tree navigation for broad browsing.
  - `Topbar Tab Mode`: browser-like tabs for active server contexts.
- Sidebar server list item format:
  - server name
  - optional compact icon/avatar
  - unread badge and mention badge
  - pinned indicator
- Sorting baseline:
  - pinned first
  - then manual or recency order
- Topbar tab requirements:
  - open/close/reorder tabs
  - pin/save tabs for persistent quick access
  - unread and mention indicators on tabs
  - expanded tabs may show labels and actions; collapsed tabs must reduce to image-only targets
- Navigation visibility behavior:
  - explicit controls switch between `Sidebar Mode` and `Topbar Tab Mode`
  - explicit collapse/expand control applies to sidebar visibility
  - user preference persists per device
  - no mobile sidebar/topbar layout switch
  - collapsed sidebar surface width and collapsed topbar surface height use the same visual size
  - collapsed mode hides labels/actions but must not reduce the app edge spacing, panel padding, or gaps between chrome surfaces
- Collapsed desktop navigation language:
  - server workspace tabs collapse to the server image only
  - contact/DM workspace tabs collapse to the user image only
  - user images are always round
  - server images are always square or softly rounded-square
  - collapsed tabs must not leave empty bordered card shells after labels/actions are hidden

### Global Hubs

- `Servers Hub` requirements:
  - searchable card and list layouts over joined servers
  - filters: Pinned, Unread, Muted
  - row/card primary action opens the server
  - create action provisions a managed local server runtime
  - join action accepts a server invite link, with advanced endpoint/server-id/invite-token fields available
  - row/card and bulk actions include pin/unpin, mute/unmute, and destructive leave
  - leave confirmation includes checked-by-default `Delete local data for this server`
- `Contacts Hub` requirements:
  - searchable people/thread card and list layouts
  - filters: Pinned, Unread, Muted
  - add action uses user search or direct identity lookup to send a friend request
  - inbound/outbound friend requests appear as pending contact states with accept, decline, or cancel actions where applicable
  - row/card primary action opens the DM
  - row/card and bulk actions include pin/unpin, mute/unmute, and destructive Block + Remove

## Why This Pattern

- Better scalability than circle rails for users in many servers.
- Preserves fast-access behavior through pinning and saved tabs.
- Adds a global management view for both community and social contexts.

## Interaction and State Rules

- Search is case-insensitive and keyboard-focusable from hubs.
- Empty states must include a next action (join server, create server, search user, send friend request, start DM).
- Selected destination from a hub must deep-link directly into target workspace.
- Servers Hub aggregates joined servers from app connection/membership state; it is not evidence that one API runtime owns every listed server.
- Card/list layout preference persists per device and is separate for Servers and Contacts.
- Servers Hub and Contacts Hub use the same card/list, filter, selection, and action-menu model with only the entity noun changing.
- Selection starts from long press; desktop right-click context menus expose select/deselect for items. Hubs do not show a persistent Select button.
- Destructive bulk actions require a confirmation modal that shows selected count and selected item names.
- Server leave removes membership, removes the saved local connection from normal app state, closes related tabs, and removes the server from the hub; Iteration 2 local data deletion is app cache only.
- Block + Remove blocks the target, removes accepted or pending contact relationship state, hides the contact from the hub, and preserves existing DM history.
- `Invite` language is reserved for server join and server peering UX. Contacts uses `friend request` language for send, accept, decline, and cancel actions.

## Mobile Behavior (MVP)

- Use a tabbed top-level switcher: `Home`, `Servers`, `Contacts`, `Settings`.
- Hubs default to list mode on mobile and still expose the approved card/list toggle.
- Workspace navigation uses slide-in drawers for server/channel trees.
- Mobile workspace drawer state resets on navigation.

## Deferred (Post-MVP)

- Custom visual themes per workspace.
- Multi-panel desktop layouts with persistent split views.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/planning/iterations/01-sprint-board.md`
