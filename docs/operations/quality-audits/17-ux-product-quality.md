# UX / Product Quality Audit

## Metadata

- topic_id: 17-ux-product-quality
- topic: UX / Product Quality
- last_audited: 2026-05-14T12:49:36Z
- source_of_truth: `docs/operations/quality-audits/17-ux-product-quality.md`

## Investigation Focus

- Inspect user flows, screen states, empty/loading/error states, product requirements, and UI implementation alignment.
- Record UX issues only as findings; do not implement UX changes without explicit approval.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-17-20260514-servers-hub-state-actions-incomplete | P2 | confirmed | Servers Hub lacks user-actionable empty/error/search states and required card actions. | `docs/product/07-ui-navigation-spec.md:73-76` requires a searchable server card grid with favorites/unread/muted filters plus card actions for open server, open settings, and pin/unpin; `docs/product/07-ui-navigation-spec.md:90-93` requires hub searches to be focusable, empty states to include a next action, deep links, and persisted hub preferences; `docs/product/08-screen-state-spec.md:44` requires loading, empty, search_no_results, permission_denied, and error states. `apps/web/app/servers/page.tsx:162-178` renders cards with only a server link and metadata, then prints `state: {state}` for every non-card condition instead of actionable state UI. | Propose the Servers Hub state and card-action UX, then replace the raw state label with loading/empty/search/error/permission surfaces and add settings/pin actions plus persisted filter behavior. | 2026-05-14T12:49:36Z |
| QA-17-20260514-contact-invite-preview-skipped | P2 | confirmed | Contact invite redemption skips the documented inviter preview and explicit recipient acceptance step. | `docs/product/02-prd.md:146-150` defines the contact-invite flow as generate link, redeem token, show inviter preview, then recipient accepts before a friend request or accepted edge is created. `apps/web/app/contacts/page.tsx:425-456` calls `redeemContactInvite` immediately from the pasted token and sets "Contact request sent"; `apps/web/app/contacts/page.tsx:527-559` labels the first action "Continue" and then shows "Request sent" rather than an inviter preview with an accept/decline decision. | Align product and API behavior: either split token preview from acceptance, or explicitly approve a one-step redeem flow and update the canonical PRD/screen-state copy before implementing UI changes. | 2026-05-14T12:49:36Z |
| QA-17-20260514-server-workspace-preview-masks-states | P2 | confirmed | Server workspace preview data can mask missing-session or load-error states with seeded Atlas content. | `docs/product/08-screen-state-spec.md:45` requires Server Workspace loading, channel_empty, permission_denied, reconnecting, and error states. `apps/web/app/servers/[serverId]/page.tsx:686-696` falls back to `PREVIEW_SERVER`, `PREVIEW_CHANNELS`, and `PREVIEW_MESSAGES`; `apps/web/app/servers/[serverId]/page.tsx:884-897` still titles the workspace from `visibleServer` while only adding loading/error text; `apps/web/app/servers/[serverId]/page.tsx:954-956` tells the user it is showing seeded Atlas preview data when no local testing profile is active. | Gate seeded fixtures behind an explicit preview/demo mode, and use the required permission/error/empty states for normal server routes so users do not confuse fixture data with live workspace state. | 2026-05-14T12:49:36Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-17-20260514-dm-workspace-send-not-wired | P1 | superseded | DM workspace exposes a composer and send button, but accepted contacts still cannot send or load E2EE DM history. | Superseded by the approval-pending plan in `docs/planning/dm-workspace-delivery-implementation-plan.md`; runtime implementation remains blocked until explicit `DMW-APP-*` approval exists. | 2026-05-15 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T12:49:36Z | Codex | Added 1 P1 and 3 P2 confirmed findings about DM workspace delivery, Servers Hub states/actions, contact-invite preview, and server workspace preview fallbacks. |
| 2026-05-15T02:04:48Z | Codex | Superseded the P1 DM workspace send/history finding with an approval-pending implementation plan; no runtime UX behavior was approved or changed. |
