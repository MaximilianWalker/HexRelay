# HexRelay MVP Verification Matrix

## Document Metadata

- Doc ID: mvp-verification-matrix
- Owner: Delivery and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-10
- Source of truth: `docs/testing/01-mvp-verification-matrix.md`

## Quick Context

- Purpose: bind requirements to verification evidence for deterministic iteration sign-off.
- Primary edit location: update when requirement/task coverage or evidence format changes.
- Latest meaningful change: 2026-03-09 expanded matrix rows with deterministic validator commands and evidence path patterns.

## Requirement to Evidence Matrix

| Requirement area | Task IDs | Evidence artifact | Validator | Evidence path pattern |
|---|---|---|---|---|
| Identity/auth/invites | T2.1.1, T2.2.1, T2.3.1, T2.4.1 | OpenAPI conformance + auth/invite integration report | `npm --prefix apps/web run e2e:smoke` and `cargo test -p api-rs --all-features` | `evidence/iteration-01/identity-auth-invites/<YYYY-MM-DD>/` |
| Friend request privacy mediation | T3.1.1, T3.1.2, T3.1.5 | Privacy policy test report (no pre-accept identity exposure) | Friend-request integration tests assert redaction before acceptance | `evidence/iteration-02/friend-privacy/<YYYY-MM-DD>/` |
| DM inbound policy defaults/overrides | T4.1.2 | DM policy matrix test output | Integration tests cover default + override policy matrix outcomes | `evidence/iteration-03/dm-policy/<YYYY-MM-DD>/` |
| E2EE DM (1:1 + group) | T4.5.1-T4.5.4 | Direct user-to-user transport assertion + decrypt success + offline outbox retry report | Crypto conformance checklist and E2EE integration suite pass | `evidence/iteration-03/e2ee-dm/<YYYY-MM-DD>/` |
| Navigation and hubs | T4.6.1-T4.6.4 | UI checklist with screenshots (desktop + mobile) | Manual checklist completed against `docs/product/07-ui-navigation-spec.md` and `docs/product/08-screen-state-spec.md` | `evidence/iteration-03/navigation/<YYYY-MM-DD>/` |
| Voice/screen share | T5.1.1-T5.3.1 | KPI profile run report (join success, reconnect, jitter) | TURN/NAT profile procedure passes with required metrics captured | `evidence/iteration-03/voice/turn-nat/<scenario-id>/` |
| Migration and reconciliation | T7.1.2, T7.5.1-T7.5.5 | Migration scenario evidence (LAN/file/cutover) | Migration validation template completed with forward, rerun, rollback, and integrity checks | `evidence/migrations/<migration-name>.md` |
| Observability/SLO alerts | T8.1.1, T8.2.1 | Dashboard export + fault-injection alert report | Observability evidence template completed with alert and recovery timestamps | `evidence/operations/observability/<YYYY-MM-DD>/` |

Observability evidence format template: `docs/testing/observability-evidence-template.md`.

## Evidence Format

- `artifact`: report/screenshot/log/export path
- `validator`: command or deterministic manual check
- `result`: pass/fail
- `timestamp`: UTC datetime

## Related Documents

- `docs/planning/iterations/README.md`
- `docs/planning/05-iteration-log.md`
- `docs/planning/kpi-slo-test-profile.md`
