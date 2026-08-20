---
name: design-change
description: Run the Paridyum change lifecycle before writing code. Use when starting any non-trivial change, when a change touches an API, schema, dependency, deployment config, or new module, or when asked to design before implementing.
---

# Design a change before implementing it

Walks ADR-0013's lifecycle. Argument, if given, is the change to design: `$ARGUMENTS`

The purpose is not the document. It is the **forced specificity** — writing the design
makes assumptions explicit while they are still cheap to change. An implementation
that works acquires momentum, and rejecting it later costs more than accepting it.

## 0. Is this even significant?

Run the check rather than guessing — significance is computed from the diff, not
self-declared, precisely because the person who wants to start coding is not a
neutral judge:

```sh
scripts/check-significance.sh origin/main
```

**Not significant** → go straight to implementation. Most work is not significant, and
designing everything is how a process gets abandoned. Bug fixes, refactors inside a
module, tests, docs, and locked-stack dependency bumps all proceed directly.

**Significant** → continue below.

## 1. Define the problem

Write the problem **without a solution in it**. If the statement contains the
approach, the design has already been decided and the rest is theatre.

- What is broken or missing, observably?
- Who experiences it, and how often?
- What happens if nothing is done?
- How will you know it is fixed?

## 2. Retrieve before designing

Non-negotiable, and the step most often skipped:

```sh
grep -rl "<topic>" constitution/ adr/ standards/ anti-patterns/
```

- Does a locked artifact already decide part of this? Follow it.
- Does an anti-pattern forbid the obvious approach?
- Does an existing pattern cover it?

A design that contradicts a locked artifact is not a design, it is a supersession —
and it needs the supersession process (C-0004), not a new RFC pretending the
constraint is absent.

## 3. Write the RFC

`rfc/RFC-00NN-<slug>.md`, next unused number, frontmatter per `knowledge.schema.json`,
`status: proposed`.

```markdown
## Problem
Observable, no solution in it.

## Constraints
Locked artifacts that bind this. Cite ids.

## Proposal
The design. Enough that someone else could implement it.

## Alternatives considered
At least two, each with why it lost.

## Consequences
What this makes worse. Required.

## Rollout
Migration, backwards compatibility, how it is undone.

## Open questions
What you do not know yet. An honest list here is worth more than a confident guess.
```

**The alternatives section is the one that matters.** A design with no rejected
alternatives has not been designed, only described — and it is checked
(`rfc.has_alternatives_section`). It is also the section future readers actually use,
because it answers "did they consider X?" without a meeting.

## 4. Get it reviewed, then commit it separately

Set `status: accepted` only after a human has reviewed it. **Land the RFC in its own
commit or PR, before the implementation.**

`pr.design_artifact_predates_implementation` compares timestamps. An RFC committed
alongside the code it justifies is a rationalisation — it carries the authority of a
design while having done none of the thinking. The check warns rather than blocks,
because exploratory work that discovers the design and then writes it up is
legitimate; what must not happen is implementation-first passed off as design-first.

## 5. Implement in increments

Each PR independently reviewable and independently revertable. No line-count rule
(STD-0004) — the test is whether a reviewer can hold it in their head, and whether
reverting it alone leaves the system working.

Reference the RFC in every PR body: `Implements RFC-00NN`.

## 6. Test, document, release

- **STD-0001** — 80% patch coverage; integration tests against real containers, never
  a mocked database.
- **STD-0002** — README, ARCHITECTURE, API, RUNBOOK as applicable. Doc examples
  compile, which is what stops documentation drifting.
- **STD-0004** — every new module carries a `//!` responsibility sentence. If it needs
  "and", you found the split.
- **STD-0005** — a changelog entry **if the change is notable to a reader**. Not every
  change is, and adding a line to satisfy a check is how a changelog fills with noise.
- **ADR-0010** — release through the Release PR.

## 7. Does this become an ADR?

An RFC describing how to build one thing is finished when it is built. An RFC whose
outcome **binds future work** graduates to an ADR — and if it can be mechanically
enforced, it gets `enforced_by` and `status: locked` (ADR-0002). If it cannot be
checked, it stays `accepted`. That keeps `locked` meaning what it says.
