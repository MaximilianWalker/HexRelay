# HexRelay Glossary

## Document Metadata

- Doc ID: glossary
- Owner: Product and engineering maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-03-04
- Source of truth: `docs/reference/glossary.md`

## Quick Context

- Primary edit location for this document's canonical topic.
- Update this file when its source-of-truth topic changes.
- Latest meaningful change: 2026-03-04 documentation standardization pass.

## Terms

- Node: A self-hosted HexRelay server instance operated locally or on a VPS.
- Persona: A distinct user account/profile context available on the same device.
- Guild: A server/community space containing channels and role permissions.
- Profile capsule: Versioned user profile payload split into signed public data and encrypted private data.
- Federation-lite: MVP discovery model using signed registry metadata rather than full decentralized DHT.
- Cutover: Optional migration action that revokes old device sessions after successful import.
- `.hxb` bundle: Encrypted and signed migration package used in full device migration.
- Shared-server discovery: User discovery mode limited to people who share at least one guild/server context.
- Nonce challenge: Server-issued one-time value signed by the client key to prove identity ownership at login.
- Forward secrecy: Property where compromise of long-term keys does not expose past DM plaintext.
- Profile replica: Node-stored copy of profile capsule data, with signed public data and encrypted private data.
- Mediated friend request: Server-routed contact request where identity bootstrap data is shared only after recipient acceptance.
- DM inbound policy: Per-user rule controlling who can start DMs (default friends-only, optional same-server or anyone).
- DM offline outbox: Encrypted local sender queue used for best-effort retries when recipient is offline.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd-v1.md`
