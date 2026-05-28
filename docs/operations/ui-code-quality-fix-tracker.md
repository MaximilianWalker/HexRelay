# UI Code Quality Fix Tracker

Temporary tracker for frontend cleanup items found during the UI rules audit. Delete this file once the items are either fixed or moved into a durable planning document.

## Current Status

- Branch: `codex/ui-framework-refactor`
- Scope: `apps/web`
- Goal: remove route-local UI, split oversized components, reduce CSS sprawl, and enforce token-backed styling.

## Pending Fixes

### 1. Split Workspace Profile Controls

- Status: completed
- Files:
  - `apps/web/components/workspace-profile-controls.tsx`
  - `apps/web/components/workspace-profile-controls.module.css`
  - `apps/web/components/workspace-profile-actions.tsx`
  - `apps/web/components/workspace-profile-actions.module.css`
  - `apps/web/components/workspace-profile-action-button.tsx`
  - `apps/web/components/workspace-profile-action-button.module.css`
  - `apps/web/components/workspace-profile-card.tsx`
  - `apps/web/components/workspace-profile-card.module.css`
  - `apps/web/components/workspace-profile-menu.tsx`
  - `apps/web/components/workspace-profile-menu.module.css`
  - `apps/web/components/workspace-profile-types.ts`
- Problem: `WorkspaceProfileControls`, `ActionButton`, `ProfileCard`, and `ProfileMenu` are substantial components in one file.
- Target structure:
  - `workspace-profile-controls.tsx`
  - `workspace-profile-card.tsx`
  - `workspace-profile-actions.tsx`
  - `workspace-profile-menu.tsx`
  - separate or clearly scoped CSS modules if the current module remains too large.
- Notes: preserve current behavior: icon-only actions, muted mic/sound red state, compact popup, and segmented Sidebar/Topbar control.

### 2. Split Workspace Shell Tabs And Menus

- Status: completed
- Files:
  - `apps/web/components/workspace-shell.tsx`
  - `apps/web/components/workspace-shell.module.css`
  - `apps/web/components/content-tab-bar.tsx`
  - `apps/web/components/content-tab-bar.module.css`
  - `apps/web/components/workspace-tabs.tsx`
  - `apps/web/components/workspace-tabs.module.css`
  - `apps/web/components/workspace-tab-list.tsx`
  - `apps/web/components/workspace-tab-item.tsx`
  - `apps/web/components/workspace-tab-item.module.css`
  - `apps/web/components/workspace-tab-types.ts`
  - `apps/web/components/workspace-context-menu.tsx`
  - `apps/web/components/workspace-context-menu.module.css`
- Problem: `WorkspaceShell` still owns workspace tab rendering, workspace context menu, scroll controls, and content tabs.
- Target components:
  - `WorkspaceTabs`
  - `WorkspaceTab`
  - `WorkspaceContextMenu`
  - `ContentTabBar`
- Notes: shell should keep layout orchestration and preference wiring only.

### 3. Extract Server Workspace UI

- Status: completed
- Files:
  - `apps/web/app/servers/[serverId]/page.tsx`
  - `apps/web/components/server-workspace/server-chat-view.tsx`
  - `apps/web/components/server-workspace/server-channel-button.tsx`
  - `apps/web/components/server-workspace/server-icon.tsx`
  - `apps/web/components/server-workspace/server-member-card.tsx`
  - `apps/web/components/server-workspace/server-overview.tsx`
  - `apps/web/components/server-workspace/server-settings-view.tsx`
  - `apps/web/components/server-workspace/server-users-view.tsx`
  - `apps/web/components/server-workspace/server-voice-channel-button.tsx`
  - `apps/web/components/server-workspace/server-voice-participant-row.tsx`
  - `apps/web/components/server-workspace/server-voice-view.tsx`
  - `apps/web/components/server-workspace/server-workspace-types.ts`
- Problem: route file owns `ServerIcon`, `ChannelButton`, `MemberCard`, `VoiceChannelButton`, `VoiceParticipantRow`, chat panels, member panels, and voice panels.
- Target components:
  - `ServerChannelRail`
  - `ServerMemberPanel`
  - `ServerVoicePanel`
  - `ServerChatHeader`
  - shared server workspace controls where useful.
- Notes: preserve existing data/API behavior while moving presentation out of the route file.

### 4. Finish Hubs Cleanup

- Status: completed
- Files:
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/surfaces.module.css`
  - `apps/web/components/hubs/contact-add-dialog.tsx`
  - `apps/web/components/hubs/contact-block-dialog.tsx`
  - `apps/web/components/hubs/contact-discovery-results.tsx`
  - `apps/web/components/hubs/contact-request-section.tsx`
  - `apps/web/components/hubs/hub-item-actions.tsx`
  - `apps/web/components/hubs/server-create-dialog.tsx`
  - `apps/web/components/hubs/server-join-dialog.tsx`
  - `apps/web/components/hubs/server-leave-dialog.tsx`
  - `apps/web/components/hubs/hubs.module.css`
- Problem: route-local pills, buttons, cards, inputs, dialogs, and bulk-action controls still bypass shared primitives and keep `surfaces.module.css` oversized.
- Target components:
  - move remaining route-owned controls into `components/hubs`
  - use shared `Button`, `IconButton`, `Field`, `Panel`, `Dialog`, `Badge`, and hub primitives.
- Notes: route-owned hub dialogs, request cards, discovery cards, and item actions now live in `components/hubs`; `surfaces.module.css` keeps only residual route state/conversation styles.

### 5. Refactor Settings Controls

- Status: completed
- Files:
  - `apps/web/app/settings/page.tsx`
  - `apps/web/components/settings/*`
  - `apps/web/app/settings/settings.module.css`
- Problem: settings page still has many raw `select` and `button` controls instead of field/settings primitives.
- Target components:
  - `SettingSelect`
  - `SettingButton`
  - split `ToggleControl` and `ReadOnlyValue` out of `setting-row.tsx` if they remain reusable.
- Notes: settings selects, buttons, toggles, and read-only values now use dedicated settings wrappers backed by shared UI primitives; live preference behavior is preserved.

### 6. Split UI Primitive Files

- Status: completed
- Files:
  - `apps/web/components/ui/field.tsx`
  - `apps/web/components/ui/dialog.tsx`
  - `apps/web/components/ui/ui.module.css`
- Problem: multiple exported primitives and a shared primitive stylesheet make ownership unclear.
- Target structure:
  - one exported primitive per file where practical.
  - split CSS by primitive if `ui.module.css` remains hard to audit.
- Notes: field, input, textarea, select, checkbox, toggle, dialog, and dialog actions now have one exported primitive per file; callers were updated atomically while the shared token-backed CSS remains in `ui.module.css`.

### 7. Clean Home And Onboarding Local UI

- Status: pending
- Files:
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/home/home.module.css`
  - `apps/web/app/onboarding/*`
- Problem: route-local buttons, inputs, tabs, badges, and one-off spacing/radius values remain.
- Target: use shared `Button`, `Field`, `Panel`, `Badge`, `SegmentedControl`, and onboarding components.

## Validation Required Per Cleanup Chunk

- `npm --prefix apps/web run lint`
- `npm --prefix apps/web run lint:styles`
- `npm --prefix apps/web run test`
- `npm --prefix apps/web run build` for shared component or route behavior changes.
- Browser screenshots for affected desktop and compact/collapsed states.
