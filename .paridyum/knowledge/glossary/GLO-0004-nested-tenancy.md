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

## Definition

**Nested tenancy is Paridyum's defining structural property: a multi-tenant platform whose
tenants are themselves multi-tenant applications. Two silos, one inside the other.**

```
Paridyum
  └── operator            GLO-0001  pays us; owns 1..N tenants
        └── tenant        GLO-0002  a business. OUTER silo.  tenant_id
              └── account GLO-0003  a unit it serves. INNER silo. account_id
```

Two independent isolation boundaries, not one boundary with extra columns:

| Boundary | Crossing it means | Severity |
| -------- | ----------------- | -------- |
| tenant ↔ tenant | one business reads another business's data, including a competitor's | catastrophic, reportable |
| account ↔ account, same tenant | one of a business's units reads another's — a customer, a division, or a second instance of one product | catastrophic for that business, and it is their reputation, not ours |

> **The inner unit was narrowed to "customer" until 2026-09-20 and is now "a unit it serves"**
> (ADR-0104, correcting GLO-0003). A second instance of one product — two dockets under one tenant
> — is an account. The mechanism is unchanged; the assumed kind of unit widened.

## Why this is not "just another column"

The obvious reading — add `account_id` next to `tenant_id` and carry on — misses three things,
each of which produces a different class of defect:

1. **Row-level security needs two predicates, and the second one has a default that leaks.**
   The outer predicate is unconditional. The inner one must be conditional, because an
   operator's own console legitimately reads every account of its tenant. Expressing "no
   account filter" as an absent or null session value means **any bug that clears the value
   silently escalates an end user to see the whole tenant.** GLO-0005 exists because that
   default must be an explicit grant instead.

2. **Three table categories, not two.** Platform-owned, tenant-scoped, and account-scoped. The
   third cannot be inferred from the schema: `invoices` in a billing product is account-scoped,
   `plans` offered by that product is tenant-scoped, and `regions` is platform-owned. Nothing
   about the names says which, so it must be declared.

3. **Every downstream promise now has two readings.** Export (REQ-0013) means an operator
   taking a tenant, or an account taking its own slice — different products with different
   completeness rules. Per-tenant restore becomes per-account restore too. Regulated data
   (ANTI-0007) gets worse, since a breach at the inner boundary exposes people who never chose
   to trust either us or the operator.

## Why the outer level is ours and the inner level is also ours

It would be simpler if the inner silo were the operator's problem — their application, their
data model. It is not, and that is the product.

C-0006's not-commodity list claims *business semantics as one inherited model*. Multi-tenancy
is the hardest of those semantics, the one every founder rebuilds badly, and the one whose
failure is unrecoverable. **A tenant inherits its own multi-tenancy from the runtime rather
than implementing it.** That is a large part of what BET-0003's contract has to express, and it
is the clearest example of the difference between selling infrastructure and selling a runtime.

## Related

- GLO-0001, GLO-0002, GLO-0003 — the three levels
- GLO-0005 operator scope — the dangerous default this creates
- ADR-0024 — the decision; supersedes the single-level ADR-0023
- RFC-0007 — the options compared
- BET-0003 — the contract that must express this
- REQ-0013 — export, which now has two meanings
