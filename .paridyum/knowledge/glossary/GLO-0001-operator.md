---
id: GLO-0001
title: Operator — the party that pays Paridyum
type: glossary
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [tenancy, identity, vocabulary, billing]
breaking: false
enforced_by: []
---

# GLO-0001 — Operator

## Definition

**An operator is the party that holds a Paridyum subscription and runs one or more tenants.**

The operator is our customer. Nobody else in the hierarchy is.

Column: `operator_id`. One operator owns 1..N tenants (GLO-0002).

## What it is not

- **Not a tenant.** BET-0002 prices per running business, plural, so one operator may own
  several tenants. Conflating the two makes the billing model unrepresentable — this is the
  specific error the term exists to prevent.
- **Not a user.** An operator is an organisation or an individual acting as one. The people who
  sign in on its behalf are members of it; a solo founder is an operator with one member, which
  is the degenerate case and not a different shape. BET-0001's beachhead section forbids
  assuming one operator per business precisely here.
- **Not an account.** An account (GLO-0003) is a customer of a tenant, two levels down, and
  has no relationship with Paridyum at all.

## Why the word

"Customer" was rejected as ambiguous: an operator is *our* customer and has customers of its
own, so the unqualified word is unusable in any sentence about money. "Business owner" is the
term used in conversation and is close, but names a role held by a person rather than the
billable party, which is what the schema needs.

"Operator" also carries the right connotation for C-0006's posture — the operator runs the
business; Paridyum runs the runtime.

## Related

- GLO-0002 tenant — what an operator owns
- GLO-0003 account — a customer of a tenant, not of Paridyum
- GLO-0004 nested tenancy — the two-level model these terms describe
- ADR-0024 — the tenancy decision that requires this distinction
- BET-0002 — prices per tenant, which is why operator and tenant cannot be the same object
