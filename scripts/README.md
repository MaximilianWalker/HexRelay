# Scripts

Workspace automation scripts.

## Available scripts

- `setup.sh`: installs web dependencies and fetches Rust crates.
- `run.sh`: prints the canonical command set to run infra/services/web.
- `test.sh`: runs Rust + web quality gates.

Use root targets:

- `make setup`
- `make run`
- `make test`
