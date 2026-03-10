# Migration Validation Template

## Document Metadata

- Doc ID: migration-validation-template
- Owner: Platform maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `docs/operations/migration-validation-template.md`

## Quick Context

- Purpose: deterministic validation template for every schema migration pull request.
- Primary edit location: update when migration evidence requirements change.
- Latest meaningful change: 2026-03-10 aligned migration evidence template with mandatory provenance fields.

Use this template for every schema migration PR before merge.

## Migration Metadata

- Migration ID:
- Owner:
- Date (UTC):
- Environment tested:
- Commit SHA:
- PR number (or CI run ID):
- Generated at (UTC):

## Forward Validation

- Command(s) executed:
- Expected outcome:
- Actual outcome:
- Evidence path (logs/artifacts):

## Idempotency and Re-run Check

- Re-run command(s):
- Expected outcome:
- Actual outcome:
- Evidence path:

## Rollback/Recovery Simulation

- Rollback or restore command(s):
- Expected outcome:
- Actual outcome:
- Evidence path:

## Data Integrity Verification

- Constraints/indexes verified:
- Row-count or key invariants checked:
- Evidence path:

## Sign-off

- Reviewer:
- Decision: pass / fail
- Notes:
