---
id: ADR-0118
title: paridyum.in is the platform's own surface, and Paridyum's business is an ordinary tenant owned by the Paridyum org
type: adr
status: superseded
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: ADR-0128
scope: [org, platform, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [portal, paridyum-in, paridyum-com, tenant-zero, genesis, bootstrap, release, console, control-plane, adr-0049, adr-0100, adr-0114, adr-0116, rfc-0022, rfc-0045, rfc-0047, rfc-0049, c-0008, plan-tenant-flow]
breaking: true
enforced_by:
  - check: domain.operator_tenant_holds_no_extra_grant
    severity: error
    rationale: >
      Existing (ADR-0049 D2, carried by ADR-0116 D2). Separating the provisioner from Paridyum's
      business creates the one place the rule is most tempting to break: Paridyum on both sides.
      Neither side may hold a grant in the other that a customer could not be given.
  - check: domain.no_tenantless_request_path
    severity: error
    rationale: >
      Existing (ADR-0116 D1). The portal is the surface most likely to be argued into running
      "above" every context; decision 1 gives it the org context instead.
verified_by:
  - TST-0016
  - TST-0019
---

# ADR-0118 — paridyum.in is the platform's own surface, and Paridyum's business is an ordinary tenant owned by the Paridyum org

## Status

**ACCEPTED** 2026-09-28 by the operator. It amends, with dated notes that keep the original text:

- **ADR-0100** — decisions 2, 3 (as revised on 2026-09-15), 4 and 6;
- **RFC-0049** — §B and §D;
- **RFC-0045**, **RFC-0047** and **TASKS** — wherever tenant zero holds the control plane;
- **PLAN-TENANT-FLOW** — D3, D4 and D8.

## Context

**The operator, 2026-09-28:**

> *"Our main portal is the business-as-a-service at paridyum.in, and using it we are deploying
> paridyum.com, so we can follow everything in the same manner as anyone else would receive them."*

**Until today one tenant was both the provisioner and a business.**

- ADR-0100 D3, as revised on 2026-09-15, made `paridyum.com/dashboard` *"Paridyum's dashboard — which
  is the control plane, because Paridyum holds that Release."*
- RFC-0049 §B drew tenant zero holding `landing`, `careers`, `docket`, `admin` **and** `app` — the
  control plane.
- So Paridyum's business was the one tenant that also provisioned every other, and "the same manner
  as anyone else" was false for exactly that tenant.
- ADR-0100's own ⚠ named what the 2026-09-15 revision gave up: the control-plane path existed on every
  host and *"no longer protects anything by itself"*.

**The org model removes the reason for it.** ADR-0100 D2 put the provisioner inside tenant zero
because ADR-0049 D1 required every request to resolve a tenant. ADR-0116 D1 lets the portal resolve
the **org** it acts for instead, and gives staff a platform context that can read nothing but the
personless skeleton. Nothing needs to live inside Paridyum's business any more.

**The estate is rebuilt from empty once this lands**, on the operator's instruction of 2026-09-28
(*"it's test data, nuke it and start fresh"*). Its application database held test data only.
Paridyum's tenant is therefore provisioned fresh, not carried over, and genesis (decision 3) is
written for an empty estate.

## Decision

**1. The portal is the platform's own surface, at paridyum.in, and it runs in no tenant.**

- **It serves orgs**, in the org context (ADR-0116 D1). It is a served surface and a BFF over
  `platform-tenancy`'s org and tenant APIs — ADR-0100 D1's rule, carried: it writes no entity itself
  and never touches another context's tables (ANTI-0003).
- **`web-dashboard` becomes the portal**, the surface of the `paridyum` platform product.
- **It holds no tenant's data**, and it reads the tenant registry only for the tenants the session's
  org owns (ADR-0114 D6).

**2. Paridyum's business is an ordinary tenant, owned by the Paridyum org.**

- **Tenant zero means Paridyum's tenant** — C-0008's *Paridyum as its own first customer* — and
  nothing else. Its canonical host is paridyum.com (ADR-0121).
- **Its Releases are business Releases**: the landing (`web-paridyum`), careers (`web-hiring`),
  `docket`, and its console. None of them provisions anything.
- **The Paridyum org is an org like any other** (ADR-0114 D8). Its region is chosen on the same
  terms as anyone's (ADR-0117 D1).

**3. Genesis is the one asymmetry, and it is written down.**

Until the portal and the saga run, nothing can be provisioned *through* them, so the first things are
provisioned *by the tooling they wrap*, in this order:

1. The **staff instance** — Paridyum's people (ADR-0119 D1).
2. The **tenant instance** of the live region.
3. **Paridyum's tenant**: its Zitadel organization in the tenant instance, and its registry row, by the
   tenant-provisioning tool (`provision-tenant.py`, ADR-0113 D4).
4. **The Paridyum org's registry row**, as that tenant's `owner_org_id`.
5. **Later, with the portal:** the region's **portal instance**, the portal's first deployment, and the
   Paridyum org's members.

**When the portal and the saga run, the Paridyum org and its tenant are adopted as they stand** —
recorded, never re-provisioned — and **from then on paridyum.com changes only through the portal**, as
any tenant's host does. Each region's genesis repeats steps 2 and 5 for that region.

**4. Surfaces.**

| Host | `/` | `/dashboard` |
| --- | --- | --- |
| **paridyum.in** | the portal's public face | the portal (orgs) |
| **a tenant's canonical host** — paridyum.com included | that tenant's landing | **that tenant's console** (`web-console`, PLAN-TENANT-FLOW D4) |

- **`paridyum.com/dashboard` is identical in kind to any tenant's `/dashboard`.**
- **The control plane is reachable on no tenant host.** ADR-0100 D3's lost defence is restored, more
  strongly: a separate registrable domain, separate cookies, a separate OIDC client (ADR-0070 D1) and a
  separate content security policy.
- **Tenant identity is configured at the portal**, as the operator directed (*"IdP and business-level
  configuration happens here at paridyum.in"*). Whether `web-admin` is retired into the portal (a
  complete rename, STD-0022) or keeps a role on tenant hosts is decided when the portal's identity
  screens are designed.

**5. The saga runs for an org.** ADR-0100 D2's *"provisioned by tenant zero"* now reads *"provisioned
for the requesting org"*. It runs within that org's entitlement (ADR-0114 D5) and ends by:

- recording ownership (`tenants.owner_org_id`);
- issuing the owner's first per-tenant grant (ADR-0120 D2);
- requesting the tenant's entitled Releases (RFC-0049 §E).

**6. Neither side holds a path into the other.**

- **Staff operate the platform in the platform context** (ADR-0116 D1), and hold **nothing in
  Paridyum's tenant** unless granted there like anyone else (ADR-0120).
- **The Paridyum org's members hold nothing in the portal** beyond what any org's members hold.
- ADR-0116 D2 applies to both.

## Consequences

**Accepted costs**

1. **"Tenant zero" changes meaning for some readers.** ADR-0100, RFC-0045, RFC-0047, RFC-0049 and
   TASKS used it for the holder of the control plane; each carries a dated note, and every reader who
   learned the old sense must be re-pointed.
2. **Genesis is a permanent special case.** It is not produced by the saga, and its path must stay
   runnable and exercised on a schedule — RFC-0049 §D's rule, *"a path nobody uses must be exercised
   on a schedule."*
3. **Adoption is a one-time reconciliation**, and for the tenant that already exists it is not "the
   same flow as anyone else". Every change after it is.
4. **Paridyum's own people work across two instances** — staff, and later the portal — and reach
   Paridyum's tenant through federation and grants, not shared users (ADR-0119).

**Gained**

- **paridyum.com is received exactly as any tenant's host**: the same console, the same saga, the same
  grants, the same screens in the portal.
- **The provisioner is on its own origin**, and ADR-0100 D3's ⚠ is closed rather than lived with.
- **RFC-0022's separation becomes structural.** Paridyum-the-landlord (the platform) and
  Paridyum-the-competing-business (its tenant) hold nothing in each other.

**Rejected alternatives**

- **Keep one tenant zero holding both** (the status quo). "The same manner as anyone else" would stay
  false for exactly the tenant that matters.
- **Run the portal inside a tenant of its own.** It needs no new context, and it makes the portal a
  business and every org that business's account — the mixing ADR-0113 ends.
- **Provision Paridyum's tenant through the portal first.** The portal does not exist on the day of the
  wipe; waiting for it would leave paridyum.com dark for no gain the adoption does not already give.

## Related

- **ADR-0100** — D2, D3, D4, D6 amended · **RFC-0049** — §B, §D · **PLAN-TENANT-FLOW** — D3, D4, D8
- **ADR-0116** — the org and platform contexts · **ADR-0114** — the Paridyum org · **ADR-0119** — the
  instances · **ADR-0120** — the owner's first grant · **ADR-0121** — paridyum.com's role
- **C-0008** — tenant zero is Paridyum as a customer · **RFC-0022** — information separation ·
  **RFC-0047** — renting the platform stays a stamp
