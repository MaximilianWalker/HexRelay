# Scripts

Workspace automation scripts.

## Available scripts

- `setup.sh`: installs web dependencies and fetches Rust crates.
- `run.sh`: starts infra, API, realtime, and web dev servers.
- `test.sh`: runs Rust + web quality gates.

Use root targets:

- `make setup`
- `make run`
- `make test`

Or npm scripts:

- `npm run setup`
- `npm run run`
- `npm run test`
