---
id: ADR-0010
title: CI is a reusable workflow called by every repository; releases run on release-plz
type: adr
status: superseded
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: ADR-0021
scope: [platform, service, library, tool, infra]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [ci, cd, github-actions, release, versioning]
breaking: true
enforced_by:
  - check: ci.calls_shared_workflow
    severity: error
    rationale: A repo with a hand-written pipeline diverges the moment the shared one is fixed.
  - check: ci.no_actions_rs_actions
    severity: error
    rationale: The actions-rs family is unmaintained and pins an ancient toolchain.
  - check: ci.actions_pinned_to_sha
    severity: error
    rationale: A mutable tag on a third-party action is remote code execution with extra steps.
  - check: ci.permissions_declared
    severity: error
    args: { default: "contents: read" }
  - check: release.release_plz_configured
    severity: error
    args: { applies_to: ["lib-*", "tool-*"] }
---

# ADR-0010 — CI is a reusable workflow; releases run on release-plz

## Status

**LOCKED** as of 2026-08-06.

## Context

Polyrepo (ADR-0001) means N pipelines. Copying a workflow into every repository
guarantees divergence: a fix lands in one, the other thirty-nine keep the bug, and
nobody knows which is authoritative. The knowledge-vendoring argument from ADR-0001
does not apply here — a workflow is executable, and ADR-0007 already established that
executable artifacts are versioned centrally rather than copied.

Separately, the toolchain choice matters more than it looks. The `actions-rs/*` family
is unmaintained, and repositories using it silently pin an old toolchain and miss
security advisories.

## Decision

### CI: one reusable workflow, called by every repository

`infra-ci` hosts `.github/workflows/rust-ci.yml` as a **reusable workflow**. Every Rust
repository's own workflow is a caller, roughly ten lines:

```yaml
jobs:
  ci:
    uses: Paridyum/infra-ci/.github/workflows/rust-ci.yml@v1
    with:
      integration: true
      coverage-patch: 80
```

Fixes and new checks land once. Repositories pin a major tag (`@v1`) so a breaking
change to the pipeline is opted into, not pushed.

### The job graph

```
        ┌── fmt ────────┐
        ├── clippy ─────┤
push/PR ├── test ───────┼── ✅ required
        ├── docs ───────┤
        ├── deny ───────┤
        └── coverage ───┘

main only:  build-image → push to GHCR → deploy dev
weekly:     cargo-mutants · cargo-audit · dependency review
```

Fast, always-run jobs are parallel and independent, so a formatting failure does not
hide a test failure — every job reports, and the developer sees the complete picture
in one run rather than fixing one thing at a time across four pushes.

Mutation testing and audits are **scheduled, not per-PR**. `cargo-mutants` takes
tens of minutes; running it per-PR would make the pipeline the bottleneck and teach
people to bypass it.

### Fixed action choices

| Purpose | Action | Rejected |
| ------- | ------ | -------- |
| Toolchain | `dtolnay/rust-toolchain` | `actions-rs/toolchain` — unmaintained |
| Cache | `Swatinem/rust-cache` | hand-rolled `actions/cache` — wrong keys, poisoned caches |
| Tool install | `taiki-e/install-action` | `cargo install` in CI — minutes per run, uncached |

**Every third-party action is pinned to a commit SHA**, not a tag. A tag is mutable;
depending on one means an upstream account compromise executes arbitrary code with
your `GITHUB_TOKEN`. Renovate keeps the pins current.

**Every workflow declares `permissions:`**, defaulting to `contents: read`. The
default token is broadly scoped, and a job that only runs tests should not be able to
push.

### Releases: `release-plz`

`release-plz` opens a **Release PR** that bumps versions and updates `CHANGELOG.md`
from Conventional Commits (C-0003), then publishes and tags on merge.

Chosen over `cargo-release` and hand-rolled tagging because it derives the next
version by **comparing local packages against the registry** rather than parsing git
tags. In a workspace where crates version independently, tag-derived versioning breaks
as soon as one crate is skipped in a release.

The Release PR model matters more than the tool: the version bump and changelog are
reviewable before they are permanent, and a release is a merge rather than a
privileged local command someone runs from their laptop.

`git-cliff` generates the changelog; `release-plz` invokes it. Keep a Changelog format.

### Deployment

CI builds and pushes an image to GHCR on merge to `main`, then deploys to `dev`
automatically. `staging` and `prod` require manual approval through a GitHub
Environment — the one gate that works on the Free plan, because environment protection
rules are not a branch-protection feature.

## Consequences

**Accepted costs**

- `infra-ci` becomes load-bearing: a bug there breaks every repository at once. It
  needs its own tests and a `@v1` / `@v2` discipline.
- Reusable workflows cannot be tested in isolation as easily as a local file; changes
  are validated in a canary repository before the tag moves.
- Pinning actions to SHAs makes upgrades noisier. Renovate handles it.
- Actions minutes are capped on GitHub Free (ADR-0004), which is why mutation testing
  and audits are weekly rather than per-PR.

**Gained**

- One pipeline definition. A new check reaches every repository by moving a tag.
- Toolchain, cache strategy, and tool versions are identical everywhere, so "works in
  CI for that repo but not this one" stops being a category of problem.
- Supply-chain surface is pinned and reviewable.

**Known gap**

On GitHub Free these checks **cannot be required** on private repositories (ADR-0004),
so a red pipeline does not block a merge. CI is a signal here; the blocking layers are
the agent hooks and pre-commit (ADR-0007). This is the same gap ADR-0004 records, and
the same upgrade closes it.
