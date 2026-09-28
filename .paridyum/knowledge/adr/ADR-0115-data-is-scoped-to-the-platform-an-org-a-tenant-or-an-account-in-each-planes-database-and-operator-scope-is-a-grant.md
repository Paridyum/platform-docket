---
id: ADR-0115
title: Data is scoped to the platform, an org, a tenant or an account, in each plane's database, and operator scope is a grant
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: [ADR-0024]
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, nested-tenancy, org, isolation, postgres, rls, authorisation, density, planes, region, adr-0024, adr-0038, adr-0114, rfc-0007, rfc-0014, std-0023, glo-0004, glo-0005, glo-0012]
breaking: true
enforced_by:
  - check: db.operator_scope_is_an_explicit_grant
    severity: error
    rationale: >
      Breadth of access expressed as the absence of a filter is escalated by every bug that loses a
      value, silently, with the query still returning rows. Carried from ADR-0024 unchanged.
  - check: db.account_scoped_tables_carry_tenant_id
    severity: error
    rationale: >
      Account surrogate values collide across tenants, so a predicate on account_id alone crosses
      the outer silo while looking correct. Carried from ADR-0024 unchanged.
  - check: db.tenant_scoped_tables_declare_tenancy
    severity: error
    rationale: >
      Four categories now, and none can be inferred from a schema. An undeclared table defaults to
      the wrong one silently. Widened from ADR-0024's three categories to four.
  - check: db.row_level_security_enabled_on_tenant_tables
    severity: error
    rationale: >
      ENABLE without FORCE exempts the table owner, and every predicate of the category must exist —
      an outer-only policy isolates businesses from each other and nobody's customers, and an org
      table with no org predicate isolates nobody. Widened to org-scoped tables.
  - check: db.no_tenant_id_from_client_input
    severity: error
    rationale: >
      Every discriminator — org, tenant, account — is enforced against session values. A client that
      can set one gets a correctly-applied policy on the wrong scope. Widened to org_id.
  - check: db.tables_declare_their_plane
    severity: error
    rationale: >
      RFC-0014 D3, resolved here. A table's plane is not inferable from its schema, and a pii column
      landing in the control plane instead of a region is a cross-border transfer every other check
      reports as green.
  - check: db.tenant_scoped_tables_are_regional
    severity: error
    rationale: >
      RFC-0014 D3, resolved here. A tenant's rows live in that tenant's region and nowhere else, so a
      tenant-scoped table declared control-plane is one of two declarations being wrong.
  - check: db.org_scoped_tables_are_regional
    severity: error
    rationale: >
      Decision 10. An org's own rows are its people — members, invitations, names — and RFC-0017
      keeps a customer's people in the customer's region. An org-scoped table declared control-plane
      moves them into the control region.
  - check: db.no_org_discriminator_on_business_data
    severity: error
    rationale: >
      Decisions 2 and 9. Business data carries two discriminators. A column naming an org on a
      tenant- or account-scoped table is the third silo decision 9 defers, arriving by accident.
verified_by:
  - TST-0016
  - TST-0017
  - TST-0018
---

# ADR-0115 — Data is scoped to the platform, an org, a tenant or an account, in each plane's database

## Status

**ACCEPTED** 2026-09-28 by the operator. **Supersedes ADR-0024** (`locked`), which is restated here
in full, as REQ-0002 requires of a supersession: a reader never needs ADR-0024 to know what holds.
It is locked in a later change, once the checks it adds have been watched failing.

**Two things changed, and everything else is ADR-0024's, unchanged:**

1. **A fourth category, `org`** (ADR-0114). The operator chose a first-class org on 2026-09-28, which
   ADR-0024's three categories could not express without calling an org a tenant or an account.
2. **"In one database" is scoped to a plane** — RFC-0014 D3 and D4, resolved here. Tenants share a
   region's database; regions do not share one. The rest of RFC-0014 is unchanged and stays
   `proposed`.

**The operator signed decision 9's reading on 2026-09-28**: the org is not the third level ADR-0024
D9 deferred.

## Context

Paridyum is a multi-tenant platform whose tenants are **themselves multi-tenant applications**
(GLO-0004). A tenant built under BET-0001 is a software product with customers of its own, and those
customers need isolation from each other. ADR-0024 therefore made two nested silos — tenant and
account — two columns and two row-level-security predicates, and RFC-0007 compared the five
mechanisms that led there. That analysis is reused unchanged.

**Two pressures arrived after it was locked.**

- **Residency.** RFC-0014 found ADR-0024's title — *"in one database"* — wrong once a tenant's data
  may not leave its region: the real topology is one control-plane database plus one database per
  region. It named this supersession as owed.
