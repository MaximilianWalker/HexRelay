# HexRelay Local Runtime Testing Plan

## Document Metadata

- Doc ID: local-runtime-testing-plan
- Owner: Platform and QA maintainers
- Status: ready
- Scope: repository
- last_updated: 2026-05-20
- Source of truth: `docs/planning/local-runtime-testing-plan.md`

## Quick Context

- Purpose: define the local testing profile, fixture, multi-instance runtime, and network simulation plan for HexRelay development.
- Primary edit location: update this file when local fixture profiles, dev-session bootstrap, runtime profiles, or network simulation strategy changes.
- Latest meaningful change: 2026-05-20 moved shared local fixture and profile JSON under top-level `fixtures/`, keeping `scripts/` focused on executable automation.

## Organization Decision

- Canonical authority: `docs/planning/local-runtime-testing-plan.md`.
- Reason: the work spans `apps/web`, `services/api-rs`, `services/realtime-rs`, `infra`, `scripts`, and evidence docs, so it belongs in `docs/planning/` with the other cross-repo test-profile plans.
- Related verification docs remain in `docs/testing/` and should link here instead of duplicating profile or fixture definitions.
- Runtime environment variable details remain in `docs/reference/runtime-config-reference.md` and should be linked when implementation introduces new config flags.
- Operational how-to guidance lives in `docs/operations/local-runtime-testing-quickstart.md`, while this file remains the source of truth for design, scope, sequencing, and acceptance criteria.

## Scope

### In Scope

- Dev-only seeded users and testing profiles.
- Deterministic local fixture data for identities, sessions, contacts, DM policy, DM history, device state, server membership, and channel messages.
- Script UX for seed, reset, status, stop, multi-instance launch, and network simulation.
- Web UX for selecting seeded personas and opening DM-ready scenarios.
- Local multi-instance runtime profiles for testing multiple HexRelay app instances on one machine.
- Local network simulation that supports offline, partition, Docker-only latency/timeout profiles, and app-level deterministic fault injection.
- Windows and Linux npm command support.
- Validation gates and evidence expectations.

### Out of Scope

- Production seed data.
- Cloud-hosted test environments.
- Server-readable plaintext, private-key custody, unencrypted DM mailboxing, or plaintext relay behavior for DM transport.
- Dedicated-server scale testing beyond local multi-instance development ergonomics.
- Full encrypted DM protocol redesign.
- Voice/media TURN/NAT validation, which remains covered by `docs/planning/turn-nat-test-profile.md`.

## Current Repository Baseline

- Root scripts currently expose setup, seed/reset, runtime start/status/stop, profile validation, runtime/network smoke tests, standard tests, and security checks through `package.json`.
- Host-process start/status/stop is implemented once in `scripts/runtime/local.mjs`; `npm run start`, `npm run status`, and `npm run stop` call that shared manager directly.
- Host-process lifecycle commands are single Node entrypoints, exposed through npm for the same Windows and Linux workflow.
- The shared manager chooses conflict-free local ports, uses the shared runtime profile JSON files, isolates each managed Next dev server in a per-run `.next-*` directory, and prints each instance's API, realtime, and web URLs.
- Local infra uses `infra/docker-compose.yml` for Postgres, Redis, MinIO, and a legacy coturn service.
- The default host-process runtime profile starts one neutral `local-server` app instance without seed persona metadata; Docker runtime/network testing uses `infra/docker-compose.runtime-test.yml` for containerized Alice/Bob API, realtime, and web instances with `alice-server`/`bob-server` network targets and Toxiproxy inter-server links.
- API migrations already provide the tables needed for realistic local profiles: `identity_keys`, `sessions`, `friend_requests`, `local_server`, `server_memberships`, `dm_policies`, `dm_profile_devices`, `dm_threads`, `dm_thread_participants`, `dm_messages`, `server_channels`, `server_roles`, and `server_channel_messages`.
- Web personas currently live in browser local/session storage through `apps/web/lib/personas.ts` and `apps/web/lib/sessions.ts`.
- Backend DM history, policy, fanout, catch-up, and profile-device APIs exist; the browser DM route exists, while full end-to-end client encryption remains incremental.
- The DM envelope delivery guardrail is active and must not be weakened by testing features: message servers may carry/store ciphertext envelopes only, while plaintext and private keys remain client/device-only.
- API integration coverage includes a local two-server HTTP smoke that runs two API routers, signs server-forwarded DM envelopes, forwards over actual HTTP, and verifies DB-backed encrypted-envelope acceptance plus sender-side outbound forwarding state.

## Guiding Principles

