---
id: REQ-0017
title: Failure is injected on purpose, so the recovery path runs before the incident does
type: requirement
status: deferred
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [resilience, fault-injection, chaos, disaster-recovery, testing, reliability]
breaking: false
trigger: >
  The first service is deployed with a dependency it does not own — a database, a cache, a
  queue, an identity provider or an external API. That is the moment a failure path exists
  that no test has executed.
review_date: 2027-02-01
verified_by:
  - TST-0034
  - TST-0035
  - TST-0036
enforced_by: []
---

# REQ-0017 — Failure is injected on purpose

## Status

**DEFERRED** as of 2026-08-08. Nothing is deployed and nothing has a dependency to lose.

## Requirement

**Every failure path a service declares — a timeout, a retry, a circuit breaker, a fallback, a
recovery procedure — shall be executed deliberately in a test before it is executed by an
incident.**

## What already exists

REQ-0004 §12 states the recovery rule and states it well: RPO and RTO stated, restores drilled,
*"an unverified backup is a belief."* That is the strongest sentence in this area and it is
already written.

ANTI-0005 forbids cross-service transactions and requires compensating paths, which is the
design-time form of the same concern. REQ-0004 §6 requires automated rollback on error-rate
triggers.

What is missing is everything between "the path is written" and "the path has run". A retry
policy, a circuit breaker and a compensating action are all code that executes for the first
time during the worst hour of a quarter, unless something makes it execute earlier.

## Numbered items

1. **Fault injection is in-process, deterministic, and belongs on the merge tier.** The
   dependency is made to fail on purpose, in a test, with an assertion about what the caller
   does. This is the cheap 80% and it is the item to start with: no cluster, no experiment
   framework, no environment.

2. **Every declared timeout has a test that reaches it.** A timeout nobody has hit is a number,
   and the common defect is not the value but the absence — an unbounded call inside a bounded
   one, which converts a slow dependency into an outage.

3. **Retries are tested for the thing that makes them dangerous.** Not that they retry, but that
   they stop, back off, add jitter, and do not amplify. A retry storm turns one slow dependency
   into a self-inflicted denial of service, and it is invisible in a single-caller test.

4. **Idempotency is a resilience property, not only a correctness one.** REQ-0005 §4 makes
   idempotent consumers mandatory because at-least-once delivery makes redelivery normal. The
   test is a deliberate duplicate, and it belongs here as much as there.

5. **The failure mode is asserted, not the survival.** "The service stayed up" is not a result.
   Degraded how, for whom, returning what, recovering when — those are the assertions. A system
   that survives by silently returning stale or empty data has a worse failure than one that
   errors.

6. **Chaos starts in staging, states a hypothesis, and never starts in production.** An
   experiment without a prior hypothesis is an outage with better tooling. The blast radius and
   the abort condition are decided before the run, not during it.

7. **Recovery is drilled with a measured RPO and RTO, not a stated one.** REQ-0004 §12 owns the
   rule; the measurement is the test. A drill that produces the same two numbers that were
   written down before it started has not been run.

8. **Restore forks at both tenancy boundaries, and this is the hardest item here.** ADR-0024
   cost 4 records it: per-tenant *and* per-account point-in-time restore, from a cluster-wide
   backup, by row-level extraction. A drill that restores the whole cluster has tested our
   disaster and not the product's actual promise — and that promise is the one an operator has
   made to customers who never heard of us.

9. **Certificate, credential and token expiry are injected like any other failure.** They are
   the most predictable outage in software and the most reliably unhandled, because the failure
   arrives on a date nobody is on call for.

10. **An incident produces a fault-injection case, the way a bug produces a regression test.**
    STD-0008 makes that rule for defects; this is the same rule for outages, and it is what
    stops a postmortem action item from being a document.

## Why this is not startable today

Items 1 to 5 need a service with a dependency. Items 6 to 9 need an environment shaped like
production. Item 8 needs a database with tenant data in it, and item 10 needs an incident.

Item 1 is the exception in principle and is deferred anyway: writing fault-injection scaffolding
before there is a dependency to fault produces a test harness with no subject, and the shape of
that harness is decided by the client libraries eventually chosen.

## Framework position

Related: NIST SSDF `PW.8.2` (scope, design, perform and document testing); NIST CSF 2.0
`ID.IM-02` (improvements identified from security tests and exercises — the subcategory that
covers drills and exercises specifically) and `RC.RP` (recovery plan execution); CIS v8.1 `11.1`
(establish and maintain a data recovery process) and `11.5` (test data recovery).

CIS `11.5` is the closest external control to item 7 and it is worth naming for what it does not
say: it requires recovery to be tested, not that recovery be tested per tenant. Item 8 is a
Paridyum obligation that no framework will produce, because it comes from the product shape
rather than from a security baseline.

## Related

- [REQ-0004](REQ-0004-runtime-and-operations-governance.md) — §12 backup and restore, §6 rollback; this is their verification
- [REQ-0016](REQ-0016-performance-budgets-and-the-tests-that-defend-them.md) — behaviour under load, the sibling axis
- [REQ-0005](REQ-0005-async-contract-governance.md) — §4 idempotent consumers, §6 dead-letter and replay
- [ANTI-0005](../anti-patterns/ANTI-0005-cross-service-transaction.md) — compensating paths, the design-time form
- [ADR-0024](../adr/ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) — cost 4, why restore forks twice
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) — the tiers, and the incident-to-test rule this reuses
- [REQ-0013](REQ-0013-tenant-data-portability.md) — the other consumer of row-level extraction
