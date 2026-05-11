# DM Encrypted-Envelope Delivery Execution Plan

## Document Metadata

- Doc ID: dm-envelope-delivery-execution-plan
- Owner: Delivery, core, API, and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/planning/infra-free-dm-connectivity-execution-plan.md`

## Quick Context

- Primary edit location for phased execution of server-node P2P DM encrypted-envelope delivery.
- The legacy file path is retained to avoid link churn; this plan no longer tracks infrastructure-free node-bypassing DM connectivity.
- Cross-scenario networking architecture authority lives in `docs/architecture/04-communication-networking-layer-plan.md`.
- Latest meaningful change: 2026-05-11 added T4.1.9 recipient-targeted realtime dispatch observability for server-node P2P message-node E2EE envelopes.

## Purpose

- Translate the E2EE encrypted-envelope DM baseline into executable backlog slices.
- Provide deterministic implementation order for delivery, metadata, retention, bootstrap, and multi-device convergence.
- Keep delivery aligned with product constraints and test evidence requirements.
- Avoid duplicating shared networking-layer design details that are canonical in the architecture plan.

## Locked Policy Inputs

- DM plaintext, decrypted views, and private keys remain client/device-only.
- Server nodes/message nodes in the server-node P2P network may carry and store only E2EE DM envelopes plus minimal delivery metadata.
- Normal DM send success uses server-node P2P encrypted-envelope delivery.
- Recipient-device LAN/WAN transport, pairing QR/manual-code bootstrap, endpoint hints/cards, connectivity preflight, WAN wizard, and parallel dial are out of scope for DM delivery.
- Unencrypted DM mailboxing, server-side decryption, private-key upload/custody, and plaintext relay behavior are forbidden.

## Execution Phases

### Phase A: Policy and Guardrail Pivot

- Task IDs: `T4.1.3`
- Outcome: product, architecture, contracts, and CI guardrails enforce ciphertext-only message-node semantics.
- Deliverables:
  - E2EE envelope baseline wording in canonical docs.
  - CI policy checks rejecting plaintext/key-custody/unencrypted-mailbox semantics.
  - CI policy checks rejecting node-bypassing DM routes/config/contracts/runtime identifiers.

### Phase B: Relationship and Encryption Bootstrap

- Task IDs: `T4.1.4`, `T4.5.1`
- Outcome: users can establish trusted identity and encryption material after accepted contact/friend state without recipient-device endpoint reachability.
- Deliverables:
  - accepted contact-invite bootstrap release.
  - mediated friend-request bootstrap release after acceptance.
  - no endpoint hints/cards, DM pairing QR/manual-code payloads, or direct-reachability requirements in bootstrap responses.
  - 1:1 E2EE session bootstrap.

### Phase C: Retire Node-Bypassing DM Surfaces

- Task IDs: `T4.1.5`, `T4.1.6`, `T4.1.11`
- Outcome: node-bypassing DM connectivity surfaces are absent from runtime routes, realtime events, web APIs, contracts, docs, tests, and guardrails.
- Deliverables:
  - remove DM preflight/troubleshooter route and web gating.
  - remove DM LAN discovery runtime events, REST routes, state, and tests.
  - remove endpoint-card, WAN wizard, pairing QR/manual-code, and parallel-dial surfaces.
  - preserve server/node discovery or voice/media NAT work only where it is explicitly non-DM.

### Phase D: Encrypted-Envelope Message-Node Delivery

- Task IDs: `T4.1.7`, `T4.1.8`, `T4.5.2`, `T4.5.3`, `T4.5.4`
- Outcome: normal DM/group DM send succeeds through ciphertext-envelope store-and-forward without server plaintext or private-key access.
- Deliverables:
  - message-node ciphertext-envelope accept/store/fanout path.
  - minimal delivery metadata schema and retention/deletion behavior.
  - client-only encrypt/decrypt for 1:1 and group DMs.
  - abuse controls that do not inspect plaintext.

### Phase E: Profile-Device Convergence

- Task IDs: `T4.1.9`, `T4.1.10`
- Outcome: encrypted envelopes converge across active and later-active profile devices.
- Deliverables:
  - active profile-device fanout for ciphertext envelopes.
  - late-device replay/catch-up using deterministic per-device cursors.
  - idempotent dedupe and explicit-read-receipt reconciliation separate from envelope delivery acks.

## Detailed Task Plan

| Task ID | Task | Owner | Depends on | Acceptance criteria |
|---|---|---|---|---|
| T4.1.3 | Enforce E2EE DM envelope policy and CI guardrails | Core | T4.1.1 | CI rejects server-readable plaintext, private-key custody, unencrypted DM mailboxing, plaintext relay semantics, and node-bypassing DM surfaces while allowing encrypted-envelope store-and-forward terminology |
| T4.1.4 | Implement relationship-scoped DM bootstrap | Core/Web | T3.1.4, T4.1.3 | Accepted contact/friend relationships release identity/profile-device bootstrap material with no endpoint hints/cards, QR/manual-code pairing, or direct-reachability requirement |
| T4.1.5 | Retire node-bypassing DM preflight and troubleshooter surfaces | Core/Web | T4.1.4 | Runtime routes, web helpers, contracts, tests, and docs no longer expose DM connectivity preflight or node-bypassing troubleshooting |
| T4.1.6 | Retire user DM LAN discovery fast path | Realtime/Core | T4.1.5 | Realtime and REST surfaces no longer accept or publish user DM LAN discovery hints; server/node discovery remains separately scoped if needed |
| T4.1.7 | Implement encrypted-envelope message-node DM delivery baseline | API/Core | T4.1.3, T4.1.4 | DM send accepts/stores/fans out ciphertext envelopes plus minimal metadata; server rejects plaintext/private-key inputs; direct reachability is not required |
| T4.1.8 | Add DM delivery metadata minimization, retention, and abuse controls | API/Core/Security | T4.1.7 | Metadata schema excludes plaintext/private keys/recipient-device endpoints, retention/deletion behavior is deterministic, and rate/abuse controls operate without plaintext inspection |
| T4.1.9 | Add DM active-device profile fanout semantics | Core/Realtime | T4.1.7 | Accepted ciphertext envelopes fan out to all currently active devices linked to recipient profile |
| T4.1.10 | Add DM late-device catch-up and per-device cursor dedupe | Core | T4.1.8, T4.1.9 | Devices activated after first delivery replay missed ciphertext envelopes and converge deterministically |
| T4.1.11 | Retire WAN wizard, endpoint-card, and parallel-dial DM backlog | Core/Web | T4.1.7 | Runtime, web, contracts, docs, tests, and guardrails contain no DM WAN wizard, endpoint-card, or parallel-dial surfaces |

## Validation and Evidence Plan

- `T4.1.3`: policy gate report proving unsafe plaintext/key-custody semantics and node-bypassing DM surfaces fail while encrypted-envelope message-node wording passes.
- `T4.1.4`: bootstrap conformance report proving accepted relationships release only identity/profile-device material and blocked/pending relationships fail closed.
- `T4.1.5`: route/API/web/contract negative checks proving DM preflight/troubleshooter surfaces are absent.
- `T4.1.6`: route/realtime/contract negative checks proving user DM LAN discovery surfaces are absent.
- `T4.1.7`: encrypted-envelope delivery report proving server accepts/stores/fans out ciphertext only and rejects plaintext/private-key inputs.
- `T4.1.8`: delivery metadata and retention report proving metadata minimization, deterministic metadata deletion behavior, and abuse controls without plaintext inspection.
- `T4.1.9`: DM active-device fanout matrix for multi-device profiles.
- `T4.1.10`: late-device replay/catch-up convergence report for offline-then-activate scenarios.
- `T4.1.11`: negative checks proving WAN wizard, endpoint-card, and parallel-dial DM surfaces are absent.

Evidence path baseline:

- `evidence/iteration-02/dm-connectivity/`
- `evidence/iteration-02/messaging-e2ee/`

## Current T4.1.8 Implementation Notes

- Metadata minimization finding: the runtime no longer has DM endpoint-card or pairing-nonce tables after migration `0019`, and the active delivery path stores only ciphertext envelopes plus identity, message/thread, device/cursor, state, retry, and timestamp metadata.
- Retention finding: canonical ciphertext history and replay metadata are separate concerns. The T4.1.8 purge path deletes expired replay/forwarding metadata but does not delete `dm_messages`.
- Fanout retention rule: delete expired `dm_fanout_delivery_log` rows when all registered profile devices have advanced past the row cursor, or when the identity has no registered profile devices and the row has expired.
- Outbound retention rule: delete expired `forwarded` and terminal `failed` `dm_outbound_forwarding_log` rows; keep queued or retry-scheduled rows until retry resolution.
- Abuse-control rule: DM dispatch uses sender-scoped rate limits; catch-up and ack use identity/device-scoped rate limits; authenticated node-forward ingress uses origin-node-scoped rate limits.
- Config defaults: `API_DM_DISPATCH_RATE_LIMIT=120`, `API_DM_CATCH_UP_RATE_LIMIT=120`, `API_DM_ACK_RATE_LIMIT=600`, `API_DM_INTERNAL_FORWARD_RATE_LIMIT=240`, `API_DM_DELIVERY_LOG_RETENTION_SECONDS=2592000`, and `API_DM_OUTBOUND_FORWARDING_LOG_RETENTION_SECONDS=604800`.
- No UX changes were introduced by this slice.

## Current T4.1.9 Implementation Notes

- Active-device targeting finding: API fanout still selects active recipient profile devices from the server-side device manifest and sends only those device ids to realtime.
- Realtime dispatch finding: realtime now returns an internal dispatch summary for every DM envelope dispatch request with target count, queued-to-verified-websocket ids, pending ids, no-connection ids, unverified-device-binding ids, saturated-queue ids, and stale connection cleanup count.
- Correctness boundary: queued-to-verified-websocket means the envelope was queued to a verified websocket connection for that profile device. It is not final recipient delivery, read state, or UX-facing delivery status.
- Ack boundary: final delivery remains `dm.envelope.ack` backed. API dispatch responses keep `delivered_device_ids` empty until ack-backed delivery state exists, and late-device catch-up remains the deterministic fallback for pending devices.
- Metadata boundary: realtime dispatch summaries contain ids, counts, and state categories only; they do not contain plaintext, private keys, endpoint hints, LAN/WAN addresses, pairing material, or direct user-to-user transport state.
- No UX changes were introduced by this slice.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Message-node delivery accidentally expands to server-readable DM content | High | CI guardrail, schema tests, logging tests, and explicit client-only decrypt/key ownership |
| Delivery metadata becomes too revealing | High | Minimal metadata schema, retention policy, and evidence row for metadata minimization |
| Abuse controls are weaker without plaintext inspection | Medium | Relationship gates, block/denylist checks, rate limits, envelope-count heuristics, and operator-visible non-content diagnostics |
| Retired node-bypassing DM concepts re-enter through docs/contracts/tests | Medium | Node-bypassing DM guardrail pattern plus contract/runtime negative checks |
| Multi-device convergence drifts across profile devices | High | Per-device cursor contracts, idempotent replay semantics, and late-activation convergence tests |

## Non-Goals

- Server-readable DM content.
- Private-key upload, escrow, or server custody.
- Unencrypted DM mailboxing or plaintext relay behavior.
- Node-bypassing LAN/WAN DM transport, pairing QR/manual-code bootstrap, endpoint hints/cards, preflight, WAN wizard, or parallel dial.
- Broad architecture refactors outside DM delivery and retired node-bypassing DM surface cleanup.

## Related Documents

- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
- `docs/testing/01-mvp-verification-matrix.md`
