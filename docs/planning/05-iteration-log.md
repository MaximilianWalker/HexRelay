# HexRelay Iteration Log

## Document Metadata

- Doc ID: iteration-log
- Owner: Delivery maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/planning/05-iteration-log.md`

## Quick Context

- Primary edit location for project-level delivery changes across iterations.
- Do not duplicate sprint task detail here; link to iteration boards when needed.
- Latest meaningful change: 2026-05-20 added the destructive singleton server storage migration and removed the multi-server API database dimension.

## Purpose

- Capture project-level delivery changes that do not fit cleanly into a single sprint board update.
- Keep an auditable history of scope, sequencing, and status decisions.

## Entry Format

- Date (UTC)
- Area affected
- Change summary
- Rationale
- Linked docs updated

## Log Entries

### 2026-05-20 (singleton server storage cleanup)

- Area affected: API database schema, server repository code, server-channel permissions/messages, dev seed fixtures, runtime REST contract, and server-to-server architecture docs.
- Change summary:
  - Added destructive migration `0026_single_server_authority` to replace the multi-server `servers` partition with one `local_server` singleton plus server-local membership/channel/role/message tables.
  - Removed `server_id` from API repository storage helpers, server-channel permission checks, dev seed fixture schemas, and cross-server local DB tests.
  - Kept API route `server_id` semantics as the connected server id and retained non-local path rejection.
  - Updated architecture/product planning docs from "transitional schema cleanup pending" to "singleton local-server storage implemented."
- Rationale:
  - The approved model is one user-facing server per runtime authority. Keeping many server authorities inside one API database would preserve the centralized shape the architecture decision rejected.
- Linked docs updated:
  - `docs/architecture/adr-0004-server-authority.md`
  - `docs/architecture/01-system-overview.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/navigation-implementation-plan.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/contracts/runtime-rest.openapi.yaml`

### 2026-05-20 (server authority lock)

- Area affected: Server architecture, server membership authorization, runtime REST contract, navigation planning, and Create/Join Server prerequisites.
- Change summary:
  - Added `docs/architecture/adr-0004-server-authority.md` as the accepted authority that one user-facing server maps to one separately runnable server runtime.
  - Clarified that the user app can aggregate or supervise several servers, but it is not the authority for many unrelated servers inside one API database.
  - Initially marked `servers` and `server_memberships` storage as local-server persistence pending cleanup; the later singleton storage entry on this date supersedes that implementation caveat.
  - Scoped API-facing server membership and directory semantics to the connected server id.
- Rationale:
  - The previous scaffold could be read as a centralized many-servers-in-one-API model, which conflicts with the self-hostable/decentralized product model the user approved.
- Linked docs updated:
  - `docs/architecture/adr-0004-server-authority.md`
  - `docs/architecture/01-system-overview.md`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/07-ui-navigation-spec.md`
  - `docs/planning/navigation-implementation-plan.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/reference/glossary.md`

### 2026-05-15 (DM workspace delivery approval package)

- Area affected: DM workspace delivery, E2EE private-message history/send behavior, quality-audit routing, and approval-gated Web implementation planning.
- Change summary:
  - Added `docs/planning/dm-workspace-delivery-implementation-plan.md` as the approval-pending implementation plan for `QA-17-20260514-dm-workspace-send-not-wired`.
  - Defined proposed DM workspace flow, controls, copy baseline, required state mapping, implementation slices, validation, evidence expectations, and `DMW-APP-*` approval decisions.
  - Kept all runtime DM workspace UI implementation blocked until explicit user approval of flow, copy, controls, and behavior.
- Rationale:
  - The selected quality finding is valid but UX-facing; the smallest mergeable prerequisite is a plan-only approval package rather than unapproved Web behavior.
- Linked docs updated:
  - `docs/planning/dm-workspace-delivery-implementation-plan.md`
  - `docs/product/08-screen-state-spec.md`
  - `docs/product/README.md`
  - `docs/planning/README.md`
  - `docs/operations/quality-audits/17-ux-product-quality.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.6 navigation approval gate hardening)

- Area affected: Servers Hub, Contacts Hub, desktop dual-mode server navigation, mobile navigation, and approval-gated Web implementation planning.
- Change summary:
  - Added an approval response template with explicit `NAV-APP-*` decision values for the runtime UI implementation PRs to cite.
  - Added a pre-implementation gate checklist covering approval evidence, slice scope, validation commands, evidence path reservation, and exclusion of unapproved adjacent UX behavior.
  - Kept the selected `T4.6.1` through `T4.6.4` cluster plan-only; no runtime UI behavior is approved or implemented by this change.
- Rationale:
  - The existing navigation plan described the pending decisions, but future automation and contributors still needed one copy-pasteable approval format and a deterministic start gate before any Web slice begins.
- Linked docs updated:
  - `docs/planning/navigation-implementation-plan.md`
  - `docs/planning/README.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.6 navigation plan-only approval package)

- Area affected: Servers Hub, Contacts Hub, desktop dual-mode server navigation, mobile navigation, and Iteration 2 planning status.
- Change summary:
  - Added `docs/planning/navigation-implementation-plan.md` as the approval-pending implementation plan and evidence authority for `T4.6.1` through `T4.6.4`.
  - Mapped the plan to the existing navigation spec, screen/state spec, configuration defaults, sprint-board acceptance criteria, and verification matrix.
  - Kept all runtime UI implementation blocked until explicit user approval of flow, copy, controls, and behavior.
- Rationale:
  - The selected navigation cluster is UX-facing and cannot be implemented under the repository UX approval gate, so the smallest mergeable prerequisite is a plan-only PR that defines the approval package and future implementation slices.
- Linked docs updated:
  - `docs/planning/navigation-implementation-plan.md`
  - `docs/planning/README.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.5.4 E2EE group DM ciphertext and recovery closeout)

- Area affected: E2EE group DM encrypt/decrypt, missing-key recovery coverage, crypto conformance evidence, and Iteration 2 planning status.
- Change summary:
  - Added a `communication-core` group session ring that decrypts only with matching group session ids and returns `session_key_missing` when a post-rekey envelope arrives before the next member session key.
  - Added regressions proving group DM payloads are ciphertext envelopes, encrypted results do not serialize plaintext, one-to-one sessions are rejected from the group session ring, and decrypt succeeds after the rekeyed member session is inserted.
  - Marked `T4.5.4` done on the Iteration 2 sprint board.
- Rationale:
  - The missing-key path was the smallest remaining mergeable prerequisite after the T4.5.1-T4.5.3 bootstrap, one-to-one rotation, and group rekey work had already landed.
- Linked docs updated:
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `evidence/iteration-02/messaging-e2ee/2026-05-13/`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.5.3 E2EE group DM bootstrap and rekey closeout)

- Area affected: E2EE group DM session bootstrap, membership key updates, crypto conformance coverage, Iteration 2 planning status, and verification evidence.
- Change summary:
  - Added `communication-core` group session bootstrap support that derives usable client sessions only for current group participants.
  - Tightened group rekey flow so membership-change plans expose added/removed identity sets and reject removed identities before deriving the next client session.
  - Recorded the previously landed T4.5.2 one-to-one rotation prerequisite and marked `T4.5.2` plus `T4.5.3` done on the Iteration 2 sprint board.
- Rationale:
  - The selected `T4.5.x` E2EE baseline remains too large for one PR; closing group bootstrap/rekey as a standalone prerequisite leaves only T4.5.4 group payload failure-recovery coverage open.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `evidence/iteration-02/messaging-e2ee/2026-05-13/`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.5.1 E2EE one-to-one session bootstrap closeout)

- Area affected: E2EE one-to-one DM session bootstrap, crypto conformance coverage, Iteration 2 planning status, and verification evidence.
- Change summary:
  - Confirmed `communication-core` already establishes one-to-one E2EE sessions from Ed25519-signed identity bootstrap material and X25519 ephemeral agreement.
  - Added a regression proving signed bootstrap material is bound to the exact session context, including thread and generation, before client session derivation.
  - Marked `T4.5.1` done on the Iteration 2 sprint board and added the corresponding messaging E2EE evidence artifact.
- Rationale:
  - The selected `T4.5.x` E2EE baseline is too large for one PR; closing the one-to-one session-bootstrap prerequisite first keeps the change mergeable while leaving 1:1 payload rotation/catch-up and group E2EE work to T4.5.2-T4.5.4.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `evidence/iteration-02/messaging-e2ee/2026-05-13/`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T3.3.2/T4.3.3/T4.3.4 profile-device sync closeout)

- Area affected: Presence convergence, server-channel profile-device hydration, shared adapter evidence, Iteration 2 planning status, and verification evidence.
- Change summary:
  - Confirmed presence and server-channel runtime paths already use the shared `ServerClientTransport` adapter/provenance helpers for their current server-client dispatch surfaces.
  - Confirmed existing websocket regressions cover presence active-device fanout, late-device hydration, missed offline transition replay, server-channel late-device hydration, reconnect dedupe, and read-denied replay exclusion.
  - Added the missing profile-device sync evidence artifact and marked `T3.3.2`, `T4.3.3`, and `T4.3.4` done on the Iteration 2 sprint board.
- Rationale:
  - The selected `T4.2-T4.3` cluster spans several already-delivered runtime surfaces. Closing the evidence/status gap is the smallest mergeable prerequisite before selecting later Iteration 2 work.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `evidence/iteration-02/profile-device-sync/2026-05-13/`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.3.2 backend fanout closeout and UX split)

- Area affected: Server-channel realtime fanout, reconnect duplicate coverage, Iteration 2 planning status, and UX approval planning.
- Change summary:
  - Strengthened the server-channel websocket integration test so reconnect assertions reject duplicate create, update, or delete events for the same message.
  - Marked the backend `T4.3.2` websocket fanout acceptance evidence done on the Iteration 2 board.
  - Split server-channel optimistic send UI into `T4.6.5` with a plan-only proposal in the screen/state spec.
- Rationale:
  - The selected task combined backend realtime reliability with UX behavior. Backend acceptance criteria can close now, while optimistic UI implementation remains blocked by the repository-wide explicit UX approval policy.
- Linked docs updated:
  - `docs/product/08-screen-state-spec.md`
  - `docs/product/README.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-13 (T4.3.1/T4.4.1 server-channel message and permission closeout)

- Area affected: Server-channel REST messaging, permission bypass coverage, and Iteration 2 planning status.
- Change summary:
  - Confirmed the runtime already exposes server-channel message list/create/edit/delete routes with reply and mention metadata, pagination, tombstones, contract coverage, and integration tests.
  - Added focused API regression coverage proving configured role read denial blocks channel history, and configured no-send roles cannot create, edit, or delete server-channel messages even when the message author is otherwise valid.
  - Marked `T4.3.1` and `T4.4.1` done on the Iteration 2 sprint board.
- Rationale:
  - The selected parent cluster spans API schema, realtime fanout, adapterization, convergence, and permission middleware. Closing the message/permission prerequisite first keeps the PR mergeable while removing stale board state that caused the selector to re-open delivered backend work.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-05-11 (T4.1.9 DM realtime dispatch summaries)

- Area affected: Realtime DM envelope dispatch, API dispatch logging, active-device fanout evidence, networking architecture docs, product DM delivery notes, and verification matrix.
- Change summary:
  - Added an internal realtime dispatch summary for `dm.envelope.dispatched` publication.
  - Classified target profile-device outcomes as queued-to-verified-websocket, pending/no-connection, pending/unverified-device-binding, pending/saturated-queue, and stale connection cleanup count.
  - Added API-side structured logging for the internal realtime summary returned by the message-server dispatch endpoint.
  - Added tests covering targeted delivery summaries, pending target reasons, saturated outbound queues, stale websocket cleanup, and the internal dispatch response body.
