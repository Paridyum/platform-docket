---
id: REQ-0007
title: An agent's authority is scoped, its model pinned, and its actions auditable
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [ai-agents, authority, model-pinning, prompt-injection, audit]
breaking: false
trigger: >
  FIRED 2026-08-08. Clause 3: RFC-0003 and RFC-0008 were both written from fetched
  external sources — CVE records, NVD rejection status, release notes — and one corrected a CVE
  id inside a binding artifact. This requirement's own deferral rationale said item 4 "needs an
  agent consuming external content, which today it does not"; that sentence was false when read.
  Clause 1 is arguably fired too: the ADR-0025 Stop hook commits and pushes to `main` on every
  response, and RFC-0011 establishes that `main` is a branch nobody reviews.
  ORIGINAL: An agent can merge or deploy without a human in the loop, OR a second agent identity
  operates in the organisation, OR an agent processes content from outside the
  organisation.
review_date: 2026-11-01
enforced_by: []
---

# REQ-0007 — An agent's authority is scoped, its model pinned, and its actions auditable

## Status

**DEFERRED** as of 2026-08-07, and this is the **closest of all the deferred requirements
to being current** — agents are the primary authors here today. It is deferred on a narrow
and checkable fact: an agent presently cannot merge or deploy anything, because there is
no merge gate to bypass and nothing to deploy to.

## Requirement

**An agent shall act within an explicitly scoped authority, on a pinned model version,
with its actions attributable and auditable — and the boundary at which a human must
intervene shall be stated rather than assumed.**

## What already exists

More than for any other deferred requirement, which is why the scope here is narrow:

| Concern | Covered by |
| --- | --- |
| An AI must never be the commit Author | C-0003, `git.no_ai_authored_as_human`, verified |
| Attribution of AI contribution | C-0003, `Co-Authored-By` trailer |
| Dangerous commands escalate to a human | C-0004, `guard-commands.sh`, verified |
| Generated code cannot be hand-edited | C-0004, `agent.generated_code_immutable` |
| Governance files escalate | C-0004 + the guard, verified |
| Locked decisions cannot be edited | ADR-0002/0014, hook + gate, verified |
| Creating a service escalates | RFC-0002 guard, verified |
| Prompts and skills are version-controlled and reviewed | ADR-0007, they live in `plugins/` |

The enforcement ladder in ADR-0007 is the substance of agent governance and it exists.

## Numbered items — the gaps

1. **Authority scoping by risk.** What an agent may read, write, merge and deploy without a
   human, expressed per tier. Today the answer is implicit: everything except what a hook
   escalates. That is an allowlist expressed as a denylist, which is the wrong direction and
   only works while the denylist is complete.
2. **Model version pinning and change control.** A silent model upgrade changes behaviour
   across every session with no diff, no review and no announcement. Nothing here records
   which model produced which work, so a behavioural regression would be undiagnosable.
   **This is the item most likely to cause a confusing incident.**
3. **AI must not be the sole approver of AI work in critical paths.** An agent-authored
   change approved only by an agent should not be mergeable for Tier 0–1. Not enforceable
   today — GitHub Free has no required reviews on private repositories (ADR-0004) — which
   is precisely why it must be written down now and switched on with the plan.
4. **Prompt injection and untrusted content.** Tool output, fetched pages, issue text and
   dependency READMEs are untrusted input. An agent that treats a fetched instruction as an
   instruction is the whole attack. Becomes urgent the moment an agent reads anything
   originating outside the organisation.
5. **Agent action audit log.** Distinct from git history: what was attempted and denied, not
   only what succeeded. The hooks already make those decisions and do not record them, so
   the most security-relevant events leave no trace.
6. **Token and cost budgets per agent**, with rate limits. See REQ-0004 §13.
7. **Evals for AI-facing features**, if any are ever shipped to users — model cards,
   red-teaming, bias and safety evaluation, human-oversight and transparency obligations.
   Entirely hypothetical today; recorded so it is not discovered during a launch.

## Why this is deferred rather than accepted

Items 1 and 3 need tiers (REQ-0004 §1) and a merge gate, neither of which exists. Item 4
needs an agent consuming external content, which today it does not. Item 7 needs a product.

Item 2 is the uncomfortable one — it is actionable now and is deferred anyway, because
recording a model version is only useful alongside a way to correlate it with an outcome,
and that is item 5. Building half of the pair produces a log nobody reads.

## Related

- ADR-0007 — the enforcement ladder; the existing substance of agent governance
- C-0003 — identity and attribution, enforced and verified
- C-0004 — protected operations, enforced and verified
- ADR-0004 — the plan limits that make item 3 unenforceable today
- REQ-0004 — risk tiering, which items 1 and 3 both depend on
- REQ-0006 — governance of the governance, of which this is the agent-shaped instance
