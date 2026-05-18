# Architecture Quality Audit

## Metadata

- topic_id: 03-architecture
- topic: Architecture
- last_audited: 2026-05-18T22:37:00Z
- source_of_truth: `docs/operations/quality-audits/03-architecture.md`

## Investigation Focus

- Check service, module, API, realtime, storage, and UI boundaries for hidden coupling or unclear ownership.
- Flag architecture drift only when it conflicts with canonical docs or makes future changes materially riskier.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| _none_ | | | | | | |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| QA-03-20260512-domain-transport-boundaries | P2 | fixed | Domain modules owned transport adapter and live connection concerns. | Reclassified API node-forwarding, realtime DM dispatch, and server-channel dispatch modules into `services/api-rs/src/transport/http/adapters/`; reclassified the API outbound-forward retry worker into `services/api-rs/src/app/dm_outbound_forwarding.rs`; reclassified realtime Redis/websocket IO services into `services/realtime-rs/src/app/{channels,dms,presence}.rs`; `rg -n "NodeDispatch|TransportError|reqwest|axum::http::HeaderMap|crate::state::AppState|sync::mpsc::Sender|ConnectionSenderEntry" services/api-rs/src/domain services/realtime-rs/src/domain` now returns no matches; `cargo check --all-targets --all-features` passes. | 2026-05-18 |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-18T22:37:00Z | Codex | Fixed QA-03-20260512-domain-transport-boundaries by moving transport/live connection adapters out of domain modules and validating domain boundary grep plus Rust all-target check. |
| 2026-05-12T21:15:24Z | Codex | Added 1 P2 found finding about domain modules retaining transport and live connection responsibilities. |
