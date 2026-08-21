---
id: ADR-0021
title: CI is a reusable workflow per stack, called by every class declared ci
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: [ADR-0010]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [ci, cd, release, workflows, registry-driven]
breaking: false
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

# ADR-0021 — CI is a reusable workflow per stack, called by every class declared ci

## Status

**LOCKED** as of 2026-08-07. Supersedes ADR-0010.

**The decision is unchanged.** One reusable workflow per stack, called rather than copied;
pinned SHAs; declared permissions; `release-plz` for releases. Two things change:

1. `applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]` becomes **every class whose
   `class-registry.yaml` entry declares `ci: true`** — provably the same four classes today.
2. The workflow is **per stack**, not one file. `rust-ci.yml` was the only one because Rust
   was the only stack; a `web` class needs its own, and saying "the reusable workflow"
   without qualification would have been read as one file for every language.

## Context

Polyrepo (ADR-0001) means N pipelines. Copying a workflow into every repository guarantees
divergence: a fix lands in one, the other thirty-nine keep the bug, and nobody knows which
is authoritative. The knowledge-vendoring argument from ADR-0001 does not apply here — a
workflow is executable, and ADR-0007 already established that executable artifacts are
versioned centrally rather than copied.

Separately, the toolchain choice matters more than it looks. The `actions-rs/*` family is
unmaintained, and repositories using it silently pin an old toolchain and miss security
advisories.

## Decision

### CI: one reusable workflow per stack, called by every repository

`infra-ci` hosts a reusable workflow per stack — `.github/workflows/rust-ci.yml` today,
`nextjs-ci.yml` when the `web` class has one. Every repository's own workflow is a caller,
roughly ten lines:

```yaml
jobs:
  ci:
    uses: Paridyum/infra-ci/.github/workflows/rust-ci.yml@v1
    with:
      integration: true
      coverage-patch: 80
```

Which workflow a repository calls is determined by its **stack**, and whether it calls one
at all by its **`ci` flag** — both from `class-registry.yaml`.

Fixes and new checks land once. Repositories pin a major tag (`@v1`) so a breaking change to
the pipeline is opted into, not pushed. That `@v1` is deliberately *not* a commit SHA, and
is the documented exemption to `ci.actions_pinned_to_sha`: a first-party reusable workflow
is pinned by ref precisely so a fix propagates.

### The job graph

```
        ┌── fmt ────────┐
        ├── lint ───────┤
push/PR ├── test ───────┼── ✅ required
        ├── docs ───────┤
        ├── deps ───────┤
        ├── secrets ────┤
        └── coverage ───┘

main only:  build-image → push to GHCR → deploy dev
weekly:     mutation testing · advisories · dependency review
```

Fast, always-run jobs are parallel and independent, so a formatting failure does not hide a
test failure — every job reports, and the developer sees the complete picture in one run
rather than fixing one thing at a time across four pushes.

Mutation testing and audits are **scheduled, not per-PR**. They take tens of minutes;
running them per-PR would make the pipeline the bottleneck and teach people to bypass it.

The job *names* are stack-neutral — `lint` rather than `clippy`, `deps` rather than `deny` —
because the graph is the decision and the tool that fills each box belongs to the stack.

### Fixed action choices, `rust` stack

| Purpose | Action | Rejected |
| ------- | ------ | -------- |
| Toolchain | `dtolnay/rust-toolchain` | `actions-rs/toolchain` — unmaintained |
| Cache | `Swatinem/rust-cache` | hand-rolled `actions/cache` — wrong keys, poisoned caches |
| Tool install | `taiki-e/install-action` | `cargo install` in CI — minutes per run, uncached |

Equivalent choices for a new stack are fixed by that stack's ADR, not here.

**Every third-party action is pinned to a commit SHA**, not a tag. A tag is mutable;
depending on one means an upstream account compromise executes arbitrary code with your
`GITHUB_TOKEN`. Renovate keeps the pins current.

**Every workflow declares `permissions:`**, defaulting to `contents: read`. The default
token is broadly scoped, and a job that only runs tests should not be able to push.

### Releases: `release-plz` for the `rust` stack

`release-plz` opens a **Release PR** that bumps versions and updates `CHANGELOG.md` from
Conventional Commits (C-0003), then publishes and tags on merge.

Chosen over `cargo-release` and hand-rolled tagging because it derives the next version by
**comparing local packages against the registry** rather than parsing git tags. In a
workspace where crates version independently, tag-derived versioning breaks as soon as one
crate is skipped in a release.

