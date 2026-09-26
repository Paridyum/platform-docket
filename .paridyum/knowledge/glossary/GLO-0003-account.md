---
id: GLO-0003
title: Account — a customer of a tenant, the inner silo
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

# GLO-0003 — Account

## Definition

**An account is the unit one instance of a tenant's product serves: the INNER silo. It is
isolated from other accounts of the same tenant, and it has no relationship with Paridyum.**

> **Widened by ADR-0104 on 2026-09-20.** This read *"one customer of a tenant's product"* until
> then, and the narrowness was load-bearing in the wrong direction: ADR-0102 reasoned correctly
> from it that docket — whose model has no customers — needed no inner silo, and was superseded
> hours later when a tenant running **two dockets** turned out to need exactly one. A unit may be a
> customer, a division, a jurisdiction or a subsidiary. What makes it an account is that its data
> must not mix with its siblings' under the same tenant, and that it has no relationship with
> Paridyum. **Both original properties are unchanged; only the assumed kind of unit widened.**

Column: `account_id`, always alongside `tenant_id` and never instead of it. Belongs to exactly
one tenant (GLO-0002).

If a tenant is a CRM product, its accounts are the companies using that CRM. Their data must
not mix, and the tenant's operator can see all of it.

If a tenant runs **two instances of one product** — two dockets, for two divisions — each instance
is an account, for the same reason and with the same mechanism. ADR-0024 decision 7 already made
this affordable before anything needed it: *"accounts are free to create."*

## Why this term is load-bearing

BET-0001's `trigger` — the condition that would kill the wedge — turns on whether a tenant
*"collects a payment from an end user of their own"*. That party is an account. The falsification
condition of the company's primary commercial bet was written against an undefined term, which
is how this entry came to exist.

## Every user has exactly one account

Added 2026-08-08. A solo user — a consumer of a tenant's product who belongs to no company —
holds a **personal account** containing only themselves. The pattern GitHub, Slack and Notion
all use, and it is here for a structural reason rather than a product one.

`account_id` must be **total**. ADR-0038 makes `paridyum.current_account()` raise on an unset
value, because a nullable discriminator inside an RLS policy is GLO-0005's absence hazard at the
exact point the inner silo is enforced. If some users had no account, that accessor would need a
nullable path and the invariant would be gone.

Upgrading a personal account to a shared one is adding members, not migrating a model.

## What it is not

- **Not a tenant.** Never called one. See GLO-0002 on why the ambiguity is dangerous rather
  than untidy.
- **Not a billing account of Paridyum's.** Paridyum bills the operator (GLO-0001) and has no
  contractual relationship with an account. Money flows operator → Paridyum and account →
  operator, on separate rails.
- **Not necessarily an organisation.** For a business-to-consumer tenant an account may be one
  person, which is the degenerate case rather than a different shape. BET-0001's beachhead
  section forbids assuming either presence or absence of accounts, so the model must carry both
  without a special case.

## Consequences that are easy to get wrong

- **`account_id` never appears without `tenant_id`.** An account id alone is not a key; two
  tenants may hold accounts with the same surrogate value, and a query filtered on
  `account_id` alone crosses the outer silo. ADR-0024 makes this a check.
- **A tenant-scoped table is not automatically account-scoped.** A tenant's own configuration,
  branding and plan belong to the tenant, not to any account. The distinction has to be
  declared, because it cannot be inferred from a column name.

## Related

- GLO-0002 tenant — the outer silo
- GLO-0004 nested tenancy — the pair
- GLO-0005 operator scope — the privileged read across all accounts of a tenant
- ADR-0024 — the two-level decision
- BET-0001 — whose trigger depends on this term
