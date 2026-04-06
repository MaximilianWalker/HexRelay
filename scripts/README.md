# Scripts

Workspace automation entrypoints live here, but the canonical workflow and gate
documentation now lives in `docs/operations/contributor-guide.md`.

Use this directory as implementation detail; use the contributor guide as the
source of truth for:

- local validation commands
- PR gate expectations
- smoke/bootstrap prerequisites
- delivery and release workflow

Common entrypoints still include:

- `scripts/setup.sh`
- `scripts/run.sh`
- `scripts/test.sh`
