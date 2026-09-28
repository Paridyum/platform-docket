---
id: ADR-0117
title: An org has a region, its registry row is personless, and a person holds one credential in one region
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy, identity]
tags: [regions, residency, control-plane, data-plane, org, identity, login, federation, portal, rfc-0014, rfc-0015, rfc-0017, rfc-0051, glo-0007, adr-0041, adr-0047, adr-0089, adr-0090, adr-0114, adr-0115]
breaking: true
enforced_by:
  - check: db.no_personal_data_in_the_control_plane
    severity: error
    rationale: >
      Decision 3 made mechanical. RFC-0014 cost 2 names the hazard — "a pii column landing in the
      control plane instead of a region is a residency breach that every check reports as green".
      Migrations classify columns inline and declare their plane, so a control-plane table carrying a
      pii or confidential column, or an unclassified one, is decidable by reading the migration.
  - check: db.org_scoped_tables_are_regional
    severity: error
    rationale: >
      An org's own rows are its people. This is the check that refuses them a control-plane table.
verified_by:
  - TST-0016
  - TST-0020
  - TST-0021
sources:
  - url: https://learn.microsoft.com/en-us/entra/external-id/what-is-b2b
    title: "What is Microsoft Entra B2B collaboration? — Microsoft Learn"
    retrieved: 2026-09-28
  - url: https://auth0.com/docs/get-started/auth0-overview/create-tenants
    title: "Create Tenants — Auth0 Docs"
    retrieved: 2026-09-28
---

# ADR-0117 — An org has a region, its registry row is personless, and a person holds one credential in one region

## Status

**ACCEPTED** 2026-09-28 by the operator, answering the question *"how we run a multi-region cloud —
would that model still persist? all the orgs can spin up in all those different regions"*: yes, in
the shape below.

**It rests on RFC-0014, RFC-0015, RFC-0017 and RFC-0051, which are `proposed`.** It takes them as
direction and decides only what the org model needs from them; accepting them remains its own act.
RFC-0014 D3 and D4 are resolved by ADR-0115.

## Context

**The estate designed for many regions before this question was asked.**