- Use real local API data rather than browser-only fake users.
- Preserve the current auth model by creating signed dev sessions instead of bypassing authentication.
- Keep every fixture idempotent and resettable.
- Refuse destructive seed/reset commands against non-local databases.
- Prefer deterministic profiles over ad hoc manual setup.
- Make Windows the baseline for usability, not an afterthought.
- Keep network simulation local and deterministic for DM; tests must validate baseline encrypted-envelope delivery without router/cloud/recipient-device transport setup.
- Make simulated failures explicit and observable in UI and logs.

## Testing Profile Catalog

### Profile Naming

- Use stable profile IDs in the form `<name>.<role>`.
- Use stable identity IDs with the prefix `usr-test-`.
- Use stable fixture IDs with the prefix `fixture-`.
- Store shared fixture and profile definitions under top-level `fixtures/`; keep test-private fixtures beside their owning test harness under `tests/`.
- Keep profile purpose text short enough to display in the web testing UI.

### Required Profiles

| Profile ID | Identity ID | Purpose | Expected State |
|---|---|---|---|
| `alice.primary` | `usr-test-alice` | Happy-path sender and primary manual-test persona | Active session, accepted contact with Bob, member of shared server, DM policy `friends_only` |
| `bob.primary` | `usr-test-bob` | Happy-path peer and receiver | Active session, accepted contact with Alice, multiple profile devices |
| `carol.pending` | `usr-test-carol` | Pending contact/request edge case | Pending friend request with Alice, no accepted DM relationship |
| `dave.restricted` | `usr-test-dave` | Negative DM policy and blocked/restricted behavior | Restrictive policy and relationship state that should block or require context |
| `erin.offline` | `usr-test-erin` | Offline/connectivity test identity | Identity and device records exist, no active session |

### Required Scenario Profiles

| Scenario ID | Purpose | Included Profiles | Data Shape |
|---|---|---|---|
| `dm-basic` | Fast manual testing for DM surfaces | Alice, Bob | Accepted friendship, DM policy, profile-device records, one encrypted DM thread |
| `contacts-edge` | Contacts UI and request-state validation | Alice, Carol, Dave | Pending inbound/outbound requests and restricted policy |
| `server-chat` | Server/channel workspace validation | Alice, Bob, Carol | Shared server, memberships, channels, server messages, unread/pinned/muted variation |
| `multi-device` | Device convergence checks | Alice, Bob, Erin | Multiple Bob devices plus active/inactive profile-device variation |
| `all` | Complete local exploratory dataset | All profiles | Combined DM, contacts, server, device, and policy states |

## Fixture Data Model

### Identity Fixtures

- Insert each testing identity into `identity_keys` with deterministic dev-only Ed25519 public key material.
- Store any private key material only in dev fixture files if needed for browser onboarding tests.
- Prefer generated dev-only keys if real signing flows require valid signatures.
- Mark fixture keys as non-production in docs and seed output.

### Session Fixtures

- Create signed sessions through a dev-only bootstrap command or endpoint.
- Insert active sessions into `sessions` where backend persistence is required.
- Mirror session IDs into browser session storage only through the web testing profile UX.
- Use realistic expiration timestamps far enough in the future for local manual testing.
- Never hardcode production signing keys.

### Contacts and Friend Requests

- Alice and Bob must have an accepted relationship.
- Alice and Carol must have a pending request state.
- Dave must cover negative policy and request-state behavior.
- Friend request rows must use stable IDs when possible, such as `fixture-fr-alice-bob`.
- The seed command must remain idempotent when accepted or pending rows already exist.

### DM Policy and Device Data

- Alice default policy: `friends_only`.
- Bob default policy: `friends_only`.
- Carol may use `same_server` for context-sensitive behavior.
- Dave should use a restrictive setup to validate blocked or not-authorized behavior.
- Bootstrap fixtures must not include recipient-device endpoint hints, endpoint cards, pairing QR/manual-code payloads, or LAN/WAN discovery data.

### DM Thread and Message Data

- Create one direct Alice/Bob DM thread with stable participants.
- Include at least five messages with alternating authors.
- Include unread state by setting Alice and Bob `last_read_seq` differently.
- Include one older thread or empty thread for pagination and empty-state checks.
- Use ciphertext placeholder values that clearly indicate fixture data, such as `fixture-ciphertext-alice-001`, until real client encryption is wired end-to-end.

### Server and Channel Data

- Create one shared test server fixture. The JSON template keeps a stable fixture id, but the seed CLI binds it to the configured local server id at seed time, defaulting to `hexrelay-local-server`.
- Add Alice, Bob, and Carol as members with varied `pinned`, `muted`, and `unread_count` values.
- Create at least two text channels, for example `general` and `ops-lab`.
- Seed server channel messages with mentions and one reply where constraints allow it.

### Profile Device Data

- Bob should have at least two active profile devices.
- Erin should have an inactive or stale device record.
- Device fixtures should cover active, inactive, and stale profile-device states for fanout and catch-up outcomes.

## Seed and Reset Tooling

### Target Files

