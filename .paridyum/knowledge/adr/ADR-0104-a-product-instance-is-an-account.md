---
id: ADR-0104
title: A product instance is an account, and instances are addressed by path
type: adr
status: accepted
created: 2026-09-20
locked_at: null
supersedes: [ADR-0102]
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, account, instance, multi-instance, docket, routing, url, adr-0024, adr-0100, adr-0101, adr-0102, glo-0003, glo-0004, rfc-0016, rfc-0048, rfc-0049]
breaking: true
enforced_by: []
---

# ADR-0104 — A product instance is an account, and instances are addressed by path

## Status

**ACCEPTED** as of 2026-09-20 by the operator. **Supersedes ADR-0102**, which was accepted earlier
the same day.

**Superseding an artifact hours old is worth a sentence rather than a shrug.** ADR-0102 declined
docket's account silo on a stated and checked premise: docket's model has no party beneath the
tenant, so there was nothing for an inner silo to separate. That premise was true of the product as
it then was. The operator then asked for something ADR-0102 had not been asked about — *a tenant
may run more than one docket* — and that requirement supplies exactly the party ADR-0102 looked for
and did not find. The ADR was not wrong; it was answering a narrower question, and it named its own
trigger badly (see decision 3).

## Context

**The requirement:** a tenant may run more than one instance of the same product. Two dockets, for
two divisions or two jurisdictions, under one tenant.

**What the estate does today makes that impossible to honour.** `docket_persons` and its four
sibling tables are `-- paridyum:scope tenant`, and the served backend derives the tenant from the
operator's token (ADR-0099/ADR-0082) with no other discriminator. So two docket deployments reached
by one operator resolve the same tenant and read the same rows: **two front doors onto one HR
system**, not two HR systems. Verified against the schema and `serve.rs` on 2026-09-20.

**Three facts that shaped the answer**, each checked rather than assumed:

1. **ADR-0024 is `locked` and is satisfied, not contradicted.** Its decision 9 defers a *third*
   level behind a funded trigger; this uses the *second*, which already exists. Its decision 7 says
   **"accounts are free to create"** — which is precisely the property a tenant adding an instance
   needs, and it was written before anything needed it.
2. **There is no `accounts` table anywhere in the schema.** `account_id` on `applications`,
   `principals` and `profiles` is a bare `uuid` with no foreign key. The inner silo has had a
   column and never an entity.
3. **TLS wildcards match one label.** `*.paridyum.com` covers `acme.paridyum.com` and **not**
   `hr.acme.paridyum.com`. Any nested-subdomain scheme therefore buys a certificate per tenant,
   which is what decided the URL question below rather than taste.

## Decision

### 1. A product instance is an account

`account_id` is the discriminator that separates one instance of a product from another within a
tenant. Two dockets means two accounts, two isolated sets of records, and the two RLS predicates
ADR-0024 already specifies.

**This is the inner silo finally earning its keep.** GLO-0004 calls nested tenancy *"Paridyum's
defining structural property"* and the account boundary has, until now, been a column three tables
carry and nothing enforces a meaning for.

### 2. ADR-0102's retrofit happens now, while the tables are empty

All five person-bearing docket tables hold zero rows (verified 2026-09-20). The change is five
`ALTER TABLE`s and five policy rewrites. ADR-0024 decision 9's warning — *retrofitting an inner
discriminator means rewriting every policy while rows exist* — is avoided by doing it in the window
ADR-0102 bought and `tool-docket`'s backup gate now holds open.

**Had this requirement arrived after the first real HR record, three of those five tables could not
have been retrofitted at all.** `docket_register`, `docket_events` and `docket_signatures` refuse
`UPDATE` by trigger (ADR-0093), so their existing rows could never have been assigned an account.
ADR-0102 cost 1 describes that outcome precisely; this decision is what avoids it.

### 3. GLO-0003's "customer" widens to "the unit an instance serves", and ADR-0102's trigger was the
defect

GLO-0003 reads *"an account is one customer of a tenant's product."* A tenant's second docket for
its UAE entity is not a customer; it is a division. The word was too narrow, and **that narrowness
is what made ADR-0102 reach the wrong conclusion from correct facts.**

An account is **the unit a product instance serves** — a customer, a division, a jurisdiction, a
subsidiary. What makes it an account is that its data must not mix with another's under the same
tenant, and that it has no relationship with Paridyum. Both original properties survive; only the
assumed *kind* of unit widens.

**ADR-0102's trigger was correspondingly too narrow** — *"the first tenant that administers people
on behalf of a distinct client organisation."* The honest trigger was always broader: **the first
tenant that needs two isolated sets of one product's data.** A trigger that names one instance of a
general condition fires late, and this one would have fired never.

GLO-0003 and GLO-0004 are `accepted` and not `locked`; they are corrected under this ADR rather
than superseded.

### 4. `accounts` becomes a table