- **The org.** paridyum.in's customer is a person or a team that owns several tenants (ADR-0114). Its
  own rows — members, invitations, names — belong to it and to no tenant. ADR-0024 D1 allowed three
  answers for such a row: platform-owned (then nothing isolates one org's people from another's),
  tenant-scoped (then the org is a tenant) or account-scoped (then the org is an account of some
  tenant). The operator rejected all three and asked that the words never mix.

## Decision

**One schema in each plane's database. Two nested silos for business data, as two columns and two
row-level-security predicates. An org's own data as a separate category with its own column and
predicate. Operator scope as an explicit positive grant. No per-org, per-tenant or per-account
infrastructure.**

**1. Four table categories, each declared, never inferred** (syntax: STD-0023).

| Category | Columns | Holds | Example |
| --- | --- | --- | --- |
| **`platform`** | none of the discriminators | Paridyum's own data, and the registries that *define* orgs and tenants | `regions`, `orgs`, `tenants` |
| **`org`** | `org_id` | one org's own data | `org_members`, `org_invitations` |
| **`tenant`** | `tenant_id` | one business's data (GLO-0002) | the `plans` a product offers |
| **`account`** | `tenant_id` **and** `account_id` | one silo inside one business (GLO-0003) | `invoices` |

No naming rule separates them — `invoices` is account-scoped, `plans` tenant-scoped, `regions`
platform-owned, `org_members` org-scoped — so defaulting to one was rejected: the common case would
need an opt-out, and an opt-out used constantly stops being read.

**2. The discriminators never mix.**

- **`account_id` never appears without `tenant_id`.** Account surrogates collide across tenants, so a
  predicate on `account_id` alone crosses the outer silo while looking correct.
- **`org_id` never appears on a tenant- or account-scoped table.** Business data does not know its
  owner (ADR-0114 D4).
- **An org-scoped table carries neither `tenant_id` nor `account_id`.** A fact that relates an org to a
  tenant is platform-owned registry data — `tenants.owner_org_id`.
- **A platform table carries none of the three discriminator columns.** It may reference an org or a
  tenant by a named key (`owner_org_id`), which a reader cannot mistake for a scope.

**3. Row-level security `ENABLE`d and `FORCE`d on every org-, tenant- and account-scoped table, with
every predicate of its category, verified against the live catalogue.**

- org: `org_id = paridyum.current_org()`;
- tenant: `tenant_id = paridyum.current_tenant()`;
- account: both of the tenant predicates, as ADR-0024 fixed them.

`ENABLE` alone exempts the table owner, and migrations run as the owner. An outer-only policy on an
account-scoped table isolates businesses from each other and nobody's customers — which every
dashboard will report as healthy.

**4. Operator scope is an explicit positive claim, verified per request (GLO-0005).** Never a
missing, null, empty or wildcard `account_id`. Breadth encoded as the absence of a restriction is
granted by every bug that loses a value — a dropped session field, an uninitialised variable, a lookup
returning null, a middleware ordering mistake. Encoded as a claim, the same bugs fail closed and loud.
**The same holds one level up:** no org role, and no ownership, is ever expressed as the absence of a
tenant filter (ADR-0114 D6).

**5. Every identifier comes from the verified session only** — `org_id`, `tenant_id` and
`account_id` alike. Never a request body, query string, path segment or client-supplied header. The
session context type is the only thing able to produce one; the accessors raise when their setting is
unset and never return NULL (ADR-0038).

**6. The runtime role cannot bypass row-level security.** Not the table owner; migrations and runtime
use different roles. Operator scope is expressed *in* the policy, never by escaping it.

**7. No per-org, per-tenant or per-account infrastructure.** A custom domain is a routing entry and a
certificate. Orgs and accounts are free to create.

**8. Escalation is deferred behind a funded trigger, not published as a tier.** Dedicated database
and virtual cluster remain the path (RFC-0006 options C and D); neither is built or offered until a
signed contract pays for it. C-0006 anti-goal 5 is the commercial form.

**9. A third silo is deferred behind a named trigger.** RFC-0007 option B's hierarchical path
generalises to any depth and indexes worse on the hot path. Two explicit columns can be migrated to a
path; a path cannot easily be migrated back. **Trigger: a real third silo with a paying customer
behind it** — a reseller or white-label partner whose customers' tenants nest inside it, or
sub-accounts within an account. Not "we can imagine one."

**The reading, signed by the operator on 2026-09-28: the org is not that third silo.**

- ADR-0024 D9 was written on 2026-08-07, when GLO-0001 defined the operator as *"the party that holds
  a Paridyum subscription and runs one or more tenants"* — the party ADR-0114 names the org. D9's
  *"a reseller or white-label partner above the operator"* is therefore a party **above the org**.
- **The org adds no discriminator to business data** (decision 2). Its own rows are their own category,
  and its ownership of a tenant is one key on the tenant registry. Business rows carry exactly
  `tenant_id` and `account_id`, as D9 left them.
