---
id: RFC-0044
title: docket as a service completes the platform against ADR-0042, tenant zero first
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, web, infra]
applies_to: ["platform-tenancy", "platform-routing", "platform-identity", "tool-docket", "web-docket", "spec-product"]
tags: [daas, multi-tenant, provisioning, custom-domain, tenant-zero, adr-0042, adr-0039, adr-0049, adr-0099, adr-0098, adr-0082, adr-0084, adr-0070, rfc-0016, c-0008, c-0010]
breaking: true
enforced_by: []
---

# RFC-0044 — docket as a service completes the platform against ADR-0042

## Problem

docket's multi-tenant *backend* is built (ADR-0099: tenant-from-token, per-request RLS store,
per-tenant config from the 0008/0009 tables; onboarding via `seed-config.py`). But a multi-tenant
backend is not a service. To sell docket — a tenant spins up **under its own domain**, provisioned
**by tenant zero** through a **dashboard**, and its HR staff use a **console** — three platform
contexts ADR-0042 already named must be *built*, one genuinely new surface must be *decided*, and
docket must finish becoming a *product*.

The request states the shape in the operator's words:

- `/app` is the provisioning dashboard — the one thing NOT provisioned by tenant zero, the bootstrap
  that provisions everything else.
- `/` is tenant zero's first app (docket for Paridyum).
- every other tenant app is provisioned by tenant zero through `/app`.
- tenants run under their own domain.

## What already decides this — do not rebuild

The platform decomposition exists. This RFC fits into it; it does not invent a parallel one.

- **ADR-0042** decomposes the runtime into five contexts: `platform-tenancy` (Operator/Tenant/
  Account), `platform-identity` (Zitadel), `platform-entitlement` (plans), `platform-routing`
  (**Route, DomainBinding** — the custom-domain entity), and *the first catalogue product* — **which
  docket is.** `platform-billing` is deferred on RFC-0009's trigger (the first operator to charge);
  `spec-product` holds each product's contract.
- **ADR-0039** already specs tenant provisioning as a compensating saga owned by platform-tenancy,
  with a public lifecycle (`requested → provisioning → active`), `POST /tenants` → `202`, and step 5
  being *"add custom domain to the instance, issue certificate"* (premium). The user's "spin up under
  their own domain" is that step; "provisioned by tenant zero" is that saga run by Paridyum.
- **ADR-0049** binds the dashboard: *"anything shaped like a support console or an internal dashboard
  — if tenant data can reach it, it is in a request path and this ADR binds."* So `/app` is **not**
  outside a tenant. It is tenant zero's control-plane *surface* that provisions others; it resolves
  the Paridyum tenant like everything else, and holds no scope a customer could not be granted
  (ADR-0049 D2). The "exception" the operator means is narrow and real: `/app` is the one surface
  whose *job* is to provision OTHER tenants — but it still runs within tenant zero.
- **ADR-0098** already decided the tenant HR interface is served (web-docket); **ADR-0084** makes
  branding per-tenant; **ADR-0082** derives the tenant id; **ADR-0070** governs its OIDC clients;
  **RFC-0016** is the edge that routes a hostname to a region.

**So the mental model maps exactly:** `/app` = a served surface of `platform-tenancy` driving
ADR-0039's saga, run by tenant zero. `/` = docket (the first catalogue product) for tenant zero.
custom domains = `platform-routing`'s DomainBinding + ADR-0039 step 5. "provisioned by tenant zero" =
ADR-0049 with Paridyum as the tenant that runs the saga.

## What remains, and who owns it

| # | Remaining work | Context / repo | Governed by | New decision? |
|---|---|---|---|---|
| 1 | Deploy the MT docket image + web-docket console for tenant zero at `/`; the provisioning surface at `/app` | tool-docket, web-docket, platform-tenancy | ADR-0098, ADR-0099 | ADR-0100 (the surface) |
| 2 | The provisioning **saga** as an API (`POST /tenants`, lifecycle) driving identity + seed + routing | platform-tenancy | ADR-0039 | — (impl of a decided saga) |
| 3 | **Custom domains**: DomainBinding + edge/cert integration, a tenant on its own hostname | platform-routing | ADR-0042, ADR-0039 §5, RFC-0016 | a routing ADR, not yet written |
| 4 | docket's **product contract** (its Zitadel project, roles, audience, tenancy scope) | spec-product | ADR-0042 §4, RFC-0012 | — (declaration) |
| 5 | **Per-tenant templates** — a tenant's own document layouts as reviewed content | tool-docket | extends ADR-0099 | a templates ADR, not yet written |
| 6 | Entitlement / **billing** | platform-entitlement, platform-billing | ADR-0042 §3 | **stays deferred** (trigger: first paying operator) |
| 7 | **Compliance to hold customer PII**: the ADR-0068/0071 backup-gate rehearsal; DPDP erasure over the API | infra, tool-docket | ANTI-0007, ADR-0071 | — (operational + tracked debt) |

