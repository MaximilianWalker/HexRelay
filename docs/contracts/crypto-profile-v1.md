# MVP Crypto Profile v1 Contract

## Document Metadata

- Doc ID: crypto-profile-v1
- Owner: Core and security maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-11
- Source of truth: `docs/contracts/crypto-profile-v1.md`

## Quick Context

- Runtime crypto contract authority for MVP identity/auth and E2EE interoperability requirements.
- Update this file when algorithm, nonce/replay, key-rotation, or crypto error-code requirements change.
- Latest meaningful change: 2026-03-11 aligned auth verify failure semantics to a single non-enumerating invalid code while preserving explicit invalid-code requirements.

## Purpose

- Define executable crypto requirements for MVP auth and E2EE DM interoperability.
- Provide testable criteria aligned to `docs/product/01-mvp-plan.md`.

## Requirements

| Req ID | Category | Requirement (normative) | Verification target |
|---|---|---|---|
| CP-ALG-001 | Algorithms | Identity signing uses Ed25519. | Sign/verify path rejects non-Ed25519 keys for identity signatures. |
| CP-ALG-002 | Algorithms | Session key exchange uses X25519 + HKDF-SHA256. | Handshake derives session material only from X25519 shared secret and HKDF-SHA256. |
| CP-ALG-003 | Algorithms | E2EE DM payload encryption uses XChaCha20-Poly1305. | DM encrypt/decrypt path uses AEAD XChaCha20-Poly1305 and fails tampered ciphertext. |
| CP-ALG-004 | Algorithms | Signature payload canonicalization uses UTF-8 JSON with sorted keys. | Equivalent payloads with different key order produce identical signature input bytes. |
| CP-NONCE-001 | Nonce | Auth challenge nonce entropy is at least 96 bits. | Nonce generator and tests enforce minimum entropy/length threshold. |
| CP-NONCE-002 | Nonce | Auth challenge nonce is single-use with 60-second TTL. | First valid verification succeeds; reuse or expired nonce fails. |
| CP-REPLAY-001 | Replay | Nonce identifier is persisted until TTL expiry for replay detection. | Nonce storage lifecycle proves duplicate detection during TTL window. |
| CP-REPLAY-002 | Replay | Duplicate nonce usage is rejected deterministically. | Duplicate verify attempts return `nonce_invalid`. |
| CP-ROTATE-001 | Key rotation | DM session keys rotate every 100 messages or 24 hours, whichever comes first. | Rotation triggers at both count and age boundaries in test fixtures. |
| CP-ERR-001 | Error codes | Crypto verification failures return explicit `*_invalid` error codes. | Failure responses map to deterministic invalid-code families. |
| CP-ERR-002 | Error codes | Error responses do not leak secret-bearing detail. | Negative-path responses omit keys, raw nonces, plaintext, and signature internals. |

## Minimum Error Code Expectations

- Required for Iteration 1 auth 401 verification failures: `nonce_invalid`, `session_invalid`.
- Required for Iteration 1 auth 400 validation errors: `signature_invalid`.
- Additional crypto failures must follow the same explicit `*_invalid` naming pattern.
- Error payloads may include stable machine-readable code and generic message only.
