---
id: STD-0009
title: Tenant isolation testing — two boundaries, every access path, and the negative case
type: standard
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["svc-*", "platform-*", "web-*"]
tags: [testing, tenancy, nested-tenancy, isolation, authorisation, rls, security]
breaking: false
verified_by:
  - TST-0016
  - TST-0017
  - TST-0018
  - TST-0019
  - TST-0020
  - TST-0021
  - TST-0027
enforced_by:
  - check: test.tenant_isolation_suite_covers_owned_tables
    severity: error
    rationale: >
      A table with a policy and no test has been proven to have a policy. ADR-0024 decision 3
      names the case where that is exactly not enough — an outer-only policy on an
      account-scoped table, which every static check passes and every dashboard reports as
      healthy.
  - check: test.operator_scope_denial_is_asserted
    severity: error
    rationale: >
      GLO-0005's rule fails in the permissive direction, so only the negative test
      distinguishes a correct implementation from the dangerous one. A suite asserting that
      an operator can see everything passes identically against both.
  - check: test.authorization_matrix_has_no_untested_cell
    severity: error
    rationale: >
      Authorisation defects are grants, not denials, and a grant nobody asked about is
      invisible. An untested cell is the default state of every matrix that is not checked
      for completeness.
---

# STD-0009 — Tenant isolation testing

Paridyum's tenants are **themselves multi-tenant** (GLO-0004). A tenant is a software product
with customers of its own, and those customers are `accounts`. So every isolation question has
two answers, and the second one is the dangerous one: a leak between tenants is our failure, and
a leak between a tenant's own accounts is a promise **they** made to people who never heard of
us. ADR-0024 cost 2 states it plainly — *"there is no version of it where we are the injured
party."*

This standard is the runtime half of ADR-0024. The schema half already exists and is already
gated: `scripts/check-migrations.sh` reads the SQL and rejects six defect classes, each proven
by a fixture that must fail.

## What static analysis cannot do

The distinction is the reason this standard exists, and it is worth stating precisely.

| Provable by reading the migration | Provable only by running a query as a role |
| --- | --- |
| a policy exists on the table | the policy's predicate is **correct** |
| RLS is `ENABLE`d **and** `FORCE`d | the runtime role cannot escape it in practice |
| `account_id` never appears without `tenant_id` | a query filtered by both actually returns nothing |
| the table declared a tenancy category | the category matches how the code uses it |
| no `NULL`/wildcard grants operator breadth | a session missing its claim returns **zero rows** |

Everything in the left column passes today. ADR-0024 decision 3 describes what the right column
catches: an outer-only policy on an account-scoped table *"isolates businesses from each other
and nobody's customers — which every dashboard will report as healthy."*

That sentence is the specification for this suite.

## The rule

**Every access path to tenant data is tested from outside the boundary it is supposed to
respect, in both directions, with the denial asserted rather than the permission.**

### ISO-001 — Both boundaries, always

Every case is written twice: once across tenants, once across accounts **within one tenant**.
A suite that only crosses the outer boundary passes against a schema with no `account_id`
predicate at all.

```
tenant A / account 1   →  tenant A / account 1   allowed
tenant A / account 1   →  tenant A / account 2   DENIED      ← the inner boundary
tenant A / account 1   →  tenant B / account 1   DENIED      ← the outer boundary
tenant A / operator    →  tenant A / account 2   allowed, and only with an explicit grant
tenant A / operator    →  tenant B / anything    DENIED      ← there is no cross-tenant scope
```

The last line is not a corner case. GLO-0005: operator scope is bounded by one tenant, and an
operator owning several tenants holds it in each **separately, never simultaneously in one
session**.

### ISO-002 — The negative case is the test

Assert the denial. A suite built from permissions — "the operator can see every account" —
passes identically against the implementation where breadth comes from a missing filter, which
is the implementation GLO-0005 exists to forbid.

The specific assertion for operator scope is: **a session with no explicit operator claim reads
zero rows outside its own account, and does so loudly.** Not fewer rows. Zero, and an error the
caller can see.

### ISO-003 — Every command, not every table

RLS policies are declared per command. `FOR SELECT` and `FOR ALL` are different statements, and
a suite that only reads will never discover that only one of them was written. Read, insert,
update and delete, each crossing each boundary.

Delete deserves separate mention because its failure is not a leak: it is destruction of another
tenant's data, and it is the only one of the four that cannot be undone by fixing the bug.

