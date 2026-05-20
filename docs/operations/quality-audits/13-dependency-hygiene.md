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
| QA-13-20260514-cargo-audit-ignore-parity-drift | P2 | found | Cargo audit advisory ignores are split across CI, local security script, and operator docs. | `.github/workflows/ci.yml:43` runs `cargo audit --deny warnings` with ignores for `RUSTSEC-2023-0071`, `RUSTSEC-2026-0049`, and `RUSTSEC-2026-0097`; `scripts/validators/cargo-audit-ignore.sh:5-7` tracks expiry for the same three advisories; `package.json:37` exposes `npm run security` with only `RUSTSEC-2023-0071`; `docs/operations/01-mvp-runbook.md:201` lists only `RUSTSEC-2023-0071` and `RUSTSEC-2026-0049`; `docs/operations/contributor-guide.md:71-73` documents only those first two advisories in the ignore-expiry policy. | Centralize the cargo-audit command or generate all local/docs/CI audit ignore lists from the validator source so temporary exceptions cannot drift by entry point. | 2026-05-14T00:43:14Z |
| QA-13-20260514-dependency-upgrade-cadence-missing | P3 | found | The repo has vulnerability gates and lockfiles but no dependency upgrade cadence or automation. | Dependency surfaces are active in `Cargo.toml`, `services/api-rs/Cargo.toml`, `services/realtime-rs/Cargo.toml`, `crates/communication-core/Cargo.toml`, and `apps/web/package.json`; lockfiles exist at `Cargo.lock` and `apps/web/package-lock.json`; `rg --files -g '.github/dependabot.yml' -g '.github/dependabot.yaml' -g 'renovate.json' -g '.renovaterc'` returned no dependency-update configuration, and `rg -n "dependabot|renovate|npm outdated|cargo outdated|cargo update|dependency update|dependency upgrade|upgrade cadence|outdated" .github docs scripts package.json Cargo.toml apps/web/package.json` found no maintained upgrade workflow beyond historical readiness-log mentions. | Add Dependabot/Renovate or document an explicit Cargo/npm/GitHub Actions review cadence with owner, frequency, and validation commands. | 2026-05-14T00:43:14Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T00:43:14Z | Codex | Added 1 P2 found finding about cargo-audit ignore parity drift and 1 P3 found finding about missing dependency upgrade cadence. |
