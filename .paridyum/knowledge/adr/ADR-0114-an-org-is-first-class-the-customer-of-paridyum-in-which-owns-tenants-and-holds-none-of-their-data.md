---
id: ADR-0114
title: An org is first-class — the customer of paridyum.in, which owns tenants and holds none of their data
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: [ADR-0109]
superseded_by: null
scope: [org]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [tenancy, org, ownership, membership, entitlement, max-tenants, billing, portal, adr-0024, adr-0038, adr-0049, adr-0101, adr-0104, adr-0109, adr-0113, adr-0115, adr-0116, glo-0001, glo-0005, glo-0012, rfc-0014, rfc-0017, rfc-0022, rfc-0047]
breaking: true
enforced_by:
  - check: db.no_org_discriminator_on_business_data
    severity: error
    rationale: >
      Decision 4 made mechanical, and the check that keeps ADR-0115 D9's signed reading true: the
      org owns a tenant through one key on the tenant registry, and a column naming an org on any
      tenant- or account-scoped table would be the third silo D9 defers. Decidable by reading a
      migration.
  - check: test.org_membership_confers_no_business_access
    severity: error
    rationale: >
      Decision 6's negative case. "Owns the tenant" read as "may act in the tenant" is the easiest
      shortcut in the model and it fails open — every request succeeds. The denial is the assertion
      (STD-0009).
  - check: domain.operator_tenant_holds_no_extra_grant
    severity: error
    rationale: >
      Existing (ADR-0049 D2, carried by ADR-0116). The Paridyum org is an org like any other and its
      members hold no scope another org's members could not.
verified_by:
  - TST-0016
  - TST-0017
  - TST-0019
sources:
  - url: https://docs.aws.amazon.com/wellarchitected/latest/security-pillar/aws-account-management-and-separation.html
    title: "AWS account management and separation — AWS Well-Architected Security Pillar"
    retrieved: 2026-09-28
  - url: https://vercel.com/docs/accounts
    title: "Account Management — Vercel Docs"
    retrieved: 2026-09-28
  - url: https://quickbooks.intuit.com/learn-support/en-us/help-article/manage-client/manage-wholesale-discount-plan-wholesale-discount/L13MoEwj6_US_en_US
    title: "Manage your ProAdvisor Preferred Pricing plan and ProAdvisor discount clients — Intuit"
    retrieved: 2026-09-27
---

# ADR-0114 — An org is first-class: the customer of paridyum.in, which owns tenants and holds none of their data

## Status

**ACCEPTED** 2026-09-28 by the operator, who chose a **first-class org** — its own scope, its own
tables, and no tenant that exists only to run the portal — over storing orgs as accounts of a tenant.

- **Supersedes ADR-0109.** Its decisions 1, 2, 4 and 5 are carried forward (decision 9 below). Its
  decision 3 — *"the commercial party above a tenant is not modelled"* — is replaced.
- **ADR-0101** carries a dated note: its three-level chain with the operator on top, its `Operator` entity and
  *"three per operator"* now read *org → tenant → account*, the org, and *three per org*.
- **The ADR-0024 D9 reading is the operator's, signed 2026-09-28**, and is recorded where D9 now
  lives — ADR-0115 D9.

## Context

**The operator, 2026-09-28:**

> *"Our main portal is the business-as-a-service at paridyum.in … people join as individuals or as
> an org / team, then deploy their first main business … we have a tenant layer that's very strong …
> my idea is introducing org on top of this."*

**The party an org names was already load-bearing and unmodelled.**

- PLAN-TENANT-FLOW D5 (self-serve signup) and D6 (`max_tenants`, default 3) need a principal that
  holds several tenants.
- ADR-0101 caps tenants *"at three per operator"* and gives `platform-tenancy` an `Operator` entity.
- RFC-0014 D2 already puts *"which operator owns which tenant, in which region, on which plan"* in the
  control plane, and RFC-0017 keeps that skeleton personless. In both, "operator" is this party.
- ADR-0109 D3 declined to model it.
- **The code has only a staff reading.** `platform-tenancy`'s `GET /v1/tenants` returns every tenancy
  to any control-plane token (`src/handlers.rs`, `require_control`) — *"Paridyum's own staff"*.

