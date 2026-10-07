---
id: ADR-0126
title: A tenant's host resolves to the tenant and its product configuration, and each tenant signs in through its own clients
type: adr
status: accepted
created: 2026-09-29
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, web, service, infra]
applies_to: ["*"]
product: [paridyum, routing, identity, tenancy]
tags: [multi-tenancy, routing, host, domain-binding, oidc, zitadel, client, pkce, saga, containment, c0, adr-0121, adr-0119, adr-0118, rfc-0045, rfc-0049]
breaking: true
enforced_by: []
---

# ADR-0126 — A tenant's host resolves to the tenant and its product configuration, and each tenant signs in through its own clients

## Status

**Accepted** 2026-09-29. The operator directed that paridyum.in brings every tenant's products up
through its API, on the same infrastructure, with separate configuration per tenant, including the
sign-in client, "because that's the only way they can use their identity at their domain". The
operator answered "go ahead" to this shape. `PLAN-PARIDYUM-IN.md` W1 to W3 build it.

## Context

**Every tenant-facing app in the estate is single-tenant today.**

- `web-hiring` has one Zitadel organization fixed in an environment variable.
- `web-paridyum` is static content in Paridyum's brand.
- The sign-in clients are one per app, registered for `paridyum.com` hosts only.

**ADR-0121 D2 gives each tenant exactly one canonical host,** its own domain or `<slug>.<app zone>`,
and PLAN-PARIDYUM-IN W1 serves a tenant's products as paths on that host. A shared app therefore
has to answer three questions on every request, before any session exists: which tenant does this
host belong to, is this product enabled for it, and with which identity configuration does it sign
that tenant's people in.

**The sign-in client cannot be shared once tenants have their own domains.**

- An OIDC client accepts only the redirect URIs registered on it. One client for every tenant would
  accumulate every tenant's domain, and a mistake in it would reach all of them.
- ADR-0119 D5: a role counts only in the organization that granted it. A shared client in a
  Paridyum-owned project would put every tenant's role vocabulary in one project.