| File | Purpose |
|---|---|
| `services/api-rs/src/bin/seed_dev.rs` | Rust entrypoint for transactional fixture seeding |
| `services/api-rs/src/dev_seed.rs` | Shared seed implementation used by CLI and tests |
| `fixtures/dev-seed/scenarios/*.json` | Versioned fixture catalog and scenario definitions |
| `scripts/seed.mjs` | Cross-platform seed command |
| `scripts/reset-dev-db.mjs` | Cross-platform local DB reset command; reseeds only when `--profile` is supplied |
| `package.json` | Root npm aliases for seed/reset commands |

### Target Commands

```bash
npm run seed -- --profile dm-basic
npm run seed -- --profile all
npm run reset-dev-db -- --yes
npm run reset-dev-db -- --profile all --yes
```

```text
npm run seed -- --profile dm-basic
npm run reset-dev-db -- --profile all --yes
```

### Seed Behavior

- Load `services/api-rs/.env` and `infra/.env` using existing script conventions.
- Verify `API_ENVIRONMENT=development` before writing data.
- Verify `API_DATABASE_URL` host is `localhost`, `127.0.0.1`, or an approved local Docker hostname.
- Refuse database names that are not local/dev names unless explicitly allowed by a dev-only override.
- Run all fixture writes in a transaction.
- Use upsert-style inserts where possible.
- Print a summary table after seeding.
- Exit non-zero on partial seed failure.

### Reset Behavior

- Stop local API/realtime/web processes if the runner owns them.
- Drop or truncate only local dev DB state.
- Re-run migrations.
- Leave the database empty unless a seed profile is explicitly supplied.
- Run the selected seed profile only when `--profile` is present.
- Print the generated local URLs and profile summary only when a seed profile is used with a runtime profile.

## Dev Session Bootstrap

### Objective

- Make seeded users immediately usable in local browsers without manual onboarding.
- Preserve existing auth/session behavior.
- Keep all bootstrap paths dev-only.

### Options

| Option | Recommendation | Notes |
|---|---|---|
| CLI session emission | Preferred first step | `seed-dev` can print session IDs/cookies for seeded users |
| Dev-only API endpoint | Useful after CLI works | Gate behind `API_ENABLE_DEV_TESTING=true` |
| Browser-only fake session | Not recommended | Would drift from server auth behavior |

### Target API/CLI Behavior

- Create session records for selected fixture identities.
- Sign session cookies with configured local signing keys.
- Support bearer-token output for API smoke tests.
- Support JSON output for web test automation.
- Refuse when `API_ENVIRONMENT=production`.
- Refuse unless `API_ENABLE_DEV_TESTING=true` if endpoint mode is added.

### Acceptance Criteria

- Alice can call `GET /contacts` with the generated session.
- Alice can call `GET /dm/privacy-policy` with the generated session.
- Dev bootstrap returns 403 or is absent when disabled.
- Production config cannot enable the bootstrap path accidentally.

## Web Testing UX

### Target Route

- Preferred route: `/settings/testing`.
- Initial implementation: a dev-only section inside `/settings`.

### Target Files

| File | Change |
|---|---|
| `apps/web/lib/api.ts` | Add dev fixture/session client methods after backend support exists |
| `apps/web/lib/personas.ts` | Add deterministic fixture persona import/switch helpers |
| `apps/web/lib/sessions.ts` | Add safe session write path for dev bootstrap response |
| `apps/web/app/settings/page.tsx` or `apps/web/app/settings/testing/page.tsx` | Add dev-only testing profile picker |
| `apps/web/lib/*test.ts` | Add unit coverage for persona/session fixture helpers |

### UI Requirements

- Show each seeded profile and its purpose.
- Show active/inactive session state.
- Show relationship state for key pairs, especially Alice/Bob.
- Show quick actions: activate persona, open contacts, open DM, open shared server, copy session details.
- Show scenario metadata and seed timestamp if available.
- Hide or disable the UI outside dev/test mode.

### Browser Multi-Profile Use

- Support separate browser contexts for Alice and Bob through Playwright tests.
- Support manual side-by-side use by opening separate web runtime URLs or separate browser profiles.
- Avoid relying on a single browser localStorage namespace for multiple active users.

## Multi-Instance Runtime Profiles

### Objective

- Start more than one local HexRelay instance for connection and multi-server testing.
- Avoid manual port edits.
- Keep instance logs and lifecycle clear.

### Runtime Profile Files

| File | Purpose |
|---|---|
| `fixtures/runtime/profiles/single.json` | One clean local server app instance with no seeded persona |
| `fixtures/runtime/profiles/dual.json` | Alice server plus Bob server |
| `fixtures/runtime/profiles/triple.json` | Alice, Bob, and Carol/Dave edge server |

### Runtime Profile Shape

The default `single` profile uses a neutral `local-server` id and omits `seedPersona`. Persona-specific runtime ids are reserved for explicit fixture profiles such as `dual` and `triple`.