**The first draft of this decision stored an org as an account of a tenant that ran the portal.** It reused
every existing mechanism and it mixed the words the operator had just asked to keep apart: an org
would have been an *account* inside a *tenant*, and the portal would have been a tenant that was not
a business. The operator rejected it for a first-class org.

**Outside practice.** AWS: *"In AWS, accounts are a hard boundary"*, with Organizations above them to
*"manage accounts centrally"*. Vercel: every workspace is a team — an individual has a *"Hobby
team"*, and a person switches between *"the list of teams you belong to"*. In both, the unit a person
joins is a management and billing layer, and the isolation boundary stays below it.

## Decision

**1. An org is its own scope, with a personless registry row and its own regional rows.**

| Fact | Where | Scope · plane |
| --- | --- | --- |
| id, region, status, plan, created | `orgs` — the org registry | `platform` · `control` — no name, no person (ADR-0117 D3) |
| name, handle, locale defaults, contacts | `org_profiles` | `org` · `regional`, in the org's region |
| members and their roles | `org_members` | `org` · `regional` |
| invitations | `org_invitations` | `org` · `regional` |

- **Org-scoped rows carry `org_id`**, and row-level security confines them to
  `paridyum.current_org()` — an accessor that raises when unset, exactly as ADR-0038's do (ADR-0115,
  STD-0023).
- **The org's id is one uuid**: the registry row's id and every org-scoped row's `org_id`.
- **An org is not a tenant, not an account and not a Zitadel organization** (ADR-0113 D1).

**2. Personal and team orgs are one model.**

- **A person's first sign-in at paridyum.in, once their address is proven (RFC-0056 §B), mints a
  personal org** with that person as its `owner`.
- **A team org is created explicitly** and has members.
- **Turning a personal org into a team is adding members.**

**3. Membership is `org_members`.**

- Fields: org, subject, role, state, who granted it and when, and a write-once revocation — the
  `operator_grants` pattern (migration `0010`).
- **Org roles are a closed vocabulary:** `owner`, `admin`, `billing`, `member`.
- **A person may belong to several orgs.** A session acts in one org at a time; switching org starts
  a new org context.
- **A member whose credential is in another region** is held by the org as its own record of them
  (ADR-0117 D7). Their credential never moves.

**4. An org owns tenants; it does not contain them.**

- **Ownership is `tenants.owner_org_id`**: `NOT NULL`, referencing `orgs`, on the platform-scoped,
  control-plane tenant registry. A tenant has exactly one owning org, always.
- **Nothing in a tenant's own rows names its org.** No tenant- or account-scoped table carries `org_id`
  or any column naming an org (`db.no_org_discriminator_on_business_data`).
- **Transferring a tenant changes that one key**, and no tenant data moves — not even across regions,
  because a tenant's region never changes. QuickBooks describes the same shape for moving a client off
  a firm's plan: *"It doesn't impact their accounting data in any way."*
- **An org cannot be deleted while it owns tenants.** They are transferred or retired first.

**5. Entitlement, billing and quota attach to the org.**

- **`max_tenants`** (default 3, PLAN-TENANT-FLOW D6) is counted per org, in the control plane, from
  the tenant registry — across every region at once. It is a gated action (RFC-0015 D1), and it is
  enforced by the provisioning saga, never by the portal — ADR-0101 D3's rule, *"the refusal belongs
  to the saga, not to the surface"*.
- **Paridyum's invoice goes to an org.** ADR-0042 D6's *"Paridyum's invoice to an operator"* names
  this party. `platform-billing` stays deferred on its own trigger.

**6. Membership authorises the portal, and never a tenant.**

- An org role lets a member ask the portal to act for the org. It confers **nothing inside any
  tenant** — not to an `owner`, not to the Paridyum org.
- Reaching into a tenant is a per-tenant grant (ADR-0120), which the saga issues to the creating owner
  at provisioning.
- **The portal lists an org's tenants to its members** from the tenant registry — ids, slugs,
  regions, states — filtered by the org the verified session resolved, never by a request parameter
  (ADR-0115 D5, applied to the org).

**7. Staff see the registry, and nothing org-scoped without a grant.**

- **Paridyum's staff operate the platform in the platform context** (ADR-0116 D1): the org registry,
  the tenant registry and the rest of the personless skeleton.