- Rationale:
  - T4.1.9 needed clearer backend observability for active-device fanout without changing UX or weakening the ack-backed delivery contract.
  - The implementation keeps final delivery tied to `dm.envelope.ack`; live websocket queueing is observable but not treated as read state or final recipient delivery.
- Linked docs updated:
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/README.md`

### 2026-05-11 (T4.1.8 DM metadata retention and abuse controls)

- Area affected: API DM encrypted-envelope delivery, persistence migrations, data lifecycle docs, configuration defaults, Iteration 2 sequencing, and migration evidence.
- Change summary:
  - Added configurable DM dispatch, catch-up, ack, and authenticated server-forward rate limits.
  - Added configurable fanout delivery-log and outbound forwarding-log retention windows.
  - Added retention purge behavior that deletes expired replay/forwarding metadata without deleting canonical ciphertext DM history.
  - Added retention indexes for fanout delivery-log `created_at` and outbound forwarding state/age scans.
  - Added tests for retention deletion semantics, sender-scoped dispatch rate limiting, and config parsing.
- Rationale:
  - T4.1.8 needed executable abuse controls and deterministic metadata deletion behavior before moving on to additional delivery observability or realtime routing work.
  - The implementation keeps abuse controls independent of plaintext inspection and avoids reintroducing endpoint hints, LAN/WAN addresses, pairing payloads, or direct user-to-user DM transport state.
- Linked docs updated:
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `evidence/migrations/0024_dm_delivery_metadata_retention_indexes.md`
  - `docs/README.md`

### 2026-05-11 (server-to-server policy graph architecture lock)

- Area affected: Communication networking architecture, product discovery roadmap, PRD discovery requirements, product clarifications, glossary terms, and risk register.
- Change summary:
  - Locked the server-to-server P2P architecture as a dynamic policy graph rather than a mandatory global network.
  - Clarified that user identity is portable and not assigned to a single primary server.
  - Defined server roles for origin, delivery, relay, discoverable, private, local-only, and LAN-only behavior.
  - Separated discovery, peering, relay, delivery, and encrypted storage permissions.
  - Added user-consented server introductions as descriptor-scoped candidate-peer creation, not automatic trust or public exposure.
  - Selected algorithm direction: static peers and signed invites first, mDNS/DNS-SD for LAN-only discovery, signed rendezvous registries next, Kademlia descriptor lookup later, HyParView peer sampling later, Plumtree gossip only for low-sensitivity server metadata, policy-constrained route selection, weighted relay selection, and store-and-forward encrypted-envelope reliability.
- Rationale:
  - HexRelay needs private online servers, local-only/LAN-only operation, and small self-created P2P networks to be first-class without reintroducing recipient-device DM transport or server-readable DM content.
- Linked docs updated:
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/architecture/01-system-overview.md`
  - `docs/architecture/README.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/README.md`
  - `docs/reference/glossary.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-08 (DM server-bypassing surface retirement)

- Area affected: Product DM delivery policy, runtime REST/realtime contracts, web contact/DM surfaces, Iteration 2 sequencing, local testing, verification evidence, and CI guardrails.
- Change summary:
  - Superseded the earlier optional server-bypassing optimization direction and locked MVP DMs to server-to-server P2P E2EE envelope delivery only.
  - Retired recipient-device pairing QR/manual-code bootstrap, connectivity preflight, LAN discovery, endpoint cards, WAN wizard, and parallel dial from runtime, web, contracts, docs, tests, and guardrails.
  - Reserved QR scope for server invites and trusted device-link/restore flows rather than contact or DM bootstrap.
  - Reframed `T4.1.5`, `T4.1.6`, and `T4.1.11` as server-bypassing DM surface retirement tasks with negative conformance evidence.
- Rationale:
  - Normal-user DMs must be zero-config and reliable through ciphertext-only message-server delivery without router, LAN, or peer-dial setup.
- Linked docs updated:
  - `AGENTS.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/08-screen-state-spec.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/architecture/01-system-overview.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/contracts/README.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/testing/01-mvp-verification-matrix.md`

### 2026-05-08 (DM encrypted-envelope delivery baseline pivot)

- Area affected: Product DM delivery policy, architecture trust boundaries, Iteration 2 sequencing, verification evidence, and CI guardrails.
- Change summary:
  - Replaced the mandatory server-bypassing DM transport baseline with server-to-server/message-server E2EE envelope delivery.
  - Locked the security boundary that DM plaintext and private keys remain client/device-only while message servers may carry/store ciphertext envelopes plus minimal delivery metadata.
  - Reframed LAN/WAN server-bypassing work as non-baseline and later retired it from MVP scope.
  - Updated Iteration 2 sequencing so `T4.1.7` becomes encrypted-envelope message-server delivery and WAN/parallel dial work moves behind explicit re-scoping.
  - Replaced the broad server-bypassing policy guardrail direction with unsafe-semantics guardrails for plaintext, private-key custody, unencrypted mailboxing, and plaintext relay behavior.
- Rationale:
  - Strict server-bypassing transport cannot provide reliable zero-config cross-WAN DMs for normal users; E2EE envelope delivery preserves privacy while making baseline delivery usable.
- Linked docs updated:
  - `AGENTS.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/architecture/01-system-overview.md`
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/planning/turn-nat-test-profile.md`
  - `docs/contracts/realtime-events.asyncapi.yaml`
  - `docs/contracts/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/README.md`
  - `docs/product/README.md`
  - `docs/planning/README.md`
  - `docs/reference/glossary.md`
  - `scripts/validate-dm-transport-policy.sh`
  - `.github/workflows/ci.yml`

### 2026-05-08 (T4.1.6 LAN discovery fast-path closeout)

- Area affected: DM LAN discovery validation, ephemeral peer snapshots, runtime REST contract, web API helpers, and Iteration 2 sequencing.
- Change summary:
  - Added shared core LAN endpoint validation for private or link-local IPv4-literal addresses with non-zero ports.
  - Tightened LAN discovery announcements to reject loopback, public-routable, DNS-hostname, relay-oriented, and stale/invalid endpoint hints while keeping LAN presence in memory only.
  - Restricted LAN peer listing and preflight LAN priority to trusted accepted-friend or shared-server relationships instead of arbitrary `anyone` policy matches.
  - Added explicit `expires_at` and `ttl_seconds` metadata to LAN discovery responses and peer summaries.
  - Kept preflight LAN priority deterministic by returning `preflight_ok_lan` only for fresh local-only peer snapshots. This behavior is superseded by the later server envelope pivot.
  - Marked `T4.1.6` done and, at closeout time, advanced the recommended Iteration 2 sequence to `T4.1.7` WAN setup work, with `T4.1.8` available in parallel. This recommendation is superseded by the later 2026-05-08 encrypted-envelope delivery baseline pivot above.
- Rationale:
  - `T4.1.6` acceptance required same-LAN server-bypassing improvements without introducing infrastructure fallback or durable LAN discovery state; this is now historical only.
- Linked docs updated:
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/contracts/README.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/testing/01-mvp-verification-matrix.md`

### 2026-05-08 (T4.1.5 DM connectivity preflight/troubleshooter closeout)

- Area affected: DM connectivity preflight validation, private-message troubleshooting UX, runtime REST contract, and Iteration 2 sequencing.
- Change summary:
  - Tightened preflight peer identity validation to the shared identity-id shape and expanded backend coverage for local bind denial, peer reachability failure, LAN-ready preference, and deterministic remediation text.
  - Enforced cookie-auth CSRF parity on the preflight POST while keeping bearer-auth diagnostics unchanged.
  - Added a web preflight API helper and private-message troubleshooter card that reported pairing availability, recipient-device status, server-bypassing transport, stable reason labels, and ordered remediation steps before enabling the composer.
  - Stored validated imported pairing metadata in session-scoped browser storage so the private-message preflight could distinguish missing pairing from ready/blocked server-bypassing outcomes without backend rendezvous.
  - Marked `T4.1.5` done and advanced the recommended Iteration 2 sequence to `T4.1.6` LAN discovery fast path.
- Rationale:
  - `T4.1.5` acceptance required failed server-bypassing connections to map to deterministic reason codes with actionable in-product remediation; this is now superseded by server-to-server P2P envelope delivery.
- Linked docs updated:
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/product/02-prd.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-07 (T4.1.4 signed DM pairing closeout)

- Historical/superseded note: this DM pairing/bootstrap slice was retired by the 2026-05-08 server-to-server P2P E2EE envelope pivot.

- Area affected: DM pairing envelope schema, contacts pairing UX, runtime REST contract, and Iteration 2 sequencing.
- Change summary:
  - Added inviter identity-key material and SHA-256 key server id output to signed DM pairing envelopes and import responses.
  - Kept the short code as an out-of-band verification code and added QR/link/manual-code import/export UX for no-rendezvous bootstrap.
  - Added backend coverage for identity-key import output and self-import rejection alongside existing replay/expiry/tamper tests.
  - Marked `T4.1.4` done and clarified QR/link/manual-code terminology across product and execution docs.
- Rationale:
  - `T4.1.4` requires pairing to exchange identity and endpoint bootstrap material without backend rendezvous while keeping replay/expiry and authenticity checks deterministic.
- Linked docs updated:
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-07 (T4.0.2 transport adapter rollout closeout)

- Historical/superseded note: the DM server-bypassing adapter work was retired by the 2026-05-08 server-to-server P2P E2EE envelope pivot; server/channel server-client adapter lessons remain relevant.

- Area affected: Shared communication transport adapters, DM direct runtime paths, server-channel dispatch, and presence dispatch.
- Change summary:
  - Added shared server-bypassing dispatch bootstraps in `communication-core` alongside the existing server-client dispatch bootstrap.
  - Routed ready DM preflight, successful DM parallel dial, and accepted DM active fanout through server-bypassing adapter boundaries while preserving server-bypassing response semantics.
  - Removed the outer current-thread presence bypass so presence edge dispatch consistently enters `ServerClientTransport`; the current-thread workaround now remains inside the local dispatch sender.
  - Expanded adapter conformance coverage for client send/connect and reran DM, presence, clippy, formatting, and server-bypassing policy validations.
- Rationale:
  - `T4.0.2` acceptance requires existing call paths to route through adapter interfaces without behavior regression; the remaining gaps were DM server-bypassing paths and a presence runtime bypass around the server-client adapter.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-07 (PH-07 local runtime documentation and adoption closeout)

- Area affected: Local runtime testing operations, runtime config reference safety notes, testing evidence discoverability, and documentation routing.
- Change summary:
  - Added `docs/operations/local-runtime-testing-quickstart.md` for clean-checkout setup, fixture seed/reset, host-process runtime profiles, Docker runtime/network simulation, evidence commands, and troubleshooting.
  - Tightened `API_ENABLE_DEV_TESTING` and `REALTIME_ENABLE_DEV_FAULTS` safety notes in the runtime config reference.
  - Added local runtime testing evidence coverage to the MVP verification matrix and testing docs index.
  - Marked PH-07 complete in the local runtime testing plan and refreshed documentation routers.
- Rationale:
  - PH-06 made the harness executable and validated; PH-07 makes it adoptable by future development work without relying on PR history or scattered command notes.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/operations/README.md`
  - `docs/operations/local-runtime-testing-quickstart.md`
  - `docs/reference/runtime-config-reference.md`
  - `docs/testing/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-07 (PH-06 local runtime validation closeout)

- Area affected: Local runtime validation, fixture invariants, web testing helpers, runtime/network smoke entrypoints, and smoke evidence capture.
- Change summary:
  - Added stronger fixture invariant coverage for `dm-basic`, `contacts-edge`, and `server-chat` local testing scenarios.
  - Added focused web helper coverage for fixture persona activation events and per-persona runtime session isolation.
  - Added `scripts/test-runtime.mjs` and `scripts/test-network.mjs` entrypoints while preserving `npm run test:runtime` as the full Docker runtime/network smoke.
  - Extended `scripts/runtime-docker.mjs smoke` with `--scope all|runtime|network` and optional `--evidence-dir` output for scenario config, runtime status before/after, event log, and verdict files.
- Rationale:
  - PH-05 made runtime/network simulation executable; PH-06 closes the validation/evidence loop so later DM/realtime work has a deterministic local harness instead of console-only smoke output.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/planning/05-iteration-log.md`

