---
id: ADR-0102
title: docket is instantiated inside a tenant and holds no account silo, until a tenant administers someone else's people
type: adr
status: superseded
created: 2026-09-20
locked_at: null
supersedes: null
superseded_by: ADR-0104
scope: [org, platform, web]
applies_to: ["*"]
product: [tenancy]
tags: [docket, tenancy, account, nested-tenancy, adr-0024, adr-0051, adr-0093, adr-0099, glo-0004, glo-0005, req-0003]
breaking: false
trigger: >
  The first tenant that administers people on behalf of a distinct client organisation — a
  staffing agency, a PEO, or an outsourced-HR provider. At that point docket's person-bearing
  tables owe an account discriminator and this ADR is superseded.
enforced_by: []
---

# ADR-0102 — docket is instantiated inside a tenant and holds no account silo, until a tenant administers someone else's people

## Status

**SUPERSEDED by ADR-0104** on 2026-09-20, hours after being accepted the same day.

**Read this for its facts, not its conclusion.** The three schema findings in §Context are correct
and were the basis for ADR-0104 too: the five tables are empty, no `accounts` table exists, and
`docket_companies` models subsidiaries rather than customers. What was wrong was the trigger —
*"the first tenant that administers people on behalf of a distinct client organisation"* — which
named one instance of a general condition. The general condition is **a tenant that needs two
isolated sets of one product's data**, and the operator supplied it the same day by asking for
multiple instances of docket. ADR-0104 §Decision 3 explains why the narrow wording produced the
wrong conclusion from right facts.

**Original status: ACCEPTED** as of 2026-09-20 by the operator. Answered **T-G.0d**, which requires *"either a migration [that] adds
`account_id` to the person-bearing docket tables, or an ADR [that] records the omission as an
accepted, triggered deferral."* This is the second.

## Context

ADR-0024 makes tenancy **nested**: two silos, `tenant_id` outside and `account_id` inside.
GLO-0004 defines the inner one precisely — an account is **the tenant's customer**.

docket's tables do not have one. `docket_persons`, `docket_identities`, `docket_register`,
`docket_events` and `docket_signatures` are all `-- paridyum:scope tenant`, while `applications`,
`principals` and `profiles` — which predate docket's move into this schema — are `scope account`.
T-G.0d raises the resulting asymmetry and puts a clock on it: **ADR-0024 decision 9 warns that
retrofitting an inner discriminator means rewriting every policy while rows already exist**, and
the window closes when T-G.7's backup gate opens on real HR records.

**Three facts were established against the schema before this was decided**, because the question
had been argued from the tables' shape rather than from what they hold:

1. **All five person-bearing docket tables are empty**, as is every other table in `public` except
   `tenants`, which holds one row. Verified 2026-09-20 against the live database. So the retrofit
   is free today in a way it will not be again.
2. **There is no `accounts` table anywhere in the schema.** `account_id` on the three tables that
   carry it is a bare `uuid` with no foreign key. Whatever an account is, this database does not
   yet say.
3. **`docket_companies` is not a list of accounts.** It comes from `org.yaml` and carries an
   `owned_by` self-FK to model a corporate group — the tenant's *own* legal entities and their
   holding structure. Mapping company onto account would have been the obvious move and would have
   been wrong: a subsidiary is not a customer.

## Decision

**1. docket is instantiated inside a tenant, and its records belong to that tenant.** A tenant
holds docket as a Release (RFC-0049); the people in it are that tenant's own people. There is no
party beneath the tenant with a claim on a docket record, so there is nothing for an inner silo to
separate.

**2. The person-bearing docket tables stay `-- paridyum:scope tenant`, and this ADR ratifies an
existing decision rather than making a new one.**

⚠ **This decision said the opposite in its first draft, and the draft was wrong.** It read: *"It
was previously neither a decision nor an omission — the tables were written tenant-scoped because
docket was single-tenant when they were written, and nothing had examined whether that survived
ADR-0099."* Migration `0006`'s own header, written 2026-09-14, says otherwise in a titled
paragraph:

> **SCOPE IS TENANT, NOT ACCOUNT.** These are the tenant's HR records, and an HR operator manages
> every person's, not only their own — so access is `current_tenant()` AND `has_operator_scope()`,
> unlike profiles (0003), which an account may also read for itself.

So it was reasoned, contemporaneously, with a justification this ADR agrees with and did not
supply. **The correction strengthens the decision rather than weakening it:** a ratification of a
reasoned choice is a firmer thing than a first examination of an accident. The error was found by
reading the migration rather than the task that described it — which is the same lesson
`PLAN-THREE-SURFACES.md` F0b records one level up.

**3. This is consistent with ADR-0051 rather than merely compatible with it.** That ADR makes a
person identifier opaque, permanent and allocated once. ADR-0099's design states the multi-tenant
reading: *"one person, one identifier, across companies — person id stays group-wide within a
tenant; company code prefixes only engagement/serials."* An account discriminator on
`docket_persons` would contradict that directly, by making a person a resident of one account
within a tenant that has several.

**4. The tenant boundary is unchanged and remains the whole isolation story for docket.** Every one
of these tables is `FORCE ROW LEVEL SECURITY` with a policy of `tenant_id = paridyum.current_tenant()
AND paridyum.has_operator_scope()`. Nothing in this decision weakens that; it declines to add a
second boundary, it does not soften the first.

