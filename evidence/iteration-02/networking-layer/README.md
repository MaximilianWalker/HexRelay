# Iteration 2 Networking-Layer Evidence

## Scope

- Story: `T4.0.1`
- Story title: `Define shared communication layer interfaces and policy engine boundary`

## Delivered Evidence

- Shared communication-layer boundary types in `crates/communication-core/src/domain/communication.rs`
- Policy routing boundary in `crates/communication-core/src/app/policy.rs`
- Shared router in `crates/communication-core/src/app/router.rs`
- Current server-channel adapter usage in `services/api-rs/src/domain/server_channels/realtime.rs`
- Current presence adapter usage in `services/realtime-rs/src/domain/presence.rs`

## Validation Commands

```powershell
cargo test -p communication-core
cargo test
```

Run the second command in both:

- `services/api-rs`
- `services/realtime-rs`

## Expected Outcome

- `CommunicationMode::DmDirect` routes to `TransportProfile::DirectPeer`
- `CommunicationMode::ServerChannel` routes to `TransportProfile::NodeClient`
- `CommunicationMode::Presence` routes to `TransportProfile::NodeClient`
- deterministic policy/provenance assertions are stable across shared tests and current integrations