- **A party above orgs** — an org selling tenants to other orgs, or renting the platform under its own
  brand — **stays deferred**, and is RFC-0047's gated stamp.

**10. One schema in each plane's database** (RFC-0014 D2 to D4).

- **The control plane is one database, in the control region.** It holds platform-scoped control
  tables: the org and tenant registries, the region registry, domain bindings. No person (ADR-0117
  D3).
- **Each region's data plane is one database.** It holds every org-, tenant- and account-scoped table
  of the orgs and tenants in that region, and the platform-scoped tables declared regional.
- **Org- and tenant-scoped tables are regional**, always (`db.org_scoped_tables_are_regional`,
  `db.tenant_scoped_tables_are_regional`).
- **Today one region is live and both planes share one database** (RFC-0015 D2). The plane directive
  on every table says where it goes when they split; nothing about the split is built before a second
  region's trigger fires.

## Costs accepted

1. **The dangerous default is the main risk**, as ADR-0024 said: a *permissive default* rather than a
   missing column, and absence is what every kind of bug produces. Decision 4 is a rule about how a
   grant is represented, and it can be satisfied on paper by code that still leaks.
2. **Two failure boundaries for business data, and the inner one is not our reputation to spend.** A
   leak between accounts of one tenant damages that business with its customers, who never heard of
   us.
3. **Noisy neighbours at two levels, still unmitigated.** Quota and rate limiting are platform
   correctness features, not billing features (BET-0002 cost 4).
4. **Restore and export fork three ways now** — per tenant, per account, and per org — from a
   cluster-wide backup, by row-level extraction. This is the weakest operational point, and it argues
   for building that machinery earlier than a compliance-only reading suggests.
5. **Every business query carries two predicates**, on the hot path; every org query carries one.
6. **A fourth category every migration author must choose correctly**, and a plane declaration on top
   (STD-0023). Both are cheap to write and decidable to check; both are one more place to be wrong.
7. **Today's cross-plane foreign keys stop being possible when the planes split.** Regional tables
   reference `tenants (id)` while both planes share one database. A second region needs a regional
   anchor for each tenant, designed with that region — recorded here so that it is a known cost
   rather than a discovery.
8. **The registries are isolated by code, not by row-level security.** `orgs` and `tenants` are
   platform-scoped, so which rows a portal session may read is the service's filter on the verified
   session (ADR-0114 cost 2).

## Alternatives rejected

- **The org as an account of a tenant that runs the portal.** It reused every mechanism and needed no
  supersession, and it made an org an account and the portal a tenant — the mixing ADR-0113 ends.
- **The org as a third column on business data.** The third silo decision 9 defers, taken without its
  trigger.
- **The org as platform-owned data, people included.** Nothing would isolate one org's people from
  another's except code, and RFC-0017 keeps those people out of the control plane.
- **Hierarchical path column** — hot-path index performance, deferred to decision 9's trigger.
- **Schema per tenant** — REQ-0008's no-downtime migration requirement.
- **The business implements the inner silo** — it gives away the largest item on C-0006's
  not-commodity list and breaks BET-0003.
- **Self-referential tenants table** — one word for two isolation guarantees is a cross-tenant leak
  waiting for somebody to assume wrong.
- **One global database with region as a column** (RFC-0014) — not a residency architecture at all.

## Consequences

- **`check-migrations.sh` learns the `org` category**, its predicate, and the two new checks, each
  shown failing against a planted defect before it is trusted (STD-0023).
- **`paridyum.current_org()` joins ADR-0038's accessors**, with the same contract: raise on unset,
  never NULL.
- **ADR-0024's `enforced_by` stays on ADR-0024**, because a superseded artifact keeps it; the
  inherited checks' `blocks` lists accumulate this ADR, and `knowledge.orphan_check_declarations`
  keeps that visible.
- **GLO-0004's tree gains the org** above the silos, as their owner and not as a level.
- **BET-0003's runtime contract must express this as an inherited primitive** — still the clearest
  single requirement on a contract that does not yet exist.

## Related

- **ADR-0024** — superseded; restated here · **RFC-0007** — the comparison ADR-0024 resolved
- **ADR-0114** — the org · **ADR-0113** — the words · **STD-0023** — the directive syntax
- **RFC-0014** — D3 and D4 resolved here · **RFC-0015** D2 — one region live · **RFC-0017** — people
  stay regional
- **ADR-0038** — the accessors · **GLO-0002** to **GLO-0005**, **GLO-0012** — the vocabulary
- **RFC-0047** — the level above orgs stays a gated stamp
- **BET-0001**, **BET-0002**, **BET-0003** · **REQ-0008**, **REQ-0013**, **ANTI-0007** · **C-0006**
