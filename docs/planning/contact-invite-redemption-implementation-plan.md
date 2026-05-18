# HexRelay Contact Invite Redemption Implementation Plan

## Document Metadata

- Doc ID: contact-invite-redemption-implementation-plan
- Owner: Web, API, product, and delivery maintainers
- Status: approval_pending
- Scope: repository
- last_updated: 2026-05-18
- Source of truth: `docs/planning/contact-invite-redemption-implementation-plan.md`

## Quick Context

- Purpose: sequence the contact-invite redemption gap without changing product UX, API behavior, or visible copy before explicit approval.
- Primary edit location: update this file when contact-invite redemption flow, copy, controls, implementation slicing, approval evidence, or validation expectations change.
- Latest meaningful change: 2026-05-18 created the approval-pending plan for `QA-17-20260514-contact-invite-preview-skipped`.

## Approval Boundary

This document is a plan-only artifact. It does not approve product UI, API contract, or runtime behavior changes.

Runtime work that changes the contact-invite redemption flow, copy, controls, API side effects, or visible behavior must wait until the user explicitly approves the proposed package below. Until approval exists, allowed work is limited to planning, test/evidence design, documentation freshness, and audit-ledger routing.

## Source Authorities

| Authority | Role |
|---|---|
| `docs/product/01-mvp-plan.md` | Locked contact-invite MVP scope and UX approval gate |
| `docs/product/02-prd.md` | Product requirement for inviter preview before recipient acceptance |
| `docs/contracts/runtime-rest.openapi.yaml` | Current runtime REST contact-invite create/redeem contract |
| `apps/web/app/contacts/page.tsx` | Current Contacts Hub redemption UI behavior |
| `apps/web/lib/api.ts` | Current web API client for contact-invite creation and redemption |
| `services/api-rs/src/tests/integration/invites_tests.rs` | Current backend evidence that redeem creates or returns a pending friend request |
| `docs/operations/quality-audits/17-ux-product-quality.md` | Original quality finding and supersession record |

## Selected Finding

| Source | Finding | Target |
|---|---|---|
| `QA-17-20260514-contact-invite-preview-skipped` | Contact invite redemption skips the documented inviter preview and explicit recipient acceptance step. | After explicit approval, runtime behavior either matches the documented preview-then-accept flow or the canonical product docs intentionally approve and describe the current one-step redeem behavior. |

## Current Evidence

- `docs/product/02-prd.md` currently requires the recipient to redeem the token, see an inviter preview, and accept before a server-mediated friend request or accepted friend edge is created.
- `apps/web/app/contacts/page.tsx` currently submits the pasted token directly through `redeemContactInvite`, labels the first action `Continue`, and renders a `Request sent` result after the POST succeeds.
- `docs/contracts/runtime-rest.openapi.yaml` currently defines `POST /contact-invites/redeem` as "Redeem a contact invite token and create friend request."
- `services/api-rs/src/tests/integration/invites_tests.rs` currently asserts that successful contact-invite redemption returns a pending `FriendRequestRecord`.
- No approved contact-invite preview/acceptance flow package currently exists in the repo.

## Current Plan-Only Split Rationale

The selected finding is valid but cannot be fixed by changing Web or API behavior without explicit UX/product approval. The smallest mergeable remediation is therefore this plan-only approval package:

- preserve the current runtime evidence without rewriting product intent by implication;
- define the exact decisions needed before Web/API changes begin;
- make the two viable product directions explicit;
- route future automation away from unapproved runtime UX implementation.

This plan must not claim that contact-invite redemption runtime acceptance criteria are complete. It only records the proposed flow, copy, controls, implementation slices, validation, and approval evidence required before a runtime branch begins.

## Plan-Only Change Scope

Allowed plan-only changes:

- clarify proposed contact-invite redemption flow options, controls, copy, state mapping, and approval questions;
- map pending approval decisions to the first implementation slice that needs them;
- define validation commands, evidence artifacts, and PR-body requirements;
- update docs indexes, iteration logs, and quality-audit routing.

Disallowed without explicit UX/product approval:

- editing `apps/web` runtime UI, route behavior, visible fixtures, browser tests, or styles for contact-invite redemption;
- changing API/realtime contracts solely to support unapproved product behavior;
- changing `POST /contact-invites/redeem` side effects or adding a preview endpoint without approved flow semantics;
- updating the PRD to approve a one-step redeem flow without an explicit user decision;
- adding evidence that claims contact-invite preview/acceptance runtime behavior is complete.

Plan-only PRs may merge before approval because they do not change product behavior. The first runtime implementation PR must cite the exact approval reference and approved `CIR-APP-*` values it implements.

## Proposed Flow Package

Status: approval_pending. The following package must be approved before runtime UI or API work begins.

### Option A: Preview Then Accept

This option keeps the current PRD intent.

Flow:

