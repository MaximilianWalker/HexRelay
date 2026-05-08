# DM Encrypted-Envelope Delivery Proposals

## Document Metadata

- Doc ID: dm-envelope-delivery-proposals
- Owner: Product and realtime maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-08
- Source of truth: `docs/product/10-infra-free-dm-connectivity-proposals.md`

## Quick Context

- Primary edit location for detailed DM delivery solution candidates and trade-offs.
- The legacy file path is retained to avoid link churn; this document no longer defines infrastructure-free peer-routed DM connectivity.
- Cross-scenario networking implementation details are canonical in `docs/architecture/04-communication-networking-layer-plan.md`.
- Latest meaningful change: 2026-05-08 locked proposals to node/server-routed E2EE envelope delivery and removed user direct-DM transport/bootstrap proposals.

## Purpose

- Convert the MVP DM delivery baseline into concrete implementation options.
- Keep the security boundary explicit: servers/message nodes may carry ciphertext envelopes, never DM plaintext or private keys.
- Keep user direct-DM LAN/WAN transport, endpoint hints/cards, pairing QR/manual-code bootstrap, connectivity preflight, WAN wizard, and parallel dial out of MVP scope.
- Keep execution sequencing in `docs/planning/infra-free-dm-connectivity-execution-plan.md` and avoid duplicating full implementation architecture here.

## Locked Constraints

- DM plaintext, decrypted views, and private keys remain client/device-only.
- Shared servers/message nodes may carry and store only E2EE DM envelopes plus minimal delivery metadata.
- Normal DM send success uses node/server encrypted-envelope delivery.
- Contact/friend bootstrap may expose only the identity and profile-device material required for client-side E2EE setup after relationship acceptance.
- Unencrypted DM mailboxing, server-side decryption, server-readable DM content, private-key upload/custody, plaintext relay behavior, and user direct-DM transport/bootstrap are out of scope.

## Evaluation Criteria

- **Confidentiality boundary**: no server-readable plaintext or private-key custody.
- **Metadata minimization**: delivery metadata is necessary, bounded, and covered by retention/deletion policy.
- **Normal-user reliability**: successful DM delivery works without NAT/router setup, LAN discovery, or direct peer reachability.
- **Client-only decryption**: recipients decrypt locally and failures are recoverable without server secrets.
- **Relationship-gated bootstrap**: public identity and profile-device material are released only through accepted contact/friend state.
- **Implementation risk**: complexity and portability across target desktop environments.

## Proposition 1 (Rank 1): E2EE Envelope Message-Node Baseline

### What Changes

- Make shared servers/message nodes the default and only MVP DM delivery path for ciphertext envelopes.
- Store canonical encrypted DM history and minimal delivery metadata before sender-visible success.
- Remove user direct-DM bootstrap/connectivity assumptions from API, realtime, web, contracts, docs, and tests.

### How It Works

1. Sender client validates relationship, DM policy, block state, and trusted bootstrap material.
2. Sender client encrypts per-recipient/device DM envelopes locally.
3. Message node accepts only ciphertext envelopes and minimal metadata for routing, dedupe, delivery state, retention, and abuse controls.
4. Message node fans out envelopes to active recipient devices and exposes per-device cursor catch-up for later-active devices.
5. Recipient clients decrypt locally and merge delivery/read state deterministically.

### Trade-Offs and Risks

- Metadata minimization needs explicit schema discipline.
- Abuse/spam controls must work without inspecting DM plaintext.
- Server storage expands to ciphertext envelope durability while preserving client-only plaintext/key ownership.

### Acceptance Criteria

- Server-side tests prove DM plaintext/private keys are never accepted, stored, logged, or returned.
- DM send success requires durable ciphertext-envelope acceptance plus delivery metadata.
- Offline and later-active devices catch up from encrypted envelopes by per-device cursor.
- Operator-visible diagnostics expose delivery state without plaintext.

## Proposition 2 (Rank 2): Client-Only Key and Envelope Guardrails

### What Changes

- Add durable guardrails around key custody, envelope shapes, logging, and server-side validation.
- Reject unsafe semantics and reintroduced user direct-DM surfaces mechanically.

### How It Works

1. Client-side crypto owns private keys and envelope encryption/decryption.
2. Runtime APIs accept ciphertext envelope fields and reject plaintext-like DM payload fields.
3. Logs and audit events record ids, state, and reason codes only.
4. CI policy checks reject server-readable plaintext, private-key upload/custody, unencrypted mailbox, plaintext relay semantics, and user direct-DM routes/config/contracts.

