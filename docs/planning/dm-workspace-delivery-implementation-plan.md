# HexRelay DM Workspace Delivery Implementation Plan

## Document Metadata

- Doc ID: dm-workspace-delivery-implementation-plan
- Owner: Web, core, API, and delivery maintainers
- Status: approval_pending
- Scope: repository
- last_updated: 2026-05-15
- Source of truth: `docs/planning/dm-workspace-delivery-implementation-plan.md`

## Quick Context

- Purpose: sequence the DM workspace delivery gap without implementing product UI before explicit approval.
- Primary edit location: update this file when DM workspace flow, copy, controls, implementation slicing, approval evidence, or validation expectations change.
- Latest meaningful change: 2026-05-15 created the approval-pending plan for `QA-17-20260514-dm-workspace-send-not-wired`.

## Approval Boundary

This document is a plan-only artifact. It does not approve product UI implementation.

Runtime work that changes the DM workspace flow, copy, controls, or behavior must wait until the user explicitly approves the proposed package below. Until approval exists, allowed work is limited to planning, test/evidence design, documentation freshness, and audit-ledger routing.

## Source Authorities

| Authority | Role |
|---|---|
| `docs/product/01-mvp-plan.md` | Locked DM delivery, client-only plaintext/private-key, and UX approval decisions |
| `docs/product/02-prd.md` | Product requirements for E2EE private messaging, delivery diagnostics, and device convergence |
| `docs/product/08-screen-state-spec.md` | Required DM workspace states, delivery indicators, and UX approval gate |
| `docs/contracts/runtime-rest.openapi.yaml` | Current runtime REST endpoints for DM fanout, catch-up, threads, messages, and read state |
| `docs/architecture/04-communication-networking-layer-plan.md` | Server-node/message-node DM delivery and no node-bypassing UX constraints |
| `docs/planning/iterations/02-sprint-board.md` | Iteration 2 E2EE DM and messaging task context |
| `docs/operations/quality-audits/17-ux-product-quality.md` | Original quality finding and supersession record |

## Selected Cluster

| Source | Finding | Target |
|---|---|---|
| `QA-17-20260514-dm-workspace-send-not-wired` | DM workspace exposes a composer and send button, but accepted contacts still cannot send or load E2EE DM history. | After explicit approval, accepted contacts can load encrypted DM history, send client-encrypted envelopes through the server-node delivery path, and see deterministic loading, empty, blocked, policy, retry, and reconnect states. |

## Current Plan-Only Split Rationale

The selected cluster is UX-facing and explicitly requires the proposed DM workspace flow to be approved before implementation. The full runtime fix spans Web state, client-side encryption/session material, REST DM history/fanout APIs, realtime/catch-up reconciliation, and accessibility/evidence work. The smallest mergeable prerequisite is therefore this plan-only approval package.

This plan must not claim that DM workspace runtime acceptance criteria are complete. It only records the proposed flow, copy, controls, implementation slices, validation, and approval evidence required before a runtime branch begins.

## Plan-Only Change Scope

Allowed plan-only changes:

- clarify the proposed DM workspace flow, controls, copy, and state mapping;
- map pending approval decisions to the first implementation slice that needs them;
- define validation commands, evidence artifacts, and PR-body requirements;
- update docs indexes, product screen-state references, and quality-audit routing.

Disallowed without explicit UX approval:

- editing `apps/web` runtime UI, route behavior, visible fixtures, browser tests, or styles for the DM workspace;
- changing API/realtime contracts solely to support unapproved product behavior;
- adding evidence that claims DM workspace runtime delivery is complete;
- adding endpoint cards, preflight checks, WAN setup, LAN discovery, parallel dial controls, or other node-bypassing DM concepts.

Plan-only PRs may merge before UX approval because they do not change product behavior. The first runtime implementation PR must cite the exact approval reference and approved `DMW-APP-*` values it implements.

## Implementation Principles

- Keep DM plaintext and private keys client/device-only.
- Send only client-encrypted envelopes plus minimal delivery metadata to server nodes/message nodes.
- Prefer existing DM thread, fanout, catch-up, mark-read, and realtime helper contracts before adding new API shape.
- Treat API durable acceptance as `Sent`; recipient-device acknowledgement is the only source for `Delivered`.
- Keep read receipts separate from delivery acknowledgement and respect participant-visible read settings.
- Keep backend dispatch summaries out of user-visible delivery labels.
- Do not introduce DM connectivity preflight, troubleshooting wizard, endpoint-card, LAN/WAN, or recipient-device reachability UI.

## Proposed Flow Package

