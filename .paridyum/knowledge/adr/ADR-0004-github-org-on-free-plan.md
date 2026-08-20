---
id: ADR-0004
title: Repositories live in the Paridyum organisation on the GitHub Free plan
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [github, governance, cost, branch-protection]
breaking: false
enforced_by:
  - check: repo.owner_is_paridyum_org
    severity: error
  - check: repo.has_codeowners
    severity: error
    rationale: Advisory on Free, but must already be correct the day the plan is upgraded.
  - check: repo.default_branch_is_main
    severity: error
  - check: org.branch_protection_reconciled
    severity: warn
    rationale: Downgraded to warn because Free cannot apply protection to private repos. Promote to error on upgrade to Team.
---

# ADR-0004 — Repositories live in the Paridyum organisation on the GitHub Free plan

## Status

**LOCKED** as of 2026-08-06.

## Context

The `Paridyum` GitHub organisation already exists and is on the **Free** plan.
Development is currently one human (`rohanbatrain`) plus AI agents.

ADR-0002 locks the position that decisions are enforced by an executable gate. The
strongest form of that gate is a **required status check on a protected branch**: CI
fails, the merge button is disabled, nothing gets in.

GitHub Free does not offer this for private repositories. Protected branches, required
reviews, and enforced CODEOWNERS review are Team-plan features for private repos. On
public repositories they are free.

This produces a direct tension with ADR-0002 that must be recorded rather than glossed.

## Decision

Repositories are created under the **`Paridyum` organisation**, on the **Free plan**,
**private by default**.

The enforcement gap is accepted and mitigated in three layers:

1. **Pre-commit hook** (`forge knowledge check --staged`) installed by `forge` at
   scaffold time. Blocks the commit locally. Bypassable with `git commit --no-verify`.
2. **CI on every PR and push to `main`.** Cannot block a merge on this plan, but it
   goes red, and red is visible in the PR, in the commit list, and in `forge audit`.
3. **`forge audit`** sweeps every repository in the organisation and reports drift.
   This is the only layer that catches a bypass after the fact, which makes it the
   most important one to build early, not last.

CODEOWNERS files, branch-protection *definitions*, and required-check *declarations*
are generated and committed from day one even though the platform ignores them today.
They are inert configuration waiting for a plan change — not work to be done later.

### Upgrade trigger

Move to GitHub Team when **any** of the following becomes true:

- a second person gains write access to any repository
- an agent bypasses the gate and merges a violation that reaches `main`
- the first service handles real user data

On upgrade, the work is: change plan, run `forge org reconcile` to apply the already
committed branch-protection definitions, and promote
`org.branch_protection_reconciled` from `warn` to `error` in this ADR's `enforced_by`.
That is a configuration change, not a migration.

### Rejected alternative: make repositories public

Public repositories get full branch protection and rulesets for free, which would close
the ADR-0002 gap at zero cost. Rejected because it would publish the knowledge base,
architecture decisions and roadmap of a business at its earliest and most copyable
stage. The gap is judged cheaper than that disclosure. Revisit per-repository — a
public `lib-*` or `tool-forge` is defensible later and would get real enforcement.

## Consequences

- **ADR-0002's guarantee is partial today.** The lock is advisory-plus-friction, not
  a hard block. Anyone with write access can merge a red build. That is one person and
  their agents, which bounds the risk but does not eliminate it.
- `forge audit` is promoted from a "nice to have after MVP" to a **v0.2 requirement**,
  because on this plan it is the only backstop.
- Actions minutes are capped on Free for private repos. Knowledge checks must stay
  fast and must not run a full Rust build to validate frontmatter.
- Naming (`svc-*`, `lib-*`) and org teams work normally; only merge-blocking is absent.
