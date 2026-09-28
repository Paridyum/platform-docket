---
id: ADR-0024
title: Nested tenancy is two columns in one database, and operator scope is a grant
type: adr
status: superseded
created: 2026-08-07
locked_at: 2026-08-07
supersedes: [ADR-0023]
superseded_by: ADR-0115
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [tenancy, nested-tenancy, isolation, postgres, rls, authorisation, density]
breaking: true
enforced_by:
  - check: db.operator_scope_is_an_explicit_grant
    severity: error
    rationale: >
      Breadth of access expressed as the absence of a filter is escalated by every bug that
      loses a value, silently, with the query still returning rows.
  - check: db.account_scoped_tables_carry_tenant_id
    severity: error
    rationale: >
      Account surrogate values collide across tenants, so a predicate on account_id alone
      crosses the outer silo while looking correct.
  - check: db.tenant_scoped_tables_declare_tenancy
    severity: error
    rationale: >
      Three categories now, and the third cannot be inferred from a schema. An undeclared
      table defaults to the wrong one silently.
  - check: db.row_level_security_enabled_on_tenant_tables
    severity: error
    rationale: >
      ENABLE without FORCE exempts the table owner, and both predicates must exist — an
      outer-only policy isolates businesses from each other and nobody's customers.
  - check: db.no_tenant_id_from_client_input
    severity: error
    rationale: >
      Both identifiers are enforced against session values. A client that can set either one
      gets a correctly-applied policy on the wrong scope.
---

# ADR-0024 — Nested tenancy is two columns in one database

## Status

**LOCKED** as of 2026-08-07. Supersedes ADR-0023, which was locked the same day.

That is not a typo and it should not be smoothed over. ADR-0023 recorded, as its own cost 6,
that *"this ADR is locked before a single migration exists, so its first real contact with code
may reveal that a rule is impractical. The remedy is supersession, which is expensive by
design."* The rule was not impractical; it was **incomplete**, and the gap was found by
describing the product out loud rather than by writing code. Supersession on the same day is
the mechanism working, at the cheapest moment it could possibly have fired — before any
migration, any row, any customer.

The alternative was to edit ADR-0023 in place, since it had not yet been committed. Rejected:
"it was not committed yet" is exactly the kind of reasoning-around CLAUDE.md forbids, and
ADR-0023's analysis — RFC-0006's five options, the arithmetic that eliminated
database-per-tenant and virtual clusters — remains valid and is reused here unchanged. The
record of why one level was chosen is the context for why two were needed.

## Context

Paridyum is a multi-tenant platform whose tenants are **themselves multi-tenant applications**
(GLO-0004). A tenant built under BET-0001 is a software product with customers of its own, and
those customers need isolation from each other.

ADR-0023 modelled one level. Everything it said remains true; what it also did was imply that
`tenant_id` is sufficient, and the first migration written from it alone would have carried a
single discriminator with no way to separate a tenant's own customers. An artifact that implies
sufficiency where there is none is more dangerous than an absent one, which is why this
supersedes rather than extends.

RFC-0007 compared five mechanisms and, before that, answered whether the inner silo is ours at
all. It is: declining it would mean declining the largest item on C-0006's not-commodity list,
and under BET-0003 it would require the generator to emit correct multi-tenancy per application
with no primitive to emit against.

## Decision

**One Postgres database, one schema. Two nested silos, expressed as two columns and two
row-level-security predicates. Operator scope is an explicit positive grant. No per-tenant or
per-account infrastructure.**

1. **Three table categories, each declared, never inferred.**
   - *platform-owned* — no tenancy column; on the migration's explicit allowlist.
   - *tenant-scoped* — `tenant_id`.
   - *account-scoped* — `tenant_id` **and** `account_id`.

   The third cannot be derived from a schema: `invoices` is account-scoped, the `plans` a
   product offers is tenant-scoped, `regions` is platform-owned, and no naming rule separates
   them. Defaulting to the most restrictive was rejected — it would make the common case
   require an opt-out, and an opt-out used constantly stops being read.

2. **`account_id` never appears without `tenant_id`.** Account surrogate values collide across
   tenants, so a predicate on `account_id` alone crosses the outer silo while looking correct.

3. **RLS `ENABLE`d and `FORCE`d, with both predicates, verified against the live catalogue.**
   `ENABLE` alone exempts the table owner and migrations run as the owner. An outer-only policy
   on an account-scoped table isolates businesses from each other and nobody's customers —
   which every dashboard will report as healthy.

4. **Operator scope is an explicit positive claim, verified per request (GLO-0005).** Never a
   missing, null, empty or wildcard `account_id`. This is the load-bearing rule of this ADR and
   the one genuinely new hazard over ADR-0023. Breadth encoded as the absence of a restriction
   is granted by every bug that loses a value — a dropped session field, an uninitialised
   variable, a lookup returning null, a middleware ordering mistake. The query runs, rows
   return, the policy is applied exactly as written to the wrong scope, and nothing errors.
   Encoded as a claim, the same bugs fail closed and loud.

