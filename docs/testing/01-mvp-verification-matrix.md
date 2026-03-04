# HexRelay MVP Verification Matrix

## Document Metadata

- Doc ID: mvp-verification-matrix
- Owner: Delivery and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/testing/01-mvp-verification-matrix.md`

## Quick Context

- Purpose: bind requirements to verification evidence for deterministic iteration sign-off.
- Primary edit location: update when requirement/task coverage or evidence format changes.
- Latest meaningful change: 2026-03-04 execution-hardening pass added MVP verification bindings.

## Requirement to Evidence Matrix

| Requirement area | Task IDs | Evidence artifact |
|---|---|---|
| Identity/auth/invites | T2.1.1, T2.2.1, T2.3.1, T2.4.1 | OpenAPI conformance + auth/invite integration report |
| Friend request privacy mediation | T3.1.1, T3.1.2, T3.1.5 | Privacy policy test report (no pre-accept identity exposure) |
| DM inbound policy defaults/overrides | T4.1.2 | DM policy matrix test output |
| E2EE DM (1:1 + group) | T4.5.1-T4.5.4 | Direct user-to-user transport assertion + decrypt success + offline outbox retry report |
| Navigation and hubs | T4.6.1-T4.6.4 | UI checklist with screenshots (desktop + mobile) |
| Voice/screen share | T5.1.1-T5.3.1 | KPI profile run report (join success, reconnect, jitter) |
| Migration and reconciliation | T7.1.2, T7.5.1-T7.5.5 | Migration scenario evidence (LAN/file/cutover) |
| Observability/SLO alerts | T8.1.1, T8.2.1 | Dashboard export + fault-injection alert report |

## Evidence Format

- `artifact`: report/screenshot/log/export path
- `validator`: command or deterministic manual check
- `result`: pass/fail
- `timestamp`: UTC datetime

## Related Documents

- `docs/planning/iterations/README.md`
- `docs/planning/05-iteration-log.md`
- `docs/planning/kpi-slo-test-profile.md`
