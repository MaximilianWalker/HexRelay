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

## Migration Evidence Contract

- Pull requests that change `services/api-rs/migrations/*.sql` must include matching evidence files at:
  - `evidence/migrations/<migration-name>.md`
- Use `docs/operations/migration-validation-template.md` as the required template baseline.

## Iteration Evidence Contract

- For evidence folders under `evidence/iteration-*/` and `evidence/operations/`, include at minimum:
  - `summary.md`
  - `validators.txt`
  - `outputs/` directory with referenced raw artifacts
- If an expected artifact is missing, record `missing` and rationale in `summary.md`.
