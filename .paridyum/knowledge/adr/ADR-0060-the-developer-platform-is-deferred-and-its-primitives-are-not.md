---
id: ADR-0060
title: The developer deployment platform is deferred; its primitives are harvested into svc-delivery
type: adr
status: accepted
created: 2026-09-02
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [product, paas, delivery, vertical, c-0006, adr-0005, adr-0009, rfc-0025, req-0004, req-0014]
breaking: false
enforced_by: []
---

# ADR-0060 — The developer platform is deferred; its primitives are not

## Status

**Accepted**, 2026-09-02.

## Context

A design document was brought forward proposing **Paridyum Deploy**: a developer-facing
application deployment platform in the shape of Vercel or Render. A developer connects a GitHub
repository, Paridyum builds an image, deploys it, returns an HTTPS URL, and can provision a
managed PostgreSQL beside it. It draws on a prior implementation (referred to as the reference
implementation) as a source of primitives and of failure history, and it is explicit that the
prior system's constraints should not become the product's architecture.

**It is a good document.** The loop it identifies — *git → build → immutable image → deploy →
public URL → managed database* — is the right thing to productise, and its instinct to separate
the product abstraction from the infrastructure is correct. This decision is not a rejection of
its content.

### Three of its recommendations are already estate decisions

Recorded because it means the document *corroborates* rather than *contributes* on these
points, and a later reader should not think they arrived with it:

| The document proposes | Already decided |
| --- | --- |
| Desired state versus runtime state, with a reconciliation engine | **RFC-0025 decision 7 — "reconcile, do not command"** |
| Immutable deployments identified by digest, never `latest` | **REQ-0004 §2** — container baseline forbids `:latest`; **REQ-0014** owns provenance |
| Do not bake WireGuard into the domain model | **RFC-0037 decision 1** — the estate is *routed addresses*; the mesh is an implementation |

### But the product it describes is a second vertical

`product-registry.yaml` records `hosting` as **vertical one**, promoted to `building` by
operator decision on 2026-08-23, and that entry states what it forecloses. Its dossier —
MKT-0001, MKT-0002, MKT-0003 — selected a **catalogue** market: containerised applications run
*for* end customers and priced per running workload, per BET-0009. The buyer is someone who
wants software run for them.

The document's buyer is a developer deploying their own repository. Different customer,
different purchase, different evidence — and **C-0006 anti-goal 6**:

> We will not add a second vertical before the first one has a paying customer who renewed.

That clause already blocks `veer-entitlements` on hosting's first **renewal** rather than its
first sale, and the registry records that as a constitutional consequence rather than a
scheduling preference. It binds this the same way. Hosting has no customer at all yet.

## Decision

### 1. The developer platform is deferred, not rejected

**Paridyum Deploy is not built as a product, and is not registered in `product-registry.yaml`,
until hosting has a paying customer who has renewed.** The trigger is C-0006 anti-goal 6's, not
a date, and it is the same trigger `veer-entitlements` waits on.

Deferred rather than rejected, and the distinction is the point of writing this down: the
analysis is sound, and re-deriving it in six months would cost more than recording it now.

### 2. The loop is harvested into `svc-delivery`, which is not a second vertical

The document's Phase 1 and `svc-delivery` are **the same machine pointed at different
repositories.**

- **Deploying Paridyum's own artifacts** to Paridyum's own fleet — *which artifact is live, at
  which hostname, for which tenant* — is `svc-delivery`, already registered as a capability and
  already owed. It sells nothing to a new market and passes no gate.
- **Deploying a customer's repository** is the product above, and is deferred.

So `svc-delivery` is built now with the loop's shape deliberately intact: a build that produces
a **digest**, a release that names that digest, a deployment that reconciles toward it, and a
rollback that is the selection of an earlier one. When the gate opens, Phase 1 exists.

**This is the whole value of the decision.** The alternative — building `svc-delivery` narrowly
now and the platform separately later — pays for the same loop twice and produces two of them.

### 3. The control plane is Rust

The document recommends Python, FastAPI, SQLAlchemy and Celery, on the grounds that this family
is already in use in prior work.

