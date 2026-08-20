---
id: STD-0007
title: Change slices and pull request size — what "small PR" means as a number
type: standard
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [review, pull-requests, slicing, spec-driven, change-lifecycle]
breaking: false
enforced_by:
  - check: pr.diff_within_review_budget
    severity: error
    rationale: >
      ADR-0013 stage 4 already requires small PRs and has required it since it was locked.
      An unquantified adjective is not a rule, and nothing has ever measured it.
  - check: pr.slice_cites_requirement
    severity: warn
    rationale: >
      A slice that cites no requirement is work whose reason lives only in someone's head,
      which is C-0001 principle 3 applied to the unit of work rather than the decision.
---

# STD-0007 — Change slices and pull request size

## Status

**LOCKED** as of 2026-08-07, with two checks.

## This is not a new rule

**ADR-0013 stage 4 is locked and already says it:** *"Implement — small PRs referencing the
RFC — each independently reviewable."*

That has been binding since ADR-0013 landed. What was missing is that **"small" is not a
number and nothing measured it.** An adjective in a locked artifact is a rule that cannot be
violated because it cannot be evaluated — the same shape as a `locked` artifact with an empty
`enforced_by`, which ADR-0002 exists to prevent.

So this standard adds no policy. It fixes the threshold ADR-0013 deliberately left open and
names the checks, exactly as STD-0006 did for ADR-0024's declaration syntax. ADR-0013 is
untouched.

## The lifecycle this sits inside

Nothing here is new machinery. Each stage already had an artifact type.

| Stage | Artifact | Already decided by |
| ----- | -------- | ------------------ |
| The large specification | an **RFC** | ADR-0013 stage 2 |
| Decomposed into contexts | one **REQ-** per bounded context, atomic items numbered | ADR-0019 step 7, ADR-0018 |
| Sliced into reviewable units | a **slice document**, generated, ephemeral | *this standard* |
| Implemented | one small PR per slice, citing `REQ-NNNN §N` | ADR-0013 stage 4 |

The only gap was between a requirement and a pull request. A `REQ-` is durable and outlives
every design that satisfies it (ADR-0018); a pull request is a single reviewable increment. The
slice document is the thing that maps one to the other.

## 1. A slice document is generated, ephemeral, and not a knowledge artifact

**Slices do not live in `knowledge/`.** They live in the repository being changed, under
`plan/`, and they are deleted or archived when the work merges.

This is deliberate and it is ADR-0018's own argument applied one level down. The
`decompose-spec` procedure rejected one file per requirement because *"a hundred requirements
as a hundred files means a hundred statuses nobody maintains; within months most are stale, and
a status you cannot trust poisons the ones you can."* Slices are **more** numerous than
requirements, so that argument binds harder here, not less. A governance base filling with
finished work items is a governance base people stop reading.

They are also **generated output**, so C-0004 applies: regenerated from the requirement, never
hand-edited into divergence from it.

A slice document states, and nothing more:

`NNNN` is written literally rather than as a plausible number. The `decompose-spec` skill
already warns that *"a concrete id in an example is indistinguishable from a real citation"*,
and the first draft of this standard used a plausible four-digit id and was rejected by the
dangling-reference check on its first run — the hazard was documented and writing it down was
not enough. The *explanation* of the mistake then repeated it, because naming the offending id
is itself a citation of it: the same self-inflicting shape `check-knowledge.sh` already records
for `DECISION-DEBT.md`, where reporting a dangling id would make the ledger cite it and turn the
gate permanently red.

```markdown
# Slice: <one sentence, no "and">

Implements: REQ-NNNN §3, §4
Depends on: <slice ids that must merge first, or "nothing">

## Done when
- [ ] <observable condition, not a task>
- [ ] <observable condition>

## Out of scope
- <the adjacent thing a reviewer will ask about>
```

**`Out of scope` is the load-bearing section**, and it is the one people skip. A small PR
without it invites the review comment *"why didn't you also…"*, and answering that in review is
how a small PR becomes a large one.

## 2. The review budget

