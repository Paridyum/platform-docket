---
id: ADR-0120
title: Reaching into a tenant or an org is a grant it can see and revoke — for owners, providers and Paridyum alike
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web]
applies_to: ["*"]
product: [tenancy, identity, paridyum]
tags: [grants, access, rung-3, provider, support-access, operator-grants, audit, account-members, per-instance, delegation, org, adr-0104, adr-0114, adr-0116, adr-0117, adr-0119, glo-0005, rfc-0022, rfc-0055, c-0008]
breaking: false
enforced_by:
  - check: test.org_membership_confers_no_business_access
    severity: error
    rationale: >
      Decision 1's negative case, as a test: a member of the owning org holding no grant in the tenant
      is refused inside it — zero rows read, every write refused. GLO-0005's absence-as-grant hazard one
      level up; the easiest shortcut in the model is "owns the tenant" read as "may act in it".
  - check: db.operator_scope_is_an_explicit_grant
    severity: error
    rationale: >
      Existing (ADR-0115 D4). A per-tenant grant arrives in the tenant as an explicit role on a verified
      token, never as an inferred breadth.
verified_by:
  - TST-0018
  - TST-0019
  - TST-0021
sources:
  - url: https://learn.microsoft.com/en-us/partner-center/customers/gdap-introduction
    title: "Granular delegated admin privileges (GDAP) introduction — Partner Center, Microsoft Learn"
    retrieved: 2026-09-27
  - url: https://learn.microsoft.com/en-us/partner-center/customers/gdap-customer-terminate
    title: "Customer-Led Removal of a Granular Admin Relationship — Partner Center, Microsoft Learn"
    retrieved: 2026-09-27
  - url: https://developers.google.com/android-publisher/getting_started
    title: "Getting Started — Google Play Developer API"
    retrieved: 2026-09-28
---

# ADR-0120 — Reaching into a tenant or an org is a grant it can see and revoke

## Status

**ACCEPTED** 2026-09-28 by the operator. It resolves:

- **ADR-0104 accepted cost 4** — per-instance operator grants;
- **GLO-0005's deferral** — *"support access is a different mechanism and a different decision
  (deferred)"*;
- **the serve rung** the 2026-09-27 research brief found missing from RFC-0055 (F-E, F-I).

## Context

**ADR-0114 D6:** org membership authorises the portal, never a tenant. So something else must carry a
person into a tenant. **Three populations need to:**

- **(a) the owning org's members** — including the person who just created the tenant;
- **(b) a provider serving a client that is its own tenant** — rung 3. A client company is never an
  account in the provider's tenant (brief F-A to F-C);
- **(c) Paridyum's support.**

**Two constraints are on record.** C-0008 forbids population (c) a path (a) and (b) lack — *"Special-
casing Paridyum's tenant anywhere — provisioning, entitlement, quota, billing or support — beyond what
is published and available to any tenant on the same terms."* RFC-0022 D5, accepted 2026-09-27,
promises that when a Paridyum operator accesses a tenant's data *"that tenant can see it."*

**The storage half exists.** `operator_grants` (migration `0010`): subject, scope, grantor, expiry and
revocation — readable by the tenant it is over, with immutable provenance and a write-once revocation.
`audit_events` (`0010`) records the access a grant produced.

**Zitadel's own writes are outside Postgres auditing** — pgaudit was scoped away from Zitadel's polling
(`infra-kubernetes#8`), which is RFC-0022 §D5a path 8. **An access decision that existed only as a
Zitadel role grant would therefore have no audited record.**

**Outside practice.** Microsoft GDAP: *"Customers must explicitly grant the least-privileged access to
their partners"*, access is *"granular and time-bound"*, and a customer can *"Remove a partner's GDAP
from their tenant"*, after which the partner's users *"no longer have access to administer services."*

## Decision

**1. Every human reach into a tenant is a per-tenant grant.** A grant names:

- the tenant;
- the grantee — a subject, and the instance it comes from (portal or staff);
- the role, from the tenant's own vocabulary — `operator` today, or an instance-scoped role (decision
  6);
- who granted it and when;
- an optional expiry — a standing grant is permitted and visible;
- its revocation.

**Nothing else confers access** — not org membership, not ownership, and not being Paridyum.

**2. Who may grant.**

- **Owners.** The owning org's `owner` and `admin` members grant within that org's tenants. The saga
  issues the creating member's first grant automatically at provisioning (ADR-0118 D5).
- **Providers — rung 3.** A provider's members request access from the portal; the tenant's admins
  approve, choosing role and expiry. This is the GDAP shape: requested, consented, scoped, time-bound.
- **Paridyum support asks exactly the same way.** There is no other path (C-0008).

