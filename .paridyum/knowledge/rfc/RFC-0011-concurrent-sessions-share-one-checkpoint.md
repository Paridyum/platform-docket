---
id: RFC-0011
title: Concurrent sessions share one checkpoint, so commits attribute and scope work falsely
type: rfc
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0043
scope: [org]
applies_to: ["*"]
tags: [hooks, git, checkpointing, concurrency, attribution, review, adr-0025]
breaking: false
enforced_by: []
---

# RFC-0011 — Concurrent sessions share one checkpoint

## Status

**PROPOSED.** Opened 2026-08-08, the day three sessions ran in this repository at once and the
evidence became unambiguous.

## The question

ADR-0025's `Stop` hook stages with `git add -A`, commits, and pushes, on every response. It was
written against a real loss and it solves that loss. It has no concept of a second session.

**When two sessions share one working tree, whose work does a checkpoint commit contain, and
whose name is on it?** Today the answer is: whatever is on disk, under whichever session's hook
fired. That is not a corner case — it is the observed default, and it happened eleven times today.

## What is not in question

Three things are settled and this RFC does not reopen them.

1. **The durability requirement is real.** Work was lost from local disk once. ADR-0025 already
   pre-rejects "rely on discipline, with the ledger as the reminder" — *"already tried, and it is
   what produced the loss."* Any option that reintroduces "commit when you remember" fails a test
   that has already been run in production.
2. **ADR-0025 is `locked`** with two `error`-severity checks. Changing its behaviour is a
   supersession, not an edit. A factual incompleteness in its costs may be an erratum under
   ADR-0031; the design change is not.
3. **Force-pushing is not on the table.** C-0004 denies it as a LAW and `guard-commands.sh`
   hard-denies it at the tool boundary. Every option below has to work forwards only.

## The evidence

**The scenario was anticipated, tested, and cleared against the wrong criterion.**
`plugins/paridyum-platform/hooks/checkpoint-session.sh`, in its own words:

> *"Two sessions in one repository both fire this hook. Tested: one wins, the other hits git's own
> index.lock and no-ops, **nothing is lost (the loser's work is picked up by the next
> checkpoint**, which fires on every response) and no lock is left behind."*

"The loser's work is picked up by the next checkpoint" is an accurate description of the
entanglement mechanism, recorded as the reassurance. *Nothing is lost* is true. It measures
durability and nothing else — not attribution, not scope, not reviewability.

**Three sessions ran on 2026-08-08**, interleaved. One checkpointed at 11:56 and again at 13:42;
another checkpointed twice inside that window. There is one worktree and one index, and the hook
stages with `git add -A` and no pathspec. So every edit the first session made in those 106
minutes was necessarily staged and committed by the second session's hook. This is mechanical,
not inferred. The first session's own 13:42 commit is four lines — what a 106-minute window looks
like after a peer has harvested it.

**Commit `26ef929` is the type case:** 23 paths, 1,629 insertions, one `session:` trailer, and
three unrelated workstreams — seven internationalisation artifacts (ADR-0032/0033/0034, GLO-0006,
STD-0010, ANTI-0008, REQ-0019 and the migration-gate checks), a JUnit-evidence change under
STD-0008 (`.gitignore`, the CI upload step, `TESTING.md`), and a `CLAUDE.md` copy edit. Its
CHANGELOG diff contains two unrelated release-note entries. The commit body records `green` — a
gate verdict on a union state neither session constructed or reviewed.

**One coherent change was split across two commits with a foreign workstream in each half.** The
JUnit machinery landed in `a8e1276` at 12:03; its `.gitignore` entry, CI step, documentation and
changelog entry landed in `26ef929` at 12:28. Nobody slices work that way. A response-boundary
commit trigger does.

**`main` is a branch nobody reviews, provably.** `scripts/check-pr.sh` on this repository, today:

> *"n/a HEAD is at the merge base with origin/main — no branch diff to review… This is also the
> normal state of this repository, which commits straight to main."*

Branch protection is unavailable: the repository is private under an Organization on GitHub Free,
which is exactly what ADR-0004 predicted and why `org.branch_protection_reconciled` is `warn`.
The CI workflow says so in its own header — *"a signal, not a gate."*

**This matters because it invalidates ADR-0025's stated reason for its own rejected alternative.**
The ADR rejected per-branch checkpointing partly because *"the protected work is on a branch
nobody reviews."* That argument assumed `main` **is** reviewed. It is not, and cannot be on this
plan. The comparison was made against a world this repository does not operate in.

## The failure modes

Ordered by severity, and each one observed rather than hypothesised.

1. **Attribution is systematically false.** The `session:` trailer is the highest-precision field
   in the history and the one most likely to be wrong. `capture-open-loops.sh` then writes
   per-session judgements into DECISION-DEBT.md on top of a per-tree record.
2. **A session publishes a peer's half-finished work.** ADR-0025 cost 5 already states *"nothing
   here is a draft once the hook has run"* — which was a statement about your own work and is now
   a statement about someone else's. The escape hatches do not help: `.paridyum-no-checkpoint` is
   a repo-root file, so one session opting out disables checkpointing for all of them.
3. **The gate certifies a combination nobody built.** Four gates run against the union of two
   trees and `green` goes into the commit message. The inverse is worse: on a red checkpoint,
   nothing in the record says whose edit caused it.
4. **STD-0007 is structurally unenforceable.** `pr.diff_within_review_budget` is `severity: error`
   on a `locked` standard with a 400-line ceiling. Four of the last ten commits are 1,651, 1,637,
   1,271 and 1,061 lines, and the check reported `n/a` for every one, because no branch diff
   exists. STD-0007 cost 5 admits this; ADR-0025 is what makes it the operating mode.
