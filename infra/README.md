# Local Infrastructure Stack

This directory defines the local development infrastructure for HexRelay blocker D-002.

## Services

- Postgres (`localhost:5432`)
- Redis (`localhost:6379`)
- MinIO API (`localhost:9000`)
- MinIO Console (`localhost:9001`)
- coturn TURN/STUN (`localhost:3478` TCP/UDP)
- coturn relay UDP range (`localhost:49160-49200`)

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

## TURN/NAT Local Test Notes (D-007)

- Canonical test procedure and pass/fail thresholds: `docs/planning/turn-nat-test-profile.md`.
- For constrained NAT scenarios, run two browser clients on separate hosts or isolated network namespaces.
- Capture TURN logs during each run:

  ```bash
  docker compose --env-file .env logs coturn --since 10m
  ```

- Save run artifacts under `evidence/iteration-03/voice/turn-nat/` using scenario and run IDs from the profile.
