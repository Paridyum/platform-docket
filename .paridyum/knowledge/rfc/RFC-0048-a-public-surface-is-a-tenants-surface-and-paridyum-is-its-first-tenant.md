---
id: RFC-0048
title: A public surface is a tenant's surface, and Paridyum is its first tenant
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web, service]
applies_to: ["*"]
product: [tenancy]
tags: [web, careers, landing, multi-tenant, branding, catalogue, product-registry, isr, cache, anti-0009, adr-0049, adr-0050, adr-0084, adr-0099, adr-0100, c-0006, c-0008]
breaking: true
enforced_by:
  - check: web.cache_key_carries_tenancy
    severity: error
    rationale: >
      Slice 1 of this RFC. Next.js ISR keys on pathname, not `Host`, so the commit that makes a
      surface multi-tenant serves one tenant's content to another — with RLS forced, the policy
      correct, and the query never running. The check fails a build that omits the tenant from the
      cache key, which is the only place this is catchable before it leaks.
---

# RFC-0048 — A public surface is a tenant's surface, and Paridyum is its first tenant

## Problem

`web-hiring` and `web-paridyum` are built as **Paridyum's own websites**. Everything else in the
estate is being built as a tenant-served product — ADR-0099 makes docket multi-tenant, ADR-0100
serves a catalogue product at `/` on its tenant's hostname, ADR-0084 makes branding per tenant —
and the two public web surfaces were not included in that direction.

Observably:

1. **`web-hiring` resolves its tenant from configuration, not from the request.** `runInTenantContext`
   reads `paridyumTenantId()` from config. There is exactly one tenant it can ever serve, which is
   the shape ADR-0099 removed from docket when it deleted `DOCKET_TENANT_ID`.

2. **Neither surface declares a `product:`.** `web-admin` declares `product: [identity]`; `web-hiring`
   and `web-paridyum` declare none. ADR-0050 makes `product` a governance axis and an absent value
   means *every* product — so two deployables that are in fact about one thing are governed as
   though they were about all of them.

3. **The careers page caches on a key that will not survive multi-tenancy.** `careers/page.tsx` sets
   `export const revalidate = 60`. Next.js ISR keys on pathname, not on `Host`. Under ADR-0100
   decision 3's model — a tenant's product at `/` on `<tenant>.<zone>` — two tenants resolve the same
   path and the second is served the first's cached postings. **This is correct today and becomes a
   live ANTI-0009 breach on the commit that makes the app multi-tenant.**

4. **A tenant running hiring has no public face.** docket is becoming a multi-tenant HR platform and
   its tenants will post jobs. There is nowhere for those jobs to be seen. A hiring product whose
   careers page only exists for Paridyum is a product with a missing half.

5. **ADR-0049 decision 3 is unsatisfied by both.** *No deployable capability ships without a declared
   tenant-facing consumer.* Each currently names Paridyum's own marketing, which decision 2 says is
   a tenant — but a capability only tenant zero can consume is exactly what C-0008 forbids.

**Who experiences it.** Paridyum, now, as tenant zero: its own careers surface is built on a shape
that must be rewritten before any second tenant, and rewritten *after* rows exist is the expensive
order. The operator's direction on 2026-09-15 was explicit — Paridyum is the first tenant and the
first customer, not a special case.

**What happens if nothing is done.** The multi-tenant commit lands on a codebase that assumes one
tenant, and the ISR key becomes a cross-tenant leak that RLS cannot see and no existing check
catches.

**How we will know it is fixed.** `web-hiring` serves any tenant's careers surface on that tenant's
hostname, with that tenant's branding, resolving the tenant from the request; Paridyum's own careers
page is one instance of that with nothing special about it; and a cache key that omits the tenant
fails a check rather than a customer.

## Constraints

- **C-0008 / ADR-0049 (locked)** — Paridyum is tenant zero and holds no scope a customer could not.
  Decision 3: no deployable ships without a declared tenant-facing consumer.
