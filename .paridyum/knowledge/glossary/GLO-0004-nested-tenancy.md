---
id: GLO-0004
title: Nested tenancy — a multi-tenant platform hosting multi-tenant applications
type: glossary
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [tenancy, isolation, architecture, vocabulary]
breaking: false
enforced_by: []
---

# GLO-0004 — Nested tenancy

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0114, ADR-0115.** The tree below now shows the
> **org** (GLO-0012) — the customer of paridyum.in — as the **owner** of tenants: above the silos,
> and not one of them. Business data still has exactly two silos. There are **four** table
> categories now, not three (ADR-0115 D1). The dated notes further down quote older trees on
> purpose; they are the record of how the drawing changed, not the current model.

## Definition

**Nested tenancy is Paridyum's defining structural property: a multi-tenant platform whose
tenants are themselves multi-tenant applications. Two silos, one inside the other.**

```
Paridyum                                the platform that runs them. Not a silo, and not a grant.
  └── org                     GLO-0012  the customer of paridyum.in. OWNS tenants; not a silo.
        │                               org_id on its own rows only; tenants.owner_org_id
        └── tenant            GLO-0002  a business. OUTER silo.  tenant_id
              └── account     GLO-0003  a unit it serves. INNER silo. account_id

              operator scope  GLO-0001  a grant HELD OVER a tenant, spanning its accounts.
                                        Not a level: it is what reads across the inner silo.
```

**The org is drawn above the tenant because it owns it, not because it contains it.** No tenant- or
account-scoped row names its org; the org's own rows — members, invitations, its profile — are a
separate `org` scope, isolated from other orgs by their own predicate (ADR-0115 D1, D2, D9).

Two independent isolation boundaries, not one boundary with extra columns:

| Boundary | Crossing it means | Severity |
| -------- | ----------------- | -------- |
| tenant ↔ tenant | one business reads another business's data, including a competitor's | catastrophic, reportable |
| account ↔ account, same tenant | one of a business's units reads another's — a customer, a division, or a second instance of one product | catastrophic for that business, and it is their reputation, not ours |

> **The inner unit was narrowed to "customer" until 2026-09-20 and is now "a unit it serves"**
> (ADR-0104, correcting GLO-0003). A second instance of one product — two dockets under one tenant
> — is an account. The mechanism is unchanged; the assumed kind of unit widened.

> **The tree above had a fourth level until 2026-09-26, and it was wrong** (ADR-0109 D5). It read
> `Paridyum → operator → tenant → account`, with `operator` as the party that pays and owns 1..N
> tenants. That is *not* what `operator` means in this system: GLO-0001 is corrected to the runtime
> grant — breadth **within** one tenant — which is the reading twelve applied migrations, a published
> library API and `tool-docket` already enforce, and which point 1 below has always described.
>
> So the commercial party is gone from the tree rather than renamed, because **it is not modelled**:
> it is ADR-0024 decision 9's deferred third level, and D9 admits it only on *"a real third level
> with a paying customer behind it… **Not 'we can imagine one.'**"* A paying party may still hold
> several tenants — BET-0002 prices per running business, plural — and that fact now has no single
> word, deliberately.
>
> **Two silos, not three, and that was always the count.** This entry's own title says two, its
> table has two rows, and the tree drew four levels. The drawing was the odd one out.

## Why this is not "just another column"

The obvious reading — add `account_id` next to `tenant_id` and carry on — misses three things,
each of which produces a different class of defect:

1. **Row-level security needs two predicates, and the second one has a default that leaks.**
   The outer predicate is unconditional. The inner one must be conditional, because a tenant's
   own administrator — a principal holding operator scope — legitimately reads every account of
   its tenant. Expressing "no
   account filter" as an absent or null session value means **any bug that clears the value
   silently escalates an end user to see the whole tenant.** GLO-0005 exists because that
   default must be an explicit grant instead.

2. **Four table categories, not two** (ADR-0115 D1). Platform-owned, org-scoped, tenant-scoped
   and account-scoped. None can be inferred from the schema: `invoices` in a billing product is
   account-scoped, `plans` offered by that product is tenant-scoped, `org_members` is org-scoped and
   `regions` is platform-owned. Nothing about the names says which, so it must be declared.

3. **Every downstream promise now has two readings.** Export (REQ-0013) means a tenant taking
   its whole estate, or an account taking its own slice — different products with different
   completeness rules. Per-tenant restore becomes per-account restore too. Regulated data
   (ANTI-0007) gets worse, since a breach at the inner boundary exposes people who never chose
   to trust either us or the tenant.

## Why the outer level is ours and the inner level is also ours

It would be simpler if the inner silo were the tenant's own problem — their application, their
data model. It is not, and that is the product.

C-0006's not-commodity list claims *business semantics as one inherited model*. Multi-tenancy
is the hardest of those semantics, the one every founder rebuilds badly, and the one whose
failure is unrecoverable. **A tenant inherits its own multi-tenancy from the runtime rather
than implementing it.** That is a large part of what BET-0003's contract has to express, and it
is the clearest example of the difference between selling infrastructure and selling a runtime.

## Related

- GLO-0012 org — the owner above the silos
- GLO-0001, GLO-0002, GLO-0003 — the three levels
- GLO-0001 operator — the grant that reads across the inner silo (corrected by ADR-0109)
- GLO-0005 operator scope — the dangerous default this creates
- ADR-0115 — the decision; supersedes ADR-0024, which superseded the single-level ADR-0023
- RFC-0007 — the options compared
- BET-0003 — the contract that must express this
- REQ-0013 — export, which now has two meanings