Measured on the **branch diff against its merge base**, not per commit — ADR-0025's automatic
checkpoints mean commit count carries no information about review effort.

| Changed lines | Verdict |
| ------------- | ------- |
| ≤ 250 | fine |
| 251 – 400 | warn: say in the PR body why it could not be split |
| > 400 | **error** |

Counted: additions plus deletions, on files that a human must actually read.

**Excluded, because a reviewer does not read them:**

- generated output (C-0004 says it is regenerated, never reviewed line by line)
- lockfiles — `Cargo.lock`, `pnpm-lock.yaml`, and equivalents
- vendored `.paridyum/knowledge/` snapshots (ADR-0001)
- test fixtures and golden files
- pure file moves with no content change

**Not excluded: tests.** Excluding tests would make "add a large feature with no tests" the
cheapest way to pass this check, which inverts STD-0001. If tests push a slice over budget, the
slice is too big.

### Why 400, and why the number is arguable

400 is where review effectiveness collapses in the published measurements, and it is generous
rather than aggressive — the intent is to catch the 4000-line review, not to police a
390-line one. **The threshold is the part of this standard most likely to be wrong.** If it
is obstructing good work, change the number here in one line rather than accumulating
exceptions, which is what C-0001 says to do with a principle that keeps getting in the way.

### The escape hatch, and its cost

A PR may exceed the budget by stating why in its body:

```
Review-budget-exception: <reason>
```

Legitimate reasons are narrow and mechanical: a rename across many files, a dependency bump
with wide but uniform effects, a generated-file refresh the exclusions did not catch. **"It was
easier to do together" is not one.** The exception is recorded in the PR rather than in a
config file specifically so that it is visible to the reviewer being asked to absorb it, and
so that a pattern of exceptions is visible in the log.

## 3. Slicing rules

1. **A slice is vertical.** It changes whatever layers it must to make one observable thing
   true. A slice that is "the database part" cannot be reviewed for correctness, because
   nothing it does is observable.
2. **A slice merges on its own.** If it must land with another slice to work, they are one
   slice — or one of them is missing a feature flag.
3. **A slice cites its requirement.** `REQ-NNNN §N`, which is why the `decompose-spec`
   procedure numbers atomic requirements inside a `REQ-` file.
4. **A slice's responsibility sentence contains no "and."** STD-0004's test, applied to work
   instead of to modules. If the sentence needs "and", the split is already visible.
5. **Ordering is declared, not discovered.** `Depends on:` names the slices that must merge
   first. A dependency found during review is a decomposition error.

## Costs accepted

1. **Slicing takes time before any code exists, and the pressure to skip it is highest exactly
   when it matters most** — on a big change, under time pressure. The mitigation is that the
   check fires at the end, when splitting is most expensive; that is a genuinely bad property
   and the honest reason to slice up front.
2. **Small PRs mean more PRs.** More CI runs, more review context-switching, more merge
   ordering to think about. This is a real cost paid to avoid a worse one.
3. **A number will be wrong for some change.** Hence the exception, and hence the warning band
   rather than a cliff at 250.
4. **`plan/` is untracked-adjacent clutter.** Slice documents accumulate in the working
   repository until someone removes them. Making them knowledge artifacts would have solved
   that by making a worse problem — see §1.
5. **The check cannot run before a pull request exists**, so it reports `n/a` in a repository
   with no branch to compare. That is honest and it means the rule is unenforced for work
   committed straight to `main`, which is how this repository currently operates.

## Related

- ADR-0013 — stage 4 already requires small PRs; this quantifies it and does not amend it
- ADR-0019, ADR-0018 — the RFC → REQ- decomposition this slices from
- STD-0006 — the same ADR-leaves-it-open, STD-fixes-it pattern, for migration syntax
- STD-0001 — why tests are not excluded from the count
- STD-0004 — the no-"and" test, applied here to work rather than modules
- C-0004 — generated output is regenerated, which is why it is excluded and why slices are too
- ADR-0025 — automatic checkpoints, which is why the budget is measured on the branch diff