- **ADR-0100 (accepted) decision 3** — a catalogue product is served at `/` on its tenant's hostname
  (`<tenant>.<zone>` or a custom domain); `<platform-zone>/dashboard` is reserved for the control plane and
  is never reachable from a tenant's hostname.
- **ADR-0099 (accepted)** — the precedent, in full: docket had a fixed `DOCKET_TENANT_ID`, the token's
  org becomes the tenant, the app derives it, RLS does the isolation. This RFC applies the identical
  move to the public surfaces.
- **ADR-0082** — `deriveTenantId(orgId)`, no lookup. Already implemented in `web-hiring`.
- **ADR-0084 (accepted)** — branding is per tenant because a tenant is an organization.
- **ADR-0074 / ADR-0086** — the careers index/application split, and what moved off the landing.
- **ADR-0050** — `product` is a fourth governance axis; absent means every product.
- **ANTI-0009** — a tenancy-blind cache key. The anti-pattern this RFC's §Proposal C exists to
  prevent becoming real.
- **ADR-0024 (locked)** — two columns, forced RLS. A public unauthenticated read still runs under a
  tenant context (`runInTenantContext` already does this correctly); what changes is where the tenant
  comes from.
- **C-0006** — the commodity list, which §Proposal B's open question turns on. A generic website
  builder is on the commodity side; a product's own public surface is not.
- **RFC-0045 / ADR-0103** — placement and containment. A tenant's public surface lives in that
  tenant's placement like everything else.
- **product-registry.yaml** — a `catalogue` product at `stage: proposed` owes an `MKT-`, and C-0006
  requires its selecting evidence be non-tenant-derived. Registering is not free.

## Proposal

### A. `web-hiring` becomes a tenant-served product — the ADR-0099 move, applied again

Identical in shape to what ADR-0099 did for docket, and deliberately so: one precedent, applied
twice, rather than two designs.

1. **The tenant is derived from the request, never from config.** For the authenticated surfaces,
   from the verified token's org via `deriveTenantId` (ADR-0082), exactly as today. For the
   **public, unauthenticated** careers surfaces, from the **hostname** — `<tenant>.<zone>` or a
   custom domain — resolved through the routing table, never from a path segment, query string or
   header a visitor controls.
2. **`paridyumTenantId()` is removed.** Paridyum's careers page is then one tenant's careers page,
   reached the same way every other tenant's is. This is ADR-0049 decision 2 made literal.
3. **An unresolvable host fails closed** — a 404, never a fallback to a default tenant. A fallback
   here is ADR-0024 decision 4's hazard in routing clothes: breadth granted by a lost value.
4. **Branding comes from the tenant** (ADR-0084), not from the build.
5. **RLS is unchanged.** `runInTenantContext` already establishes a tenant-only context with
   `operator_scope=false` for the public read, and the `job_postings` SELECT policy already admits
   `status = 'open'` rows for any session that has set the tenant. The policy needs no change; only
   the source of the tenant id does.

### B. `web-paridyum` — the landing — **is** the same move. Corrected 2026-09-15.

**An earlier draft of this section argued the opposite and was wrong.** It claimed a tenant landing
page is a website builder, and therefore a commodity C-0006 forbids positioning on. That rested on a
misreading: it assumed "a tenant's landing page" meant *arbitrary pages, arbitrary copy, arbitrary
layout*. It does not. It means **the same product, rendering that tenant's content.**

The precedent is already in the estate and already migrated. `job_postings` (migration 0007) is
`-- paridyum:scope tenant` with localized `jsonb` columns per ADR-0096 — `title`, `description`,
`responsibilities`, `location` — so the careers *product* renders whatever rows its tenant has.
Paridyum's careers page is not special code; it is the careers product with Paridyum's rows.

**The landing is the same shape.** Its content becomes tenant-scoped rows with a declared schema, and
Paridyum's landing is the landing product with Paridyum's rows. That is not a CMS: a website builder
sells *arbitrary structure*, and this sells a **structured product whose content is per tenant** —
the same distinction that makes `job_postings` a hiring product rather than a database with a form
in front of it.