5. **A non-fast-forward compounds instead of failing fast.** No fetch, no rebase, no backoff — the
   hook commits again on the diverged tip on the next response, so the stranded pile grows
   monotonically. The remedy it prints is a history operation.
6. **The `index.lock` pre-flight silently drops a checkpoint**, by design and correctly, which
   makes the durability guarantee probabilistic under concurrency. ADR-0025 does not say so.

## Constraints inherited before any option is compared

- Durability must not regress. Loss on disk is the failure this system exists to prevent.
- No force-push, ever (C-0004).
- The hook must stay fast and dependency-free; it runs on every response.
- macOS ships no `flock`, and a `mkdir` lock adds a stale-lock failure mode worse than the noise
  it removes — the hook's own note, and it constrains any locking option.
- Whatever lands must work with a single shared worktree, because that is what the harness gives
  us today.

## Options

**A. Per-session checkpoint branch.** Each session checkpoints to `checkpoint/<session-id>`,
never to `main`. Durability is unchanged — the work is off the machine. Attribution becomes true
by construction. `main` receives merges, which finally gives `check-pr.sh` an input and makes
STD-0007 enforceable for the first time. Cost: a merge step nobody does today, and a branch
graveyard needing a reaper. ADR-0025's rejection of this rested on `main` being reviewed, which
the evidence above refutes.

**B. Session-scoped staging.** Replace `git add -A` with a pathspec derived from files the
session actually touched. Keeps one branch. Cost: the hook must learn what a session touched,
which the harness does not currently tell it, and anything missed is silently *not* preserved —
trading a scope defect for a durability defect, which is the wrong direction.

**C. Advisory lock with hand-off.** A real mutex so only one session checkpoints at a time, with
the others deferring. Cost: the hook's own note explains why this is worse than it looks on
macOS, and it fixes contention rather than attribution — the winner still commits everyone's work.

**D. Honest labelling only.** Keep the behaviour; stop claiming a single author.

A first draft of this option said "record every session id observed in the tree". **That is not
implementable and the reason is worth stating**, because it constrains every option here: the hook
learns its identity from `jq -r '.session_id' <<<"$payload"` — its **own**, from its own hook
payload. It has no way to enumerate other live sessions. Any design that assumes a session can see
its peers is assuming a harness feature that does not exist.

What is implementable today, with no harness change:

1. **Rename the trailer.** `session:` asserts authorship. `checkpointed-by:` asserts what actually
   happened — this session ran the hook. One word, and the history stops making a claim it cannot
   support. `capture-open-loops.sh` consumes this field and should be read in the same change.
2. **Use the contention signal that is already being discarded.** The `index.lock` pre-flight is a
   *positive detection* of a concurrent session, and it currently exits without recording it. Have
   the loser leave a breadcrumb; have the next checkpoint read it and mark the commit as covering
   a shared tree. This is the only reliable peer-detection available, and the hook already has it.
3. **Downgrade the verdict wording when the tree is shared.** `green` means "the gates passed on a
   state someone built". On a union state nobody built, `unverified` is the honest word.

Cheapest by far, fixes nothing structural, and makes the record stop lying. A credible first slice
under any of A, B or C.

**E. Do nothing, and squash before release.** ADR-0025 cost 1 already prescribes squashing. It
addresses noise and addresses none of failure modes 1-4: a squash cannot separate two sessions'
work after the fact, and by then everything is on `origin/main` and unsplittable without a
force-push.

## Discussion

The entanglement already in the history is **permanent**. Every affected commit is an ancestor of
`origin/main`, and separating them means rewriting published history. So this RFC cannot repair
`26ef929`, `a8e1276`, `9aa201e`, `bf64538` or `8b29ffa`. It can only stop the next one.

D is the only option that is obviously correct in isolation, and it is not sufficient. A is the
only option that makes STD-0007 enforceable, which is a second locked rule currently held up by
this one — and that suggests the question is bigger than checkpointing. Two locked artifacts are
in tension: ADR-0025 requires committing on every response, STD-0007 requires changes to arrive
in reviewable slices, and today ADR-0025 wins silently by making STD-0007's check report `n/a`.

A rule that cannot fire is not a weaker rule; it is an absent one wearing a severity.

## Recommendation

Take **D immediately** — it is small, it regresses nothing, and it stops the history asserting
things that are false. Then decide between **A** and **B**, with A preferred on the evidence,
because it is the only one that also returns an input to `check-pr.sh`.

Resolving this needs an ADR superseding ADR-0025, not an edit to it. ADR-0025 cost 6 — which
scopes concurrency to *"concurrent work from two machines"* — may additionally warrant an erratum
under ADR-0031, since the statement is incomplete rather than wrong.

## Related

- ADR-0025 — the decision this questions; `locked`, so any change is a supersession
- ADR-0031 — the erratum mechanism, for cost 6's incompleteness
- ADR-0004 — GitHub Free, which is why no branch protection exists and `main` is unreviewed
- STD-0007 — the review budget this makes unenforceable
- ADR-0027 — hooks have a second channel; tolerates double registration, does not address sessions
- ADR-0002 — *"Paridyum is built primarily by AI agents working in parallel"*, the stated premise
- REQ-0006, REQ-0007 — governance and agent authority, both adjacent, neither recording this
- C-0004 — force-push denial, which makes the existing entanglement permanent