```json
{
  "name": "dual",
  "instances": [
    {
      "id": "alice-server",
      "apiPort": 18080,
      "realtimePort": 18081,
      "webPort": 3002,
      "seedPersona": "alice.primary"
    },
    {
      "id": "bob-server",
      "apiPort": 18180,
      "realtimePort": 18181,
      "webPort": 3012,
      "seedPersona": "bob.primary"
    }
  ]
}
```

### Target Commands

```text
npm run start -- --runtime-profile dual --seed-profile dm-basic
npm run status
npm run stop -- --runtime-profile dual
```

### Runtime Behavior

- Each instance gets unique API, realtime, and web ports.
- Each instance writes logs under `.local-run/<instance-id>/`.
- Each instance prints its API, realtime, websocket, and web URLs.
- Each web instance receives matching `NEXT_PUBLIC_API_BASE_URL` and `NEXT_PUBLIC_REALTIME_WS_URL`.
- Status commands report process IDs, health, ports, and active runtime profile.
- Stop commands only stop tracked local processes by default.
- Shared infra mode uses one Postgres/Redis/MinIO stack with per-instance naming or namespace rules.
- Isolated infra mode can be added later if shared state causes test ambiguity.

### Docker Runtime Test Stack

- Normal development remains host-process based through `npm run start`.
- Docker runtime testing is reserved for heavier runtime/network scenarios and CI-style validation.
- `infra/docker-compose.runtime-test.yml` starts per-server Postgres, Redis, MinIO, and two runtime servers.
- Each runtime server has API, realtime, and web containers sharing a server network namespace.
- Runtime-test host ports are bound to `127.0.0.1` only.
- `alice-server` exposes API `18080`, realtime `18081`, and web `3002` on loopback.
- `bob-server` exposes API `18180`, realtime `18181`, and web `3012` on loopback.
- Servers attach to per-server infra networks plus one shared simulation network so Docker network partitions do not sever local Postgres/Redis/MinIO connectivity or leave an alternate Alice/Bob peer path through shared infra.
- `.local-run/runtime-state.json` records `containerName` and simulation `networkName` metadata so `npm run network` can resolve `alice-server` and `bob-server` as Docker targets.
- Docker runtime seeding prints dev session cookies/headers, but the web Settings testing-profile picker remains disabled in this stack because API dev-testing endpoints require loopback-only API/database binds.
- `runtime:docker -- up --seed-profile <profile>` seeds both Alice and Bob server databases.

### Docker Runtime Commands

```bash
npm run runtime:docker -- up --seed-profile dm-basic
npm run runtime:docker -- status
npm run network -- --profile offline-alice
npm run network -- --reset
npm run runtime:docker -- down
```

Use `npm run runtime:docker -- down --force` only for failed-smoke cleanup when the normal reset path cannot complete.
Generic `npm run stop` refuses Docker runtime state; use `runtime:docker -- down` so containers, network state, and runtime-test data volumes are cleaned together.

```bash
npm run test:runtime
npm run test:network
node scripts/runtime/docker.mjs smoke --scope runtime --evidence-dir .local-run/evidence/runtime-smoke
```

### Windows Parity

- Windows remains first-class.
- Runtime profile parsing should be shared across Windows and Unix through JSON files.
- Windows should continue choosing conflict-free ports when profile ports are unavailable.
- Windows status/stop scripts should not require interactive shells.

## Network Simulation

### Technology Stack

| Layer | Technology | Primary Use | Windows Support | Realism |
|---|---|---|---|---|
| Docker network controls | `docker network connect/disconnect`, named compose networks | Offline, reconnect, partitions | Good | Medium |
| Docker-only peer proxy | Toxiproxy TCP proxies in the runtime stack | Latency, jitter, timeout behavior | Good | Medium |
| Dev-only app fault injection | Realtime env-gated delay/drop/disconnect hooks | Deterministic websocket failures | Good | Medium |
| Browser/runtime isolation | Separate browser contexts plus runtime profiles | Alice/Bob side-by-side testing | Good | Medium |

### Network Profile Files

| File | Purpose |
|---|---|
| `fixtures/network/profiles/normal.json` | Clear all shaping and partitions |
| `fixtures/network/profiles/high-latency.json` | Add fixed latency to selected target |
| `fixtures/network/profiles/packet-loss.json` | Force peer-link loss with Toxiproxy timeout toxicity |
| `fixtures/network/profiles/offline-alice.json` | Disconnect Alice server from selected network |
| `fixtures/network/profiles/partition-alice-bob.json` | Block Alice and Bob from reaching each other |
| `fixtures/network/profiles/flaky-mobile.json` | Delay plus intermittent disconnect/failure behavior |

### Target Commands

