---
id: ADR-0007
title: The agent harness is a plugin; knowledge content is vendored
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [ai-agents, claude-code, hooks, plugins, enforcement, governance]
breaking: true
enforced_by:
  - check: repo.declares_paridyum_marketplace
    severity: error
    args: { path: ".claude/settings.json" }
    rationale: A repo that does not declare the marketplace gets no enforcement hooks.
  - check: repo.enables_paridyum_platform_plugin
    severity: error
  - check: repo.claude_md_under_line_budget
    severity: warn
    args: { max_lines: 200 }
    rationale: Root CLAUDE.md is loaded on every session by every engineer; bloat is a per-session tax.
  - check: repo.no_vendored_harness
    severity: error
    rationale: Hooks and skills must come from the versioned plugin, never copied per-repo.
---

# ADR-0007 — The agent harness is a plugin; knowledge content is vendored

## Status

**LOCKED** as of 2026-08-06.

## Context

ADR-0001 established that knowledge is vendored into every repository so an agent
working in one checkout can read its rules offline. ADR-0002 established that a lock
must be an executable check. Both left a gap this ADR closes.

**The enforcement gap.** ADR-0002 assumed the strongest lock was a required CI status
check. ADR-0004 then established the organisation is on GitHub Free, where private
repositories cannot require a status check before merge. The lock was reduced to a
red build nobody is forced to heed.

There is a stronger mechanism, and it sits earlier in the pipeline. Claude Code
`PreToolUse` hooks run before a tool call executes and can deny it. A hook that
denies an edit **cannot be reasoned around by the agent, does not decay after context
compaction, and does not depend on GitHub's billing plan.** For the specific failure
this platform was designed around — an agent drifting from a rule it read two hours
ago, or a parallel agent that never read it — a hook is not a weaker substitute for
CI. It is the better instrument, because it prevents the violation instead of
reporting it.

**The distribution gap.** Vendoring works for prose. It is wrong for executable
harness config. Hooks and skills copied into forty repositories mean forty edits to
fix one bug, and forty opportunities for versions to diverge. Claude Code's plugin
system exists for exactly this: a versioned bundle distributed from a marketplace,
updated in one place.

## Decision

**Split by artifact kind.**

| Kind | Examples | Distribution | Why |
| ---- | -------- | ------------ | --- |
| **Content** | constitution, ADRs, standards, glossary | Vendored to `.paridyum/knowledge/` (ADR-0001) | Must be readable offline from a single checkout |
| **Harness** | hooks, skills, subagents, settings | Plugin `paridyum-platform` from the `paridyum` marketplace | Executable; needs one version and one place to fix |

The marketplace is hosted in `Paridyum/knowledge` at `.claude-plugin/marketplace.json`,
with the plugin under `plugins/paridyum-platform/`. Every repository declares it in
`.claude/settings.json` via `extraKnownMarketplaces` and `enabledPlugins`, so a
contributor who trusts the folder is prompted to install it — the harness arrives with
the clone rather than by instruction.

### The enforcement ladder

Four layers, strongest first:

1. **`PreToolUse` hook** — denies the edit before it happens. Cannot be reasoned
   around, survives compaction, plan-independent. Covers all work done through
   Claude Code, which is most of it.
2. **Pre-commit hook** — `check-knowledge.sh --staged`. Catches edits made outside
   Claude Code. Bypassable with `--no-verify`.
3. **CI** — runs on every PR and push. Reports red; on GitHub Free it cannot block
   the merge (ADR-0004).
4. **`forge audit`** — org-wide sweep. The only layer that catches a violation that
   already reached `main`.

Layer 1 is new and is the practical answer to ADR-0004's gap. It does not make CI
redundant: CI covers commits from any source, and layer 1 covers only sessions with
the plugin enabled.

### The `SessionStart` hook

Retrieval, not just prevention. It detects the repository class from its name,
locates the knowledge base, and injects the ids, titles and paths of the locked
artifacts whose `scope` matches — before the first prompt. It prints an **index, not
bodies**: the agent reads selectively. Dumping the full knowledge base would recreate
the context bloat this design avoids.

### Context budget

Root `CLAUDE.md` stays under 200 lines and holds only what must be true every
session. Procedures go in skills, which load their body only when invoked. This is a
per-session, per-engineer cost, so it is checked rather than trusted.

## Consequences

**Accepted costs**

- Enforcement now depends on the plugin being installed and enabled. A contributor
  who declines the install gets no layer-1 protection — which is why layers 2–4
  remain and why `repo.enables_paridyum_platform_plugin` is checked.
- Hooks execute on every matching tool call, so they must stay fast and must never
  wedge a session. The guard exits 0 on internal error and reports the degradation
  rather than failing closed.
- Two distribution mechanisms to understand instead of one.
- Layer 1 is Claude Code-specific. A different agent, or a human in an editor, is
  covered only by layers 2–4.

**Gained**

- The strongest available lock no longer depends on the GitHub plan, which
  substantially repairs ADR-0004's gap without spending money.
- Harness bugs are fixed once, at a version, rather than in every repository.
- Agents start with the applicable rules present rather than needing to know to look.

**Rejected alternatives**

- *Vendor hooks and skills per repository* — consistent with ADR-0001's vendoring,
  but wrong for executable config: N copies, N divergences, N edits per fix.
- *Put every rule in `CLAUDE.md`* — instructions are advisory and degrade under
  context pressure. That is the failure this platform exists to prevent.
- *Rely on CI alone* — cannot block merges on the current plan, and reports the
  violation after it is written rather than preventing it.
