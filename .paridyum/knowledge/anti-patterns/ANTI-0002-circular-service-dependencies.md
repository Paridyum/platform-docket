---
id: ANTI-0002
title: Circular dependencies between services
type: anti-pattern
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [platform, service, library]
applies_to: ["platform-*", "svc-*", "lib-*"]
tags: [coupling, dependency-graph, deployment, events]
breaking: false
enforced_by:
  - check: domain.no_service_dependency_cycles
    severity: error
    rationale: The dependency graph is already declared in manifest.yaml, so a cycle is detectable today with a check rather than a new mechanism.
---

# ANTI-0002 — Circular dependencies between services

## The shape

```
svc-billing  ──calls──▶  svc-notification  ──calls──▶  svc-billing
```

Rarely that obvious. Usually three or four hops, introduced one edge at a time by people
who could each see only their own edge.

## Why it is forbidden

**A cycle means there is no order in which the services can be deployed, started, or
reasoned about.**

- **Deployment.** Neither can be released first without the other already accepting the new
  contract. You get a synchronised deploy, which is two services pretending to be one while
  paying the cost of being two.
- **Startup.** If either blocks on the other at boot, the pair cannot cold-start. This is
  discovered during an incident, at the worst moment, because it only manifests when both
  are down simultaneously.
- **Reasoning.** "What breaks if I change this?" has no bounded answer. The blast radius is
  the cycle, and cycles grow.
- **Testing.** Neither can be tested against a real dependency without the other, so both
  end up tested against mocks that drift from the thing they mock.

A cycle is also a **strong signal the boundary is wrong**. Two services that need each other
synchronously are frequently one service that was split along the wrong seam — or two
services plus a missing third that should own the shared concern.

## What to do instead

**Invert one edge into an event.** This is almost always available and almost always right:

```
svc-billing  ──publishes──▶  InvoiceIssued  ──▶  svc-notification
```

`svc-notification` now depends on billing's *event contract*, and billing depends on
nothing. Billing does not know notification exists, which is the point — it is what lets
you add a second subscriber without touching billing at all.

The general rule: **a service must not need a synchronous answer from something downstream
of it.** If it does, either the downstream work is not actually needed to complete the
request (make it an event), or the boundary is wrong (merge, or extract a third service that
both depend on).

Extracting a third is the other honest answer. If A and B both need shared logic, that
logic is a domain neither owns, and naming it usually clarifies the whole design.

## How this is detected

`domain.no_service_dependency_cycles` reads `depends.services` from every
`manifest.yaml` and fails on any cycle. **The graph is already declared** — STD-0002
established that field — so this needs a check, not new machinery.

Two limits worth knowing:

- It sees only **declared** dependencies. A service calling another by hard-coded URL
  without declaring it is invisible here. `check-significance.sh` already treats a new
  `manifest.yaml` service dependency as significant, which is the pressure toward declaring.
- **Event edges are deliberately not cycles.** `A publishes → B subscribes → B publishes →
  A subscribes` is legal and often correct, because neither blocks on the other. Only
  `depends.services` — synchronous calls — participates in cycle detection. A check that
  flagged event loops would forbid the very fix this document recommends.

## The legitimate exception

None for synchronous cycles.

Event loops are not exceptions, they are the intended design. If you believe you have found
a case needing a synchronous cycle, the boundary is wrong — write the RFC that moves it.

## Related

- STD-0002 — defines `manifest.yaml` and `depends.services`, the graph this reads
- C-0002 — event naming, `<Subject><PastTenseVerb>`
- ADR-0003 — Kubernetes; a cold-start cycle is a cluster-wide outage, not a slow boot
- STD-0004 — `module.no_dependency_cycles`, the same rule one level down
- ANTI-0006 — a god service, which is what merging to break a cycle can produce if done carelessly