```text
npm run network -- --profile offline-alice
npm run network -- --profile partition-alice-bob
npm run network -- --reset
```

### Docker Network Controls

- Use named Docker networks for runtime profile groups.
- Use disconnect/reconnect for offline simulation.
- Use separate networks for partition simulation.
- Keep reset commands deterministic and idempotent.
- Record applied network profile state under `.local-run/network-state.json`.
- `network --reset --force` is reserved for runtime cleanup after failed Docker smoke paths and may remove stale local network-state metadata after best-effort restore.
- Current `single`/`dual`/`triple` runtime profiles launch host processes, so Docker network actions fail safe for those host-process instance IDs.
- The Docker runtime test stack writes container metadata for `alice-server` and `bob-server`, so Docker-backed profiles can apply against those instance IDs.
- Docker-backed profiles act on the runtime simulation network, not the infra network.
- Infra dependencies are per-server; Alice and Bob do not share Postgres, Redis, MinIO, or an infra network with each other.

### Toxiproxy Peer Links

- Use Toxiproxy in the Docker runtime stack for latency, jitter, and timeout-based TCP simulations.
- Runtime state records Toxiproxy proxy metadata so `network --profile high-latency --target alice-server` applies to Alice's proxied peer links.
- `packet-loss` is a deterministic timeout approximation, not packet-level loss.
- Toxiproxy profiles require Docker runtime targets; host-process runtime profiles fail safe.
- Keep Toxiproxy reset deterministic by deleting active toxics recorded in `.local-run/network-state.json`.

### Dev-Only App Fault Injection

- Realtime exposes internal dev-fault endpoints only when `REALTIME_ENABLE_DEV_FAULTS=true`.
- `npm run start` and `runtime:docker` enable realtime dev faults for local testing; production config rejects them.
- Non-loopback realtime binds require a non-default channel dispatch internal token when dev faults are enabled.
- `flaky-mobile` maps to realtime delay, deterministic drop rate, and disconnect-after settings through `npm run network -- --profile flaky-mobile --target <instance>`.
- `network --reset` restores the previous realtime dev-fault config.

### DM Envelope Delivery Guardrail

- Network simulation must not add server-readable plaintext, private-key custody, unencrypted mailboxing, or plaintext relay behavior to DM runtime behavior.
- Optional server-to-server failures under simulated networks should fail explicitly with user-visible guidance without blocking baseline encrypted-envelope message-server delivery.
- Voice/media TURN/NAT tests remain separate under `docs/planning/turn-nat-test-profile.md`.
- The existing `scripts/validators/dm-transport-policy.mjs` guardrail should be extended if new runtime/config surfaces can affect DM plaintext, private keys, or envelope storage semantics.

## Validation Strategy

### Unit and Integration Tests

| Area | Validation |
|---|---|
| Fixture parser | Unit test scenario parsing, required field validation, and stable IDs |
| Seed command | Integration test idempotent inserts against test Postgres |
| Reset command | Script smoke test against local-only test DB name |
| Dev session bootstrap | API tests for enabled and disabled modes |
| Web persona helpers | Vitest coverage for fixture persona import/switch/session write |
| Runtime profile parser | Unit tests for port conflicts, missing fields, and profile matrix |
| Network profile parser | Unit tests for scenario validation and reset state |

### Browser Tests

- Use isolated browser contexts for Alice and Bob.
- Activate Alice through the dev testing profile UI.
- Activate Bob through the dev testing profile UI.
- Confirm Alice sees Bob in contacts.
- Confirm Alice can open the Bob DM route.
- Confirm seeded DM history appears once the web route is wired to backend history.
- Confirm restricted/pending profiles show blocked or pending UI states.

### Runtime Smoke Tests

- `single` profile starts and becomes healthy.
- `dual` profile starts and both web instances load.
- `status` reports every tracked instance.
- `stop` stops every tracked process.
- `network --reset` restores connectivity after each simulated network scenario.
- `test:runtime` validates app-level Alice/Bob API reachability before, during, and after Docker offline/partition profiles.
- `test:runtime` validates Toxiproxy peer-link latency and timeout apply/reset without kernel-level network shaping.
- `test:runtime` validates realtime app-fault apply/reset against the runtime stack.
- `test:network` runs the same Docker runtime network scenario set explicitly through `tests/runtime/network-smoke.mjs`.
- `scripts/runtime/docker.mjs smoke --evidence-dir <path>` writes raw smoke output files that can be copied under a durable evidence bundle's `outputs/` directory.

### CI Strategy

- Keep standard `npm run test` CI-safe and deterministic.
- Keep heavier runtime/network checks behind `npm run test:runtime` and the separate `runtime-network-smoke` CI job.
- Run seed parser and API fixture invariants in CI.
- Keep Toxiproxy coverage in the Docker runtime smoke so normal test jobs do not need extra runtime services.

## Evidence Artifacts