So C-0006 is not engaged. Nothing here positions on, prices on, or describes as our value the ability
to author arbitrary web pages. What is sold is a product; what differs per tenant is data.

**The operator's formulation, which is the clearest statement of the whole model:**

> *Tenant zero just seeds the data. That is what it does differently.*

`web-paridyum` therefore takes §Proposal A's treatment: tenant from the request, content from
tenant-scoped rows, branding from ADR-0084. What is left in the repository is the **product** — the
layout, the components, the routes — and what leaves it is Paridyum's copy, which becomes rows like
every other tenant's.

**One thing genuinely does stay Paridyum's**, and it is worth naming so it is not migrated by
accident: the pages whose content is *fixed by artifacts* — the category, the forbidden framings, the
segment, the unit of sale, all of which the manifest says are set by C-0006, BET-0006 and BET-0009.
Those are Paridyum's positioning, not a template slot. They belong to product zero's own content
rows, not to the product's schema.

### C. A cache key carries the tenant, and a check says so

The concrete defect §Problem 3 predicts, closed before it opens:

1. **Every cached value keys on the tenant.** For ISR that means the tenant is in the route — a
   rewrite that puts the resolved tenant into the path before the page renders, so two tenants
   cannot collide on `/en/careers`. Not a `cacheTag` bolted on: ISR's key is the path, so the path is
   what must carry it.
2. **The key comes from the server-resolved tenant, never the raw `Host`.** ANTI-0009's most
   dangerous form is a key built from a caller-supplied value.
3. **A check.** `web.cache_key_carries_tenancy` — declared now, so ANTI-0009 stops being prose with
   no owner. It is the only item in this RFC that is a *latent defect* rather than a missing feature,
   which is why it is proposed independently of the rest and should land first.

### D. Both surfaces declare a `product:`

`web-hiring` → the hiring/careers product. `web-paridyum` → `[paridyum]`, since it is product zero's
own front door. ADR-0050: absent means every product, which is true of neither.

**Registering a new `careers` catalogue product is not free** — `catalogue` at `stage: proposed` owes
an `MKT-`, and C-0006 requires its evidence be non-tenant-derived. Whether careers is its own product
or part of the hiring product is §Open questions 2.

## Alternatives considered

1. **Leave both single-tenant; build a separate tenant-facing careers app later.** Rejected: two
   codebases rendering job postings diverge immediately, and the ISR defect stays latent in the one
   that will eventually be made multi-tenant anyway. It also leaves ADR-0049 decision 3 unsatisfied
   for a deployable that already exists.

2. **Keep the landing single-tenant as Paridyum's own site.** What an earlier draft of this RFC
   proposed, on the argument that a tenant landing page is a website builder and therefore commodity
   under C-0006. **Rejected on 2026-09-15**: the argument rested on a misreading (§Proposal B). A
   structured product with per-tenant content rows is not a builder of arbitrary structure, and
   `job_postings` already proves the pattern in this estate. Recorded because the commodity boundary
   is real and would bite a *genuine* page-builder — which this is not.

3. **Serve every tenant's careers page on a Paridyum path (`paridyum.com/t/<tenant>/careers`).**
   Trivially fixes the cache key by putting the tenant in the path. Rejected: it contradicts
   ADR-0100 decision 3 and BET-0005's *"a tenant's end customers need never learn Paridyum exists"* —
   the whole white-label posture is the tenant's own hostname.

4. **Keep ISR and vary on `Host`.** Rejected: it depends on cache-layer behaviour that is easy to get
   wrong, silent when wrong, and not checkable from the source. Putting the tenant in the route makes
   the key structurally correct and inspectable.

## Consequences

**What this makes worse.**

1. **Paridyum's careers page becomes a tenant's careers page, with no shortcuts available.** Every
   convenience that assumed one tenant — a hardcoded brand, a build-time job read, a fixed locale set
   — has to go. That is ADR-0049 decision 2 being expensive exactly where it is supposed to be.
