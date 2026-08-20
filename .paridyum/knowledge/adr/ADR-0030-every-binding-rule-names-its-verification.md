---
id: ADR-0030
title: A rule that binds running software names how it is verified, not only what blocks it
type: adr
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [testing, verification, traceability, requirements, evidence, governance]
breaking: false
verified_by: []
enforced_by:
  - check: knowledge.verification_methods_resolve
    severity: error
    rationale: >
      A TST- id that does not exist answers "how is this verified" confidently and wrongly,
      and reads as coverage forever. Identical in shape to an unresolved check id, which
      ADR-0014 already treats as the typo case and fails hard on.
  - check: knowledge.verification_mapping_is_symmetric
    severity: warn
    rationale: >
      A claim written in only one of the two places is one nothing can contradict. The
      enforced_by/blocks pair is compared for exactly this reason and found real drift; this
      is the same comparison for the verification pair, at the same severity, with the same
      excuse for artifacts that cannot legally be edited.
---

# ADR-0030 — A rule that binds running software names how it is verified

## Status

**LOCKED** as of 2026-08-08.

## Context

This repository has one bridge between prose and machinery, and it points in one direction.
`enforced_by` names the check that **rejects a violating change**. ADR-0002 made it the price
of `locked`, ADR-0014 gave the ids a registry, and `check-knowledge.sh` section 4 prints the
backlog of locked artifacts that name nothing.

That bridge answers a narrower question than it appears to. A check reads a repository: it can
see that a file exists, that a pattern is absent, that a policy statement was written. It cannot
see that the policy is **correct**, because correctness is a property of the running system.

The distance between those two is where this platform's worst failure lives, and it is already
written down. `db.row_level_security_enabled_on_tenant_tables` proves a policy exists on a table.
It cannot prove the policy's predicate is right. ADR-0024 decision 3 states the consequence
directly: an outer-only policy on an account-scoped table isolates businesses from each other
and nobody's customers — *"which every dashboard will report as healthy"*. Every declared check
passes. The gate is green. A tenant's customers can read each other's rows.

There is no field anywhere in this repository that would have caught that, because the missing
statement is not "what rejects a violation" but "what demonstrates the behaviour is present".

The failure mode of leaving it unstated is not that testing does not happen. It is that testing
happens against whatever the author found interesting, and the rule the test was supposed to
defend is never connected to it. Six months later "is REQ-0004 §12 satisfied?" is answered by
reading code, by the person who wrote it, under time pressure — and that answer will be wrong in
the generous direction, which is the same conclusion ADR-0028 reached about framework mappings.

`test-registry.yaml` (ADR-0029) removed the reason this could not be stated. Verification methods
now have ids.

## Decision

**An artifact may name `verified_by` — test-type ids from `test-registry.yaml` stating how the
rule is demonstrated to hold in running software. It mirrors `enforced_by`/`blocks` exactly:
the artifact names its types, the registry's `governed_by` names the artifacts each type serves,
and the gate compares the two directions.**

1. **Two bridges, different questions, neither substituting for the other.** `enforced_by` stops
   the wrong change from landing. `verified_by` shows the right behaviour is present. An artifact
   with both is fully covered; with only the first it is defended at the diff and unproven at
   runtime; with only the second it is proven and undefended.

2. **An unresolved `TST-` id fails hard.** Same rule and same reason as an unresolved check id
   (ADR-0014): an id nobody wrote down twice is a typo, and a typo here reads as coverage.

3. **The two directions are compared, and disagreement warns.** Section 7 already does this for
   `blocks` against the artifacts citing a check, and it found real drift. The excuse is the same
   one: an artifact that is `locked` or `superseded` **cannot** gain a `verified_by` entry,
   because that is a content edit and immutability forbids it. So a test type may name an
   immutable artifact unilaterally, and that is the normal path rather than an exception —
   STD-0001 will never carry `verified_by` and is governed by nine test types.

4. **A `deferred` artifact is excused in the same way.** Its verification method does not exist
   yet by construction. It may still name one, and REQ-0016 through REQ-0018 do, because writing
   it while the reasoning is loaded is the entire argument for a trigger-based requirement
   (ADR-0018).

5. **`verified_by` is NOT a precondition for `locked`.** ADR-0002's bargain is unchanged: naming
   a check is what buys the lock. Requiring a verification method as well would make it
   impossible to lock any rule about a system that does not exist yet, which describes almost
   everything here — and a rule nobody can lock is a rule nobody writes down.

