---
id: ADR-0029
title: A test type is registry data, and it declares which failure it catches and when
type: adr
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [testing, taxonomy, registry, tiers, evidence, verification]
breaking: false
verified_by: []
enforced_by:
  - check: knowledge.test_registry_valid
    severity: error
    rationale: >
      A taxonomy nothing validates is a document, and this one names ids that other files
      resolve against. An axis that is not declared, a tier that is not declared, a check id
      that does not exist and a deferred entry with no trigger are each a silent hole — the
      entry reads as governed and governs nothing, which is the exact defect ADR-0014 was
      written to convert into a loud failure.
---

# ADR-0029 — A test type is registry data

## Status

**LOCKED** as of 2026-08-08.

## Context

STD-0001 is a good testing standard for the thing it covers, and what it covers is one axis.
It fixes a Rust toolchain, four layers — unit, component, integration, mutation — coverage
thresholds and determinism rules. Everything in it is correct and none of it reaches the
questions this platform is about to face.

Nothing in this repository said where a cross-tenant isolation test lives, when a load test
runs, whether a fuzz target belongs on a pull request, what a migration test must assert, or
what evidence a security scan leaves behind. Those are not gaps in STD-0001; they are outside
its subject. The gap was that there was no place for them to be outside it *in*.

That absence has a predictable shape, and this repository has already recorded it four times.
A list that must be extended, held in a place where extending it is expensive, does not get
extended — it gets bypassed. ADR-0014 found it in check ids hardcoded in code. ADR-0015 found
it in scope resolution written as an enumeration. C-0005 found it in a repository-class table
inside a locked constitution. ADR-0022 found it in artifact types hardcoded at fifteen sites,
three of which were already carrying a stale copy. Every instance failed the same way: the list
was never wrong, it was never *grown*, and the work that needed a new entry went somewhere
untracked instead.

The test-type list was the next instance and it had two aggravating properties. First, STD-0001
is **locked**, so adding a fifth layer costs a supersession — a price high enough that the
realistic outcome is a CI job appearing with a name somebody invented that morning. Second,
the list is already duplicated: the standard's prose, the nextest profile names and the shared
workflow's job names are three copies with nothing comparing them.

There is a third pressure specific to what testing is. A testing taxonomy assembled from the
industry's vocabulary arrives with entries that cannot be owned or evidenced — "security
testing", "reliability testing" — which are categories of failure, not things a repository has
a suite of. Left as test types they produce a permanently unsatisfiable list, and a list nobody
can satisfy is one nobody consults.

## Decision

**`test-registry.yaml` is the complete id-space of Paridyum test types. Every test type has a
`TST-` id, exactly one axis, exactly one tier, a status, the failure it detects, and the
machine-readable artifact a run leaves behind.**

1. **Three independent axes, never fused.** `axis` says which kind of failure the type
   addresses. `tier` says when it runs and therefore what it may cost. `status` says whether it
   binds today. Most taxonomies fuse these and become unusable; keeping them separate is what
   makes "run everything everywhere" and "run nothing until release" both expressible as
   mistakes rather than as the only two options.

2. **An axis is not a test type.** `security` and `resilience` are axes. Nothing owns a "security
   testing" suite, nothing evidences one, and no status can honestly be assigned to one. They
   are declared in `axes:` and the entries beneath them are concrete.

3. **A tier declares its own budget and whether it blocks.** Ten minutes on a pull request is a
   constraint on what may be added there, not a target to drift past. A tier that blocks nothing
   must say so, because a non-blocking tier nobody reads is indistinguishable from an absent one.

4. **`status` is `required`, `recommended` or `deferred`, and `deferred` must carry a trigger.**
   The same three-way distinction ADR-0018 makes for requirements, for the same reason: a
   taxonomy marking everything required is one nobody can comply with, and one marking nothing
   required is a glossary. Fifty-eight types are declared and most are deferred, each naming the
   event that would start it.

5. **`evidence` is a required field.** A test type that produces no machine-readable artifact
   cannot answer "did that run, on that commit" three months later. `none` is a legal value and
   is an admission rather than a default. This is also what makes SSDF `PO.3.3` — configure
   tools to generate artifacts of their support of secure development — true here rather than
   claimed.

6. **`detects` is phrased as a defect, never as an activity.** "Runs k6 against the service" is a
   tool preference. "Expected traffic producing worse latency than the declared budget" is a
   failure class, and it survives the tool being replaced.

7. **The registry resolves outward and is checked in every direction it claims.** Check ids
   against `check-registry.yaml`, framework ids against `framework-registry.yaml` and its
   `id_pattern`s, stacks against `class-registry.yaml`, and artifact ids through the existing
   cross-reference scanner. Nothing here is asserted where it could be compared.

8. **Numbering restarted at `TST-0001` in axis order.** The taxonomy this was drafted from
   numbered its umbrella entries; removing them would have left permanent holes that read as
   deleted types. Ids are immutable and never reused from here on, exactly as artifact ids are.

## Costs accepted

1. **Fifty-eight declared types against zero implemented suites.** This is the largest gap
   between declaration and reality in the repository, and the honest reading is that it is a
   vocabulary rather than a plan. The mitigation is `status` and `trigger`, which make the gap
   legible per entry instead of leaving one number to be quoted out of context — but a reader
   who skims will still come away with the wrong impression.

2. **A sixth registry, and the cost of registries is now cumulative.** `check-registry.yaml`,
   `class-registry.yaml`, `artifact-registry.yaml`, `framework-registry.yaml`,
   `test-registry.yaml` and `knowledge.schema.json` are six files that must agree, and the gate
   is the only thing that knows they must. Each one was individually justified; nobody has
   asked whether five cross-referencing registries is the right shape, and this ADR does not
   answer that.