The inner silo has had a column and no entity since `0001`. An instance is a row: its slug, its
display name, the product it instantiates, when it was created. Without it, `account_id` names
nothing and "list this tenant's docket instances" has no query.

### 5. Instances are addressed by path, not by subdomain

    <tenant-host>/app/<product>/<account>/<the product's own routes…>

    acme.paridyum.com/                          the tenant's landing      ← platform owns the root
    acme.paridyum.com/dashboard                 the tenant's console      ← platform (ADR-0100 D3)
    acme.paridyum.com/app/docket/default/check  the default docket
    acme.paridyum.com/app/docket/hr-india/check another one
    acme.paridyum.com/app/docket                → redirects to …/docket/default

⚠ **This decision was corrected twice, and the second correction is the one that makes it scale.**

**First draft:** the account segment was *optional* — `/docket` the default, `/docket/<account>`
any other — so account names sat in the same position as the product's own top-level routes.
docket serves `/new`, `/check`, `/retention` and `/person`, so `/docket/new` was ambiguous, and an
account named `check` would have shadowed its tenant's own check page.

**Second draft fixed that positionally and left the identical defect one level up.** Products sat
at the tenant host's root, alongside the platform's own `/` and `/dashboard` (ADR-0100 D3) — plus
`/api`, `/healthz`, `/.well-known`. A product slug `dashboard` would collide with the console on
every host, and preventing it needs **a reserved-word list at the root**: exactly what the first
correction refused one level down, and exactly the hardcoded-list failure `class-registry.yaml`,
`check-registry.yaml` and `product-registry.yaml` each exist to kill.

**`/app` is the fix, and it is the same rule applied consistently:**

| Segment | Owner | May collide with |
| --- | --- | --- |
| the root (`/`, `/dashboard`, `/api`, …) | the **platform** | nothing — products are not here |
| `app` | the **platform**, one reserved word, forever | nothing |
| `<product>` | `product-registry.yaml` | nothing — the platform is not here |
| `<account>` | the **tenant** | nothing — products are not here |
| everything after | the **product** | nothing — accounts are not here |

**Every boundary is positional and every namespace has exactly one owner.** A new product can
never collide with a platform path; a new platform path can never collide with a product; a tenant
naming an account can never shadow a route. None of it requires anyone to consult a list.

**`/app` was free**, and the word is not arbitrary: ADR-0100 D3's revision renamed `/app` to
`/dashboard` and left it unused. It is the word the operator reached for originally, restored to
the job it actually describes — a namespace of applications, rather than the console that
provisions them.

⚠ **This decision was written the other way round and was wrong.** The first draft read
*"`<tenant-host>/<product>` the default instance, `<tenant-host>/<product>/<instance>` any other"*
— so the instance segment was optional and sat in the same position as the product's own top-level
routes. The operator caught it immediately, and docket makes the collision concrete: its console
already serves `/new`, `/check`, `/retention` and `/person`.

Under the original scheme `/docket/new` means either *the instance called `new`* or *the default
instance's create-person page*, and nothing can say which. **The failure is worse than ambiguity:
a tenant creating an instance named `check` would shadow its own check page** — a tenant breaking
its own application by naming something.

**The fix is positional, not a reserved-word list.** Segment 2 is the instance and belongs to the
platform; everything after it belongs to the product. An instance named `check` yields
`/docket/check/check`, which is ugly and unambiguous. A reserved-word list was rejected: it is the
"list that must be extended, hardcoded in a place extending it is expensive" failure that
`class-registry.yaml`, `check-registry.yaml` and `product-registry.yaml` each exist to kill — and
it would make **adding a route to any product a breaking change for every tenant** whose instance
happened to share its name.

**The platform must not need to know a product's route table.** That is the real property being
bought, and it is why this costs a segment.

**The bare `/app/<product>` redirects to `/app/<product>/default`**, so the short URL still works
and a tenant with one account never types the word. The canonical form always carries it, because
a URL that is sometimes ambiguous is a URL nobody can safely link to.

**Two reserved words in total, both the platform's own: `app` and `default`.** That is the
distinction that makes reserving them safe and reserving `check` not — the platform may reserve
inside namespaces it owns, and may not reach into a product's or a tenant's.

**THE URL CARRIES NO KEYWORD FOR THE ACCOUNT**, and that is deliberate. Position identifies it, so
nothing has to be called `instance` or `account` in a path. The word matters in the schema and the
glossary, where decision 3 settles it — and `account` is that word, because ADR-0024 is `locked`
with `account_id`. "Instance" stays prose for what an account *does* for a product; it is not a
column, a path segment, or a second name for the row.

**Decided by the certificate constraint, not by preference.** A nested subdomain
(`hr-india.acme.paridyum.com`) reads better and gives each instance its own origin — separate
cookie scope, separate OIDC client — and costs a certificate per tenant, because wildcards do not
nest. A flat one (`acme-hr-india.paridyum.com`) is covered by the existing wildcard and makes the
tenant/instance boundary a hyphen, which stops being parseable the moment a tenant slug contains
one.

