---
id: RFC-0012
title: Identity is a product built on Zitadel — organization per tenant, proxy without minting
type: rfc
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0047
scope: [org, platform, service, web, mobile, sdk, spec, infra]
applies_to: ["*"]
tags: [identity, zitadel, multi-tenancy, oidc, vendor, tenancy, rfc-0009]
breaking: false
enforced_by: []
---

# RFC-0012 — Identity is a product built on Zitadel

## Status

**ACCEPTED** as of 2026-08-08, resolved by ADR-0047. Opened and closed the same day, so the
feedback deadline below never came into play — recorded because the reasoning for choosing it
still stands: **2026-09-07** — set inside ADR-0011's 30-day
staleness warning rather than beyond it, because RFC-0003 and RFC-0010 both carry deadlines that
fall *after* the warning fires and are therefore flagged stale before anyone is late.

Answers RFC-0009 Q1. Revises RFC-0009 §3's framing of `platform-identity`.

## The question

RFC-0009 treats identity as infrastructure: *"`platform-identity` — Issues a session naming the
scope a principal holds."* That sentence is now wrong in two directions. It does not issue —
Zitadel does. And identity is not infrastructure the platform *has*; it is a product the
platform *sells*, headless and per-tenant branded.

Decomposing a product as though it were a dependency produces the wrong boundaries. This RFC
re-answers it.

## What is already decided elsewhere

Not restated: ADR-0024 (nested tenancy), GLO-0001–0005 (vocabulary), ANTI-0001/0005/0006
(boundaries), ADR-0032 (locale-neutral identifiers), C-0006 (market posture), BET-0003
(generated application layer). Two decisions below depend directly on the last of those.

## Decision 1 — Zitadel is the engine; `platform-identity` proxies it

Self-hosted Zitadel, **one deployment per region**, Postgres 14–18.

`platform-identity` terminates two planes and neither of the other two:

| Plane | Terminated by `platform-identity`? |
| --- | --- |
| **Control** — provisioning, tenant self-service, capability gating | **yes** — this is the product |
| **Login/session** — headless auth API, custom login UI (Session API) | **yes** — this is white-label and headless |
| **Token issuance** | **no** — Zitadel mints |
| **Token validation** | **no** — products validate locally against JWKS |

Zitadel's Session API is explicitly designed for this: it *"can be used instead of OpenID
Connect or SAML to authenticate users by building your own custom login user interface"*, and
Zitadel ships a self-hostable Next.js reference login built on it — the same framework RFC-0008
proposes.

### Why Zitadel keeps minting

A virtual instance carries its own custom domain, so `iss = login.tenantbrand.com`. **White-label
at the issuer is already free**; minting our own tokens buys nothing there and costs signing
keys, rotation, JWKS publication, revocation, discovery and OIDC conformance — the one part of an
identity provider that is genuinely hard and already certified.

The security argument is the stronger one. With Zitadel issuing, tenant identity comes from the
**verified issuer**, bound to the key that signed the token. If `platform-identity` minted, `iss`
would be the same for every tenant and tenant identity would demote to *a claim our own code
sets* — moving the catastrophic, reportable boundary of GLO-0004 from "guaranteed by signature
verification" to "guaranteed by our proxy being correct."

### Why validation never routes through the proxy

A proxy on the validation path is a hop on every authenticated request in every product and a
single point of failure for the platform. Products fetch JWKS and validate locally. **Terminate
the login; never terminate the validation.**

## Decision 2 — organization per tenant, in one shared instance per region

```
EVERY REGION — one Zitadel deployment, one Postgres
│
└── virtual instance: shared              login.paridyum.com
    ├── org: tenant-a                     default domain, own branding + policy
    ├── org: tenant-b  ← login.tenantb.com   premium: an ADDITIONAL custom domain
    ├── org: tenant-c                        on the SAME instance
    └── Projects: THE PRODUCT CATALOGUE — one copy each, granted to tenant orgs

HOME REGION additionally
└── virtual instance: paridyum            id.paridyum.com
    ├── org: paridyum-internal (staff)
    ├── org: operator-<id>
    └── Projects: internal and operator tools
```

