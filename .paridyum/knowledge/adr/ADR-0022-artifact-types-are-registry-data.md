---
id: ADR-0022
title: An artifact type is registry data, and a bet may never be locked
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, artifact-types, registry, strategy, bets]
breaking: true
enforced_by:
  - check: knowledge.artifact_registry_valid
    severity: error
    rationale: >
      The registry claims to generate the schema's type enum and id patterns. A
      "generates" claim nothing verifies is the defect this repository keeps finding.
  - check: knowledge.resolved_by_names_live_decision
    severity: warn
    rationale: >
      A resolved_by pointing at a superseded decision still resolves, so nothing complains
      while a reader follows it to a rule that was deliberately replaced.
  - check: knowledge.review_dates_not_overdue
    severity: warn
    rationale: >
      Requiring a review_date and never reading it leaves a wager whose verdict was due
      months ago sitting green — the failure the field exists to prevent.
  - check: knowledge.bet_states_its_falsification
    severity: error
    rationale: >
      A wager with no stated losing condition is an opinion with frontmatter, and a
      locked bet is a strategic guess made immune to evidence at the highest precedence
      level in the repository.
---

# ADR-0022 — An artifact type is registry data, and a bet may never be locked

## Status

**LOCKED** as of 2026-08-07, on the condition this document set for itself.

It landed as `accepted` rather than `locked`, deliberately: the mechanism had generated exactly
one type, and *"locking early would freeze a shape whose second use has not tested it."* The
second use has now happened. `glossary` was added as an artifact type by editing
`artifact-registry.yaml` and `knowledge.schema.json` and nothing else — and section 8c
immediately failed on the three sites that still carried the old prefix set, which is precisely
the silent failure this ADR exists to convert into a loud one. The mechanism was exercised, it
caught real drift on its first unassisted run, and the condition is met.

The checks always ran. Severity does not depend on status.

## Context

RFC-0005 asked where commercial strategy lives. Answering it required a new artifact type,
and attempting to add one exposed a larger problem: **the artifact type list was hardcoded
at fifteen sites, fourteen of which fail silently when one is missed.**

The worst is `ART_DIRS` in `check-knowledge.sh`. A directory absent from that array is not
scanned — no error, no warning, an entire artifact class unenforced while the gate prints
`Knowledge check passed`. Adding `bets/` to some of the fifteen sites would have produced a
governance directory that validated nothing and looked fine.

**The count was twelve until three more sites were found, and two of those three had already
drifted.** One was a hole in ADR-0007's layer 1 — the layer CLAUDE.md describes as *"a hook
that denies a tool call cannot be reasoned around"*:

- **`guard-locked-decisions.sh` omitted `requirements/`.** Section 3 of `check-knowledge.sh`
  has protected that directory since REQ- existed, so layers 1 and 2 disagreed about which
  artifacts are immutable. A locked REQ- could be rewritten in place, in-session, and the
  guard permitted it. Reproduced against the pre-fix hook with synthetic tool-call JSON,
  then fixed and re-verified.
- **`capture-open-loops.sh` carried the old prefix set**, so a dangling reference to a new
  type would never have reached the ADR-0011 ledger — the mechanism whose whole purpose is
  that a forgotten loop is put back in front of the next person.
- **`check-registry.schema.json`'s `blocks` pattern** would have rejected any check
  declaring that it blocks an artifact of a new type.

An undercount discovered while arguing that the list is too scattered to count is the
argument, so it is recorded rather than quietly corrected.

This is the fourth instance of one pattern:

| Artifact | Moved out of code | Into |
| -------- | ----------------- | ---- |
| ADR-0014 | check ids | `check-registry.yaml` |
| ADR-0015 | scope resolution | a rule, not an enumeration |
| C-0005 | repository classes | `class-registry.yaml` |
| **ADR-0022** | **artifact types** | **`artifact-registry.yaml`** |

Two latent defects surfaced while writing RFC-0005, both present for the life of the
repository. The id pattern in `knowledge.schema.json` is copied at four sites and nothing
required them to agree — a prefix valid in `id` but absent from `supersedes` produces an
artifact that validates and can never be superseded. And `knowledge.frontmatter_valid`'s
spec enumerated seven directories while the gate scanned eight, omitting `requirements/`.

## Decision

**The set of artifact types is data in `artifact-registry.yaml`, and it generates the type
enum, the id-prefix set, the scanned-directory list, the immutability filter and the
per-type status vocabulary. A type declaring `lockable: false` may never take
`status: locked`.**

Three axes, because conflating them is what made the old rules special cases:

- **`lockable`** — may this type ever be `locked`? False for `rfc` and `bet`.
- **`deferrable`** — may it take the requirement-only `deferred` status? True for
  `requirement` alone. This replaces the gate's hardcoded `type == requirement`.
- **`requires_trigger`** — `never`, `when_deferred`, or `always`. ADR-0018's rule for
  requirements and this ADR's rule for bets are one axis at two settings, not two rules.

### `bet` — the type RFC-0005 needed

A falsifiable commercial wager. `requires_trigger: always`, so `trigger` and `review_date`
are both mandatory: the evidence that would kill it, and the date it gets judged anyway.