- **RFC-0014 D2** gives three planes: the **edge** (global, rented, nothing durable), the **control
  plane** (the control region, one — *"the global skeleton — which operator owns which tenant, in
  which region, on which plan — plus the region registry, domain bindings, and staff identity. **No
  operator personal data**"*) and a **data plane** per region. *"No plane writes into another."* In
  that sentence the operator is the org (ADR-0113 D1).
- **RFC-0015 D1:** *"Regional data stays in its region… Nothing is copied into a region."* Login and
  business data never need the control plane; plan- and quota-gated actions do; provisioning is *"the
  one operation that spans both planes."* D2: one region live. D5: the region is chosen explicitly and
  *"never inferred from the address a signup arrives from."*
- **RFC-0017** splits two populations: **staff** — one directory, control region — and **customers**,
  whose people live *"in their own region, beside the tenants they run"*. Only *"a skeleton with no
  person in it"* stays central. It already imagines a customer *"with businesses on two continents."*
- **GLO-0007** retires the older name for the control region: there is the control region (ours,
  one) and a tenant's region.
- **State.** `region-registry.yaml` has one live region, `in` — `control: true` — and further
  jurisdictions `on_request`.

## Decision

**1. An org has a region: the region its own people's data lives in.**

- Chosen **explicitly at signup** from the regions that exist, on RFC-0051's terms: labelled by regime,
  ordered by `suggest_for` against the visitor's country, nothing preselected, never inferred, refused
  rather than defaulted.
- **Immutable except by migration and contract amendment** — ADR-0041 D3's rule for a tenant's
  region, extended to the org.
- It is the org's *region* — never the retired name GLO-0007 replaced (ADR-0113 D5).

**2. An org's tenants may be in any region.** Each tenant's region is chosen when it is created
(RFC-0015 D5), independently of its org's. An org in `in` may own a tenant in `eu`, and that tenant's
data never touches `in`.

**3. The org is split across the planes by the kind of fact.**

| Fact | Plane | Where |
| --- | --- | --- |
| id, region, status, plan and entitlement; which tenants it owns (`tenants.owner_org_id`), and each tenant's region | **control** | the org and tenant registries — `platform`-scoped, **no person, no name** |
| name, handle, members, invitations, roles, contacts, locale defaults, audit trail | **regional** | `org`-scoped tables in the org's region |

**One id, two homes for two kinds of fact** — RFC-0015's rule applied, not an exception to it.
**No control-plane table carries a `pii` or `confidential` column, and every one classifies its
columns** (`db.no_personal_data_in_the_control_plane`). A tenant's display name is regional too; the
registry keeps the slug its hosts need.

**4. The portal is one host, and every data request is served by a region.**

- **paridyum.in is served from the edge as one host.** Its static interface carries no data.
- **Each region runs the portal's regional backend**, over that region's database. A session is
  served by the region its credential belongs to (decision 6), never by a request parameter.
- **Listing an org's tenants reads the control plane** — ids, slugs, regions, states. Opening one
  reads that tenant's region, under a per-tenant grant (ADR-0120).

**5. Identity follows the populations** (RFC-0017):

| Population | Instance | Where |
| --- | --- | --- |
| **Staff** — Paridyum's own people | the **staff instance** | the control region. ADR-0047 D1 narrowed as RFC-0017 asks: it holds staff, and "operators" leaves the sentence |
| **Org members** — the people of every org | a **portal instance** per region | each org's region. Its one Zitadel organization is a directory; it mirrors no org and no tenant (ADR-0113 D2) |
| **Tenant users** — a tenant's staff and customers | the **tenant instance** per region | the tenant's region. One Zitadel organization per tenant (ADR-0047 D2, unchanged) |

**6. A person holds one credential, in one region — the region they signed up in.**

- **Signing in to the portal starts with the region.** It is offered on RFC-0051's terms, remembered
  on the device, and named in every link a region's mail carries.
- **There is no global index from a person to a region.** Any such index — even a hash of an address
  — is personal data in the control plane.
- Auth0 meets the same constraint by making the region part of the host: `example-co.au.auth0.com`.

**7. A person admitted outside their credential's region is recorded by whoever admitted them.**

When an org or tenant in region R2 admits a person whose credential lives in R1 — an org invitation,
or a per-tenant grant — **R2 holds its own record of that person**: a linked user, federated to R1's
portal instance (ADR-0119 D4), plus the membership or the grant.

- **Minimal**: subject, display name, a contact address, and language, locale and timezone
  preferences.
- **Consented** at the invitation or grant, **refreshed** at each sign-in, **removed** with the last
  membership or grant. Its history stays in the admitting side's append-only audit (ADR-0044).
- **It is the admitting side's own fact** — who it let in, and what they did. It is not a copy of the
  person's account, which never leaves R1. Microsoft Entra B2B works the same way: partners *"use
  their own credentials"*, and once admitted are *"represented in your directory as a user object."*

**8. Five things cross a region boundary, and nothing else.**

1. Gated actions read the control plane.
2. The provisioning saga writes the control plane and the target region.
3. Sign-in redirects to the credential's region.
4. A federation assertion creates or refreshes an admitting side's record (decision 7).
5. The portal reads the control plane — identifiers and regions only — to list an org's tenants.

Nothing is replicated (RFC-0015 D1).

**9. Staff reach a tenant or an org the same way anyone does.** A support grant admits a
staff-instance credential into one tenant or one org (ADR-0120), recorded by that tenant or org in its
own region — the same record as any provider's.

**10. One region stays live** (RFC-0015 D2).

- The model is **N-region-correct in shape**: region columns, plane declarations, per-region identity
  hosts, and the skeleton/person split.
- **Nothing is built for a second region** until its trigger fires.
- In `in` today, the staff instance and `in`'s tenant instance share one Zitadel deployment as virtual
  instances (ADR-0090). **`in`'s portal instance is added with the portal** (ADR-0118 D3), as a third.

## Consequences

**Accepted costs**

1. **An org's people depend on their region being up** (RFC-0017 cost 1). Accepted: nothing there
   could be administered anyway.
2. **Sign-in starts with a region.** One extra step on a device's first sign-in, and a person who
   signs up in two regions has two credentials.
3. **Two identity populations, permanently** (RFC-0017 cost 2). Paridyum's own people hold a staff
   identity, and a portal identity as members of the Paridyum org.
4. **Admitting-side records are personal data held in another region, by design.** Consented and
   minimal, and they belong in every attestation and processing record that says where personal data
   lives (RFC-0045 §E; REQ-0003 item 8).
5. **The control plane is the only place that knows an org's tenants across regions.** Its outage
   stops gated actions and provisioning — not sign-in and not tenant traffic (RFC-0015 D1's failure
   shape, unchanged).
6. **A tenant's display name leaves the tenant registry**, so the portal's list shows slugs until a
   tenant is opened.

**Gained**

- **Residency is true for every person** — customers, their people and staff — not only for a
  tenant's end users.
- **An org can run tenants in any region** without its people's data following them.
- **The model reaches N regions with no redesign.** Adding a region is RFC-0014's legal act plus
  genesis.

**Rejected alternatives**

- **Every org's people in the control region.** It reopens exactly the hole RFC-0017 closed.
- **A control plane per continent.** RFC-0017's rejected alternative 1: N staff directories, and an
  org with tenants on two continents becomes unrepresentable.
- **A global address-to-region index.** Personal data in the control plane; hashing does not change
  that.
- **Inferring the region from the visitor's address or language.** RFC-0015 D5 and GLO-0006 refuse it.
- **Replicating a person into every region they work in.** It breaks RFC-0015 D1; decision 7's record
  is scoped, minimal and owned by the admitting side instead.

## Related

- **RFC-0014** · **RFC-0015** (D1, D2, D5) · **RFC-0017** · **RFC-0051** — `proposed`, load-bearing
- **GLO-0007** · **GLO-0008** · **ADR-0041** D3 · **ADR-0047** D1/D2 · **ADR-0089** D1 · **ADR-0090**
- **ADR-0044** · **RFC-0045** §E · **REQ-0003** item 8
- **ADR-0114** · **ADR-0115** · **ADR-0118** · **ADR-0119** · **ADR-0120** · **ADR-0121** ·
  **ADR-0122**
- Research record `docs/2026-09-28-org-model-regions-and-locales-research.md`, Part B
