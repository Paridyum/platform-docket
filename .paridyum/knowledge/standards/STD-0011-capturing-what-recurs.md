---
id: STD-0011
title: Knowledge that will be needed again is written where it will be retrieved
type: standard
status: accepted
created: 2026-08-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [knowledge-capture, documentation, runbooks, agents, context-loss]
breaking: false
enforced_by: []
---

# STD-0011 — Capturing what recurs

## Status

**`accepted`, deliberately not `locked`, and `enforced_by` is deliberately empty.** The central
judgement — *will this be needed again?* — cannot be mechanically made, which is ANTI-0004's and
C-0006's reason at a different setting. ADR-0002's bar for `locked` is a check that rejects the
violation, and **nothing rejects a fact that was never written.**

C-0006 states the temptation and refuses it in the same words this artifact needs: *"`enforced_by`
is empty because naming a proxy check would be the false confidence ADR-0002 exists to prevent."*
A heuristic over session transcripts is a proxy. It is worth building — see the enforcement
section — and it is not this rule.

## The rule

> **If something learned in the course of work will be needed again, write it into the knowledge
> base before the session that produced it ends.**

Not into a chat log, not into a commit message, not into someone's memory. Into the artifact a
person or an agent would actually retrieve it from.

## Why this is a rule rather than a habit

CLAUDE.md states the rule that generates the others: **never infer architecture, retrieve it.**
That rule is worth nothing unless something was captured — retrieval presupposes capture, and
nothing in this repository previously required it. The two rules are one mechanism with two
halves, and only one half was written down.

The asymmetry that makes this expensive:

- **A transcript is not a knowledge base.** Context compacts, sessions end, and an agent starting
  tomorrow begins with what is in `knowledge/` and nothing else. A fact discovered at 2pm and not
  written by 4pm is gone in every practical sense, even though a log of it technically exists.
- **The cost of capture is minutes; the cost of rediscovery is the original debugging, again,
  by someone who does not know it was ever solved.** They will not know to search for it, because
  the thing they lack is precisely the knowledge that it is knowable.
- **An agent that cannot retrieve will infer.** C-0006 and `bets/` exist on exactly this argument
  — *"the absence of a written answer is not neutral here; it is an instruction to guess."*

## What counts as "will be needed again"

Mechanical triggers, so no one has to exercise taste under deadline. Any one of these:

1. **Documentation was wrong or incomplete**, and you found the right answer. The next reader
   follows the same wrong documentation.
2. **A cause took more than a few minutes to find.** The duration is the signal — it means the
   answer was not derivable from what was written.
3. **A workaround is now load-bearing.** Anything that works for a reason that is not obvious from
   reading the code.
4. **A decision was made in conversation.** ADR-0011's hook already catches the loudest of these;
   this rule is why that hook exists.
5. **A fact about an external system was verified.** Version behaviour, an API's actual response,
   a vendor's real constraint. RFC-0020 governs how it is cited; this governs *that* it is.
6. **A named thing was chosen** — a code, a path, a threshold, a convention.

## What does not

Say so plainly, because a rule that appears to demand everything gets ignored entirely.

- A fact already recorded somewhere retrievable. Link it; do not copy it. A second copy is a second
  thing to drift.
- Anything derivable by reading the code or the git history. STD-0002 covers what a repository
  documents about itself.
- A genuinely one-off: an incident with no recurrence path, a decision that expired the same day.
- Working notes for a task in progress. Capture on completion, not continuously.

## Where it goes

Routing, because "write it down" without a destination produces prose nothing scans.

| What you learned | Where it belongs |
| --- | --- |
| How to do an operational procedure | `runbooks/` |
| A decision, already taken | `adr/` |
| A question not yet decided | `rfc/` |
| A rule about how things are written | `standards/` |
| A shape that must not be built | `anti-patterns/` |
| A term used inconsistently | `glossary/` |
| A commercial wager | `bets/` |
| Something true only of one repository | that repository's own documentation, per STD-0002 |
| A list that will be extended | a registry, never prose — C-0007: *the grammar is constitutional, the list is data* |

**Correcting an existing artifact beats writing a new one.** If a runbook was wrong, fix the
runbook. A new artifact recording that an old artifact is wrong leaves both, and readers find the
wrong one first. ADR-0031's errata mechanism exists for the locked case.

## The worked example, which is why this standard exists

On 2026-08-15, configuring commit signing per RUN-0001 produced two facts by debugging:

- The documented agent socket path was **wrong for the sandboxed build** — the file existed and
  refused connections, while the live socket sat inside the app container.
- The socket export belongs in `~/.zshenv`, **not `~/.zshrc`**, because ADR-0025's checkpoint hook
  runs a non-interactive shell. In `.zshrc` it works every time a human tests it and fails every
  time the hook fires.

Neither was inferable. The second is worse than a missing fact: it is a configuration that passes
every manual test and silently breaks the mechanism that exists to prevent work loss. Both were
written into RUN-0001, and the same session found that `bootstrap.sh` warned on the now-recommended
configuration — a check reporting a defect where none exists, which trains people to ignore it.

Three discoveries, one session, none of them derivable, all of them certain to recur on the next
workstation. That is the ordinary case, not an unusual one.

## Enforcement, and why nothing is declared yet

The mechanism worth building is an extension of ADR-0011's `capture-open-loops` hook from
decision-shaped statements to discovery-shaped ones — *"it turns out"*, *"the actual path is"*,
*"that fails because"* — surfacing them in `DECISION-DEBT.md` when the session changed no artifact.
It should be `warn` and stay `warn`: an error-severity heuristic over natural language is routed
around within a week, and a check people route around is worse than none because it also carries
the appearance of one.

**It is not declared in `check-registry.yaml`, and the reason is a registry defect found while
trying to declare it on 2026-08-15.** `owner` is the *only* signal the gate uses to decide
implemented-versus-owed: `tool-forge` means owed, and **every other value counts as implemented**.
There is no way to express *owed by `paridyum-platform`*. Declaring this check against the plugin
therefore moved it straight into the implemented column while nothing had been written — a check
reporting success while enforcing nothing, which is precisely the 2026-08-06 audit finding the
registry was hardened against.

So the choice was to declare it dishonestly or not at all, and not at all is correct until the
registry can carry the distinction. **This paragraph is the record that the check is wanted**, and
it is itself an instance of the rule above.

**And the honest limit, for whenever it is built.** It detects an omission; it cannot detect a
fact. A discovery phrased flatly will not be caught, and nothing verifies that what was written is
correct or complete — the same limit RFC-0022 D2a records for a commitment that resists direct
testing. It raises the cost of quietly skipping; it does not make skipping impossible.

## Related

- `CLAUDE.md` — *never infer architecture, retrieve it*, of which this is the other half
- ADR-0011 — the `capture-open-loops` hook this extends
- ADR-0031 — errata, for correcting a locked artifact rather than superseding it
- STD-0002 — what a repository documents about itself
- RFC-0020 — how an external fact is cited once captured
- C-0007 — a list that will be extended is registry data, never prose
- ADR-0002 — why this is `accepted` rather than `locked`
