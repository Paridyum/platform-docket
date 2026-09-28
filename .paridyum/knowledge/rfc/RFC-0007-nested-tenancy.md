---
id: RFC-0007
title: How the inner silo is isolated when tenants are themselves multi-tenant
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0024
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [tenancy, isolation, postgres, rls, nested-tenancy, density]
breaking: true
enforced_by: []
---

# RFC-0007 — How the inner silo is isolated when tenants are themselves multi-tenant

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0115.** The tree below puts an **operator** above the tenant.
> That party is the **org** (GLO-0012), and it is not a silo: business data carries `tenant_id` and
> `account_id` only, and an org's own rows are a separate `org` scope (ADR-0115 D1, D9). The
> comparison of mechanisms below is unchanged and still the basis of ADR-0115.

## Status

**ACCEPTED**, resolved by ADR-0024.

## The question

RFC-0006 and ADR-0023 modelled tenancy as one level: Paridyum has tenants, each tenant has
data, `tenant_id` separates them. That model is incomplete, and the omission is structural
rather than a detail.

**Paridyum's tenants are themselves multi-tenant applications.** A tenant built under BET-0001
is a software product with customers of its own, and those customers need isolation from each
other. So there are two nested silos, not one:

```
Paridyum → operator → tenant → account
                      OUTER    INNER
```

ADR-0023 is locked and says `tenant_id` on every tenant-scoped table. That remains true. What
it also does is **imply that one identifier is sufficient**, and an artifact that implies
sufficiency where there is none is worse than an absent one — the first migration written from
ADR-0023 alone would carry a single discriminator and no way to separate a tenant's own
customers.

So: **how is the inner silo isolated, and is it ours to isolate at all?**

## Why the second question comes first

The tempting simplification is that the inner silo is not our problem. We isolate tenants; what
a tenant does with its own customers is application logic, in an application we do not write.

**Rejected, and this is the most important paragraph in this document.**

- C-0006's not-commodity list claims *business semantics as one inherited model* and *provable
  tenant isolation*. Multi-tenancy is the hardest of those semantics and the one whose failure
  is unrecoverable. Declining the inner silo means declining the largest single item on that
  list.
- Under BET-0003 the application layer is **generated**. If the runtime does not provide the
  inner silo, the generator must emit correct multi-tenant isolation per application, with no
  primitive to emit against. That is precisely the class of subtly-wrong output BET-0003 warns
  about: *"its permissions, tenancy and billing semantics are subtly wrong in ways no reviewer
  catches until a customer does."*
- Every operator would hand-roll it, most would get it wrong, and the breach would be on our
  platform. The reputational blast radius does not respect the contractual boundary.
- It is the clearest available demonstration of the difference between selling infrastructure
  and selling a runtime — a tenant *inherits* its own multi-tenancy rather than implementing it.

So the inner silo is ours. The remaining question is mechanism.

## Constraints inherited before any option is compared

Unchanged from RFC-0006, and now applied twice over:

- **BET-0001** puts many low-price, mostly-idle tenants on the platform. Per-tenant
  infrastructure is unaffordable; per-*account* infrastructure is unaffordable by another two
  orders of magnitude, since accounts outnumber tenants.
- **BET-0002** charges a flat price per tenant. Density is margin, and accounts are free to
  create.
- **REQ-0008** requires migrations be deployable without downtime, which eliminates anything
  O(tenants) or O(accounts).

## Options

### A. Two columns in shared tables, two-predicate row-level security

`tenant_id` on every tenant-scoped table; `account_id` additionally on account-scoped tables,
never alone. RLS policies test both, with the inner predicate conditional on operator scope.

- **Cost at rest:** rows. Zero per account.
- **Isolation:** database-enforced at both levels, provided RLS is `FORCE`d and both session
  values are untrusted from the client.
- **Migration cost:** O(1). One schema.
- **The specific hazard:** the inner predicate has to permit an operator reading all accounts.
  If that breadth is expressed as *the absence of an account filter*, then any bug that loses
  the value escalates privilege silently — the query runs, rows return, RLS is applied exactly
  as written to the wrong scope, and nothing errors. GLO-0005 exists for this and turns it into
  a positive grant.
