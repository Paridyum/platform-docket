---
id: ADR-0027
title: Hook reachability is checked, and a second channel is authorised as a fallback
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [agent-harness, hooks, enforcement, distribution, durability]
breaking: false
enforced_by:
  - check: repo.hooks_are_reachable
    severity: error
    rationale: >
      ADR-0007's layer 1 and ADR-0025's durability both depend on hooks loading by SOME path.
      Checking that the plugin is declared proved nothing; checking that a path exists does.
  - check: repo.plugin_snapshot_matches_head
    severity: warn
    rationale: >
      The plugin is a pinned snapshot, so a committed hook fix is inactive until /plugin update —
      correct in git, stale on disk, and invisible either way.
---

# ADR-0027 — Hook reachability is checked

## Status

**LOCKED** as of 2026-08-07, with an implemented check.

**Its premise was corrected before it landed, and that is recorded rather than tidied away.**
The first draft asserted that the plugin channel was *unavailable*, on the evidence that
`/plugin marketplace add` returned *"/plugin isn't available in this environment."* That message was
about the interactive command surface; **the registration itself succeeded** — the marketplace was
added, fetched, and the plugin installed at user scope minutes later. The draft was revised rather
than committed-then-superseded, because it had never entered the record and its factual claim was
simply wrong. What survives is the part that was independently true: **reachability was never
checked, and the check is the substance of this decision.**

**This does not supersede ADR-0007.** Its content/harness split, its four-layer ladder and its
reasoning all stand unchanged, and the plugin remains the canonical channel for a consuming
repository. What this adds is a fallback for the case ADR-0007 did not anticipate, recorded as an
ADR rather than applied quietly, because it authorises a distribution path ADR-0007's table does
not list.

## Context

ADR-0007 distributes the harness — hooks, skills, settings — as the `paridyum-platform` plugin,
and says a contributor who trusts the folder *"is prompted to install it, so the harness arrives
with the clone rather than by instruction."*

Three findings on 2026-08-07 undermine the assumption underneath that.

**First, declaring is not installing, and the difference was invisible.** An audit found
`.claude/settings.json` declaring and enabling the plugin while
`~/.claude/plugins/known_marketplaces.json` had never heard of the marketplace. Nothing was
installed. The consequences were total and silent:

- layer 1 had never denied an edit, so locked artifacts and gate scripts were freely editable;
- the `SessionStart` context had never been injected;
- **the ADR-0025 Stop hook had never fired**, so every checkpoint commit that day was invoked by
  hand — a durability rule adopted *because work had been lost* was not running.

`repo.enables_paridyum_platform_plugin` passed throughout, because it reads the config file. A
config file is a statement of intent.

**Second: the install surface can report itself absent while working.** `/plugin marketplace add`
returned *"/plugin isn't available in this environment"* and the registration nonetheless succeeded.
So the failure mode is worse than a missing command — the channel can be *believed* unavailable
while functioning, and believed present while absent. Neither direction is observable from the
repository, which is why the decision below is a check rather than a channel.

**Third: the installed plugin is a pinned snapshot.** `installed_plugins.json` records a
`gitCommitSha`, so a committed hook fix is **not active until `/plugin update`**, silently. A guard
whose code is correct in git and stale on disk is the ADR-0022 failure — configured, silent,
indistinguishable from working — arriving through the distribution layer instead of the config.

## Decision

**The harness may be wired by either channel, and at least one must be present.**

1. **The plugin remains canonical for a consuming repository.** ADR-0007 is unchanged: a
   `svc-*` or `web-*` repository gets the harness from the marketplace, because it does not
   contain the scripts and must not copy them (`repo.no_vendored_harness`, ANTI-0004's shape
   applied to tooling).

2. **A repository that *contains* the hooks MAY register them directly** in its own
   `.claude/settings.json`, pointing at its own paths. This is the source repository only —
   `knowledge` — and it is the same distinction `repo.no_vendored_harness` already makes when it
   reports `n/a` here with *"this repository is the harness source."* The source does not need the
   harness distributed to it; the scripts are already on disk.

   **Not used here while the plugin works.** Both channels live means four Stop hooks per response,
   two of them redundant — the duplication ADR-0007 chose a single channel to avoid. The
   registration was written, measured at double cost, and removed; the authorisation remains for the
   case where the canonical channel is genuinely absent.

