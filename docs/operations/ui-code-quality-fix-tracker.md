# UI Code Quality Fix Tracker

Temporary tracker for frontend cleanup items found during the UI rules audit. Delete this file once the items are either fixed or moved into a durable planning document.

## Current Status

- Branch: `codex/ui-framework-refactor`
- Scope: `apps/web`
- Goal: remove route-local UI, split oversized components, reduce CSS sprawl, and enforce token-backed styling.

## Pending Fixes

### 1. Split Workspace Profile Controls

- Status: pending
- Files:
  - `apps/web/components/workspace-profile-controls.tsx`
  - `apps/web/components/workspace-profile-controls.module.css`
- Problem: `WorkspaceProfileControls`, `ActionButton`, `ProfileCard`, and `ProfileMenu` are substantial components in one file.
- Target structure:
  - `workspace-profile-controls.tsx`
  - `workspace-profile-card.tsx`
  - `workspace-profile-actions.tsx`
  - `workspace-profile-menu.tsx`
  - separate or clearly scoped CSS modules if the current module remains too large.
- Notes: preserve current behavior: icon-only actions, muted mic/sound red state, compact popup, and segmented Sidebar/Topbar control.

### 2. Split Workspace Shell Tabs And Menus

- Status: pending
- Files:
  - `apps/web/components/workspace-shell.tsx`
  - `apps/web/components/workspace-shell.module.css`
- Problem: `WorkspaceShell` still owns workspace tab rendering, workspace context menu, scroll controls, and content tabs.
- Target components:
  - `WorkspaceTabs`
  - `WorkspaceTab`
  - `WorkspaceContextMenu`
  - `ContentTabBar`
- Notes: shell should keep layout orchestration and preference wiring only.

### 3. Extract Server Workspace UI

- Status: pending
- File: `apps/web/app/servers/[serverId]/page.tsx`
- Problem: route file owns `ServerIcon`, `ChannelButton`, `MemberCard`, `VoiceChannelButton`, `VoiceParticipantRow`, chat panels, member panels, and voice panels.
- Target components:
  - `ServerChannelRail`
  - `ServerMemberPanel`
  - `ServerVoicePanel`
  - `ServerChatHeader`
  - shared server workspace controls where useful.
- Notes: preserve existing data/API behavior while moving presentation out of the route file.

### 4. Finish Hubs Cleanup

- Status: pending
- Files:
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/surfaces.module.css`
- Problem: route-local pills, buttons, cards, inputs, dialogs, and bulk-action controls still bypass shared primitives and keep `surfaces.module.css` oversized.
- Target components:
  - move remaining route-owned controls into `components/hubs`
  - use shared `Button`, `IconButton`, `Field`, `Panel`, `Dialog`, `Badge`, and hub primitives.
- Notes: `surfaces.module.css` should shrink substantially or be deleted when replaced.

### 5. Refactor Settings Controls

- Status: pending
- Files:
  - `apps/web/app/settings/page.tsx`
  - `apps/web/components/settings/*`
  - `apps/web/app/settings/settings.module.css`
- Problem: settings page still has many raw `select` and `button` controls instead of field/settings primitives.
- Target components:
  - `SettingSelect`
  - `SettingButton`
  - split `ToggleControl` and `ReadOnlyValue` out of `setting-row.tsx` if they remain reusable.
- Notes: keep existing live preference behavior intact.

### 6. Split UI Primitive Files

- Status: pending
- Files:
  - `apps/web/components/ui/field.tsx`
  - `apps/web/components/ui/dialog.tsx`
  - `apps/web/components/ui/ui.module.css`
- Problem: multiple exported primitives and a shared primitive stylesheet make ownership unclear.
- Target structure:
  - one exported primitive per file where practical.
  - split CSS by primitive if `ui.module.css` remains hard to audit.
- Notes: avoid breaking imports by updating all in-repo callers atomically.

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
