---
id: ADR-0002
title: A locked decision is enforced by an executable check, not by prose
type: adr
status: superseded
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: ADR-0014
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [governance, ci, ai-agents, enforcement]
breaking: true
enforced_by:
  - check: knowledge.frontmatter_valid
    severity: error
    rationale: An artifact the gate cannot parse is an artifact the gate cannot enforce.
  - check: knowledge.locked_artifacts_immutable
    severity: error
    rationale: Editing a locked artifact in place erases the audit trail; supersede instead.
  - check: knowledge.check_ids_resolve
    severity: error
    rationale: A typo'd check id would silently enforce nothing, which is worse than no rule.
  - check: repo.ci_runs_knowledge_check
    severity: error
    rationale: The gate must run in every repo, or it binds only the repos that opted in.
  - check: repo.precommit_hook_installed
    severity: warn
---

# ADR-0002 — A locked decision is enforced by an executable check, not by prose

## Status

**LOCKED** as of 2026-08-06.

## Context

Paridyum is built primarily by AI agents working in parallel. The failure mode this
ADR addresses is specific and observed: an agent reads a standard at the start of a
session, works for two hours, and drifts. Not maliciously — the instruction simply
falls out of the effective context. A second agent, started fresh, never saw it at all.

Prose in a repository is *advisory to a system that forgets*. Conversational memory is
worse: it is unversioned, unauditable, and per-session.

The stated goal — "lock decisions so AI cannot forget them" — cannot be met by writing
better documents. It can only be met by making violation **fail loudly and
mechanically**, at a point the agent cannot route around.

## Decision

**A decision is not locked until something executable rejects its violation.**

Concretely:

1. Every artifact with `status: locked` carries an `enforced_by` list naming checks
   implemented in `tool-forge`'s check registry (schema: `knowledge.schema.json`).
2. `forge knowledge check` resolves and runs those checks against a repository.
3. It runs in three places, deliberately redundant:
   - **pre-commit hook** — fastest feedback, catches the agent before the commit exists
   - **CI on every pull request and every push to `main`** — catches `--no-verify`
   - **`forge audit` sweeping the whole org on a schedule** — catches repos that drifted
4. An unknown check id is a **hard failure**, never a skip. A rule that silently
   enforces nothing is more dangerous than an absent rule, because it manufactures
   false confidence.
5. `forge knowledge check --audit` lists every locked artifact with an empty
   `enforced_by`. That list is the enforcement backlog and is expected to be non-empty
   — it is an honest inventory of which decisions are currently only advisory.

### Prose keeps its job

Markdown carries the *why* — context, rejected alternatives, consequences. That is what
future-you and a new agent need in order to reason about whether a decision still
holds. The check carries the *what*. Neither substitutes for the other; a check without
an ADR is a rule nobody can evaluate, and an ADR without a check is a suggestion.

### Not every rule is checkable

"Service boundaries should follow business capabilities" cannot be mechanically
verified. Such rules stay `status: accepted`, not `locked`. **`locked` is reserved for
what can be enforced.** This keeps the word meaningful — when an agent reads LOCKED, it
is reading a claim that something will actually stop it.

## Consequences

**Accepted costs**

- Writing a rule now costs a check implementation, not just a paragraph. This is
  intentional friction: it caps the constitution at rules worth the effort.
- The check registry in `tool-forge` becomes a load-bearing component requiring its own
  tests. A false positive blocks all work across every repository.
- Bootstrapping order is constrained: `forge knowledge check` must exist before the
  locks in this repository mean anything. Until it ships, C-0002 and ADR-0001 through
  ADR-0005 are enforced
  by intent only, and this document says so rather than pretending otherwise.

**Gained**

- Enforcement is identical for a human, an agent, or a future hire. No onboarding gap.
- Parallel agents cannot diverge on anything the gate covers, regardless of what any
  individual session remembered.
- The set of locked-but-unenforced decisions is *visible* rather than assumed.

## Known limitation on the current plan

Under ADR-0004 the organisation is on GitHub Free, where **private repositories cannot
require a status check to pass before merge**. CI will run and report red, but the
merge button stays enabled. Until that changes, the enforceable layers are the
pre-commit hook and the `forge audit` sweep, both bypassable by a determined actor.
This is a real gap in this ADR's guarantee and is tracked in ADR-0004.