Status: approval_pending. The following package must be approved before runtime UI work begins.

Scope: accepted 1:1 contact DM workspace. Group DM presentation may reuse the same row states only if explicitly approved in the approval reference; otherwise group DM UI remains a later slice.

Flow:

1. User opens a contact DM workspace from Contacts Hub, a workspace tab, or a direct contact message route.
2. The workspace validates profile/session state, contact existence, accepted relationship state, block state, and inbound/outbound policy before enabling send.
3. For an accepted contact, the client loads the matching DM thread and message page from runtime DM history. If the existing API cannot deterministically map a contact to a thread, the first runtime slice must add the missing non-UX contract and tests before changing visible behavior.
4. The client decrypts messages locally from ciphertext envelopes and never sends plaintext or private keys to the server.
5. Empty history shows the approved empty state and keeps the composer enabled only when relationship, bootstrap, and policy checks pass.
6. Sending a non-empty message prepares client-side encrypted envelope payloads, adds a local pending row, and calls the server-node fanout dispatch path.
7. Durable API acceptance replaces the pending row with `Sent` state using the persisted message/thread identity.
8. Recipient-device ack or catch-up reconciliation can advance the row to `Delivered`; read state appears only when an approved participant-visible read receipt exists.
9. Retryable send failures preserve the submitted text and show approved retry controls. Reconnect refreshes from durable history and does not auto-resend without explicit user action.

Controls:

| Control | Proposed behavior |
|---|---|
| Message composer | Enabled only for accepted contacts with policy/bootstrap checks passing |
| Send action | Encrypt and submit one non-empty message; disabled while local encryption/submission is active |
| Failed-row `Retry` | Resubmit the preserved plaintext from local device memory after the user acts |
| Failed-row `Edit` | Restore the preserved text to the composer |
| Failed-row `Discard` | Remove only the local failed row |
| Policy shortcut | Open DM policy settings only if approved for `policy_denied` state |
| Back to contacts | Return to the Contacts Hub without changing message state |

Copy baseline:

| State | Proposed copy |
|---|---|
| Loading | `Loading private chat` |
| Empty | `No messages yet` |
| Pending contact | `Finish the contact request before starting an encrypted conversation` |
| Blocked | `Messaging is blocked for this contact` |
| Policy denied | `Your DM policy does not allow this conversation` |
| Reconnecting | `Reconnecting` |
| Send pending | `Sending` |
| Durable acceptance | `Sent` |
| Recipient-device ack | `Delivered` |
| Retryable failure | `Could not send` |
| History load failure | `Private chat could not load` |

State mapping:

| Required state | Runtime source | UI behavior |
|---|---|---|
| `loading` | Contact, thread, message, bootstrap, or local key material is loading | Show loading state and disable send |
| `empty` | Accepted contact and no decrypted messages in current thread page | Show empty state with enabled composer when policy permits |
| `blocked` | Bidirectional block policy denies messaging | Show blocked state and disable send |
| `policy_denied` | DM inbound/outbound policy denies conversation | Show policy state and optional approved settings shortcut |
| `send_failed_retryable` | Encryption, network, fanout, or acceptance failed in a retryable way | Preserve local text with `Retry`, `Edit`, and `Discard` controls |
| `reconnecting` | Realtime/catch-up channel is unavailable or refreshing | Keep durable history visible; mark live delivery/catch-up as reconnecting |

Delivery indicators:

| Indicator | Backend truth |
|---|---|
| `Sending` | Local encryption or fanout submission is in progress |
| `Sent` | API durably accepted ciphertext into DM history plus delivery metadata |
| `Queued` | Durable acceptance exists, but no recipient-device ack is known |
| `Delivered` | At least one recipient profile device acked the encrypted envelope |
| `Read` | Approved participant-visible read receipt exists |
| `Failed` | Send failed or became unrecoverable |

## Work Packages

