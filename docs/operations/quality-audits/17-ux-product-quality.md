# UX / Product Quality Audit

## Metadata

- topic_id: 17-ux-product-quality
- topic: UX / Product Quality
- last_audited: 2026-05-18T08:59:40Z
- source_of_truth: `docs/operations/quality-audits/17-ux-product-quality.md`

## Investigation Focus

- Inspect user flows, screen states, empty/loading/error states, product requirements, and UI implementation alignment.
- Record UX issues only as findings; do not implement UX changes without explicit approval.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-17-20260514-servers-hub-state-actions-incomplete | P2 | confirmed | Servers Hub lacks user-actionable empty/error/search states and required card actions. | `docs/product/07-ui-navigation-spec.md:73-76` requires a searchable server card grid with favorites/unread/muted filters plus card actions for open server, open settings, and pin/unpin; `docs/product/07-ui-navigation-spec.md:90-93` requires hub searches to be focusable, empty states to include a next action, deep links, and persisted hub preferences; `docs/product/08-screen-state-spec.md:44` requires loading, empty, search_no_results, permission_denied, and error states. `apps/web/app/servers/page.tsx:162-178` renders cards with only a server link and metadata, then prints `state: {state}` for every non-card condition instead of actionable state UI. | Propose the Servers Hub state and card-action UX, then replace the raw state label with loading/empty/search/error/permission surfaces and add settings/pin actions plus persisted filter behavior. | 2026-05-14T12:49:36Z |
| QA-17-20260514-contact-invite-preview-skipped | P2 | confirmed | Contact invite redemption skips the documented inviter preview and explicit recipient acceptance step. | `docs/product/02-prd.md:146-150` defines the contact-invite flow as generate link, redeem token, show inviter preview, then recipient accepts before a friend request or accepted edge is created. `apps/web/app/contacts/page.tsx:425-456` calls `redeemContactInvite` immediately from the pasted token and sets "Contact request sent"; `apps/web/app/contacts/page.tsx:527-559` labels the first action "Continue" and then shows "Request sent" rather than an inviter preview with an accept/decline decision. | Align product and API behavior: either split token preview from acceptance, or explicitly approve a one-step redeem flow and update the canonical PRD/screen-state copy before implementing UI changes. | 2026-05-14T12:49:36Z |
| QA-17-20260514-server-workspace-preview-masks-states | P2 | watch | Server workspace preview data can mask missing-session or load-error states with seeded Atlas content, but runtime UX changes are blocked until an explicit approved Server Workspace flow/copy/control spec exists. | Rechecked 2026-05-18: `apps/web/app/servers/[serverId]/page.tsx` still derives normal no-session routes from `PREVIEW_SERVER`, `PREVIEW_CHANNELS`, and `PREVIEW_MESSAGES`, still titles the shell from `visibleServer`, and still tells users it is showing seeded Atlas preview data. `docs/product/08-screen-state-spec.md:30-45` names the required Server Workspace states but also keeps UX flow, copy, controls, and behavior changes behind explicit consent; repo search found no approved Server Workspace preview/demo approval package to cite. | Keep runtime implementation blocked until a Server Workspace preview/demo and state-handling approval package exists; after approval, gate seeded fixtures behind the approved mode and render the required normal route states. | 2026-05-18T08:59:40Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-17-20260514-dm-workspace-send-not-wired | P1 | superseded | DM workspace exposes a composer and send button, but accepted contacts still cannot send or load E2EE DM history. | Merged plan-only PR #165 converted the runtime implementation request into an approval-pending `DMW-APP-*` flow package in `docs/planning/dm-workspace-delivery-implementation-plan.md`; runtime work remains blocked until explicit UX approval exists. | 2026-05-15T05:04:39Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T08:59:40Z | Codex automation | Reclassified `QA-17-20260514-server-workspace-preview-masks-states` as `watch`: the cited fallback fixture behavior is still present, but the only current Server Workspace authority lists required states and the global UX approval gate; no approved flow/copy/control package exists for the runtime fix. |
| 2026-05-15T05:04:39Z | Codex automation | Reclassified `QA-17-20260514-dm-workspace-send-not-wired` as `superseded` by merged plan-only PR #165 so autonomous development does not repeatedly select unapproved DM workspace UX implementation. |
| 2026-05-14T12:49:36Z | Codex | Added 1 P1 and 3 P2 confirmed findings about DM workspace delivery, Servers Hub states/actions, contact-invite preview, and server workspace preview fallbacks. |
