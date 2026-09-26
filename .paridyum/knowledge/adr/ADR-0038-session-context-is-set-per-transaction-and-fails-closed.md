---
id: ADR-0038
title: Session context is set per transaction from a verified token, and every accessor fails closed
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library]
applies_to: ["platform-*", "svc-*", "lib-*"]
tags: [tenancy, rls, postgres, identity, session, security, adr-0024, glo-0005]
breaking: false
enforced_by:
  - check: db.session_context_accessors_fail_closed
    severity: error
    rationale: >
      A `RETURN NULL` where a `RAISE` belongs turns every RLS policy that reads it into a
      predicate comparing against NULL, which is never true — or, in the operator-scope case,
      into a predicate that is trivially true. Both outcomes are silent. This is the single
      line of SQL on which ADR-0024's entire isolation model rests, and it is decidable by
      reading the function body.
  - check: test.account_assertion_is_exercised
    severity: error
    rationale: >
      Decision 5 is redundant with RLS on purpose, and redundant code rots unnoticed: while RLS
      works the assertion never fires, and one that never fires is indistinguishable from one
      that was deleted. Only a test that plants a foreign account_id keeps it real.
  - check: db.no_tenant_id_from_client_input
    severity: error
    rationale: >
      Already declared under ADR-0024, and this ADR is what finally gives it a definition of
      "client input": anything not derived from a signature-verified token. A tenant id taken
      from a header, a body, a path parameter or a custom JWT claim is all the same defect.
---

# ADR-0038 — Session context is set per transaction and fails closed

## Status

**ACCEPTED** as of 2026-08-08. Created `accepted` per `new-decision` §5b.

## Context

`tests/fixtures/migrations/pass/migrations/0001_init.sql` — the fixture the migration gate
treats as the canonical correct schema, and the file every future migration will be pattern
matched against — contains this policy:

```sql
CREATE POLICY invoices_scoped ON invoices
    USING (tenant_id = paridyum.current_tenant()
           AND (paridyum.has_operator_scope() OR account_id = paridyum.current_account()));
```

**None of those three functions is defined anywhere in this organisation.** A repository-wide
search for `CREATE FUNCTION` returns nothing. STD-0006 §2 prints the same policy as the shape
every account-scoped table must adopt. So the platform's entire isolation model is expressed in
terms of three functions that do not exist, and whoever writes the first migration will define
them — silently, in passing, while thinking about something else.

ADR-0024 says exactly what that costs: *"The first migration must establish three categories,
the two-column convention, the two-predicate policy, the two-role split, and operator scope as a
claim. None of it is a later refactor."*

Two further facts make this urgent rather than tidy. Identity is now Zitadel behind
`platform-identity`, so there is a specific, known answer to *where the values come from* —
which did not exist when ADR-0024 was written. And GLO-0005 states the failure mode precisely:
**operator scope is "a grant, never an absence"**, because *"absence is what every dropped
session field, uninitialised variable, null lookup and middleware ordering bug produces."* An
accessor returning `NULL` on an unset session is that hazard, installed at the root.

## Decision

**1. Three accessors, in a `paridyum` schema, and they are the only way a policy learns who is
asking.**

```sql
paridyum.current_tenant()      -> uuid     -- never NULL
paridyum.current_account()     -> uuid     -- never NULL
paridyum.has_operator_scope()  -> boolean  -- never NULL
```

**2. Every accessor raises on an unset or empty setting. None returns `NULL`. Ever.**

```sql
CREATE FUNCTION paridyum.current_tenant() RETURNS uuid
  LANGUAGE plpgsql STABLE AS $$
DECLARE v text := current_setting('paridyum.tenant_id', true);
BEGIN
  IF v IS NULL OR v = '' THEN
    RAISE EXCEPTION 'paridyum.tenant_id is not set'
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN v::uuid;
END $$;
```

