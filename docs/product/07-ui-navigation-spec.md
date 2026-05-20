# HexRelay UI Navigation Spec (MVP)

## Document Metadata

- Doc ID: ui-navigation-spec
- Owner: Product and design maintainers
- Status: revision_in_progress
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/product/07-ui-navigation-spec.md`

## Quick Context

- Purpose: define the MVP navigation and layout model with Discord-like UX baseline and explicit deviations.
- Primary edit location: update this file when navigation patterns, primary surfaces, or interaction hierarchy changes.
- Latest meaningful change: 2026-05-20 locked Iteration 2 navigation decisions, aligned navigation terminology with the accepted server authority model, and locked friend-request-only Contacts behavior.

## Design Direction

- Baseline: heavily Discord-inspired interaction and layout conventions for familiarity.
- Explicit deviation: server navigation must not use the small circular icon rail as the primary pattern.
- Goal: preserve fast switching behavior while improving scalability for large server lists.
- Navigation style requirement: browser-like affordances are used for server switching through tabs, pinned/saved tabs, and manual reorder.
- UX approval gate: no UX flow, copy, control, or behavior change may be implemented until the user explicitly consents to it.
- Current approval status: Iteration 2 navigation UX is approved for implementation according to this document and `docs/planning/navigation-implementation-plan.md`.

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
- Navigation visibility behavior:
  - explicit controls switch between `Sidebar Mode` and `Topbar Tab Mode`
  - explicit collapse/expand control applies to sidebar visibility
  - user preference persists per device
  - no mobile sidebar/topbar layout switch

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