- **An org's own rows are closed to them.** `paridyum.current_org()` is unset in the platform context,
  so every org-scoped predicate raises. Support inside an org is a grant the org approves, sees and
  revokes (ADR-0120 D9).
- **This replaces "a control-plane token is Paridyum's staff"**, for which `GET /v1/tenants` returns
  every tenancy — names included — to any staff token today.

**8. The Paridyum org is an org like any other.** It owns Paridyum's tenant (tenant zero, C-0008) and
holds no grant, no quota and no path another org could not hold (ADR-0116 D2).

**9. Carried forward from ADR-0109.**

- **D1** — *operator* means a principal with breadth within one tenant. It never names the org.
- **D2** — GLO-0001 is corrected, not superseded.
- **D4** — `operator_id` is not claimed. The org's column is `org_id`, on the org's own rows only.
- **D5** — the corrections ADR-0109 listed stand.

**D3 is replaced**: the commercial party above a tenant is modelled, and it is the org.

## Consequences

**Accepted costs**

1. **A new scope category, and a new accessor.** Every migration author must now choose among four
   categories rather than three, and `check-migrations.sh` learns the fourth (STD-0023). The
   supersessions this forces — ADR-0024, STD-0006, ADR-0049 — are the full price of a first-class
   org, paid once.
2. **The registry half of an org is isolated by code, not by row-level security.** `orgs` and
   `tenants` are platform-scoped, as `tenants` always was, so "an org sees only the tenants it owns"
   is the portal's filter on the verified session. What a defect there would expose is which org owns
   which tenants — sensitive, and personless by ADR-0117 D3.
3. **Deprovisioning fans out.** A member leaving an org must lose every per-tenant grant they hold
   through it (ADR-0120 D5), and a reconciler must prove that they did.
4. **Per-org single sign-on waits.** All orgs of a region share the portal instance's directory
   (ADR-0119 D1), so an org cannot bring its own identity provider at organization level yet. C-0006
   anti-goal 5 defers enterprise anyway.
5. **ADR-0109 is superseded two days after its acceptance**, and ADR-0101 and PLAN-TENANT-FLOW carry
   dated notes.

**Gained**

- **The vocabulary holds in the schema.** An org is `org_id`, a tenant is `tenant_id`, an account is
  `tenant_id` + `account_id`, and no row is two of them.
- **Self-serve gets its missing pieces:** an owner, a key for `max_tenants`, a payer, and a "my
  tenants" list that is not a list of every tenant.
- **An org's people are isolated from other orgs by row-level security**, not by a service's `if`.
- **The landlord and the competing business are separated structurally.** Staff hold nothing inside
  any org or tenant without a visible grant (decision 7, ADR-0120), which is RFC-0022's information
  separation as a property rather than a promise.

**Rejected alternatives**

- **An org as an account of a tenant that runs the portal** (the first draft). It reused every mechanism, and it
  made an org an account inside a tenant and the portal a tenant that is not a business — the
  mixing ADR-0113 exists to end. The operator rejected it.
- **An org as a Zitadel organization.** Per-org SSO would come free, and "Zitadel organization" would
  mean two things again — undoing ADR-0113 D2 and ADR-0047 D2's one-to-one mapping.
- **An `org_id` column on tenant data.** It is the third silo ADR-0115 D9 defers, and D9's own warning
  applies: *"Two explicit columns can be migrated to a path; a path cannot easily be migrated back."*
- **An org wholly in the control plane, people included.** Members' names and addresses in the
  control region are the hole RFC-0017 exists to close.
- **Keep ADR-0109 D3.** It leaves self-serve, `max_tenants`, billing and the portal without an owner,
  and `/v1/tenants` as the only list there is.

## Related

- **GLO-0012** · **ADR-0113** · **ADR-0115** (the `org` scope; D9) · **ADR-0116** (the portal resolves
  an org) · **ADR-0117** (regions) · **ADR-0118** (the portal and the Paridyum org) · **ADR-0120**
  (grants)
- **ADR-0109** — superseded · **ADR-0101** — noted · **ADR-0104** — an account is an instance
- **RFC-0014** D2 · **RFC-0015** D1 · **RFC-0017** — the skeleton and its populations
- **ADR-0042** D6 · **PLAN-TENANT-FLOW** D5, D6 · **RFC-0047** (renting the platform stays gated)
- **GLO-0001**, **GLO-0005** — operator, corrected and relied on
