# HexRelay TURN/NAT Test Profile (Iteration 3)

## Document Metadata

- Doc ID: turn-nat-test-profile
- Owner: Platform and Realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/turn-nat-test-profile.md`

## Quick Context

- Purpose: define a deterministic TURN/NAT validation profile for Iteration 3 voice and screen share gates.
- Primary edit location: update this file when NAT scenarios, thresholds, or evidence requirements change.
- Latest meaningful change: 2026-03-04 created D-007 TURN/NAT profile with measurable pass/fail criteria and evidence contract.

## Scope and Alignment

- Applies to Iteration 3 tasks `T5.1.2`, `T5.2.1`, and `T5.3.1` in `docs/planning/iterations/03-sprint-board.md`.
- Complements `docs/planning/kpi-slo-test-profile.md`; this profile is the constrained-network procedure for the voice/media KPI targets.
- Scope clarification: this profile is for Iteration 3 voice/screen-share media paths only; DM transport remains infra-free direct-only under `D-007` in `docs/product/04-dependencies-risks.md`.

## Test Environment Baseline

- TURN server: local `coturn` from the compose stack with UDP/TCP 3478 and relay range UDP 49160-49200 enabled.
- ICE policy for test clients: `iceTransportPolicy=all` so direct path is attempted first and fallback is observable.
- Test clients: two browser clients on separate network namespaces/hosts to avoid shared-NAT false positives.
- Media payload: Opus voice stream + 720p/30fps screen share stream for all scenarios.
- Time sync: all test hosts use NTP-synced clocks to keep timestamps consistent across logs.

## Network Scenario Matrix

| Scenario ID | NAT/Network shape | Expected media route | Primary objective |
|---|---|---|---|
| NAT-01 | Restricted cone NAT on caller, full cone NAT on callee, UDP open | Direct (`srflx`) preferred, TURN optional | Verify no TURN regression in moderate NAT conditions |
| NAT-02 | Symmetric NAT on both sides, inbound UDP blocked | TURN relay over UDP/TCP | Validate reliable join and screen share with relay fallback |
| NAT-03 | Symmetric NAT on both sides, UDP blocked, TCP 3478 open | TURN relay over TCP | Validate fallback when UDP relay is unavailable |
| NAT-04 | Double NAT + 120 ms RTT + 2% packet loss + 20 ms jitter | TURN relay expected in most attempts | Validate reconnect and stability under constrained quality |

## Run Procedure

1. Start compose stack and confirm `coturn` health before test traffic.
2. For each scenario (`NAT-01` to `NAT-04`), execute 3 runs.
3. In each run, execute 10 voice join attempts and 10 screen share start/stop attempts.
4. Capture ICE selected candidate type for every successful attempt.
5. Export run artifacts to `evidence/iteration-03/voice/turn-nat/<scenario-id>/<run-id>/`.

## Pass/Fail Criteria

### Voice Join (per scenario across 3 runs, 30 attempts)

- Success rate must be >= 98%.
- Call setup latency p95 must be < 3 seconds.
- For `NAT-02` to `NAT-04`, at least 95% of successful joins must select `relay` ICE candidate type.
- Any one-way-audio event longer than 5 seconds is an automatic fail.

### Screen Share (per scenario across 3 runs, 30 attempts)

- Success rate must be >= 95%.
- First remote frame time p95 must be <= 5 seconds.
- Reconnect after induced 5-second network drop must recover in <= 5 seconds for >= 95% of attempts.
- For `NAT-02` to `NAT-04`, at least 95% of successful sessions must select `relay` ICE candidate type.

### Global Gate

- All four scenarios must pass voice and screen share criteria.
- If any scenario fails, `D-007` remains blocked and Iteration 3 exit evidence is incomplete.

## Evidence Artifacts Required Per Run

- `scenario-config.json`: NAT profile, shaping parameters, client versions, and TURN endpoint settings.
- `attempt-log.ndjson`: one record per attempt with timestamps, outcome, setup time, reconnect time, and selected ICE candidate type.
- `kpi-summary.json`: computed success rates and p95 metrics for voice join and screen share.
- `client-a-webrtc-internals.json` and `client-b-webrtc-internals.json`: browser WebRTC diagnostics export.
- `coturn.log`: TURN server log covering allocation, authentication, and relay lifecycle for the run window.
- `verdict.md`: explicit pass/fail decision for the run with failed-attempt IDs.

## Evidence Aggregation and Sign-off

- Scenario-level aggregate report path: `evidence/iteration-03/voice/turn-nat/report.md`.
- Report must include per-scenario metric table, failures, and remediation links when thresholds are missed.
- Final sign-off requires references from Iteration 3 evidence ledger in `docs/planning/iterations/03-sprint-board.md`.

## Related Documents

- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/product/04-dependencies-risks.md`
