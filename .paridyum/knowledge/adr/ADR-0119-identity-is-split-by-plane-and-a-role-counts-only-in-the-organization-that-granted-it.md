---
id: ADR-0119
title: Identity is split by plane, and a role counts only in the organization that granted it
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, web, infra, spec]
applies_to: ["*"]
product: [identity, paridyum]
tags: [identity, zitadel, planes, instances, federation, oidc, login-domain, mail, dkim, claims-contract, lib-auth, roles, staff, portal, adr-0047, adr-0067, adr-0070, adr-0082, adr-0085, adr-0088, adr-0089, adr-0090, adr-0091, adr-0113, adr-0117, rfc-0012, rfc-0017]
breaking: true
enforced_by:
  - check: test.role_is_honoured_only_in_its_granting_organization
    severity: error
    rationale: >
      Decision 5 is a property of two published libraries, and the failure it prevents is silent: a
      role granted in one Zitadel organization read as a role in another, with every request
      succeeding. The check requires the negative test in both libraries, and it must be watched
      failing against today's code before it is trusted.
verified_by:
  - TST-0019
  - TST-0020
sources:
  - url: https://zitadel.com/docs/guides/integrate/identity-providers/introduction
    title: "Identity providers — ZITADEL Docs"
    retrieved: 2026-09-28
  - url: https://zitadel.com/docs/guides/integrate/retrieve-user-roles
    title: "Retrieve User Roles in ZITADEL — ZITADEL Docs"
    retrieved: 2026-09-27
  - url: https://zitadel.com/docs/concepts/features/external-user-grant
    title: "External User Role Assignments — ZITADEL Docs"
    retrieved: 2026-09-27
---

# ADR-0119 — Identity is split by plane, and a role counts only in the organization that granted it

## Status

**ACCEPTED** 2026-09-28 by the operator, whose answer was **by plane**. It replaces, for everything
except the portal, the 2026-09-27 direction that all identity and mail re-home to `.in`.

It amends ADR-0047 D1, D3 and D5, and adds dated notes to ADR-0085, ADR-0089, ADR-0090 and ADR-0091.
RFC-0012 D2's separate issuers are kept.

**The rebuild measures before it changes.** The instances are recreated from their registries after
the 2026-09-28 wipe; the hosts below are the target, and what the instances report on the day is the
record.

## Context

**Two virtual instances existed** (ADR-0090): the tenant instance, logging in at `auth.paridyum.com`,
and the control-plane instance, logging in at `id.paridyum.com` — *"one human identity per person"*
for staff (ADR-0089 D1). RFC-0012 D2 keeps them as separate issuers.

**RFC-0017 decides who lives where.** Staff — Paridyum's own people — keep one directory in the control
region, because ADR-0047 D1's argument (*"N regional copies of a staff directory means N joiner
processes and N places to forget to deprovision"*) is sound about staff. Customers — orgs and their
members — live in their own region (ADR-0117 D5).

**A tenant's customers must sign in, and receive mail, on the tenant's own domain.** BET-0005: *"a
tenant's end customers need never learn Paridyum exists."* Paridyum's tenant is held to the same rule:
its customers sign in on paridyum.com.

**An org member who operates a tenant needs one credential** (ADR-0089). Zitadel supports this across
instances: an identity provider can be linked *"exclusively to a specific organization"*, a *"generic
OIDC provider"* lets one instance be another's provider, and *"automatic creation"* and *"automatic
linking"* create or link the user at first login.

**The libraries drop where a role was granted** (research brief F-H). `lib-auth-rust`'s
`roles_from_claims` returns role names only; `lib-auth-typescript` returns `Object.keys(rolesClaim)`;
both derive the tenant from `resource_owner`. Zitadel's roles claim maps each role to the Zitadel
organization ids it was granted in, and Zitadel lets an organization *"grant access to projects within
your organization to users from other organizations."* So a role granted in one organization would be
honoured in another. Nothing creates such an assignment today.

## Decision

**1. Three populations, three kinds of instance.**

| Population | Instance | Where | Login host | Mail from |
| --- | --- | --- | --- | --- |
| **Staff** — Paridyum's own people | the **staff instance** (today's control-plane instance) | the control region | `id.paridyum.com` — it is the company's, like its mail (decision 3) | paridyum.com |
| **Org members** — the people of every org | a **portal instance** per region | each org's region | a paridyum.in host for that region (ADR-0121 D6) | paridyum.in's sending subdomain |
| **Tenant users** — a tenant's own staff and customers | the **tenant instance** per region (ADR-0047 D2) | the tenant's region | the tenant's own domain (decision 2) | the tenant's own domain |

- **ADR-0047 D1 is narrowed as RFC-0017 asks:** the staff instance holds staff, and "operators" leaves
  the sentence.
- **Each region's portal instance is new.** A virtual instance in that region's Zitadel deployment,
  with its own login deployment — ADR-0090's pattern. **Its one Zitadel organization is a directory: it
  mirrors no org and no tenant** (ADR-0113 D2). Org membership and org roles are ours (`org_members`,
  ADR-0114 D3), never Zitadel's.
