---
id: ADR-0047
title: Identity is self-hosted Zitadel, one organization per tenant, proxied but never minted
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, mobile, sdk, spec, infra]
applies_to: ["*"]
tags: [identity, zitadel, multi-tenancy, oidc, tenancy, vendor, rfc-0012]
breaking: false
enforced_by:
  - check: db.session_context_accessors_fail_closed
    severity: error
    rationale: >
      Decision 4 makes the tenant a signed claim rather than a verified issuer, and decision 3
      makes the account boundary Paridyum's alone. Both land on the same three functions, so the
      accessor that raises instead of returning NULL is where this ADR's isolation model
      actually lives.
  - check: openapi.error_responses_declare_a_code
    severity: error
    rationale: >
      Every product validates tokens locally and returns its own errors; a client that must
      parse English to distinguish "not entitled" from "wrong audience" cannot be an SDK.
---

# ADR-0047 — Identity is Zitadel, one organization per tenant

## Status

**ACCEPTED** as of 2026-08-08. **Resolves RFC-0012**, which holds the analysis, the vendor
evidence and the capability matrix. This records only what was decided.

## Decision

**1. Self-hosted Zitadel, one deployment per region**, Postgres 14–18, one shared virtual
instance per region. A separate `paridyum` instance in the home region holds staff and operators
— it does not replicate, because N regional copies of a staff directory means N joiner processes
and N places to forget to deprovision.

**2. A tenant is a Zitadel organization. An account is not a Zitadel object.**

| Paridyum | Zitadel |
| --- | --- |
| tenant (GLO-0002, outer silo) | **organization** |
| account (GLO-0003, inner silo) | *none — Paridyum's own tables* |
| product | **project**, granted to tenant orgs |
| interface | **application** |

Organization-per-tenant rather than instance-per-tenant because BET-0004 makes the product a
**catalogue**, and a Project Grant is organization-to-organization **within one instance**. No
grant crosses an instance boundary. At 100 tenants × 10 products, instance-per-tenant is 1,000
Project definitions each needing update on every release; this is one Project, one update, N
grants.

**3. A custom login domain is a paid upgrade, not a tenancy model.** Zitadel's System API adds
multiple custom domains to one instance, and branding follows the domain. A premium tenant gets
`login.theirbrand.com` for a DNS record and an API call.

**4. Zitadel issues tokens. `platform-identity` never mints and never validates.** It terminates
the control plane (provisioning, tenant self-service, capability gating) and the login plane
(Session API, custom login UI). Products fetch JWKS and validate locally — a proxy on the
validation path is a hop on every authenticated request and a single point of failure for the
platform.

**5. The claims contract:** tenant from `urn:zitadel:iam:user:resourceowner:id`, account resolved
from Paridyum's own tables on the signed `sub`, roles from the project-roles claim, and `aud`
must contain the product's project id. Every resource server validates **`iss` and `aud`**, not
signature alone.

**6. Authorization federates to products.** Identity answers *who, in which tenant, holding which
granted roles*. Each product answers whether that principal may act. Centralising per-product
permissions is how this becomes ANTI-0006.

**7. Three capabilities are never delegated to a tenant:** the System API (it creates and
destroys organizations), Actions (custom code in the token flow, in a process shared by every
tenant in the region), and quotas (they are the meter).

## Consequences

**Accepted costs**

1. **The inner silo has one isolation layer.** Zitadel offers no level below the organization, so
   account↔account is Postgres RLS alone, with ADR-0038 decision 5's repository assertion beside
   it. ADR-0045 R1 records this, and it is the largest single cost of decision 2.

2. **Metering leaves the vendor.** Quotas are per instance, so per-tenant metering moves to
   `platform-identity`, which sees logins and monthly actives but not per-API-call usage inside
   products.

3. **Every tenant's identity data shares one Postgres per region.** A database compromise is
   regional (ADR-0045 R4), and it is inherent to one deployment per region.

4. **`platform-identity` is the highest-availability service we will operate.** Off the
   validation path, so an outage stops new logins rather than all traffic — a materially smaller
   blast radius, and still the one that must not fail.

5. **Vendor concentration.** Zitadel becomes as load-bearing as Postgres, and the masterkey is
   immutable per region. REQ-0012 owns it.

**Gained**

- A new product is a Project, roles, applications and a grant — no identity work, which is the
  test of whether this is a platform.
- White-label, headless login, federation at both silos, and SCIM per organization are vendor
  capabilities rather than roadmap.
- The catalogue scales without a reconciler, a fan-out, or drift detection.

**Rejected alternatives** — argued in full in RFC-0012, summarised here: building identity
in-house (reimplements OIDC, SAML, passkeys and conformance); Zitadel Cloud (loses regional
placement, which residency requires); instance-per-tenant (decision 2); and
`platform-identity` minting its own tokens, which would demote tenant identity from a signature
guarantee to a claim our own proxy writes.

## Related

- RFC-0012 — the analysis, capability matrix and vendor evidence this ratifies
- ADR-0038 — claims to session context, and the account assertion
- ADR-0039 — the saga that creates these resources
- ADR-0042 — `platform-identity` as one of five first-wave contexts
- ADR-0045 — the threat model, whose R1 is cost 1 stated from the other side
- ADR-0041, ADR-0044 — compliance sequencing and the audit trail this must feed
- BET-0004 — the catalogue wager decision 2 depends on
- GLO-0004, ADR-0024 — the two silos and the tenancy model this maps onto
