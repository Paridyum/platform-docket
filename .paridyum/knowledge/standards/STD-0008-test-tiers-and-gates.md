---
id: STD-0008
title: Test tiers and gates — what runs where, what blocks, and what a run leaves behind
type: standard
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [testing, ci, tiers, gates, flaky, regression, evidence]
breaking: false
verified_by:
  - TST-0001
  - TST-0005
  - TST-0006
  - TST-0007
  - TST-0008
  - TST-0009
  - TST-0010
  - TST-0011
  - TST-0012
  - TST-0013
  - TST-0014
  - TST-0022
  - TST-0023
  - TST-0024
  - TST-0054
  - TST-0056
enforced_by:
  - check: test.tiers_are_declared_in_ci
    severity: error
    rationale: >
      A suite with no tier runs everywhere or nowhere. Both are failures and neither is
      visible until somebody measures the pull-request wall clock or the release surprise.
  - check: test.bugfix_carries_a_regression_test
    severity: error
    rationale: >
      A defect that reached a user once will reach them twice, and the only moment anybody
      knows how to reproduce it is the moment it is fixed.
  - check: test.flaky_tests_are_quarantined_with_an_owner
    severity: error
    rationale: >
      An unowned flaky test teaches "just rerun CI", and that habit costs more than every
      failure the suite would have caught.
  - check: test.testing_md_declares_untested_surface
    severity: error
    args: { file: "TESTING.md" }
    rationale: >
      STD-0001 requires the untested surface to be stated and nothing required the file it
      is stated in, so the requirement was satisfiable by omission.
  - check: test.evidence_is_published_for_blocking_tiers
    severity: error
    rationale: >
      A tier that blocks a change and leaves nothing behind cannot be audited, re-read, or
      distinguished from a tier that was skipped.
  - check: test.default_profile_does_not_retry
    severity: error
    rationale: >
      A retry in the local profile hides the flake at the only moment it is cheap to see.
  - check: test.no_io_in_unit_tier
    severity: error
    rationale: >
      A unit test that reaches the network is an integration test with a misleading label
      and a pull-request budget it cannot keep.
  - check: test.static_analysis_runs_on_pull_request
    severity: error
    rationale: >
      Static analysis is the only security control that is cheap enough to run on every
      change, so running it later is choosing to find the defect after it has been built on.
  - check: test.migration_is_applied_and_reverted_in_ci
    severity: error
    rationale: >
      REQ-0008 §4 calls an untested down-migration a comment. Applying both directions in CI
      is what makes it code.
  - check: test.mutation_score_meets_threshold
    severity: warn
    args: { scope: "changed-files", threshold: 60 }
    rationale: >
      Warn, not error: mutation score is noisy on small diffs and a hard gate on a noisy
      metric is a gate that gets disabled. It is the only signal that distinguishes a test
      from a line that merely executes.
---

# STD-0008 — Test tiers and gates

STD-0001 says how to write a test. This says **when it runs, what its failure costs, and what
it leaves behind.** Those are different questions, and the second one is what decides whether a
suite survives its first year.

The failure this standard exists to prevent has two symmetrical forms. Run everything on every
push and the pull-request loop becomes twenty minutes, so people batch changes, and batched
changes are the ones review misses. Run nothing until release and the release becomes an event
somebody schedules around. Tiers are how a large matrix stays runnable without either.

The vocabulary is `test-registry.yaml` (ADR-0029). This standard does not restate it — every
type named here is an id there, with its axis, its tier and the evidence it produces.

## The tiers

| Tier | Starts when | Budget | Blocking | Failure means |
| ---- | ----------- | ------ | -------- | ------------- |
| `pr` | every push to a branch with an open PR | **10 min** | yes | the change does not proceed |
| `merge` | a change lands on the default branch | 30 min | yes | no release candidate is cut |
| `nightly` | a schedule, when nobody is waiting | 4 h | no | an owner is notified |
| `pre-release` | a release candidate is cut | 2 h | yes | the candidate is rejected |
| `post-deploy` | a deployment completes, any environment | 10 min | yes | the rollout halts or reverses |
| `on-demand` | a person, or an incident | — | no | a finding, tracked |

**The 10-minute pull-request budget is a constraint, not a target.** It is the only tier a
person waits for. Anything that cannot meet it moves down the list; it does not get a faster,
weaker version here. When the budget is exceeded the correct response is to move a suite, not
to raise the number — a budget that is raised whenever it binds is not a budget.

**`blocking` on `merge` cannot mean blocking the merge.** GitHub Free has no required checks on
private repositories (ADR-0004), so a red merge-tier run is a signal a person must act on. This
is stated rather than glossed because a table claiming a gate that does not exist is worse than
one admitting the gap.

**`nightly` is non-blocking and is therefore the tier most likely to be silently broken for
months.** A nightly tier without a failure notification and a named owner is compute spent on
nobody's behalf. If neither exists, delete the schedule rather than leaving it green-by-neglect.

## What runs where

Only types whose `status` is `required` or `recommended` today are listed. Everything else is in
the registry with a trigger; nothing needs to be remembered.

```
pr           TST-0001 unit · TST-0002 component · TST-0003 integration
             TST-0009 sanity · TST-0010 regression · TST-0011 property · TST-0015 snapshot
             TST-0022 static-analysis · TST-0023 dependency-scan · TST-0024 secret-scan
merge        (nothing required yet — TST-0004 contract and TST-0021 out-of-band-isolation land here)
nightly      TST-0013 mutation · TST-0028 benchmark · TST-0047 runtime-version
pre-release  (nothing required yet — TST-0054 release-candidate is the gate that will hold it)
post-deploy  (nothing required yet — TST-0008 smoke and TST-0056 production-smoke land here)
```

