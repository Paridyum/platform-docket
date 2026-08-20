---
id: ADR-0031
title: A factual error in a locked artifact is corrected by an appended, dated erratum
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, immutability, errata, archive-integrity, adr-0002]
breaking: false
verified_by: []
enforced_by:
  - check: knowledge.errata_are_additive_only
    severity: error
    rationale: >
      This decision opens the only hole that has ever existed in the immutability control,
      so the hole must be narrower than the sentence describing it. Every line above the
      errata heading byte-identical, existing errata a strict prefix of the new ones, and
      the section last in the file — three conditions, all mechanical, none requiring a
      reviewer to notice that a body edit was smuggled in under an erratum.
---

# ADR-0031 — A factual error in a locked artifact is corrected by an appended, dated erratum

## Status

**ACCEPTED** as of 2026-08-08. Deliberately not `locked`, and §6 is why.

Resolves [REQ-0002](../requirements/REQ-0002-locked-artifact-errata.md), whose second trigger
clause — *"one of the known defects actively misleads someone"* — fired on 2026-08-08.

## Context

REQ-0002 was recorded on 2026-08-07 with two known instances and deferred on the judgement
that two is not enough signal to design a mechanism well. It named the trigger as a fifth
instance **or** a defect that actively misleads. Instances three and four arrived the next
day, in ADR-0029, and the second clause is what fired rather than the count.

The defect is worth stating precisely, because it is the argument. ADR-0029 establishes the
registry pattern and, in its own cost section, miscounts the registries it is describing:
*"a **sixth** registry"* where the list it then prints has five entries, and the paragraph's
own closing line says "five cross-referencing registries". A reader who trusts the number and
skips the list is misled by the artifact whose subject is that list. Alongside it, a
consequence claims *"**Fourteen** framework mappings land under SSDF `RV.*`"* where the actual
figure is twelve.

Neither is a wrong decision. Both are wrong text, in the two places a reader is most likely to
quote without re-deriving — which is what "actively misleads" means.

**How they got there is the load-bearing part of this context.** Both were written into an
artifact created with `status: locked`, and the ADR-0025 checkpoint hook committed and pushed
it mid-authoring. The artifact was frozen before its author had finished counting. The
corrections were then written, rejected by `knowledge.locked_artifacts_immutable`, and
reverted — the control working exactly as designed, and simultaneously the clearest available
demonstration of REQ-0002's complaint: the only correction path costs a whole-artifact
supersession, which for two words will never be paid, so the wrong text stays.

That is the real failure mode. Not that errors happen, but that the cost of fixing one is set
by the cost of reversing a decision, so the rational choice is always to leave it.

## Decision

**A locked or superseded artifact may carry a trailing `## Errata` section. Entries are
appended, dated and never removed; every line above the heading stays byte-identical.**

1. **In the artifact, not in an index.** REQ-0002 criterion 5: a reader of ADR-0029 must see
   that its cost 2 is corrected, without knowing an index exists. Today's practice — recording
   corrections in `check-registry.yaml` notes — fails that, and it is why instance 1 has been
   invisible to every reader of STD-0001 since it was written.

2. **`## Errata` is the last section of the file, and appears once.** Not a placement
   preference: it is what makes the check expressible. Everything above the first `## Errata`
   heading is the artifact; everything below is annotation. A heading inserted mid-file
   truncates the body under that rule and fails, which is the smuggling case rejecting itself.

3. **An entry states the date, what the text says, what is wrong, and what was intended.**
   In that order. "Corrected a typo" is not an erratum — the reader has to be able to decide
   whether the original text misled *them*, which needs both versions.

4. **Errata are append-only.** Existing entries are as immutable as the body. An erratum that
   can be rewritten is a second editable surface with a different name, and the archive would
   be back where it started one level down.

5. **An erratum may never change what the artifact decides.** No check can judge that, and
   this decision does not pretend one can. What is mechanical is that the original text is
   untouched — so a reader can always see the decision as it stood, and an erratum claiming to
   change it is visibly contradicting the text above it. Anything that alters the decision is
   still supersession, and ADR-0002 still governs.

6. **`accepted`, not `locked`, and this is a decision rather than an omission.** REQ-0002's
   closing argument: *"Every new mechanism written during this session had a defect found in it
   within the hour. A back door into the immutability control is the last place to accept that
   hit rate."* That argument did not stop being true because the mechanism became necessary.
   Locking this the hour it was written would also repeat the exact mistake that produced the
   instances it exists to fix. It locks after it has survived contact with a correction nobody
   wrote it for.

7. **Permitted on `superseded` artifacts too.** REQ-0002 open question 3 gave both sides. The
   deciding argument is that a superseded artifact is where errors are most likely to be found,
   by the person doing archaeology, who is precisely the reader least able to check a number
   independently. Because the mechanism is additive and dated, nothing about the historical
   record is destroyed by permitting it — a reader can always separate what was written then
   from what was appended later, which is the property "entirely frozen" was protecting.

