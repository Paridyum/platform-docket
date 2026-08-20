---
id: ADR-0035
title: A count written in an artifact is a snapshot; the registry and the gate are authoritative
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [knowledge, registries, drift, counts, errata, governance]
breaking: false
enforced_by:
  - check: knowledge.hardcoded_count_baseline_does_not_rise
    severity: error
    args: { baseline: 48 }
    rationale: >
      Decision 5 says new artifacts do not state a count a registry can compute, and until now
      nothing could tell a new one from the amnestied ones — the warn below always fires, so
      two counts added on 2026-08-10 moved it from 48 to 50 invisibly, and one of them
      contradicted a sentence two lines away. This adds no judgement about any individual
      line, only that the total must not rise, which is the one thing here that is decidable.
  - check: knowledge.no_hardcoded_registry_counts
    severity: warn
    rationale: >
      `warn` and not `error`, because the check cannot tell a count from a quantity: "two
      isolation boundaries" is a structural fact and "twenty-two locked artifacts" is a
      measurement, and both are a number next to a noun. An error-severity check on an
      undecidable distinction is one people route around, which is worse than a list nobody
      is forced to clear.
---

# ADR-0035 — Counts in prose are snapshots, not claims

## Status

**ACCEPTED** as of 2026-08-08.

## Context

This repository has now been caught by a hand-maintained number at least seven times, and the
seventh was found by an audit rather than by any gate:

| Artifact | Says | Actually |
| --- | --- | --- |
| ADR-0030 (locked) | "Fifty-nine checks still await `tool-forge`" | 71 |
| ADR-0030 (locked) | "all twenty-two of them unlockable" | 43 |
| ADR-0015 (locked) | "Four artifacts are scoped `[org]`" · "The eight artifacts…" | 44 · 9 |
| ADR-0022 (locked) | "Nine types are declared" | 10 — **and it was 10 at its own lock commit** |
| ADR-0014 (locked) | "100 distinct check ids across 26 namespaces, cited by 23 artifacts" | 160 · 29 · 56 |
| test-registry.yaml | "not a plan to build 56 test suites" | 60 |
| CLAUDE.md | "six forbidden shapes" | 8 |
| check-registry.yaml | "`owner` … Five values today" | 8, and the same comment lists 7 |

The pattern is not carelessness. Every one of these was true when written. The repository
already knows this — `check-registry.yaml`'s own header records that its per-owner counts *"were
wrong when measured… Nobody had introduced an error — the counts simply were not maintained,
which is the third time this repository has been caught by a hand-maintained number and the
reason README.md now refuses to state one."*

That last clause is the whole decision. **README solved this for itself and nobody generalised
it.** Meanwhile ADR-0022 — the artifact whose entire argument is that a list which must be
extended should not be hardcoded — hardcoded a count of the list it was creating, and froze it
wrong on the same day.

The pressure the other way is real and worth stating. A count is how an argument earns its
weight: *"fifty-nine checks await tool-forge"* is what makes the backlog feel like a problem,
and *"four artifacts are scoped `[org]`"* is the evidence for a deduction. Removing numbers
makes artifacts weaker to read. That cost is accepted below rather than denied.

## Decision

**1. A count in an artifact is a measurement taken on its `created` date, not a claim about the
present.** It carries no more authority than any other observation of a moment.

**2. Where an artifact and a registry disagree about a count, the registry wins — always, with
no erratum required and no supersession.** The artifact is not wrong; it is old.

**3. The gate prints the authoritative numbers on every run.** `scripts/check-knowledge.sh` is
the only source that is true when you read it, because it counts at the moment of counting.

**4. Existing stale counts are therefore not defects.** They do not require errata under
ADR-0031, and the ten-odd locked artifacts carrying them are not in violation of anything. This
clause exists to prevent the obvious and wrong response to the table above: ten errata, correcting
numbers that will be stale again within a month.

**5. New artifacts do not state a count a registry can compute.** Cite the registry, or cite the
gate, or write the shape of the thing without its cardinality. `knowledge.no_hardcoded_registry_counts`
lists candidates at `warn`.

**6. A count used as *evidence* is different, and must be re-derived before the conclusion
resting on it is relied upon.** ADR-0015's scope deduction is the live example: its numbers are
wrong by an order of magnitude, and whether its conclusion survives is a question, not an
assumption. Decision 4 forgives the number. It does not forgive an argument nobody has rechecked.

## Consequences

**Accepted costs**

1. **Artifacts get harder to read and weaker to argue with.** "The backlog is large" is a worse
   sentence than "fifty-nine checks await `tool-forge`". This is the real price and it is paid on
   every future artifact, in exchange for not paying a correction cost on every past one.

2. **The gate becomes load-bearing for comprehension, not just for enforcement.** A reader who
   does not run it now has strictly less information than one who does. That is a change in how
   the repository is meant to be used, and it is not written down anywhere else.

3. **Decision 4 will read as an amnesty.** It is one. The alternative is a class of defect that
   regenerates faster than it can be corrected, which is how a backlog becomes a fiction.

4. **The check cannot be precise.** Six anti-patterns is a count; two isolation boundaries is a
   structural fact; three enforcement layers is arguably both. `warn` is the honest severity and
   the list will always need a human.

**Gained**

- Roughly twenty findings across ten locked artifacts stop being defects, without a single
  erratum and without touching an immutable file.
- The failure class stops recurring, rather than being corrected for the eighth time.
- It generalises a pattern this repository already proved on itself in README.md.

**Rejected alternatives**

- **Write ten errata.** The mechanical response, and it is what ADR-0031 exists for. Rejected on
  arithmetic: ten artifacts of work to correct numbers whose half-life is demonstrably under a
  month, leaving the class fully intact for the next artifact written.

- **A check that validates stated counts against the registry.** Attractive and undecidable. It
  would need to know *what* each number counts — "six forbidden shapes" is `ls anti-patterns/`,
  "two isolation boundaries" is not countable from any file, and "three enforcement layers" is a
  concept. A check that guesses would produce confident false failures on exactly the artifacts
  that matter.

- **Forbid numbers outright.** Rejected because decision 6's case is real: sometimes the number
  *is* the argument, and banning it would push the reasoning out of the artifact rather than
  keeping it honest.

- **Do nothing.** The status quo, and it produced a locked constitution-adjacent ADR whose central
  deduction rests on "four" where the answer is 44.

## Related

- ADR-0031 — errata, which decision 4 deliberately declines to use
- ADR-0014 — the check registry, and the precedent of moving a list out of prose
- ADR-0022 — artifact types as registry data; its own count froze wrong on its lock day
- ADR-0015 — the live instance of decision 6: an argument whose evidence has drifted
- C-0004 — generated output is regenerated, never hand-edited. A count is the same idea,
  one step earlier: a number that must be maintained by hand is output pretending to be source
