# Iteration Boards

## Document Metadata

- Doc ID: planning-iterations-index
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/planning/iterations/README.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 execution-hardening added screen/config/data/runbook/testing artifacts.

## Canonical Boards

- Iteration 1: `docs/planning/iterations/01-sprint-board.md`
- Iteration 2: `docs/planning/iterations/02-sprint-board.md`
- Iteration 3: `docs/planning/iterations/03-sprint-board.md`
- Iteration 4: `docs/planning/iterations/04-sprint-board.md`

## PRD Requirement to Task Trace Matrix

| Requirement area | Task IDs |
|---|---|
| Identity key registration and auth challenge/verify | T2.1.1, T2.3.1 |
| Direct user add via contact invite link/QR | T3.1.3, T3.1.4 |
| Server-mediated friend request privacy model | T3.1.1, T3.1.2, T3.1.5 |
| Multi-persona support and switching | T2.1.3 |
| Mandatory recovery phrase onboarding | T2.1.4 |
| Invite create/redeem with mode/expiration/max-uses | T2.2.1 |
| Join fingerprint verification (fail closed) | T2.4.1 |
| Friends/block/mute/presence | T3.1.1, T3.1.2, T3.2.1, T3.3.1 |
| Global/shared-server user discovery with abuse controls | T3.4.1 |
| DM/group DM direct transport + server-channel messaging primitives | T4.1.1, T4.3.1, T4.3.2 |
| DM inbound policy defaults and user overrides | T4.1.2 |
| E2EE 1:1 DM | T4.5.1, T4.5.2 |
| E2EE group DM | T4.5.3, T4.5.4 |
| Servers/contacts hubs and dual-mode navigation | T4.6.1, T4.6.2, T4.6.3, T4.6.4 |
| Voice/call/screen share reliability | T5.1.1, T5.1.2, T5.2.1, T5.3.1 |
| Attachments, quotas, local moderation controls | T6.1.1, T6.1.2, T6.2.1, T6.3.1 |
| Discovery registry and profile replication | T7.2.1, T7.2.2, T7.4.1, T7.4.2 |
| Export/import/full migration and cutover | T7.1.1, T7.1.2, T7.5.1, T7.5.2, T7.5.3, T7.5.4, T7.5.5 |
| Observability, SLOs, beta docs | T8.1.1, T8.2.1, T8.3.1 |
| NFR: reliability (reconnect, ordering, consistency) | T3.3.1, T4.3.2, T5.1.1, T5.3.1 |
| NFR: security (nonce replay, auth verification, no guild-server DM relay/storage) | T2.3.1, T2.4.1, T4.5.2, T4.5.4 |
| NFR: performance (chat latency, call setup/jitter, alert detection windows) | T4.3.2, T5.2.1, T8.2.1 |
| NFR: operability (compose-first, CI gates, observability dashboards) | T1.1.2, T1.2.1, T8.1.1 |
| KPI: message delivery p95 < 300ms | T4.3.2, T8.1.1, T8.2.1 |
| KPI: voice join success > 98% | T5.1.1, T5.1.2, T8.1.1, T8.2.1 |
| KPI: screen share success > 95% | T5.3.1, T8.1.1, T8.2.1 |
| KPI: challenge-signature auth success > 99.5% | T2.3.1, T8.1.1, T8.2.1 |
| KPI: E2EE DM decrypt success > 99.5% | T4.5.2, T4.5.4, T8.1.1 |
| KPI: migration success in at least 3 scenarios | T7.5.2, T7.5.3, T7.5.4 |
| Discovery policy: public listing support and private-by-default behavior | T7.2.1, T7.2.2, T7.3.1 |

## Iteration Handoff Matrix

| Iteration | Produces | Required by next iteration |
|---|---|---|
| Iteration 1 | Local stack, CI gates, identity/auth/invite contract and flows | Iteration 2 social graph, messaging, E2EE DM |
| Iteration 2 | Messaging/realtime baseline, E2EE 1:1+group DM, navigation hubs | Iteration 3 voice/media features |
| Iteration 3 | Voice/screen share/media and moderation controls | Iteration 4 migration hardening and observability gates |
| Iteration 4 | Discovery portability, migration completeness, SLO and beta docs | MVP release readiness |

## Artifact Gate Checklist

| Iteration | Required artifacts before execution | Evidence owner |
|---|---|---|
| Iteration 1 | `docs/contracts/iteration-01-identity-auth-invites.openapi.yaml`, MVP Crypto Profile v1 alignment | API/Core |
| Iteration 2 | `docs/contracts/mvp-rest-v1.openapi.yaml`, navigation spec trace matrix, E2EE group DM task set, `docs/contracts/realtime-events-v1.asyncapi.yaml` | Web/Core/Realtime/API |
| Iteration 3 | TURN/NAT test environment and voice quality test profile | Platform/Realtime |
| Iteration 4 | Migration conflict policy, bundle schema/version compatibility rules, SLO alert test profile, `docs/testing/01-mvp-verification-matrix.md` | API/Core/Platform/QA |

## Exit Evidence Pack Format

| Field | Description |
|---|---|
| `artifact` | Contract, schema, report, dashboard export, or test result |
| `owner` | Accountable role or team |
| `validation` | Command or deterministic manual verification |
| `result` | Pass/Fail with timestamp |
| `link` | Path to evidence file or run output location |
