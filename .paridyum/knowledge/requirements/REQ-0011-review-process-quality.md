---
id: REQ-0011
title: A change is small enough to review, and reviewed by someone who did not write it
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [code-review, pull-requests, collaboration, process]
breaking: false
trigger: >
  A second person or a second agent identity can open a pull request — the moment
  "someone who did not write it" becomes possible.
review_date: 2026-11-01
enforced_by: []
---

# REQ-0011 — A change is small enough to review, and reviewed by someone who did not write it

## Status

**DEFERRED** as of 2026-08-07. The organisation has one member. There is no second
reviewer, no merge gate — GitHub Free has no required reviews on private repositories
(ADR-0004) — and direct push to `main` is the working path.

Review rules with one person are theatre. Written now, they would be waived on day one,
and a waived rule teaches that rules are waivable.

## Requirement

**A change shall be small enough that a reviewer can hold it in their head, reviewed by
someone other than its author, within a stated response time — with review depth scaling
by risk rather than applied uniformly.**

## What already exists

ADR-0013 governs **whether a change was designed**: significance computed from the diff, a
referenced RFC or ADR required, design predating implementation, and the PR body completed
against a template. Four checks, all declared.

STD-0004 governs **how the code is organised**, and its "state the responsibility in one
sentence without *and*" test applies to a pull request as readily as to a module.

C-0004 escalates governance-file edits to a human. ADR-0012 sets read-plus-fork access.

The gap is the **mechanics of review itself** — size, timing, who, and how deep.

## Numbered items

1. **PR size limits.** Warn above roughly 400 changed lines, require a split above roughly
   1000, excluding generated and vendored files. This is the highest-leverage review rule
   that exists: review quality collapses with size, and a 2000-line PR receives approval
   rather than review. Every other item here matters less than this one.
2. **No self-approval and no self-merge** for anything above the lowest risk tier.
3. **A merge queue** rather than merge races, once more than one change is ever in flight.
4. **Review SLA.** A first response within a stated window, and stale pull requests closed
   rather than accumulating. An unreviewed PR is worse than a rejected one — it blocks
   without deciding.
5. **Domain-expert review required** for authentication, cryptography, payments, personal
   data, migrations, and CI configuration. These are the changes where a generalist review
   is indistinguishable from no review.
6. **Review depth by risk tier**, per REQ-0004 §1. A README fix and a payments change
   receiving the same process is how the process gets abandoned for both.
7. **Comment conventions.** Blocking, nit, and question distinguished explicitly, so an
   author can tell what must be addressed from what is an opinion.
8. **Branch and merge policy stated once.** Rebase or merge commit, squash rules, long-lived
   branch limits — and **trunk-based versus a release-branch model chosen deliberately**,
   because the two pull in opposite directions and drifting between them produces the worst
   of each.
9. **Generated and vendored code exempted** from size limits and from line-by-line review,
   but not from the check that it is actually generated and current.
10. **Large-scale change process.** Who may touch 500 files at once, and how a codemod is
    reviewed when reading the diff is not possible. The answer is reviewing the *codemod*,
    not its output.
11. **An AI-authored change is not approved solely by an AI** above the lowest tier. Recorded
    here and in REQ-0007 §3; unenforceable today for the same reason as item 2.

## Why this is not startable today

Items 2, 3, 5, 6 and 11 need a second reviewer. Items 1, 4 and 7 need pull requests, which
need a merge gate, which needs a paid plan (ADR-0004).

Item 8 is decidable now and is deliberately deferred: branching strategy is a function of
team size and release cadence, and choosing one for a team of one would lock in a model
sized for the wrong organisation.

## Related

- ADR-0013 — whether a change was designed; this is whether it was reviewed
- ADR-0004 — the plan limits that make most of this unenforceable today
- ADR-0012 — read-plus-fork, which is the access model this assumes
- REQ-0004 — risk tiering, which items 6 and 11 depend on
- REQ-0007 — agent authority; item 11 is the same rule from the agent side
- STD-0004 — the one-sentence-without-"and" test, applied to a diff
