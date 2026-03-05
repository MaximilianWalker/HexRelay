# Evidence Artifacts

This directory stores machine-generated and manually curated validation artifacts.

## CI Evidence Contract

- CI integration-smoke uploads a run-scoped artifact folder at:
  - `evidence/ci/<run_id>/`
- Minimum expected files:
  - `manifest.txt`
  - `summary.json`
  - `api.log`
  - `realtime.log`
  - `smoke-e2e.log`
  - `health-checks.log`
  - `web-coverage-summary.json`

If a file is absent for a run, `manifest.txt` must explicitly mark it as `missing`.
