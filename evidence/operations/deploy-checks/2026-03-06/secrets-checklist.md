# Dedicated Server Secrets Checklist

## Metadata

- Date (UTC): 2026-03-06
- Owner: Maximilian Walker
- Environment: CI parity + dedicated-server baseline
- Related PR: #1

## Required Secret Mappings

- `API_DATABASE_URL`
  - Source: GitHub Actions job environment for CI (`.github/workflows/ci.yml`)
  - Validation: API tests and integration smoke passed on run `22744031770`
  - Result: pass
- `API_SESSION_SIGNING_KEYS`
  - Source: GitHub Actions job environment for CI (`.github/workflows/ci.yml`)
  - Validation: session issue/validate/revoke tests and integration smoke passed on run `22744031770`
  - Result: pass

## Required Non-Secret Mappings

- `API_ALLOWED_ORIGINS` and `REALTIME_ALLOWED_ORIGINS`
  - Validation: integration smoke (`web->api->realtime`) passed with expected origin wiring
  - Result: pass
- `API_BIND` and `REALTIME_BIND`
  - Validation: `/health` checks passed in integration smoke job
  - Result: pass

## Checks Performed

- `gh run view 22744031770 --json status,conclusion,jobs`
- Verified `security-audit`, `rust-check (services/api-rs)`, `rust-check (services/realtime-rs)`, `web-check (apps/web)`, `rust-coverage-gate`, `integration-smoke` all `success`.

## Conclusion

- Result: pass
- Notes: This artifact verifies CI environment mapping and dedicated-server baseline readiness for continued development. Production secret store mappings remain environment-specific and must be attached during release execution.
