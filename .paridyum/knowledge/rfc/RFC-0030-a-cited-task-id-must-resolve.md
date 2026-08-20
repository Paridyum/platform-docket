---
id: RFC-0030
title: A task id cited by a governed artifact must resolve, and two files must not disagree about it
type: rfc
status: proposed
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [tasks, roadmap, cross-reference, dangling-references, adr-0022, adr-0014, c-0007, governance]
breaking: false
enforced_by: []
---

# RFC-0030 — A cited task id must resolve

## Problem

**`ROADMAP.md` and `TASKS.md` carry the gates that block every phase of this company, and
nothing verifies a single claim in either file.**

Observable, on 2026-08-19:

- The `T-` id namespace spans Phase 0 through Phase 3 in `TASKS.md`, and roughly a third of
  those ids are also cited in `ROADMAP.md` as the content of phase gates.
- `T-` ids are cited from **governed artifacts** — `ADR-0048`, `RFC-0013`, `RFC-0028` and
  `BET-0005` each name one.
- Neither `ROADMAP.md` nor `TASKS.md` is referenced anywhere in `check-registry.yaml`.
  `SECURITY.md` is in the same position; `CLAUDE.md`, `README.md`, `TESTING.md`,
  `DECISION-DEBT.md` and `CHANGELOG.md` are all referenced.
- The `T-` prefix is not in `artifact-registry.yaml`, so the gate's cross-reference scanner
  does not resolve it. `ID_PREFIXES` is built from the registry; a prefix absent from it is
  invisible rather than rejected.

**Three defects follow, and none of them can be detected today:**

1. **A dangling citation.** `BET-0005` may cite a `T-` id that `TASKS.md` no longer contains.
   Nothing resolves the reference. This is the defect the gate already catches for every
   registered prefix and cannot catch for this one.
2. **Two files disagreeing.** A task struck through as done in `TASKS.md` while `ROADMAP.md`
   still marks it `⛔` blocking a phase — or the reverse. Both files are hand-maintained, both
   are long, and they are edited at different times for different reasons.
3. **A silently dropped gate.** Phase 0's exit gate is written as prose naming tasks. A task
   removed from `TASKS.md` leaves the gate reading as though it were still covered.

**Who experiences it:** anyone reading a phase gate to decide whether work may start. Phase 0's
exit gate blocks the first runtime commit, and it is expressed *entirely* in ids that resolve to
nothing checkable.

**What happens if nothing is done:** the failure is not that the files rot — they are well
maintained and the prose in them is better than most artifacts here. It is that **their
correctness rests on attention, and this repository exists because attention was judged an
insufficient control.** `README.md` records the same lesson twice from the other direction: a
hand-maintained table went stale within a week, its replacement within a day.

**How we will know it is fixed:** removing a cited `T-` id from `TASKS.md`, or striking one
through while `ROADMAP.md` still blocks on it, fails the gate.

## Constraints

**`T-0.1` already decided that this prose lives ungated, and decided it deliberately.** Its
closing note is explicit — the objection that moving these files "puts them under the gate" was
examined and found false, because *"`check-knowledge.sh` scans only the directories
`artifact-registry.yaml` names."* **This RFC must therefore argue against a considered position
rather than correct an oversight**, and it does not dispute that conclusion: this proposal does
not put `ROADMAP.md` or `TASKS.md` under the artifact gate, and does not make them artifacts.
It adds cross-reference resolution over ids they already publish. That distinction is the whole
design.

**`ADR-0022` and `C-0007` — the grammar is constitutional, the list is data.** Whatever holds the
task list must be extendable without editing code, and the id prefix must be resolvable from
declared data rather than a hardcoded alternation. The prior instance of a hardcoded alternation
here held directories against the registry's larger set, so several artifact classes were never
scanned while everything read as configured.

**`ADR-0014` and `ADR-0002` — a check id is declared before it is implemented**, with a
falsifiable spec, and `locked` requires something that rejects the violation.

**`ADR-0035` — counts in prose are snapshots.** Any generated or compared block introduced here
must not add hardcoded cardinalities.