3. **`repo.plugin_snapshot_matches_head` warns when the pin is behind.** The pin is the third
   finding above: correct in git, stale on disk, invisible. A warning rather than an error, because
   a snapshot legitimately lags a working tree between commits.

4. **`repo.hooks_are_reachable` requires one of the two.** Not that the plugin is declared —
   that was the check that proved nothing — but that hooks will actually load: either the
   marketplace is installed, or settings.json registers the scripts by path.

5. **Double registration is tolerated, not prevented.** If the plugin is later installed in a
   repository that also registers directly, both fire. Every hook here is safe under that:
   `checkpoint-session.sh` exits immediately on a clean tree and holds an `index.lock`
   pre-flight, and `capture-open-loops.sh` regenerates a file it owns. The cost is a duplicate
   run, which is cheaper than the coordination a mutual exclusion would need.

## Costs accepted

1. **Two channels means two places a hook can be wired, which is the duplication ADR-0007 chose
   the plugin to avoid** — *"executable; needs one version and one place to fix."* That reasoning
   is still correct, and this decision weakens it. The mitigation is narrow scope: only a
   repository that already contains the scripts may use the second channel, so there is still
   exactly one copy of every hook. What is duplicated is the *registration*, not the code.

2. **The direct channel cannot be verified from a script.** Whether a session actually loaded a
   settings.json hook is Claude Code's state, not a file property. `repo.hooks_are_reachable`
   checks that a path exists and is registered, which is necessary and not sufficient — the same
   gap `repo.paridyum_plugin_is_installed` has, one level in.

3. **A settings.json hook only takes effect on a new session.** Registering it does nothing to the
   session that registered it, which is precisely how this ADR's own change will fail to protect
   the session that wrote it.

4. **`.claude/settings.json` is a C-0004 governance file**, so editing it is meant to escalate to
   a human — and it could not, because the guard that would escalate is the thing being wired.
   Circular, unavoidable on first install, and worth stating rather than glossing.

## Alternatives rejected

**Do nothing and rely on the plugin.** This is nearly what the decision does — the plugin *is* the
channel in use. What it cannot do alone is tell anyone when it stops working: it was absent for a
whole session while the configuration read as complete, and its snapshot can fall behind silently.
Relying on it is fine; relying on it *unchecked* is what produced the failure.

**Supersede ADR-0007 entirely.** The honest option, and rejected as disproportionate. ADR-0007 is
mostly about the content/harness split and the enforcement ladder, both of which are correct and
would have to be restated verbatim to preserve them. Supersession is for a decision that was
wrong; ADR-0007's distribution row is incomplete rather than wrong.

**Copy the hooks into each repository.** What `repo.no_vendored_harness` exists to forbid. A
copied hook is a fork that stops receiving fixes — the `guard-locked-decisions.sh` hole ADR-0022
found would live on in every copy.

**Register hooks in the user's `~/.claude/settings.json`.** Works immediately and is invisible to
the repository: a contributor cloning the repo would get none of it, and nothing in git would
record that enforcement depends on a file outside the tree.

## Consequences

- The plugin is installed and is the channel in use. A settings.json `hooks` block was written,
  measured at four Stop hooks per response with two redundant, and removed — decision 2 keeps the
  authorisation without the cost.
- `repo.paridyum_plugin_is_installed` is superseded in function by `repo.hooks_are_reachable`,
  which accepts either channel. It remains as the narrower signal, warning when the canonical
  channel specifically is absent.
- **ADR-0025 becomes enforceable for the first time.** Until this, its Stop hook had never run
  except by hand.
- A consuming repository still has no working channel where `/plugin` is unavailable, and none
  exists yet to be affected. That is an open problem for whoever creates the first one, and it is
  ADR-0004's plan question in a new place.

## Related

- ADR-0007 — the single-channel decision this adds to rather than replaces
- ADR-0025 — the durability rule that was not running, and the reason this could not wait
- ADR-0022 — where a copied hook's staleness was found, which is why copying is still forbidden
- C-0004 — settings.json is a governance file; cost 4 records the circularity
- ADR-0004 — the plan gap that makes local enforcement matter more than it should