**ADR-0005 makes Rust the primary implementation language**, and the web surface design already
registers `svc-delivery` as `class: service, stack: rust`. The recommendation is declined.

The argument offered for it is the reason to decline it rather than a reason to accept: the
familiarity is with the **archived** estate, and **ADR-0009 walked away from that estate
deliberately**. Reaching back for its stack because it is comfortable is precisely the pull
ADR-0009 exists to resist. If Rust turns out to be wrong for a control plane, that is an
argument to supersede ADR-0005 on its merits — not a thing to decide by inheritance.

### 4. Provider abstraction is deferred with the product

The document proposes an abstract infrastructure API so the platform can target Kubernetes, VMs
or bare metal interchangeably. Correct eventually, and **premature now**: the estate has exactly
one runtime, and **ADR-0008 adopts rules only where enforceable and currently applicable**.

An abstraction with one implementation is not an abstraction; it is an unverified guess about
the second one, and it costs a layer that every change must pass through. `svc-delivery` targets
Kubernetes directly, and the seam is drawn where the document's own advice is cheapest to
follow — **the build service returns an image digest and knows nothing about what deploys it.**

### 5. The failure list is adopted as a checklist, and it is the document's most useful part

The document enumerates what the reference implementation got wrong — an unauthenticated
registry, an end-of-life operating system, absent private-repository support, incomplete
backups, no load testing, ten-second cold starts, and errors in reconciliation, RBAC, logging,
connection pooling and performance measurement.

**These are adopted as requirements on `svc-delivery` rather than as an appendix**, because a
list of another system's mistakes is only worth having if something checks against it. Two are
already live debts on this estate and neither was noticed from the document: nothing backs up
etcd, and `infra-terraform` skips TLS verification against the Proxmox node.

## Consequences

**Accepted costs.**

- **Paridyum will not have a developer platform for some time**, and a competitor may. That is
  what anti-goal 6 costs, and it was accepted when the anti-goal was written.
- **`svc-delivery` is built with a shape it does not yet need.** Immutable digests, releases
  and reconciliation are more than "put our landing page somewhere". This is deliberate and it
  is a real cost paid now for an option later.
- **The judgement that these are different verticals could be wrong.** If hosting's customers
  turn out to be developers, the gate was never crossed and the deferral cost time for nothing.
  The evidence available says otherwise, and the evidence is thin.

**Gained.**

- The gate stays meaningful. An anti-goal overridden twice constrains nothing, and it was
  already overridden once — deliberately and on the record — to promote hosting.
- The analysis is preserved rather than re-derived.
- Phase 1 arrives with the gate rather than after it.

## Alternatives considered

- **Build the platform now as an explicit constitutional exception.** The mechanism exists and
  was used on 2026-08-23 to promote hosting. Rejected: using it twice makes anti-goal 6
  decorative, and it splits attention before vertical one has a single customer.
- **Redefine hosting as the developer platform.** Legitimate if believed — but hosting's dossier
  selected a catalogue market on external, dated evidence, and C-0006 forbids tenant-derived
  evidence. A developer market needs its own MKT- artifacts. That is real work, not a relabel,
  and nobody has done it.
- **Reject the document and build `svc-delivery` narrowly.** Cheapest today. Rejected because
  the loop would be built twice, and the second one would have to replace the first while it
  was carrying traffic.
- **Adopt the Python stack for the control plane only.** Faster to write and a larger ecosystem
  for this kind of service. Rejected under ADR-0005, and because a per-service language decision
  is a question every later service then re-asks.

## Related

- **C-0006 anti-goal 6** — the gate this defers to, and the one `veer-entitlements` also waits on
- **ADR-0005** — Rust as primary language, which decides §3
- **ADR-0009** — the archived estate, which is why §3's argument runs the way it does
- **ADR-0008** — enforceable and currently applicable, which decides §4
- **RFC-0025 D7** · **REQ-0004 §2** · **REQ-0014** · **RFC-0037 D1** — already-held positions the
  document independently arrives at
- **`product-registry.yaml`** — `hosting` as vertical one, and what its promotion foreclosed