`has_operator_scope()` follows the same shape and returns the parsed boolean — **it does not
default to `false` on an unset setting.** A default of `false` is safe for reads and wrong as a
rule: it converts a configuration failure into a silently narrower result set, which nobody
reports and which hides the bug until the day the value is needed and still absent.

**3. The settings are set with `SET LOCAL`, inside the transaction, by `lib-auth-typescript`.**

> **Named `lib-auth` when this was written.** The repository was renamed on 2026-09-24 under
> ADR-0107, when the same capability needed a Rust sibling and two repositories wanted one name.
> The published package — `@paridyum/auth` — did not change, so every code reference in this ADR
> still resolves. STD-0022 requires the live name here rather than the historical one.

```sql
SET LOCAL paridyum.tenant_id     = '...';
SET LOCAL paridyum.account_id    = '...';
SET LOCAL paridyum.operator_scope = 'true';
```

`SET LOCAL` and not `SET`: connections are pooled, and a session-scoped setting outlives the
request that set it. A pooled connection carrying the previous request's `tenant_id` is a
cross-tenant read that no test will find, because it only appears under concurrency.

**4. The values are derived from a signature-verified token and from nothing else.**

| Setting | Source |
| --- | --- |
| `paridyum.tenant_id` | claim `urn:zitadel:iam:user:resourceowner:id` — the Zitadel **organization**, which is the tenant |
| `paridyum.account_id` | resolved from **Paridyum's own tables**, keyed on the signed `sub` |
| `paridyum.operator_scope` | presence of the operator-scope **role** in the project roles claim |

Never a header, a body field, a path parameter, or a custom claim written by our own code.

**Two of those deserve their reasoning, because both changed on 2026-08-08 when RFC-0012 moved
from instance-per-tenant to organization-per-tenant.**

**Tenant.** An earlier draft derived it from the verified `iss`, which worked while each tenant
had its own Zitadel instance and therefore its own issuer. With one shared instance per region
the issuer no longer discriminates, so the tenant comes from the organization claim. That is
still safe, and the distinction is the load-bearing one: **Zitadel sets this claim from the
user's actual organization membership and signs it.** The rule this ADR enforces is not "never
trust a claim" — it is never trust a claim *we* wrote. A Zitadel-derived claim is exactly as
trustworthy as the signature that carries it.

**Account.** Zitadel has no isolation level below the organization — the vendor's own words:
*"ZITADEL does not enforce a native hierarchy or inheritance for users or organizations."* So an
account is not a Zitadel object at all; it is Paridyum's entity, and `lib-auth-typescript` resolves it from
Paridyum's tables. Storing `account_id` in Zitadel user metadata was considered and rejected
under **ANTI-0001**: it would create a second writer for a fact this organisation owns, and the
drift would land on the inner silo.

**Every user belongs to exactly one account, including a solo user, whose account is a personal
account containing only them.** That is what keeps `account_id` total — and a total invariant is
what lets `paridyum.current_account()` raise unconditionally. A nullable account discriminator
inside an RLS policy is GLO-0005's absence hazard, installed at the one place this ADR exists to
protect.

**5. The repository layer asserts the account discriminator on every row it returns.** Added
2026-08-08 in response to ADR-0045 risk R1.

`lib-auth-typescript` compares each returned row's `account_id` against `paridyum.account_id` and raises on
a mismatch — a defect, never a filtered result. This is **deliberately redundant with RLS**, and
the redundancy is the point: since RFC-0012 moved tenants to Zitadel organizations, the
account↔account boundary is enforced by exactly one control, and GLO-0004 rates crossing it
*"catastrophic for that business."*

The two controls fail independently. RLS fails through a policy that omits `account_id`, a
missing `FORCE`, or a role holding `BYPASSRLS`. The assertion fails only if `lib-auth-typescript` itself is
wrong. Neither catches the other's failure, which is what makes two of them worth the cost.