| Package | Scope | Dependencies | Suggested validation |
|---|---|---|---|
| `DMW-01` | Non-visible API client/model helpers for DM thread lookup, message history, fanout dispatch, mark-read, and typed delivery states | Approval package accepted, or separate approval for a no-visible-UI foundation slice | Web unit tests for typed helpers and route construction; contract parity if API shape changes |
| `DMW-02` | Accepted-contact history hydration, local decrypt handoff, and loading/empty/blocked/policy state rendering | `DMW-01`, existing bootstrap/session material | Web render tests, targeted API integration tests if contact-to-thread mapping changes |
| `DMW-03` | Client-encrypted send path, local pending row, durable acceptance reconciliation, and no plaintext/private-key server boundary | `DMW-01`, approved send flow/copy/controls | Web unit/render tests, crypto/client-only checks, DM transport policy guard |
| `DMW-04` | Realtime ack, catch-up, reconnect, mark-read, and duplicate reconciliation | `DMW-02`, `DMW-03`, existing realtime helpers | Web helper tests, API/realtime integration coverage as needed |
| `DMW-05` | Retry/edit/discard failure recovery, accessibility labels, keyboard focus, and delivery indicator details | `DMW-03`, `DMW-04`, approved failed-row controls | Web accessibility/render tests plus manual browser checklist |
| `DMW-06` | Evidence pack, audit closeout, and iteration documentation update | `DMW-02` through `DMW-05` complete | Evidence under `evidence/iteration-02/dm-workspace-delivery/<YYYY-MM-DD>/` plus repo validators |

## Smallest Mergeable Slices After Approval

1. `DMW-01` if explicitly approved as a no-visible-UI foundation slice.
2. `DMW-02` history hydration and required non-send states.
3. `DMW-03` encrypted send and durable acceptance state.
4. `DMW-04` realtime/catch-up/ack reconciliation.
5. `DMW-05` retry/accessibility/delivery detail hardening.
6. `DMW-06` evidence, audit closeout, and planning status update.

Each slice must be independently mergeable and must not claim the full quality-audit finding is fixed until accepted contacts can load history and send encrypted envelopes with the approved state behavior.

## Validation Plan

For this plan-only PR:

- run the autonomous diff-scope validator with `--plan-only`;
- run docs index freshness validation against `origin/master...HEAD`;
- run no web/Rust runtime gates unless non-doc files are touched.

Before each runtime implementation PR after approval:

- run `npm --prefix apps/web run lint`;
- run `npm --prefix apps/web run test:coverage`;
- run `npm --prefix apps/web run build`;
- run `./scripts/validate-dm-transport-policy.sh`;
- run contract/docs validators when API contracts or canonical docs change;
- run targeted Rust API/realtime tests if route, fanout, catch-up, or ack behavior changes;
- capture browser screenshots/checklist evidence once visible UI behavior exists.

## Evidence Checklist

When implementation is approved and complete, the evidence pack must include:

- `summary.md` with approved UX reference, covered `DMW-*` slices, scope, and outcome;
- `validators.txt` with exact commands and manual checks;
- `provenance.json` with commit SHA, PR number or run ID, and generation timestamp;
- screenshots or browser captures for loading, empty, sent, queued/delivered, retryable failure, blocked, policy denied, and reconnecting states;
- notes for any missing artifact with explicit rationale.

## Open Approval Questions

| Area | Approval question |
|---|---|
| Scope | Is the first runtime scope accepted 1:1 contact DMs only, or should group DM workspace behavior be included? |
| Empty/loading/policy copy | Is the proposed copy baseline approved as written? |
| Failed send controls | Should failed rows expose `Retry`, `Edit`, and `Discard`, or a smaller control set? |
| Delivery indicators | Are compact HUD pips with the proposed labels approved for the DM workspace? |
| Policy shortcut | Should `policy_denied` include a direct settings shortcut? |
| Retry persistence | Should retryable failed sends survive route changes in local device storage, or only remain in memory for the active session? |

## Approval Decision Record

Record approval against each decision below before runtime UI work begins. Approval may live in a PR comment, issue comment, or project note, but the first implementation PR must cite the exact approval reference.

| Decision ID | Required approval | Approved value |
|---|---|---|
| `DMW-APP-01` | First runtime scope: accepted 1:1 only or includes group DM workspace behavior | pending |
| `DMW-APP-02` | Loading, empty, blocked, policy, reconnecting, and failure copy baseline | pending |
| `DMW-APP-03` | Failed-row controls and retry/edit/discard behavior | pending |
| `DMW-APP-04` | Delivery indicator visual direction and label set | pending |
| `DMW-APP-05` | Policy-denied settings shortcut behavior | pending |
| `DMW-APP-06` | Retryable failed-send persistence boundary | pending |

If any decision is explicitly deferred, implementation for the affected behavior must either avoid that behavior or stay plan-only for that slice.

## Approval-To-Slice Map

