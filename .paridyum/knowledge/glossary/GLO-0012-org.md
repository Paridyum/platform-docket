---
id: GLO-0012
title: Org — the customer of paridyum.in, a person or a team, which owns tenants and holds none of their data
type: glossary
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [tenancy, vocabulary, org, ownership, membership, portal, paridyum-in, adr-0113, adr-0114, adr-0115, glo-0002, glo-0003, glo-0005]
breaking: false
enforced_by:
  - check: repo.retired_org_identifiers_absent
    severity: warn
    rationale: >
      This term is safe in code only once the older use of a bare "org" for a tenant's Zitadel
      organization is gone. A retired identifier that returns — `require_organization`,
      `orgForRequest` — is the old meaning coming back through code, where no glossary is read.
      `warn` while ADR-0113's staged rename runs; `error` from its stage 3.
---

# GLO-0012 — Org

## Definition

**An org is the customer of paridyum.in — one person or a team — and it owns zero or more
tenants.** It is what a person joins when they sign up, what Paridyum bills, and what `max_tenants`
is counted against.

- **Column: `org_id`.** Org-scoped tables carry it, and row-level security confines them to
  `paridyum.current_org()` (ADR-0115, STD-0023).
- **An org owns tenants; it does not contain them.** The tenant registry records each tenant's
  owner as `tenants.owner_org_id`. Nothing in a tenant's own rows names its org (ADR-0114 D4).
- **Personal and team orgs are one model.** A personal org has one member and a team org has many.
  Turning one into the other is adding members, not migrating anything.
- **A person may belong to several orgs**, and a session acts in one at a time.
- **An org has a region** — where its own people's data lives — chosen explicitly at signup and
  never inferred (ADR-0117). It is never a "home region" (GLO-0007). The tenants it owns may be in
  any region.

## The hierarchy, once

```
Paridyum — the platform: our software and our estate
  └── org          the customer of paridyum.in                  org_id               GLO-0012
        │  owns (tenants.owner_org_id — a key on the tenant registry, not a column on tenant data)
        └── tenant   a business — the outer silo                 tenant_id            GLO-0002
              └── account   a silo inside a tenant               tenant_id + account_id  GLO-0003
```

**The org sits above the silos as their owner. It is not a third silo.** Business data carries
exactly two discriminators, `tenant_id` and `account_id`, and never `org_id` (ADR-0115 D1, D9).

## What it is not

- **Not a tenant.** A tenant is a business: its data, its domain, its identity, its branding
  (GLO-0002). An org owns tenants and holds none of that. "Platform tenant" is not a term — the
  portal is the platform's own surface and runs in no tenant (ADR-0118).
- **Not an account.** An account is a silo *inside* a tenant (GLO-0003). An org is never stored as
  one.
- **Not a Zitadel organization.** In the tenant instance, one Zitadel organization mirrors one
  tenant (ADR-0047 D2, unchanged). No Zitadel organization mirrors an org. In code the vendor object
  is `zitadel_org` / `resource_owner`, never a bare `org` (ADR-0113 D2).
- **Not operator scope, and not an operator.** *Operator* means a principal with breadth across the
  accounts of **one tenant** (GLO-0001, GLO-0005, ADR-0109 D1 as carried by ADR-0114). Being a member
  of an org confers **nothing inside any tenant** — only a per-tenant grant does (ADR-0120).
- **Not the org in `scope: [org]` or in the `org.*` checks.** In artifact frontmatter, `scope: [org]`
  keeps ADR-0015's meaning — *binds every repository* — and `org.*` check ids keep meaning the GitHub
  organisation (ADR-0004). Both are fenced by ADR-0113 D3. This entry's own `scope: [org]` is the
  first example: it means the term binds everywhere, not that it is about orgs.

## Words beside it

| Word | Means | Never |
| --- | --- | --- |
| **member** | a person in an org, with an org role: `owner`, `admin`, `billing` or `member` | an operator, a tenant user |
| **staff** | Paridyum's own people, in the staff instance | an org's members |
| **the portal** | paridyum.in, where orgs sign in and manage their tenants | a tenant, a tenant's console |
| **the console** | a tenant's own administration surface, on that tenant's host | the portal |
| **the Paridyum org** | the org Paridyum itself holds; it owns Paridyum's tenant (tenant zero, C-0008) | the platform |

## Why this word

It is the operator's word (2026-09-28), and it is how people describe what they sign up as — *"join
as an individual or as an org / team"*. Taking it for this level cost a rename, because the code had
borrowed Zitadel's "organization" for tenants. ADR-0109's rule chose where the qualifier goes: **the
reading already written into something immutable wins.** The vendor object is immutably Zitadel's,
so it keeps the vendor's name, and the new level takes the plain word.

## Related

- **ADR-0114** — the org as an entity; **ADR-0113** — the vocabulary and the rename that frees the
  word; **ADR-0115** — the `org` scope; **ADR-0117** — the org across regions
- **GLO-0002** tenant · **GLO-0003** account · **GLO-0004** nested tenancy · **GLO-0005** operator
  scope · **GLO-0007** control region
- **ADR-0047** D2 — one Zitadel organization per tenant, which this term does not change
- **ADR-0109** — superseded by ADR-0114; its method for correcting a word is reused
