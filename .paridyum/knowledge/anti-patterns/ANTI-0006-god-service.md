---
id: ANTI-0006
title: A service that accumulates unrelated responsibilities
type: anti-pattern
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [boundaries, cohesion, coupling, judgement]
breaking: false
enforced_by: []
---

# ANTI-0006 — A service that accumulates unrelated responsibilities

## Status

**ACCEPTED**, not `locked`. ADR-0002 already settled why: *"'Service boundaries should
follow business capabilities' cannot be mechanically verified."* No metric distinguishes a
large service with one responsibility from a medium one with four. Line counts, endpoint
counts and table counts all fail in both directions. Upheld by review.

STD-0004's `module.has_responsibility_doc` and `module.responsibility_is_singular` catch the
same disease one level down, inside a crate. They are the closest available proxies and they
do not reach service boundaries.

## The shape

`svc-core`. Or `svc-platform`, `svc-api`, `svc-backend`, `svc-main`.

It starts as the one service. Then a feature arrives that does not obviously belong
anywhere, and this is where things go that do not obviously belong anywhere. Repeat for
eighteen months.

```
svc-core
  ├── users          ├── invoices       ├── notifications
  ├── auth           ├── payments       ├── file uploads
  ├── audit log      ├── reporting      └── the cron jobs
```

Nobody decided this. Each addition was the smallest available change.

## Why it is forbidden

**Every cost of a monolith, plus every cost of distribution, and the benefits of neither.**

- **The blast radius is everything.** A bad deploy takes down auth, payments and uploads
  together. The reason to have services at all was that they fail separately.
- **It cannot be scaled sensibly.** File uploads are IO-bound, reporting is CPU-bound, auth
  is latency-critical. One deployable means one resource profile, sized for the worst case
  and wrong for everything.
- **It becomes everyone's dependency.** Because it owns nine things, every other service
  needs it, which puts it at the centre of the graph and makes ANTI-0002 cycles nearly
  unavoidable.
- **Nobody owns it.** `manifest.yaml` names one `owner`, but nine responsibilities means the
  real owners are nine teams, so review has no expert and changes get approved by whoever is
  around.
- **It is the hardest thing to fix.** Splitting a service that nine others depend on requires
  changing nine callers. The cost of the mistake grows with every month it survives, which is
  what makes it worth an explicit rule despite being unenforceable.

## What to do instead

**A service exists to own one business capability. If you cannot name it in one sentence
without "and", it is more than one service.**

That is STD-0004's test for modules, applied to services. It works for the same reason.

**When a new feature does not obviously belong anywhere**, that is information — the most
useful signal in the design. It usually means one of:

1. It belongs to an existing capability and you have not seen how. Look harder before
   creating anything.
2. It is a **new capability** and wants its own service.
3. It is a **shared capability** several services need — file storage, notification delivery,
   audit — and wants its own service that others call. This is the case most often
   mishandled, because absorbing it into an existing service is always the smaller diff
   today.

What it never means is "put it in the big one for now."

**Naming is the early warning.** `svc-core`, `svc-platform`, `svc-api`, `svc-common`,
`svc-shared`, `svc-main` are all names that describe *no capability*. C-0002 already requires
the name to be a noun naming the capability, so a service that cannot be named specifically
has already failed a locked rule — and that is the practical enforcement available here.

## How to spot it in review

- Can you state its responsibility in one sentence with no "and"? (STD-0004's test.)
- Does its name identify a business capability, or a position in the architecture?
- Do its tables belong to one domain? Nine unrelated tables is nine responsibilities.
- Do most other services depend on it? Being at the centre of the graph is a symptom.
- Would two of its endpoints ever need different scaling, different SLOs, or different
  on-call?

## Related

- ADR-0002 — establishes that this class of rule cannot be mechanically verified
- C-0002 — the name must be a noun naming the capability; the usable proxy
- STD-0004 — the same rule for modules, with checks that do exist
- ANTI-0002 — cycles, which a central service makes nearly inevitable
- ANTI-0004 — the other shape a missing boundary takes: a library instead of a service
