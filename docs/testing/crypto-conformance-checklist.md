# Crypto Conformance Checklist (MVP Profile v1)

## Document Metadata

- Doc ID: crypto-conformance-checklist
- Owner: QA and core maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/testing/crypto-conformance-checklist.md`

## Purpose

- Define per-requirement verification steps for `docs/contracts/crypto-profile-v1.md`.
- Standardize evidence required for release and audit readiness.

## Verification Checklist

| Req ID | Verification steps | Expected evidence artifact |
|---|---|---|
| CP-ALG-001 | Run identity signature tests with valid/invalid key types; assert only Ed25519 is accepted. | Test report showing pass/fail cases for accepted and rejected key types. |
| CP-ALG-002 | Run handshake integration test; confirm shared secret uses X25519 and KDF is HKDF-SHA256. | Captured test log with algorithm identifiers and successful key agreement. |
| CP-ALG-003 | Run DM encrypt/decrypt tests including tamper case; verify tampered payload fails auth. | Test output for round-trip success and tamper rejection. |
| CP-ALG-004 | Sign semantically identical JSON payloads with different key order; compare canonical bytes. | Fixture output proving identical canonical payload bytes and signature input. |
| CP-NONCE-001 | Generate nonce sample set in tests; assert entropy/length threshold meets >=96 bits. | Unit test artifact with nonce size/entropy assertion results. |
| CP-NONCE-002 | Verify first-use success, replay failure, and expiry failure at >60 seconds. | Integration test trace with success, duplicate failure, and expiry failure. |
| CP-REPLAY-001 | Inspect nonce persistence behavior across TTL window in auth tests. | Storage/state trace proving nonce id retention until TTL expiry. |
| CP-REPLAY-002 | Trigger duplicate nonce verify attempt and assert deterministic code. | API response capture showing `nonce_invalid` on duplicate. |
| CP-ROTATE-001 | Send 100 messages in one session and separately age a session past 24h; assert rotation in both paths. | Session metadata diff or logs showing rotation event by count and by age. |
| CP-ERR-001 | Execute negative crypto auth/decrypt scenarios; assert `*_invalid` codes. | Contract test report mapping each failure mode to expected code. |
| CP-ERR-002 | Review negative-path responses; assert no keys, raw nonces, plaintext, or signature internals are returned. | Sanitized response samples and test assertions for forbidden fields. |

## Evidence Package Minimum

- Automated test report (CI artifact) covering all CP-* requirements.
- API response samples for nonce/signature/session invalid paths.
- Session rotation evidence for both trigger modes (message-count and time-based).