## Costs accepted

1. **This is a hole in the strongest control in the repository, and it is the first one.**
   `knowledge.locked_artifacts_immutable` had no exceptions but the three supersession fields.
   It now has a second, larger one whose boundary is a heading in a markdown file. The
   mitigation is that the boundary is checked in three ways rather than described in one, but
   the honest statement is that the control is weaker than it was yesterday.

2. **The guard is weaker than the gate, deliberately, and they can drift.** The `PreToolUse`
   hook must stay fast and dependency-free, so it permits an edit that is strictly additive at
   its anchor and touches an errata section — it cannot diff against the committed version the
   way the gate does. Layer 1 therefore accepts a narrow class of edit that layer 3 will
   reject. That is fail-safe in direction and is still a disagreement between two layers, which
   is the shape of two real holes this repository has already found.

3. **An erratum is a place to put an argument.** Nothing stops a long enough erratum from
   functioning as an amendment while satisfying every check. §5 says it must not, and §5 is
   prose. The only defence is that the original text sits directly above it, unaltered.

4. **Instance 1 stays wrong.** STD-0001's corrupted rationale is in **frontmatter**, above any
   errata section, so this mechanism cannot correct the parsed value — only annotate it. REQ-0002
   suspected as much: *"both known defects are in frontmatter rather than prose, which may mean
   the real requirement is narrower than errata."* Half of that suspicion is confirmed here, and
   frontmatter errata remain unsolved.

## Alternatives rejected

**Supersede ADR-0029 with a new ADR.** The only path available yesterday. A 190-line
near-duplicate whose entire delta is two words, which is exactly the disproportion REQ-0002
was written to complain about — and the record would then carry two nearly identical ADRs,
making the archive harder to read rather than more correct.

**Rewrite the commits that locked ADR-0029.** Attractive, because the artifact was frozen
mid-authoring by a durability hook rather than by any deliberate act, and had never been read
by anyone. Rejected on two independent grounds: the commits are already pushed, so this is a
C-0004 protected operation; and they carry an unrelated artifact being authored concurrently
in another session, which a force-push would put at risk. The second reason would have been
sufficient alone, and it is a direct consequence of the ADR-0025 hook's `git add -A`.

**Corrections in a separate errata file or in `check-registry.yaml`.** What happens today, and
it fails REQ-0002 criterion 5 — instance 1 has been recorded in `check-registry.yaml` since it
was found and no reader of STD-0001 has ever seen it. An index nobody knows to consult is a
correction nobody receives.

**Permit editing a locked artifact within some window after locking.** A time-boxed hole is
worse than a shaped one: it is invisible in the diff, it cannot be checked against anything a
reader can see, and it rewards locking early. It also converts "locked" into "locked after a
while", which is the word meaning two things — the objection ADR-0022 raised against making a
bet lockable.

## Consequences

- ADR-0029 carries the first `## Errata` section, correcting cost 2 and one consequence.
- REQ-0002 moves from `deferred` to `accepted` and names this ADR. Its open questions 1, 2 and 3
  are answered by §§1–2, the check, and §7 respectively; question 4 is answered by rejecting the
  index approach above.
- `knowledge.locked_artifacts_immutable` keeps its spec and gains a sibling rather than being
  rewritten. It is `owner: check-knowledge.sh` and cited by an immutable artifact, so its spec
  could not have been narrowed in place regardless.
- The `new-decision` skill gains the correction path, because that is where an agent looks when
  it discovers a locked artifact is wrong and currently finds only "supersede it".
- This decision does not make an artifact's *text* less trustworthy. It makes the difference
  between the text and the correction visible, which is the opposite.

## Related

- [REQ-0002](../requirements/REQ-0002-locked-artifact-errata.md) — the requirement this resolves, and its four known instances
- [ADR-0002](ADR-0002-ci-gate-is-the-lock.md) — immutability as the mechanism; superseded by ADR-0014, argument intact
- [ADR-0014](ADR-0014-declared-check-ids.md) — carries instance 2, and first named this gap as a known limitation
- [ADR-0029](ADR-0029-test-types-are-registry-data.md) — carries instances 3 and 4, and the first errata section
- [ADR-0025](ADR-0025-session-work-is-checkpointed-automatically.md) — the hook that froze ADR-0029 mid-authoring, and whose `git add -A` also blocked the rewrite alternative
- [ADR-0007](ADR-0007-agent-harness-distribution.md) — the enforcement ladder cost 2 describes a disagreement inside
- [STD-0001](../standards/STD-0001-testing.md) — carries instance 1, which cost 4 admits this cannot fix