### 2026-05-07 (PH-05 Docker runtime and network simulation)

- Area affected: Local runtime testing, Docker runtime/network simulation, and script discoverability.
- Change summary:
  - Added `infra/docker-compose.runtime-test.yml` for containerized Alice/Bob runtimes with API, realtime, and web containers.
  - Added `scripts/runtime-docker.mjs` and root commands `npm run runtime:docker` and `npm run test:runtime`.
  - Added shared runtime tsconfig generation for containerized Next dev instances.
  - Split per-server infra networks from the shared simulation network so offline/partition profiles do not sever Postgres/Redis/MinIO connectivity or leave an alternate Alice/Bob peer path.
  - Validated Docker offline and partition apply/reset flows through `npm run test:runtime` with app-level Alice/Bob API reachability assertions.
  - Added Docker-only Toxiproxy apply/reset for peer-link latency and timeout-based loss profiles.
  - Added realtime dev-fault hooks and `flaky-mobile` app-fault apply/reset support.
  - Added a separate `runtime-network-smoke` CI job for the heavier Docker runtime/network smoke.
  - Documented the hybrid model: host-process normal development plus Docker runtime/network tests.
- Rationale:
  - PH-05 Docker network controls need real container targets, while normal Tauri/web/Rust development should keep the faster host-process loop.
- Linked docs updated:
  - `README.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `scripts/README.md`
  - `infra/README.md`

### 2026-05-07 (release packaging direction)

- Area affected: Release packaging, runtime deployment modes, dedicated-server deployment guidance, and MVP runtime planning.
- Change summary:
  - Added the canonical release packaging authority for Windows/Linux desktop artifacts, dedicated-server artifacts, and code signing expectations.
  - Locked Tauri as the default desktop shell for release planning unless a later explicit decision replaces it.
  - Clarified that dedicated server mode is a separate service/package family and is not bundled by default into the desktop installer.
- Rationale:
  - MVP-end release planning needs deterministic artifact boundaries so desktop users, operators, and future CI packaging work do not drift into mixed installer/service assumptions.
- Linked docs updated:
  - `docs/operations/03-release-packaging.md`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/operations/02-dedicated-server-deployment.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/README.md`
  - `docs/operations/README.md`

### 2026-05-06 (local runtime testing fixture and network profile slice)

- Area affected: Local fixture catalog, API seed tooling, network simulation profiles, and local runtime testing plan.
- Change summary:
  - Added `contacts-edge` and `server-chat` scenario fixtures for pending/restricted contact states, shared servers, memberships, channels, mentions, and replies.
  - Extended the dev seed parser and transactional seeding path for invite, server, membership, channel, and server channel message fixture data.
  - Added bundled network simulation profile JSON files plus `npm run validate:network-profiles` for the PH-05 schema/validation slice.
  - Added `npm run network` plus Windows and Unix wrappers for applying/resetting network profile state, with Docker container-target support and fail-safe handling for current host-process runtime instances.
- Rationale:
  - The remaining PH-01 scenarios unblock broader local exploratory testing, and PH-05 needs validated profile definitions before network apply/reset wrappers are added.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `scripts/README.md`

### 2026-05-05 (local runtime testing multi-instance profiles)

- Area affected: Runtime scripts, web dev-server isolation, local runtime testing profiles, and local runtime testing plan.
- Change summary:
  - Added `single`, `dual`, and `triple` runtime profile JSON files plus a shared validator/normalizer.
  - Extended Windows and Unix runners to start named API/realtime/web instances with per-instance ports, logs, web env, and tracked runtime state.
  - Added cross-platform `status` and `stop` commands that inspect and stop only tracked `.local-run` processes.
  - Isolated Next.js dev build directories per runtime instance so multiple web dev servers can run side by side.
- Rationale:
  - Manual local DM and multi-server testing needs deterministic multi-instance startup without hand-editing ports or killing broad process names.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/README.md`
  - `README.md`

### 2026-05-05 (local runtime testing reset validation)

- Area affected: Local development database reset workflow and local runtime testing plan.
- Change summary:
  - Ran the explicit destructive local reset smoke with `npm run reset-dev-db -- --yes --profile dm-basic`.
  - Verified the reset database by rerunning `npm run seed -- --profile dm-basic --json` and confirming the expected `dm-basic` fixture counts.
  - Marked PH-02 reset tooling done in the local runtime testing plan.
- Rationale:
  - The guarded reset workflow was implemented but intentionally left open until a user-approved destructive local DB reset confirmed the full reset and reseed path.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`

### 2026-05-05 (local runtime testing web picker)

- Area affected: Web settings, local fixture persona activation, and local runtime testing plan.
- Change summary:
  - Added web API client methods for the dev testing profile/session endpoints.
  - Added a dev-only Settings card that lists seeded testing profiles, shows purpose and active-session state, and activates real API-backed sessions into persona/session storage.
  - Added Vitest coverage for dev testing API calls and deterministic fixture persona upserts.
- Rationale:
  - Seeded local profiles need one-click browser activation without drifting into browser-only fake user state.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`

### 2026-05-05 (local runtime testing dev sessions)

- Area affected: API runtime config, dev-only testing API, and local runtime testing plan.
- Change summary:
  - Added `API_ENABLE_DEV_TESTING=false` as the production-disabled gate for local fixture/session testing endpoints.
  - Added dev-only API routes for listing testing profiles and issuing DB-backed fixture sessions/cookies for seeded identities.
  - Added targeted API tests for disabled-by-default behavior and real seeded profile session validation.
- Rationale:
  - The web testing profile picker needs a safe backend bootstrap path that creates real local API sessions instead of browser-only fake users.
- Linked docs updated:
  - `docs/reference/runtime-config-reference.md`
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/README.md`

### 2026-05-05 (local runtime testing reset slice)

- Area affected: Local development database reset workflow and seed tooling.
- Change summary:
  - Added `reset_dev_db` as a Rust CLI that requires `--yes`, refuses production/non-local database targets, resets the local dev schema, reruns migrations, and reseeds the selected profile.
  - Added `npm run reset-dev-db` with Windows and Unix wrappers.
  - Updated local runtime testing docs to track the reset wrapper slice as implemented but still awaiting explicit destructive reset smoke validation.
- Rationale:
  - Repeatable local runtime tests need a guarded way to return the database to a known fixture state without manual Postgres commands.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `README.md`
  - `scripts/README.md`

### 2026-05-05 (local runtime testing seed slice)

- Area affected: Local fixture catalog, API seed tooling, and workspace scripts.
- Change summary:
  - Added the initial `dm-basic` fixture catalog for Alice/Bob local DM testing.
  - Added `services/api-rs/src/bin/seed_dev.rs` and `services/api-rs/src/dev_seed.rs` for transactional local fixture seeding with production and non-local database guards.
  - Added `npm run seed`, Windows/Unix seed wrappers, and seed fixture validation tests.
- Rationale:
  - The web profile picker and multi-instance runtime work need real local API identities, sessions, contacts, policies, profile devices, and DM history before they can be useful. Then-current endpoint-card fixture assumptions were retired by the 2026-05-08 server-to-server P2P E2EE envelope pivot.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `scripts/README.md`
  - `README.md`

### 2026-05-04 (local runtime testing plan)

- Area affected: Local testing workflow, fixture profile planning, multi-instance runtime planning, and network simulation planning.
- Change summary:
  - Added `docs/planning/local-runtime-testing-plan.md` as the canonical planning authority for precreated local testing profiles, seeded fixture data, dev session bootstrap, multi-instance runtime profiles, and local network simulation.
  - Routed related planning, testing, operations, KPI/SLO, TURN/NAT, and docs index entries to the new authority without duplicating runtime config details or verification evidence rules.
  - Captured the intended network simulation technology stack: Docker network controls, Docker-only peer proxies, dev-only app-level fault injection, and browser/runtime isolation.
- Rationale:
  - Local manual and automated testing now need a repeatable profile/fixture/runtime plan before implementation starts, especially after PR #96 added workspace DM UI and Windows runner baseline improvements.
- Linked docs updated:
  - `docs/planning/local-runtime-testing-plan.md`
  - `docs/planning/README.md`
  - `docs/testing/README.md`
  - `docs/operations/README.md`
  - `docs/planning/kpi-slo-test-profile.md`
  - `docs/planning/turn-nat-test-profile.md`
  - `docs/README.md`
  - `README.md`

### 2026-04-11 (T4.1.4 DM pairing web slice)

- Historical/superseded note: this DM pairing QR/link/manual-code web slice was retired by the 2026-05-08 server-to-server P2P E2EE envelope pivot.

- Area affected: Iteration 2 DM pairing/bootstrap web delivery.
- Change summary:
  - Added DM pairing API client methods in `apps/web/lib/api.ts` for pairing-envelope create/import.
  - Added `apps/web/lib/dm-pairing.ts` helper utilities plus tests for `hexrelay://dm-pairing/...` link build/parse behavior.
  - Added contacts-page UI for DM pairing share/import with QR rendering, verification-code display, envelope-link copy, and import result feedback.
  - Marked `T4.1.4` in progress on the Iteration 2 sprint board.
- Rationale:
  - `T4.1.4` is the first audited story whose Web half was genuinely missing. This slice delivers the smallest coherent web implementation on top of the already-shipped backend pairing flow.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-11 (T4.1.3 DM transport guardrail closeout)

- Area affected: Iteration 2 DM transport policy enforcement and CI guardrails.
- Change summary:
  - Confirmed the runtime already enforced the then-current server-bypassing DM transport through `communication-core` routing, DM endpoint-hint validation, and existing DM connectivity/runtime tests.
  - Expanded `scripts/validate-dm-transport-policy.sh` so the CI guardrail now scans the actual DM runtime transport callsite plus DM-related workflow/config surfaces (`.github/workflows/ci.yml`, runtime config docs, and service config files) instead of only a narrow Rust filename subset.
  - Marked `T4.1.3` done on the Iteration 2 sprint board.
- Rationale:
  - The story was only partially delivered: runtime behavior already matched policy, but the CI guardrail still missed forbidden config-style regressions. Widening the check closes the acceptance-criteria gap without inventing new transport behavior.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-11 (T4.1.2 DM privacy-policy closeout)

- Area affected: Iteration 2 DM privacy-policy delivery traceability.
- Change summary:
  - Confirmed the backend/runtime already delivers the `T4.1.2` acceptance criteria through the existing DM privacy-policy read/update endpoints, default `friends_only` behavior, persisted per-identity override state, and recipient-policy enforcement across DM preflight/fanout/parallel-dial paths.
  - Added one missing explicit integration assertion in `services/api-rs/src/tests/integration/dm_policy_tests.rs` proving that `same_server` can be set via `POST /dm/privacy-policy` and read back unchanged.
  - Marked `T4.1.2` done on the Iteration 2 sprint board.
