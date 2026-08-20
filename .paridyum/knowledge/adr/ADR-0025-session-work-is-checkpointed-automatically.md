---
id: ADR-0025
title: Session work is committed and pushed automatically, and only a secret blocks it
type: adr
status: superseded
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: ADR-0043
scope: [org]
applies_to: ["*"]
tags: [durability, git, hooks, data-loss, automation, agent-harness]
breaking: false
enforced_by:
  - check: repo.session_checkpoint_hook_registered
    severity: error
    rationale: >
      An automation that silently stops being registered protects nothing, and its absence
      looks identical to its success — no commits either way until the disk is gone.
  - check: repo.git_remote_configured
    severity: error
    rationale: >
      A commit on one disk is still on one disk. Without a remote the checkpoint preserves
      work against a mistake but not against the failure that actually happened.
---

# ADR-0025 — Session work is committed and pushed automatically

## Status

**LOCKED** as of 2026-08-07, with two implemented checks.

Locked at the owner's explicit instruction, and the instruction is well-founded: this is a
decision whose value comes entirely from being unconditional. A durability rule that holds
except when somebody is in a hurry is a durability rule that fails on exactly the day it
matters.

## Context

**Work in this repository has been lost from local disk once.** DECISION-DEBT.md has reported
*"governance edits that exist on one disk only"* since 2026-08-06, and at the moment this
decision was taken there were more than thirty uncommitted files — an entire session of
constitutional, architectural and tooling work, none of it anywhere but one laptop.

The irony is the argument. This repository exists so that *"a decision that exists only in
someone's head, or only in a chat log, does not exist"* (C-0001 principle 3). A knowledge base
whose whole purpose is that rules outlive the people who wrote them is worthless if it does not
outlive a disk. C-0001 principle 2 is the other half: *"the second time a task is performed by
hand is the signal to automate it."* Committing has been performed by hand many times and
missed at least once, at a cost.

ADR-0011 already established that a Stop hook is the right mechanism for something that must
happen whether or not anyone remembers it exists.

## Decision

**A Stop hook runs the full gate pipeline, then commits everything and pushes it. The secret
scan is the only gate permitted to block the commit; every other result is recorded in the
commit message.**

1. **The pipeline runs first** — `check-knowledge.sh`, `check-repo.sh`,
   `check-migrations.sh`, `check-changelog.sh`, `test-checks.sh` — and each result is written
   into the commit body.

2. **A failing gate does not block the commit.** This inverts the usual arrangement and the
   inversion is the whole design. A gate that refuses to commit failing work protects the
   history and loses the work, which is backwards when the failure being defended against is
   data loss. **A red commit that exists beats a green working tree that does not.** Failing
   checkpoints are findable: `git log --grep 'gates: .*FAIL'`.

3. **A secret blocks the commit, and nothing else does.** Committing a credential is the one
   outcome a later commit cannot undo — it is in the history, and history is what an attacker
   clones. C-0004 already treats this as protected and `repo.secret_scan_clean` already exists.
   Scanned twice: the working tree before staging, and the staged set after, because those
   differ when a path is ignored by one and not the other.

4. **It pushes.** A commit on one disk is still on one disk, and one disk is what failed. A
   push failure is reported loudly and never resolved by force — C-0004 denies force-pushing to
   `main`, and this hook must not become the thing that quietly rewrites history.

5. **Attribution is inherited, not overridden.** C-0003 makes the human who initiated and
   accepted the work the Author; the hook does not set an author, so it inherits the configured
   identity, which is correct rather than merely convenient. The AI appears as a
   `Co-Authored-By` trailer. Signing is inherited too — see cost 2.

6. **It exits 0 unconditionally,** like `capture-open-loops.sh`. A Stop hook that fails must
   never disrupt the session, and one that blocked a session in order to protect a branch would
   have inverted its own purpose.

7. **Ordering: after `capture-open-loops.sh`.** That hook rewrites DECISION-DEBT.md, so a
   checkpoint running before it would leave the ledger update uncommitted until the next
   session — the exact gap this closes, reintroduced one file at a time.

8. **There is an escape hatch:** a `.paridyum-no-checkpoint` file or the
   `PARIDYUM_NO_CHECKPOINT` environment variable, and an automatic skip during rebase, merge,
   cherry-pick and bisect. An automation with no way out gets disabled wholesale, and then it
   protects nothing.

## Costs accepted

1. **The history becomes noisy, and this is the real price.** A Stop hook fires whenever the
   assistant finishes responding, so an active session produces a series of
   `chore(checkpoint):` commits rather than one curated change. Mitigated only partially by
   skipping when the tree is clean. **Squash before any release**; the checkpoint series is a
   safety net, not a narrative. Accepted because a readable history of work that no longer
   exists is worth nothing.

2. **The commits are unsigned, and this decision does not fix that.** No signing key is
   configured, so every commit in this repository is unsigned today while C-0003 requires
   otherwise and `git.commits_are_signed` reports it. The hook inherits that configuration
   deliberately rather than passing `-S` and failing, or suppressing the check. Automating an
   unsigned commit makes an existing gap higher-volume, which is an honest cost and not a new
   violation. Configuring a key closes it everywhere at once.

