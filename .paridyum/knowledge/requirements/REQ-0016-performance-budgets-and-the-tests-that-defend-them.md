---
id: REQ-0016
title: A service states its performance budget before it is measured, and a test defends it
type: requirement
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["svc-*", "platform-*"]
tags: [performance, load, capacity, slo, budgets, testing]
breaking: false
trigger: >
  FIRED 2026-08-08. `templates/rust-service/manifest.yaml` declared `slo: availability
  99.9 / latency_p99_ms 250`, so the trigger had fired against the scaffold — and would have
  auto-fired on service #1, copying numbers "derived from taste", which is the exact failure this
  requirement exists to prevent. The block has been removed from the template; the requirement
  stays live so the numbers are chosen rather than inherited.
  ORIGINAL: A repository declares an `slo` block in its manifest.yaml — the first moment a latency
  or availability number exists that something could be measured against.
review_date: 2027-02-01
verified_by:
  - TST-0028
  - TST-0029
  - TST-0030
  - TST-0031
  - TST-0032
  - TST-0033
enforced_by: []
---

# REQ-0016 — A service states its performance budget before it is measured

## Status

**DEFERRED** as of 2026-08-08. No service, no traffic, no `slo` block anywhere.

The trigger is the `slo` declaration rather than the first deploy, and that ordering is the
substance of this requirement. A load test written before a budget exists produces a graph, and
a graph is not a pass or a fail — it is a number somebody will defend later for no stated reason.

## Requirement

**A service shall declare its latency and availability budget in machine-readable form before
any performance test is written, and every performance test shall assert against that
declaration rather than reporting a measurement.**

## What already exists

More than is obvious. `manifest.yaml` already carries an `slo` block with `availability` and
`latency_p99_ms` (STD-0002), validated by `docs.manifest_schema_valid`. So the budget has a home,
a schema and a gate before the first test exists.

REQ-0004 §10 owns SLOs and error budgets as an operational discipline — measured against real
traffic, with an exhausted budget blocking feature work. This requirement is the pre-production
half: the same numbers, asserted before users supply the measurement.

STD-0001 restricts `criterion` benchmarks to places where a performance claim is being made,
which is the right constraint at the micro level and says nothing about the service level.

The gap is that nothing connects the declared number to anything that runs.

## Numbered items

1. **Percentiles, not averages, and at least three.** `p50`, `p95`, `p99`. An average hides the
   tail entirely, and the tail is what a user calls slow. `manifest.yaml` currently carries only
   `latency_p99_ms`; a budget with one percentile cannot distinguish "usually fast, occasionally
   terrible" from "uniformly mediocre", and those need different fixes.

2. **The budget is per endpoint or per workflow, not per service.** A service whose median
   endpoint is fast and whose export endpoint takes forty seconds meets a service-level budget
   and fails its users. Aggregate budgets are the performance equivalent of average latency.

3. **A test asserts the budget and fails.** This is item 1's whole purpose. A performance suite
   whose output is a report is a suite nobody reads after the first month.

4. **Shared runners are noisy, and the mitigation is stated rather than assumed.** A single run
   on a shared CI runner cannot distinguish a 10% regression from neighbour noise. Compare
   distributions across repeated runs, or run the gate on a dedicated environment, or make the
   threshold wide enough to be meaningful — but decide which, because the default is a flaky
   gate that gets deleted and takes the discipline with it.

5. **All five load shapes are distinct questions, and running only one is the common mistake.**
   Load asks whether the budget holds at expected traffic (TST-0029). Stress asks *how* it fails
   beyond capacity (TST-0030) — and the pass condition is a failure **mode**, not a number:
   shed load, reject cleanly, recover unattended. Spike asks whether a sudden multiple is
   absorbed (TST-0031). Soak asks what degrades over hours (TST-0032) and is the only one whose
   signal is a trend. Capacity asks where the SLO breaks (TST-0033).

6. **Capacity is a commercial measurement.** BET-0002 prices on density and C-0006 makes
   not-infrastructure-resold a market position; density is exactly TST-0033's number. An
   unmeasured capacity assumption is an unmeasured margin assumption, and it will be discovered
   by a pricing decision rather than by an engineer.

7. **The load generator produces tenant-shaped traffic.** ADR-0024 puts two predicates on every
   query on the hot path — an accepted cost, stated as such. A load test using a single tenant
   measures a query plan no real workload will produce, and the failure it misses is a plan that
   degrades as tenant count grows rather than as request rate grows.

8. **Frontend budgets are separate and are not covered here.** Bundle size and Core Web Vitals
   belong with whatever RFC-0008 resolves to. Naming them here would settle a decision that has
   not been made.

9. **A budget that is repeatedly missed is renegotiated in the open or removed.** A budget quietly
   raised whenever it binds is not a budget, and one that fails every night trains people to
   ignore the tier it runs in — the same decay FLAKY-001 describes, applied to a threshold.

## Why this is not startable today

Every item measures a running system under traffic. There is none. Item 1 could be decided now —
adding `latency_p50_ms` and `latency_p95_ms` to `manifest.schema.json` is a small change — and is
deliberately deferred with the rest, because percentile targets chosen before a single query
plan exists would be numbers derived from taste, and they would harden into an SLO that
something is then engineered against.

## Framework position

Related: NIST SSDF `PO.4.1` (define criteria for software security checks and track them through
the SDLC — a performance budget is such a criterion) and `PW.8.2` (scope, design, perform and
document testing); NIST CSF 2.0 `ID.IM-02` (improvements identified from tests and exercises).

No CIS mapping. The CIS Controls are a security baseline and performance is not one of their
outcomes; an invented mapping here would read as coverage, which ADR-0028 forbids.

## Related

- [REQ-0004](REQ-0004-runtime-and-operations-governance.md) — §10 SLOs and error budgets, §13 cost; the operational half of the same numbers
- [REQ-0017](REQ-0017-failure-is-injected-not-awaited.md) — behaviour under failure, the sibling axis
- [STD-0002](../standards/STD-0002-documentation.md) — `manifest.yaml`'s `slo` block, where the budget already lives
- [STD-0001](../standards/STD-0001-testing.md) — `criterion`, and the rule that a benchmark needs a claim behind it
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) — the tiers these types run in
- [ADR-0024](../adr/ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) — cost 5, two predicates on every hot-path query
- [BET-0002](../bets/BET-0002-price-is-not-infrastructure-resold.md) — why capacity is a commercial number