- Rationale:
  - The runtime behavior was already present; the real gap was stale planning status plus one missing explicit readback regression for the `same_server` policy value.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-11 (T4.1.1 DM thread/history closeout)

- Area affected: Iteration 2 DM thread/history delivery traceability.
- Change summary:
  - Confirmed the backend/runtime already delivers the `T4.1.1` acceptance criteria through the existing DM thread list, DM message history pagination, and mark-read endpoints plus integration coverage.
  - Added one missing explicit integration assertion in `services/api-rs/src/tests/integration/dm_threads_tests.rs` proving the returned thread list includes a `group_dm` item with the expected participant set.
  - Marked `T4.1.1` done on the Iteration 2 sprint board.
- Rationale:
  - The runtime behavior was already present; the real gap was stale planning status plus one missing explicit regression around the `group_dm` response shape.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-11 (T4.0.2 server-client adapter boundary slice)

- Area affected: Iteration 2 communication adapter rollout and planning traceability.
- Change summary:
  - Added shared adapter primitives in `crates/communication-core/src/transport/mod.rs`: `UnsupportedDirectPeerTransport`, `NodeDispatch`, and `DispatchingServerClientTransport`.
  - Routed the current production server-client send paths in `services/api-rs/src/domain/server_channels/realtime.rs` and `services/realtime-rs/src/domain/presence.rs` through the shared dispatching adapter instead of service-local `ServerClientTransport` implementations.
  - Extended `communication-core` router tests to cover shared dispatching-adapter mode enforcement and payload forwarding semantics.
  - Marked `T4.0.2` as in progress on the Iteration 2 sprint board.
- Rationale:
  - The currently exercised production adapter path is server-client transport, not server-bypassing transport. Centralizing that path first closes a real duplication gap and moves `T4.0.2` forward without inventing premature DM transport machinery.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-11 (T4.0.1 shared communication-layer closeout)

- Area affected: Iteration 2 communication-layer foundation and planning traceability.
- Change summary:
  - Replaced hand-rolled server-client provenance construction in current presence and server-channel integrations with `PolicyEngine::build_provenance(...)` from `crates/communication-core`.
  - Extended `communication-core` policy tests to cover deterministic provenance for `server_channel` and `presence` alongside the existing DM route case.
  - Marked `T4.0.1` done on the Iteration 2 sprint board and added the expected evidence path under `evidence/iteration-02/networking-layer/`.
- Rationale:
  - The shared communication-layer boundary was already mostly implemented; this closes the cleanest remaining gap by making current integrations consume the shared provenance logic instead of duplicating it, and by aligning planning status with the shipped core foundation.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/README.md`
  - `docs/planning/iterations/README.md`
  - `docs/README.md`

### 2026-04-10 (iteration-board closeout for delivered presence and discovery work)

- Area affected: Iteration 2 planning status and delivery traceability.
- Change summary:
  - Marked `T3.3.1` as done on the Iteration 2 sprint board with merged evidence from PRs `#53-#54` and the Redis-backed reconnect/hydration coverage already present in `services/realtime-rs/src/tests/ws_transport_tests.rs`.
  - Marked `T3.4.1` as done on the Iteration 2 sprint board with merged evidence from PR `#52` and follow-up discovery parity/policy hardening that already landed in the API/runtime contract and integration tests.
  - Removed the stale `In Progress` bookkeeping entry that no longer matched the merged codebase.
- Rationale:
  - Planning status should match repository reality; leaving delivered stories marked as pending or in-progress distorts dependency sequencing and makes next-story selection worse.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-04-10 (docs-governance cleanup for freshness policy and dedicated deployment boundary)

- Area affected: Docs governance, deployment guidance, and readiness watch routing.
- Change summary:
  - Closed the docs-index freshness wording watch by matching `docs/README.md` and `docs/operations/contributor-guide.md` to the exact trigger enforced by `scripts/validate-docs-index-freshness.sh`.
  - Tightened dedicated deployment guidance so the currently validated topology is explicitly single-server and realtime websocket abuse controls are clearly documented as process-local.
  - Added dedicated deployment checklist sign-off language for operators who attempt multi-instance realtime topologies.
- Rationale:
  - Docs governance rules should match CI enforcement exactly, and operator docs should not imply multi-instance realtime equivalence when websocket abuse controls are still process-local.
- Linked docs updated:
  - `docs/README.md`
  - `docs/operations/contributor-guide.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/02-dedicated-server-deployment.md`
  - `docs/operations/readiness-corrections-log.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-10 (contract-parity hardening for selected realtime semantics)

- Area affected: CI contract parity, realtime contract enforcement, and readiness watch routing.
- Change summary:
  - Added selected realtime semantic parity validation for the receive-side `realtime.connected`, `presence.updated`, `channel.message.created`, `channel.message.updated`, and `channel.message.deleted` envelopes in `scripts/contract_parity/engine.py` and `scripts/contract_parity/validator.py`.
  - Added a `fail-realtime-envelope-semantics` fixture regression and wired it into `scripts/test-contract-parity.sh` so envelope/data drift fails deterministically.
  - Refreshed contract/readiness docs so they describe the stronger gate accurately without overstating closure of the broader semantic-depth watch.
- Rationale:
  - The remaining parity-depth watch was still too broad on the realtime side; selected receive-side websocket event semantics were stable enough to enforce mechanically and high-value enough to deserve CI coverage now.
- Linked docs updated:
  - `docs/README.md`
  - `docs/contracts/README.md`
  - `docs/operations/contributor-guide.md`
  - `docs/operations/readiness-corrections-log.md`
  - `docs/planning/05-iteration-log.md`

### 2026-04-09 (readiness-governance cleanup after DM durability hardening)

- Area affected: Readiness routing, docs governance, and iteration/planning caveat authority.
- Change summary:
  - Made the `Active Watch Summary` in `docs/operations/readiness-corrections-log.md` exhaustive for the current open watches, including the web-coverage-policy and docs-index-freshness-policy watches.
  - Removed stale entry-doc caveats that still described DM replay-backlog durability as unresolved after the DM durability/docs alignment had already closed that finding.
  - Aligned `docs/README.md` and contributor guidance with the strict docs-index metadata refresh rule enforced by `scripts/validate-docs-index-freshness.sh`.
- Rationale:
  - Readiness entry points need one current watch authority; stale or incomplete caveats make planning and future audits drift-prone even when the underlying runtime/docs fixes are already complete.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/operations/contributor-guide.md`
  - `docs/operations/readiness-corrections-log.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-25 (T4.3.1 backend-first server-channel baseline)

- Area affected: Iteration 2 API sequencing, runtime REST contract, and server workspace enablement.
- Change summary:
  - Started `T4.3.1` as a backend-first runtime slice centered on server-channel message read/create endpoints under the existing `/servers/{server_id}/channels/{channel_id}/messages` namespace.
  - Locked the first delivered scope to persisted message listing, message creation, same-channel reply validation, and same-server mention validation.
  - Explicitly deferred edit/delete mutations, websocket fanout, richer audit-event semantics, and deeper channel/role permissions to later `T4.3.x` and `T4.4.x` follow-ups.
- Rationale:
  - The repo now has server-membership authorization primitives, but still needs a coherent persisted channel message API before fanout/UI expansion. Narrowing the first slice reduces risk while unblocking the placeholder server workspace.
- Linked docs updated:
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-03-16 (multi-device profile convergence hardening)

- Area affected: Communication networking semantics, iteration sequencing, and verification traceability.
- Change summary:
  - Added explicit profile-device convergence contract requiring active-device fanout plus late-device catch-up for DM communication and server communication paths.
  - Extended Iteration 2 backlog and execution docs with convergence tasks (`T4.1.9`, `T4.1.10`, `T3.3.2`, `T4.3.4`) and updated exit criteria/evidence mapping.
  - Updated product clarifications, defaults, and risk register to preserve the then-current server-bypassing DM policy while requiring deterministic multi-device eventual consistency.
  - Updated testing matrix to require profile-device convergence evidence for both DM and server-channel/presence flows.
- Rationale:
  - One profile can be active on multiple devices; communication state must converge without introducing infrastructure-dependent DM fallback.
- Linked docs updated:
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`

### 2026-03-12 (networking docs consistency hardening)

- Area affected: Networking architecture/planning/product authority boundaries and readiness traceability.
- Change summary:
  - Trimmed redundant sequencing content from architecture/product proposal docs and delegated execution authority to the DM connectivity execution plan.
  - Added cross-scenario networking architecture references across product/planning indexes to keep source-of-truth routing explicit.
  - Fixed DM pairing wording drift in Iteration 2 exit criteria (`QR/manual code`), keeping then-current peer-bootstrap terminology consistent.
  - Logged this correction in readiness governance history.
- Rationale:
  - Reduce drift risk by enforcing clear ownership boundaries between architecture, product options, and planning execution docs.
- Linked docs updated:
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/README.md`
  - `docs/README.md`
  - `docs/operations/readiness-corrections-log.md`

### 2026-03-12 (infra-free DM connectivity full planning alignment)

- Area affected: Product requirements, iteration feature plan, configuration defaults, and verification evidence model for DM connectivity.
- Change summary:
  - Added shared communication networking-layer architecture plan with explicit DM peer path vs server communication divergence boundaries.
  - Added full infrastructure-free DM connectivity execution plan with phased delivery and deterministic task gates.
  - Updated MVP plan and PRD to require server-bypassing DM transport, signed out-of-band pairing bootstrap, deterministic failure guidance, and no infra-assisted DM fallback.
  - Expanded Iteration 2 backlog with shared communication-layer tasks (`T4.0.1` to `T4.0.3`, `T4.3.3`) and server-bypassing tasks (`T4.1.3` to `T4.1.8`).
  - Updated configuration defaults and verification matrix to enforce server-bypassing policy at runtime and evidence level.
- Rationale:
  - Convert high-level policy lock into executable delivery artifacts so implementation work remains deterministic and policy-compliant.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/product/10-infra-free-dm-connectivity-proposals.md`
  - `docs/architecture/04-communication-networking-layer-plan.md`
  - `docs/planning/infra-free-dm-connectivity-execution-plan.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/README.md`
  - `docs/planning/README.md`

### 2026-03-12 (infra-free DM connectivity policy lock)

- Area affected: DM connectivity architecture guardrails and dependency/risk register.
- Change summary:
  - Added repository-level guardrail rejecting infrastructure-dependent DM connectivity solutions (including STUN/TURN/relay).
  - Locked clarification entry that accepted DM connectivity candidates must be infrastructure-free and fail with explicit user guidance when server-bypassing connectivity is unavailable.
  - Updated dependency/risk register entries to remove TURN fallback assumptions and raise NAT-restricted server-bypassing risk visibility.
- Rationale:
  - Enforce hard product direction toward no-infrastructure DM connectivity and prevent incremental drift toward hosted connectivity dependencies.