**Tenant = organization**, not virtual instance. This is the decision that changed on
2026-08-08, and BET-0004 is why: with a **product catalogue**, one Project must serve many
tenants, and a Project Grant is organization-to-organization **within one instance**. No grant
crosses an instance boundary. Instance-per-tenant would therefore require every product's Project
replicated into every tenant's instance — at 100 tenants and 10 products that is 1,000 Project
definitions, each needing update on every release, with no transaction and no rollback.
Organization-per-tenant makes it **one Project, one update, N grants**.

**A premium tenant keeps its own login domain without its own instance.** Zitadel's System API
adds multiple custom domains to one instance — the docs describe adding domains *"if you want to
access ZITADEL at an Organization Domain"* — and each *"will be used to route requests to this
instance."* Branding follows the domain. So the custom login URL is a paid upgrade that costs a
DNS record and an API call, not a new tenancy model.

**Paridyum plane** — a separate virtual instance in the home region, for staff and operators.
Separate issuer, separate user store. It does not replicate across regions: Zitadel instances do
not replicate, so an instance per region would mean N copies of the staff directory. Staff
authenticate at home; regional administration uses region-local service accounts.

**Paridyum's own instance lives in a home region.** Zitadel instances do not replicate across
deployments, so an instance per region would mean N copies of the staff directory — N joiner
processes, N MFA enrolments, N places to forget to deprovision. Staff authenticate at home;
regional administration uses region-local service accounts with private key JWT.

### Mapping

| Paridyum | Zitadel | Enforced by |
| --- | --- | --- |
| tenant (GLO-0002, outer silo) | **organization** | Zitadel **and** Postgres RLS |
| account (GLO-0003, inner silo) | *not a Zitadel object* | Postgres RLS |
| product | **project**, granted to tenant orgs | audience |
| interface (web/mobile/CLI/API) | **application** | flow |

### The isolation consequence, stated plainly

**Zitadel has exactly one isolation level below the instance.** The vendor's own words: *"ZITADEL
does not enforce a native hierarchy or inheritance for users or organizations, but you can use
organization metadata or your own business logic to map complex hierarchies."* There is no
sub-organization. So a single instance cannot enforce both of GLO-0004's silos, and the option
space is exactly three:

| | tenant ↔ tenant | account ↔ account | Catalogue |
| --- | --- | --- | --- |
| instance = tenant, org = account | Zitadel | Zitadel | ❌ grants cannot cross instances |
| **org = tenant** *(chosen)* | **Zitadel + RLS** | RLS | ✅ native |
| org = account, tenant in metadata | RLS | Zitadel | ✅ native |

The third is strictly worse: it puts Zitadel on the *inner* boundary and leaves the outer one to
us, and GLO-0004 rates tenant↔tenant *"catastrophic, **reportable**"* against account↔account
*"catastrophic for that business, and it is **their** reputation, not ours."* The reportable one
carries Paridyum's own legal exposure, so it is the one that earns the second control.

**The chosen row gives the tenant boundary two independent controls** — a Zitadel organization
*and* ADR-0024's RLS, since those policies constrain `tenant_id` and `account_id` both. The
account boundary has one. That is a real reduction against instance-per-tenant and it is the
price of the catalogue, but "RLS only" would understate it: Postgres RLS with `FORCE` is enforced
by six checks in `check-migrations.sh`, required denial-asserting tests under STD-0009, and
fail-closed accessors under ADR-0038. Zitadel's organization boundary is also software enforcing
a predicate — in code we do not control and cannot test.

### What organization-per-tenant costs

**Metering.** Quotas are per *instance*, so the native per-tenant meter is lost. Metering moves
to `platform-identity`, which is already the proxy on the login plane and sees every
authentication. That covers logins and monthly actives; it does not see per-API-call usage inside
products, which would have to be metered there.

**Noisy-neighbour containment** becomes rate limiting at the proxy rather than a Zitadel quota.

## Decision 3 — the claims contract

| Fact | Source |
| --- | --- |
| **tenant** | `urn:zitadel:iam:user:resourceowner:id` — the **organization** claim |
| account | resolved from **Paridyum's own tables**, keyed on the signed `sub` |
| roles | `urn:zitadel:iam:org:project:{projectId}:roles` |
| audience | `aud` must contain the product's project id, requested via `urn:zitadel:iam:org:project:id:{projectId}:aud` |

