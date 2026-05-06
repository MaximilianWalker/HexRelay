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
- `scripts/seed.sh`
- `scripts/reset-dev-db.sh`
- `scripts/run.sh`
- `scripts/status.sh`
- `scripts/stop.sh`
- `scripts/network.sh`
- `scripts/test.sh`

Local runtime testing fixture and seed details live in
`docs/planning/local-runtime-testing-plan.md`.

Runtime profile files live in `scripts/runtime-profiles/` and are validated with
`npm run validate:runtime-profiles`.

Network simulation profile files live in `scripts/network-profiles/` and are
validated with `npm run validate:network-profiles`.
Apply or reset network simulation state with `npm run network -- --profile <profile>`
or `npm run network -- --reset`.