**`artifact-registry.yaml` is the sole source of the scanned directory set, the id pattern, the
cross-reference prefix set and the immutability set.** Three independent consumers already
assert agreement with it: the `PreToolUse` guard, the open-loop ledger's `ART_DIRS`, and
`knowledge.schema.json`. **Anything adding a prefix must satisfy all three or it will desynchronise
them**, which is the failure mode this registry was created to end.

## Proposal

**Do not make a task an artifact, and do not move the task list into YAML.** Add resolution and
agreement checks over the prose that exists.

**1. Declare the `T-` prefix as a resolvable reference target without declaring an artifact
type.** `artifact-registry.yaml` currently conflates two things that are separable: *what may be
authored as an artifact* and *what a citation may resolve to*. Introduce a second, smaller block:

```yaml
# Prefixes that a citation may resolve to, which are not artifact types. A reference
# target has an id and a home file; it has no frontmatter, no status and no lifecycle.
reference_targets:
  - prefix: T
    home: TASKS.md
    pattern: '^T-[0-9]+\.[0-9]+$'
    declared: 2026-08-19
    notes: >
      Task ids. Cited by governed artifacts and by ROADMAP.md phase gates. Not an
      artifact type: a task is ephemeral, has no supersession, and must not become
      immutable — the three properties every artifact type here carries.
```

This is the minimum addition that lets the existing cross-reference scanner resolve `T-`
without granting tasks a lifecycle they should not have.

**2. Three checks**, declared in `check-registry.yaml`, owner `check-knowledge.sh`:

| Check id | Rejects |
| --- | --- |
| `knowledge.task_references_resolve` | a `T-` id cited in any scanned artifact, or in `ROADMAP.md`, with no matching heading in `TASKS.md` |
| `knowledge.task_status_agrees_across_files` | a task struck through as done in `TASKS.md` while `ROADMAP.md` marks it blocking, or the reverse |
| `knowledge.phase_gate_items_resolve` | a `ROADMAP.md` phase gate naming a `T-` id absent from `TASKS.md` |

**3. `TASKS.md` gains a stated id grammar and nothing else.** One line near the legend fixing
`T-<phase>.<sequence>` and stating that the heading is the id's definition. The prose, the
strikethrough convention and the *what / why it blocks / done when* structure are unchanged, and
deliberately so — they are the reason this file is worth checking rather than replacing.

**4. Each check is verified by planting the defect it should catch.** A `T-` id is added to a
scratch artifact and the gate must fail; a task is struck through while `ROADMAP.md` still blocks
on it and the gate must fail. **Treat a green check as unproven until you have watched it fail.**

## Alternatives considered

**A. A `task` artifact type, prefix `T-`, one file per task.** Rejected, and it was the first
idea. Every artifact type here carries `status`, `supersedes`/`superseded_by`, and immutability
once locked or superseded. A task has none of those properties honestly: it is done or it is not,
it is not superseded by a successor task, and freezing a completed task's text prevents the
correction `T-0.1` actually made when it recorded that its own objection had been wrong. It would
also fragment prose whose value is that it reads as one argument across a phase, and it would put
a large number of short files under an immutability rule designed for decisions.

**B. `task-registry.yaml` — tasks as registry data, `TASKS.md` generated from it.** Rejected,
though it is the option most consistent with the four prior instances of *the grammar is
constitutional, the list is data*. Two reasons. First, `ADR-0048`'s test is *does this change on
a cadence I do not set?* — a task's cadence is entirely ours, so that ADR does not reach it, and
the registry precedents it cites are all external or gate-computed values. Second and
decisively: `README.md` already states the governing aesthetic judgement — *"a generated table
dropped into the middle of an argument reads worse than a stale one."* `TASKS.md` is an argument.
`T-0.1`'s entry is three paragraphs explaining why the task's own objection was mistaken, and
that paragraph cannot be generated from a YAML row. Generating it would destroy the thing worth
keeping in order to check the thing that is easy to check.

**C. Put `ROADMAP.md` and `TASKS.md` fully under the artifact gate.** Rejected as directly
contradicting `T-0.1`, which examined and settled this. It would also demand frontmatter,
`status` and supersession on documents whose value is that they are readable narrative.

