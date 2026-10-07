---
id: ADR-0130
title: A tenant's products are paths on its host routed at the edge, and its owner is provisioned already linked
type: adr
status: accepted
created: 2026-10-02
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, web, service, infra]
applies_to: ["*"]
product: [paridyum, routing, identity, tenancy, hiring, docket]
tags: [multi-tenancy, routing, edge, nginx, tls, lego, host, zitadel, idp-link, owner, login-host, adr-0126, adr-0121, adr-0120, adr-0119]
breaking: true
enforced_by: []
---

# ADR-0130 — A tenant's products are paths on its host routed at the edge, and its owner is provisioned already linked

## Status

**Accepted** 2026-10-02. The operator answered "lets build all of it" to `PLAN-TENANT-HOSTS.md`, whose
decisions H2 to H8 this records. H1, buying the app zone, is the operator's purchase and is not decided
here.

## Context

ADR-0121 gives a tenant one canonical host and ADR-0126 resolves a host to its tenant, its product's
account and its sign-in client (`paridyum.resolve_surface`, migration 0020, live). Between an `active`
tenant and a person using its products, the estate as read on 2026-10-02 lacks six things:

1. **Path routing.** The cluster runs no ingress controller; each app is a LoadBalancer VIP (landing
   `.100`, careers `.103`, admin `.104`, docket `.106`), and the edge (`infra-edge`, nginx) maps one
   host to one VIP.
2. **Certificates** for tenant hosts. The edge's lego job issues one SAN certificate by DNS-01 on
   Cloudflare, so only names on zones Paridyum holds.
3. **A host for synthetic tenants** while the app zone is unbought.
4. **A login host.** ADR-0126 makes `login.<canonical host>` standard and notes that registering a
   domain on the tenant instance needs the System API, which ADR-0123 confines to windows. Zitadel
   v4.15.3 also offers instance *trusted domains* ("trusted to be used as public hosts"), but adding one
   needs `iam.write`, which only `IAM_OWNER` holds (`cmd/defaults.yaml`, read 2026-10-02).
5. **The owner's access.** The saga writes the owner's `operator_grants` row (ADR-0120 D2), but every
   app authorises on the token's `operator` role, and nothing grants a role in a new tenant's
   organization. The per-tenant hiring project declares no roles.
6. **Content.** A Release writes no `landing_sections`, postings or docket configuration.

Three facts from Zitadel v4.15.3 decide how 5 is closed: `AddHumanUser` (user v2) accepts a
client-chosen `user_id`, an `organization`, a verified email and `idp_links`; and `IAM_ORG_MANAGER`,
which `tenancy-provisioner` already holds, carries `user.write`, `user.grant.write` and
`project.role.write`.

## Decision

**D1 — Paths are routed at the edge.** `infra-edge` gains one catch-all `server` for tenant hosts whose
`location`s send `/` to the landing, `/careers` to careers, `/dashboard` to the dashboard and
`/api/docket` to docket, passing `Host` and `X-Forwarded-*` unchanged. (`/api/docket` waits on a mesh
policy: the edge reaches the cluster on port 80 only, and docket's VIP is 8080.) Every app resolves its tenant
from `Host` and answers 404 for no row; the edge does not decide tenancy. No ingress controller is
added.

**D2 — Synthetic tenants are first-level names under paridyum.com until the app zone exists:**
`tenant-a.paridyum.com` and `tenant-b.paridyum.com`, each recorded as the tenant's **own domain**, so
verification runs for real against a TXT record Paridyum writes. They are proxied like every edge host:
the edge admits HTTPS only from the rented tier, whose free certificate covers first-level names only, so
a second-level zone (`*.tenants.paridyum.com`, tried first on 2026-10-02) cannot be reached. Each is an
explicit edge site, so no unlisted name reaches the edge. These tenants are test fixtures: they are
removed at S8, and no real tenant is given such a host.

**D3 — A real tenant's own domain needs a decision before W5,** because it cannot reach the edge
through the rented tier as D2's names do: either the edge admits HTTPS beyond the rented tier, with a
certificate issued by HTTP-01 **only for a `domain_active` binding** from a host list rendered from
platform-routing, or the rented tier's custom-hostname product carries it. The edge never requests a
certificate for a name nobody has proven.

**D4 — Sign-in for new tenants is on `auth.paridyum.com`, scoped to the tenant's organization, until a
login host can be registered without widening the saga's privilege.** The app requests the
organization scope (`urn:zitadel:iam:org:id:<zitadel_org_id>`), so the login shows that tenant's
organization and "Continue with Paridyum". `login.<canonical host>` (ADR-0119 D2, ADR-0126) stays the
target; it waits on a decision between a narrowly used `IAM_OWNER` account for trusted domains and
ADR-0123 windows, and this exception is reviewed then. This amends ADR-0126 §Consequences ("only a
tenant whose login host is already registered can be brought up"): a new tenant is brought up, signing
in on the instance's registered host.

**D5 — The owner is provisioned already linked.** A saga step `owner_access`, after `releases` and
before `first_grant`:

- creates the owner's human user in the tenant's organization under a **derived id** (UUIDv5 of the
  provisioning), with the requester's **verified** address and an **IdP link** to the portal identity
  provider for the requester's portal subject; an existing user with that id is success;
- grants that user `operator` on each enabled product's project in the organization.

"Continue with Paridyum" then signs the owner in **to that user**, so the role is in the token from
the first sign-in. Compensation removes the grants and deactivates the user. The request records the
requester's verified address (regional plane), and the hiring project declares `operator`.

**D6 — Defaults, not seeds.** A tenant with no rows renders defaults: the landing its display name and
locale from `tenant_settings`, careers an empty list, docket its default configuration. The saga
writes nothing in tables it does not own (ANTI-0003).

**D7 — A non-canonical host redirects.** An additive SECURITY DEFINER function
`paridyum.canonical_host(host)` returns the canonical host of the tenant an active binding serves, or
nothing; each app answers 308 to it when the request's host differs.

## Consequences

**Accepted costs.**

- **Every app request on a tenant host crosses the edge's path table.** A new product path is an edge
  change, reviewed like any other.
- **New tenants sign in on `auth.paridyum.com`,** not on their own domain, until D4's decision is taken.
  Their people see Paridyum's host in the address bar during sign-in.
- **The saga creates a person's user in a tenant organization,** which is personal data in the tenant
  instance's region; it is the person who asked, with the address they proved.
- **`tenants.paridyum.com` is a Paridyum-held zone serving test tenants.** It must hold nothing a
  visitor could mistake for Paridyum's own.

**Gained.**

- A tenant created through paridyum.in is usable on its host with no hand step: routing, certificate,
  content, sign-in and the owner's role are all data or one-time configuration.
- The owner's first sign-in needs no hook on the login path and cannot race a grant.

**Rejected alternatives.**

- **An ingress controller in the cluster** — a new platform component for routing the edge already
  does; revisit at C1, when per-tenant isolation needs it.
- **Granting the owner on first sign-in through a Zitadel action** — a hook on every login, failing open
  or closed on a remote call; D5 makes it a provisioning fact instead.
- **`IAM_OWNER` for the saga's provisioner** to add trusted domains — full control of the tenant
  instance for one call; D4 keeps the narrow role and defers the login host.
- **Seeding content from the saga** — writes into tables owned by the landing, careers and docket.