**3. Our record is authoritative, and identity enforces it.**

- **The grant row in `operator_grants`** — extended with the role, the grantee's instance and the
  optional account — is the authoritative, audited record.
- **The portal applies it** as a role on the grantee's linked user in the tenant's Zitadel organization
  (ADR-0119 D4).
- **A reconciler compares the two and alerts on drift in either direction**: a grant with no role, or a
  role with no grant. The second is exactly what an unaudited Zitadel write looks like.

**4. The tenant sees every grant and every use.**

- Its console lists live and ended grants; `operator_grants` is already tenant-readable.
- It lists the access those grants produced, from `audit_events` (RFC-0022 D5).
- **Revocation belongs to the tenant**: one action, effective at the grantee's next token refresh, and
  the grantee is told.

**5. Membership changes fan out.** When a member leaves an org, or an org loses ownership of a tenant,
every grant that membership produced is revoked, and the reconciler proves it — a leaver's standing
access is the failure an access review exists to find.

**6. Per-instance grants — ADR-0104 cost 4.**

- A grant's role may be scoped to one account — one instance — of the tenant.
- `account_members` is the in-tenant binding for that: tenant-scoped, holding the account, the
  subject and the role.
- For non-operator sessions, account-scoped tables gain the account predicate that binding feeds.
- **The denial is the assertion** (STD-0009): an instance-scoped grantee cannot read a sibling instance.

**7. Pointed outward, the same shape.** When a tenant grants Paridyum access to one of its own external
accounts — a Google Play developer account (*"Users & Permissions … Invite new users"*), App Store
Connect, a payment provider — the record, visibility and revocation rules above apply. The store
accounts are RFC-0058's; this decision fixes only that they are grants, never credentials handed over.

**8. A grant across regions is the same grant.** The grantee's credential may be in any region. The
grant, its linked user and its audit all live **in the tenant's region** (ADR-0117 D7):

- **The linked user is the tenant's own record of who it let in** — minimal, consented at the grant.
- **The grantee's region's portal instance is linked** as an identity provider of the tenant's
  organization when the first grant to someone from that region is approved (ADR-0119 D4), and the link
  goes when no grantee from that region remains.
- **Nothing about the grant is written to the grantee's region.** Their portal lists "tenants I can
  reach" from the control plane, plus a read of the tenant's region, under their own session, when they
  open one.

**9. An org is reached the same way.**

- **Support inside an org** — reading its members or invitations to answer a question — is an
  org-scoped grant that the org's `owner` approves, sees and revokes, with its own audit, in the org's
  region.
- **Until the first support case needs it, it is not built, and no staff path reads org-scoped data at
  all.** The platform context cannot (ADR-0116 D1), so the absence fails closed.

## Consequences

**Accepted costs**

1. **Every access path into a tenant has two moving parts** — our record and Zitadel's role — plus a
   reconciler to keep them honest. That is the price of path 8.
2. **A grant interface on both sides:** the portal for grantees and requesters, the tenant's console
   for grantors and revocation.
3. **An owner reaches their own tenant through the same machinery as a stranger.** The saga's automatic
   first grant hides the step, and a bug in it locks an owner out of what they own.
4. **Support is slower.** Paridyum asks and waits like anyone else, including during an incident — the
   moment C-0008's second accepted cost names as the one where the exception is most tempting.

**Gained**

- **Owner, provider and support access are one mechanism, built once.** The rung-3 flow, GLO-0005's
  deferred support access and ADR-0104's per-instance question are answered together.
- **C-0008's no-special-path rule and RFC-0022 D5's tenant-visible access hold by construction.**
- **An access decision never exists only in an unaudited write.**

**Rejected alternatives**

- **Ownership or org membership implies tenant access.** The absence-as-grant GLO-0005 forbids, one
  level up; it would make every member of a large team an operator of every tenant the team owns.
- **A Paridyum support role present in every tenant.** The privileged path C-0008 forbids, and the one
  RFC-0022's whole defence rests on not having.
- **Zitadel as the only record.** Outside pgaudit (path 8), not tenant-readable, and a grant nobody can
  list is a grant no access review can find.

## Related

- **ADR-0114** D6 · **ADR-0116** D1 · **ADR-0117** D7–D9 · **ADR-0118** D5 · **ADR-0119** D4, D5
- **GLO-0005** (the deferral this resolves) · **ADR-0104** cost 4 · **ADR-0115** · **STD-0009**
- **RFC-0022** D5 and §D5a path 8 · **C-0008** · **RFC-0055** (the serve rung) · **RFC-0058** (stores)
- `platform-tenancy/migrations/0010` — `operator_grants`, `audit_events`