1. Recipient opens Contacts Hub and pastes a contact invite link or token.
2. Client validates the token shape locally before network submission.
3. Runtime preview checks token validity, expiration, exhaustion, self-redeem, and block state without creating a friend request.
4. Recipient sees an inviter preview containing only the approved identity/profile fields.
5. Recipient chooses `Accept invite` or `Cancel`.
6. `Accept invite` creates a server-mediated pending friend request or accepted friend edge according to approved product settings.
7. Invalid, expired, exhausted, blocked, and self-redeem states use deterministic error feedback.

Controls:

| Control | Proposed behavior |
|---|---|
| Token input | Accept `hexrelay://contact-invite/<token>` or raw token input |
| Preview action | Check token and show inviter preview without creating relationship state |
| Accept invite | Create the approved relationship state after recipient review |
| Cancel | Close the preview without consuming the token |
| Close | Dismiss the panel without changing relationship state |

Copy baseline:

| State | Proposed copy |
|---|---|
| Token entry action | `Preview invite` |
| Preview loading | `Checking invite` |
| Preview title | `Contact invite` |
| Accept action | `Accept invite` |
| Cancel action | `Cancel` |
| Accepted pending request | `Contact request sent` |
| Invalid token | `Invite link is invalid` |
| Expired token | `Invite link has expired` |
| Exhausted token | `Invite link has already been used` |
| Blocked | `This invite cannot be used because one of you has blocked the other` |
| Self redeem | `You cannot use your own contact invite` |

### Option B: Approved One-Step Redeem

This option intentionally changes canonical product behavior to match the current runtime shape.

Flow:

1. Recipient pastes a contact invite link or token.
2. Client submits the token directly.
3. Successful redemption immediately creates or returns a pending friend request.
4. The canonical PRD and screen-state copy are updated to remove the required preview/acceptance split.
5. Error feedback remains deterministic for invalid, expired, exhausted, blocked, and self-redeem states.

This option requires explicit product approval because it removes the documented recipient preview and acceptance step.

## Work Packages

| Package | Scope | Dependencies | Suggested validation |
|---|---|---|---|
| `CIR-01` | Approval-backed API contract decision: preview endpoint plus accept endpoint, or documented one-step redeem semantics | Approved `CIR-APP-01` and `CIR-APP-02` | Contract parity plus targeted API integration tests |
| `CIR-02` | Web API client helpers and typed result states for the approved flow | `CIR-01` | `apps/web` API unit tests |
| `CIR-03` | Contacts Hub token entry, preview/accept or one-step state rendering, and deterministic error surfaces | Approved `CIR-APP-03` through `CIR-APP-05` | Web render tests, lint, coverage, build |
| `CIR-04` | Evidence pack, audit closeout, PRD/screen-state updates if required, and iteration documentation update | Runtime slices complete | Evidence under `evidence/iteration-02/contact-invite-redemption/<YYYY-MM-DD>/` plus repo validators |

## Smallest Mergeable Slices After Approval

1. `CIR-01` API/contract behavior for the approved product direction.
2. `CIR-02` no-visible Web client helpers and typed state handling.
3. `CIR-03` Contacts Hub runtime UI behavior for the approved flow.
4. `CIR-04` evidence, audit closeout, and canonical docs update.

Each slice must be independently mergeable and must not claim the quality-audit finding is fixed until runtime behavior, canonical docs, tests, and evidence all match the approved direction.

## Validation Plan

For this plan-only PR:

- run the focused temporary harness proving the approval package and ledger supersession exist;
- run the autonomous diff-scope validator against `origin/master...HEAD`;
- run docs index freshness validation against `origin/master...HEAD`;
- run no web/Rust runtime gates unless non-doc files are touched.

Before each runtime implementation PR after approval:

- run `npm --prefix apps/web run lint`;
- run `npm --prefix apps/web run test:coverage`;
- run `npm --prefix apps/web run build`;
- run contract/docs validators when API contracts or canonical docs change;
- run targeted Rust API tests if invite-preview, accept, redeem, friend-request, or block behavior changes;
- capture browser screenshots/checklist evidence once visible UI behavior exists.

## Evidence Checklist

When implementation is approved and complete, the evidence pack must include:

- `summary.md` with approved UX/product reference, covered `CIR-*` slices, scope, and outcome;
- `validators.txt` with exact commands and manual checks;
- `provenance.json` with commit SHA, PR number or run ID, and generation timestamp;
- screenshots or browser captures for token entry, preview, accepted/pending request, invalid, expired, exhausted, blocked, and self-redeem states for Option A, or the approved one-step equivalent for Option B;
- notes for any missing artifact with explicit rationale.

## Approval Decision Record

Record approval against each decision below before runtime work begins. Approval may live in a PR comment, issue comment, or project note, but the first implementation PR must cite the exact approval reference.

| Decision ID | Required approval | Approved value |
|---|---|---|
| `CIR-APP-01` | Product direction: preview-then-accept or one-step redeem | pending |
| `CIR-APP-02` | API behavior: non-mutating preview plus accept/redeem mutation, or direct one-step mutation | pending |
| `CIR-APP-03` | Token-entry, loading, preview, accepted, and error copy baseline | pending |
| `CIR-APP-04` | Controls shown in the Contacts Hub redemption panel | pending |
| `CIR-APP-05` | Inviter preview fields allowed before relationship acceptance | pending |
| `CIR-APP-06` | Evidence scope and browser-state checklist required for closeout | pending |

