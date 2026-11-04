---
id: ADR-0143
title: The tenant console is web-console, one host-resolved surface at /dashboard
type: adr
status: accepted
created: 2026-10-10
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [paridyum, docket]
tags: [console, web-console, tenant-console, dashboard, host-resolution, multi-tenancy, oidc, rename, std-0022, adr-0113, adr-0118, adr-0126, adr-0127, adr-0128, adr-0130, adr-0136, plan-tenant-flow]
breaking: true
enforced_by: []
---

# ADR-0143 — The tenant console is `web-console`, one host-resolved surface at `/dashboard`

## Status

**Accepted** 2026-10-10. It carries **ADR-0128 D4** (from ADR-0118 D4) and **PLAN-TENANT-FLOW D4**,
which named `web-console` as the tenant console but left two things open: **which repository builds
it**, and **how it relates to the interim `web-admin`**. It decides both. It does not reopen ADR-0126's
host resolution, ADR-0127's docket screens, or ADR-0130's paths.

## Context

**The console is already decided in shape, and has no repository of its own.**

- **ADR-0113 D1** names **the console** — *"a tenant's own administration surface, on the tenant's
  host"* — as distinct from **the portal** at paridyum.in.
- **ADR-0128 D4** (carried from ADR-0118 D4) serves it at `<tenant's canonical host>/dashboard` and
  names it **`web-console`**.
- **ADR-0126** decides how a host resolves to a tenant and its clients; **ADR-0127** moves docket's
  screens into it; **ADR-0130 D1** serves it as a path; **ADR-0136** names the app zone.
- **PLAN-TENANT-FLOW D4** and **PLAN-PARIDYUM-IN W1** name `web-console` as **a new repository**, and
  record that **`web-admin` serves it until then**.

**But `web-admin` is already the console, in code.** Measured 2026-10-10, `web-admin`'s `main`:

| Behaviour | Where | Decided by |
| --- | --- | --- |
| Served under `/dashboard` on a host | `src/lib/base-path.ts` (`basePath: "/dashboard"`) | ADR-0130 D1 |
| Resolves the tenant from `Host` | `src/lib/tenancy/site.ts` — `resolve_surface(host, 'docket')` | ADR-0126 D3 |
| Signs in with the tenant's own client | `src/lib/tenancy/site.ts` (client, project, redirect URI) | ADR-0126 D4 |
| Redirects a non-canonical host | `siteForHost` → `canonical_host()` | ADR-0130 D7 |
| Relays the member's own token to docket | `src/lib/docket.ts` | ADR-0127 D2 |

It is deployed **once**, on `admin.paridyum.com`, and its `manifest.yaml` still describes it as *"the
tenant-admin surface on admin.paridyum.com … a BFF in front of platform-identity"*, `product:
[identity]`, `depends: [platform-identity]`. `console-docket`'s screens are not in it. ADR-0118 D4 left
one question open — *"whether `web-admin` is retired into the portal (a complete rename, STD-0022) or
keeps a role on tenant hosts"*.

**So the work is not to build a console.** It is to **name the one that exists, finish it, and serve it
on every tenant's host.**

## Decision

**1. The tenant console is `web-console`.** The repository now named `web-admin` is **renamed to it, in
one complete change** (STD-0022). One name; `web-admin` does not survive as a second surface.

- It is a **`web-`** deployable — the class test is reachability, and tenant traffic reaches it, so it
  is not the workstation `console-` class (class-registry).
- It **answers ADR-0118 D4's open question**: `web-admin` keeps a role on tenant hosts, and that role
  is the console. It is **not** retired into the portal.
- **Whether the console also carries a tenant's own identity screens** (its members and roles) is
  ADR-0128 D4's deferred question, decided with the portal's identity screens, not here. What makes
  this repository the console is the **host-resolved product surface**, not those screens.

**2. One deployment, host-resolved.** Every tenant's console is the same image. The tenant is resolved
from `Host` through **`paridyum.resolve_surface(host, product)`** (ADR-0126 D3), cached per host with
the tenant in the key.

- An **unknown host, an unverified binding, or a product the tenant has not enabled answers 404.** It
  never falls back to Paridyum (ADR-0126 D3).
- A **non-canonical host** of a tenant answers **308** to the canonical one (ADR-0130 D7).

**3. It signs in with the tenant's own client, and forwards the member's own token.** The client,
project and Zitadel organization come from the resolved surface (ADR-0126 D2, D4); the session is
scoped to the host and `/dashboard`; every call to a product carries the signed-in person's own access
token (ADR-0127 D2).

**4. `/dashboard` is its root, and the tenant's enabled products are its sections.** Docket's screens
move in from `console-docket` (ADR-0127 D1, D3); a product appears only when the tenant holds its
Release (ADR-0126 D3).

**5. The console shows the tenant's insides; the portal shows the org's.** The portal owns orgs, their
members and their businesses, and **links into** `https://<canonical host>/dashboard`; the console
never renders an org's members, invitations or grants, and the portal never renders a tenant's products
(ADR-0113 D1's line).

**6. `admin.paridyum.com` retires.** It becomes a **permanent redirect** to `paridyum.com/dashboard`
(ADR-0126 D6), and the `web-admin` OIDC client retires in the same change (STD-0022).

**7. The console holds no business rules.** It is a BFF that relays the person's token to the tenant's
services and renders their answers; a rule that belongs to a product lives in that product.

## Consequences

**Accepted costs.**

- **A complete rename of a deployed repository** — directory, package identity, manifest, docs, the
  Zitadel client and its redirect URIs, the registries, and the live cross-references (STD-0022's
  checklist). The old host and the old client retire with it.
- **One deployment serves every tenant**, so a fault in it reaches all of them; the fail-closed path
  (404, no fallback) is the mitigation and the property most worth a test.
- **The console is empty for a tenant that has enabled nothing** until products' screens land; docket's
  is the first.

**Gained.**

- **The console the estate designed exists under the name it was given**, and `web-admin`'s stale
  description stops disagreeing with its code.
- **The portal's line holds**: org things at paridyum.in, a tenant's own things at its own `/dashboard`.
- **Paridyum's HR is `paridyum.com/dashboard`**, like any tenant's (ADR-0127 D1).

**Rejected alternatives.**

- **Keep `web-admin` and call it the console.** The name is the interim one and its description is one
  the code has outgrown; the estate already named the console `web-console` (ADR-0128 D4), so keeping
  the old name needs an amendment rather than a decision.
- **A new repository beside `web-admin`.** It would deploy the same host-resolved surface twice, with
  two clients on overlapping redirect URIs — the two-things-one-capability shape STD-0022 forbids.

## Related

- **ADR-0113 D1** — the console vs the portal · **ADR-0118 D4 / ADR-0128 D4** — the console's host, and
  the open `web-admin` question this ADR answers
- **ADR-0126** — host resolution and per-tenant clients · **ADR-0127** — docket's screens in the console
- **ADR-0130** — products as paths, `/dashboard` · **ADR-0136** — the app zone
- **STD-0022** — the rename · **PLAN-TENANT-FLOW D4**, **PLAN-PARIDYUM-IN W1**
