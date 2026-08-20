---
name: slice-requirement
description: Turn an accepted requirement into ordered, independently reviewable slices, one per pull request, before writing code. Use after decompose-spec has produced REQ- artifacts, when a requirement is too large for one PR, or when asked to plan implementation of something already specified.
---

# Slice a requirement into reviewable units

Input: `$ARGUMENTS` — a `REQ-` id, optionally with section numbers.

This is the stage between a requirement and a pull request. `decompose-spec` answers *what
must be true* and produces `REQ-` artifacts; this answers *in what order, in what increments*
and produces slice documents. STD-0007 fixes what a slice is and how big a PR may be.

**Do not write implementation code during this skill.** The output is a plan. It is also
generated output under C-0004 — regenerate it from the requirement rather than editing it into
divergence.

---

## 0. Retrieve, do not infer

```sh
cat requirements/REQ-NNNN-*.md          # the requirement, with its numbered items
ls plan/                                # slices that already exist for it
cat standards/STD-0007-*.md             # the budget and the slice shape
```

The requirement's `status` decides whether this is legitimate. **A `deferred` requirement is
not ready to slice** — deferred means accepted as real but deliberately unscheduled (ADR-0018),
so slicing it is starting work somebody decided not to start. If its `trigger` has fired, say
so and change the status first, in its own change.

State plainly which numbered items you are slicing. `REQ-NNNN §3, §4` is the unit of
traceability and it exists because `decompose-spec` numbers atomic requirements for exactly
this.

---

## 1. Find the vertical seams

A slice is **vertical**: it changes whatever layers it must so that one observable thing
becomes true. The failure to avoid is horizontal slicing — "the migration", then "the service",
then "the endpoint" — because none of those can be reviewed for correctness. A reviewer looking
at a migration alone can check its syntax and cannot check whether it is the right migration.

For each candidate slice, write **one sentence with no "and"**. STD-0004's test, applied to
work rather than to modules: if the sentence needs "and", the seam is already visible and you
have found two slices.

> Horizontal, wrong: *Add the accounts table.*
> Vertical, right: *An operator can create an account and see it listed.* — no; that is two.
> Vertical, right: *An operator can create an account.* / *An operator can list accounts.*

---

## 2. Order them, and declare the order

`Depends on:` names the slices that must merge first. Nothing else.

Ask of each pair: **can these two merge in either order?** If yes, they are independent and
should not declare a dependency — a false dependency serialises work for no reason. If no, say
which comes first and why in one clause.

**A dependency discovered during review is a decomposition error**, not a surprise. It means
this step was skipped.

If a slice cannot merge alone because the feature would be half-visible, the answer is usually
a flag rather than a bigger slice. Say so explicitly; "hidden behind `X`" is part of the plan.

---

## 3. Size each slice against the budget

STD-0007: **≤250 lines is fine, 251–400 warns, >400 fails.** Additions plus deletions, on files
a human reads — tests included, lockfiles and generated output excluded.

Estimate before writing, not after. The check fires at the end, when splitting is most
expensive, and that is a bad property of the check rather than a reason to discover the problem
late.

If a slice looks like it will exceed 400 lines, one of these is true and you must say which:

- it is two slices wearing one sentence — split it;
- it is mechanically large — a rename, a generated refresh — and needs
  `Review-budget-exception:` with the mechanical reason;
- it is genuinely one indivisible change over 400 lines, which is rare and worth stating as
  such rather than assuming.

---

## 4. State what each slice is not

**`Out of scope` is the section people skip and the one that does the work.** It pre-empts
*"why didn't you also…"* in review, and answering that question inside a review is the most
common way a small PR becomes a large one.

Name the adjacent thing a reviewer will reach for: the validation you are not adding yet, the
second entity that looks similar, the error path deferred to a later slice.

---

## 5. Write the slices

One file per slice, in the repository being changed:

```
plan/<NN>-<kebab-summary>.md
```

**Not in `knowledge/`.** STD-0007 §1 gives the reason: `decompose-spec` rejected one file per
requirement because a hundred files means a hundred statuses nobody maintains, and slices are
more numerous than requirements, so that argument binds harder here. Slices are working
documents that end at merge; requirements outlive the designs that satisfy them.

Use STD-0007's template exactly. Every slice cites `REQ-NNNN §N`.

---

## 6. Check the plan before proposing it

- Every numbered item in scope is covered by exactly one slice — gaps and overlaps are both
  errors, and an overlap means two PRs will conflict.
- No slice's sentence contains "and".
- No dependency cycle. Two slices that each depend on the other are one slice.
- Every slice is independently mergeable, or names the flag that makes it so.
- The slices in total do not quietly exceed the requirement. Scope added here was never
  reviewed as a requirement, which is how a specification gets amended by an implementation.

---

## 7. Report, then stop

```
Requirement  REQ-NNNN §<items>, status <status>
Slices       N, estimated <min>–<max> reviewable lines each
Order        <the dependency chain, or "all independent">
Flags        <feature flags the plan needs>
Uncovered    <numbered items deliberately not sliced yet, and why>
Exceptions   <any slice expected to need Review-budget-exception, with the mechanical reason>
```

**Then stop and ask.** Slicing is a plan, and ADR-0013 puts a human between a plan and its
implementation. If the answer is "just build it", the honest reply is that an unreviewable PR
costs more than the planning did — the plan is minutes and a 4000-line review is either hours
of somebody's attention or an approval nobody actually gave.