**5. The trigger is the first tenant that administers people on behalf of a distinct client
organisation** — a staffing agency, a PEO, an outsourced-HR provider. Such a tenant genuinely has
accounts in GLO-0004's sense, and its client companies' people must not be visible to each other.
At that point the five tables owe `account_id`, their policies owe an inner predicate, and this ADR
is superseded.

**6. The trigger is evaluated at docket onboarding, by a human, and the question is asked
explicitly.** ADR-0018's failure is a trigger nobody can evaluate, and a trigger phrased as "when
we notice" is that failure. Concretely: the onboarding step that seeds a tenant's docket config
(`seed-config.py`, ADR-0099 stage 4) is where someone must answer *"does this tenant administer
people who work for someone else?"* — before its first person record, because after it the answer
costs a migration against live HR data.

**7. The window this deferral depends on is now held open by a mechanism rather than by
attention.** The whole deferral is only safe while no real person record exists, and that
condition was already declared in four places — ADR-0064 D3, ADR-0068 §5, ADR-0093 §5, and
migration `0006`'s own header — and **enforced in none of them inside docket.** `web-hiring` had
implemented the identical gate on its apply route since it shipped; `tool-docket`, which is what
actually writes personnel records, checked nothing.

That is fixed alongside this ADR (`tool-docket` PR #16): the four person-writing routes answer
`503` until `PARIDYUM_BACKUPS_CONFIRMED=1`, which is T-G.7. **So the retrofit stays five
`ALTER TABLE`s against empty tables until a deliberate act opens the gate**, rather than until
somebody remembers. Without that, this ADR would have been a deferral resting on a rule nothing
applied.

## Consequences

**Accepted costs**

1. **The retrofit, if the trigger fires, happens against live rows.** This is ADR-0024 D9's
   expensive path, chosen knowingly. Today the change is five `ALTER TABLE`s against empty tables;
   after the first tenant's HR records exist it is a backfill with no determinate value — nobody
   can say retroactively which account a person belonged to — plus a policy rewrite on five tables,
   three of which are append-only sealed chains whose rows cannot be modified at all.

   **That last clause is the real cost and it is worse than the general warning.** `docket_register`,
   `docket_events` and `docket_signatures` refuse UPDATE by trigger (ADR-0093). A retrofit cannot
   assign an account to rows that already exist in them; the honest outcome would be a nullable
   column with a documented "pre-retrofit" cohort, permanently.

2. **The asymmetry with `applications`, `principals` and `profiles` persists**, and a reader will
   keep asking why three tables in this schema have an inner silo and five do not. The answer is
   that those three serve hiring, where a tenant's *applicants* are plausibly the tenant's
   customers' concern, and these five serve the tenant's own staff. That reasoning now has a home.

3. **What keeps this honest is the backup gate, not a schema check**, and none is declared. `enforced_by` is
   empty deliberately rather than aspirationally: the rule "these five tables stay tenant-scoped"
   is not a rule worth a check, because adding `account_id` to one of them is exactly what
   superseding this ADR *should* do. What is worth a check is that the onboarding question in
   decision 6 is actually asked, and that belongs to whatever implements onboarding rather than to
   the schema gate.

**Gained**

- **docket ships with the silo it can defend.** An `account_id` with no accounts table, no defined
  meaning and one value per tenant is a column that looks like isolation and is not — and it would
  have satisfied `db.account_scoped_tables_carry_tenant_id` while enforcing nothing, which is the
  defect class ADR-0014 exists to name.
- **The question is now on the record with a date**, which is the difference between a deferral and
  a thing nobody looked at.

**Rejected alternatives**

- **Add `account_id` now with one implicit default account per tenant.** Free today, and it makes
  the eventual agency case a data change rather than a migration. Rejected because it contradicts
  ADR-0051's group-wide person identifier (decision 3), and because every query and policy would
  carry an inner predicate for the whole life of the product to serve tenants that may never exist.
- **Map `account` onto `docket_companies`.** The obvious reading, and the one this decision was
  closest to taking. Rejected on GLO-0004: a company in `org.yaml` is a subsidiary of the tenant,
  and an account is a customer of it. Encoding a group's internal structure as the platform's
  isolation boundary would make a tenant's own subsidiaries invisible to each other by default,
  which is the opposite of what a corporate HR group needs.
- **Leave it open and revisit.** T-G.0d's own framing argues against it: the information that would
  settle it arrives exactly when the migration becomes expensive.

## Related

- **T-G.0d** — the Genesis task this answers, and its two permitted outcomes
- **ADR-0024** — nested tenancy, and decision 9's warning about retrofitting the inner discriminator
- **GLO-0004** — an account is the tenant's customer, which is the definition that decided this
- **GLO-0005** — operator scope is a grant, never an absence; unchanged by this
- **ADR-0051** — the opaque, permanent person identifier that decision 3 turns on
- **ADR-0093** — the append-only sealed chains, which is why cost 1 is worse than the general case
- **ADR-0099** — docket as a multi-tenant platform; the move that made this question live
- **RFC-0049** — a tenant is a set of Releases, which is what "instantiated inside a tenant" means
- **PLAN-TENANT-FLOW.md** §7 — the three schema facts established before this was decided