If any decision is explicitly deferred, implementation for the affected behavior must either avoid that behavior or stay plan-only for that slice.

## Approval-To-Slice Map

| Approval decision | Blocks slice | First eligible implementation PR after approval | Required approval evidence |
|---|---|---|---|
| `CIR-APP-01` | `CIR-01` through `CIR-04` | API/product direction implementation | Approved preview-vs-one-step decision |
| `CIR-APP-02` | `CIR-01`, `CIR-02` | REST contract and client helper work | Approved API side-effect semantics |
| `CIR-APP-03` | `CIR-03` | Contacts Hub state rendering | Approved copy values |
| `CIR-APP-04` | `CIR-03` | Contacts Hub controls | Approved control set |
| `CIR-APP-05` | `CIR-01`, `CIR-03` | Preview payload and preview rendering | Approved pre-acceptance identity/profile fields |
| `CIR-APP-06` | `CIR-04` | Evidence and audit closeout | Approved evidence checklist |

## Approval Response Template

Use this template in the approving PR comment, issue comment, or project note. The first runtime implementation PR must link to the exact approval reference and copy the approved decision values into its PR body.

```text
Contact invite redemption approval reference:
- Scope approved: CIR-01 only | CIR-02 | CIR-03 | CIR-04 evidence closeout | full contact-invite redemption sequence
- CIR-APP-01 Product direction: preview-then-accept | one-step redeem | deferred
- CIR-APP-02 API behavior: non-mutating preview plus accept/redeem mutation | direct one-step mutation | deferred
- CIR-APP-03 Copy baseline: approved as written | approved with changes: <changes> | deferred
- CIR-APP-04 Contacts Hub controls: approved as written | approved with changes: <changes> | deferred
- CIR-APP-05 Preview fields: identity id only | identity id plus display name/avatar when available | other: <fields> | deferred
- CIR-APP-06 Evidence scope: approved as written | approved with changes: <changes> | deferred
- Additional constraints: <any limits, exclusions, or required validation>
```

Approval can cover one slice at a time. If approval covers only `CIR-01`, runtime work remains limited to API/contract behavior and tests with no visible Contacts Hub behavior.

## Pre-Implementation Gate Checklist

Before any runtime contact-invite redemption branch starts, verify all of the following:

| Gate | Required evidence |
|---|---|
| Approval reference exists | PR/issue/project note link with non-pending `CIR-APP-*` values for the slice |
| Scope is slice-bounded | PR body names the exact `CIR-*` slice and related quality-audit target |
| UX copy/control deltas are frozen | Any changed copy, controls, preview fields, or relationship side effects appear in the approval reference |
| Contract impact is explicit | PR body states whether existing contact-invite contracts are sufficient or names the exact contract updates |
| Validation commands are selected | PR body lists web, Rust, contract, docs, and browser/evidence checks that apply |
| Evidence path is reserved when UI exists | `evidence/iteration-02/contact-invite-redemption/<YYYY-MM-DD>/` path and artifact list are named before screenshot capture |
| No unapproved adjacent behavior is bundled | PR body explicitly excludes deferred `CIR-APP-*` decisions and unrelated Contacts Hub or DM workspace UX changes |

Missing approval evidence is a hard stop. Do not use this plan, a quality-audit finding, or an automated PR merge as implied approval for product UI or API behavior.

## Runtime Implementation Hard Stops

Stop and keep the work plan-only when any of these conditions apply:

- the slice has no approval reference with non-pending `CIR-APP-*` values;
- the approval reference changes copy, controls, API side effects, or preview fields but the PR body does not quote the approved value set;
- the implementation attempts to include a later `CIR-*` slice whose approval is missing or deferred;
- the implementation updates the PRD to one-step redeem semantics without explicit approval of `CIR-APP-01`;
- the implementation exposes raw key/profile-identifying data before the approved relationship gate;
- validation evidence would require claiming the quality-audit finding is fixed before runtime behavior and canonical docs match the approved direction.

## Known Limits

- This plan does not introduce backend API contracts.
- This plan does not approve any runtime UI copy, control, API side effect, or behavior.
- Current runtime behavior still creates or returns a pending friend request from `POST /contact-invites/redeem`.
- Current Contacts Hub behavior still sends the token directly and shows `Request sent`.
- The PRD still requires inviter preview and recipient acceptance until an approved product decision changes it.

## Related Documents

- `docs/product/01-mvp-plan.md`
- `docs/product/02-prd.md`
- `docs/contracts/runtime-rest.openapi.yaml`
- `apps/web/app/contacts/page.tsx`
- `apps/web/lib/api.ts`
- `services/api-rs/src/tests/integration/invites_tests.rs`
- `docs/operations/quality-audits/17-ux-product-quality.md`
