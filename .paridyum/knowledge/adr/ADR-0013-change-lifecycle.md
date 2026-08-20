---
id: ADR-0013
title: Significant changes are designed and reviewed before they are implemented
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [sdlc, process, rfc, design, review, discipline]
breaking: true
enforced_by:
  - check: pr.significant_change_references_design
    severity: error
    rationale: Significance is computed from the diff, so the gate cannot be skipped by not self-declaring.
  - check: pr.design_artifact_predates_implementation
    severity: warn
    rationale: An RFC committed in the same PR as the code it justifies is a rationalisation, not a design.
  - check: pr.body_completed
    severity: error
    args: { template: ".github/pull_request_template.md" }
  - check: rfc.has_alternatives_section
    severity: error
    rationale: A design with no rejected alternatives has not been designed, only described.
---

# ADR-0013 — Significant changes are designed and reviewed before implementation

## Status

**LOCKED** as of 2026-08-06.

## Context

The default failure of AI-assisted development is speed in the wrong direction: an
agent can produce a working implementation of the wrong design faster than anyone can
review it. Once the code exists it acquires momentum — it works, it has tests, and
rejecting it now costs more than accepting it.

Google's internal analysis of design docs found their primary value was not the
document but **the forced specificity of the author's thinking**, with teams reporting
substantially fewer post-hoc architecture disputes when one existed. That effect is
larger, not smaller, with agents: an agent asked to design first must make its
assumptions explicit while they are still cheap to correct.

The obstacle is that "significant change requires a design" is unenforceable as
written. Everyone believes their change is not significant, particularly at the moment
they want to start coding.

## Decision

**The lifecycle is fixed, and significance is computed from the diff rather than
self-declared.**

### The lifecycle

| Stage | Artifact | Gate |
| ----- | -------- | ---- |
| 1. Define | issue or RFC §Problem | the problem stated without a solution in it |
| 2. Design | RFC with alternatives | written before implementation begins |
| 3. Review | RFC `status: accepted` | a human accepted it |
| 4. Implement | small PRs referencing the RFC | each independently reviewable |
| 5. Test | STD-0001 | patch coverage, integration where it applies |
| 6. Document | STD-0002, STD-0005 | README/ARCHITECTURE/API/RUNBOOK, changelog |
| 7. Release | ADR-0010 | Release PR, controlled deploy |

Stage 2 produces an **RFC**; if the outcome is a decision that binds future work, it
graduates to an **ADR** at stage 3. Not every RFC becomes an ADR — most describe how
to build one thing and are finished when it is built.

### Significance is mechanical

A change is **significant**, and requires a referenced RFC or ADR, when the diff does
any of:

- changes a **public API** — added, removed, or altered `pub` items in a `lib-*` crate
- changes an **HTTP or event contract** — `specs/openapi.yaml`, event struct shapes
- adds or removes a **dependency** in `Cargo.toml` outside the ADR-0005 locked stack
- adds a **migration** or alters a schema
- adds a **new crate**, or a new top-level module under `src/`
- changes anything under `helm/`, `terraform/`, or `.github/workflows/`
- introduces a **new service dependency** in `manifest.yaml`

Everything else — bug fixes, refactors within a module, test additions,
documentation, dependency bumps inside the locked stack — proceeds directly to
stage 4. Most work is not significant, and treating it as though it were is how a
process gets abandoned.

The list is deliberately narrow and mechanical. A judgement-based definition would be
resolved in favour of skipping it every time.

### Design before code, checked by ordering

`pr.design_artifact_predates_implementation` compares commit timestamps: the RFC must
exist before the implementation commits. An RFC committed alongside the code it
justifies is a rationalisation written to satisfy a gate, which is worse than no RFC
because it carries the same authority while having done none of the thinking.

It is a **warning**, not an error, because a legitimate case exists: exploratory work
that discovers the design, then writes it up and rebuilds. What must not happen
silently is implementation-first passed off as design-first.

### Small, reviewable increments

Implementation PRs are scoped so each is independently reviewable and independently
revertable. No line-count rule — the same reasoning as STD-0004. The test is whether a
reviewer can hold the change in their head and whether reverting it alone leaves the
system working.

An RFC accepted at stage 3 typically yields several stage-4 PRs. That is the intended
shape: the design is reviewed once, the increments are reviewed for correctness
against it.

## Consequences

**Accepted costs**

- Significant changes are slower to start. That is the purpose; the cost is paid
  before the code exists rather than after.
- The significance list will be wrong at the edges — something significant will slip
  through and something trivial will be gated. Both are fixed by amending the list,
  which is a visible change to this ADR rather than a private judgement call.
- Writing an RFC for a change an agent could implement in ten minutes feels
  disproportionate. The check is narrow specifically so this is rare.

**Gained**

- Design assumptions surface while they are cheap to change.
- An agent cannot reach a working implementation of an unreviewed architecture, which
  is the specific failure mode this exists to prevent.
- Every significant decision has a written rationale, which is what C-0001 principle 3
  and the whole knowledge base depend on.

**Rejected alternatives**

- *Require an RFC for every change* — abandoned within a month, and the abandonment
  discredits the process for the changes that needed it.
- *Leave significance to author judgement* — the author wants to start coding; that
  judgement is not neutral.
- *Block implementation with a PreToolUse hook* — considered, since ADR-0007
  establishes hooks as the strongest lock. Rejected: an agent cannot know at first
  file-write whether the eventual diff will be significant, so the hook would fire
  constantly on trivial work and be disabled. The gate belongs where the complete
  diff is visible, which is the PR.

## Errata

### 2026-08-08 — the lifecycle table routes stage 7 to a superseded decision

**Says:** stage 7 of the lifecycle table names **ADR-0010** as the governing artifact for
"Release PR, controlled deploy".

**Wrong:** ADR-0010 is `superseded_by: ADR-0021`. This matters more than a stale citation
usually would, because this table is what the `design-change` skill routes agents through — an
agent following it for a release lands on a replaced decision, which is the exact failure
CLAUDE.md's *"never infer architecture, retrieve it"* rule depends on not happening.
Nothing detected it: `knowledge.resolved_by_names_live_decision` covers the `resolved_by`
frontmatter field only, and no check reads prose citations for supersession.

**Intended:** stage 7 to name **ADR-0021**, which supersedes ADR-0010 and holds the CI and
release design.

### 2026-08-08 — `pr.body_completed` points at a template this repository does not have

**Says:** `enforced_by` cites `pr.body_completed` at `severity: error` with
`args: { template: ".github/pull_request_template.md" }`.

**Wrong:** no such file exists here — `.github/` contains only `workflows/`. This ADR's `scope`
includes `org`, so under ADR-0015 it binds this repository, pointing an error-severity check at
an absent file. Latent only because the check's owner is `tool-forge`, which does not exist.
The template does ship in `templates/rust-service/.github/`, so the defect is that this
repository never adopted its own scaffold rather than that the file was never written.

**Intended:** no change to the decision. Recorded so the check is not implemented against a
path that will fail on the repository that declares it.
