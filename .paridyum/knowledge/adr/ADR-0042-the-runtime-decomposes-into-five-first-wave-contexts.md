---
id: ADR-0042
title: The runtime decomposes into five first-wave bounded contexts, one repository each
type: adr
status: superseded
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: ADR-0101
scope: [org, platform, service, spec]
applies_to: ["*"]
tags: [decomposition, bounded-contexts, tenancy, identity, adr-0019, rfc-0009]
breaking: false
enforced_by:
  - check: domain.entity_has_single_owner
    severity: error
    rationale: >
      The decomposition's value is entirely in who owns what. RFC-0009 §2 found two name
      collisions where one word meant two entities, and ANTI-0001 is the shape that results
      when the answer is left implicit — so the check that resolves an entity to exactly one
      owning context is the one that keeps this ADR true after it stops being read.
  - check: domain.no_service_dependency_cycles
    severity: error
    rationale: >
      Five contexts with an ordering — tenancy orchestrates, identity provisions, entitlement
      decides — is a shape that acquires a back-edge the first time one of them needs a fact
      it did not own. ANTI-0002 is locked and undetectable by review once there are five.
---

# ADR-0042 — Five first-wave contexts

## Status

**ACCEPTED** as of 2026-08-08. **Resolves RFC-0009.** Created `accepted` per `new-decision` §5b.

## Context

ADR-0019 is locked: *"A specification is decomposed into bounded contexts with declared ownership
before any service is created."* RFC-0009 produced that decomposition and then stopped, correctly
— it named two open questions as blocking, and ADR-0013 requires a human between a decomposition
and any code.

Both blockers are now answered. **Q1** — one identity context or two — is answered by RFC-0012:
one, with the two populations separated by Zitadel organization rather than by service boundary.
**Q3** — the compensating path for a half-created tenant — is answered by ADR-0039's saga.

One thing changed underneath in the same period, and it removes a context rather than adding
one. BET-0003 was withdrawn and replaced by BET-0004: the product is a **catalogue**, not
generated applications. `platform-generation` and `platform-spec` were justified by the old bet,
and RFC-0009 Revision 2 re-justified both with different roles and different classes — one of
which is no longer first-wave.

## Decision

**1. The first wave is five contexts, one repository each (ADR-0001).**

| Context | Responsibility — one sentence, no "and" | Owns |
| --- | --- | --- |
| `platform-tenancy` | Maintains the operator → tenant → account hierarchy. | Operator, Tenant, Account |
| `platform-identity` | Proxies and provisions the identity engine on a tenant's behalf. | Zitadel organizations, projects, grants, capability policy |
| `platform-entitlement` | Decides what a tenant is permitted to do. | PlatformPlan, Subscription, Entitlement |
| `platform-routing` | Resolves a request to a tenant and a region. | Route, DomainBinding |
| *first catalogue product* | The first vertical application sold. | its own entities |

**2. `platform-tenancy` is built first.** It owns the tenant lifecycle, so it owns ADR-0039's
provisioning saga, and every other context is a dependency of that saga rather than a peer of it.

**3. `platform-billing` stays deferred**, on RFC-0009's trigger: *the first operator who must be
charged.* BET-0004 makes per-product pricing tempting and BET-0002 does not authorise it, so
building billing before the pricing model is decided would encode a guess.

**4. `spec-product` replaces `platform-spec`**, as a `spec-` class repository holding **product
contracts** — each catalogue product's Zitadel Project, role vocabulary, audience and tenancy
scope. RFC-0012 requires that declaration and ADR-0039's saga reads it; today it is prose.

**5. `tool-generate` replaces `platform-generation`, deferred.** Trigger: *the third catalogue
product is scoped, or two shipped products share a structure that was copied rather than
generated.* It is the factory Paridyum builds its catalogue with, not the product.

**6. Two entity collisions are resolved by splitting, never by reconciling.** RFC-0009 §2 found
both:

- **`invoices`** is two entities — Paridyum's invoice to an operator (`platform-billing`) and a
  tenant's invoice to its accounts (a catalogue product). One word, two owners, two lifecycles.
- **`member`/`user`** is two — an OperatorMember in the Paridyum plane and an AccountMember in a
  tenant's organization. RFC-0012 separates them by Zitadel instance.

**7. Creating any of these still escalates to a human**, per ADR-0019 decision 5. This ADR
supplies the decomposition that decision requires; it does not pre-authorise the repositories.

## Consequences

**Accepted costs**

1. **Five repositories before one product ships.** Four of them are platform plumbing a customer
   never sees, and each carries the full weight of STD-0002's required files, CI, and a manifest.
   That is the cost ADR-0001 accepted when it chose one context per repository, paid up front.

2. **`platform-routing` is the least justified of the five.** It exists because tenant custom
   domains and regional residency have to resolve somewhere, but it could plausibly start as a
   module inside `platform-tenancy`. It is listed separately because ADR-0041 makes region a
   contractual fact, and a contractual fact with no owner ends up owned by whoever touched it
   last.

3. **The first catalogue product is unnamed here**, deliberately, and that is a gap: four
   platform contexts have no customer until it exists. Choosing it is a commercial decision this
   ADR does not make and BET-0004 does not settle.

4. **Deferring billing means entitlement has no revenue signal.** `platform-entitlement` will
   decide what a tenant may do with no knowledge of what they paid, which works only while the
   answer is the same for everyone.

**Gained**

- The blocker on `platform-tenancy` and `platform-identity` is removed, which is what ADR-0019
  required before any service exists.
- Six contexts became five, and the one removed was removed by a commercial decision rather than
  by scope-cutting.
- Both name collisions are settled before either name appears in a schema, which is the only
  point at which settling them is cheap.

**Rejected alternatives**

- **Merge `platform-identity` into `platform-tenancy`.** Tempting — both are about who someone
  is, and both talk to Zitadel. Rejected because their availability profiles differ by an order
  of magnitude: login must be near-perfect, provisioning may be down for an hour. ANTI-0006 is
  the shape that results from merging on topical similarity.

- **Start with the product and extract platform contexts later.** The fastest route to something
  demonstrable, and it inverts ADR-0019. Extraction after the fact is how the `invoices`
  collision would have been discovered — in production, with rows in both.

- **Keep all six contexts and defer generation without reclassifying it.** Leaves a `platform-`
  repository that is `deployable: true` for a build-time tool, dragging ADR-0020's Kubernetes
  obligations onto something that never runs in a cluster.

## Related

- RFC-0009 — resolved by this; its Revision 2 re-justified the two contexts BET-0003 carried
- ADR-0019 — decomposition precedes implementation; this is the artifact it demands
- RFC-0012, ADR-0039 — the answers to Q1 and Q3 that unblocked this
- BET-0004 — the catalogue wager that removed generation from the first wave
- ADR-0001 — one context, one repository, which is what makes decision 1 expensive
- ANTI-0001, ANTI-0002, ANTI-0006 — the three shapes decision 6 and the checks defend against
- ADR-0041 — why region is contractual, which is why routing owns it
