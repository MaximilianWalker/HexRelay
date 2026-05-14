# T4.5.1-T4.5.4 E2EE Session Bootstrap, Rotation, Group Rekey, and Recovery Evidence

- artifact: `evidence/iteration-02/messaging-e2ee/2026-05-13/`
- validator: `cargo test -p communication-core e2ee --all-features`
- result: pass
- timestamp: 2026-05-13T17:22:06Z
- run_id: `hexrelay-autonomous-developer-2026-05-13T17:17:18Z`

## Scope

- `T4.5.1`: peers establish encrypted sessions with verifiable identity keys.
- `T4.5.2`: one-to-one client encrypt/decrypt and rotation planning.
- `T4.5.3`: group DM session bootstrap and membership key-update planning.
- `T4.5.4`: group DM ciphertext encrypt/decrypt and missing-key recovery.

## Evidence

- `communication-core` signs DM session bootstrap material with the participant Ed25519 identity key.
- The bootstrap payload binds the identity id, identity public key, X25519 session public key, thread id, participants, generation, creation time, and derived session id.
- One-to-one regressions verify matching client sessions, forged identity-key rejection, tampered-signature rejection, nonparticipant rejection, wrong-context rejection, XChaCha20-Poly1305 ciphertext round trip, tamper failure, rotation-boundary signaling, rotated-session derivation, old-session rejection, and serialized encrypted results without plaintext.
- Group regressions verify member-scoped group bootstrap, nonparticipant rejection before deriving a client session, membership-change rekey plans with added/removed identity sets, removed-member rejection before deriving the next session, post-rekey decrypt by current members, old-session rejection for post-rekey traffic, and serialized encrypted results without plaintext.
- Group missing-key regressions verify post-rekey envelopes return `session_key_missing` before the next group session key arrives, reject one-to-one sessions from the group session ring, and decrypt successfully after the rekeyed member session is inserted.

## Known Limits

- Offline catch-up evidence remains owned by the DM encrypted-envelope delivery/catch-up task set.
