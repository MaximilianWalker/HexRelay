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

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-18-20260514-server-menu-aria-without-keyboard-model | P2 | fixed | Server actions exposed ARIA menu roles without the keyboard and focus behavior those roles require. | `apps/web/app/servers/[serverId]/page.tsx` now exposes server actions as an ordinary disclosure panel with `aria-controls="server-actions-panel"`, removes `role="menu"`/`role="menuitem"`, and returns focus to the trigger with `serverMenuButtonRef.current?.focus()`; `apps/web/lib/server-actions-accessibility.test.ts` guards the disclosure semantics and focus-return contract. | 2026-05-18T12:05:27Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T12:05:27Z | Codex issue remediator | Fixed `QA-18-20260514-server-menu-aria-without-keyboard-model` by replacing the ARIA menu contract with ordinary disclosure semantics, focus return, and durable web test coverage. |
| 2026-05-14T15:50:43Z | Codex | Added 2 P2 confirmed findings for missing accessible names on core text-entry controls and server menu ARIA semantics without matching keyboard/focus behavior. |
