---
id: ADR-0121
title: A tenant is served on one canonical host — its own domain, or a name under an app zone that is a public suffix
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra, web]
applies_to: ["*"]
product: [routing, paridyum]
tags: [routing, domains, canonical-host, public-suffix-list, app-zone, custom-domain, awaiting-domain, certificates, paridyum-in, paridyum-com, portal, rfc-0016, rfc-0015, adr-0061, adr-0039, adr-0117, adr-0118, rfc-0044, plan-tenant-flow]
breaking: true
enforced_by:
  - check: edge.tenant_has_exactly_one_canonical_host
    severity: error
    rationale: >
      The routing table already states the rule in prose — "EXACTLY ONE HOSTNAME PER TENANT IS
      CANONICAL" — and a second origin breaks sessions, CORS, caching and ranking at once, three of
      them silently. Once hosts are generated per tenant rather than written by hand, the rule needs a
      check at the generator, not a comment.
verified_by: []
sources:
  - url: https://devcenter.heroku.com/articles/cookies-and-herokuapp-com
    title: "Cookies and the Public Suffix List — Heroku Dev Center"
    retrieved: 2026-09-28
---

# ADR-0121 — A tenant is served on one canonical host: its own domain, or a name under an app zone that is a public suffix

## Status

**ACCEPTED** 2026-09-28 by the operator, who chose a **separate app zone on the Public Suffix List**,
with a tenant's own domain becoming canonical once added. This is the routing ADR RFC-0044 names as
owed: *"A custom-domain routing ADR — NOT YET WRITTEN."*

- It replaces PLAN-TENANT-FLOW D8 (`<tenant>.paridyum.com`) and retires RFC-0044 Stage B's
  `<tenant>.paridyum.in`.
- It reverses the 2026-09-27 direction "every tenant brings its own domain".
- **The app zone's name and registration are the operator's.** Until it is bought, no tenant other than
  Paridyum's is served.

`verified_by` is empty by ADR-0030 D6's allowance. The property that matters — tenants cannot set
cookies for one another or for the portal — is verified by a browser check in the change that first
serves a second tenant.

## Context

**`infra-edge/dns/routing-table.yaml` already reasons toward this**, in its "not yet decided" block:

> *"Tenant-serving hostnames need a zone that is not paridyum.com, so that the marketing zone and the
> tenant-serving zone can move provider independently and a mistake in one cannot take the other. The
> intended shape is `<tenant>.<app domain>`; the domain is not registered."*

It prefers `.app` because it *"is HSTS-preloaded, so browsers refuse plaintext to it without any
header from us"*, and fixes two rules kept here: *"A top-level domain must never be load-bearing for
routing"*, and *"Exactly one hostname per tenant is canonical. Every other redirects to it,
permanently, and none of them is a second origin."*

**The table's zone roles no longer match the model.** It lists `paridyum.com` as `marketing` —
*"not tenant-serving; carries no tenant data"* — and `paridyum.in` as `tenant-canonical`, planned.
Under ADR-0118, paridyum.in is the portal and paridyum.com is a tenant's own domain.

**Each prior answer fails the new model.** PLAN-TENANT-FLOW D8 mints `<tenant>.paridyum.com`, placing
every tenant inside Paridyum's own tenant's zone. "Own domain only" stops an individual with no domain
from ever going live (RFC-0056 U1).

**Why the zone must be a public suffix.** Heroku lists `herokuapp.com` so that *"applications in the
herokuapp.com domain are prevented from setting cookies for `*.herokuapp.com`"*, judging that *"the
benefit of registering ourselves with the Public Suffix List outweighed the loss of what is a dangerous
and minimally useful bit of functionality — being able to share cookies across multiple apps."*

## Decision

**1. Three kinds of zone, three roles.**

| Zone | Role | Notes |
| --- | --- | --- |
| `paridyum.in` | **the portal** — the platform's own surface (ADR-0118) | Never a tenant host. One host; data served by the session's region; a sign-in host per region (decision 6) |
| **the app zone** | **default tenant hosts**, `<slug>.<app zone>` | Name and registration are the operator's; `.app` preferred, per the routing table. **Submitted to the Public Suffix List before any tenant other than Paridyum's is served on it** |
| **each tenant's own domain** | **that tenant's canonical host**, once verified | Includes `paridyum.com` — Paridyum's tenant's own domain, so residency binds it through that tenant's region. The table's `marketing … carries no tenant data` note is retired |

