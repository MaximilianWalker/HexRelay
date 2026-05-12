# Architecture Quality Audit

## Metadata

- topic_id: 03-architecture
- topic: Architecture
- last_audited: 2026-05-12T21:15:24Z
- source_of_truth: `docs/operations/quality-audits/03-architecture.md`

## Investigation Focus

- Check service, module, API, realtime, storage, and UI boundaries for hidden coupling or unclear ownership.
- Flag architecture drift only when it conflicts with canonical docs or makes future changes materially riskier.

## Active Findings

| ID | Priority | Status | Summary | Evidence | Next step | Last seen |
|---|---|---|---|---|---|---|
| QA-03-20260512-domain-transport-boundaries | P2 | found | Domain modules still own transport adapter and live connection concerns. | `docs/architecture/adr-0003-rust-service-module-architecture.md:86` requires mandatory layer direction, while `:96` says handlers parse IO and call domain services and `:100` assigns wire types to DTO modules. Current code keeps transport details in domain modules: `services/api-rs/src/domain/dm/realtime.rs:4` imports transport traits, `:17` owns an internal HTTP path, and `:78` stores a `reqwest::Client`; `services/api-rs/src/domain/dm/forwarding.rs:1` imports `axum::http::HeaderMap`, `:8` imports `reqwest::Url`, and `:28` owns another internal route path; realtime domain modules import `AppState`, `NodeDispatch`, mpsc senders, and live connection state at `services/realtime-rs/src/domain/presence.rs:3`, `:8`, `:14`, `:623`, with transport-shaped state defined at `services/realtime-rs/src/state.rs:12` and `:15`. | Move internal HTTP path constants, request/response wire structs, `reqwest`/`axum` dependencies, `NodeDispatch` implementations, and websocket sender-map mutation behind transport/app adapters so domain modules expose policy/data decisions rather than IO plumbing. | 2026-05-12 |

## Resolved Findings

| ID | Priority | Status | Summary | Resolution evidence | Resolved |
|---|---|---|---|---|---|
| _none_ | | | | | |

## Run History

| Date (UTC) | Auditor | Result |
|---|---|---|
| 2026-05-12T21:15:24Z | Codex | Added 1 P2 found finding about domain modules retaining transport and live connection responsibilities. |
