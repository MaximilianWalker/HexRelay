# T4.5.1 E2EE One-to-One Session Bootstrap Evidence

- artifact: `evidence/iteration-02/messaging-e2ee/2026-05-13/`
- validator: `cargo test -p communication-core e2ee --all-features`
- result: pass
- timestamp: 2026-05-13T14:21:32Z
- run_id: `hexrelay-autonomous-developer-2026-05-13T14:15:18Z`

## Scope

- `T4.5.1`: peers establish encrypted sessions with verifiable identity keys.
- The full selected `T4.5.x` cluster was split because 1:1 encryption/catch-up and group E2EE are larger follow-on tasks.
- `T4.5.2`, `T4.5.3`, and `T4.5.4` remain open.

## Evidence

- `communication-core` signs DM session bootstrap material with the participant Ed25519 identity key.
- The bootstrap payload binds the identity id, identity public key, X25519 session public key, thread id, participants, generation, creation time, and derived session id.
- The regression suite verifies matching client sessions, forged identity-key rejection, tampered-signature rejection, nonparticipant rejection, wrong-context rejection, XChaCha20-Poly1305 ciphertext round trip, tamper failure, and rotation-boundary signaling.

## Known Limits

- This evidence closes only one-to-one session bootstrap.
- Offline catch-up evidence remains owned by the DM encrypted-envelope delivery/catch-up task set.
- Group session bootstrap, membership rekey packaging, and group failure recovery remain owned by `T4.5.3` and `T4.5.4`.