## The sequence (the priority)

Dogfood first (ADR-0049 D2 — Paridyum is a tenant, and the first), widen second, monetise last.

- **Stage A — tenant zero is served end to end.** Deploy the multi-tenant docket image, seed
  Paridyum's config, serve docket for Paridyum at `/`, web-docket (the HR console, ADR-0098) as its
  GUI, and the provisioning surface at `/app` (ADR-0100) — which at first provisions only by an
  operator's action. **Outcome:** Paridyum runs its own HR entirely on the product it sells, GUI and
  all. Nothing here is new capability; it is deployment + the surface.
- **Stage B — the provisioning saga becomes an API.** platform-tenancy exposes ADR-0039's
  `POST /tenants` lifecycle; `/app` drives it; a second tenant is created by tenant zero through the
  dashboard and gets its org, derived tenant, seeded defaults (`--defaults --operator`), OIDC client,
  and a **subdomain** hostname. **Outcome:** a second tenant exists, on `<tenant>.paridyum.in`.
- **Stage C — custom domains.** platform-routing's DomainBinding + the edge cert flow (ADR-0039 §5)
  put a tenant on **its own domain.** **Outcome:** the headline capability — a tenant under its own
  domain.
- **Stage D — per-tenant templates** (owes its own ADR, not yet written): a tenant's documents render with its own layout.
- **Stage E — billing** stays deferred until a real operator must be charged (ADR-0042 §3); building
  it earlier encodes a pricing guess BET-0002 does not authorise.

## Rejected alternatives

- **A fresh decomposition for "DaaS."** Rejected: ADR-0042 already decomposed the runtime, ADR-0039
  already specs provisioning, ADR-0049 already binds the dashboard. A second decomposition would give
  two answers to one question (the decompose-spec failure mode) and collide on `tenant`/`invoice`/
  `member`, which RFC-0009 already split.
- **`/app` as a tenant-less control plane.** Rejected by ADR-0049 D4 — a dashboard that tenant data
  reaches is a request path and must resolve a tenant. `/app` runs within tenant zero and provisions
  others; it is not an exemption.
- **Custom domains before subdomains.** Rejected: ADR-0039 makes the domain step *premium* and last
  in the saga precisely because certificate issuance is the slow, failure-prone step; a subdomain
  proves the saga end to end without it (Stage B before C).
- **Billing now.** Rejected: ADR-0042 §3 defers it on a trigger; encoding pricing before BET-0002
  decides it is a guess that hardens.
- **One `web-` app serving both `/` and `/app`.** Open question below — the surfaces have different
  audiences (a tenant's HR vs the platform operator) and ADR-0070 D2 keeps populations in separate
  projects, which argues for separate clients at least, possibly separate repos.

## New decisions this RFC raises (ADRs, once accepted — ADR-0013 stage 3)

- **ADR-0100** — the provisioning surface (`/app`): a served surface of platform-tenancy, run by
  tenant zero, that provisions every other tenant; bound by ADR-0049; the `/` vs `/app` URL model.
- **A per-tenant templates ADR — NOT YET WRITTEN.** Document templates as reviewed content, extending ADR-0099's config plane.
- **A custom-domain routing ADR — NOT YET WRITTEN.** platform-routing's domain binding and the edge certificate flow (fills
  ADR-0039 §5 / RFC-0016 with a mechanism).

## Open questions

- Does `/app` live in the same `web-docket` deployable as `/`, a distinct `web-console`/`console-`
  repo, or a surface of platform-tenancy? (ADR-0100 decides.)
- Custom-domain certificate authority and challenge type at the edge (RFC-0016, and the unwritten routing ADR).
- The default template set a new tenant starts from — Paridyum's, or a neutral generic. Owed by the unwritten templates ADR.
