---
id: REQ-0012
title: An external dependency is approved, scoped, and exitable; controls produce evidence
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [vendor, third-party, compliance, audit-evidence, soc2, exit-plan]
breaking: false
trigger: >
  FIRED 2026-08-08. On the ORGANISATION half, which the deferral rationale answered only
  for customer data. GitHub has held this organisation's entire source of truth since ADR-0004 —
  private repositories, Actions, GHCR — and item 6 already conceded the concentration risk is
  "real and current". Anthropic processing the repository is a second instance.
  ORIGINAL: A third-party service is introduced that holds organisation or customer data, OR a
  customer contract or certification requires evidence that a control operates.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0012 — An external dependency is approved, scoped, and exitable; controls produce evidence

## Status

**DEFERRED** as of 2026-08-07. Two vendors exist — GitHub and, prospectively, a secret
manager (REQ-0001) — and neither holds customer data because there are no customers. No
contract or certification obliges anything.

Both halves are recorded together because they share a trigger shape: an obligation
arriving from **outside** the organisation rather than from engineering judgement.

## Requirement

**A third-party service holding organisation or customer data shall be approved before
adoption, scoped to least privilege, and have a stated exit path — and where an obligation
requires it, controls shall produce durable evidence that they operated.**

## Numbered items — vendor

1. **An approval step before adoption**, not after. Security posture, data handling, and
   what happens to the data on termination. The cheapest moment to ask is before the
   integration exists.
2. **A signed data processing agreement** wherever personal data is involved (see REQ-0003).
3. **Least-privilege credentials, rotated**, with custody stated. REQ-0001 is the mechanism;
   this is the policy that says which vendor gets what.
4. **Egress allowlists.** Which external hosts a service may reach at all. This is also the
   strongest available mitigation for prompt injection and for a compromised dependency
   phoning home (REQ-0007 §4).
5. **Criticality tiering and an exit plan.** For anything the platform cannot run without:
   what the migration looks like, and how long it would take. A vendor with no exit plan is
   a dependency with unbounded switching cost — and the plan is worth writing while the
   integration is small.
6. **Concentration is a risk.** GitHub is currently source control, CI, artifact storage,
   secret storage and identity. That is convenient and it is one failure domain, as the
   Actions outage during this work demonstrated: a GitHub incident stopped every gate at
   once. Worth stating before it is worth fixing.

## Numbered items — compliance and evidence

7. **Map obligations to actual controls**, not to intentions. If SOC 2, ISO 27001, HIPAA or
   PCI ever applies, each requirement maps to a specific check, hook or documented process
   — and the ones with no mapping are the honest gap list.
8. **Separation of duties.** The author of a change is not its sole approver or its
   deployer. Unenforceable today with one member (REQ-0011 §2), and the first thing an
   auditor asks.
9. **Immutable audit trail from ticket to deploy.** Requirement → RFC → ADR → commit → PR →
   build → artifact → deploy. Most of this chain already exists as artifact ids and git
   history; what is missing is the build and deploy end, and anything proving *who
   approved*.
10. **Evidence collected automatically.** Compliance evidence gathered by hand at audit time
    is expensive, incomplete, and frequently retrospective fiction. Controls that emit
    evidence as a side effect of operating are the only ones that survive scrutiny.
11. **Access reviews on a cadence**, with offboarding that revokes, rotates and reassigns
    ownership. Trivial with one member; the first thing to rot with three.
12. **Retention of approval and review records**, which is a decision about how long CI
    logs, PR history and audit logs are kept — and therefore a cost decision as well as a
    compliance one.

## Why this is not startable today

Items 1–5 need a vendor that holds data. Items 7–12 need an obligation; building an
evidence pipeline for a certification nobody has asked for is expensive scaffolding of
exactly the kind ADR-0008 rejects.

Item 6 is the exception worth noting now: the concentration risk is **real and current**,
and it is recorded rather than acted on because the alternatives — self-hosted runners, a
second forge — cost more than the risk at this size. That calculus changes the moment an
outage stops paying work rather than a side project.

## Related

- REQ-0001 — credential delivery, the mechanism behind item 3
- REQ-0003 — personal data, which triggers item 2
- REQ-0006 — governance metrics and rule ownership; evidence is the same problem
- REQ-0007 §4 — untrusted content, which item 4 mitigates
- REQ-0011 §2 — separation of duties from the review side
- ADR-0004 — the plan, and what it does and does not make available