6. **The unverified set is reported, never fatal.** `check-knowledge.sh` prints artifacts that
   state a runtime obligation and name no verification method, the way section 4 prints the
   enforcement backlog. It is a reading list, not a gate, because no check can decide whether an
   artifact states a runtime obligation and pretending otherwise would produce a rule that is
   satisfied by adding an irrelevant id.

## Costs accepted

1. **A second bridge is a second thing to keep current, and the first one is not finished.**
   Fifty-nine checks still await `tool-forge`. Adding a field that will accumulate its own
   backlog, while the existing backlog is a third of the registry, is a real risk of governance
   that describes more than it does — the failure C-0001 principle 8 warns about, taken on
   deliberately because the alternative is that the tenancy hole above stays unnameable.

2. **The symmetry check warns rather than fails, so it can be ignored.** Chosen from the section
   7 precedent, where a hard failure would have produced a permanent warning with no legal
   remedy for already-locked artifacts. The cost is that drift here decays quietly, and the
   mitigation is only that the warning is printed on every run.

3. **`verified_by` on a deferred requirement is a promise with no date.** REQ-0016's entry says
   how it will be verified, not that anything verifies it. That is honest and it is also exactly
   the shape that reads as coverage to a skim reader. The status field is the only defence.

4. **This makes the frontmatter contract larger.** `knowledge.schema.json` gains a field that
   every artifact author must now decide about. The decision is usually "leave it empty", and a
   field whose usual answer is empty is a field people stop reading.

## Alternatives rejected

**Require `verified_by` for `locked`.** The obvious symmetric move, and wrong. Every locked
artifact in this repository governs a system that does not exist; requiring a runtime
verification method would have made all twenty-two of them unlockable, and the practical result
is that rules stop being written rather than that verification starts happening. ADR-0002's
price for locking is a check, and it stays a check.

**Put the mapping only in `test-registry.yaml`.** `governed_by` alone would have avoided touching
the schema and would have kept one source of truth. Rejected because it puts the answer in the
wrong place for the question people actually ask. A reader holding REQ-0004 wants to know how §12
is verified; making them grep a registry for the reverse index is the same ergonomics as removing
`enforced_by` and asking them to grep `blocks`. The repository already decided this once, in the
opposite direction, and both fields exist for that reason.

**A `verification` section in each artifact's prose.** Unresolvable, uncomparable, and it would
drift the day a test type was renamed. Prose that nothing checks is a suggestion — the sentence
`README.md` opens with.

**Trace to individual test functions rather than to test types.** Finer-grained and more
faithful, and it would break on the first refactor. A test-type id survives a suite being
rewritten; a test path does not, and a traceability matrix that breaks on rename is one that
gets deleted rather than fixed.

## Consequences

- STD-0008, STD-0009 and REQ-0016 through REQ-0018 carry `verified_by` from creation. They are
  the first artifacts in this repository that state both what rejects a violation and what
  demonstrates the behaviour.
- The chain the repository has been assembling is now complete end to end: a requirement states
  what must be true (ADR-0018), an RFC and an ADR decide how (ADR-0013), a standard says how to
  write it, `enforced_by` rejects the wrong diff (ADR-0002, ADR-0014), `verified_by` names the
  test type that demonstrates the right behaviour (this ADR), the registry's `evidence` field
  says what the run leaves behind (ADR-0029), and `frameworks:` maps the whole thing outward
  (ADR-0028). No link in that chain is prose.
- The unverified report will be long on its first run and that is the point. It is the
  companion to the enforcement backlog and it should be read the same way — as the ordered list
  of what to build, not as a score.
- Nothing about existing artifacts changes. `verified_by` defaults to absent, twenty-two locked
  artifacts cannot acquire it, and no gate result moves.

## Related

- [ADR-0029](ADR-0029-test-types-are-registry-data.md) — the id-space this field resolves against
- [ADR-0002](ADR-0002-ci-gate-is-the-lock.md) · [ADR-0014](ADR-0014-declared-check-ids.md) — the first bridge, and why an unresolved id fails hard
- [ADR-0018](ADR-0018-requirements-are-first-class.md) — deferred, and why a trigger is mandatory
- [ADR-0024](ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) — decision 3, the failure this decision exists to make nameable
- [ADR-0028](ADR-0028-controls-map-outward-to-frameworks.md) — the same argument about answering from memory under pressure
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) · [STD-0009](../standards/STD-0009-tenant-isolation-testing.md) — the first users