### Durable Evidence Paths

- Fixture seed summaries: `evidence/local-runtime-testing/fixtures/<run-id>/`.
- Runtime profile smoke outputs: `evidence/local-runtime-testing/runtime/<profile>/<run-id>/`.
- Network simulation runs: `evidence/local-runtime-testing/network/<profile>/<scenario>/<run-id>/`.
- Browser scenario outputs: `evidence/local-runtime-testing/browser/<scenario>/<run-id>/`.

### Required Durable Evidence Per Run

- `summary.md`: requirement IDs, scope, outcome, owner, and missing-artifact rationale if any output is unavailable.
- `validators.txt`: exact commands or manual checks run.
- `provenance.json`: commit SHA, PR or run ID, and generation timestamp.
- `outputs/`: raw generated artifacts, screenshots, logs, or exports referenced by `summary.md`.

### Runtime/Network Smoke Output Files

- `outputs/scenario-config.json`: runtime profile, network profile, targets, and shaping parameters.
- `outputs/runtime-status-before.json`: health and ports before simulation.
- `outputs/runtime-status-after.json`: health and ports after reset.
- `outputs/event-log.ndjson`: ordered events for apply, observe, fail/pass, and reset.
- `outputs/verdict.md`: explicit pass/fail outcome with failed checks.

## Implementation Phases

| Phase ID | Title | Objective | Status |
|---|---|---|---|
| PH-01 | Fixture foundation | Define deterministic testing profiles and backend fixture catalog | done |
| PH-02 | Seed/reset tooling | Add safe local seed and reset commands | done |
| PH-03 | Dev sessions and web profile UX | Make seeded users easy to activate in browser sessions | done |
| PH-04 | Multi-instance runtime profiles | Start multiple local app instances with clear lifecycle and ports | done |
| PH-05 | Network simulation | Add local offline, partition, latency, and deterministic fault simulation | done |
| PH-06 | Validation and evidence | Add tests and evidence outputs for fixture, runtime, and network workflows | done |
| PH-07 | Documentation and adoption | Add runbook summaries and troubleshooting docs | done |

### PH-01 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-01-EP-01-ST-01-TK-01 | Define fixture catalog schema | `fixtures/dev-seed/`, seed command parser | Unit test parser | Schema supports profiles, identities, sessions, contacts, DM data, server data, and devices | done |
| PH-01-EP-01-ST-01-TK-02 | Add `dm-basic` fixture profile | `fixtures/dev-seed/scenarios/dm-basic.json` | Seed dry run | Alice and Bob are DM-ready and documented | done |
| PH-01-EP-01-ST-01-TK-03 | Add contacts edge fixture profile | `fixtures/dev-seed/scenarios/contacts-edge.json` | `cargo test -p api-rs dev_seed`; seed dry run | Pending and restricted states are reproducible | done |
| PH-01-EP-01-ST-01-TK-04 | Add server chat fixture profile | `fixtures/dev-seed/scenarios/server-chat.json` | `cargo test -p api-rs dev_seed`; seed dry run | Shared server, channels, memberships, and messages exist | done |

### PH-02 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-02-EP-01-ST-01-TK-01 | Add Rust seed implementation | `services/api-rs/src/bin/seed_dev.rs`, `services/api-rs/src/dev_seed.rs` | `cargo test -p api-rs dev_seed` | Transactional idempotent seed for selected profile | done |
| PH-02-EP-01-ST-01-TK-02 | Add seed command | `scripts/seed.mjs` | `npm run seed -- --help` | Command loads env and calls seed implementation consistently on Windows and Linux | done |
| PH-02-EP-01-ST-01-TK-03 | Add local reset command | `scripts/reset-dev-db.mjs` | `npm run reset-dev-db -- --yes`; `npm run reset-dev-db -- --yes --profile dm-basic`; `npm run seed -- --profile dm-basic --json` | Reset refuses unsafe DB, resets cleanly without fixture data by default, and reseeds only when a profile is supplied | done |
| PH-02-EP-01-ST-01-TK-04 | Add root npm aliases | `package.json` | `npm run seed -- --help`, `npm run reset-dev-db -- --help` | Commands are discoverable from repo root | done |

### PH-03 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-03-EP-01-ST-01-TK-01 | Add dev session bootstrap mode | `services/api-rs` auth/session modules | `cargo test -p api-rs dev_testing` | Sessions are valid only in dev-enabled mode | done |
| PH-03-EP-01-ST-01-TK-02 | Add web testing profile picker | `apps/web/app/settings/page.tsx`, `apps/web/lib/personas.ts`, `apps/web/lib/sessions.ts` | Browser smoke and Vitest | Seeded profiles can be activated with one click | done |
| PH-03-EP-01-ST-01-TK-03 | Gate web UX in dev mode | `apps/web` env/config | Production build check | Testing UI hidden or inert outside dev/test mode | done |