**D. Do nothing; the files are well maintained.** Rejected, but it is the honest status quo and
has more merit than the others listed. They *are* well maintained. The argument against is the
one this repository was founded on: **an unenforced convention is a suggestion, and a suggestion
is what a long-running or freshly-started agent drifts from.** The cost of being wrong here is
not cosmetic — it is starting Phase 3 work against a gate that reads closed and is not.

## Consequences

**1. A second kind of thing in `artifact-registry.yaml`.** `reference_targets` is not an
artifact type, and the file's own header warns at length that conflating independent axes is what
made the old rules special-cased. This adds an axis to the registry that governs axes. The
mitigation is that the block is deliberately minimal — a prefix, a home, a pattern — and carries
none of the artifact fields. It is still a genuine increase in the registry's conceptual surface,
and a future reader may reasonably ask why tasks are half-governed.

**2. Checks that parse Markdown structure rather than frontmatter.** Every existing knowledge
check reads YAML frontmatter or registry data. These read headings and strikethrough conventions
in prose, which is a weaker contract: a reformatting of `TASKS.md` could break them, and a check
that breaks on formatting will eventually be worked around rather than fixed. This is the
weakest part of the proposal and it should be stated as such rather than discovered.

**3. `TASKS.md` acquires a formatting obligation it does not have today.** The heading convention
becomes load-bearing. Freedom to restructure the file is genuinely reduced.

**4. It does not make the tasks correct.** Resolution and agreement are syntactic. A task whose
*"done when"* condition is wrong, or which is marked done without being done, passes every check
proposed here. This closes the dangling-reference class only, and calling it task governance
would overstate it.

## Rollout

1. Extend `artifact-registry.schema.json` for the `reference_targets` block, then declare `T-`.
2. Verify the three consumers that assert agreement with the registry — the `PreToolUse` guard,
   the ledger's `ART_DIRS`, and `knowledge.schema.json` — still agree. `reference_targets` must
   not leak into the artifact type enum or the scanned directory set. **This is the step most
   likely to go wrong**, and each has been wrong before.
3. Declare the check ids with falsifiable specs.
4. Add the id grammar line to `TASKS.md`.
5. Implement the checks, planting each defect first.

**Backwards compatibility:** additive. No artifact acquires an obligation. Existing `T-`
citations either resolve already or are the defects this exists to surface — **the first run is
expected to fail**, and that failure is the finding rather than a regression.

**How it is undone:** delete the `reference_targets` block and the check ids. `TASKS.md` keeps
its id grammar line harmlessly.

## Open questions

**1. Is `reference_targets` the right generalisation, or is `T-` the only member it will ever
have?** A block with one entry is a special case wearing a general name — the error this
repository's registries otherwise avoid. Candidate second members exist: phase ids from
`ROADMAP.md`, and `D1`–`D5` in `RFC-0013`. If none is ever added, a narrower mechanism would have
been better and this should be reconsidered before implementation.

**2. Should `ROADMAP.md` phase ids be resolvable too?** Phase gates are cited across artifacts by
phrases rather than ids. Giving phases ids would make the gates checkable and is a larger change
than this RFC proposes.

**3. How is a task's completion recorded so that a check can read it?** The current convention is
a strikethrough heading plus a bolded date. That is readable but not a contract, and question 2
of the Consequences applies. An explicit marker would be more robust and less pleasant to read.

**4. Does this belong in the vendored snapshot?** `T-` ids are cited by governed artifacts that
*are* vendored, so a downstream repository can receive a dangling citation it has no way to
resolve — `TASKS.md` is not part of the snapshot. This interacts with the snapshot currency gap
recorded separately and may be the stronger argument for the whole proposal.

## Related

`ADR-0022` (the type list, and fourteen silent failures), `ADR-0014` (declared checks),
`ADR-0002` (locked requires a check), `ADR-0035` (counts in prose), `C-0007` (grammar and list),
`ADR-0001` (vendored snapshots), `RFC-0015` (a fact has one home),
`T-0.1` (the decision this argues against), `ROADMAP.md` Phase 0 exit gate,
`ADR-0048`, `RFC-0013`, `RFC-0028`, `BET-0005` (governed artifacts citing `T-` ids).
