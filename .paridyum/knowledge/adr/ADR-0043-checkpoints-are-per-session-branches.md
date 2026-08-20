---
id: ADR-0043
title: Checkpoints commit to a per-session branch, and a shared tree is labelled rather than hidden
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: [ADR-0025]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [hooks, git, checkpointing, concurrency, attribution, review, rfc-0011]
breaking: true
enforced_by:
  - check: repo.session_checkpoint_hook_registered
    severity: error
    rationale: >
      Carried forward from ADR-0025 unchanged. The hook is the mechanism; a repository where it
      is not registered has no durability at all, which is the failure that produced ADR-0025.
  - check: repo.git_remote_configured
    severity: error
    rationale: >
      Also carried forward. A commit on one disk is still on one disk — the sentence ADR-0025
      was written around, and equally true of a branch.
  - check: repo.checkpoint_branch_is_not_default
    severity: error
    rationale: >
      The whole change is that checkpoints stop landing on the branch everything is reviewed
      from. A hook that silently falls back to committing on `main` would restore the previous
      behaviour while the ADR claims otherwise, and nothing else would notice.
---

# ADR-0043 — Checkpoints commit to a per-session branch

## Status

**ACCEPTED** as of 2026-08-08. **Supersedes ADR-0025. Resolves RFC-0011.**

ADR-0025's durability requirement is carried forward unchanged and is not reopened. Work was lost
from local disk once; that is why the hook exists, and nothing here weakens it.

## Context

RFC-0011 documented what ADR-0025's design did not contemplate: **two sessions sharing one
working tree.** ADR-0025 cost 6 scopes concurrency to *"concurrent work from two machines"* —
remote divergence between clones — and the case that actually occurred is one index, one tree,
and `git add -A` staging whatever is on disk under whichever session's hook fires first.

On 2026-08-08 that put 23 paths and 1,629 lines from three unrelated workstreams into one commit
under a single `session:` trailer, with `green` recorded for a gate verdict on a union state
neither session constructed.

The mechanism was anticipated and cleared against the wrong criterion. The hook's own comment:
*"one wins, the other hits git's own index.lock and no-ops, nothing is lost (the loser's work is
picked up by the next checkpoint)."* **Nothing is lost** is true and measures durability alone.
"The loser's work is picked up by the next checkpoint" is a precise description of the
entanglement, recorded as the reassurance.

The rejected alternative in ADR-0025 was per-branch checkpointing, declined because *"the
protected work is on a branch nobody reviews."* RFC-0011 establishes that premise is false:
branch protection is unavailable for private repositories on GitHub Free (ADR-0004), and
`check-pr.sh` reports `n/a` by construction. **`main` is also a branch nobody reviews** — so the
comparison was made against a world this organisation does not operate in.

## Decision

**1. Checkpoints commit to `checkpoint/<session-id>`, never to the default branch.** The hook
creates the branch on first use and stays on it for the session's lifetime. Durability is
unchanged — the work is committed and pushed, off the machine.

**2. `main` receives merges, not checkpoints.** Which gives `pr.diff_within_review_budget` an
input for the first time. STD-0007's 400-line ceiling is `severity: error` on a locked standard
and has never once evaluated, because no branch diff has ever existed.

**3. Attribution states what happened, not who authored.** `checkpointed-by:` replaces
`session:`. Already shipped ahead of this ADR — recorded here because RFC-0011 noted the
inversion, and ADR-0019's *"the habit is worth not repeating"* applies to us.

**4. A shared tree is labelled, never hidden.** The `index.lock` pre-flight is the only component
that positively detects a peer — the hook reads its own `session_id` and cannot enumerate others.
The loser leaves a breadcrumb; the next checkpoint records `unverified (shared tree)` and cites
RFC-0011. Also already shipped.

**5. Absent gates and an unrun secret scan degrade the verdict.** `green` asserts the gates
passed on this content. It must not survive a gate that never ran. Already shipped.

**6. Branches are reaped.** A `checkpoint/*` branch merged or older than 30 days is deleted. A
branch graveyard is the cost of decision 1 and it is not self-limiting.

**7. Everything else in ADR-0025 is carried forward verbatim** — a failing gate does not block
the commit, a secret does and is the only thing that does, the hook exits 0 unconditionally,
attribution is inherited with the AI as a trailer, and the escape hatches remain.

## Consequences

**Accepted costs**

1. **A merge step exists that did not before**, and nobody does it today. Work now lands on a
   branch that must be merged deliberately — which is the point, and it is also a new way for
   work to sit unnoticed. A stale `checkpoint/*` branch is less visible than a noisy `main`.

2. **Two sessions can still entangle within one branch.** If both are on the same worktree they
   still share an index; per-session *branches* separate their histories only when their
   `session_id`s differ, which is the normal case but not a guarantee. This narrows the failure,
   it does not eliminate it — eliminating it needs per-session worktrees, which the harness does
   not provide.

3. **The reaper deletes work if it is wrong.** A 30-day rule applied to an unmerged branch that
   somebody still wanted is data loss, in the mechanism built to prevent data loss. It must
   refuse to delete anything unmerged without an explicit flag.

4. **The existing entanglement is permanent.** Every affected commit is an ancestor of
   `origin/main`, and separating them means rewriting published history, which C-0004 denies.
   `26ef929`, `a8e1276`, `9aa201e`, `bf64538` and `8b29ffa` stay as they are.

5. **Supersession costs the record.** ADR-0025 was locked and is now superseded, so a reader
   must follow one more hop to learn why checkpointing exists at all. Its two errata — the
   signing correction and cost 6's incompleteness — remain readable there.

**Gained**

- Attribution stops being false. The `session:` trailer was the highest-precision field in the
  history and the one most likely to be wrong.
- STD-0007 becomes enforceable. A locked `error`-severity rule that has never evaluated is an
  absent rule wearing a severity, and this is what gives it an input.
- A shared tree is visible in the record rather than inferred from a commit's shape.

**Rejected alternatives**

- **Keep committing to `main` and label honestly.** RFC-0011's option D, most of which shipped
  already. It stops the record lying and leaves STD-0007 permanently unenforceable, which is the
  second locked rule this defect disables.

- **Session-scoped staging** — replace `git add -A` with a pathspec of files the session touched.
  Keeps one branch, and the harness does not tell the hook what a session touched. Anything
  missed is silently *not preserved*, trading a scope defect for a durability one.

- **A real mutex.** Fixes contention, not attribution — the winner still commits everyone's work.
  macOS ships no `flock` and a `mkdir` lock adds a stale-lock failure worse than the noise.

- **Do nothing, squash before release.** ADR-0025 cost 1's prescription. A squash cannot separate
  two sessions' work after the fact, and by then it is all on `origin/main`.

## Related

- ADR-0025 — superseded; its durability argument is carried forward intact
- RFC-0011 — the analysis this resolves
- ADR-0004 — GitHub Free, why no branch protection exists and `main` is unreviewed
- STD-0007 — the review budget decision 2 finally gives an input
- C-0004 — force-push denial, which makes cost 4 permanent
- ADR-0031 — errata; ADR-0025 carries two and they remain readable
- ADR-0027 — hooks have a second channel
