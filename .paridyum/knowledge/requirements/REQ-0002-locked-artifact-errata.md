---
id: REQ-0002
title: A factual error in a locked artifact can be corrected without superseding it
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, immutability, errata, archive-integrity]
breaking: false
trigger: >
  FIRED 2026-08-08 on the second clause. A fifth factual defect is found in a locked
  artifact, or one of the known defects actively misleads someone — an agent or a human
  acts on the wrong text.
review_date: 2027-02-01
verified_by: []
enforced_by:
  - check: knowledge.errata_are_additive_only
    severity: error
    rationale: >
      Criterion 4 of this requirement: a mechanism whose limits are described rather than
      checked is a convention, and a convention that weakens immutability will be abused.
---

# REQ-0002 — A factual error in a locked artifact can be corrected without superseding it

## Status

**ACCEPTED** as of 2026-08-08, resolved by [ADR-0031](../adr/ADR-0031-errata-are-additive-and-dated.md).

**DEFERRED** 2026-08-07 with two known instances, on the judgement that two is not enough to
design the mechanism well. The **second** trigger clause fired the next day rather than the
count: instances 3 and 4 are in ADR-0029's cost section and consequences, which is where a
reader quotes a number without re-deriving it. Two of the five acceptance criteria below are
now checked mechanically; criteria 2, 3 and 5 are prose that ADR-0031 §§3–5 own.

The judgement that deferred it was not wrong. Waiting produced two more instances, and those
two are what made the shape of the mechanism obvious — in particular that the boundary has to
be a *section* rather than a *field*, which a design built on the original two frontmatter
defects would have got backwards.

## Requirement

**A locked artifact containing a factual error in its text — as distinct from a decision
that has changed — shall be correctable by an additive, dated, attributable annotation
that leaves the original text intact.**

Not a licence to edit. The requirement is for a correction path whose cost is
proportionate to a typo, because the only path today costs the same as reversing a
decision.

## The problem

Locked artifacts are immutable, and that is the point: ADR-0002 makes immutability the
mechanism by which a decision stays retrievable as it stood when the code under it was
written. `knowledge.locked_artifacts_immutable` and the PreToolUse guard both enforce it,
and as of 2026-08-07 they also cover `superseded`, so the archive is properly frozen.

The gap is that immutability makes no distinction between *changing a decision* and
*fixing a typo*. Both require a full supersession, and supersession here is
whole-artifact — the replacement must restate everything the original said. Rewriting an
entire testing standard because of one stray `#` is disproportionate enough that it will
never actually happen, which means in practice the errors are permanent.

So the two available options collapse into one: **do nothing.** That is the real reason
this is a requirement rather than a preference.

## Known instances

**1. STD-0001, `test.no_ignored_tests_without_reason` rationale.** The line reads:

```yaml
rationale: An #[ignore] with no reason is a permanently disabled test nobody will revisit.
```

The unquoted `#` starts a YAML comment, so the parsed value is the string `"An"`. A
reader of the raw file sees the intent; every tool that parses the frontmatter sees `An`.
**Annotated in STD-0001 itself on 2026-08-08** under ADR-0031, and still wrong: frontmatter sits
above the errata section, so the annotation makes the defect visible and cannot repair the parsed
value. That is ADR-0031 cost 4, and it is the half of this requirement that remains open.

**2. ADR-0014, "The arithmetic".** States *"`tool-forge` does not exist, so **0 are
implemented**."* True of tool-forge's registry, false of enforcement: 35 checks were
already implemented in bash, hooks and CI when it was written, and the audit that
established this was what exposed the claim. ADR-0014's argument is unaffected — no id
resolved against any registry, so the gate still could not have a first green run — but
the number misleads. **Annotated in ADR-0014 itself on 2026-08-08** under ADR-0031; prose, so the
erratum corrects it fully.

**3. ADR-0029, cost 2.** Opens *"A **sixth** registry, and the cost of registries is now
cumulative."* It is the **fifth** — `check-registry.yaml`, `class-registry.yaml`,
`artifact-registry.yaml`, `framework-registry.yaml`, `test-registry.yaml`. The sentence that
follows is correct and is what the count was taken from: those five *plus*
`knowledge.schema.json` are six **files** that must agree. The paragraph's own closing line
already says "five cross-referencing registries", so the artifact contradicts itself in nine
lines. Found on 2026-08-08, minutes after locking, by counting the list it names.

**4. ADR-0029, consequences.** States *"**Fourteen** framework mappings land under SSDF
`RV.*`."* The actual figure is **twelve**, across four distinct task ids — `RV.1.1`, `RV.1.2`,
`RV.3.3` and `RV.3.4` — nine of them in `test-registry.yaml` and three in
`check-registry.yaml`. Counted, not estimated, on 2026-08-08. The claim it supports is
unaffected: `RV.*` was empty before and is not now.