**`lockable: false` is the load-bearing value.** CLAUDE.md defines `locked` as *"must not be
contradicted — not by you, not by a human, not temporarily"*. A bet exists precisely to be
contradicted, by evidence, on a date already written down. Permitting a locked bet would
make `locked` mean two incompatible things — the same argument C-0001 uses to stay
`accepted` — and would let a market guess sit in the highest-precedence half of the
knowledge base, immune to evidence, while CLAUDE.md instructs agents to retrieve it rather
than infer. That is the most expensive thing this type could accidentally permit, which is
why it is a check and not a convention.

A bet ends by being **superseded**, and section 3 freezes superseded artifacts as hard as
locked ones. A killed bet is the most valuable kind, and the easiest to quietly rewrite into
one that was never wrong.

## Costs accepted

1. **Fifteen sites became two unchecked ones, not zero.** Ten are generated from the
   registry. Three stay hardcoded for stated reasons and are *compared* by section 8c — the
   two hooks must remain fast and dependency-free, because a YAML parse on every tool call
   is a hook people disable, and `check-registry.schema.json` is not generated by this gate.
   That leaves `README.md` and `CLAUDE.md`, which describe the layout in prose and still
   drift silently. Generating them was considered and rejected: they are written for a human
   reading top to bottom, and a generated block dropped into the middle of an argument reads
   worse than a stale one. Those two are now the only places this list can rot unobserved,
   and that is a deliberate choice rather than an oversight.

2. **The duplication in `knowledge.schema.json` is unavoidable, not merely tolerated.**
   `scripts/lib/validate-schema.py` implements neither `$defs` nor `$ref`, and its
   `unsupported()` reporter skips keys beginning with `$` — so a `$ref` would be silently
   ignored and the field would carry **no constraint at all**. That is strictly worse than
   four copies, by the validator's own docstring. Four copies plus a check that they agree
   is the best shape available until the validator grows `$ref`.

3. **The registry reproduces two asymmetries rather than fixing them.**
   `immutable_when_locked` is false for `rfc` and `runbook` because section 3's filter has
   always omitted those directories. A locked runbook is as much a historical record as a
   locked standard, so this is probably wrong — but changing behaviour while relocating it
   is how detail gets lost, which is the lesson `class-registry.yaml` records about
   ADR-0010's `ci: false` for infra. One line here, and a separate decision.

4. **The cross-reference scanner still reads only `*.md` and `*.sh`.** Ids in
   `check-registry.yaml`'s `blocks` lists and in `class-registry.yaml`'s prose are never
   resolved, so a typo there is invisible. Widening the include list is a behaviour change
   with its own blast radius and was deliberately not bundled here.

5. **`bet` adds a governance surface that cannot be mechanically judged.** The gate can
   force a bet to state its falsification condition. Nothing can check whether the condition
   is a good one, or whether anyone looked on the review date. This buys structure, not
   judgement, and mistaking the first for the second is the risk the type introduces.

## Alternatives rejected

**Hand-edit twelve sites.** Cheapest today, and it re-arms every silent failure for the
next type. The three prior instances of this pattern were all fixed the same way; there is
no argument for a fourth exception.

**Tag an ADR `strategy` instead of adding a type.** No place for `trigger`, and `lockable`
could not be varied per type — so a strategic wager could be locked. The check that matters
most would have been unexpressible.

**A `bet` status rather than a type.** Statuses describe where an artifact is in its
lifecycle; this describes what kind of claim it makes. It would also have needed the same
per-type table to know which types may use it, arriving at this design one indirection
later.

## Consequences

- `bets/` joins the scanned directories. Nine types are declared; `pattern` and `bet` had no
  populated directory when the registry landed.
- New checks: `knowledge.artifact_registry_valid`, `knowledge.bet_states_its_falsification`.
  Both are implemented in `check-knowledge.sh`, both were verified by planting the defect
  they should catch, and one of them was wrong on the first attempt — it counted distinct id
  patterns rather than occurrences, so it reported success when all four agreed and would
  have reported success just as happily if three had been deleted. Found by running it.
- `knowledge.frontmatter_valid`'s spec no longer enumerates directories.
- ADR-0007's layer 1 now guards `requirements/` and `bets/`, closing a hole in which a
  locked requirement could be edited in place in-session while section 3 forbade it. Section
  8c fails if the guard and section 3 ever disagree again.
- `capture-open-loops.sh` and `check-registry.schema.json` carry the current prefix set, and
  section 8c compares both against the registry.
- The per-owner and declared/implemented counts were removed from `check-registry.yaml`'s
  header. They were measured while adding these entries and were wrong — 11 and 3 against
  actual 14 and 4, totals of 111/36 against 113/40. Nobody had introduced an error; the
  numbers simply were not maintained. Third time, same lesson README.md already records.

## Related

- RFC-0005 — the question, and the twelve-site table
- ADR-0014, ADR-0015, C-0005 — the three prior instances of list-in-code
- ADR-0018 — the `trigger` argument, reused at `requires_trigger: always`
- ADR-0002 — why `locked` is reserved for what a check can reject
- C-0006, BET-0001, BET-0002, BET-0003 — the first artifacts this made possible
