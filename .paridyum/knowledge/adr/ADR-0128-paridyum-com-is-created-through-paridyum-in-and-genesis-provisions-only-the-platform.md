---
id: ADR-0128
title: paridyum.com is created through paridyum.in, and genesis provisions only the platform
type: adr
status: accepted
created: 2026-09-29
locked_at: null
supersedes: [ADR-0118]
superseded_by: null
scope: [org, platform, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [portal, paridyum-in, paridyum-com, tenant-zero, genesis, bootstrap, saga, adr-0118, adr-0119, adr-0120, adr-0126, rfc-0056]
breaking: true
enforced_by:
  - check: domain.operator_tenant_holds_no_extra_grant
    severity: error
    rationale: >
      Carried from ADR-0118. Paridyum is on both sides of the portal: the platform that provisions,
      and a business that is provisioned. Neither side may hold a grant in the other that a customer
      could not be given.
  - check: domain.no_tenantless_request_path
    severity: error
    rationale: >
      Carried from ADR-0118 (ADR-0116 D1). The portal serves orgs, in the org context.
verified_by:
  - TST-0016
  - TST-0019
---

# ADR-0128 — paridyum.com is created through paridyum.in, and genesis provisions only the platform

## Status

**Accepted** 2026-09-29. The operator directed that paridyum.in brings paridyum.com up "as we want",
and chose, when asked, to create Paridyum's tenant **through the API as proof that the flow works**
rather than adopt the one tooling made. Supersedes ADR-0118. Its decisions 1, 2, 4, 5 and 6 are
carried here **unchanged**, under the same numbers; only decision 3 changes.

## Context

ADR-0118 D3 had tooling make Paridyum's org and tenant at genesis, and the portal adopt them later as
they stood. It rejected provisioning Paridyum's tenant through the portal first, because *"the portal
does not exist on the day of the wipe; waiting for it would leave paridyum.com dark."*

Two things have changed since.

- **paridyum.com is not waiting on the portal to stay up.** It is served today by deployments that
  keep serving while the portal is built, and the switch to the tenant the portal creates is one
  short cutover at the end.
- **Adoption proves nothing about the flow.** The operator's direction is that paridyum.in brings
  every tenant up and that nothing is hand-spun (PLAN-PARIDYUM-IN §0). A first tenant that the flow
  never created leaves the flow unproven for exactly the tenant everyone will look at. ADR-0118's
  own accepted cost 3 said so: *"for the tenant that already exists it is not the same flow as
  anyone else".*

## Decision

**1 (carried). The portal is the platform's own surface, at paridyum.in, and it runs in no tenant.**
As ADR-0118 D1.

**2 (carried). Paridyum's business is an ordinary tenant, owned by the Paridyum org.** As ADR-0118 D2.

**3. Genesis provisions only the platform; paridyum.com is created through paridyum.in.**

Genesis — by tooling, because nothing else can yet — provisions exactly what the portal and the saga
need in order to run:

1. the **staff instance** (ADR-0119 D1);
2. the region's **tenant instance**;
3. the region's **portal instance** and its sign-in host under paridyum.in (ADR-0119 D1,
   ADR-0121 D6);
4. the shared deployments, the database and its migrations, the edge.

**No org and no tenant is made by genesis.** Then, through the portal, as anyone would:

5. **The operator joins paridyum.in** (RFC-0056) and creates the **Paridyum org**.
6. **The Paridyum org creates Paridyum's tenant.** It gets the slug `paridyum`, `paridyum.com` as its
   own domain, and the products it enables. The saga does the rest: the Zitadel organization,
   federation to the portal instance, the per-product clients (ADR-0126), the Releases, the host
   bindings, and the creator's first grant (ADR-0120 D2).
7. **Cutover.** paridyum.com is served by the tenant the saga created. The rows and the Zitadel
   organization that tooling made on 2026-09-28 are removed, not adopted. They hold only synthetic
   rows (ADR-0071's gate has kept real personal data out).

**From then on paridyum.com changes only through the portal,** as any tenant's host does. Each new
region's genesis repeats steps 2 and 3 for that region.

**4 (carried). Surfaces.** As ADR-0118 D4.

**5 (carried). The saga runs for an org.** As ADR-0118 D5.

**6 (carried). Neither side holds a path into the other.** As ADR-0118 D6.

## Consequences

**Accepted costs.**

- **paridyum.com's tenant id and Zitadel organization change at cutover.** Values that name them —
  environment variables, seeds and runbooks — are replaced by what the saga records, and the removed
  organization's sessions end.
- **A short cutover for paridyum.com,** and the first public signup is Paridyum's own. RFC-0056 §F's
  personal-data gate therefore binds the operator's own signup too: it is exercised against synthetic
  identities until that gate opens.
- **Genesis must still be exercised on a schedule** (ADR-0118 accepted cost 2, carried), but it is now
  smaller: platform only.

**Gained.**

- **The first tenant is the flow's first proof,** not its exception. ADR-0118 accepted cost 3 is gone.
- **Genesis holds no business.** Nothing it makes is a tenant or an org, so nothing needs adopting.

**Rejected alternatives.**

- **Adopt the tooling-made tenant (ADR-0118 D3).** It is faster, and it leaves paridyum.com the one
  tenant the flow never made.
- **Create Paridyum's tenant through the portal, and keep the tooling-made one beside it.** Two
  Paridyum tenants would each claim `paridyum.com`, and `domain_bindings` rightly refuses that.

## Related

- ADR-0118 — superseded; decisions 1, 2, 4, 5 and 6 carried.
- ADR-0119 — the instances, and federation to the portal instance.
- ADR-0120 — the creator's first grant.
- ADR-0126 — per-tenant clients and host resolution.
- RFC-0056 — joining paridyum.in, and its §F personal-data gate.
- PLAN-PARIDYUM-IN — the sprint that builds this.