3. **`--no-verify` is used, and it is not a bypass.** The hook has already run every gate the
   pre-commit hook would run. Letting the hook run again would re-run the same checks and, more
   importantly, would let a failing gate abort the commit — reinstating precisely the
   refuse-to-save behaviour decision 2 rejects. The gates are not skipped; they are moved
   earlier and made non-fatal.

4. **`git add -A` commits everything, including files nobody meant to keep.** A scratch file,
   a stray download, a half-finished experiment. The secret scan catches the dangerous subset;
   the merely untidy subset lands in the history. `.gitignore` is the remedy and it is now
   load-bearing.

5. **Automatic pushing publishes.** Work reaches the remote without anyone deciding it is
   ready. The repository is private under ADR-0004, so the audience is the owner, but the
   property is real: nothing here is a draft once the hook has run.

6. **A push conflict needs a human.** The hook refuses to rebase or force, so concurrent work
   from two machines leaves commits stranded locally with a loud message. That is the correct
   failure — the alternative is an automation that resolves history conflicts unattended.

## Alternatives rejected

**Commit only when every gate is green.** The obvious design, and it fails the actual
requirement: the moments most worth preserving are the ones mid-refactor, where something is
red. This would have protected the history and lost the work.

**Rely on discipline, with the ledger as the reminder.** Already tried, and it is what
produced the loss. DECISION-DEBT.md reported the one-disk state accurately for a day and
nothing happened, because a reminder competes with whatever else is in front of the person.

**Commit locally, push manually.** Half a solution. The failure that occurred was disk-level,
and a local commit does not survive it.

**Checkpoint to a separate branch or ref.** Keeps `main` clean and is the tidiest option. It
also means the protected work is on a branch nobody reviews, and it needs conflict handling
between the checkpoint branch and `main` that nothing else in this repository needs. Rejected
as complexity bought with the wrong currency — cost 1 is a real cost and it is cheaper than a
second branch model.

**A time-based backup outside git.** Would preserve the tree without touching history, and
would not preserve *why* anything changed. The commit message is where the gate results and the
session id live, and that context is most of the value.

## Consequences

- `plugins/paridyum-platform/hooks/checkpoint-session.sh` exists and is registered as the
  second `Stop` hook.
- Two new checks, both implemented in `check-repo.sh`:
  `repo.session_checkpoint_hook_registered` and `repo.git_remote_configured`. Both are
  defect-proven in `test-checks.sh`, per the rule that a check counts as implemented only if
  that suite plants the defect it claims to reject.
- `.gitignore` becomes load-bearing (cost 4). A path that should never be committed must be
  ignored rather than merely left alone.
- The `git.commits_are_signed` gap is now higher-volume and unchanged in kind. Configuring a
  signing key is the single highest-value follow-up to this decision.

## Related

- C-0001 — principle 2 (automate the second time) and principle 3 (unrecorded is nonexistent)
- C-0003 — attribution; the human is Author, the AI is a trailer
- C-0004 — secrets and force-push are protected operations
- ADR-0007 — the harness ships as a plugin; hooks live there
- ADR-0011 — the precedent that a Stop hook is how something happens regardless of memory
- ADR-0002 — why this is locked rather than accepted: two checks enforce it

## Errata

### 2026-08-08 — cost 2 states that commits are unsigned; they are signed

**Says:** *"The commits are unsigned… No signing key is configured, so every commit in this
repository is unsigned today."*

**Wrong:** `commit.gpgsign` is `true`, `gpg.format` is `ssh`, and every recent commit verifies.
SSH commit signing was configured and the CHANGELOG records it. The decision is unaffected —
this ADR still does not override signing, it inherits whatever is configured, which is exactly
what cost 2 argued for. What changed is the configuration, not the design.

**Intended:** *"The commits inherit whatever signing is configured. When this was written none
was, so every commit was unsigned; signing was configured on 2026-08-08 and this ADR needed no
change for that to take effect — which is the point of not overriding it. `git.commits_are_signed`
remains declared and unimplemented, so nothing enforces it."*

### 2026-08-08 — cost 6 scopes concurrency to two machines; the live case is two sessions in one worktree

**Says:** *"A push conflict needs a human. The hook refuses to rebase or force, so concurrent
work from two machines leaves commits stranded locally with a loud message."*

**Wrong:** incomplete rather than incorrect. The stated case — two clones diverging at the remote
— is real and handled. The case that actually occurred is two sessions sharing **one** working
tree and one index, where `git add -A` commits whichever session's work is on disk under
whichever session's hook fires first. On 2026-08-08 that put 23 paths and 1,629 lines from three
unrelated workstreams into one commit under a single `session:` trailer. RFC-0011 documents it.

**Intended:** cost 6 to name both cases — remote divergence between clones, and concurrent
sessions in a shared worktree, the second of which this ADR's design does not address.