| Approval decision | Blocks slice | First eligible implementation PR after approval | Required approval evidence |
|---|---|---|---|
| `DMW-APP-01` | `DMW-02` through `DMW-06` | History hydration and workspace state rendering | Approved 1:1/group scope |
| `DMW-APP-02` | `DMW-02`, `DMW-05` | Required state rendering | Approved copy values |
| `DMW-APP-03` | `DMW-03`, `DMW-05` | Failed-row recovery controls | Approved failed-send control model |
| `DMW-APP-04` | `DMW-03`, `DMW-04`, `DMW-05` | Delivery indicator rendering and accessibility labels | Approved indicator labels/visual treatment |
| `DMW-APP-05` | `DMW-02` | Policy denied state behavior | Approved settings shortcut behavior or explicit exclusion |
| `DMW-APP-06` | `DMW-03`, `DMW-05` | Retryable failed-send persistence | Approved in-memory vs device-local persistence boundary |

`DMW-01` may be approved separately as a no-visible-UI foundation slice. If only `DMW-01` is approved, runtime work must stay limited to typed helpers, route construction, and tests that do not encode visible UI behavior.

## Approval Response Template

Use this template in the approving PR comment, issue comment, or project note. The first runtime implementation PR must link to the exact approval reference and copy the approved decision values into its PR body.

```text
DM workspace delivery approval reference:
- Scope approved: DMW-01 only | DMW-02 | DMW-03 | DMW-04 | DMW-05 | DMW-06 evidence closeout | full accepted-contact DM workspace sequence
- DMW-APP-01 First runtime scope: accepted 1:1 only | include group DM workspace behavior | deferred
- DMW-APP-02 State copy baseline: approved as written | approved with changes: <changes> | deferred
- DMW-APP-03 Failed-row controls: Retry/Edit/Discard | Retry/Discard only | other: <controls> | deferred
- DMW-APP-04 Delivery indicators: compact HUD pips with listed labels | approved with changes: <changes> | deferred
- DMW-APP-05 Policy-denied settings shortcut: include shortcut | no shortcut | deferred
- DMW-APP-06 Retry persistence: active-session memory only | device-local persistence | deferred
- Additional constraints: <any limits, exclusions, or required validation>
```

Approval can cover one slice at a time. If approval covers only `DMW-01`, runtime work remains limited to non-visible helpers and tests.

## Pre-Implementation Gate Checklist

Before any runtime DM workspace branch starts, verify all of the following:

| Gate | Required evidence |
|---|---|
| Approval reference exists | PR/issue/project note link with non-pending `DMW-APP-*` values for the slice |
| Scope is slice-bounded | PR body names the exact `DMW-*` slice and related product/audit target |
| UX copy/control deltas are frozen | Any changed copy, controls, indicator labels, shortcut behavior, or persistence behavior appears in the approval reference |
| Contract impact is explicit | PR body states whether existing DM contracts are sufficient or names the exact contract updates |
| Validation commands are selected | PR body lists web, Rust, contract, DM transport, and browser/evidence checks that apply |
| Evidence path is reserved when UI exists | `evidence/iteration-02/dm-workspace-delivery/<YYYY-MM-DD>/` path and artifact list are named before screenshot capture |
| No unapproved adjacent behavior is bundled | PR body explicitly excludes deferred `DMW-APP-*` decisions and unrelated navigation/server-channel UX changes |

Missing approval evidence is a hard stop. Do not use this plan, a quality-audit finding, or an automated PR merge as implied approval for product UI behavior.

## Runtime Implementation Hard Stops

Stop and keep the work plan-only when any of these conditions apply:

- the slice has no approval reference with non-pending `DMW-APP-*` values;
- the approval reference changes copy, controls, or behavior but the PR body does not quote the approved value set;
- the implementation attempts to include a later `DMW-*` slice whose approval is missing or deferred;
- the slice needs new backend/API behavior that is not scoped in the implementation PR;
- the implementation would expose plaintext/private keys to server-side code;
- the implementation would add recipient-device reachability, LAN/WAN discovery, endpoint-card, preflight, WAN wizard, or parallel-dial behavior;
- validation evidence would require claiming the quality-audit finding is fixed before accepted contacts can both load history and send encrypted envelopes.

## Known Limits

- This plan does not introduce backend API contracts.
- This plan does not approve any runtime UI copy, control, or behavior.
- Existing runtime APIs may still lack a deterministic contact-to-thread mapping for the current contact route; that must be proven or added in a future approved implementation slice.
- Client-side decrypt/render behavior depends on local key/session availability and must preserve client-only plaintext/private-key boundaries.
- Manual browser evidence remains required until deterministic render coverage exists for every required DM workspace state.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/product/08-screen-state-spec.md`
- `docs/contracts/runtime-rest.openapi.yaml`
- `docs/architecture/04-communication-networking-layer-plan.md`
- `docs/planning/iterations/02-sprint-board.md`
- `docs/operations/quality-audits/17-ux-product-quality.md`