- Linked docs updated:
  - `AGENTS.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-12 (readiness hardening cycle: limiter determinism, parity guards, and governance metadata)

- Area affected: API/realtime limiter resilience, CI security/parity guardrails, and canonical documentation governance.
- Change summary:
  - Hardened API distributed limiter cleanup cadence to avoid per-request cleanup amplification while preserving stale-window pruning.
  - Normalized realtime websocket limiter/auth-cache hashing to deterministic digest-derived keys and aligned realtime tests.
  - Added CI gate for cargo-audit ignore expiry and expanded contract-parity watchlists to include limiter modules.
  - Reconciled canonical docs metadata/context drift across runbook, contributor guide, readiness log, and MVP plan; added explicit regression-linked closure notes in readiness corrections log.
- Rationale:
  - Raise readiness confidence by eliminating operational drift and ensuring runtime abuse-control behavior remains predictable across instances and over time.

### 2026-03-10 (readiness revalidation pass: recurrence prevention and runtime safeguard hardening)

- Area affected: Readiness governance, runtime startup resilience, websocket abuse controls, and evidence traceability.
- Change summary:
  - Added readiness correction governance policy in `AGENTS.md` and introduced `docs/operations/readiness-corrections-log.md` as the recurring-finding authority.
  - Hardened realtime websocket ingress: binary frames now share message-rate limits, handshake rejections return machine-readable error envelopes, and connect limit keying falls back to peer address when proxy headers are not trusted.
  - Added realtime numeric config guardrails for zero/degenerate values and minimum inbound payload thresholds.
  - Moved API and realtime tracing initialization before config parse to ensure startup failures are observable in logs.
  - Improved DB-backed API test behavior to use deterministic local default DB URL with explicit skip reason outside CI.
  - Expanded runtime REST contract to include implemented servers/contacts/friends endpoints and health probe.
  - Strengthened evidence provenance requirements (`commit_sha`, `pr_number`/`run_id`, `generated_at_utc`) across testing/evidence docs.
- Rationale:
  - Eliminate repeated readiness-audit rediscovery loops while increasing operational confidence and traceability for future feature work.

### 2026-03-05 (quality tightening pass: fail-closed controls and evidence completeness)

- Area affected: Realtime ingress trust boundary, API limiter resilience semantics, contact directory correctness, and CI evidence completeness.
- Change summary:
  - Tightened realtime websocket policy to require allowed `Origin`; missing origin now rejected.
  - Updated API distributed rate-limit behavior to fail closed when DB-backed limiter is unavailable in DB runtime mode.
  - Removed silent DB/decode fallback in contacts directory path; DB errors now surface as explicit API errors.
  - Raised Rust coverage CI threshold from 55% to 65% for stronger baseline regression confidence.
  - Expanded CI evidence artifacts with machine-readable `summary.json`, SHA256 file hashes, and coverage summary capture; added evidence index doc.
- Rationale:
  - Ensure implemented hardening improves real runtime quality under incident and abuse conditions rather than masking failures.

### 2026-03-05 (auth/key-management priority clarification)

- Area affected: Product security hardening scope and MVP prioritization.
- Change summary:
  - Recorded that passphrase-gated local key unlock remains optional hardening and is not an MVP priority.
  - Confirmed current MVP baseline remains cookie-first auth transport + CSRF + runtime abuse controls without mandatory passphrase UX.
- Rationale:
  - Preserve low-friction onboarding and avoid premature UX/security coupling while core functionality is still under active delivery.

### 2026-03-05 (readiness controls pass: security gates, evidence automation, distributed limiting, realtime guardrails)

- Area affected: CI security posture, release evidence quality, API abuse control scalability, realtime resilience, and API handler maintainability.
- Change summary:
  - Added CI security automation gates for Rust dependencies (`cargo audit`), web dependencies (`npm audit --omit=dev --audit-level=high`), and static analysis (`semgrep`).
  - Added deterministic CI evidence collection script and integration-smoke artifact upload under `evidence/ci/<run_id>/`.
  - Added DB-backed distributed API rate limiting counters (`rate_limit_counters`) to preserve abuse-control behavior across multi-instance API deployments sharing Postgres.
  - Added relational FK constraints for `sessions`, `auth_challenges`, and `friend_requests` against `identity_keys` to tighten persistence integrity.
  - Added realtime websocket guardrails: per-identity connection cap, inbound message-size cap, and per-identity message-rate cap.
  - Added realtime websocket `Origin` allowlist enforcement for browser-originated upgrades.
  - Added Rust coverage threshold gate in CI to provide quantitative backend test-confidence enforcement.
  - Continued handler decomposition by extracting directory/list endpoints into dedicated `directory_handlers` module.
- Rationale:
  - Improve confidence on substantive remaining quality risks while preserving local-first desktop defaults and enabling stronger dedicated-server safety under active development.

### 2026-03-05 (auth transport migration to HttpOnly cookie + CSRF)

- Area affected: Runtime auth transport across API, web client, realtime validation path, and runtime contracts.
- Change summary:
  - Switched runtime web auth transport from JS-managed bearer token usage to HttpOnly session cookie (`hexrelay_session`).
  - Added double-submit CSRF enforcement (`hexrelay_csrf` cookie + `x-csrf-token` header) for authenticated mutation routes.
  - Updated web API calls to `credentials: include` and removed auth token plumbing from page-level calls.
  - Updated realtime session validation forwarding to support cookie-authenticated websocket handshakes.
  - Updated runtime OpenAPI contracts and runbook auth language to reflect cookie-first transport.
  - Supersedes prior runtime bearer-token transport notes in historical entries below.
- Rationale:
  - Reduce token exfiltration risk from browser script-accessible storage while keeping runtime auth/session behavior explicit and testable.

### 2026-03-04 (security and hygiene hardening: token rotation, rate limiting, and runtime contract cleanup)

- Area affected: Auth/session security, abuse controls, runtime contract governance, and dead/legacy runtime path cleanup.
- Change summary:
  - Added signed bearer token format (`HEXTOKEN`) with signing key ID support and keyring-based token validation.
  - Added API rate limits for auth challenge/verify and invite create/redeem paths.
  - Added realtime websocket connect rate limiting.
  - Removed non-test runtime fallback behavior for identity/auth/invite/session critical storage paths; runtime now requires DB-backed authority for these flows.
  - Promoted runtime REST contract authority to `docs/contracts/runtime-rest.openapi.yaml`; the old legacy alias was removed later when internal API compatibility aliases were retired.
  - Added confidence-hardening evidence artifact baseline under `evidence/iteration-01/confidence-hardening/`.
- Rationale:
  - Reduce attack surface and runtime drift before additional feature expansion, while cleaning legacy authority naming and dead fallback runtime branches.
- Linked docs updated:
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/contracts/README.md`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/contracts/mvp-rest.openapi.yaml`
  - `docs/README.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`
  - `evidence/iteration-01/confidence-hardening/2026-03-04-quality-validation.md`
  - `services/api-rs/src/session_token.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/invite_handlers.rs`
  - `services/api-rs/src/auth.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/state.rs`

### 2026-03-04 (confidence hardening: independent-audit blocker closure)

- Area affected: Readiness blocker remediation
- Change summary:
  - Split current-runtime contracts and target-state model contracts by adding a contracts index and runtime realtime contract artifact.
  - Updated docs index routing so runtime behavior references runtime contracts while roadmap contracts remain explicitly model-only.
  - Added missing metadata/Quick Context blocks for runtime ADR and crypto contract/checklist docs, and linked the crypto checklist from testing index.
  - Updated sprint board metadata statuses to match active execution state for Iteration 1 and 2.
  - Reconciled stale Iteration 1 board notes that still described identity/session persistence as in-memory.
  - Added dedicated-server restore evidence contract requirements in runbook.
  - Added license artifact for documented AGPL baseline.
  - Reduced session token exposure persistence by storing access tokens in `sessionStorage` while keeping session metadata in local storage.
  - Enforced DB test confidence in CI by failing when `API_DATABASE_URL` is missing under CI context.
- Rationale:
  - Resolve independent hard-pass blockers that were not stylistic and materially affected confidence.
- Linked docs updated:
  - `docs/contracts/README.md`
  - `docs/contracts/realtime-events-runtime.asyncapi.yaml`
  - `docs/contracts/crypto-profile.md`
  - `docs/contracts/mvp-rest.openapi.yaml`
  - `docs/contracts/realtime-events.asyncapi.yaml`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/README.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `docs/testing/README.md`
  - `docs/testing/crypto-conformance-checklist.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `LICENSE`
  - `apps/web/lib/sessions.ts`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/db.rs`

### 2026-03-04 (confidence hardening: realtime contract and transport safety)

- Area affected: Realtime trust boundary and contract conformance
- Change summary:
  - Hardened realtime config to enforce valid API URL scheme and require HTTPS for non-loopback API upstreams.
  - Added strict realtime HTTP client timeout/connect-timeout defaults for auth validation calls.
  - Replaced websocket text echo behavior with structured event-envelope routing for call signaling event types.
  - Enforced realtime sender identity binding by validating `from_user_id` against authenticated session identity before accepting signaling payloads.
  - Added realtime contract tests for unsupported event handling, malformed payloads, and websocket roundtrip envelope shape.
  - Added negative integration test for websocket auth flow when API upstream is unreachable.
  - Added invite-token hash enforcement and removed plaintext invite-token redeem behavior.
  - Added web unit tests for secure-store provider failure fallback and recovery phrase derivation stability.
  - Aligned product stack wording to current HMAC bearer token model (removed JWT phrasing drift).
- Rationale:
  - Reduce auth-gate failure ambiguity and establish deterministic realtime event contract behavior before broader fanout feature work.
- Linked docs updated:
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/state.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/Cargo.toml`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/migrations/0007_invite_token_hash_constraint.sql`
  - `services/api-rs/src/invite_handlers.rs`
  - `apps/web/lib/secure-store.test.ts`
  - `apps/web/lib/recovery.test.ts`
  - `docs/product/01-mvp-plan.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (confidence hardening phase 1-2 kickoff)

- Area affected: Deployment clarity and security baseline
- Change summary:
  - Added explicit source-of-truth authority notes in infra and service READMEs to reduce runtime/deployment guidance drift.
  - Expanded MVP runbook with concrete dedicated-server startup order, TLS boundary assumptions, and restart validation checks.
  - Added runtime term mapping in glossary and linked PRD/plan runtime sections to glossary authority.
  - Replaced static onboarding recovery phrase with generated per-session phrase flow.
  - Introduced secure-store abstraction for private key encryption materials (provider-backed when available, session fallback otherwise).
  - Hardened invite storage by persisting hashed invite tokens instead of plaintext tokens for new records (with backward-compatible redeem matching).
  - Aligned long-range REST contract bearer token format wording to current token model (`HEXTOKEN`).
- Rationale:
  - Raise readiness confidence before further feature expansion by tightening both contributor-operational clarity and critical auth/privacy handling paths.
- Linked docs updated:
  - `infra/README.md`
  - `services/api-rs/README.md`
  - `services/realtime-rs/README.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `docs/reference/glossary.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/contracts/mvp-rest.openapi.yaml`
  - `apps/web/lib/secure-store.ts`
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/recovery.ts`
  - `apps/web/app/onboarding/recovery/page.tsx`
  - `services/api-rs/src/invite_handlers.rs`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (documentation alignment: runtime/deployment model)

- Area affected: Product and architecture context clarity
- Change summary:
  - Locked and documented primary runtime as bundled desktop local-first mode.
  - Added explicit local UI launch options: embedded desktop shell or local-browser access on localhost.
  - Documented dedicated server mode as supported optional deployment path.
  - Added ADR-0002 for runtime/deployment modes and aligned README, product, operations, and service docs.
