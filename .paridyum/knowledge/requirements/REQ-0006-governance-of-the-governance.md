---
id: REQ-0006
title: Every rule has an owner, an expiry path for exceptions, and evidence it works
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [meta, waivers, ratcheting, risk-tiering, metrics, rule-ownership]
breaking: false
trigger: >
  FIRED 2026-08-08. Clause 1: STD-0007's `pr.diff_within_review_budget` is
  `severity: error` on a locked standard with a 400-line ceiling, and reported `n/a` for four
  commits of 1,651 / 1,637 / 1,271 / 1,061 lines — a bypass invisible in the next run (RFC-0011).
  Clause 3: ADR-0030 landed 2026-08-08 and was applied retroactively.
  Clause 2 has not fired; CODEOWNERS still names one member.
  ORIGINAL: A rule is bypassed and the bypass is not visible in the next run, OR a third person
  works in the organisation, OR a check must be applied to code that predates it.
review_date: 2026-11-01
enforced_by: []
---

# REQ-0006 — Every rule has an owner, an expiry path for exceptions, and evidence it works

## Status

**DEFERRED** as of 2026-08-07, with the **nearest review date of any requirement** —
three months rather than six. This is the item most likely to become urgent without
announcing itself.

## Requirement

**The governance layer shall itself be governed: exceptions shall expire, new rules shall
apply to new code before old, enforcement shall escalate rather than arrive as a wall, and
each rule shall have an owner and evidence it is worth its cost.**

## Numbered items

1. **Waivers with a named owner, a reason and a hard expiry.** Three domain-specific
   waiver mechanisms already exist and work — `.gitleaksignore` (ADR-0017), the `declared`
   check state (ADR-0014), and `deferred` requirements with triggers (ADR-0018). What is
   missing is the general case, and the tell is that **none of the three expires**. A
   waiver with no expiry is a permanent exception written in the language of a temporary
   one.
2. **Ratcheting.** New and changed code must pass; existing code is grandfathered and burnt
   down on a schedule. Nobody fixes 500k lines at once, so a hard global gate on legacy is
   not enforced — it is switched off, and the rule dies with it.
3. **Enforcement gradient.** `warn → soft-fail → hard-fail`, with a stated migration
   window. This repository already does this by instinct — `git.commits_are_signed` and
   `org.branch_protection_reconciled` are `warn` pending a plan change — but the pattern is
   not written down, so each instance is re-argued.
4. **Risk tiering.** Rigour scales with blast radius. See REQ-0004 §1; recorded in both
   because it is simultaneously a runtime concern and the main defence against governance
   being ignored wholesale.
5. **Rule ownership.** Every rule names an owner and a rationale. `check-registry.yaml` has
   `owner`, but that is the *implementing component*, not a person or team accountable for
   whether the rule should exist. Unowned rules are never deleted, only accumulated.
6. **Effectiveness metrics.** DORA — lead time, deploy frequency, change failure rate,
   MTTR — plus review latency, CI pass rate, flake rate and waiver count. Without these
   there is no evidence any of this helps, only the belief that it does.
7. **Standards are versioned and dated, and changing one has a process.** ADR-0013 governs
   changing code; nothing governs changing a standard except supersession, which is
   whole-artifact and therefore too expensive for small changes. This has bitten
   repeatedly — see below.
8. **A single source of truth for policy.** Already true: this repository. Recorded so it
   stays true.
9. **Enforcement must be declared by an artifact.** A rule enforced by CI or a hook that no
   artifact demands is an *unowned* rule — it can be deleted by anyone tidying a workflow
   and nothing will notice, and it is invisible to `--audit`. This is the inverse of the
   failure ADR-0002 names, and equally bad: a decision with no enforcement is at least
   honest about it, while enforcement with no decision looks like nothing at all.
10. **Trigger clustering must be sequenced.** Deferred requirements activate on events, and
    events cluster. Three requirements — REQ-0001, REQ-0003 and REQ-0004 — all fire around
    the first real service, which is precisely when there is least capacity to absorb them.
    A trigger schedule needs a stated order of adoption, or the first service arrives with
    three large governance obligations at once and honours none.

## The evidence already collected

This requirement is unusual in having its evidence gathered before it was written:

- **Extending a locked artifact has no mechanism.** Hit three times in two days —
  changelog category uniqueness folded into `changelog.format_valid`, grants-agreement
  folded into `domain.entity_has_single_owner`, and the service-creation guard shipped with
  no check id at all because none could be cited. Each fold was correct locally and each
  made a check id mean slightly less than its name.
- **Errata have no path.** Two locked artifacts carry factual errors with no
  proportionate correction — REQ-0002, which is a subset of item 7.
- **Live enforcement the registry cannot see.** The creation guard and the CI JSON check
  both enforce without a check id. That is the exact condition that hid four broken checks
  for weeks.
- **Hand-maintained figures rot.** README counts went stale three times in two days before
  the numbers were removed rather than corrected again.
- **Three controls enforce with no declaring artifact**, found by audit on 2026-08-07: the
  service-creation guard, CI's JSON-validity step, and CI's shell-syntax step. Each is real
  enforcement that `check-registry.yaml` cannot see and no artifact requires. The JSON step
  is the cautionary one — it had never executed, and when it finally would have run it would
  have failed on a correct file. Nothing owned it, so nothing noticed for weeks.
- **Triggers cluster around the first service.** Found by audit while writing REQ-0003 and
  REQ-0004: three deferred requirements share one activation moment.

## Why this is deferred despite the evidence

Because the mechanism must not be designed from a sample of one organisation-of-one. Every
new mechanism written during this period had a defect found in it within the hour, and a
waiver system is a **back door into every other control** — the last place to accept that
hit rate.

The trigger is deliberately not a date. It is the first moment the absence actually costs
something: a bypass that stays invisible, a second collaborator, or legacy code that a new
rule cannot be applied to.

## Related

- REQ-0002 — errata, a strict subset of item 7
- ADR-0014 — the `declared` state, an existing waiver that does not expire
- ADR-0017 — `.gitleaksignore`, an existing waiver that does not expire
- ADR-0018 — `deferred` requirements, an existing waiver that at least has a trigger
- ADR-0008 — adopt only what governs something that exists; the rule this obeys
- REQ-0004 — risk tiering, the same item from the runtime side