It is cheap — one comparison per row, on data already in memory — and it must **raise rather than
filter**. Silently dropping foreign rows would convert a catastrophic isolation defect into a
mildly wrong result set, which nobody reports.

**6. The application connects as a role that is `NOBYPASSRLS` and is not the table owner.**
ADR-0024 already requires `FORCE ROW LEVEL SECURITY`; this states the other half. A migration
runs as the owner; a request must not.

**7. Anything that runs outside a request sets the context explicitly or does not run.** A
background worker, a migration backfill and a scheduled job have no token. They either set the
context from a durable record of whose work they are doing, or they operate on a
`platform`-scoped table where no policy applies. There is no ambient default.

## Consequences

**Accepted costs**

1. **Every query path must set three settings, and forgetting is a hard error rather than a
   wrong answer.** This is the intended trade and it will be felt: a missed `SET LOCAL` takes
   the endpoint down instead of quietly returning nothing. Loud is correct here, and it is
   still an outage.

2. **`STABLE` and not `IMMUTABLE`, so the planner re-evaluates per statement.** `current_setting`
   is not immutable and marking it so would let the planner cache a value across a
   transaction — the exact bug this ADR exists to prevent, introduced as an optimisation.
   Accept the planning cost.

3. **Three settings on every transaction is measurable overhead** on short queries. Mitigable by
   batching into one `SET LOCAL` statement; not mitigable away entirely.

4. **The `iss` → `tenant_id` resolution is a lookup, and it is on the hot path.** It must be
   cached, and a cache of a security-relevant mapping is a place where staleness becomes a
   cross-tenant read. The mapping only ever grows — an issuer is never reassigned to a different
   tenant — which is what makes caching safe, and that property has to be enforced by
   `platform-tenancy` rather than assumed.

5. **A background job with no token is a real gap** and decision 6 disposes of it in one
   sentence. The first worker that needs to act across accounts will test whether that sentence
   was enough.

**Gained**

- The largest hard blocker to the first migration is removed, and removed *before* the migration
  rather than by it.
- GLO-0005's hazard is closed at the root: no code path produces a widened scope from a missing
  value, because a missing value produces no result at all.
- `db.no_tenant_id_from_client_input` gets an operational definition it has lacked since it was
  declared.

**Rejected alternatives**

- **Return `NULL` and let policies handle it.** The conventional pattern, and it is the GLO-0005
  hazard verbatim. `tenant_id = NULL` is never true, so reads fail closed by luck — but
  `paridyum.has_operator_scope()` returning `NULL` inside `(has_operator_scope() OR account_id =
  ...)` makes the operator branch unknown rather than false, and any policy an author writes as
  `NOT has_operator_scope()` inverts. Safety by accident of three-valued logic is not safety.

- **`SET` rather than `SET LOCAL`.** One statement per connection instead of per transaction, and
  it leaks across pooled requests. The failure appears only under concurrency, in production.

- **Pass tenant explicitly as a query parameter, no session context.** Honest and auditable, and
  it makes every query author responsible for isolation — so the first forgotten `WHERE`
  crosses a silo. RLS exists to make that structurally impossible.

- **A custom `paridyum:tenant_id` claim written by a Zitadel Action.** One less lookup, and it
  demotes the catastrophic boundary from "guaranteed by signature verification" to "guaranteed
  by our Action being correct." Rejected on consequence, not on cost.

- **Defer until the first service exists.** Which is precisely when it gets decided by accident,
  in a migration whose author is thinking about a table.

## Related

- ADR-0024 — the tenancy model these functions implement; *"none of it is a later refactor"*
- STD-0006 — prints the policy shape that calls them
- GLO-0005 — operator scope is a grant, never an absence. Decision 2 is that sentence in SQL
- GLO-0004 — the two silos the three accessors discriminate
- STD-0009 — the runtime tests; the denial is the assertion
- RFC-0012 — where the claims come from, and why `iss` is the tenant
- C-0005 — `paridyum` schema and function naming
