# HexRelay UI Navigation Spec (MVP)

## Document Metadata

- Doc ID: ui-navigation-spec
- Owner: Product and design maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/product/07-ui-navigation-spec.md`

## Quick Context

- Purpose: define the MVP navigation and layout model with Discord-like UX baseline and explicit deviations.
- Primary edit location: update this file when navigation patterns, primary surfaces, or interaction hierarchy changes.
- Latest meaningful change: 2026-03-04 added dual-mode server navigation (sidebar + topbar tabs) and burger collapse behavior.

## Design Direction

- Baseline: heavily Discord-inspired interaction and layout conventions for familiarity.
- Explicit deviation: server navigation must not use the small circular icon rail as the primary pattern.
- Goal: preserve fast switching behavior while improving scalability for large server lists.
- Navigation style requirement: browser-like affordances are encouraged for server switching (tabs, saved tabs, folders).

## Primary App Surfaces

- `Home`: landing surface for recent activity and quick resume.
- `Servers Hub`: global page showing all joined servers as cards.
- `Contacts Hub`: global page showing friends/DM threads as cards or dense list.
- `Server Workspace`: selected server with channel navigation and message area.
- `DM Workspace`: selected direct message/group DM with message area.

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
  - favorite/pin indicator
- Server grouping:
  - folder support required in MVP
  - collapse/expand behavior required
- Sorting baseline:
  - pinned/favorites first
  - then manual order within folders
- Topbar tab requirements:
  - open/close/reorder tabs
  - pin/save tabs for persistent quick access
  - folder assignment for saved tabs
  - unread and mention indicators on tabs
- Burger toggle behavior:
  - visible in server workspace header
  - toggles sidebar to `expanded`/`collapsed`/`hidden`
  - user preference persists per device

### Global Hubs

- `Servers Hub` requirements:
  - searchable card grid
  - filters: favorites, unread, muted
  - card actions: open server, open settings, pin/unpin
- `Contacts Hub` requirements:
  - searchable people/thread list with optional card mode
  - filters: online, unread, favorites
  - row/card actions: open DM, mute/unmute, block/unblock

## Why This Pattern

- Better scalability than circle rails for users in many servers.
- Preserves fast-access behavior through pinning/folders.
- Adds a global management view for both community and social contexts.

## Interaction and State Rules

- Search is case-insensitive and keyboard-focusable from hubs.
- Empty states must include a next action (join server, create server, add friend, start DM).
- Selected destination from a hub must deep-link directly into target workspace.
- Hub filters and sort preferences persist per user.

## Mobile Behavior (MVP)

- Use a tabbed top-level switcher: `Home`, `Servers`, `Contacts`, `Settings`.
- Hubs default to list mode on mobile with optional card toggle.
- Workspace navigation uses slide-in drawers for server/channel trees.

## Deferred (Post-MVP)

- Custom visual themes per workspace.
- Advanced smart folders and AI-suggested server grouping.
- Multi-panel desktop layouts with persistent split views.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/planning/iterations/01-sprint-board.md`
