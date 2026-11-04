---
id: ADR-0134
title: A harness change advances the plugin version, and the pull request is where that is checked
type: adr
status: accepted
created: 2026-10-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["knowledge"]
tags: [plugin, harness, version, gate, adr-0059, adr-0007, adr-0027, rfc-0060, check-pr]
breaking: false
enforced_by:
  - check: pr.harness_change_advances_plugin_version
    severity: error
    rationale: >
      Claude Code refreshes an installed plugin only when its version changes, so a harness change
      merged without a bump reaches no workstation. Measured per pull request, not per commit —
      the difference ADR-0059's objection turns on.
  - check: knowledge.marketplace_agrees_with_plugin_version
    severity: error
    rationale: >
      Two declarations of one plugin's version are two answers to whether an installed copy is
      stale; they disagreed for two weeks.
---

# ADR-0134 — A harness change advances the plugin version, and the pull request is where that is checked

## Status

**ACCEPTED** 2026-10-08, by the operator under ADR-0087's personal authority.

**Amends ADR-0059 decision 4** — its rule stands, its means changes — and records a process
failure honestly: the gate this ADR adopts was written and merged (#49) **before** ADR-0059 was
read, although ADR-0059 had rejected a gate for this rule. The operator chose to record the gate
on its evidence rather than revert it; this artifact is that record, written after the code it
justifies, which is the order ADR-0013 exists to prevent.

## Context

**ADR-0059 decision 4:** *"A change to the plugin's hook set bumps `version` in `plugin.json`. The
cache is keyed by version… This is discipline rather than a gate."* Its rejected alternative:
*"Fail a commit that touches `plugins/` without bumping `version`… would fire for the wrong reason
on nearly every session: ADR-0025 checkpoints with `git add -A`, so reformatting a skill would
demand a bump. A gate that fires when nobody did anything wrong is suppressed within a month."*

**Discipline did not hold.** `plugin.json` stayed `0.2.0` from 2026-09-24 through every hook change
after it — the SessionStart listing of accepted ADRs and two fixes merged on 2026-10-07 among them —
and `.claude-plugin/marketplace.json` declared `0.1.0` throughout. Claude Code compares versions,
not commits, so `claude plugin update` answered *"already at the latest version"* while the cached
hooks held none of the changes and `installed_plugins.json` recorded a newer commit. Every
workstation ran a harness older than `main`, and nothing said so. #47 released `0.3.0` by hand.

## Decision

1. **ADR-0059 decision 4's rule stands:** a change to the harness advances the plugin's version.
2. **It is checked once per pull request, against the merge base** —
   `pr.harness_change_advances_plugin_version` in `check-pr.sh`. The harness is the plugin's
   `hooks/`, `skills/`, `agents/` and `commands/`; a README beside them is not.
3. **The two declarations of the version agree** — `knowledge.marketplace_agrees_with_plugin_version`
   in `check-knowledge.sh`.
4. **ADR-0059's objection is answered, not dismissed.** It was to a gate on every *commit*, which
   the checkpoint hook makes on every session. A pull request is one review of one change; a bump
   it demands is one line in that change, and it demands it exactly when the installed copy would
   otherwise differ from the repository — the condition `repo.plugin_snapshot_matches_head`
   (ADR-0027) already warns about after the fact.

## Consequences

**Accepted costs**

1. **A pull request that only reformats a skill must still bump the version.** ADR-0059's cost,
   moved from every session to every such pull request, and accepted at that price.
2. **Versions move often**, and carry no meaning beyond "the harness changed". A version is a cache
   key here; semantic versioning's promise to consumers is not one this plugin can make.
3. **The gate runs in CI and in a local `check-pr.sh`.** With the CI runners offline, it binds only
   when someone runs it — the same residue every gate here carries.

**Gained**

- A merged harness change reaches every workstation on its next update, and an installed copy can
  be trusted to be the version it says.

**Rejected alternatives**

**Discipline, as ADR-0059 chose.** Measured above: two weeks, and nothing noticed.

**A gate per commit**, ADR-0059's rejected form. Its objection still holds for it.

**A bot that bumps the version after merge.** It needs a token with write access to `main` and a CI
runner, and it would release changes nobody reviewed as a release — the version change would be
the one line no person read.

## Related

- ADR-0059 — decision 4, amended here; its other decisions are untouched
- ADR-0007 — the harness distributed as a versioned plugin
- ADR-0027 — `repo.plugin_snapshot_matches_head`, the after-the-fact warning
- ADR-0013 — the design-before-code order this artifact was written out of
- RFC-0060 — the sweep that found the stale harness