### PH-04 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-04-EP-01-ST-01-TK-01 | Define runtime profile JSON schema | `fixtures/runtime/profiles/` | `npm run validate:runtime-profiles` | `single`, `dual`, and `triple` profile files validate | done |
| PH-04-EP-01-ST-01-TK-02 | Implement shared host-process runtime manager | `scripts/runtime/local.mjs`, `scripts/run.mjs`, `scripts/status.mjs`, `scripts/stop.mjs` | `npm run start -- --runtime-profile dual --seed-profile dm-basic`; `npm run status`; `npm run stop` | Starts multiple named instances with unique ports from one cross-platform implementation | done |
| PH-04-EP-01-ST-01-TK-03 | Keep one cross-platform lifecycle command surface | `scripts/run.mjs`, `scripts/status.mjs`, `scripts/stop.mjs` | `npm run start -- --help`; `npm run status -- --help`; `npm run stop -- --help` | Windows and Linux users share the same Node command implementation | done |
| PH-04-EP-01-ST-01-TK-04 | Add status and stop commands | `scripts/runtime/local.mjs`, `scripts/status.*`, `scripts/stop.*` | Windows `single` and `dual` start/status/stop smoke | Processes are tracked and cleaned deterministically | done |

### PH-05 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-05-EP-01-ST-01-TK-01 | Add network profile schema | `fixtures/network/profiles/`, `scripts/validators/network-profiles.mjs` | `npm run validate:network-profiles` | Normal, offline, partition, latency, and flaky profiles validate | done |
| PH-05-EP-01-ST-01-TK-02 | Add Docker network simulation command | `scripts/network.mjs` | `npm run network -- --reset --json`; parser checks | Command layer and idempotent reset exist; Docker container targets are supported, while current host-process runtime targets fail safe | done |
| PH-05-EP-01-ST-01-TK-03 | Add Toxiproxy latency/timeout support | `scripts/network.mjs`, `infra/docker-compose.runtime-test.yml` | Apply and reset latency/timeout profiles; `npm run test:runtime` | Docker runtime targets support cross-platform peer-link degradation without kernel shaping | done |
| PH-05-EP-01-ST-01-TK-04 | Add dev app fault injection | `services/realtime-rs`, `scripts/network.mjs` | Realtime integration tests; `npm run test:runtime` | Delay/drop/disconnect knobs work only in dev/test mode | done |
| PH-05-EP-01-ST-01-TK-05 | Add Docker runtime test stack | `infra/docker-compose.runtime-test.yml`, `scripts/runtime/docker.mjs`, `package.json` | Compose config validation; `npm run runtime:docker -- status --json`; `npm run test:runtime` | Alice/Bob containerized runtime servers expose API/realtime/web endpoints and validate offline, partition, Toxiproxy, app-fault, and reset paths | done |

### PH-06 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-06-EP-01-ST-01-TK-01 | Add fixture invariant tests | `services/api-rs/src/dev_seed.rs` | `cargo test -p api-rs fixture` | Seeded profiles match expected local runtime scenario invariants | done |
| PH-06-EP-01-ST-01-TK-02 | Add web persona tests | `apps/web/lib/personas.test.ts`, `apps/web/lib/sessions.test.ts` | `npm run test --prefix apps/web` | Persona/session helpers are covered | done |
| PH-06-EP-01-ST-01-TK-03 | Add runtime smoke tests | `tests/runtime/runtime-smoke.mjs`, `scripts/runtime/docker.mjs` | `npm run test:runtime`; `node scripts/runtime/docker.mjs smoke --scope runtime` | Docker runtime health checks pass and optional evidence can be emitted | done |
| PH-06-EP-01-ST-01-TK-04 | Add network reset smoke tests | `tests/runtime/network-smoke.mjs`, `scripts/runtime/docker.mjs` | `npm run test:network`; `npm run test:runtime` | Reset restores baseline connectivity after offline, partition, Toxiproxy, and app-fault profiles | done |

### PH-07 Tasks

| Task ID | Task | Touchpoints | Validation | Acceptance Criteria | Status |
|---|---|---|---|---|---|
| PH-07-EP-01-ST-01-TK-01 | Add operations quickstart after implementation | `docs/operations/local-runtime-testing-quickstart.md`, `docs/operations/README.md`, `README.md` | Docs review against clean-checkout flow | Developer can seed, launch, inspect, stop, run Docker runtime smoke, and troubleshoot profiles from docs | done |
| PH-07-EP-01-ST-01-TK-02 | Update runtime config reference when env flags land | `docs/reference/runtime-config-reference.md` | Docs review | Dev-only env vars have defaults, production requirements, and safety notes | done |
| PH-07-EP-01-ST-01-TK-03 | Update evidence and verification docs | `docs/testing/README.md`, `docs/testing/01-mvp-verification-matrix.md` | Docs review | Runtime and network evidence paths and files are discoverable | done |

