## What changed

<!-- One paragraph. The reader has not seen the issue. -->

## Why

<!-- The problem, not the solution. -->

## Lifecycle (ADR-0013)

- [ ] Not a significant change — `check-significance.sh` reports clean, **or**
- [ ] Implements RFC-____ , which was reviewed and landed before this code

## Organization (STD-0004)

- [ ] Every new module opens with a one-sentence `//!` responsibility
- [ ] No module named `utils` / `helpers` / `common` / `misc` / `shared`
- [ ] Nothing was split to satisfy a size rule — there is no size rule

## Knowledge impact

- [ ] No locked artifact is contradicted
- [ ] A new binding decision here is recorded as an ADR, or explicitly needs none
- [ ] `manifest.yaml` still accurate

## Verification

- [ ] `just test` passes locally
- [ ] Patch coverage >= 80% (STD-0001)
- [ ] Docs updated: README / ARCHITECTURE / API / RUNBOOK as applicable (STD-0002)
- [ ] Doc examples compile (`cargo test --doc`)

## Changelog (STD-0005)

- [ ] Notable to a reader, and an entry was added under one of the six categories
- [ ] **Not** notable — no entry, deliberately

<!-- There is no per-PR changelog gate. Do not add a line just to pass a check. -->

## Breaking changes

<!-- API, schema, config, or event-shape. Write "none" if none. -->

## Rollback

<!-- How to undo this in production. "Revert the commit" only if that is genuinely
     sufficient — a migration usually is not. -->
