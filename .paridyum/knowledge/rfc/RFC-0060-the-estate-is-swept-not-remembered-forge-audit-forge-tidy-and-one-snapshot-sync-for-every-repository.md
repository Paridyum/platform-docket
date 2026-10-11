---
id: RFC-0060
title: "The estate is swept, not remembered: forge audit, forge tidy, and one snapshot sync for every repository"
type: rfc
status: accepted
created: 2026-10-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [workflow, forge, audit, tidy, worktrees, branches, snapshot, ci, runners, adr-0004, adr-0001, adr-0059, adr-0124, rfc-0027]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
---

# RFC-0060 — The estate is swept, not remembered

## Status

**ACCEPTED** 2026-10-08, by the operator under ADR-0087's personal authority. Proposed the same
day. All five decisions adopted. The channel D5 names and the GitHub-hosted fallback stay open
questions, as written below.

D1 was not new authority: ADR-0004, locked, already requires `forge audit` and calls it *"the most
important one to build early, not last"*. It is in this RFC because nobody built it, and because
the sweep that stood in for it on 2026-10-08 measured what it has to report. D2 to D5 are new, and
are adopted by this acceptance.

## Problem

**The operator, 2026-10-08:**

> *"fix all the repo and knowledge been a headache see if we have good workflows"* — then, on the
> findings: *"go on"*.

The sweep that answered it found that **nothing had been lost and almost nothing was where it
belonged**. Every repository had zero uncommitted changes — the checkpoint hook (ADR-0025, ADR-0043)
does its job. What no mechanism does is put local state back after the work it served is merged.
Measured on 2026-10-08, across the local clones of every repository in the organisation:

| Residue | Measured |
| --- | --- |
| Local branches whose work was already on `origin` or in a GitHub PR head | 253 |
| Linked worktrees of finished work, left in place | 30 |
| Worktree registrations whose directory no longer existed — created inside a session's own scratchpad, which was later deleted | more than 30 |
| Clones parked on a feature branch rather than `main` | 9 |
| Clones behind `origin/main` | 20, the furthest by 57 commits |
| Vendored snapshots differing from `knowledge` `main` | all of them, by 36 to 156 files, nine days after the last refresh |
| Research-log lines that existed only on one disk | 92, in the `knowledge` main checkout and in finished worktrees |
| Installed harness behind `main` | every workstation: the plugin version had not moved through two weeks of hook changes (#47) |
| Self-hosted CI runners online | none, with `main`'s run queued for more than a day and no one told |

Each row was cleared by hand on 2026-10-07 and 2026-10-08, in more than twenty pull requests and a
sweep script that lived in a session scratchpad. **Clearing it by hand is the problem**: it will
be back in a fortnight, found the same way, by the same reading of every repository.

## What is already decided

- **ADR-0004 (locked):** `forge audit` *"sweeps every repository in the organisation and reports
  drift. This is the only layer that catches a bypass after the fact, which makes it the most
  important one to build early, not last."*
- **ADR-0001 (locked):** every repository vendors the knowledge base at `.paridyum/knowledge/`,
  refreshed by `forge knowledge sync`, with a 30-day freshness window. The redundancy is accepted;
  the refresh cost is not designed.
- **ADR-0059:** *a warning no channel carries is not a warning*.
- **ADR-0124:** CI runs on a self-hosted runner in the estate.
- **The `knowledge-audit` skill** describes the per-repository sweep by hand, *"until
  `forge audit` ships"*, and says: *do not fix drift silently*.
- **STD-0013:** a capability is retrieved before it is rebuilt, and every tool carries an SOP.
- **RUN-0001:** the workspace layout, where `scratch/` is *"deleted without warning"*.

## Proposal

### D1 — `forge audit` reports drift and changes nothing

Implements ADR-0004. Run over a workspace laid out per RUN-0001, it reports for every local clone:
its branch, how far it is behind or ahead of `origin/main`, uncommitted changes, stashes, linked
worktrees (present or dead), and local branches that exist nowhere else; whether its vendored
snapshot matches the `knowledge` checkout; and whether its name carries a prefix
`class-registry.yaml` declares. It reads only local refs unless asked to fetch. **It changes
nothing, ever** — the `knowledge-audit` skill's rule, kept: a repository that drifted may have
drifted for a reason worth knowing.

### D2 — `forge tidy` removes only what is provably held elsewhere

A plan by default; `--apply` acts. It may remove a worktree or a local branch only when its commit
is **an ancestor of `origin/main`, patch-equivalent to it, contained in its own
`origin/<branch>`, or contained in a pull request's head** — the four tests the 2026-10-08 sweep
used, which deleted 253 branches and lost nothing. It prunes dead worktree registrations, and
moves a clean clone parked on a held branch back to `main` and fast-forwards it.

**It never** drops a stash, deletes a branch it cannot prove is held, removes a worktree with
changes, force-pushes, or touches a remote. What it keeps it lists, with the reason, for a human.

### D3 — A worktree lives under `scratch/pd-wt/`, and a merge removes its own

Never inside a session scratchpad, whose deletion strands the registration. The session that
merges a pull request removes that worktree and that local branch in the same breath; D2 catches
whatever a session forgot.

### D4 — One command refreshes every snapshot

`forge knowledge sync --all <workspace>` runs the existing sync in every clone, from one clean
export of `knowledge` `main`, on one branch per repository — `chore/knowledge-snapshot-<sha>`, the
name the 2026-09-28 and 2026-10-08 refreshes used. Opening and merging the pull requests stays a
separate, visible step. A CI job that does this on every merge to `knowledge` `main` is the end
state, and waits on D5.

### D5 — CI that cannot run says so

When no runner is online, or a run on `main` has been queued past a threshold, a channel the
operator reads carries it (ADR-0059). Which channel is open question 1.

## Alternatives considered

**Keep sweeping by hand, with the `knowledge-audit` skill as the procedure.** It is what happened,
and it cost two days. The skill's own text says it stands in for `forge audit` *until it ships*.

**GitHub's "automatically delete head branches".** It deletes the remote branch on merge, which is
the half that was never the problem: every residue above is local.

**A scheduled `forge tidy --apply` with nobody reading the plan.** Rejected. Even with D2's four
tests, a destructive sweep nobody watches is the shape the `knowledge-audit` skill warns against.
The audit may run on a schedule; the tidy runs when a person asks.

**A monorepo, so there is one clone and one snapshot.** ADR-0001 is locked, and it chose the
polyrepo for agent context locality. The refresh cost is its accepted price; D4 makes the price
one command rather than a session.

## Consequences

**Accepted costs**

1. **`forge` becomes a tool every workstation must have installed** — it was not on the operator's
   machine on 2026-10-08, and was built from source to run the sync. RUN-0001 must install it.
2. **D2 can still delete the wrong thing** if `origin` is wrong — a force-pushed remote branch, a
   pull request edited after the fact. The tests prove containment in what GitHub holds now.
3. **D4 still produces one pull request per repository.** The mechanism is cheaper; the review
   load is not, until the CI job exists.

**Gained**

- The residue is reported before it is a headache, by the backstop ADR-0004 has required since
  2026-08-06.
- Cleaning up is a command with a plan, not a reading of every repository.

## Rollout

1. `forge audit` — the local report (D1).
2. `forge tidy` — plan and `--apply` (D2), with D3 written into RUN-0001.
3. `forge knowledge sync --all` (D4).
4. The CI alarm (D5), once its channel is chosen.

Each is its own pull request in `tool-forge`, with its SOP section (STD-0013).

## Open questions

1. **Which channel carries "CI cannot run"?** It must be one the operator reads (ADR-0059).
   **Resolved 2026-10-08 by the operator: the session summary** — the channel ADR-0059 decision 3
   already uses. `session-context.sh` asks GitHub with the operator's own `gh` login, bounded to a
   few seconds, and says nothing when it cannot ask.
2. **Should the bash knowledge gate fall back to GitHub-hosted runners** when none of the estate's
   is online? It needs no Rust build and would have kept `knowledge` verified through the outage.
   It departs from ADR-0124, so it would be an ADR of its own, not a clause here.
3. **Parallel pull requests collide on `CHANGELOG.md`**, every one inserting at the top of the same
   section — twice on 2026-10-07. Per-change changelog files would end it, and are STD-0005's to
   decide.
4. **Research logs are written into whichever checkout a session started in** and are stranded
   there when it is not the one that gets committed — the 92 lines above. RFC-0027's capture
   should commit with the session's own work.

## Related

- ADR-0004 — `forge audit`, required and unbuilt
- ADR-0001 — vendored knowledge and `forge knowledge sync`
- ADR-0025 · ADR-0043 — the checkpoint hook, which is why nothing was lost
- ADR-0059 — a warning no channel carries is not a warning
- ADR-0124 — CI on the estate's own runners
- RFC-0027 — research captured where it happens
- STD-0013 — the SOP every tool carries
- RUN-0001 — the workspace layout