- Rationale:
  - Remove ambiguity about browser-only hosted assumptions and keep implementation decisions aligned with off-grid desktop goals.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/architecture/README.md`
  - `docs/architecture/adr-0002-runtime-deployment-modes.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/reference/glossary.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/operations/contributor-guide.md`
  - `apps/web/README.md`
  - `services/api-rs/README.md`
  - `services/realtime-rs/README.md`
  - `AGENTS.md`

### 2026-03-04 (execution batch: contacts optimism, invite UX, cross-service smoke gate)

- Area affected: Iteration 2 delivery velocity and integration safety
- Change summary:
  - Added optimistic friend-request UX behavior in Contacts hub with rollback/error messaging and action busy states for send/accept/decline.
  - Added in-app invite create/redeem controls to Contacts hub to execute invite workflows outside onboarding.
  - Added cross-service smoke path for `web -> api -> realtime` with CI `integration-smoke` job (Postgres-backed services + websocket auth handshake validation).
  - Added smoke runner script (`apps/web/scripts/e2e-smoke.mjs`) and web package command `e2e:smoke`.
  - Started API handler modularization by extracting invite handlers into `services/api-rs/src/invite_handlers.rs` and re-exporting via `services/api-rs/src/handlers.rs`.
- Rationale:
  - Continue feature delivery while preserving confidence through real cross-service validation and reducing handler-file growth pressure.
- Linked docs updated:
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/scripts/e2e-smoke.mjs`
  - `apps/web/package.json`
  - `.github/workflows/ci.yml`
  - `services/api-rs/src/invite_handlers.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (readiness uplift: persistence and CI coverage gates)

- Area affected: Future-development readiness hardening
- Change summary:
  - Added DB-backed identity-key persistence with new migration and API handler DB paths for registration/challenge/verify identity lookup.
  - Added DB-backed auth-challenge and invite durability (`auth_challenges`, `invites`) with restart-safe verification/redeem test coverage.
  - Aligned auth challenge TTL to 60 seconds (`CHALLENGE_TTL_SECONDS = 60`) to match crypto profile expectations.
  - Made API session signing key mandatory from environment to remove insecure fallback-key behavior.
  - Added realtime websocket-gate integration tests (authorized upgrade + unauthorized rejection) and expanded web API transport tests.
  - Raised web coverage thresholds and enforced coverage execution in CI via `test:coverage`.
  - Updated CI to provision Postgres for Rust checks and pass API DB/signing env vars so DB integration paths execute under CI.
- Rationale:
  - Raise confidence from "good" to "high" by ensuring critical auth/persistence paths are both enforced and continuously validated in CI.
- Linked docs updated:
  - `services/api-rs/migrations/0004_identity_keys.sql`
  - `services/api-rs/migrations/0005_auth_challenges.sql`
  - `services/api-rs/migrations/0006_invites.sql`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/lib.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `apps/web/lib/api.test.ts`
  - `apps/web/vitest.config.ts`
  - `apps/web/package.json`
  - `apps/web/package-lock.json`
  - `.github/workflows/ci.yml`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (stabilization follow-up: replay race, migration safety, contract sync)

- Area affected: Auth/session correctness and persistence safety gates
- Change summary:
  - Made auth challenge consumption atomic in verify flow (challenge removed under write lock before signature verification) to eliminate replay race window.
  - Hardened migration lock lifecycle with guaranteed unlock attempt after migration execution path returns.
  - Added DB-backed integration tests for session validate/revoke lifecycle and migration checksum mismatch detection/lock release behavior.
  - Added concurrent replay test ensuring only one verify succeeds for duplicate challenge verification attempts.
  - Updated Iteration 1 OpenAPI contract to include bearer-auth requirements, session validate endpoint, and `access_token` in auth verify response.
- Rationale:
  - Complete mandatory hardening preconditions so future Iteration 2 feature work builds on deterministic auth and migration invariants.
- Linked docs updated:
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/lib.rs`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/05-iteration-log.md`
  - `docs/planning/iterations/02-sprint-board.md`

### 2026-03-04 (session-token enforcement and migration checksum hardening)

- Area affected: Cross-cutting auth/session and persistence integrity
- Change summary:
  - Added signed bearer session token flow end-to-end: API issues `access_token` on verify and web stores/uses it for protected API calls.
  - Tightened API auth-sensitive routes by requiring authenticated context for revoke and list endpoints, with session-id match enforcement for revoke.
  - Updated API CORS allow-headers to include `Authorization` for browser preflight compatibility.
  - Added DB `sessions` migration plus migration checksum tracking and advisory-lock guarded migration execution.
  - Updated realtime API-validation bridge to require and forward `Authorization` when checking websocket session validity.
  - Rewired web hubs/onboarding/home to consume `access_token` consistently for servers/contacts/friend-request and session revoke paths.
- Rationale:
  - Remove header-forgery-prone session-only transport as the primary path and align runtime auth to signed token + server-side validation.
- Linked docs updated:
  - `services/api-rs/migrations/0003_sessions.sql`
  - `services/api-rs/src/session_token.rs`
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/Cargo.toml`
  - `services/realtime-rs/src/handlers.rs`
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/api.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `Cargo.lock`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (quality hardening batch: auth, cors, realtime gate)

- Area affected: Cross-cutting quality/security hardening
- Change summary:
  - Added identity registration conflict guard to prevent silent key overwrite for existing identities.
  - Added `GET /auth/sessions/validate` and reused centralized `AuthSession` extractor for server-side session-bound auth context.
  - Restricted API CORS from wildcard to env-driven explicit allowlist (`API_ALLOWED_ORIGINS`).
  - Hardened friend-request handlers to require database pool in non-test runtime and keep in-memory path only for tests.
  - Added realtime websocket auth gate by validating `x-session-id` against API before upgrade.
  - Improved frontend hubs (`/servers`, `/contacts`) with explicit network error catch/finalization paths.
- Rationale:
  - Close immediate trust-boundary and operational safety gaps before additional feature expansion.
- Linked docs updated:
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/.env.example`
  - `services/realtime-rs/src/state.rs`
  - `services/realtime-rs/src/config.rs`
  - `services/realtime-rs/src/app.rs`
  - `services/realtime-rs/src/handlers.rs`
  - `services/realtime-rs/src/main.rs`
  - `services/realtime-rs/.env.example`
  - `services/realtime-rs/Cargo.toml`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (migration governance and transition-matrix hardening)

- Area affected: Iteration 2 social graph hardening (`T3.1.1`)
- Change summary:
  - Added versioned migration runner with tracked `schema_migrations` table and explicit migration files for friend-request schema/indexes.
  - Added centralized Axum `AuthSession` extractor and rewired friend-request handlers to use shared auth context instead of duplicated header parsing.
  - Added strict transition matrix behavior for friend requests: pending-only mutations, requester-only cancel, target-only accept/decline.
  - Added idempotent semantics for repeated same terminal action and `409 transition_invalid` for conflicting non-pending transitions.
  - Extended tests for missing session auth, wrong actor rejection, cancel flow, and conflicting transition rejection.
- Rationale:
  - Enforce durable schema evolution and deterministic social-graph mutation rules before scaling social features.
- Linked docs updated:
  - `services/api-rs/src/auth.rs`
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/migrations/0001_friend_requests.sql`
  - `services/api-rs/migrations/0002_friend_requests_transition_index.sql`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (friend request Postgres persistence hardening)

- Area affected: Iteration 2 social graph persistence (`T3.1.1`)
- Change summary:
  - Added `sqlx` Postgres integration in `api-rs` with startup schema preparation for `friend_requests`.
  - Added DB-backed create/list/accept/decline friend-request handlers with fallback to in-memory state for non-DB contexts.
  - Added centralized Axum auth extractor (`AuthSession`) for session-bound actor enforcement via `x-session-id` and server-side session lookup.
  - Added pending-only transition guards so accept/decline cannot mutate non-pending requests or unauthorized actors.
  - Added runtime `API_DATABASE_URL` config and updated service env template.
  - Preserved and revalidated full Rust/Web test suite after dependency/version compatibility pinning.
- Rationale:
  - Move friend-request lifecycle off volatile in-memory storage to a durable persistence path before expanding social graph features.
- Linked docs updated:
  - `services/api-rs/src/db.rs`
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/Cargo.toml`
  - `services/api-rs/.env.example`
  - `Cargo.lock`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (onboarding scope simplification)

- Area affected: Onboarding UX scope (`T2.1.4`)
- Change summary:
  - Removed server join/contact request actions from onboarding access step.
  - Converted access step into completion/handoff screen that routes users into the main app hubs for join/invite flows.
- Rationale:
  - Reduce onboarding complexity and user confusion by keeping onboarding focused on identity and recovery only.
- Linked docs updated:
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (friend-request API baseline and contacts request actions)

- Area affected: Iteration 2 social graph bootstrap (`T3.1.1`, `T3.1.2`)
- Change summary:
  - Implemented friend-request endpoints in `api-rs`: create/list plus accept/decline transitions.
  - Added query validation and in-memory state tracking for pending request lifecycle.
  - Added API tests for create/list and accept/decline behavior.
  - Wired Contacts hub to live friend-request endpoints and added send/accept/decline UI actions.
  - Added API-backed Servers/Contacts read endpoints and dynamic server workspace route scaffold for route continuity.
- Rationale:
  - Start Iteration 2 social graph execution on top of already stable identity/auth/invite primitives while keeping implementation deterministic and test-covered.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/src/lib.rs`
  - `apps/web/lib/api.ts`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/servers/[serverId]/page.tsx`
  - `apps/web/app/surfaces.module.css`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (servers and contacts API-backed hub wiring)

- Area affected: Iteration 1 web/backend integration (`T2.1.3` support)
- Change summary:
  - Added API read endpoints `GET /servers` and `GET /contacts` with deterministic query filtering.
  - Added backend tests covering server/contact list filtering paths.
  - Rewired web Servers and Contacts routes to call live API endpoints instead of local in-file datasets.
  - Added dynamic server workspace route scaffold at `/servers/[serverId]` and linked server cards to workspace route navigation.
  - Preserved screen-state mapping (`loading`, `error`, `empty`, `search_no_results`, request states) while changing data source to backend.
- Rationale:
  - Reduce placeholder logic and align hub surfaces with real API contracts ahead of friend/server persistence work.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/lib.rs`
  - `apps/web/lib/api.ts`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (hub state interactivity pass)

- Area affected: Iteration 1 web hub/state execution (`T2.1.3` support)
- Change summary:
  - Upgraded Servers and Contacts routes to client-interactive hubs with search and filter toggles.
  - Added explicit screen-state rendering outputs (`empty`, `search_no_results`, `friend_request_pending`, `friend_request_inbound`, `ready`) in hub surfaces.
  - Added Settings DM inbound policy persistence (`friends_only`, `same_server`, `anyone`) with per-device local preference storage.
- Rationale:
  - Move hub pages from static placeholders to stateful surfaces aligned with MVP screen-state spec before deeper backend query wiring.
- Linked docs updated:
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/settings/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (workspace shell and top-level navigation surfaces)

- Area affected: Iteration 1 web navigation execution (`T2.1.3`, `T2.1.4` support)
- Change summary:
  - Added shared workspace shell component with top-level navigation (`Home`, `Servers`, `Contacts`, `Settings`) and mobile tab switcher.
  - Added server dual-navigation baseline affordances: collapsible sidebar preference and top tab strip.
  - Added initial route surfaces for `/servers`, `/contacts`, and `/settings` aligned with hub/filter state requirements.
  - Migrated `/home` to run inside shared shell while preserving persona/session controls.
- Rationale:
  - Align executable UI structure with navigation spec so subsequent feature work lands on stable route/layout primitives.