Every resource server validates **`iss` and `aud`**, and reads the tenant from the organization
claim rather than from the issuer — with one shared instance per region, the issuer no longer
discriminates between tenants.

**This is still a signed fact, not an asserted one.** Zitadel sets the organization claim from
the user's actual membership. The rule is not "never trust a claim"; it is never trust a claim
*we* wrote. An account, by contrast, is not a Zitadel object at all, so it resolves from
Paridyum's tables — putting `account_id` in Zitadel user metadata was rejected under ANTI-0001 as
a second writer for a fact this organisation owns.

Role and scope values are lowercase ASCII identifiers under ADR-0032 —
`billing.admin`, not `Billing Admin`. Custom claims, if ever used, take a `paridyum:` namespace;
Zitadel's `setClaim` ignores keys prefixed `urn:zitadel:iam`.

ADR-0038 converts these claims into the Postgres session context.

## Decision 4 — federation at both silos

IdPs attach **per instance** (tenant-wide) or **per organization** (one account) — OIDC, SAML,
LDAP, social and enterprise. Both of GLO-0004's silos get native federation with no design work.

## Decision 5 — the capability matrix is data

| Capability | Zitadel level | Controlled by | In product |
| --- | --- | --- | --- |
| Custom login domain | instance | tenant | ✅ |
| Branding / theme | instance + org | tenant, per-account override | ✅ |
| Password, lockout, MFA, passkeys | instance | tenant | ✅ |
| External IdP federation | instance / org | tenant / account | ✅ |
| Users, credentials | org | tenant | ✅ |
| Organizations | instance | tenant | ✅ |
| Applications | project | tenant | ✅ |
| Projects & roles | org | product owner | ✅ |
| Service accounts, PATs | org | tenant | ✅ |
| Domain verification | instance | tenant | ✅ |
| Audit / event log | instance | tenant, read-only | ✅ |
| **Actions (custom code)** | instance | **Paridyum only** | ❌ |
| **Quotas & limits** | instance | **Paridyum only** | ❌ |
| **System API** | deployment | **Paridyum only** | ❌ never |
| **Masterkey** | deployment | Paridyum only | ❌ |

**`platform-identity` authorizes and forwards to Zitadel's Management API rather than
re-implementing it**, gated by this matrix. That is what makes a new Zitadel feature available
the day it ships, and it is why the matrix is a data file — the sixth instance of the pattern
ADR-0014 started.

Three are never delegated. The **System API creates and destroys instances**, so a tenant
holding it can destroy other tenants. **Actions run custom code inside the token flow**, in a
process shared by every tenant in the region, and spend that region's action-seconds. **Quotas
are the meter**; a tenant that can raise its own limits cannot be billed.

## Decision 6 — per-interface flows

Web and mobile: Authorization Code + PKCE. **CLI and devices: Device Authorization Grant, RFC
8628**, application type Native. APIs: local JWT validation. Service-to-service: **private key
JWT** preferred over client credentials, one service account per service, audience-scoped.

## Decision 7 — authorization federates

Identity answers *who, in which tenant, holding which granted roles*. **Each product answers
whether that principal may perform an action.** Roles are defined in the product's Project;
`platform-identity` carries grants and never adjudicates them.

Centralising per-product permissions is how this becomes ANTI-0006 — the artifact that must
change whenever any product adds a feature. ABAC is worse: "may this user edit *this* invoice"
depends on data identity must never hold.

## Consequences

**Accepted costs**

1. **Every tenant's identity data shares one Postgres per region, and its isolation is
   Zitadel's, not ours.** Our own tables carry `ENABLE` + `FORCE` RLS, two-column tenancy and
   per-level policies, checked by `check-migrations.sh`. **None of that discipline can be applied
   to Zitadel's schema.** Application data is more strictly isolated than identity data. That is
   a defensible trade and it must not be made silently — REQ-0015 owns modelling it.

2. **Restore granularity is unproven.** Restoring one tenant's identity data without restoring
   the region is unestablished, and REQ-0013 depends on it.

3. **Custom-domain provisioning brings DNS and ACME onto the critical path** — the slowest and
   least reliable step in ADR-0039's saga, and the likeliest source of `compensation_failed`.

