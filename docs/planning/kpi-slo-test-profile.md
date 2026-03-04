# HexRelay KPI/SLO Test Profile (MVP)

## Document Metadata

- Doc ID: kpi-slo-test-profile
- Owner: Platform and delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/kpi-slo-test-profile.md`

## Quick Context

- Purpose: define one fixed test profile so KPI/SLO results are comparable across iterations.
- Primary edit location: update this file when benchmark environment or load profile changes.
- Latest meaningful change: 2026-03-04 linked constrained TURN/NAT validation profile for Iteration 3 voice/media gates.

## Fixed Test Profile

- Concurrency baseline: 200 concurrent active users.
- Network profile mix: 70% WiFi (stable), 30% Fast 4G.
- Browser matrix: latest stable Chrome and Firefox.
- Region profile: single-region staging environment.
- Voice/media profile: TURN available and enabled in test environment.
- TURN/NAT constrained-network validation profile: `docs/planning/turn-nat-test-profile.md`.

## KPI/SLO Evidence Rules

- Run each benchmark scenario at least 3 times and report median + p95.
- Store raw outputs and summarized metrics in the iteration evidence pack.
- Any failed target requires a linked remediation task before iteration sign-off.
- For voice and screen share under NAT constraints, follow the procedure and thresholds in `docs/planning/turn-nat-test-profile.md`.

## Related Documents

- `docs/product/02-prd-v1.md`
- `docs/planning/turn-nat-test-profile.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/planning/iterations/04-sprint-board.md`