None of the four is a wrong decision. All four are wrong text, and **all four are now annotated
in the artifact that carries them** — which is criterion 5, and which none of them satisfied for
the first day of their lives.

**Instances 3 and 4 are the first ones this repository could have avoided and did not**, and
how they arrived is the useful part. Both were written into an artifact that was `locked` at
creation, and the ADR-0025 checkpoint hook committed and pushed it **mid-authoring** — so the
artifact was frozen before its author had finished counting. The corrections were written,
rejected by `knowledge.locked_artifacts_immutable`, and reverted. That is the control working
exactly as designed, and it is also the clearest evidence yet that the correction path costs
more than the defect: superseding a 150-line ADR to change "sixth" to "fifth" will not happen,
so the wrong words stay.

There is a cheaper fix than the mechanism this requirement asks for, and it is worth stating
because it needs no new machinery: **do not create an artifact as `locked`.** Create it
`accepted`, and lock it in a later, deliberate commit once the counts have been checked.
`status` is one of the three fields section 3 already permits to move, so the transition is
legal by construction, and locking stops being something that happens to a draft by accident
of a durability hook.

## Acceptance criteria

1. The original text of a locked artifact is never altered — an erratum is additive only.
2. An erratum carries a date and states what is wrong and what was intended.
3. The mechanism cannot be used to change a decision. Anything that alters what the
   artifact *decides* still requires supersession.
4. A check enforces (1) and (3), or the mechanism is a convention and will be abused.
5. Errata are discoverable from the artifact itself, not only from a separate index — a
   reader of STD-0001 must see that line 24 is corrected.

## Out of scope

- Changing a decision. That is supersession and ADR-0002 already governs it.
- The two known instances specifically. They are the evidence, not the deliverable; a
  mechanism designed around two examples will fit two examples.

## Open questions — all four answered by ADR-0031

1. **Append-only `## Errata` section, or a separate file?** In-artifact satisfies
   criterion 5 directly but means the immutability check must permit additions to exactly
   one section — a narrow hole in a control whose value is that it has none.
2. How does a check distinguish "added an erratum" from "edited the body and added an
   erratum to cover it"? Probably: every line outside the errata section must be
   byte-identical to the committed version, which is checkable.
3. Should an erratum be permitted on a `superseded` artifact? Argument for: that is where
   errors are most likely to be found and least likely to matter. Against: a superseded
   artifact is pure history and should be entirely frozen.
4. Is there an alternative that avoids the hole altogether — for example, corrections
   living only in `check-registry.yaml` and a generated index, with the artifact
   untouched? That is what happens today, and it fails criterion 5.

## How they were answered

1. **In-artifact `## Errata` section** (ADR-0031 §1). The separate-file option was rejected on
   criterion 5, with evidence rather than argument: instance 1 has been recorded in
   `check-registry.yaml` since the day it was found, and no reader of STD-0001 has ever seen it.
2. **Exactly as this question guessed** (ADR-0031 §2, `knowledge.errata_are_additive_only`).
   Everything above the first `## Errata` heading must be byte-identical to the committed
   version. The guess turned out to also solve the case it did not ask about: a heading inserted
   mid-file truncates the body under the same split, so the smuggling case rejects itself with no
   rule of its own.
3. **Permitted on `superseded` too** (ADR-0031 §7). The deciding argument was not in either side
   as originally stated: a superseded artifact is read by someone doing archaeology, who is the
   reader least able to check a number independently — and because errata are additive and dated,
   nothing about the historical record is destroyed by permitting them.
4. **No.** Corrections living only in an index is what already happens and it fails criterion 5.

## What remains unsolved

**Frontmatter.** Instance 1 is a corrupted YAML value, which sits above any errata section, so
ADR-0031 can annotate it and cannot correct the parsed string. This requirement's own suspicion —
that "the real requirement may be narrower than errata and actually about frontmatter validation"
— is half confirmed: the prose half is solved and the frontmatter half is not. It is not
re-deferred as a new requirement because there is exactly one instance, which is where this
document started.

## Related

- ADR-0002 — immutability is the mechanism; superseded by ADR-0014 but the argument stands
- ADR-0014 — carries instance 2, and its known-limitation section first named this gap
- ADR-0015 — implemented `knowledge.supersession_is_symmetric`, the third gap named
  alongside these two
- STD-0001 — carries instance 1
- ADR-0018 — established the requirement type this artifact uses
