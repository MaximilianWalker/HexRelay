# Confidence Hardening Validation - 2026-03-04

## Scope

- Token hardening: versioned bearer format with key-id based signing key selection.
- Abuse controls: rate limits on auth challenge/verify, invite create/redeem, and realtime websocket connect.
- Runtime persistence convergence: non-test runtime paths require DB for identity/auth/invite/session critical flows.
- Contract cleanup: canonical runtime REST contract moved to `docs/contracts/runtime-rest-v1.openapi.yaml`.

## Validation Commands

```bash
npm run test
```

## Expected Evidence Contract

- Rust checks pass for `services/api-rs` and `services/realtime-rs`.
- Web lint/test/build pass for `apps/web`.
- No dirty runtime fallback usage in non-test auth/invite/session runtime paths.
- Runtime docs route to `docs/contracts/runtime-rest-v1.openapi.yaml` as authority.

## Notes

- Legacy runtime REST contract path is retained as compatibility alias:
  - `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`
- Compatibility alias remains non-authoritative and should be removed only after downstream references are retired.