- **In a tenant instance, one Zitadel organization is one tenant** (ADR-0047 D2, unchanged; ADR-0082:
  the tenant's id derives from it).
- **Portal system mail** uses ADR-0085's mechanism — DKIM, DMARC tightening from `p=none` — on
  paridyum.in's own sending subdomain.

**2. The tenant plane lives on each tenant's own domain.**

- **Each tenant's Zitadel organization gets a login host on its canonical host**: `login.<its domain>`,
  or `login.<slug>.<app zone>` while it has none (ADR-0121). ADR-0047 D3's custom login domain stops
  being an upgrade and becomes the standard path.
- **Paridyum's tenant keeps `auth.paridyum.com`**, or renames to `login.paridyum.com` within its own
  domain — the operator's choice; either satisfies this decision.
- **A tenant's system mail sends from its own domain**, DKIM-signed per tenant. **Paridyum's tenant
  keeps `noreply@mail.paridyum.com`** exactly as ADR-0085 has it.
- **The portal is where an owner configures all of it**, through `platform-identity`: login domain,
  branding (ADR-0084), login policy, the identity providers the tenant's customers may use, SSO for the
  tenant's staff, and the sending domain. The service runs on the tenant's domain; only its settings
  are set at paridyum.in.

**3. Paridyum's company mail and person addresses stay on paridyum.com.** ADR-0067's
`firstname.<person id>@paridyum.com` and ADR-0088's forwarding are unchanged. They belong to the
company; the portal holds no person addresses of its own.

**4. Cross-plane access is federation, never a shared user store.**

- **At provisioning, a tenant's Zitadel organization links the owner org's regional portal instance**
  as an organization-level generic OIDC identity provider — "Continue with Paridyum" — with automatic
  linking on.
- **When a grant admits someone whose credential is in another region**, that region's portal
  instance is linked too; so is the staff instance, for a support grant (ADR-0120). A tenant's
  organization trusts exactly the instances its grantees come from, and no others.
- **An org member signs into a tenant with their one credential.** Zitadel links a user in the
  tenant's organization to it, and that linked user is the tenant's own record of them (ADR-0117 D7).
- **The portal grants roles in the tenant to that linked user**, per tenant (ADR-0120).
- **The issuer boundary (RFC-0012 D2) is kept, and no grant crosses an instance** — ADR-0047 D2's own
  rationale.
- **A tenant's customers never need a paridyum.in identity.** They are native users of the tenant's
  organization.

**5. A role counts only in the Zitadel organization it was granted in.** The claims contract (ADR-0047
D5) changes:

- the libraries keep, for each role, the organization ids the roles claim says it was granted in;
- **a role is honoured only if it was granted in the organization the tenant is derived from**;
- a token holding `operator` granted anywhere else confers nothing, in either tenant.

This lands in both libraries in ADR-0113's stage 1, **before anything creates a cross-organization
assignment.** Federation (decision 4) means none is needed for the flows here, so this guards the one
someone eventually clicks in a console.

**6. Staff single sign-on stays in the staff instance, and staff are never customers' neighbours.**

- **Estate-operations systems federate to the staff instance, unchanged** (ADR-0089 D2): the
  Kubernetes API, observability, the consoles — granted only through the operations project, which
  ADR-0070 D2 says is *"not granted to anyone"*.
- **Staff and customers share no user store.**
- **Paridyum's people who run Paridyum's tenant are members of the Paridyum org**, with a portal
  credential like any member, once the portal instance exists. ADR-0089 D1 already keeps staff SSO
  *"separate from tenant identities."*
- **Staff reach a tenant or an org only by grant** (ADR-0120). No staff identity holds a standing role
  in any org or tenant.

## Consequences

**Accepted costs**

1. **A new virtual instance and login deployment in every region that is built** — the portal's, added
   with the portal. The staff instance keeps `id.paridyum.com`, so no staff client's issuer moves.
2. **One linked user per person per tenant.** Deprovisioning must follow every link (ADR-0120 D5), and
   a reconciler must prove it did.
3. **Login domains, certificates and DKIM records per tenant, at scale**, with the verification
   experience in the portal (ADR-0121 D4).
4. **Paridyum's own people hold two credentials** — staff, and a portal credential as members of the
   Paridyum org. RFC-0017 cost 2 names this as the permanent price of keeping the populations apart.
5. **Decision 5 is a breaking change to both libraries**, staged with ADR-0113.

**Gained**

- **Every tenant signs in and mails on its own domain, Paridyum's included.**
- **One credential per person across the portal and every tenant they work in**, without merging the
  planes.
- **The escalation in brief F-H is closed before it can be reached.**
- **RFC-0012's cryptographic boundary** between the people who provision tenants and the people
  tenants serve is intact.

**Rejected alternatives**

- **Everything to `.in`** (2026-09-27). Paridyum's tenant would be the one whose customers sign in on
  the platform's domain, and white-labelling would be impossible for every other tenant.
- **One instance for both planes.** It removes the federation step and RFC-0012 D2's separate issuers
  with it.
- **Staff and customers in one directory.** Simpler, and a customer's directory beside the people who
  hold estate powers is the neighbourhood nobody should design on purpose (RFC-0017).
- **One Zitadel organization per org in the portal instance.** Per-org SSO would come free, and
  "Zitadel organization" would mean two things again (ADR-0113 D2).
- **Cross-organization role assignments as the primary mechanism.** They put the F-H fix on every
  request path and need a tenant-selection step in every token.

## Related

- **ADR-0047** — D1, D3, D5 amended; D2, D7 unchanged · **RFC-0012** D2 — the issuers
- **ADR-0089** · **ADR-0090** · **ADR-0091** · **ADR-0085** · **ADR-0067** · **ADR-0088** · **ADR-0070** ·
  **ADR-0082** · **ADR-0084**
- **RFC-0017** · **ADR-0117** (D5–D7) · **ADR-0113** (stage 1 carries decision 5) · **ADR-0118** ·
  **ADR-0120** · **ADR-0121** · **ADR-0122** (login and message texts per content locale)
- Research brief `docs/2026-09-27-rfc-0055-consume-serve-operate-research-brief.md` — F-H