- **Second hazard:** `account_id` used without `tenant_id`. Surrogate values collide across
  tenants, so a filter on `account_id` alone crosses the outer boundary. Mechanically checkable.

### B. One hierarchical path column

A materialised path or `ltree` value per row — `tenant.account` — with RLS matching on prefix.
Generalises to any depth.

- Uniform, and the only option that extends to a third level without a schema change. That
  matters more than it first appears: a reseller or white-label partner tier above the operator
  is a plausible consequence of BET-0001's beachhead section, and so is a sub-account level
  inside an account.
- **Rejected for now.** Prefix-match predicates index worse than equality on two columns, which
  is the hot path on every query in the system. It is also harder to reason about at review
  time, and "can I read this row" becoming a string operation is a poor trade when the failure
  mode is a cross-tenant leak. Premature generality, at the cost of the one property that has
  to be fast.
- **Worth revisiting on a stated trigger**, recorded in ADR-0024: a genuine third level with a
  real customer behind it. Two explicit columns can be migrated to a path; a path cannot easily
  be migrated back.

### C. Schema per tenant, accounts as rows within

- Better inner failure mode, and per-tenant restore becomes easy.
- **Rejected**, on exactly RFC-0006's grounds: migrations become O(tenants) against REQ-0008's
  no-downtime requirement, and that ceiling arrives when the company is succeeding.

### D. The operator implements the inner silo

Rejected in *Why the second question comes first* above. It is the cheapest option and it
gives away the product.

### E. A "tenant" row that can contain other tenants — self-referential single table

One `tenants` table with `parent_id`, and no separate account concept.

- Superficially elegant, and it is what the two-level model looks like if you squint.
- **Rejected on vocabulary, which turns out to be a real cost rather than a stylistic one.** The
  two levels have different isolation guarantees, different consequences on breach, and
  different rules about who may read across them. A single word for both means every artifact,
  every check name and every code review has to re-establish which level is meant, and the one
  time somebody assumes wrong is a cross-tenant leak. GLO-0002 records the decision to keep
  `tenant` unambiguous; this option is what it was made against.

## Discussion

**A is the only option that satisfies the inherited constraints.** B, C and E fail on
performance, migration cost and clarity respectively; D fails on strategy.

**A's danger is worse than ADR-0023's, in a specific way.** ADR-0023's hazard was a *missing*
column — an omission, which a check can see. A's new hazard is a *permissive default*: breadth
granted by absence. Absence is what every kind of bug produces. So the two additions ADR-0024
makes over ADR-0023 are not symmetrical with it; the operator-scope rule is the load-bearing
one, and it is a rule about how a grant is *represented*, not about what exists in the schema.

**Three table categories, and the third cannot be inferred.** Platform-owned, tenant-scoped,
account-scoped. `invoices` in a billing product is account-scoped; `plans` that product offers
is tenant-scoped; `regions` is platform-owned. Nothing in the names distinguishes them, so the
author declares it and review sees the declaration. Defaulting to the most restrictive —
account-scoped — was considered and rejected: it would make the common tenant-scoped case
require an opt-out, and an opt-out that is used constantly stops being read.

**Downstream promises acquire two readings each**, and this is the part most likely to be
forgotten. REQ-0013's export means an operator taking a tenant *or* an account taking its own
slice. Per-tenant restore becomes per-account restore. ANTI-0007 gets worse: a breach at the
inner boundary exposes people who never chose to trust either us or the operator.

## Recommendation

**Option A, superseding ADR-0023 rather than extending it** — because ADR-0023 implies one
identifier suffices, and a locked artifact that misleads about sufficiency is more dangerous
than one that is absent. Option B's path model is deferred behind a named trigger.

Resolved by ADR-0024.

## Related

- ADR-0024 — the decision
- ADR-0023 — superseded; its option analysis and arithmetic are reused unchanged
- RFC-0006 — the five single-level options, still the basis for rejecting C
- GLO-0001 to GLO-0005 — the vocabulary this RFC required before it could be written
- BET-0003 — why the inner silo has to be a runtime primitive
- REQ-0008, REQ-0013, ANTI-0007 — the three promises this doubles