**The runtime roles do not read the registry alike.** `paridyum_app` holds `SELECT` on `orgs` and
`tenants` (migration 0018), which carry no row-level security: it can read every tenant's registry
row. `docket_app` holds nothing there (migration 0019's probe: "read the tenant registry —
refused"). A lookup every surface can use, without widening `docket_app` and without depending on
`paridyum_app`'s broad read, has to be designed rather than assumed.

## Decision

**1. Host bindings, owned by `platform-routing`.**

- A `domain_bindings` table maps a host to exactly one tenant. Its columns: the host, the tenant, the
  state (`awaiting_domain` → `domain_active`, ADR-0121 D4), and whether the host is **canonical**.
- It holds every host, including the default `<slug>.<app zone>` host, so resolution is one lookup.
- At most one canonical host per tenant, enforced in the schema.
- It is declared in the one migration stream, as migration 0013 declares `platform-deployment`'s
  table. It is control-plane data (ADR-0117 D3).

**2. Per-tenant product configuration lives with the Release.**

Each enabled product's instance is an account (ADR-0104) with a Release (RFC-0049). Its identity
configuration is recorded beside it, per tenant and per product:

- the tenant's Zitadel organization id;
- the project id that tokens must be audienced for;
- the OIDC client id.

These are public identifiers. PKCE clients have no secret, so nothing here is a credential.

**3. One resolution function, and nothing else, crosses the registry boundary.**

- `paridyum.resolve_surface(host, product)` returns the tenant, the product's account, and the
  identity configuration of decision 2 — for a **`domain_active`** binding and an **enabled** product
  only.
- It is `SECURITY DEFINER`, owned by the schema owner, and granted to the runtime roles. It returns
  nothing else: no tenant name, no org, no other host.
- An unknown host, an unverified binding or a product the tenant has not enabled returns no row, and
  the app answers **404**. It never falls back to Paridyum's tenant.
- After resolving, an app reads tenant content through the ordinary tenant session (the RLS settings
  tool-docket's `enter()` sets), never through the function.
- Callers cache the result per host with the tenant in the key (T-G.1's rule), for a short time, so
  a binding's removal takes effect within that window.

**4. Each tenant signs in through its own clients, created by the saga.**

The saga's identity step (PLAN-PARIDYUM-IN W3) creates, in the region's tenant instance:

- the tenant's Zitadel organization and its owner;
- a project in that organization with the product's roles;
- **one PKCE client per enabled product**, redirecting only to that tenant's canonical host and its
  redirecting hosts.

It records the ids as decision 2's configuration. When the canonical host changes (a verified own
domain, ADR-0121 D2), the saga updates the client's redirect URIs in the same step that makes the
binding canonical.

**Sign-in happens on the tenant's own domain, and its owners arrive through federation.** This decision
adds the clients; ADR-0119 already decides the rest, and it is restated here so the saga's identity step
is complete:

- **The login host is `login.<canonical host>`** (ADR-0119 D2): the standard path, not an upgrade.
  Paridyum's tenant keeps `auth.paridyum.com`.
- **The saga links the owning org's portal instance** to the tenant's Zitadel organization as its
  "Continue with Paridyum" identity provider, with automatic linking (ADR-0119 D4). An org member signs
  in to the tenant with their one paridyum.in credential, and the creator's first grant is applied as a
  role on that linked user (ADR-0120 D2, D3).
- **Registering a login domain on the tenant instance goes through the System API,** which ADR-0123
  confines to windows. How the saga does that safely for every tenant is an open decision, owed before
  the first tenant whose login host is not already registered. Paridyum's `auth.paridyum.com` is
  already registered, so it does not block paridyum.com.

**5. Shared deployments, one pool seam.**

- Every tenant runs on the shared deployments (RFC-0045 rung C0). Provisioning writes rows and
  Zitadel objects; it deploys nothing.
- Apps obtain their database pool through one function keyed by tenant (RFC-0045 slice 3). At C0 it
  returns the shared pool. A tenant contracted for C1 or above gets its own database without an app
  change.

**6. Paridyum's tenant is configured the same way.** Its bindings (`paridyum.com` canonical), its
Releases' identity configuration and its clients are recorded as decisions 1 to 4 describe. The
saga's first run adopts them (ADR-0118 D3). `careers.paridyum.com` and `admin.paridyum.com` become
permanent redirects to paths on `paridyum.com`.

## Consequences

**Accepted costs.**

- **One `SECURITY DEFINER` function is a privileged path.** It is kept to one statement and returns
  only public identifiers. A check that it returns nothing for an unknown host, an unverified binding
  or a disabled product is owed before this ADR can be locked.
- **A lookup per request,** mitigated by a short per-host cache. A removed binding keeps serving for
  up to that cache window.
- **More Zitadel objects:** an organization, a project and a client per product per tenant. They
  are rows, created in milliseconds, but the saga must compensate them on failure (ADR-0039).
- **Login domains for new tenants wait on a decision.** Until the saga can register `login.<domain>`
  under ADR-0123's rules, only a tenant whose login host is already registered can be brought up.
  Paridyum's is.
- **Breaking for the web apps:** `web-hiring`'s fixed organization and `web-paridyum`'s static content
  are removed. Paridyum's content moves into Paridyum's rows first.

**Gained.**

- A tenant is brought up by writing rows and Zitadel objects, with no deploy, no new certificate and
  no new secret.
- A tenant's sign-in, tokens and roles are its own. One tenant's configuration error cannot reach
  another, and one tenant's access can be revoked alone.
- Paridyum is a tenant like any other, with no special code path.

**Rejected alternatives.**

- **One sign-in client per shared app, for every tenant.** Every tenant's domains would sit on one
  client, every tenant's roles in one project, and one mistake would reach all of them. It also
  cannot carry tenant branding cleanly.
- **The edge injects a tenant header.** It moves the registry to the edge and is safe only if nothing
  can reach an app except through the edge. On this estate network, that is not guaranteed.
- **A deployment per tenant.** It multiplies capacity and operations by the number of tenants, and it
  is what RFC-0045's C2 and C3 rungs are for, by contract.
- **Grant every runtime role `SELECT` on the registry,** as `paridyum_app` already has. That exposes
  every tenant's registry row to every app, which ADR-0115 D6's least privilege argues against. With
  the function in place, `paridyum_app`'s registry read can be withdrawn once nothing depends on it,
  and that is recorded as debt rather than done here.

## Related

- ADR-0121 — one canonical host; domain bindings; `awaiting_domain` → `domain_active`.
- ADR-0119 — identity split by plane; a role counts only where it was granted.
- ADR-0118 — paridyum.in provisions; Paridyum's tenant is adopted.
- ADR-0104, RFC-0049 — an account is a product instance; a Release is a product running for a tenant.
- RFC-0045 — the containment ladder and the pool seam.
- ADR-0123 — the System API only in a window, which is why login-domain registration needs its own decision.
- `PLAN-PARIDYUM-IN.md` — W1 to W3.
