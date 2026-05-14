# Build and Release Quality Audit

## Metadata

- topic_id: 16-build-and-release-quality
- topic: Build and Release Quality
- last_audited: 2026-05-14T09:48:12Z
- source_of_truth: `docs/operations/quality-audits/16-build-and-release-quality.md`

## Investigation Focus

- Review reproducible builds, CI gates, packaging, release evidence, rollback paths, and branch protection alignment.
- Flag mismatches between documented release expectations and implemented automation.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-16-20260514-release-packaging-automation-missing | P2 | found | Release artifact model has no packaging, signing, or install-smoke automation yet. | `docs/operations/03-release-packaging.md:20` says release automation, signing infrastructure, and install smoke tests are not implemented; `docs/operations/03-release-packaging.md:48-57` defines Windows/Linux desktop, portable, dedicated-server, container, checksum, and signature artifacts; `docs/operations/03-release-packaging.md:96-105` requires build/launch validation, checksums, and signed-or-blocked public artifacts. `rg --files --hidden` with Tauri/release/packaging/installer/signing patterns returned no Tauri config, release workflow, installer script, or signing manifest outside docs, CI, package metadata, and signature-domain tests. | Add staged release packaging workflows or scripts for Windows/Linux desktop and dedicated-server artifacts, with checksum/signing gates or an explicit unsigned-block decision before release candidates. | 2026-05-14T09:48:12Z |
| QA-16-20260514-runtime-network-smoke-not-required | P2 | found | Docker runtime/network smoke is not aligned with the documented required release gates. | `.github/workflows/ci.yml:216-235` defines `runtime-network-smoke` and runs `npm run test:runtime`; `.github/workflows/ci.yml:360-363` makes `integration-smoke` depend on other gates but not `runtime-network-smoke`; `docs/operations/contributor-guide.md:79` lists required jobs without `runtime-network-smoke`; `README.md:70` and `docs/testing/01-mvp-verification-matrix.md:28` describe `npm run test:runtime` as validating offline, partition, Toxiproxy, app-fault, reset, and local runtime adoption evidence. | Decide whether `runtime-network-smoke` is required for protected merges and align branch-protection docs, CI dependency structure, and contributor guidance; if optional, document the release-candidate trigger and failure policy. | 2026-05-14T09:48:12Z |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-14T09:48:12Z | Codex | Added 2 P2 found findings about missing release packaging automation and runtime-network smoke required-gate drift. |
