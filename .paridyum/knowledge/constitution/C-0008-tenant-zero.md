---
id: C-0008
title: Every runtime capability is a product, and Paridyum is tenant zero
type: constitution
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [charter, tenancy, dogfooding, resale, platform-before-product, self-preferencing]
breaking: true
enforced_by: []
---

# Constitution C-0008 — Tenant zero

## Status

**`accepted`, deliberately not `locked`,** for the reason C-0001 gives: this states a principle,
and nothing can check a principle. **ADR-0049 is the locked half** — it carries the mechanical
rules this generates, and those are what a check can reject. `enforced_by` is empty here because
naming a proxy check would be the false confidence ADR-0002 exists to prevent.

**C-0001 outranks this document**, and this specifies rather than amends it. Principle 1 —
*platform before product* — says a capability built once in the platform is worth more than the
same capability built well three times. This says what that capability must be **built as**.

## The principle

> **Every runtime capability Paridyum builds is a product a tenant can buy. Paridyum is its first
> customer and never a privileged one.**

Two halves, and both are load-bearing:

**Nothing works without a tenant.** There is no internal mode, no operator bypass, no path that
exists because it was convenient during development. When Paridyum runs a business, it provisions
a tenant through the same saga as anyone else and holds no scope its own customers would not.

**No runtime capability is built that only Paridyum could consume.** Identity is not Paridyum's
authentication that tenants are permitted to borrow. It is a product, and Paridyum happens to be
the first tenant to buy it. The same is true of tenancy, entitlement, routing and everything that
follows them.

## Why this is a principle and not a practice

**This is stronger than dogfooding, and the difference is the whole point.** Dogfooding is *we
also use it* — a discipline, maintained by intention, abandoned under deadline. This is *there is
no other way to use it*. A privileged path cannot be taken if it was never built.

Three things follow that nothing else in this repository delivers.

**It is what makes BET-0005 survivable.** Paridyum operates the catalogue it franchises, with no
territorial or vertical boundary, so a tenant's protection is not a promise about markets — it is
that the operator has no mechanism a tenant lacks. RFC-0022's exposures are informational
precisely because they cannot be structural. **Remove this principle and that argument collapses**:
a competing landlord with a private code path is a different company than one without.

**It makes C-0001 principle 6 achievable rather than aspirational.** *"A service that needs an
exception is a signal that the platform is missing a capability."* An operator bypass is the most
attractive exception there is, and the one nobody records, because it never looks like an
exception at the time — it looks like the obvious way to get the first thing working.

**It is the only credible test of whether a capability is finished.** Software that has never been
bought by anyone is software whose seams have never been found. If Paridyum cannot consume a
capability through the contract a tenant would use, the contract is not done, and building the
tenant-facing half later is the rewrite this principle exists to avoid.

## What this forbids

- An "internal" or "admin" mode that reaches data or operations a tenant's own path cannot.
- A capability shipped without the interface a tenant would buy it through, on the reasoning that
  Paridyum needs it now and tenants can have it later.
- A service whose only conceivable consumer is Paridyum itself.
- Special-casing Paridyum's tenant anywhere — provisioning, entitlement, quota, billing or
  support — beyond what is published and available to any tenant on the same terms.

## The boundary, which is not a loophole

**This governs runtime capability: `platform-*`, `svc-*`, `lib-*`, and catalogue products.** It
does not govern `tool-*` or `infra-*`.

`tool-forge` is a factory and `infra-ci` is a pipeline. ADR-0042 already made `tool-generate`
`class: tool`, `deployable: false`, *"build-time, internal, never in a request path."* Those are
not products, will never be sold, and forcing them through a tenant would be ceremony with no
customer at the end of it.

**The test is whether the thing runs in a request path.** If a tenant's traffic can reach it, it is
a product and this principle binds. If it only ever runs at build time or in a pipeline, it does
not. Stated plainly because an unbounded version of this rule would either block Paridyum's own
tooling or be ignored, and a principle that is routinely ignored teaches that principles are
optional.

## Costs accepted

C-0001 principle 8: *an artifact that lists only benefits is incomplete and probably wrong.*

1. **Everything is slower to first working state.** The shortest path to a running invoicing
   business is to build invoicing, not to build tenancy, identity, entitlement and routing and
   then build invoicing on top of them. This principle forbids the shortcut, and the cost is paid
   in full at the start, before any revenue exists to justify it. That is C-0001 principle 1's
   cost, restated where it will actually be felt.

2. **Paridyum's own operations inherit every limitation it sells.** When a tenant cannot do
   something, neither can we, and the temptation to grant ourselves the exception arrives exactly
   when it is most expensive to refuse — during an incident, with our own customers affected.

3. **It cannot be fully checked.** ADR-0049 names what is mechanically rejectable; the residue is
   judgement. A determined engineer can satisfy every check and still build a capability shaped
   around one consumer.

4. **The `tool-`/`infra-` boundary will be argued over.** Something will sit near the line — a
   support console, an internal dashboard reading tenant data — and the honest answer is usually
   that if it reads tenant data it is in a request path and therefore a product. Expect the
   argument; do not expect the boundary to move quietly.

## Related

- C-0001 — principle 1 *platform before product* and principle 6 *every service is replaceable*, which this specifies
- **ADR-0049** — the locked mechanical rules this principle generates
- BET-0005 — Paridyum operates the catalogue it franchises; this is what makes that survivable
- RFC-0022 — the operator-tenant conflict, whose exposures stay informational only because of this
- ADR-0024, GLO-0005 — nested tenancy, and operator scope as a grant rather than an absence
- ADR-0042 — `class: tool`, `deployable: false`, the boundary this adopts
- C-0006 — business semantics as one inherited model, which this is the construction rule for