- Linked docs updated:
  - `apps/web/components/workspace-shell.tsx`
  - `apps/web/components/workspace-shell.module.css`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/servers/page.tsx`
  - `apps/web/app/contacts/page.tsx`
  - `apps/web/app/settings/page.tsx`
  - `apps/web/app/surfaces.module.css`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (key-at-rest encryption and persona session revoke wiring)

- Area affected: Iteration 1 web security and session lifecycle execution (`T2.1.2`, `T2.1.3`, `T2.3.1`)
- Change summary:
  - Replaced plain localStorage private-key persistence with persona-scoped AES-GCM encrypted storage.
  - Added Home persona remove action and switch-time session revoke integration using `POST /auth/sessions/revoke`.
  - Added persona cleanup paths to remove encrypted key/session records on persona deletion.
  - Added lightweight onboarding/home telemetry event tracking for API flow stages and failures.
- Rationale:
  - Tighten local key handling and enforce deterministic session lifecycle behavior during persona transitions.
- Linked docs updated:
  - `apps/web/lib/sessions.ts`
  - `apps/web/lib/personas.ts`
  - `apps/web/lib/api.ts`
  - `apps/web/lib/telemetry.ts`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/home/home.module.css`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (identity auth wiring and invite create UX integration)

- Area affected: Iteration 1 onboarding API integration (`T2.1.2`, `T2.2.1`, `T2.3.1`)
- Change summary:
  - Wired identity onboarding to live API flow: register identity key -> challenge issue -> challenge verify.
  - Added client crypto utilities for ed25519 key generation/import parsing and nonce signature generation.
  - Added persona-scoped local session/private-key storage utilities and stored auth session on successful verify.
  - Added onboarding access action to create test invites via live `POST /invites` before redemption.
  - Extended web API client module to cover identity/auth/invite endpoints.
- Rationale:
  - Replace onboarding placeholders with executable integration against implemented Iteration 1 API primitives.
- Linked docs updated:
  - `apps/web/lib/api.ts`
  - `apps/web/lib/crypto.ts`
  - `apps/web/lib/sessions.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `apps/web/package.json`
  - `apps/web/package-lock.json`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (persona isolation scaffold)

- Area affected: Iteration 1 web identity execution (`T2.1.3`)
- Change summary:
  - Added browser-local persona storage utilities with active-persona tracking.
  - Wired onboarding identity step to persist/select persona before moving to recovery.
  - Replaced `/home` placeholder with persona management and switching surface showing active-session context.
- Rationale:
  - Establish deterministic client-side persona/session isolation baseline before deeper auth/session integration.
- Linked docs updated:
  - `apps/web/lib/personas.ts`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `apps/web/app/home/home.module.css`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (server id verification and onboarding API wiring)

- Area affected: Iteration 1 join-flow security and web onboarding integration (`T2.4.1`)
- Change summary:
  - Added invite-bound server id enforcement in `api-rs` redeem flow; mismatched server id now fails with `server_mismatch`.
  - Added CORS middleware to API router so web onboarding can call local API endpoints in dev.
  - Added `API_SERVER_ID` runtime config and threaded value into application state.
  - Added API tests for server id mismatch rejection and updated invite redeem tests to include expected server id.
  - Wired onboarding access screen to live `POST /invites/redeem` calls and mapped API error codes (`invite_invalid`, `invite_expired`, `invite_exhausted`, `server_mismatch`).
- Rationale:
  - Enforce fail-closed join verification at API boundary and remove placeholder token simulation from onboarding.
- Linked docs updated:
  - `services/api-rs/src/config.rs`
  - `services/api-rs/src/main.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/Cargo.toml`
  - `services/api-rs/.env.example`
  - `Cargo.lock`
  - `apps/web/lib/api.ts`
  - `apps/web/app/onboarding/access/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (onboarding flow shell implementation)

- Area affected: Iteration 1 onboarding web execution (`T2.1.2`, `T2.1.4`)
- Change summary:
  - Replaced starter web screen with route-based onboarding flow: `/onboarding/identity`, `/onboarding/recovery`, `/onboarding/access`.
  - Added identity create/import UX shell with validation-state feedback and persona labeling scaffold.
  - Added mandatory recovery checkpoint UX requiring phrase word confirmation before progression.
  - Added access choice UX for server invite, contact request, or skip path plus `/home` post-onboarding placeholder.
  - Updated global web styling baseline and font stack for a dedicated product visual direction.
- Rationale:
  - Move from scaffolding UI to executable onboarding flow aligned with Iteration 1 product requirements.
- Linked docs updated:
  - `apps/web/app/page.tsx`
  - `apps/web/app/layout.tsx`
  - `apps/web/app/globals.css`
  - `apps/web/app/onboarding/onboarding.module.css`
  - `apps/web/app/onboarding/page.tsx`
  - `apps/web/app/onboarding/identity/page.tsx`
  - `apps/web/app/onboarding/recovery/page.tsx`
  - `apps/web/app/onboarding/access/page.tsx`
  - `apps/web/app/home/page.tsx`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (invite create/redeem baseline)

- Area affected: Iteration 1 invite execution (`T2.2.1`)
- Change summary:
  - Implemented `POST /invites` and `POST /invites/redeem` in `services/api-rs`.
  - Added invite mode/expiry/max-uses validation including one-time invite max-use enforcement.
  - Added deterministic invalid, expired, and exhausted invite behavior with explicit error codes.
  - Added API tests for multi-use redeem success, one-time exhaustion, and expired invite rejection.
  - Updated Iteration 1 OpenAPI with invite create/redeem response schemas.
- Rationale:
  - Complete baseline invite lifecycle behavior needed for Iteration 1 join/auth flow dependencies.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (auth verify and session revoke baseline)

- Area affected: Iteration 1 auth execution (`T2.3.1`)
- Change summary:
  - Implemented `POST /auth/verify` with nonce lookup/expiry checks, ed25519 signature verification, single-use challenge consumption, and in-memory session issuance.
  - Implemented `POST /auth/sessions/revoke` with deterministic invalid-session rejection.
  - Added API tests covering verify/revoke success path and invalid signature rejection.
  - Updated Iteration 1 OpenAPI to include `AuthVerifyResponse` and explicit `400/401` verify outcomes.
- Rationale:
  - Complete the core challenge-signature auth loop so session lifecycle behavior is executable before moving to invite/join hardening.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/src/errors.rs`
  - `services/api-rs/Cargo.toml`
  - `Cargo.lock`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (auth challenge endpoint baseline)

- Area affected: Iteration 1 auth bootstrap execution (`T2.3.1`)
- Change summary:
  - Implemented `POST /auth/challenge` in `services/api-rs` with registered-identity enforcement and nonce challenge issuance.
  - Added in-memory challenge store to API state and modularized handler wiring to include auth challenge routing.
  - Added API tests for challenge issuance (registered identity) and unknown identity rejection.
  - Updated Iteration 1 OpenAPI contract to include `AuthChallengeResponse` schema.
- Rationale:
  - Unblock signature-verify flow by providing deterministic challenge issuance behavior aligned to the Iteration 1 contract.
- Linked docs updated:
  - `services/api-rs/src/app.rs`
  - `services/api-rs/src/handlers.rs`
  - `services/api-rs/src/lib.rs`
  - `services/api-rs/src/models.rs`
  - `services/api-rs/src/state.rs`
  - `services/api-rs/src/validation.rs`
  - `services/api-rs/Cargo.toml`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (iteration 1 identity endpoint start)

- Area affected: Iteration 1 `T2.1.1` execution progress
- Change summary:
  - Implemented `POST /identity/keys/register` in `services/api-rs` with fail-fast validation for algorithm and public key format.
  - Added API tests covering success path and invalid algorithm/key rejection.
  - Aligned Iteration 1 OpenAPI error-code enum with identity registration validation errors.
  - Marked `T2.1.1` as in progress in the Iteration 1 board.
- Rationale:
  - Establish executable identity registration baseline before challenge/verify and invite flows.
- Linked docs updated:
  - `services/api-rs/src/main.rs`
  - `services/api-rs/Cargo.toml`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (iteration 1 quality and config gates)

- Area affected: Iteration 1 quality gate enforcement and configuration validation
- Change summary:
  - Hardened CI workflow to run active Rust/Web quality gates without scaffold-skip detection.
  - Added runtime environment validation for API and realtime services (`API_BIND`, `REALTIME_BIND`).
  - Added web environment schema validation for API and realtime endpoint URLs.
  - Added `.env.example` templates for `apps/web`, `services/api-rs`, and `services/realtime-rs`.
  - Marked `T1.2.1` and `T1.3.1` completed in the Iteration 1 board.
- Rationale:
  - Ensure invalid configuration fails fast and CI gates are enforceable before starting identity/auth implementation tasks.
- Linked docs updated:
  - `.github/workflows/ci.yml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `apps/web/.env.example`
  - `services/api-rs/.env.example`
  - `services/realtime-rs/.env.example`

### 2026-03-04 (iteration 1 foundation kickoff)

- Area affected: Iteration 1 execution tracking
- Change summary:
  - Marked `T1.1.1`, `T1.1.2`, and `T1.1.3` as complete in the Iteration 1 board.
  - Added one-command workspace flows via root npm scripts (`setup`, `run`, `test`).
  - Updated root getting-started guidance to reflect runnable scaffold bootstrap.
- Rationale:
  - Align task status with completed implementation bootstrap work before moving to `T1.2.x` and `T1.3.1`.
- Linked docs updated:
  - `README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `scripts/README.md`
  - `package.json`

### 2026-03-04 (development bootstrap execution)

- Area affected: Project development readiness and execution gates
- Change summary:
  - Initialized runnable web scaffold in `apps/web` with lint/test/build scripts.
  - Initialized Rust service scaffolds in `services/api-rs` and `services/realtime-rs` with workspace `Cargo.toml`.
  - Added local infra stack in `infra/` with compose, env defaults, and TURN configuration.
  - Added CI workflow in `.github/workflows/ci.yml` with Rust/Web quality gates.
  - Replaced placeholder workspace automation with executable scripts and `Makefile` targets.
  - Promoted dependency gates `D-001` to `D-007` to `ready` in dependency register.
- Rationale:
  - Move from planning-only to an executable baseline so Iteration 1 development can begin with enforceable quality gates.
- Linked docs updated:
  - `README.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (dm offline policy lock)

- Area affected: DM reliability semantics and UX expectations
- Change summary:
  - Locked MVP DM offline behavior to best-effort online delivery.
  - Added encrypted local outbox retry expectation to DM execution and verification docs.
  - Registered config default, risk, and decision entries for offline DM behavior.
- Rationale:
  - Preserve the then-current server-bypassing DM transport decision without introducing server-side DM queues in MVP.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/reference/glossary.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (dm transport architecture correction)

- Area affected: Core messaging architecture and MVP execution tasks
- Change summary:
  - Corrected DM architecture to the then-current server-bypassing transport model with no community server relay/storage.
  - Updated plan, PRD, Iteration 2 tasks, REST/realtime contracts, data lifecycle matrix, and verification matrix to match this model.
  - Removed server-ciphertext DM assumptions from execution and validation language.
- Rationale:
  - Align implementation docs with core product intent: server communities should not be DM transport intermediaries.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/contracts/mvp-rest.openapi.yaml`
  - `docs/contracts/realtime-events.asyncapi.yaml`
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (execution hardening deep-pass)

- Area affected: Full-MVP documentation precision and implementation readiness
- Change summary:
  - Added MVP REST contract coverage baseline for Iterations 2-4.
  - Added canonical screen-state spec and configuration defaults register.
  - Added architecture-level data lifecycle/retention/replication matrix.
  - Added MVP operations runbook and requirement-to-evidence verification matrix.
  - Added UI/flow state mappings and evidence ledgers to Iterations 1-4 boards.
  - Updated docs indexes to register new canonical artifacts.
- Rationale:
  - Reduce cross-team ambiguity during parallel implementation.
  - Make requirement -> task -> evidence trace deterministic.
- Linked docs updated:
  - `docs/contracts/mvp-rest.openapi.yaml`
  - `docs/product/08-screen-state-spec.md`
  - `docs/product/09-configuration-defaults-register.md`
  - `docs/architecture/02-data-lifecycle-retention-replication.md`
  - `docs/operations/01-mvp-runbook.md`
  - `docs/testing/01-mvp-verification-matrix.md`
  - `docs/testing/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/product/README.md`
  - `docs/architecture/README.md`
  - `docs/operations/README.md`
  - `docs/planning/README.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (privacy-first social policy lock)

