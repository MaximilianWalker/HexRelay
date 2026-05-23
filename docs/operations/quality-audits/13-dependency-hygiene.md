# Dependency Hygiene Quality Audit

## Metadata

- topic_id: 13-dependency-hygiene
- topic: Dependency Hygiene
- last_audited: 2026-05-14T00:43:14Z
- source_of_truth: `docs/operations/quality-audits/13-dependency-hygiene.md`

## Investigation Focus

- Inspect dependency scope, maintenance risk, lockfile discipline, audit exceptions, and upgrade paths.
- Flag unnecessary packages, stale critical dependencies, or security gates that can silently degrade.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-13-20260514-dependency-upgrade-cadence-missing | P3 | found | The repo has vulnerability gates and lockfiles but no dependency upgrade cadence or automation. | Dependency surfaces are active in `Cargo.toml`, `services/api-rs/Cargo.toml`, `services/realtime-rs/Cargo.toml`, `crates/communication-core/Cargo.toml`, and `apps/web/package.json`; lockfiles exist at `Cargo.lock` and `apps/web/package-lock.json`; `rg --files -g '.github/dependabot.yml' -g '.github/dependabot.yaml' -g 'renovate.json' -g '.renovaterc'` returned no dependency-update configuration, and `rg -n "dependabot|renovate|npm outdated|cargo outdated|cargo update|dependency update|dependency upgrade|upgrade cadence|outdated" .github docs scripts package.json Cargo.toml apps/web/package.json` found no maintained upgrade workflow beyond historical readiness-log mentions. | Add Dependabot/Renovate or document an explicit Cargo/npm/GitHub Actions review cadence with owner, frequency, and validation commands. | 2026-05-14T00:43:14Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-13-20260514-cargo-audit-ignore-parity-drift | P2 | resolved | Cargo audit advisory ignores were split across CI, local security script, and operator docs. | `scripts/security/advisories.mjs` is now the only advisory policy source; `npm run security`, `node scripts/validators/cargo-audit-ignore.mjs`, CI, and contributor docs all reference the canonical Node command path instead of copied ignore lists. | 2026-05-20T17:30:00Z |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T00:43:14Z | Codex | Added 1 P2 found finding about cargo-audit ignore parity drift and 1 P3 found finding about missing dependency upgrade cadence. |
| 2026-05-20T17:30:00Z | Codex | Resolved cargo-audit ignore parity drift by centralizing advisory policy in `scripts/security/advisories.mjs`. |
