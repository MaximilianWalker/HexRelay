This project follows the parent/global AGENTS.md loaded by the agent.
Only project-specific constraints are defined here.

# HexRelay Repo Rules

## 1) Scope

- Primary goal: build an open-source, Discord-like communication platform with strong user data ownership.
- MVP focus: reliable friends, DMs, server channels, and voice before federation complexity.
- Architecture baseline: Rust-first backend services with desktop local-first runtime packaging and reusable web UI layer.
- Development baseline: monorepo scaffolds, local infra compose stack, and CI gates are active and should be maintained.
- Windows and Linux are both mandatory first-class targets for development, testing, packaging, and release planning; never plan Windows-only delivery or leave Linux as a later afterthought.
- Tauri is the default desktop shell choice for release planning unless a later explicit decision replaces it.
- HTTP API and realtime surfaces are internal/local app surfaces and must not use speculative project-owned version names. Do not add path prefixes such as `/v1`, `v1` contract artifact filenames, `V1` schema aliases, event-version envelope fields, cache-key version prefixes, token-format version segments, or storage-key version suffixes unless a real migration/rollout constraint already requires them; update all in-repo consumers and contracts atomically instead of adding compatibility aliases.

## 2) Editing Boundaries

- Keep architecture docs current in the canonical `docs/architecture/*` authorities when major runtime or boundary decisions change; update `docs/product/01-mvp-plan.md` as well when those decisions change product scope or strategy.
- Keep requirements and dependency/risk state current in `docs/product/02-prd.md` and `docs/product/04-dependencies-risks.md` when behavior changes.
- During MVP setup, prefer clean canonical structure over minimal diffs. If a touched area contains obsolete names, old references, dead code, duplicated command paths, misplaced files, or incorrect structure, remove or refactor it in the same change.
- Do not preserve wrappers, aliases, deprecated paths, compatibility layers, or old data shapes unless the user explicitly asks for compatibility or a real external integration requires it.
- When cleanup breaks old local data, paths, fixtures, commands, or docs, update every in-repo caller and reference atomically instead of leaving transitional support.

## 3) Product Guardrails

- Do not introduce paywalled core communication features.
- Preserve portability and export/import capabilities in all storage decisions.
- Treat decentralization as phased delivery to avoid blocking UX quality.
- For MVP-stage protocol and API work, prefer the cleanest single-shape design that can evolve later; avoid speculative versioning, dual-schema migrations, or compatibility layers unless an actual consumer or rollout constraint already makes them necessary.
- DM plaintext and private keys must remain client/device-only; servers/message servers in the server-to-server network may carry and store only end-to-end encrypted DM envelopes plus minimal delivery metadata.
- DM delivery must route through servers/message servers; client devices must not establish recipient-device LAN/WAN DM transport or bootstrap paths, and server discovery work must not reintroduce server-bypassing DM paths.
- Do not introduce server-readable DM content, private-key upload, or unencrypted DM mailbox/relay behavior.
- Do not implement UX changes until the user explicitly approves the proposed flow, copy, controls, and behavior.

## 4) Frontend UI Code Rules