4. **`platform-identity` availability is the login availability of every product.** It is off the
   validation path, so an outage stops new logins rather than all traffic — a materially smaller
   blast radius, and still the highest-availability service we will operate.

5. **Vendor concentration.** Zitadel becomes as load-bearing as Postgres. REQ-0012 is `accepted`
   and owns it.

6. **The masterkey cannot be changed once initialised.** A one-way door, per region.

**Gained**

- A new product is a Project, roles, applications and a grant. No identity work — the test of
  whether this is a platform.
- White-label, headless, federation and metering are vendor capabilities rather than roadmap.
- Tenant identity is cryptographically bound rather than asserted.

## Alternatives considered

- **Build identity in-house.** Reimplements OIDC, SAML, passkeys, federation and conformance. The
  fastest way to spend a year not shipping the product.
- **Zitadel Cloud instead of self-hosted.** Loses control of regional placement and residency,
  which is the stated constraint.
- **Instance per tenant *and* per product.** Multiplies deployments by the product catalogue.
- **`platform-identity` mints tokens.** Decision 1.
- **Organization per tenant.** Decision 2. Loses domains, metering and noisy-neighbour
  protection.
- **One Zitadel globally.** Contradicts the regional constraint and residency.

## Open questions

1. *(Resolved 2026-08-08 — both confirmed, with one caveat.)* **Zitadel is an outbound SAML
   IdP**: it publishes SAML endpoints and is documented being configured as the SAML *provider*
   to Entra ID acting as service provider. **SCIM v2.0 is supported** as a service-provider
   interface, authenticated by a service account — and its endpoint is
   `https://${CUSTOM_DOMAIN}/scim/v2/{orgId}`, **scoped per organization**, which fits
   organization-per-tenant exactly: a tenant's own Okta or Entra provisions users directly into
   that tenant's org, with no per-tenant integration work from us.
   **Caveat: SCIM is marked Preview.** It may be listed in the capability matrix; it must not be
   contracted as stable until it is GA.
2. **Zitadel System API idempotency.** ADR-0039 assumes provisioning steps can be safely retried
   on a durable key. Unverified; if absent, step 2 becomes query-before-create, which is a race.
3. *(Resolved — instance density was the falsification condition while tenants were instances.
   With organization-per-tenant there is one instance per region, and the question is moot.)*
3. **Notification providers** — Paridyum default, or tenant override. Unresolved.
4. **Splitting control plane from login plane.** One repository now (ADR-0001). Their availability
   profiles differ — login must be near-perfect, provisioning may be down for an hour — which is
   the observable trigger for revisiting.

## Reversal conditions

- **A tenant requires an isolation boundary an organization cannot give** — a regulated customer
  whose auditor rejects logical separation inside a shared instance, or a contract demanding a
  dedicated user store. That tenant needs its own virtual instance, and the catalogue would have
  to be replicated into it. Under ADR-0041 this arrives with phase 2 compliance, so it is
  foreseeable rather than sudden.
- **The catalogue stops being the product.** BET-0004 supersedes BET-0003 and is itself
  falsifiable. If generation returns as the product, tenants stop sharing Projects, and
  instance-per-tenant becomes viable again — the topology reverts rather than breaks.
- **Branding stops following the domain.** The premium tier assumes a tenant's custom domain
  resolves to that tenant's organization branding without the client passing a scope. Confirmed
  by the operator, not by us. If it proves false, a custom domain delivers a URL and not a
  branded page, which is not what is being sold.

## Related

- RFC-0009 — Q1 answered here; §3's identity responsibility superseded by decision 1
- ADR-0038 — claims to session context
- ADR-0039 — the provisioning saga that creates these resources
- ADR-0024, GLO-0001–0005 — the tenancy model this maps onto
- ANTI-0001, ANTI-0003, ANTI-0006 — ownership, no foreign database, no god service
- BET-0002 — metering; BET-0003 — the generated layer decision 2 depends on
- REQ-0001 — runtime secrets: masterkey, DB credentials, service-account keys
- REQ-0012 — vendor governance; REQ-0015 — the threat model cost 1 requires
- RFC-0008 — Next.js, which is also Zitadel's reference login stack
