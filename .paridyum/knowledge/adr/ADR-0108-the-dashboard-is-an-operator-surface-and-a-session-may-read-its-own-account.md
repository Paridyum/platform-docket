---
id: ADR-0108
title: The dashboard is an operator surface, and a session may read its own account and no other
type: adr
status: accepted
created: 2026-09-25
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, account, rls, dashboard, operator-scope, adr-0024, adr-0038, adr-0100, adr-0104, glo-0004, glo-0005, rfc-0015, rfc-0049]
breaking: false
enforced_by:
  - check: db.operator_scope_is_an_explicit_grant
    severity: error
---

# ADR-0108 — The dashboard is an operator surface, and a session may read its own account and no other

## Status

**ACCEPTED** as of 2026-09-26 by the operator. Proposed 2026-09-25. Answers T-G.20, which migration `0014` deliberately left open rather
than settling in SQL: this is a decision about who may read what, and the estate's rule is that the
artifact precedes the code.

**What acceptance releases:** the `accounts_read` policy, which decision 2 specifies and which
migration `0014` deliberately did not include — the ADR was `proposed`, and writing the policy then
would have decided it in SQL.

## Context

**A signed-in tenant can read its Release and cannot name the product it is an instance of.**

Migration `0013`/`0014` gives `releases` a conditional inner predicate:

```sql
USING (tenant_id = paridyum.current_tenant()
       AND (account_id = paridyum.current_account() OR paridyum.has_operator_scope()))
```

so a non-operator session reads its own Release row. But `releases` deliberately does not carry
`product` — `accounts.product` holds it, and RFC-0015 forbids the second home. And `accounts`
carries exactly one policy, `accounts_operator`, which is `FOR ALL` and requires
`has_operator_scope()`.

**So the non-operator branch of `releases_read` resolves to a row the session cannot interpret.** It
can see that it runs *something*, at a version, in a placement — and not what.

**The second problem is larger and was hiding behind the first.** ADR-0100 decision 3 makes
`/dashboard` the signed-in surface of whichever tenant owns the host, showing *"that tenant's
dashboard — its own products"*. Plural. A tenant running landing, careers, docket and admin has
**four accounts**, and `paridyum.account_id` is one value per session (ADR-0038).

So listing a tenant's products is not a read of one account. **It is a read across every account of
the tenant** — and that is the exact operation GLO-0004 point 1 names:

> *"The inner one must be conditional, because an operator's own console legitimately reads every
> account of its tenant… that default must be an explicit grant instead."*

Widening `accounts` so that any session of a tenant reads all of them would be that default. GLO-0005
exists to forbid it.

## Decision

**1. Listing a tenant's products is a cross-account read, and `/dashboard` therefore requires
operator scope.**

This is not a new grant and not a widening. It is what operator scope already *means*: GLO-0004
point 1 describes this console by name, and `paridyum.operator_scope` is the explicit positive claim
GLO-0005 requires, sourced from a role rather than from the absence of a value.

**`/dashboard` is an operator surface.** A signed-in tenant user without the operator role does not
get a narrowed dashboard — they get their product's own surface. A dashboard that silently lists one
product instead of four is worse than one that refuses: it looks like the tenant runs one thing.

ADR-0100 D3's *"its own products"* is hereby read as **an operator's view of its tenant**, which is
consistent with ADR-0099/ADR-0082 already deriving docket's tenant from an operator's token.

**2. A session may read its own account row, and only its own.**

```sql
CREATE POLICY accounts_read ON accounts
  FOR SELECT
  USING (tenant_id = paridyum.current_tenant()
         AND (id = paridyum.current_account() OR paridyum.has_operator_scope()));
```

Same shape as `releases_read`, for the same reason, and it grants exactly the row the session already
identifies as itself — `current_account()` is an accessor that RAISES when unset (ADR-0038), never a
NULL and never a coalesce.

**This is what makes the non-operator branch of `releases_read` usable at all.** Without it that
branch returns a row nothing can interpret, which is not a boundary doing its job — it is a boundary
that made a feature impossible and nobody noticed for as long as nothing read it.

**3. Writes on `accounts` stay operator-only.** `accounts_operator` is unchanged. An account
creating or renaming itself is provisioning itself, which is ADR-0039's saga and not a row insert —
the same argument `releases_operator_write` already makes.

**4. `product` is not copied onto `releases`.** RFC-0015 stands. The join is the price of one fact
having one home, and decision 2 is what makes the join affordable rather than the copy necessary.

**5. This does not decide per-instance operator grants.** ADR-0104 accepted cost 4 leaves open which
operator may reach which instance; `operator_grants` (migration `0010`) is the table that would
carry it. An operator with the tenant's operator role still reaches every instance, on purpose and
for now. **That question gets harder the moment a tenant runs two dockets with different staff**, and
this ADR deliberately does not pre-empt it.

## Consequences

**Accepted costs**

1. **Every tenant administrator needs the operator role in its Zitadel organization to see its own
   dashboard.** That is a provisioning obligation ADR-0039's saga must satisfy, not something a
   tenant can grant itself.
2. **The operator population is still tenant-wide** (ADR-0104 cost 4, restated). Decision 5 names
   this rather than fixing it.
3. **A non-operator session sees exactly one product.** Correct, and it will read as a bug to
   whoever meets it first — which is why decision 1 refuses rather than narrows.

**Gained**

- The non-operator branch of `releases_read` becomes usable, so T-G.9 can render a product name.
- The account boundary is held identically on `accounts` and `releases` — one shape, learned once.
- `/dashboard`'s authorisation requirement is stated before it is built, rather than discovered by
  whoever finds the query returning nothing.

## Alternatives considered

- **Widen `accounts` to any session of the tenant.** Simplest, and it is precisely the
  widening-by-absence GLO-0005 forbids: every account of a business would read every sibling's name,
  slug and product. Rejected.
- **Copy `product` onto `releases`.** Removes the join and the problem. Rejected by RFC-0015 — a
  second home drifts, and "what product is this" would become a question with two answers, which is
  the thing `releases` exists to end.
- **A `SECURITY DEFINER` function returning the tenant's products.** Works, and punches a hole around
  RLS that no policy describes. The estate has exactly one such function (`paridyum.resolve_account`)
  and the care its body documents — saving and restoring session values, elevating for one statement
  — is the argument against casually adding a second.
- **Give `/dashboard` a narrowed non-operator view.** Rejected by decision 1: a dashboard listing one
  of four products is a wrong answer presented as a complete one.

## Related

- **ADR-0100** D3 — the URL model this reads as an operator view
- **ADR-0104** — a product instance is an account; cost 4 is decision 5's open question
- **ADR-0024** — `locked`; the two boundaries
- **ADR-0038** — the accessors raise rather than defaulting, which is why decision 2 is safe
- **GLO-0004** point 1 — names this console explicitly; **GLO-0005** — the grant is never an absence
- **RFC-0015** — a fact has one home, which decision 4 upholds
- **RFC-0049** — `Release`, whose read model decision 2 completes
- **T-G.20**, **T-G.9** — the task this answers, and the one it unblocks