5. **Both identifiers come from the verified session only.** Never a request body, query string,
   path segment or client-supplied header. The session context type should be the only thing
   able to produce either, so the type system rejects the alternative and the check defends
   against the type being bypassed.

6. **The runtime role cannot bypass RLS.** Not the table owner. Migrations and runtime use
   different roles. Operator scope is expressed *in* the policy, never by escaping it.

7. **No per-tenant or per-account infrastructure.** A custom domain is a routing entry and a
   certificate. Accounts are free to create.

8. **Escalation is deferred behind a funded trigger, not published as a tier.** Dedicated
   database and virtual cluster remain the path (RFC-0006 options C and D); neither is built or
   offered until a signed contract pays for it. C-0006 anti-goal 5 is the commercial form.

9. **A third level is deferred behind a named trigger.** RFC-0007 option B's hierarchical path
   generalises to any depth and indexes worse on the hot path. Two explicit columns can be
   migrated to a path; a path cannot easily be migrated back. **Trigger: a real third level with
   a paying customer behind it** — a reseller or white-label partner above the operator, or
   sub-accounts within an account. Not "we can imagine one."

## Costs accepted

1. **The dangerous default is now the main risk, and it is not the one ADR-0023 had.**
   ADR-0023's hazard was a *missing column* — an omission, which a check can see directly. The
   new hazard is a *permissive default*, and absence is what every kind of bug produces. The
   two additions here are not symmetrical with the original decision: decision 4 is a rule about
   how a grant is represented, and it can be satisfied on paper by code that still leaks.

2. **Two failure boundaries, and the inner one is not our reputation to spend.** A leak between
   accounts of one tenant damages that operator's business with their customers. They will have
   promised isolation to people who never heard of us, and we will have broken it on their
   behalf. That is a harder conversation than a platform outage and there is no version of it
   where we are the injured party.

3. **Noisy neighbours at two levels, still unmitigated.** One heavy account degrades its
   tenant's other accounts and every other tenant. Quota and rate limiting are platform
   correctness features, not billing features — the same conclusion BET-0002 cost 4 reaches from
   pricing.

4. **Restore and export both fork.** Per-tenant *and* per-account point-in-time restore, from a
   cluster-wide backup, by row-level extraction. REQ-0013's export now describes two products
   with different completeness rules. This is the weakest operational point of the decision and
   it argues for building that machinery earlier than a compliance-only reading suggests.

5. **Every query in the system carries two predicates.** A real cost on the hot path, paid on
   every read, and the reason RFC-0007 option B was rejected despite being more general.

6. **Locked again, before any migration exists.** The same bet ADR-0023 made and lost within a
   day. Accepted for the same reason and with less confidence: the alternative is the first
   service deciding tenancy by accident, and that is strictly worse. If this needs superseding
   too, that is the mechanism working rather than failing.

## Alternatives rejected

See RFC-0007. Briefly: **hierarchical path column** on hot-path index performance, deferred to
decision 9's trigger; **schema per tenant** on REQ-0008's no-downtime migration requirement;
**operator implements the inner silo** because it gives away the largest item on C-0006's
not-commodity list and breaks BET-0003; **self-referential tenants table** because one word for
two different isolation guarantees is a cross-tenant leak waiting for somebody to assume wrong.

## Consequences

- The first migration must establish three categories, the two-column convention, the
  two-predicate policy, the two-role split, and operator scope as a claim. None of it is a
  later refactor.
- Two new checks — `db.operator_scope_is_an_explicit_grant`,
  `db.account_scoped_tables_carry_tenant_id` — and three inherited from ADR-0023 with their
  specs widened to two levels. All five await `tool-forge`, and per README.md none should be
  believed until it has been watched failing against a planted defect.
- The three inherited checks now name both ADR-0023 and ADR-0024 in `blocks`, because a
  superseded artifact keeps its `enforced_by` — only `status`, `superseded_by` and `locked_at`
  may move in a supersession. `blocks` lists therefore accumulate across supersessions, and
  `knowledge.orphan_check_declarations` is what makes that visible rather than silent.
- GLO-0001 to GLO-0005 exist because this decision could not be written without them, and
  `glossary` became an artifact type to hold them (ADR-0022's second use).
- REQ-0013 and ANTI-0007 both acquire a second reading. Neither has been rewritten; both are
  annotated.
- BET-0003's contract must express nested tenancy as an inherited primitive. That is now the
  clearest single requirement on the runtime contract that does not yet exist.

## Related

- RFC-0007 — the comparison this resolves
- ADR-0023 — superseded; its arithmetic and option analysis are reused unchanged
- RFC-0006 — the single-level comparison, still the basis for rejecting schema-per-tenant
- GLO-0001 to GLO-0005 — the vocabulary
- BET-0001, BET-0002 — the density and price constraints
- BET-0003 — the contract that must express this
- REQ-0008, REQ-0013, ANTI-0007 — the promises this doubles
- C-0006 — anti-goal 5, and the not-commodity list this defends
