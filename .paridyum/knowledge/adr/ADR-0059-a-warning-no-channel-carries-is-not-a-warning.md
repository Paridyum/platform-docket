---
id: ADR-0059
title: A warning no channel carries is not a warning, so every gate's warnings are surfaced where they are read
type: adr
status: accepted
created: 2026-08-31
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [ai-agents, claude-code, hooks, plugins, enforcement, governance, adr-0007, adr-0014, adr-0025, adr-0027, rfc-0027]
breaking: false
enforced_by:
  - check: repo.gate_warnings_are_surfaced
    severity: error
    rationale: >
      This IS a repository property rather than machine state, which is why it is an error where
      ADR-0007's installed-check is a warning: whether a script discards a gate's output is a fact
      about a tracked file, readable in CI, and fixable by whoever the check fails. A hook that
      routes a warning to /dev/null is indistinguishable from one that produced none, which is the
      ADR-0022 shape — configured, silent, and identical to working.
---

# ADR-0059 — A warning no channel carries is not a warning

## Status

**ACCEPTED** as of 2026-08-31. `accepted` rather than `locked` until
`repo.gate_warnings_are_surfaced` has run against something other than this repository; ADR-0002
reserves `locked` for a rule CI mechanically rejects, and §5b of the new-decision procedure says
to lock in a separate commit.

**Amended 2026-10-08 by ADR-0134 — decision 4 only.** Its rule stands: a harness change advances
the plugin's version. Its means changes from discipline to a gate, checked once per pull request
(`pr.harness_change_advances_plugin_version`), because discipline did not hold — `0.2.0` carried
two weeks of hook changes to no workstation, and `marketplace.json` said `0.1.0` throughout. The
rejected alternative below is left as written: it was about a gate on every *commit*, and for that
gate it is still right.

## Context

RFC-0027 was written on 2026-08-18 to capture research provenance at the moment of the fetch. On
2026-08-31 it was found never to have run: the installed plugin cache
(`~/.claude/plugins/cache/paridyum/paridyum-platform/0.1.0`, pinned to commit `3bfcfe8` of
2026-08-15) contained six hook scripts and no `log-web-sources.sh`, and zero `WebSearch` matchers.
`research/` was empty because nothing was capturing, not because nothing was read.

**The first hypothesis was that a check was missing. It was not.**
`repo.plugin_snapshot_matches_head` was declared on 2026-08-07, is implemented at
[`scripts/check-repo.sh`](../scripts/check-repo.sh), and is correct. Run by hand it says exactly
the right thing, with the exact remedy:

> `!` the installed plugin is pinned at `3bfcfe815fb1` and 5 harness file(s) have changed since —
> those fixes are NOT running until `/plugin update paridyum-platform@paridyum` (ADR-0027)

**It had been saying that for twelve days into two closed pipes.**

1. **The checkpoint hook discards it.** `run_gate` in `checkpoint-session.sh` runs each gate as
   `"$@" >/dev/null 2>&1` and records `pass` or `FAIL` from the exit code alone. `check-repo.sh`
   exits 0 on a warning — deliberately, and correctly — so every checkpoint for twelve days
   recorded `repo: pass` over a warning that named the failure and its fix.

2. **The pre-commit hook discards it too.** `"$R/scripts/check-repo.sh" >/dev/null || exit 1`.
   Same shape, same result: errors block, warnings evaporate.

**Every severity below `error` was therefore unreachable from inside the harness.** That is not a
property of this one warning. `repo.paridyum_plugin_is_installed`, `repo.precommit_hook_installed`
and RFC-0020's uncited-source warning are all `warn` for reasons each artifact argues carefully —
CI cannot satisfy machine state, a fresh clone legitimately lacks a git hook, most fetches are
dead ends. **Those arguments chose `warn` on the premise that a warning is read by someone. The
premise was false, and had been since the checkpoint hook was written.**

There is a sharper version of the same defect one file over. `session-context.sh` prints, at every
session start:

> **Research capture** — Armed via `PARIDYUM_KNOWLEDGE_ROOT`: web fetches log to `…/research/`