**Paths also extend ADR-0100 D3 rather than competing with it.** That decision already fixes
host → tenant and path → surface; an instance is a surface-level distinction, so it belongs in the
path. Nothing about DNS, the routing table, or certificates changes.

**A tenant with one instance still never meets the concept**, because the bare `/<product>`
redirects. What changed from the first draft is that the concept is no longer *absent* from the
canonical URL, only from the one a person types — and that is the difference between a scheme that
is convenient and one that cannot be made ambiguous by a tenant naming something.

### 6. `Release` carries the instance

`releases` is keyed on the account: `account_id` with `UNIQUE (account_id)`. Migration `0012`'s
original `UNIQUE (tenant_id, product)` forbade this requirement by construction and is corrected in
the same PR rather than superseded later.

**`product` is NOT repeated on `releases`.** An account is an instance *of* one product, so
`accounts.product` already holds it and a second copy is the second home RFC-0015 forbids. An
intermediate draft carried `product` *and* an `instance` text slug — the slug duplicating
`accounts.slug`, the product duplicating `accounts.product`. Two facts, four homes.

**`releases` is `scope account`, with a conditional inner predicate**, and the checker is what
established it: an outer-only policy *"isolates businesses from each other and nobody's
customers."* The worry — that a tenant console must list every instance — was already answered by
GLO-0004 point 1, which requires the inner predicate to be conditional on operator scope as an
**explicit grant** rather than an absence.

### 7. This does **not** fire ADR-0024 decision 9

D9 defers a third level behind *"a real third level with a paying customer behind it — a reseller
or white-label partner above the operator, or sub-accounts within an account."* An instance is
neither: it is the second level, used for what the second level is for. **Sub-instances within an
instance would fire D9**, and are out of scope here.

## Consequences

**Accepted costs**

1. **Every docket query and policy gains an inner predicate**, permanently, including for the
   majority of tenants who will run exactly one instance. That is the price of the boundary being
   real rather than notional, and ADR-0024 cost 1 already names the hazard it introduces: a
   permissive default leaks, so `account_id` must be an explicit positive claim and never an
   absence (GLO-0005).

2. **A default instance is still an account**, so "one docket" means one account row, not a null.
   A nullable `account_id` meaning "the default" would be exactly the widening-by-absence
   `db.operator_scope_is_an_explicit_grant` rejects.

3. **ADR-0102 was accepted and superseded on the same day.** The record now carries a decision with
   a lifetime of hours. That is the honest cost of deciding from a stated premise and then being
   given a new requirement — and it is cheaper than the alternative, which was discovering the
   requirement after rows existed.

4. **Two instances share an operator population.** An operator with the `operator` role in the
   tenant's Zitadel org can reach every instance. Per-instance operator grants are a real question
   and are **not** decided here; `operator_grants` (migration `0010`) is the table that would carry
   them.

**Gained**

- **A tenant can run two dockets with genuinely separate records**, which is the requirement.
- **The retrofit happens at its cheapest possible moment** — five empty tables — rather than at its
  most expensive.
- **The inner silo acquires a meaning the whole estate can use.** Any product, not just docket,
  now has an answer to "how does one tenant run two of these".
- **No DNS, certificate or routing work.** Decision 5 costs nothing operationally.

**Rejected alternatives**

- **A third discriminator, `instance_id`, distinct from account.** Keeps GLO-0003's "customer"
  intact. Rejected because ADR-0024 is `locked` at two columns and D9 defers a third behind a
  funded trigger this does not meet — and because the two concepts are indistinguishable in
  practice: both mean "a unit under a tenant whose data must not mix with its siblings".
- **Two instances over one dataset.** No schema change. Rejected because it answers a different
  question — scale or geography, not separation — and a tenant asking for two dockets is not asking
  to see the same people twice.
- **A database or schema per instance.** The strongest isolation. Rejected by RFC-0006 and ADR-0024
  decision 8 on the same grounds they were rejected per tenant, and instances are strictly more
  numerous than tenants.

## Related

- **ADR-0102** — superseded by this; its facts were right and its trigger was too narrow
- **ADR-0024** — `locked`; D7 (accounts are free to create), D9 (a third level is deferred), and
  the two-column mechanism this uses
- **GLO-0003, GLO-0004** — the inner silo, corrected under decision 3
- **GLO-0005** — operator scope is a grant, never an absence; the reason cost 2 exists
- **ADR-0100** — D3's host → tenant, path → surface model, which decision 5 extends
- **ADR-0101** — `platform-deployment` and `Release`, whose uniqueness decision 6 corrects
- **ADR-0093** — the append-only chains that make the timing in decision 2 load-bearing
- **RFC-0016** — per-hostname certificates, and why decision 5 needs none of them
- **T-G.0d** — answered by ADR-0102, and answered differently here
