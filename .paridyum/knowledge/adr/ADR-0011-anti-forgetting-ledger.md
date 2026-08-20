---
id: ADR-0011
title: Open loops are recorded automatically and resurfaced every session
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [ai-agents, memory, governance, decision-debt, hooks]
breaking: false
enforced_by:
  - check: repo.decision_debt_ledger_present
    severity: warn
    args: { path: "DECISION-DEBT.md" }
  - check: repo.no_dangling_decision_references
    severity: error
    rationale: A cited-but-unwritten ADR is a decision that was discussed and then dropped.
  - check: knowledge.no_stale_proposed_artifacts
    severity: warn
    args: { max_age_days: 30 }
    rationale: An RFC proposed for a month is either forgotten or should be withdrawn.
---

# ADR-0011 — Open loops are recorded automatically and resurfaced every session

## Status

**LOCKED** as of 2026-08-06.

## Context

ADR-0002 and ADR-0007 solve one direction of the memory problem: rules that **are**
written down get retrieved and enforced. Neither addresses the other direction —
a decision that was made in conversation and never written at all.

That gap is not an agent problem. It is a *both parties* problem: the human forgets
they decided something, the agent's context is discarded at session end, and the
decision survives nowhere. Nothing in the enforcement ladder catches it, because
there is no artifact to check.

The stated requirement was direct: a mechanism that does not let Claude forget **even
when the user has also forgotten.** That rules out anything requiring either party to
remember to invoke it.

## Decision

**Open loops are detected automatically at the end of every session, written to a
committed ledger, and injected back into context at the start of the next one.**

Two hooks, already part of `paridyum-platform` (ADR-0007):

- **`Stop` → `capture-open-loops.sh`** runs when Claude finishes responding. It
  recomputes the ledger and appends anything suspicious from the session.
- **`SessionStart` → `session-context.sh`** reads the ledger and prints the open
  items before the first prompt.

Neither requires anyone to remember anything. That is the entire design constraint.

### The ledger has two halves, and they behave differently

**Regenerated** — recomputed from scratch every run, so it is self-healing. Close the
loop and the entry disappears without anyone deleting it. Covers what is mechanically
detectable:

- artifacts sitting in `draft` or `proposed`, with their age
- decision ids cited in prose that were never written — a dangling `ADR-00NN` is
  almost always a decision that got discussed, referenced, and dropped
- locked artifacts with an empty `enforced_by` (ADR-0002's backlog)
- uncommitted governance changes, and a missing git remote

**Appended** — heuristic, and deliberately never auto-cleared. The hook scans the
session transcript for decision-shaped language and flags a session that contains a
lot of it but produced no artifact. Only a human can judge whether a real decision
occurred, so these accumulate until explicitly resolved.

That asymmetry is intentional. **An entry nobody clears is itself the signal.** A
ledger that quietly tidied itself would recreate the forgetting it exists to prevent.

### Calibration

The heuristic is biased toward false positives. A false positive costs one line to
dismiss; a false negative is precisely the failure this ADR addresses. The threshold
(eight or more decision-shaped statements, and no artifact created or changed in six
hours) was chosen to stay quiet during ordinary implementation work and speak up
during design discussion.

It is a prompt, not a claim. The flag says "was something decided that is not written
down?" — it never asserts that something was.

## Consequences

**Accepted costs**

- `DECISION-DEBT.md` is machine-written and committed, so it appears in diffs and
  occasionally causes noise in an otherwise clean commit. Judged worth it: a
  gitignored ledger would not survive a fresh clone, and durability is the point.
- The transcript heuristic will sometimes flag a session where nothing was decided.
  Dismissing costs a tick.
- The Stop hook runs on every response. It is bounded by `find` over the knowledge
  base, and exits 0 unconditionally — a Stop hook that fails must never disrupt a
  session.
- Covers Claude Code sessions only. A decision made in a meeting, in Slack, or in a
  terminal is invisible to it.

**Gained**

- The set of things that were decided but not recorded stops being invisible.
- Every session begins by seeing what the previous one left open, without anyone
  needing to have remembered.
- The mechanism demonstrated its value on first run: it immediately surfaced a
  dangling `ADR-0009` reference, seven uncommitted governance changes, and the fact
  that this repository had **no git remote at all** — meaning the entire knowledge
  base existed on a single disk. None of the three was noticed by either party
  before the hook reported them.

**Rejected alternatives**

- *Rely on the agent to notice and record decisions* — this is exactly what fails
  after two hours of context, and what a fresh parallel agent never had.
- *Ask the user at session end* — requires them to be present and to care at the
  moment they are least likely to, and fails in non-interactive runs.
- *Full NLP extraction of decisions from the transcript* — better recall, but a
  confident wrong summary of a decision is worse than a flag saying "check this".
  The ledger points; it does not paraphrase.