It reports on a variable, not on a hook. Had that version been installed it would have announced
capture as armed while the capture hook was absent from the same cache — an assurance derived from
configuration, which is precisely the intent-versus-installation conflation ADR-0007's own note
names and `repo.paridyum_plugin_is_installed` exists to keep apart.

## Decision

**1. A gate's warnings are captured and surfaced, never discarded.** `run_gate` records `pass`,
`pass (N warning)` or `FAIL`, and the warning text goes into the checkpoint commit message beside
the gate results it already records. `git log --grep 'warning'` becomes a list, which ADR-0025
already argues is a better artifact than a tree nobody saved.

**2. The pre-commit hook stops sending gate output to `/dev/null`.** Errors already block; the
redirect bought quiet and cost the only channel the warnings had.

**3. The session summary reports the harness against the machine, never against a variable.**
`session-context.sh` prints the stale-plugin condition at session start, where the operator is
looking, and states research capture as *armed* only when `log-web-sources.sh` is present in the
installed plugin — not when an environment variable is set.

**4. A change to the plugin's hook set bumps `version` in `plugin.json`.** The cache is keyed by
version. `0.1.0` has now covered at least three distinct hook sets — the 2026-08-15 cache, the
2026-08-19 marketplace checkout, and today's tree — so a version that never moves is a cache that
never refetches. This is discipline rather than a gate, for the reason under rejected alternatives.

## Consequences

**Accepted costs**

1. **Checkpoint commit messages get longer**, on every session, by the current warning count. That
   is the cost of the channel existing, and it is the same bargain ADR-0011 made for the ledger.

2. **A surfaced warning can still be ignored** — by a human now rather than by a redirect. That is
   a genuinely weaker guarantee than an error and it is the one the `warn` severities were
   arguing for all along. **This ADR restores the premise those arguments assumed; it does not
   improve on it.**

3. **Warning volume becomes a new failure mode.** RFC-0020 consequence 3 and RFC-0027 consequence 3
   both record it: past a handful, the aggregate-plus-baseline treatment applies. Nothing here
   measures that yet, and this is where it will show up first.

4. **Decision 4 is unenforced, and stated as discipline rather than dressed as a rule.** ADR-0030
   requires every binding rule to name its verification; this one names none, so it is not binding.
   Recorded rather than quietly upgraded.

**Gained**

- Every `warn` severity in `check-registry.yaml` becomes reachable. That is roughly a dozen checks
  whose chosen severity was, until today, equivalent to not running them.
- The twelve-day silence has a named cause, so the next empty `research/` directory has a second
  hypothesis and a one-line command that tests it.

**Rejected alternatives**

- **A new check comparing the installed hook set to the source.** The first draft of this ADR, and
  it was wrong: `repo.plugin_snapshot_matches_head` already does this and already worked. Adding a
  second check for a defect an existing check had correctly reported would have hidden the actual
  defect — the discarded output — behind a plausible fix.

- **Raise the stale-plugin warning to `error`.** Fails every CI run, which has no `~/.claude` and
  can never satisfy it. `repo.paridyum_plugin_is_installed`'s note settles this and the argument is
  unchanged.

- **Fail a commit that touches `plugins/` without bumping `version`.** Enforces decision 4
  directly and would fire for the wrong reason on nearly every session: ADR-0025 checkpoints with
  `git add -A`, so reformatting a skill would demand a bump. A gate that fires when nobody did
  anything wrong is suppressed within a month — RFC-0020's argument, which RFC-0027 reuses.

- **Leave `run_gate` alone and read the gates by hand.** What has been happening. It found this
  defect on the day someone thought to ask, twelve days after it started.

## Related

- **ADR-0025** — the checkpoint whose `run_gate` this changes, and whose commit message is the channel
- **ADR-0027** — hooks have a second channel; `repo.plugin_snapshot_matches_head` reported this correctly
- **ADR-0007** — intent versus installation; decision 3 applies its note to the session summary
- **ADR-0014** — declared is not implemented; here, implemented is not *read*
- **ADR-0002** — a lock is an executable check, which is why decision 4 is not one
- **RFC-0027** — the mechanism that had never run, and the reason this was found
- **REQ-0006** — governance of the governance, whose scope this sits inside
