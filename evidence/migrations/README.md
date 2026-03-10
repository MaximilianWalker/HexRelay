# Migration Evidence Artifacts

Store one evidence file per migration changed in a pull request.

- Path format: `evidence/migrations/<migration-name>.md`
- Example: migration `services/api-rs/migrations/0010_add_index.sql` requires `evidence/migrations/0010_add_index.md`
- Start from: `docs/operations/migration-validation-template.md`

## Starter Baseline

- Historical starter artifacts for existing migrations are included as `0001_*` through `0009_*`.
- Update the corresponding file in the same pull request whenever that migration is amended or superseded.

CI enforces this for migration-changing pull requests via `migration-evidence-check`.