## Dependencies and Critical Path

| Item | Depends On | Blocks |
|---|---|---|
| Fixture catalog | none | Seed command, web testing UX, fixture tests |
| Seed/reset tooling | Fixture catalog | Dev session UX, browser scenario tests |
| Dev session bootstrap | Seeded identities and sessions | Web profile picker, browser tests |
| Runtime profiles | Existing run scripts | Network simulation, multi-instance tests |
| Network profiles | Runtime profiles | Network scenario smoke tests |
| Validation/evidence | All earlier phases | Release readiness for testing runtime feature set |

Critical path:

1. Define `dm-basic` fixture catalog.
2. Implement seed command.
3. Implement dev session bootstrap.
4. Add web testing profile picker.
5. Add `dual` runtime profile.
6. Add offline/partition network simulation.
7. Add runtime and browser smoke tests.

## Decisions

| Decision ID | Context | Chosen Option | Rationale | Impact |
|---|---|---|---|---|
| DEC-01 | Fixture authority | Versioned fixture catalog plus Rust seed command | Keeps DB writes typed, transactional, and aligned with API migrations | Requires Node commands for good UX |
| DEC-02 | Auth model | Signed dev sessions, not browser-only fake users | Preserves server auth behavior | Requires dev-only bootstrap guard |
| DEC-03 | Runtime profiles | Named JSON profile files shared by Windows and Linux | Avoids OS-specific command drift | Requires a shared parser or strict schema convention |
| DEC-04 | Network simulation | Docker controls plus Toxiproxy plus app-level dev faults | Keeps Windows/Linux behavior Docker-only and deterministic | Toxiproxy is TCP-level rather than packet-level shaping |
| DEC-05 | DM transport | Validate encrypted-envelope message-server delivery as the only MVP DM transport path | Preserves product guardrail | Testing must not introduce plaintext/key custody, unencrypted relay behavior, or server-bypassing client DM transport/bootstrap surfaces |
| DEC-06 | Documentation authority | One planning authority with testing/operations indexes linking to it | Matches existing docs convention for test profiles | Avoids duplicating commands before implementation lands |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Seed/reset targets a non-local DB | High | Refuse unless env and database URL pass local allowlist checks |
| Static dev keys are mistaken for production material | Medium | Mark fixture key material as dev-only and avoid using production-looking names |
| Browser personas drift from backend sessions | Medium | Use API-backed dev sessions and only mirror session state into browser storage |
| Toxiproxy is TCP-level rather than packet-level | Medium | Use Docker partition/disconnect plus app-level realtime faults for deterministic MVP coverage |
| Runtime profiles create stale processes | Medium | Track PIDs in `.local-run/`, provide status and stop commands, refuse unsafe reuse |
| Network reset leaves containers partitioned | Medium | Store network state and make reset idempotent |
| DM delivery remains partially wired in UI | Medium | Validate backend history/fanout/catch-up now and document web delivery limitations until implemented |
| Legacy coturn service confuses DM testing | Medium | Document coturn as voice/media-only legacy test infra, not DM runtime support |

## Minimum Viable Delivery Slice

- Add `dm-basic` fixture profile with Alice and Bob.
- Add seed command and cross-platform Node commands.
- Add dev session bootstrap for Alice and Bob.
- Add dev-only web testing profile picker.
- Add `dual` runtime profile.
- Add offline and partition network simulation.
- Add docs and smoke tests for the above.

## Full Acceptance Criteria

- `npm run seed -- --profile dm-basic` creates Alice and Bob with valid local sessions and DM-ready backend state.
- Alice and Bob can be opened in separate browser contexts or local web instances.
- Alice sees Bob in contacts.
- Alice can open the Bob DM route.
- Alice and Bob can read seeded DM history once the web route consumes backend history.
- `dual` runtime starts without port collisions on Windows and Unix.
- `status` reports every instance and health URL.
- `stop` cleans every tracked local process.
- `network --profile offline-alice` makes Alice unreachable or disconnected in a visible way.
- `network --profile partition-alice-bob` degrades server/realtime connectivity without introducing plaintext/key custody, unencrypted relay behavior, or server-bypassing DM transport.
- `network --reset` restores normal connectivity.
- Dev session bootstrap is unavailable or inert outside development mode.
- Production builds do not expose test profile controls.

## Related Documents

- `README.md`
- `docs/README.md`
- `docs/operations/local-runtime-testing-quickstart.md`
- `docs/planning/README.md`
- `docs/testing/README.md`
- `docs/reference/runtime-config-reference.md`
- `docs/planning/kpi-slo-test-profile.md`
- `docs/planning/turn-nat-test-profile.md`
- `docs/product/10-infra-free-dm-connectivity-proposals.md`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/operations/dev-prerequisites.md`