### ISO-004 — Identifiers come from the session, and the test proves it

ADR-0024 decision 5: both identifiers come from the verified session only — never a request
body, query string, path segment or client header. The test is that a request **carrying** a
`tenant_id` or `account_id` is **rejected**, not that it is ignored. Ignoring is
indistinguishable from honouring until the day some handler reads it.

### ISO-005 — Every path, not only the request path

This is the item most often missed, and nested tenancy makes each entry worse rather than
equally bad. The request carried a session; none of these do.

| Path | What nested tenancy adds |
| ---- | ------------------------ |
| background jobs, scheduled tasks | the tenant arrives as an argument somebody remembered to pass |
| queue consumers | a message shaped before the account boundary existed carries only `tenant_id` |
| cache keys | a key with `tenant_id` and no `account_id` leaks **between a tenant's own customers** |
| search indexes | one index per tenant is not enough; the filter must be per account too |
| object storage prefixes | a per-tenant bucket prefix has exactly the same defect as a cache key |
| webhooks and notifications | the destination is tenant-owned; the payload may be account-scoped |
| exports and reports | REQ-0013's export forks per tenant *and* per account (ADR-0024 cost 4) |
| logs, traces and analytics | tenant data in a shared index is a leak with a longer fuse |
| admin and support tooling | the highest-privilege path and usually the least tested |

A cross-tenant test suite covering only HTTP is a suite covering the place everybody already
looks.

### ISO-006 — The matrix is a property, not an enumeration

An authorisation matrix written by hand is `roles × resources × actions` entries that grow
multiplicatively and are maintained by whoever last remembered. Express it as a property
instead (TST-0011):

> For all principals P and resources R: if P holds no grant covering R, every operation on R
> returns denied.

Enumerating the allowed cells is still worth doing — they are few, and they are the
specification. It is the denied cells that must be generated, because they are the ones that
grow and the ones that fail silently.

### ISO-007 — The fixture tenants are hostile, and there are at least four

Two tenants, two accounts each. Fewer than that and some boundary is untested by construction:
one tenant cannot test the outer boundary, one account cannot test the inner one.

Seed them with **distinguishable** data. Row counts that happen to match, or ids that collide
across tenants, produce tests that pass for the wrong reason — and account surrogate values
**do** collide across tenants, which is the whole argument for ADR-0024 decision 2.

### ISO-008 — Isolation tests run in the `pr` tier

They are the one class of test where finding the defect later has no acceptable version. They
are also cheap: a real Postgres in a container, four fixture tenants, a table of assertions.
STD-0008's 10-minute budget accommodates them; if it stops doing so, something else moves.

## Authentication is separate, and comes first

TST-0020 is its own type because "login works" is one case out of roughly fifteen and it is the
only one an implementation can pass while being wrong. Expired, revoked, replayed, wrong issuer,
wrong audience, wrong signature, a session not invalidated at logout, a reset token reused, an
account not locked after repeated failure. **The negative cases are the suite.**

Isolation testing assumes the session is authentic. If that assumption fails, none of the rest
of this standard is testing anything.

## What this does not cover

Whether the *design* is right. That is threat modelling, and REQ-0015 §4 already names
multi-tenancy as the standing threat for this platform: *"a tenant-facing design that does not
consider a tenant as the attacker has not been threat-modelled."* This standard tests an
implementation against a boundary that has already been decided.

## Related

- [ADR-0024](../adr/ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) — the decision this verifies; decisions 2–5 are ISO-001 to ISO-004
- [GLO-0004](../glossary/GLO-0004-nested-tenancy.md) · [GLO-0005](../glossary/GLO-0005-operator-scope.md) — the vocabulary, and the rule ISO-002 exists for
- [GLO-0002](../glossary/GLO-0002-tenant.md) · [GLO-0003](../glossary/GLO-0003-account.md) — the two boundaries
- `scripts/check-migrations.sh` — the schema half, already gated and already proven by fixtures
- [STD-0008](STD-0008-test-tiers-and-gates.md) — the tier these run in
- [REQ-0015](../requirements/REQ-0015-threat-modelling-and-risk-register.md) — §4, the design half
- [REQ-0013](../requirements/REQ-0013-tenant-data-portability.md) — export, which forks at both boundaries
- [ANTI-0007](../anti-patterns/ANTI-0007-regulated-data-before-controls.md) — what a leak costs when the data is regulated
