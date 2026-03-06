# ADR-0003: Rust Service Module Architecture Contract

## Document Metadata

- Doc ID: adr-0003-rust-service-module-architecture
- Owner: Architecture maintainers
- Status: accepted
- Scope: repository
- last_updated: 2026-03-06
- Source of truth: `docs/architecture/adr-0003-rust-service-module-architecture.md`

## Quick Context

- Primary decision authority for Rust API crate structure in HexRelay.
- Update this ADR when folder/module boundaries, layer responsibilities, or dependency rules change.
- Latest meaningful change: 2026-03-06 established shared module contract for `api-rs` and `realtime-rs` to support full structural migration.

## Status

Accepted

## Context

HexRelay backend behavior is currently functional but module layout is inconsistent across services and overly concentrated in a small set of large files. This raises review cost, slows refactoring, and increases regression risk.

The project needs one structure contract that applies to both Rust services (`services/api-rs` and `services/realtime-rs`) so migration can be staged without ambiguity.

## Decision

Both Rust services must converge on the same top-level crate layout:

```text
src/
  main.rs
  lib.rs

  app/
    mod.rs
    config.rs
    router.rs
    state.rs

  domain/
    <feature>/
      mod.rs
      model.rs
      service.rs
      validation.rs

  transport/
    http/ or ws/
      mod.rs
      error.rs
      middleware/
        mod.rs
        <concern>.rs
      handlers/
        mod.rs
        <feature>.rs
      dto/
        mod.rs
        <feature>.rs

  infra/
    db/
      mod.rs
      pool.rs
      migrations.rs
      repos/
        mod.rs
        <feature>_repo.rs
    crypto/
      mod.rs
      <concern>.rs

  shared/
    mod.rs
    errors.rs
    types.rs

  tests/
    mod.rs
    <feature>_tests.rs
```

Layer dependency direction is mandatory:

- `transport -> domain -> infra`
- `app` composes and wires layers but does not contain feature business rules.
- `domain` cannot depend on `transport`.
- `infra` cannot depend on `transport`.
- `shared` contains reusable primitives only; it must not become a feature dumping ground.

Module responsibility contract:

- Handlers parse IO and call domain services.
- Domain services own business behavior and state transitions.
- Repositories own SQL/querying and persistence mapping.
- Middleware owns cross-cutting request concerns (auth, csrf, rate limits, correlation IDs).
- DTO modules contain wire types, distinct from domain models where practical.

`lib.rs` contract:

- Keep it as module wiring only.
- No endpoint business logic in `lib.rs`.

## Consequences

- Positive:
  - Predictable folder/module ownership for both services.
  - Lower churn risk from giant files.
  - Easier review and safer feature additions after migration.
- Trade-offs:
  - Temporary migration overhead and higher file count.
  - Additional discipline needed to prevent layer boundary violations.

## Alternatives Considered

- Keep current mixed structure and only split the largest files.
  - Rejected: reduces pain locally but preserves cross-service inconsistency and unclear ownership boundaries.
- Introduce framework-heavy architecture macros/abstractions.
  - Rejected: unnecessary complexity for MVP and harder debugging.

## Migration Guardrails

- Migrate `api-rs` first, then `realtime-rs` after structure and conventions stabilize.
- Use no-behavior-change staged PRs per layer boundary.
- Each migration stage must pass:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`
  - workspace CI required checks

## Related Documents

- `docs/architecture/adr-0001-stack-baseline.md`
- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/operations/contributor-guide.md`
- `services/api-rs/README.md`
- `services/realtime-rs/README.md`
