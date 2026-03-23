# HexRelay MVP Verification Matrix

## Document Metadata

- Doc ID: mvp-verification-matrix
- Owner: Delivery and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-22
- Source of truth: `docs/testing/01-mvp-verification-matrix.md`

## Quick Context

- Purpose: bind requirements to verification evidence for deterministic iteration sign-off.
- Primary edit location: update when requirement/task coverage or evidence format changes.
- Latest meaningful change: 2026-03-22 T3.3.1 added end-to-end Redis-backed contact hydration coverage, and T3.3.2 added late-device presence convergence verification for websocket profile devices.

## Requirement to Evidence Matrix

| Requirement area | Task IDs | Evidence artifact | Validator | Evidence path pattern |
|---|---|---|---|---|
| Identity/auth/invites | T2.1.1, T2.2.1, T2.3.1, T2.4.1 | OpenAPI conformance + auth/invite integration report | `npm --prefix apps/web run e2e:smoke` and `cargo test -p api-rs --all-features` | `evidence/iteration-01/identity-auth-invites/<YYYY-MM-DD>/` |
| Friend request privacy mediation | T3.1.1, T3.1.2, T3.1.5 | Privacy policy test report (no pre-accept identity exposure) | Friend-request integration tests assert redaction before acceptance | `evidence/iteration-02/friend-privacy/<YYYY-MM-DD>/` |
| DM inbound policy defaults/overrides | T4.1.2 | DM policy matrix test output | Integration tests cover default + override policy matrix outcomes | `evidence/iteration-03/dm-policy/<YYYY-MM-DD>/` |
| Infrastructure-free DM connectivity conformance | T4.1.3-T4.1.8 | Direct-connect conformance report (policy gate, pairing validation, diagnostics, LAN/WAN pathing) | Direct-connect suite confirms no STUN/TURN/relay fallback and deterministic failure guidance behavior | `evidence/iteration-02/dm-connectivity/<YYYY-MM-DD>/` |
| DM multi-device eventual-sync convergence | T4.1.9, T4.1.10 | DM fanout + late-device catch-up report | Tests verify one message converges across all profile devices, including devices activated after first receive | `evidence/iteration-02/dm-connectivity/<YYYY-MM-DD>/` |
| Server-channel/presence multi-device convergence | T3.3.2, T4.3.4 | Server event fanout + hydration report | Tests verify channel/presence events hydrate all profile devices via per-device cursor after reconnect/late activation | `evidence/iteration-02/profile-device-sync/<YYYY-MM-DD>/` |
| E2EE DM (1:1 + group) | T4.5.1-T4.5.4 | Direct user-to-user transport assertion + decrypt success + offline outbox retry report | Crypto conformance checklist and E2EE integration suite pass | `evidence/iteration-02/messaging-e2ee/<YYYY-MM-DD>/` |
| Navigation and hubs | T4.6.1-T4.6.4 | UI checklist with screenshots (desktop + mobile) | Manual checklist completed against `docs/product/07-ui-navigation-spec.md` and `docs/product/08-screen-state-spec.md` | `evidence/iteration-02/navigation/<YYYY-MM-DD>/` |
| Voice/screen share | T5.1.1-T5.3.1 | KPI profile run report (join success, reconnect, jitter) | TURN/NAT profile procedure passes with required metrics captured | `evidence/iteration-03/voice/turn-nat/<scenario-id>/` |
| Migration and reconciliation | T7.1.2, T7.5.1-T7.5.5 | Migration scenario evidence (LAN/file/cutover) | Migration validation template completed with forward, rerun, rollback, and integrity checks | `evidence/migrations/<migration-name>.md` |
| Observability/SLO alerts | T8.1.1, T8.2.1 | Dashboard export + fault-injection alert report | Observability evidence template completed with alert and recovery timestamps | `evidence/operations/observability/<YYYY-MM-DD>/` |

Observability evidence format template: `docs/testing/observability-evidence-template.md`.

## Evidence Format

- `artifact`: report/screenshot/log/export path
- `validator`: command or deterministic manual check
- `result`: pass/fail
- `timestamp`: UTC datetime
- `commit_sha`: commit used to generate evidence
- `pr_number` or `run_id`: source execution context

## Minimum Artifact Set Per Matrix Row

- Every matrix evidence folder must include:
  - `summary.md` (requirement IDs, scope, outcome, owner)
  - `validators.txt` (exact command/manual checklist run)
  - `outputs/` (raw logs, screenshots, or exports referenced by `summary.md`)
  - `provenance.json` with:
    - `commit_sha`
    - `pr_number` (or `run_id` for CI-only evidence)
    - `generated_at_utc`
- If a required output is unavailable, record it explicitly in `summary.md` with `missing` status and rationale.
- CI enforces provenance for changed evidence artifacts via `evidence-provenance-check`.

## Related Documents

- `docs/planning/iterations/README.md`
- `docs/planning/05-iteration-log.md`
- `docs/planning/kpi-slo-test-profile.md`
