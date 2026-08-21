---
id: ADR-0054
title: Capability is added, never swapped, and a working path is kept until keeping it is untenable
type: adr
status: accepted
created: 2026-08-20
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, web, mobile, sdk]
applies_to: ["*"]
tags: [evolution, compatibility, dependencies, migration, toolchain, adr-0021, c-0010]
breaking: false
enforced_by: []
---

# ADR-0054 — Capability is added, never swapped

## Status

**ACCEPTED** as of 2026-08-20. Not locked: the rule turns on a judgement —
whether keeping a path has become untenable — and `locked` is for rules
something can mechanically reject (ADR-0002). What CAN be checked is narrower
and is listed under Consequences; those checks are not written yet, which is
why `enforced_by` is empty rather than aspirational.

## Context

The decision that produced this rule was small. `tool-docket` renders documents
with `pdflatex`, which requires a multi-gigabyte TeX Live installation, and
Tectonic does the same job from a single binary that fetches what it needs.
The obvious move was to swap one for the other.

The obvious move was wrong, and the reason generalises.

**A working path has users you have not enumerated.** Every machine in the
estate already has TeX Live; every rendered document in the register was
produced by it and its reproducibility claim is against it. Swapping the engine
would have invalidated the toolchain recorded on every existing register entry,
for a benefit — an easier install — that only a person who has not installed it
yet experiences.

**The pressure to swap is strongest exactly when the new thing is better.**
That is what makes this a rule rather than a preference. "Tectonic is better"
was true and was not the question. The question was what breaks for the people
already served, and the honest answer is available only if you ask it before
deleting the old path rather than after.

**This estate has already paid for the opposite instinct once.** ADR-0009
records an entire previous estate archived rather than migrated. That was the
right call at the time and it is expensive precisely once; a habit of replacing
rather than extending makes it recurring.

The competing pressure is real and must be stated: **carrying two paths costs
more than carrying one.** Two code paths, two sets of failure modes, two things
to test, and a combinatorial surface that grows with each addition. A rule that
never permits removal produces a system nobody can change. That is why this
decision has a stopping condition rather than being absolute.

## Decision

**1. Adding a capability must not remove a working one.** Where a new
implementation, engine, format or interface is introduced alongside an existing
one, the existing one keeps working. The new one is reached by detection,
configuration or an explicit flag — never by the old one disappearing.

**2. Prefer detection over configuration, and configuration over a flag day.**
Where both paths can be present, detect which is available and use it. Where
detection is ambiguous, let the operator choose. Never require every consumer
to change on the same day.

**3. Record which path produced a durable artifact.** Anything that outlives
the run — a rendered document, a migration, a generated file — records the
implementation and version that produced it. Without that, "reproduce this"
becomes unanswerable the moment a second path exists.

**4. A path is removed only when keeping it is untenable, and removal is its
own decision.** Untenable means one of: the upstream is unmaintained or has a
live vulnerability with no fix; it blocks a correctness or security fix that
cannot be made otherwise; or its cost is demonstrable rather than asserted.
"Newer", "cleaner", "nobody uses it any more" and "it complicates the code" are
not sufficient on their own. Removal is recorded as a superseding decision that
names which of the three conditions is met.

**5. This does not license carrying every path forever.** A path retained under
this rule carries an owner and a review date, in the repository's
DECISION-DEBT. A retained path nobody has looked at in a year is not being
maintained, it is being ignored, and that is a finding.

## Consequences

**Accepted costs.**

- **Two paths cost more than one**, in code, in tests and in the failure modes
  an operator has to understand. This is the whole price and it is paid every
  time the rule applies.
- **The combinatorial surface grows.** Two engines and two output formats is
  four combinations, and nothing here caps that growth except point 5's review.
- **It slows down cleanup.** Some code that ought to go will stay longer than a
  free hand would keep it, and some of that will look like cowardice in
  hindsight.
- **The stopping condition is a judgement**, so it can be argued with, and a
  determined author can always frame a preference as a cost. Point 4's three
  conditions narrow that but do not close it.

**Gained.**

- An upgrade stops being an event. A machine with TeX Live keeps working; a
  machine without it gets Tectonic; nobody is blocked on a migration.
- Reproducibility survives, because point 3 makes every durable artifact say
  what produced it.
- C-0010's "no prototype tier" gets a companion: not only is what we build
  finished, what we have built stays working.

**Rejected alternatives.**

- **Swap and migrate.** Cheapest to reason about, and it externalises the cost
  onto whoever depended on the old path — including the register's existing
  reproducibility claims, which cannot be migrated because they are historical
  statements about what happened.
- **Support everything forever.** Removes the judgement, and produces a system
  where no dependency can ever be dropped. Point 4 exists so that removal has a
  route.
- **Decide case by case with no rule.** What was happening already. It works
  until the week someone is in a hurry, which is exactly when the decision is
  worst and least examined.

## Notes

The mechanically checkable parts, for whoever writes the checks:

- a durable artifact record that names no producing implementation and version
  (point 3) is detectable in a schema;
- a retained alternate path with no owner or review date in DECISION-DEBT
  (point 5) is detectable by reading that file;
- a commit that deletes an implementation without a superseding decision naming
  a point 4 condition (point 4) is detectable at review.

Point 1 itself is not mechanically checkable, because "working" is a property
of consumers rather than of source.
