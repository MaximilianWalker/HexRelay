# Local Infrastructure Stack

This directory defines the local development infrastructure for HexRelay blocker D-002.

## Authority

- Runtime/deployment mode authority: `docs/architecture/adr-0002-runtime-deployment-modes.md`.
- Operational procedure authority: `docs/operations/01-mvp-runbook.md`.
- This file is environment-focused (compose services and local verification), not deployment-policy authority.

## Services

- Postgres (`localhost:5432`)
- Redis (`localhost:6379`)
- MinIO API (`localhost:9000`)
- MinIO Console (`localhost:9001`)
- coturn TURN/STUN (`localhost:3478` TCP/UDP)
- coturn relay UDP range (`localhost:49160-49200`)

## Runtime Test Stack

- Normal local development uses `infra/docker-compose.yml` for dependencies and host-process app services via `npm run start`.
- PH-05 runtime/network testing uses `infra/docker-compose.runtime-test.yml` for containerized Alice/Bob app instances.
- The runtime test stack is intentionally separate so Docker network controls can target containers without forcing daily development into containers.
- Runtime-test ports bind to `127.0.0.1` only and are not intended for shared-host or LAN exposure.
- The stack uses per-server Postgres, Redis, MinIO, and infra networks so Alice/Bob partitions do not intentionally disconnect local dependencies and cannot bypass the simulation network through shared infra.
- The runtime stack includes Toxiproxy for Docker-only peer-link latency and timeout profiles.
- Realtime containers enable internal dev-fault hooks for app-level delay/drop/disconnect profiles.
- Docker runtime seeding prints dev session cookies/headers; the web Settings testing-profile picker is not enabled in this stack.
- Start it from the repository root with:

  ```bash
  npm run runtime:docker -- up --seed-profile dm-basic
  ```

- Stop it and remove runtime-test data volumes with:

  ```bash
  npm run runtime:docker -- down
  ```

## Startup

1. Copy environment defaults:

   ```bash
   cp .env.example .env
   ```

2. Start the full stack:

   ```bash
   docker compose --env-file .env up -d
   ```

## Health Verification

1. Check container status and health:

   ```bash
   docker compose --env-file .env ps
   ```

   Expected: `postgres`, `redis`, and `minio` show `healthy`.

2. Validate service-level probes:

   ```bash
   docker compose --env-file .env exec postgres pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DB"
   docker compose --env-file .env exec redis redis-cli ping
   curl -fsS http://localhost:9000/minio/health/live
   ```

    Expected outputs:
    - Postgres: `accepting connections`
    - Redis: `PONG`
    - MinIO health endpoint returns HTTP 200

3. Validate TURN allocation credentials:

   ```bash
   docker compose --env-file .env exec coturn turnutils_uclient -t -n 1 -u "$TURN_USER" -w "$TURN_PASSWORD" localhost
   ```

   Expected: allocation succeeds and client output reports at least one successful relay test.

## TURN/NAT Local Test Notes (D-010)

- Canonical test procedure and pass/fail thresholds: `docs/planning/turn-nat-test-profile.md`.
- This section is scoped to Iteration 3 voice/screen-share constrained-network validation; `D-007` remains the separate DM-only infrastructure-free NAT test dependency in `docs/product/04-dependencies-risks.md`.
- For constrained NAT scenarios, run two browser clients on separate hosts or isolated network namespaces.
- Capture TURN logs during each run:

  ```bash
  docker compose --env-file .env logs coturn --since 10m
  ```

- Save run artifacts under `evidence/iteration-03/voice/turn-nat/` using scenario and run IDs from the profile.
