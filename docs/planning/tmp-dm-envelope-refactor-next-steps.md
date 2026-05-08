# Temporary DM Envelope Refactor Next Steps

## Status

- Temporary planning file for the post-pivot DM refactor.
- Created after commit `f0a036c` (`Complete DM E2EE envelope baseline pivot`).
- Remove or promote into canonical planning docs before final PR merge.

## Guardrails

- DM send success means durable acceptance of E2EE ciphertext envelopes by a server/message node.
- Do not claim active-device delivery until a realtime dispatch/ack path exists.
- Do not reintroduce user-to-user direct DM transport, pairing QR/manual code, endpoint cards, DM preflight/troubleshooter, LAN/WAN optimization, WAN wizard, or parallel dial.
- Do not add new DM delivery UX without explicit approval of flow, copy, and advanced controls.

## Action Plan

1. Commit the current green pivot as the rollback anchor.
   - Status: done in `f0a036c`.

2. Keep friend requests server-mediated and defer bootstrap naming cleanup.
   - Status: deferred by user preference; friend-request APIs remain in place.
   - Revisit route/schema naming only if it blocks clarity or external API stability.
   - Keep payload limited to accepted relationship identity key and profile-device material.
   - Avoid endpoint hints/cards or direct reachability fields.

3. Define realtime DM envelope dispatch and ack contracts.
   - Status: implemented in target-state contract in the current working tree after `604516e`.
   - Distinguish durable acceptance, realtime dispatch attempt, device ack, offline/pending, and catch-up replay.
   - Keep dispatch response as `pending_delivery` until ack evidence exists.

4. Implement realtime ciphertext-envelope dispatch and ack persistence.
   - Route ciphertext envelopes to active profile devices.
   - Persist idempotent device acks.
   - Populate `delivered_device_ids` only from confirmed ack evidence.

5. Tighten per-device catch-up cursor semantics.
   - Decide whether cursor advances on fetch, decrypt ack, or explicit delivery ack.
   - Cover offline replay, duplicate replay, restart persistence, and cross-device cursor isolation.

6. Reintroduce user-facing delivery states after UX approval.
   - Show accepted/pending/offline/catch-up states only.
   - Do not add preflight, direct-connect controls, or DM troubleshooting wizard behavior.

7. Keep guardrails and evidence current.
   - Extend `scripts/validate-dm-transport-policy.sh` for new surfaces.
   - Add migration evidence for schema changes.
   - Keep contract parity and docs freshness gates green.