2. **A public unauthenticated surface now depends on host resolution.** A routing misconfiguration
   becomes a 404 for a tenant's careers page, which is a visible outage on a page that used to be
   static.
3. **ISR's benefit narrows.** Per-tenant cache entries mean a lower hit rate and more revalidation,
   and the fast-static property ADR-0095 §8 was protecting is partly spent.
4. **`breaking: true`.** `paridyumTenantId()` and the config it reads are removed, and the careers
   routes change shape.
5. **The landing's content leaves the repository.** Paridyum's own copy stops being reviewable as a
   pull request and becomes rows — which is what ADR-0074 already accepted for postings, and is still
   a real loss of the review path for text that states the company's positioning. §Proposal B's last
   paragraph is the mitigation and it depends on someone honouring a distinction rather than a check.

**What it does not change.** RLS, the session contract, the policies on `job_postings`. The tenant
context for a public read is already established correctly; only its source moves.

## Rollout

| Slice | What lands | Why this position |
|---|---|---|
| **1** | `web.cache_key_carries_tenancy` declared; the careers route carries the resolved tenant; ANTI-0009 gains an owner | The only **latent defect** here. Lands before multi-tenancy, not with it |
| **2** | `product:` declared on both manifests | Trivial, and ADR-0050 already requires it |
| **3** | Host → tenant resolution for the public careers surfaces; unresolvable host 404s | The ADR-0099 move |
| **4** | `paridyumTenantId()` removed; Paridyum becomes one tenant among N | Only after 3 works |
| **5** | Per-tenant branding on the public surfaces (ADR-0084) | Independent |
| **6** | Landing content schema; `web-paridyum` tenant-served; Paridyum's copy becomes its own rows | The largest slice — it is a content migration, not a routing change |

**Slice 1 is the one with a deadline**, and it is small. Everything else can wait for docket's
tenants to be real.

**Backwards compatibility.** Slices 1–2 are additive. Slice 3 is additive if the existing config
path is kept as a fallback *during the slice only* — and that fallback must be deleted in slice 4,
because a default tenant left in place is §Proposal A.3's failure waiting to happen.

**How it is undone.** Slices 1, 2, 5, 6 are droppable. 3 and 4 revert together or not at all.

## Open questions

1. ~~**Does a tenant get a landing page?**~~ **ANSWERED 2026-09-15 by operator direction: yes.** It
   is the same product with the tenant's own content rows, not a page builder. What remains open is
   narrower: **what is the landing's content schema?** `job_postings` had a natural one; a landing
   page's sections do not, and an over-general schema is how a product becomes a builder by accident.
2. **Is `careers` its own catalogue product, or the public half of the hiring product?** Decides
   whether an `MKT-` is owed and how the manifests declare `product:`.
3. **How does a tenant's custom domain reach the careers surface?** Depends on the routing and
   certificate work ADR-0100 references and RFC-0044 stage 3 lists — currently unwritten.
4. **What does a tenant's careers page show before it has postings?** An empty state on a tenant's
   own domain is a first impression; it is a product decision, not a template default.
5. **Does the public surface need its own quota?** An unauthenticated, cacheable, per-tenant-keyed
   page is the most abusable surface in the estate, and RFC-0045 §Proposal G's quotas were designed
   for authenticated paths.

## Related

- **ADR-0099** — the precedent this applies a second time
- **ADR-0100** — decision 3's URL model, which decides where a tenant's surface is served
- **ADR-0049, C-0008** — Paridyum is tenant zero; a capability only tenant zero consumes is forbidden
- **ADR-0084** — per-tenant branding
- **ADR-0074, ADR-0086** — the existing careers/landing split this builds on
- **ANTI-0009** — the anti-pattern §Proposal C gives an owner
- **ADR-0050, product-registry.yaml** — the `product:` axis and what registering one owes
- **C-0006** — the commodity boundary §Proposal B turns on
- **RFC-0047** — the same question one level up: what is a product, and who may run it
