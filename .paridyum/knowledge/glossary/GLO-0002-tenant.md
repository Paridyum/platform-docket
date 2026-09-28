---
id: GLO-0002
title: Tenant — one business running on Paridyum, the outer silo
type: glossary
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [tenancy, identity, vocabulary, isolation]
breaking: false
enforced_by: []
---

# GLO-0002 — Tenant

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0114.** A tenant is **owned by exactly one org**
> (GLO-0012), recorded as `tenants.owner_org_id` — read *org* wherever this entry says a tenant is
> owned by an *operator*. An org owns tenants and is not one; "business" on screen means a tenant.

## Definition

**A tenant is one business running on Paridyum: the unit of isolation, of branding, of custom
domain, and of billing. It is the OUTER of the two silos.**

Column: `tenant_id`. Owned by exactly one operator (GLO-0001). Contains 0..N accounts
(GLO-0003).

A tenant is the thing C-0006 means by "a running business" and BET-0002 means by "per running
business". When an artifact says *business* without qualification, it means a tenant.

## The word "tenant" is doing double duty, and that is the hazard

Paridyum is a multi-tenant platform hosting **applications that are themselves multi-tenant**.
So "tenant" is ambiguous unless the level is stated, and the ambiguity is dangerous rather
than merely untidy: the two levels have different isolation guarantees, different failure
consequences, and different columns.

**In this repository, `tenant` always means the outer level — the business.** The inner level
is an `account` and is never called a tenant. GLO-0004 defines the pair.

## What it is not

- **Not the operator.** One operator, many tenants.
- **Not an account.** An account is a customer *of* a tenant. A tenant's own multi-tenancy is
  accounts, not nested tenants, and the vocabulary is fixed so that no artifact has to say
  "tenant" and mean either.
- **Not a deployment.** ADR-0024 provisions no per-tenant infrastructure. A tenant is rows and
  a routing entry, not a workload.

## Related

- GLO-0001 operator — who owns a tenant
- GLO-0003 account — the inner silo
- GLO-0004 nested tenancy — why the levels must be named separately
- ADR-0024 — `tenant_id` on every tenant-scoped table, RLS enabled and forced
- ANTI-0007 — why a tenant is not an acceptable home for regulated data