**2. Exactly one canonical host per tenant.**

- A tenant starts on `<slug>.<app zone>`.
- When its own domain is verified and its certificate issued, **the own domain becomes canonical**, and
  the default host redirects to it permanently.
- There is never a second origin — enforced at the generator (`edge.tenant_has_exactly_one_canonical_host`).

**3. The slug is platform-owned and permanent once served.** Minted at provisioning, unique within the
app zone, and never moved to another tenant. A tenant that renames keeps its old slug host as a
redirect. The slug lives on the tenant registry, because routing is control-plane data (ADR-0117 D3).

**4. A custom domain is the standard path, not a premium one.**

- ADR-0039 step 5 runs for every tenant that adds a domain: a `DomainBinding` in `platform-routing`,
  DNS verification with a live check in the portal, and certificate issuance.
- **A tenant is `active` on its default host.** The own-domain step has its own state —
  `awaiting_domain` → `domain_active` — and **never blocks activation**.
- Compensation is ADR-0039's: remove the binding, revoke the certificate, fall back to the default
  host.

**5. The region stays the tenant's.** No zone encodes a region, and `.in` is a name.

**6. The portal is one host, and its data is always served by a region** (ADR-0117 D4, D6).

- **`paridyum.in` is served from the edge as one host**; its static interface carries no data.
- **Every data request is routed to the region of the session**, taken from the verified token's
  issuer — the region's portal instance — never from a request parameter.
- **Each region's portal instance has its own sign-in host under `paridyum.in`.** An identity
  instance never moves region, so its host may name the region it serves — an infrastructure name, as
  a datacentre id is, not a tenant hostname.
- **With one region live there is one sign-in host.** A second appears with the second region.
- **A tenant's hosts — default and own — route to that tenant's region**, from the table, as every
  tenant hostname already does.

**7. Retired.** PLAN-TENANT-FLOW D8's `<tenant>.paridyum.com` (its reason survives: the canonical host
is fixed from the start and never retrofitted); RFC-0044 Stage B's `<tenant>.paridyum.in`; the routing
table's `paridyum.in: tenant-canonical` and `paridyum.com: marketing` roles, updated in `infra-edge`.

## Consequences

**Accepted costs**

1. **One more domain to buy, renew and protect**, and a public-suffix listing is slow to obtain and hard
   to reverse. That is why it is a *separate* zone: neither `paridyum.in` nor `paridyum.com` is listed.
2. **Certificates at scale** — a wildcard for the app zone, plus one per own domain. Issuance is still
   the likeliest failure in the saga (ADR-0039 step 5), now on the standard path.
3. **White-labelling is complete only when a tenant adds its own domain.** Until then, the default host
   shows the app zone's name.
4. **`paridyum.com` becomes residency-bound**: it serves a tenant's data, so its region is a real
   constraint rather than a formality.

**Gained**

- **An individual goes live the moment the saga completes**, with no domain required.
- **Tenants cannot set cookies for one another, or for the portal.** The portal is a different
  registrable domain, and the app zone is a public suffix.
- **One canonical host per tenant**: no split sessions, no doubled cache, no split ranking.

**Rejected alternatives**

- **Own domain only** (2026-09-27). It blocks individuals without a domain; reversed 2026-09-28.
- **`<slug>.paridyum.in`.** Customer sites on the portal's own registrable domain, beside its sign-in —
  a phishing surface and a cookie-scoping hazard.
- **`<slug>.paridyum.com`** (PLAN-TENANT-FLOW D8). Every tenant inside Paridyum's own tenant's zone.

## Related

- **`infra-edge/dns/routing-table.yaml`** — the reasoning ratified, and the zone roles changed
- **RFC-0016** · **ADR-0061** · **RFC-0015** — the edge, and one home per fact
- **ADR-0039** step 5 · **RFC-0044** — Stage B retired, its owed ADR written · **PLAN-TENANT-FLOW** D8
- **ADR-0117** (D3, D4, D6) · **ADR-0118** (the zones' owners) · **ADR-0119** (login hosts) ·
  **RFC-0056** (U1, U3)
