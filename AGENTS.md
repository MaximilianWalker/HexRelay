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
- Prefer minimal diffs and avoid broad refactors during MVP setup.

## 3) Product Guardrails

- Do not introduce paywalled core communication features.
- Preserve portability and export/import capabilities in all storage decisions.
- Treat decentralization as phased delivery to avoid blocking UX quality.
- For MVP-stage protocol and API work, prefer the cleanest single-shape design that can evolve later; avoid speculative versioning, dual-schema migrations, or compatibility layers unless an actual consumer or rollout constraint already makes them necessary.
- DM plaintext and private keys must remain client/device-only; servers/message servers in the server-to-server network may carry and store only end-to-end encrypted DM envelopes plus minimal delivery metadata.
- DM delivery must route through servers/message servers; client devices must not establish recipient-device LAN/WAN DM transport or bootstrap paths, and server discovery work must not reintroduce server-bypassing DM paths.
- Do not introduce server-readable DM content, private-key upload, or unencrypted DM mailbox/relay behavior.
- Do not implement UX changes until the user explicitly approves the proposed flow, copy, controls, and behavior.

## 4) Readiness Feedback Loop (Required)

- When a readiness finding is fixed, record it in `docs/operations/readiness-corrections-log.md` in the same change.
- For repeated findings, add or tighten a durable rule in `AGENTS.md` or the canonical owning document in the same change.
- Before opening a new readiness audit cycle, check `docs/operations/readiness-corrections-log.md` and treat open findings as first-pass candidates.
- Do not re-open previously closed findings unless new code/docs changes invalidate the prior fix; if invalidated, record the regression explicitly in the log.

## 5) Standard Readiness Execution Flow (Repeatable)

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

## 6) Protected Branch Delivery Flow (Repeatable)

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