The Release PR model matters more than the tool: the version bump and changelog are
reviewable before they are permanent, and a release is a merge rather than a privileged
local command someone runs from their laptop. **That model is the decision; the tool is per
stack**, and a non-Rust stack picks its own equivalent while keeping the Release PR shape.

`git-cliff` generates the changelog; `release-plz` invokes it. Keep a Changelog format.

### Deployment

CI builds and pushes an image to GHCR on merge to `main`, then deploys to `dev`
automatically. `staging` and `prod` require manual approval through a GitHub Environment —
the one gate that works on the Free plan, because environment protection rules are not a
branch-protection feature.

## Consequences

**Accepted costs**

- `infra-ci` becomes load-bearing: a bug there breaks every repository at once. It needs its
  own tests and a `@v1` / `@v2` discipline. **It does not exist yet**, which currently blocks
  all CI for every class.
- Reusable workflows cannot be tested in isolation as easily as a local file; changes are
  validated in a canary repository before the tag moves.
- Pinning actions to SHAs makes upgrades noisier. Renovate handles it.
- Actions minutes are capped on GitHub Free (ADR-0004), which is why mutation testing and
  audits are weekly rather than per-PR.
- **One workflow per stack means N workflows to maintain**, and the job graph must stay
  aligned across them or "CI passed" means something different per repository. The
  stack-neutral job names exist to make divergence visible.
- **`ci` is now a registry boolean.** Marking a class `ci: true` without writing its
  workflow produces a repository whose caller references a file that does not exist.
  Nothing checks that today.

**Gained**

- Fixes land once, for every repository on that stack.
- **A new class with an existing stack costs a registry entry and nothing else** — React
  Native reuses the TypeScript workflow. Under ADR-0010 it would have cost a supersession
  of this artifact.
- The decision (job graph, Release PR, pinned SHAs, declared permissions) is separated from
  the tooling, so a second stack inherits the guarantees without inheriting `cargo`.

## Known limitation

**`infra` is declared `ci: false`**, reproducing ADR-0010's `applies_to`, which listed
`platform`, `svc`, `lib` and `tool` but not `infra`. Infrastructure repositories therefore
have no CI standard binding them — including `infra-kubernetes` and `infra-terraform`,
which are the repositories that will hold the cluster.

That is a real gap. It is **preserved rather than fixed here**, because a supersession must
be provably equivalent to what it replaces and changing behaviour inside one is how detail
gets lost. Correcting it is a one-line registry change and a separate decision.

## Errata

### 2026-08-20 — the Free-plan deployment gate does not exist on private repositories

**Says:** *"`staging` and `prod` require manual approval through a GitHub Environment — the one
gate that works on the Free plan, because environment protection rules are not a
branch-protection feature."*

**Wrong:** the second half is true and the conclusion drawn from it is not. Environment
protection rules are indeed not a branch-protection feature, and they are still unavailable on
this organisation's plan. Verified against the API on 2026-08-20: creating an environment on a
private repository in the `Paridyum` organisation succeeds and returns `"protection_rules":[]`;
attaching a required reviewer to it is refused with

> `Failed to create the environment protection rule. Please ensure the billing plan supports the required reviewers protection rule.`

The organisation is on the Free plan (ADR-0004) and `knowledge`, `infra-ci` and `tool-forge` are
private. Required reviewers on private repositories are a paid-plan feature; the free allowance
covers public repositories. So an environment can be created and it carries no gate — which is
worse than having none, because the deployment page shows an environment and implies one.

**Intended:** the decision is unaffected — a deploy to `staging` or `prod` is gated on a human,
and CI does not release on its own. What is wrong is the named mechanism. **No approval gate is
available on this plan for private repositories**, and the substitute is a `workflow_dispatch`
deploy that a person triggers rather than approves.

**What the substitute does not give**, stated because it is the reason to revisit this: a trigger
records who *ran* the deploy, not who *authorised* it, and there is no second pair of eyes by
construction. Under ADR-0044 the audit trail is append-only and answerable; "the person who
clicked run" is a weaker answer than "the person who approved" and it is the same person as the
author. Restoring the original mechanism is a billing change, not an engineering one.

**Why this is an erratum rather than a supersession.** ADR-0031 decision 5 permits an erratum for
a factual error and forbids one that changes what an artifact decides. The false statement here is
a claim about a vendor's billing tiers — precisely the class RFC-0020 exists for, and it carried no
citation or date when written. The decision it sits inside is untouched.

**This falsified work written on 2026-08-19 that relied on it**, corrected the same
day this erratum was appended: GLO-0011, which asserted that one manually gated environment
satisfies this ADR completely, and `docs/superpowers/plans/2026-08-19-phase-2b-substrate.md` Task 7,
which instructed the reader to approve the deploy by hand.
