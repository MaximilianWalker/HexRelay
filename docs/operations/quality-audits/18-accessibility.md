# Accessibility Quality Audit

## Metadata

- topic_id: 18-accessibility
- topic: Accessibility
- last_audited: 2026-05-14T15:50:43Z
- source_of_truth: `docs/operations/quality-audits/18-accessibility.md`

## Investigation Focus

- Review keyboard access, semantics, focus behavior, contrast, responsive layout, and pointer-only interactions.
- Prioritize accessibility gaps in core communication workflows.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-18-20260514-core-inputs-missing-accessible-names | P2 | confirmed | Core web text-entry controls rely on placeholders or nearby visual text instead of durable accessible names. | `rg -n '<input\|<textarea\|placeholder=' apps/web/app/home/page.tsx apps/web/app/servers/page.tsx apps/web/app/contacts/page.tsx` shows unlabeled persona, server search, contact invite, invite-use-count, and contact search inputs at `apps/web/app/home/page.tsx:135`, `apps/web/app/servers/page.tsx:151`, and `apps/web/app/contacts/page.tsx:532,588,753`; DM and server composers likewise expose only placeholder text at `apps/web/app/contacts/[contactId]/messages/page.tsx:231,236` and `apps/web/app/servers/[serverId]/page.tsx:1151,1155`. By contrast, onboarding identity/recovery fields use explicit labels, so this is inconsistent on core communication workflows. | Add visible labels or `aria-label`/`aria-labelledby` plus helpful `aria-describedby` text for search, invite, persona, and composer controls; add a render/lint guard so future text-entry controls cannot ship unnamed. | 2026-05-14T15:50:43Z |
| QA-18-20260514-server-menu-aria-without-keyboard-model | P2 | confirmed | Server actions expose ARIA menu roles without the keyboard and focus behavior those roles require. | `apps/web/app/servers/[serverId]/page.tsx:922` toggles the action menu with only `aria-expanded`; `apps/web/app/servers/[serverId]/page.tsx:932-942` renders `role="menu"`/`role="menuitem"` controls, but the server page has no `onKeyDown` handler for Escape or arrow navigation and does not move focus into or back out of the menu. This creates a screen-reader contract for a menu while behaving like a generic disclosure. | Either implement the full menu button keyboard/focus pattern or remove menu roles and expose the actions as ordinary disclosure content with predictable tab order and focus return. | 2026-05-14T15:50:43Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T15:50:43Z | Codex | Added 2 P2 confirmed findings for missing accessible names on core text-entry controls and server menu ARIA semantics without matching keyboard/focus behavior. |