- Area affected: MVP friend request and DM onboarding behavior
- Change summary:
  - Locked server-mediated friend request model for in-server contact flows.
  - Locked default privacy rule preventing raw key/profile-identifying data exposure before acceptance.
  - Added Iteration 2 tasks for mediated identity bootstrap release and DM inbound policy defaults/overrides.
  - Added risk and decision coverage for identity scraping prevention.
- Rationale:
  - Preserve user privacy by default while keeping server-assisted contact discovery usable.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/reference/glossary.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (server invite policy normalization)

- Area affected: MVP server onboarding policy and invite semantics
- Change summary:
  - Locked server invite policy to allow optional expiration and optional max-uses.
  - Explicitly allowed non-expiring multi-use invite links as an open-access pattern.
  - Updated Iteration 1 task acceptance and OpenAPI schema to cover optional invite policy fields.
  - Added clarification and decision entries for this policy.
- Rationale:
  - Keep invite-based architecture while supporting practical open-server behavior without separate join modes.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (old contact-link proposal)

- Area affected: MVP social graph onboarding and contact-add flow
- Change summary:
  - Added an expiring link + QR proposal to MVP plan and PRD; current product direction now uses friend requests for Contacts.
  - Added Iteration 2 API/Web tasks for link create/redeem and share/scan UX.
  - Extended requirement-to-task matrix with contact-add coverage.
- Rationale:
  - Allow users to add each other by invite without depending on global/shared-server discovery.
  - Align user add UX with invite-based mental model already used for server joins.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (post-MVP discovery roadmap lock)

- Area affected: Post-MVP product roadmap direction
- Change summary:
  - Locked post-MVP discovery strategy to hybrid mode.
  - Federation discovery remains supported, trusted-registry scopes are planned, and decentralized server discovery is an optional later mode.
  - Updated plan, PRD, clarifications, and decisions register to reflect this direction.
- Rationale:
  - Preserve self-hosted usability and selective discoverability while keeping a clear path toward deeper decentralization.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (migration precedence decision lock)

- Area affected: Iteration 4 migration reconciliation policy
- Change summary:
  - Resolved `C-014` with canonical rule: user-signed profile data is authoritative for profile fields.
  - Locked server role to identity/security/membership enforcement, not profile-field authority.
  - Updated migration and profile authority wording in product plan, PRD, risk register, and Iteration 4 entry gate.
- Rationale:
  - Preserve user data ownership model while keeping server-side security and permission enforcement deterministic.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (clarification resolution and artifact lock)

- Area affected: Full-picture pre-MVP planning gates
- Change summary:
  - Resolved `C-012` by adding realtime contract artifact `docs/contracts/realtime-events.asyncapi.yaml`.
  - Resolved `C-013` by adding fixed KPI/SLO benchmark profile `docs/planning/kpi-slo-test-profile.md`.
  - Linked Iteration 2/3/4 gate language to resolved artifacts and clarification IDs.
  - Kept `C-014` open pending migration conflict precedence decision.
- Rationale:
  - Remove remaining planning ambiguity for realtime contracts and KPI/SLO evidence.
  - Preserve one explicit final decision gate before Iteration 4 migration sign-off.
- Linked docs updated:
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/contracts/realtime-events.asyncapi.yaml`
  - `docs/planning/kpi-slo-test-profile.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/README.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (full-picture iteration documentation pass)

- Area affected: Iteration 1-4 planning visibility before MVP kickoff
- Change summary:
  - Added cross-iteration handoff matrix, artifact gate checklist, and evidence pack format to iteration index.
  - Added explicit `Entry Criteria` and `Exit Evidence` sections to Iteration 1-4 boards.
  - Added open clarifications for remaining execution questions (realtime contract artifact scope, KPI/SLO test profile, migration conflict precedence).
  - Added risk-to-task mitigation matrix and updated dependency status for navigation mapping.
  - Linked iteration gate sentences directly to clarification IDs and aligned template parity with active boards.
- Rationale:
  - Provide a full-picture execution plan before coding starts.
  - Make remaining unknowns explicit and trackable instead of implicit assumptions.
- Linked docs updated:
  - `docs/planning/iterations/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (post-hardening precision parity)

- Area affected: Iteration 2-4 execution precision and planning consistency
- Change summary:
  - Added touchpoint/validation gate sections to Iteration 2, 3, and 4 boards for schema parity with Iteration 1.
  - Extended PRD-to-task trace matrix with KPI and discovery-policy coverage rows.
  - Updated template to include touchpoint/validation gate section by default.
  - Normalized stale metadata in `docs/reference/README.md` and dependency status for OpenAPI artifact gate.
- Rationale:
  - Remove remaining non-blocking precision gaps before full parallel MVP execution.
  - Ensure future sprint boards retain deterministic execution quality.
- Linked docs updated:
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/reference/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (execution hardening pass)

- Area affected: MVP execution readiness and sprint precision
- Change summary:
  - Expanded dependency register with contract, crypto, navigation-mapping, and voice test-environment gates.
  - Reconciled E2EE risk language with locked MVP requirement for 1:1 and group DM E2EE.
  - Added Iteration 1 OpenAPI artifact gate and touchpoint/validation matrix for all Iteration 1 tasks.
  - Hardened Iteration 2 with group-DM E2EE tasks and navigation-spec trace matrix.
  - Tightened ownership and binary acceptance criteria in Iterations 2-4.
  - Normalized `last_updated` metadata in Iterations 2-4 to 2026-03-04.
- Rationale:
  - Remove remaining contradictions and ambiguity before coding kickoff.
  - Improve deterministic execution quality for AI agents across API/Core/Web/Realtime work.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/contracts/runtime-rest.openapi.yaml`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/iterations/02-sprint-board.md`
  - `docs/planning/iterations/03-sprint-board.md`
  - `docs/planning/iterations/04-sprint-board.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/README.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (server navigation interaction model lock; current wording updated 2026-05-20)

- Area affected: MVP navigation interaction model
- Change summary:
  - Locked dual server navigation mode: sidebar list plus topbar browser-like tabs.
  - Locked pinned/saved tabs and manual tab reorder as required navigation capabilities.
  - Locked explicit collapse controls for server navigation visibility inside a server workspace.
  - Updated plan, PRD, navigation spec, and clarifications to align on this model.
- Rationale:
  - Improve navigation speed and organization for large server sets while preserving Discord-like familiarity.
  - Allow focused in-server interaction by temporarily hiding navigation chrome.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/07-ui-navigation-spec.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (navigation design direction lock)

- Area affected: MVP product design and navigation architecture
- Change summary:
  - Locked UI direction to be heavily Discord-inspired with explicit server-navigation deviation.
  - Added canonical navigation/layout specification document for MVP.
  - Locked global `Servers` and `Contacts` hub pages as first-class surfaces.
  - Updated plan, PRD, clarifications, and Iteration 1 board to reference the new navigation authority.
- Rationale:
  - Capture product-level design decisions in canonical docs before implementation expands.
  - Improve navigation scalability for users in large server sets.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/07-ui-navigation-spec.md`
  - `docs/product/README.md`
  - `docs/README.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`

### 2026-03-04 (live clarification resolution)

- Area affected: Remaining MVP planning questions and execution precision
- Change summary:
  - Converted live user answers into locked decisions for group DM E2EE, discovery abuse controls, recovery policy, and UI behavior authority.
  - Updated MVP plan and PRD to require group DM E2EE in MVP.
  - Added discovery rate-limit and denylist baseline for MVP discovery.
  - Added mandatory recovery-phrase onboarding policy.
  - Added per-flow UI state tables in Iteration 1 sprint board as the execution authority.
  - Removed file-based quiz workflow and kept clarifications in `docs/product/03-clarifications.md`.
- Rationale:
  - Remove remaining ambiguity that blocked deterministic AI execution on E2 and onboarding paths.
  - Keep decision capture in canonical docs rather than temporary questionnaires.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/03-clarifications.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/planning/05-iteration-log.md`
  - `docs/README.md`
  - `docs/product/README.md`

### 2026-03-04 (readiness detail pass)

- Area affected: MVP execution readiness for Iteration 1 identity/auth/invite work
- Change summary:
  - Locked invite semantics to mode + expiration + max-uses with join-eligibility-only scope.
  - Added MVP Crypto Profile for identity/auth and baseline DM cryptography.
  - Added Iteration 1 OpenAPI endpoint and error-code baseline for identity/invite/auth.
  - Tightened Iteration 1 sprint acceptance criteria for invite exhaustion and nonce replay behavior.
  - Captured remaining product and UX questions for live user-driven resolution.
- Rationale:
  - Remove blocker ambiguity for E2 implementation tasks while preserving unresolved product decisions in a controlled queue.
  - Improve deterministic execution quality for AI agents.
- Linked docs updated:
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/planning/iterations/01-sprint-board.md`
  - `docs/product/03-clarifications.md`
  - `docs/product/README.md`
  - `docs/README.md`

### 2026-03-04

- Area affected: Documentation governance and onboarding
- Change summary:
  - Added explicit planning-only onboarding guidance in `README.md`.
  - Added contributor workflow guide at `docs/operations/contributor-guide.md`.
  - Established canonical ADR with `docs/architecture/adr-0001-stack-baseline.md`.
  - Reduced duplicated locked-decision and risk content in `docs/product/02-prd.md` by pointing to canonical sources.
  - Added clarifications and dependency/risk source docs under `docs/product/`.
- Rationale:
  - Improve new-contributor orientation before implementation scaffold exists.
  - Reduce drift risk across PRD and planning docs.
  - Start explicit architecture decision tracking.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/product/02-prd.md`
  - `docs/architecture/README.md`
  - `docs/reference/glossary.md`

### 2026-03-04 (standardization pass)

- Area affected: Documentation standards and canonical ownership boundaries
- Change summary:
  - Removed duplicated task authority from `docs/product/01-mvp-plan.md` and delegated task-level ownership to iteration boards.
  - Removed KPI threshold duplication from `docs/product/01-mvp-plan.md` and kept KPI authority in `docs/product/02-prd.md`.
  - Normalized repeated iteration links to point at `docs/planning/iterations/README.md` from top-level indexes.
  - Added `Quick Context` sections to canonical operational docs to make edit intent explicit.
  - Normalized ADR metadata with `Status: canonical` and explicit `Decision status: accepted`.
  - Added deterministic docs QA checks to contributor workflow.
- Rationale:
  - Eliminate planning drift risk between strategy and sprint docs.
  - Reduce maintenance overhead for link updates.
  - Tighten documentation governance consistency.
- Linked docs updated:
  - `README.md`
  - `docs/README.md`
  - `docs/product/01-mvp-plan.md`
  - `docs/product/02-prd.md`
  - `docs/product/04-dependencies-risks.md`
  - `docs/planning/README.md`
  - `docs/planning/iterations/README.md`
  - `docs/planning/sprint-board-template.md`
  - `docs/operations/README.md`
  - `docs/operations/contributor-guide.md`
  - `docs/architecture/adr-0001-stack-baseline.md`

## Related Documents

- `docs/planning/iterations/01-sprint-board.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/planning/iterations/03-sprint-board.md`
- `docs/planning/iterations/04-sprint-board.md`