### Acceptance Criteria

- CI guardrail passes legitimate `encrypted envelope`, `store-and-forward`, and `message node` terminology.
- CI guardrail fails fixtures or callsites that introduce plaintext DM storage, server-side decryption, private-key upload semantics, or user direct-DM endpoints.
- Crypto and API tests cover envelope-only server handling and client-only decrypt behavior.

## Proposition 3 (Rank 3): Relationship and Encryption Bootstrap

### What Changes

- Use accepted contact-invite redemption and accepted mediated friend requests as trust gates for identity and encryption bootstrap material.
- Decouple bootstrap from direct endpoint reachability entirely.

### How It Works

1. Contact-invite redemption or accepted friend request establishes trusted relationship state.
2. API returns only the peer identity key and profile-device snapshot required for client-side E2EE setup.
3. Block state, request state, and DM policy checks fail closed before bootstrap material is released.
4. Once trusted, the message-node path carries encrypted envelopes without requiring direct dial success.

### Acceptance Criteria

- Accepted mediated friend requests release only the bootstrap material needed for DM relationship and encryption setup.
- Pending/declined/blocked relationships cannot retrieve bootstrap material.
- Bootstrap responses contain no direct endpoint hints/cards, LAN/WAN data, QR/manual-code pairing payloads, or relay secrets.

## Proposition 4 (Rank 4): Delivery Metadata, Retention, and Abuse Controls

### What Changes

- Define the minimum metadata message nodes need to route, dedupe, retain, delete, and rate-limit encrypted envelopes.
- Add explicit retention/deletion behavior for encrypted envelopes and delivery receipts.

### How It Works

1. Each accepted envelope has stable message/thread ids, sender/recipient/device routing ids, timestamps, delivery state, and dedupe metadata.
2. Metadata excludes plaintext, plaintext-derived searchable content, private keys, and direct endpoint material.
3. Retention policy applies to ciphertext envelopes and delivery metadata separately from client-local decrypted views.
4. Abuse controls use relationship state, rate limits, deny/block state, and envelope counts rather than content inspection.

### Acceptance Criteria

- Metadata schema is documented and covered by tests for omission of plaintext/private-key/direct-endpoint material.
- Retention/deletion tests cover ciphertext envelope tombstones and per-device cursor behavior.
- Abuse/rate-limit tests work without plaintext inspection.

## Proposition 5 (Rank 5): Delivery State and Diagnostic Semantics

### What Changes

- Model delivery states around node acceptance, live fanout, pending delivery, and catch-up rather than peer connectivity.
- Keep new UX copy/control changes behind explicit user approval.

### How It Works

1. Sender-visible success means durable envelope acceptance.
2. Live fanout failures become pending delivery states with bounded retry/catch-up semantics.
3. Later-active devices use per-device cursor replay and dedupe to converge.
4. Delivery diagnostics explain policy blocks, missing bootstrap, message-node availability, and replay/catch-up failures without adding DM preflight/troubleshooter controls or asking normal users to configure routers or LAN discovery.

### Acceptance Criteria

- Reason codes are deterministic and do not reference direct peer connectivity.
- Pending delivery/catch-up states preserve accepted encrypted-envelope history.
- UX-facing changes are proposed and explicitly approved before implementation.

## Delivery Ownership

- This document is the option catalog and trade-off authority.
- Sequencing and task ownership are canonical in `docs/planning/infra-free-dm-connectivity-execution-plan.md`.
- Architecture boundaries are canonical in `docs/architecture/04-communication-networking-layer-plan.md`.

## Non-Goals

- Server-readable DM content.
- Private-key upload, escrow, or server custody.
- Unencrypted DM mailboxing or plaintext relay behavior.
- User-to-user direct DM LAN/WAN transport, endpoint hints/cards, connectivity preflight, pairing QR/manual-code bootstrap, WAN wizard, or parallel dial.
- Reworking unrelated app-layer features while implementing DM delivery changes.

## Related Documents

- `AGENTS.md`
- `docs/product/01-mvp-plan.md`
- `docs/product/03-clarifications.md`
- `docs/product/04-dependencies-risks.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/infra-free-dm-connectivity-execution-plan.md`
- `docs/planning/05-iteration-log.md`