That `merge` and `pre-release` rows are empty is the accurate current state, not an omission.
Nothing is deployed.

## The gate

```yaml
# The shape a repository's CI declares. Names are TST- ids so the pipeline
# and the registry cannot drift into two different vocabularies.
tiers:
  pr:          [TST-0001, TST-0002, TST-0003, TST-0010, TST-0022, TST-0023, TST-0024]
  merge:       []
  nightly:     [TST-0013, TST-0028, TST-0047]
  pre-release: []
  post-deploy: []
```

Nobody has to remember the process once the pipeline is the process. That is the whole reason
the tier is declared as data rather than described in a README.

## A fix carries a regression test

**No bug fix is complete without a test that fails before it and passes after.** A `fix:` commit
whose diff touches no test file is rejected.

The reason is timing rather than rigour. The only moment anybody knows how to reproduce a defect
is the moment it is understood well enough to fix, and that knowledge decays within days. A
regression suite built this way is the only part of a test suite where every entry is evidence
that a specific failure was possible — as opposed to evidence that somebody imagined one.

The exception is real and narrow: where the reproduction genuinely cannot be automated — a race
that needs a specific scheduler, a defect in a third party — say so in the commit body and name
what would have to change for it to become testable. Silence is not an exception.

This maps to SSDF `RV.3.3` and `RV.3.4`, which the ADR-0028 mapping found completely unpopulated.
The finding at the time was that this repository governed how code is written far better than
what happens after a defect is found. This is the first control on that side.

## Flaky tests

A flaky test is worse than a failing test and much worse than an absent one, because it teaches
one specific lesson: **rerun CI.** Once that is learned it is applied to real failures too.

```
FLAKY-001  A test that fails nondeterministically is QUARANTINED within one working day —
           removed from the blocking tier, never deleted, never silently retried.

FLAKY-002  A quarantine entry carries an owner, the date it was first seen, the observed
           failure rate, and a tracking reference. Missing any of these, it is a disabled
           test with paperwork.

FLAKY-003  A quarantine entry EXPIRES. On expiry it is fixed or the test is deleted with the
           reason recorded. Silence is not renewal.

FLAKY-004  The quarantine list lives in TESTING.md, in the repository the test is in.
```

Expiry rather than permanence is the same mechanic ADR-0017 applies to scan exceptions and
REQ-0006 §1 to waivers, and it should reuse that machinery rather than becoming a third
implementation of the same idea.

**CI retries absorb infrastructure noise, not test flakiness.** STD-0001 already sets `retries = 0`
locally and a bounded backoff in CI. A test that only passes on retry is failing; the runner's
flaky reporting is what surfaces it, and the response is quarantine, not a higher retry count.

## `TESTING.md`

Required in every repository that has tests. STD-0001 requires the untested surface to be stated
"in the repository's `TESTING.md`" and nothing required that file to exist, so the requirement
was satisfiable by omission — the same defect class as a check that verifies less than its name
claims.

Four sections, all four required:

| Section | Answers |
| ------- | ------- |
| Tiers | which TST- types run in which tier here, and why anything deviates from the table above |
| Not tested | what is deliberately uncovered, and the reason — generated code, third-party wrappers, `main()` wiring |
| Quarantine | the FLAKY-002 table, or the sentence "none" |
| Evidence | where each tier's artifacts go, and how long they are kept |

An undocumented gap reads as an oversight to whoever finds it next. A documented one is a
decision, and can be argued with.

## Evidence

**Every blocking tier publishes a machine-readable artifact.** Not a log — logs expire, and a
log is not searchable three months later when the question is "did that run, on that commit".

The registry's `evidence` field says what each type produces. The tier is what makes it durable:
a run that blocks a change and leaves nothing behind cannot be distinguished afterwards from a
run that was skipped. This is SSDF `PO.3.3` — configure tools to generate artifacts of their
support of secure development — and it is the difference between a claim and a record.

Retention is stated in `TESTING.md` rather than fixed here, because it depends on what the
repository is; the requirement is that a number exists.

## Determinism is a tier property, not only a test property

STD-0001 requires every test to be deterministic. This adds the tier-level consequence:

- **No network in the `pr` tier** beyond containers the test starts itself. A unit test that
  reaches the network is an integration test wearing the wrong label, and it is the single most
  common way a 10-minute budget becomes a 25-minute one.
- **No shared mutable environment across tiers.** A `nightly` run that leaves state a `pr` run
  depends on produces a suite that passes in one order and fails in another.
- **No test that depends on another tier having run.** Each tier is entered from a clean state
  or the tier boundary means nothing.

## What this standard does not cover

Performance budgets (REQ-0016), resilience and recovery (REQ-0017), outside-in security testing
(REQ-0018) and tenant isolation (STD-0009). Each is named here so the absence is visible and
none is restated, because two copies of a rule is one rule and one liability.

## Related

- [ADR-0029](../adr/ADR-0029-test-types-are-registry-data.md) · [test-registry.yaml](../test-registry.yaml) — the vocabulary this standard schedules
- [ADR-0030](../adr/ADR-0030-every-binding-rule-names-its-verification.md) — why this artifact carries `verified_by`
- [STD-0001](STD-0001-testing.md) — how a test is written; unchanged and assumed here
- [STD-0009](STD-0009-tenant-isolation-testing.md) — the isolation axis, which has its own tier rules
- [ADR-0010](../adr/ADR-0010-ci-cd-architecture.md) · [ADR-0021](../adr/ADR-0021-ci-is-a-reusable-workflow.md) — where the tiers are implemented
- [ADR-0017](../adr/ADR-0017-scan-exceptions-are-reviewable.md) — the expiry mechanic FLAKY-003 reuses
- [REQ-0008](../requirements/REQ-0008-database-schema-and-migration-governance.md) — §4, the migration rule this gates
