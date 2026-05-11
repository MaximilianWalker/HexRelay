# HexRelay Release Packaging

## Document Metadata

- Doc ID: release-packaging
- Owner: Platform maintainers
- Status: needs-detail
- Scope: repository
- last_updated: 2026-05-11
- Source of truth: `docs/operations/03-release-packaging.md`

## Quick Context

- Purpose: define HexRelay release artifact families, desktop/server package boundaries, and signing expectations.
- Primary edit location: update this file when supported release targets, installer formats, dedicated-server packaging, or code signing policy changes.
- Latest meaningful change: 2026-05-11 added the initial Windows/Linux first-class artifact model, Tauri desktop default, dedicated-server package boundary, app-mediated server administration stance, and code signing stance.

## Status and Scope

- Current status is `needs-detail` because release automation, signing infrastructure, and install smoke tests are not implemented yet.
- Windows and Linux are mandatory first-class targets for release planning; do not plan Windows-only delivery or defer Linux as a post-MVP afterthought.
- This document covers MVP-end packaging intent and release-readiness expectations.
- This document does not claim that release artifacts, signing certificates, package repositories, or auto-update infrastructure already exist.

## Distribution Defaults

- Desktop distribution is the primary end-user path.
- Tauri is the default desktop shell unless a later explicit architecture decision replaces it.
- Desktop packages install the UI and local personal runtime/sidecars needed for local-first operation.
- Dedicated server distribution is a separate service/package family, not a separate desktop app.
- The same Rust service code should be reused where practical across desktop local runtime and dedicated server modes.

## Package Boundary

- The desktop installer remains client-focused and runs in user space by default.
- The desktop installer must not silently install or enable a public/network-facing dedicated server service.
- Dedicated server artifacts are installed, configured, logged, backed up, upgraded, and secured as operator-managed services.
- The normal HexRelay app is the intended administration surface for dedicated servers when the connected identity has node-owner/admin permissions.
- Dedicated server artifacts may expose authenticated admin/operator APIs, but they should remain headless and should not bundle a separate server-specific frontend by default.
- App-mediated dedicated-server management should connect to an operator-installed dedicated server endpoint rather than bundling the server service into every desktop install.

## Why The Server Is Separate

- Dedicated server operation has a different security posture than a normal desktop client.
- It requires service lifecycle management, persistence paths, firewall/port exposure, ingress/TLS, secrets, backups, logs, and upgrade/rollback procedures.
- Bundling it into the default desktop installer would increase the attack surface for normal users.
- Desktop users should not accidentally install a long-running public service.

## Artifact Matrix

| Artifact family | Windows target | Linux target | Notes |
|---|---|---|---|
| Desktop installer | `.msi` or `.exe` | `.AppImage` and `.deb` | Windows and Linux are both required first-class targets. |
| Desktop portable | `.zip` | `.tar.gz` | Portable packages support manual install and smoke testing. |
| Dedicated server native | binary/package | binary `.tar.gz` and `.deb` with `systemd` unit | Server artifacts are separate from desktop artifacts. |
| Dedicated server container | container image where useful | container image plus Compose example | Container delivery supports operators and runtime test stacks. |
| Checksums/signatures | signed checksums or equivalent | signed checksums or repository metadata | Required before public stable release. |

`.rpm` may be added if Fedora/RHEL-family support becomes worth maintaining before or after MVP.

## Desktop Packaging Expectations

- Build the reusable web UI once and embed or serve it through the Tauri shell according to the chosen desktop runtime design.
- Package local runtime sidecars only for user-local desktop operation.
- Keep default desktop networking scoped to loopback/local trust boundaries unless the user explicitly configures otherwise.
- Provide install and launch smoke coverage on both Windows and Linux before release candidates are considered ready.

## Dedicated Server Packaging Expectations

- Package `api-rs` and `realtime-rs` as headless server-mode services.
- Windows dedicated server delivery may be a service-capable package or console binary package.
- Linux dedicated server delivery should include native binaries, a `.deb` package with a `systemd` unit, and a container image with a Compose example.
- Do not add a separate dedicated-server UI artifact by default. Admin/operator screens should ship through the normal app surface and consume authenticated server APIs.
- Operator-managed dependencies, ingress, secrets, persistence, backups, and rollback remain governed by `docs/operations/02-dedicated-server-deployment.md`.

## Code Signing

Code signing is a cryptographic signature attached to an executable, installer, package, update artifact, or release metadata.

It proves:

- Publisher identity: the artifact came from the claimed project owner.
- Integrity: the artifact was not modified after signing.

It does not prove that the code is secure, bug-free, or audited. It proves origin and tamper resistance.

## Signing Expectations

- Windows uses Authenticode-style signing for executables and installers.
- Windows signing reduces SmartScreen and unknown-publisher warnings and is required for credible public installers and auto-update.
- Linux signing usually covers package repositories, `.deb` metadata, checksums, detached signatures, or release artifacts.
- Tauri updater support requires signed update artifacts or metadata, so auto-update should wait until signing is properly solved.
- Internal alpha builds may be unsigned if they are clearly labeled.
- Public beta/stable releases should sign Windows artifacts and Linux packages/checksums.

## Release Validation Checklist

- Windows desktop installer builds and launches.
- Windows desktop portable package extracts and launches.
- Linux desktop package builds and launches.
- Linux desktop portable package extracts and launches.
- Dedicated server native package starts `api-rs` and `realtime-rs` with documented config.
- Dedicated server container stack starts and passes health checks.
- Checksums are generated for every artifact.
- Public beta/stable artifacts are signed or explicitly blocked until signing is available.

## Related Documents

- `docs/architecture/adr-0002-runtime-deployment-modes.md`
- `docs/product/01-mvp-plan.md`
- `docs/operations/02-dedicated-server-deployment.md`
- `docs/operations/01-mvp-runbook.md`
- `docs/reference/runtime-config-reference.md`