- Keep route files and shell components focused on data loading, layout orchestration, and composition; move reusable UI, interaction logic, and visual variants into named components.
- Prefer one exported component per `.tsx` file. Do not stack multiple substantial components in one file; split them into separate files with colocated CSS modules. Small private helpers are allowed only when they are trivial and not reusable.
- Prefer simple, direct names. Use names such as `ProfileActions`, `NavLink`, `Row`, or `MessageBubble`; avoid broad, branded, clever, or implementation-heavy names unless the domain requires them.
- Do not repeat path context in local component, file, function, or type names. If a component lives in `components/hubs`, use names such as `Toolbar`, `Surface`, or `ItemActions` instead of `HubToolbar`, `HubSurface`, or `HubItemActions`; let imports and folders provide the missing context.
- Components should own one clear responsibility. If a component manages profile state, action buttons, menus, and layout at once, split it before adding more behavior.
- Use existing internal UI primitives first (`Button`, `IconButton`, `Badge`, `Avatar`, `Panel`, `Field`, `Notice`, `Dialog`, `Toolbar`, and related app primitives). Do not create route-local copies of buttons, tabs, cards, switches, fields, menus, or badges.
- Shared control behavior should come from internal UI APIs such as `Button`, `ButtonLink`, `ToggleButton`, `ToggleGroup`, `Menu`, `MenuItem`, and list/action recipes rather than local CSS classes or direct `aria-pressed` buttons.
- Do not pass route-local `className` overrides or locally sized SVG classes into shared UI primitives such as buttons, icon buttons, badges, alerts, menus, toggle controls, or list actions. Add a small, typed primitive prop or a named recipe component instead; use wrapper elements for layout-only spacing when needed.
- Use Tabler icons through shared button/link/icon patterns. Icon-only controls must have `aria-label` and `title`; visible labels should be added only when the layout calls for text.
- CSS Modules should be component-scoped and small enough to audit. If a CSS module grows because unrelated surfaces share it, split by component or feature instead of adding more classes.
- Avoid CSS specificity fights, duplicated classes, and state encoded through long selector chains. Prefer explicit component props mapped to a small set of classes or data attributes.
- Component CSS must use semantic tokens from `apps/web/app/styles/*`. Do not add raw hex/rgb colors, one-off shadows, one-off radii, or arbitrary spacing unless the token set is missing a real reusable value.
- Do not fix broken layout with visual nudges, arbitrary offsets, compensating margins, or breakpoint-only tweaks. First identify the incorrect parent layout, alignment model, content hierarchy, or component boundary; then implement the clean structural fix using grid/flex alignment, intrinsic sizing, and tokens.
- Keep styling restrained and native to the app. Reference images guide layout and interaction ideas; do not copy captions, decorative panels, gradients, or visual noise unless those elements are intentionally adopted into the design system.
- Collapse and responsive states should hide optional text or reflow layout without changing core target sizes, margins, or icon alignment unexpectedly.
- Repeated UI behavior must live in a shared component before the third copy. This includes tab scrolling, popovers, setting rows, action docks, search fields, empty states, and selection controls.
- Prefer deterministic UI state. Avoid effect-driven state mirroring when a value can be derived from props, preferences, route state, or stores.
- For meaningful frontend changes, validate in the browser with screenshots for the affected states and run the relevant gates: `npm --prefix apps/web run lint`, `npm --prefix apps/web run lint:styles`, tests, and build when shared code is touched.

## 5) Readiness Feedback Loop (Required)

- When a readiness finding is fixed, record it in `docs/operations/readiness-corrections-log.md` in the same change.
- For repeated findings, add or tighten a durable rule in `AGENTS.md` or the canonical owning document in the same change.
- Before opening a new readiness audit cycle, check `docs/operations/readiness-corrections-log.md` and treat open findings as first-pass candidates.
- Do not re-open previously closed findings unless new code/docs changes invalidate the prior fix; if invalidated, record the regression explicitly in the log.

## 6) Standard Readiness Execution Flow (Repeatable)

- When asked to check readiness before continuing development, run two parallel subagent audits:
  - docs audit (`documentation-governor`)
  - API/realtime code audit (`code-reviewer`)
- After audits, always run a strict revalidation pass:
  1. create a todo list with each reported issue,
  2. classify each issue as `valid` vs `nitpick` with repository evidence,
  3. fix only valid, high-signal items,
  4. append each correction/reason to `docs/operations/readiness-corrections-log.md`.
- If an issue is real but not safe for a minimal pass (architectural/high-risk), do not force a partial fix; log it as `watch` in the readiness corrections log with explicit deferral reason.
- For docs-only readiness passes, skip unnecessary code test reruns; for code-touching passes, run formatter/tests/clippy for touched services before commit.

## 7) Protected Branch Delivery Flow (Repeatable)

- Default sequence for delivering changes:
  1. commit on current branch,
  2. attempt push,
  3. if `master` is blocked by protection, create `chore/*` branch and push,
  4. open PR,
  5. watch required checks to completion,
  6. resolve unresolved review threads,
  7. merge PR,
  8. sync local `master`,
  9. delete local and remote feature branch.
- Do not skip the check-watch step; merge only after required checks are green and conversation-resolution requirements are satisfied.
- If `gh pr merge` reports that base-branch policy prohibits the merge even after required checks are green, inspect PR mergeability and active branch-protection requirements before retrying.
- Treat unresolved review threads as a first-class merge blocker, even when they are outdated bot threads; resolve them explicitly when their feedback has already been addressed.
- If GitHub reports `mergeable` but `mergeStateStatus` is still blocked, refresh PR state after resolving threads before retrying the merge.
- After merge, verify the PR is merged, confirm local `master` matches `origin/master`, and confirm branch cleanup rather than assuming `--delete-branch` completed everything.
