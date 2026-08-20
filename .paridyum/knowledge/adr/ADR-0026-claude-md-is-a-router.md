---
id: ADR-0026
title: CLAUDE.md is a router, not a manual, and every route must resolve
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [agent-harness, context, entrypoint, navigation, duplication]
breaking: false
enforced_by:
  - check: repo.claude_md_links_resolve
    severity: error
    rationale: >
      A router with a dangling edge is worse than no router: the agent follows the link, finds
      nothing, and falls back to inferring — the one failure the entrypoint exists to prevent.
  - check: repo.claude_md_under_line_budget
    severity: error
    rationale: >
      A router that grows past its budget has stopped being a router, and the growth is always
      something inlined that belonged behind a link.
---

# ADR-0026 — CLAUDE.md is a router, not a manual

## Status

**LOCKED** as of 2026-08-07, with two implemented checks.

## Context

`CLAUDE.md` had grown to 240 lines and was read in full at the start of every session, so its
length was a tax on every task. More seriously, most of that length was **duplication of
artifacts that already existed**:

| Section | Already lived in |
| --- | --- |
| Organising code | STD-0004 |
| Writing a new knowledge artifact | the `new-decision` skill, `knowledge.schema.json` |
| Before you write code — the lifecycle | ADR-0013, the `design-change` skill |
| Spec-driven work and PR size | STD-0007, the `slice-requirement` skill |
| Commercial context | C-0006, `bets/` |
| **Decisions already locked** (33 lines) | the artifacts, **and** `session-context.sh` |

The last row was the worst of them, and in a way that mattered rather than merely being untidy.
It was a **hand-maintained table of locked artifacts** — the fifth instance of the pattern this
repository has now been caught by five times (check-registry counts, the C-0002 class table,
artifact types, README counts, this). Four rows had been added to it in a single day. And it was
**strictly worse than the mechanism that already existed**: `session-context.sh` emits the same
list at `SessionStart` *filtered by the repository's class*, so the table showed an agent in a
`web-*` repository a list of Rust ADRs that do not bind it.

Duplication in this repository is not a style question. README.md's opening argument is that
knowledge is enforced rather than advisory; a second uncheckable copy of a standard is advisory
by construction, and when the two disagree there is no rule about which wins.

## Decision

**`CLAUDE.md` holds only what an agent needs in order to choose what to read next, plus the rules
whose violation is destructive. Everything else is one hop away, and every route must resolve.**

1. **Inline only if it passes one of two tests.** Either the rule is needed to *interpret the
   graph itself* — precedence, what `locked` means, where to stop and ask — or violating it is
   **destructive and irreversible**: writing a vendored snapshot, editing a locked artifact,
   editing generated output. Nothing else is inline.

2. **One hop, and every target is complete on its own.** No route points at another router. An
   agent that must chase three files to learn one rule will infer instead on the second hop.

3. **Each route states its trigger, not its topic.** The table reads *"if you are about to write a
   migration"*, never *"migrations"*. A link with no trigger condition is a link nobody follows at
   the right moment, which is the same as no link.

4. **Where a rule appears here and in a standard, the standard is the authority.** Stated in the
   file itself, so a future divergence has a resolution rule rather than an argument.

5. **Every relative link and every named skill must exist**, checked by
   `repo.claude_md_links_resolve`.

6. **150-line budget**, down from 250. A budget at twice the actual size constrains nothing, and
   250 was sized for the manual. If it grows past 150 again, something has been inlined that
   belongs behind a link.

7. **No hand-maintained list of artifacts.** `session-context.sh` already emits the constitution
   and the class-filtered locked set at `SessionStart`. A table here would be a third copy of a
   fact that already has two homes, one of which is more accurate.

### The one exception, and why it is stated

The changelog brief stays inline at ~10 lines even though STD-0005 governs it. **STD-0005
requires it here specifically** — Keep a Changelog 2.0.0 recommends the brief be reproduced in
`CLAUDE.md` and STD-0005 adopts that recommendation. Removing it would violate a locked standard.
Recorded as an exception rather than quietly kept, so nobody later "tidies" it away.

## Costs accepted

1. **A hop costs a tool call, and some agents will not take it.** An agent in a hurry, or one with
   a small context budget, may act on the router alone and miss a standard. This is the real cost
   and the mitigation is imperfect: the trigger column is written so that the row itself carries
   the one-line version of the rule (*"operator scope is a grant, never an absence"*), which is
   enough to prevent the worst mistake even unread.

2. **The graph can rot in a way a manual cannot.** A manual is wrong in place and visibly; a
   router is wrong by pointing somewhere that moved. Hence check 5 — but it verifies *existence*,
   not that the target still says what the row claims. A renamed section inside a standard leaves
   the route resolving and lying, and no check can see that.

3. **`repo.claude_md_links_resolve` blocks a locked artifact it cannot be attached to.** The check
   most naturally belongs to ADR-0007, which is `locked` and so cannot gain an `enforced_by` entry
   without being superseded. This ADR exists partly to give the check an attributable home — which
   is a legitimate reason to write an ADR and also, honestly, a slightly circular one.

4. **150 lines is arbitrary and will eventually be wrong.** It is one line to change here, which
   is the point; the failure to avoid is exceptions accumulating against a number nobody revisits.

## Alternatives rejected

**Leave it as a manual and just trim.** Cheapest, and it does not address the duplication —
trimming a second copy of STD-0004 leaves a shorter second copy of STD-0004.

**`@`-import the specialised files.** Claude Code supports `@path` imports in `CLAUDE.md`, which
would keep the source file short. Rejected because imports are resolved **eagerly**: the content
lands in context anyway, so the token cost is unchanged and the only gain is a tidier source. The
goal was lazy traversal, and eager inclusion is its opposite.

**Create a new `.claude/guides/` tree of specialised markdown.** The obvious reading of "split it
into specialised files", and rejected because the specialised files **already exist** — they are
the standards, the constitutions and the skills. Creating a parallel tree would have produced a
third copy of each rule, in a directory with no frontmatter, no status and no checks: precisely
the `docs/` directory README.md explains this repository is not.

**Generate the router from the artifacts.** Tempting, and it would kill the rot in cost 2. Rejected
because the trigger column is editorial judgement — *when* to read a standard is not derivable from
the standard's frontmatter, and a generated table would list topics instead of triggers, which
decision 3 says is the same as no table.

## Consequences

- `CLAUDE.md` is 142 lines, down from 240, with no rule lost: each removed section was verified
  present in the artifact its route points to before removal.
- The `new-decision` skill gained the artifact-registry and `bet` rules that had been living in
  `CLAUDE.md`. Content moved to its home rather than being deleted.
- Two checks, both implemented in `check-repo.sh` and both defect-proven: a dangling path, a named
  skill with no `SKILL.md`, and the budget crossed at 154 lines.
- The hand-maintained locked-decisions table is gone. `session-context.sh` was already doing it
  better.

## Related

- ADR-0007 — the harness, the hooks, and `session-context.sh` which supersedes the deleted table
- ADR-0002 — why an uncheckable second copy of a rule is advisory by construction
- ADR-0014, ADR-0015, ADR-0022, C-0005 — the four prior instances of a hand-maintained list
- STD-0005 — requires the changelog brief inline; the one stated exception
- STD-0007 — the router's largest single route, and the newest
- C-0001 — principle 3, and the reason duplication is a correctness problem here
