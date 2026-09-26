---
id: ADR-0100
title: The provisioning surface is tenant zero's control plane, served at /dashboard
type: adr
status: accepted
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [daas, provisioning, control-plane, tenant-zero, routing, web, adr-0042, adr-0039, adr-0049, adr-0098, adr-0070, adr-0084, anti-0001, anti-0003, anti-0006, c-0008, rfc-0044]
breaking: true
enforced_by: []
---

# ADR-0100 — The provisioning surface is tenant zero's control plane, served at `/dashboard`

## Status

**ACCEPTED** as of 2026-09-15. Settles the surface question RFC-0044 raised. Builds on ADR-0042
(the five contexts), ADR-0039 (the provisioning saga), ADR-0049 (nothing runs outside a tenant),
and ADR-0098 (the HR interface is served).

## Context

docket-as-a-service means a tenant spins up **under its own domain**, provisioned **by tenant zero**
through a **dashboard** (RFC-0044). The operator's model: `/app` is the dashboard that provisions
everything; `/` is a tenant's own app; every other app is provisioned by tenant zero through
`/dashboard`.

Two things must be reconciled with what is already decided. First, ADR-0049 D4: a dashboard that
tenant data reaches is a request path and **must resolve a tenant** — so `/dashboard` is not "outside a
tenant." Second, ADR-0042 already gives `platform-tenancy` the tenant lifecycle and ADR-0039 makes
provisioning its saga — so the dashboard must not become a second owner of `Tenant` (ANTI-0001) or a
god-surface that both provisions and owns (ANTI-0006).

## Decision

**1. `/dashboard` is a served *surface*, not an owner.** The provisioning dashboard is a `web-` deployable
(ADR-0020: chart, probes, digest-pinned image) that is a **BFF over `platform-tenancy`'s API** — it
calls `POST /tenants` and reads the ADR-0039 lifecycle. It writes no tenant entity itself; it never
touches platform-tenancy's tables (ANTI-0003). `platform-tenancy` remains the sole owner of
`Operator`/`Tenant`/`Account` and the saga (ANTI-0001). The surface is the control plane's *view*.

**2. It runs within tenant zero, and is bound by ADR-0049.** `/dashboard` resolves the **Paridyum tenant**
like every other request path; the operator acting there is a tenant-zero operator whose token
carries no scope a customer could not be granted (ADR-0049 D2, GLO-0005). "Provisioned by tenant
zero" is exactly this: the saga runs as tenant zero. The narrow, real "exception" the operator means
is that this is the ONE surface whose *job* is to provision OTHER tenants — but it is not a
tenant-less plane, and it holds no god scope.

**3. The URL model. — REVISED 2026-09-15; the original is preserved below.**

**Every host has the same two levels: `/` is the public surface and `/dashboard` is the signed-in
one. Which tenant owns the host decides what they contain.**

| Host | `/` | `/dashboard` |
| --- | --- | --- |
| `<platform-zone>` (`paridyum.com`) | Paridyum's landing | Paridyum's dashboard — **which is the control plane**, because Paridyum holds that Release |
| `<tenant>.<zone>` or a tenant's own domain | that tenant's landing | that tenant's dashboard — its own products |

*The original text read:* "The provisioning surface is served at **`/app` on the platform zone
only** (`paridyum.com/app`), never on a tenant's own domain… `platform-routing` reserves
`<platform-zone>/app` for this surface."

**Two things changed, and the second is the one that matters.**

**The name.** `/app` became `/dashboard`, by operator decision on 2026-09-15. It was already the
word this ADR used for the thing — §Context describes the operator's model as *"`/app` is the
dashboard that provisions everything"* — so the path now matches the vocabulary rather than
competing with it. Renamed completely, per STD-0022.

**The shape.** `/dashboard` is no longer a platform-only path. It is the signed-in surface of
**whatever tenant owns the host**, which makes the URL model uniform: Paridyum is a tenant with a
landing and a dashboard, exactly like every other tenant, and its dashboard happens to be the
control plane because it is the tenant that holds the control-plane Release (RFC-0049). This is
ADR-0049 decision 2 reaching the routing layer — *"tenant zero just seeds the data"* — and it makes
decision 4's asymmetry smaller still.

**⚠ What this gives up, stated plainly because it was a real defence.** The original reserved the
path, so the control plane was unreachable from a tenant's hostname **by routing**. Under the
revision the path exists on every host and no longer protects anything by itself. **The control
plane is now gated by entitlement and by the Release a tenant holds, not by the URL.** A tenant
reaching `<their-host>/dashboard` gets their own products because that is all they are entitled to
and all that is deployed for them — not because the router refused.

That is a weaker mechanism than a reserved path and it must be treated as such: it is
`domain.no_self_serve_path_to_a_dedicated_rung` (ADR-0103) and ADR-0024's operator-scope rules doing
the work a route used to do. If those are wrong, a tenant reaches a provisioning surface. The
original's reserved path is recoverable at any time if that trade proves bad, and the revision is
written so the reader knows there is something to recover.

**4. It is the bootstrap, and that is the one asymmetry.** Every tenant app — including Paridyum's
own docket at `/` — is produced BY the ADR-0039 saga this surface drives. The surface itself is not
produced by that saga (there is no tenant whose provisioning creates the platform's control plane);
it is deployed by the estate's normal pipeline (ADR-0020/ADR-0052) as tenant zero's control plane.
This is the bootstrap the operator names: the provisioner is not provisioned by a tenant.

**5. Its buyer is named (ADR-0049 D3).** A provisioning surface is how the platform is *sold*: an
operator provisions a tenant, and later a prospective tenant self-provisions. Its `manifest.yaml`
names that consumer. Operator-driven creation ships first; **self-service signup is a later mode of
the same saga** — a public route that starts `requested → provisioning` for a would-be tenant — and
does not change ownership, only who may call `POST /tenants`. Deferring it keeps the first cut from
encoding a signup/identity flow before it is designed.

**6. It is distinct from `web-docket`.** `web-docket` (ADR-0098) is a *tenant's HR console* — one
population (a tenant's operators), served per tenant. The provisioning surface is the *platform
operator's* console — a different population (ADR-0070 D2 keeps populations in separate projects),
served once on the platform zone. They are two `web-` deployables with two OIDC clients, not one
surface with two modes.

## Consequences

- **A new `web-` repository** for the provisioning surface (name to be minted under C-0002, e.g.
  `web-console`), `deployable: true`, a BFF over `platform-tenancy` — owning no domain entity, so it
  is a view and not a context of its own (it is `platform-tenancy`'s surface, the way a product's web
  app is that product's surface).
- **`platform-tenancy` gains the saga API** ADR-0039 specs (`POST /tenants`, `GET /tenants/{id}`),
  which this surface consumes — the work RFC-0044 Stage B names.
- **`platform-routing` resolves `<host>` to a tenant**, then `/` to that tenant's public surface and
  `/dashboard` to its signed-in one. *(Revised 2026-09-15 — it previously reserved
  `<platform-zone>/app`.)*
- **The control plane is reachable only by the tenant entitled to it.** *(Revised 2026-09-15.)* This
  was a routing property and is now an entitlement one — see decision 3's ⚠. Weaker, deliberate,
  and recoverable.
- **Paridyum at `/` is not special, and now neither is Paridyum at `/dashboard`.** Both are served
  the way every tenant's are; what differs is which Releases Paridyum holds.