3. **`evidence` will be resented before it is valued.** It is the field most likely to be filled
   in with something plausible for a suite that publishes nothing, and no check can tell the
   difference until an artifact is actually looked for. It is required anyway, because the
   alternative is a registry that describes intent and cannot describe what happened.

4. **Two boundaries are drawn on judgement and may not hold.** End-to-end versus system, and
   smoke versus sanity, are pairs no reviewer reliably separates. Definitions are given rather
   than left implicit, and each entry says where it is thin — but a definition that has never
   been applied to a real suite is a guess with better formatting.

5. **Locked before a single suite exists**, which is the same bet ADR-0023 made and lost within
   a day. Accepted for the same reason: the alternative is the first service deciding its
   testing vocabulary by accident, and a vocabulary invented per repository is the thing this
   file exists to prevent.

## Alternatives rejected

**Supersede STD-0001 with one large testing standard.** Rejected on two counts. It would discard
a correct and locked artifact to add material that is not about the same subject, and it would
put the extensible list back inside a locked file — reintroducing the exact cost that motivated
this decision. STD-0001 remains authoritative for the Rust functional pyramid and this registry
cites it rather than restating it.

**Leave the taxonomy as prose in a standard.** This is what the repository already does with
directory layouts in `README.md` and `CLAUDE.md`, and it says why those two are the only places
it accepts: a generated table dropped into the middle of an argument reads worse than a stale
one. That reasoning does not transfer. A layout block is read by a person once; a test-type list
is resolved against by a gate on every run, and prose cannot be resolved against.

**One check per test type.** Rejected as the shape that would make the registry look enforced
while enforcing nothing. Most test types cannot be detected by reading a repository — no check
can tell a load test that asserts a budget from one that draws a graph — and declaring
fifty-eight checks to obtain a full column would be the ADR-0014 audit's finding reproduced
deliberately. `checks:` is empty on most entries and that is the accurate state.

**Adopt an external taxonomy wholesale.** ISTQB and IEEE 29119 both offer one, and both are
organised for a testing profession rather than for a deployment pipeline. Neither has a concept
of tier, which is the field that makes the matrix runnable, and neither would have produced the
`isolation` axis — the one most specific to this platform's largest failure class.

## Consequences

- The `isolation` axis exists, separate from `security`, and holds six types. That separation is
  the substantive architectural claim in this decision: nested tenancy (GLO-0004, ADR-0024) has
  two boundaries rather than one, and folding it into a category dominated by dependency
  advisories would bury the platform's largest failure class.
- `TST-0018 operator-scope` is the highest-value single entry. GLO-0005's rule is that operator
  scope is a grant and never an absence; the test that proves it is the negative one, and a suite
  asserting only that an operator can see everything passes identically against the dangerous
  implementation.
- Fourteen framework mappings land under SSDF `RV.*`, which the ADR-0028 mapping found completely
  empty. That gap was real and named at the time: this repository governed how code is written
  far better than what happens after a defect is found.
- STD-0008 and STD-0009 are the standards this registry made writable. Neither could have been
  written first, because both are lists of types with tiers attached.
- ADR-0030 depends on this: a rule cannot name its verification method until the methods have ids.

## Related

- [STD-0001](../standards/STD-0001-testing.md) — the Rust functional pyramid, unchanged and cited
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) — which tier runs what, and what blocks
- [STD-0009](../standards/STD-0009-tenant-isolation-testing.md) — the `isolation` axis in full
- [ADR-0030](ADR-0030-every-binding-rule-names-its-verification.md) — the other half: rules point back
- [ADR-0014](ADR-0014-declared-check-ids.md) · [ADR-0022](ADR-0022-artifact-types-are-registry-data.md) — the first and fourth instances of this pattern
- [ADR-0028](ADR-0028-controls-map-outward-to-frameworks.md) — the mapping direction this registry inherits
- [ADR-0024](ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) · [GLO-0005](../glossary/GLO-0005-operator-scope.md) — why `isolation` is an axis

## Errata

Appended under [ADR-0031](ADR-0031-errata-are-additive-and-dated.md). Additive and dated; the
text above is unchanged and is what this artifact decided. An erratum corrects a fact, never a
decision — nothing here alters what is decided above.

### 2026-08-08 — cost 2 miscounts the registries it is describing

**Says:** *"A **sixth** registry, and the cost of registries is now cumulative."*

**Wrong:** `test-registry.yaml` is the **fifth** registry, not the sixth —
`check-registry.yaml`, `class-registry.yaml`, `artifact-registry.yaml`,
`framework-registry.yaml`, `test-registry.yaml`.

**Intended:** *"A fifth registry."* The sentence that follows is correct and is where the six
came from: those five **plus `knowledge.schema.json`** are six *files* that must agree. The
paragraph's own closing line already says "five cross-referencing registries", so the artifact
contradicts itself within nine lines.

The decision is unaffected. Recorded as REQ-0002 known instance 3.

### 2026-08-08 — the `RV.*` mapping count is fourteen and should be twelve

**Says:** *"**Fourteen** framework mappings land under SSDF `RV.*`."*

**Wrong:** the figure is **twelve** — nine in `test-registry.yaml` and three in
`check-registry.yaml` — across four distinct task ids: `RV.1.1`, `RV.1.2`, `RV.3.3`, `RV.3.4`.
Counted, not estimated.

**Intended:** *"Twelve mappings land under SSDF `RV.*`, across four task ids."*

The claim the sentence supports is unaffected: `RV.*` was empty before ADR-0029 and is not
now. Recorded as REQ-0002 known instance 4.
