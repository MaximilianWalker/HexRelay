## Change Type

- [ ] move-only
- [ ] content-change

## Scope

- What changed:
- Why it changed:

## Canonical Documents Updated

- [ ] `docs/README.md` (if routing or ownership changed)
- [ ] `docs/product/01-mvp-plan.md` (if product intent/constraints changed)
- [ ] `docs/product/02-prd-v1.md` (if requirements changed)
- [ ] `docs/planning/iterations/*.md` (if execution plan changed)
- [ ] `docs/reference/glossary.md` (if terms changed)

## Validation

- [ ] Links and paths are valid.
- [ ] Metadata block is present and updated in changed canonical docs.
- [ ] No lock-in/paywalled core assumption was introduced.
- [ ] If this PR changes `services/api-rs/migrations/*.sql`, I updated `evidence/migrations/<migration>.md` using `docs/operations/migration-validation-template.md`.
- [ ] I ran `npm run security` locally (fast Rust-audit gate), and for CI-level parity I also ran the extra checks required by `docs/operations/contributor-guide.md` when applicable.
- [ ] For deployment-impacting changes, I named release decision owner + backup and linked evidence artifacts.
