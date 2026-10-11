---
id: RFC-0072
title: The portal at paridyum.in takes a person from joining to a working tenant, with no staff step
type: rfc
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [adr-0136, portal, web-dashboard, signup, self-serve, onboarding, tenant-creation, domains, plan-paridyum-in, s7, rfc-0056, adr-0128, adr-0118, adr-0121, adr-0129, adr-0039]
breaking: false
enforced_by: []
---

# RFC-0072 — The portal at paridyum.in takes a person from joining to a working tenant, with no staff step

## Status

Accepted by the operator, 2026-10-09. PLAN-PARIDYUM-IN S7. It designs the surface. RFC-0056 already decides the
flows, so this RFC decides how those flows are built and which of them ship first.

## Problem

Today a tenant can be created only by someone holding a portal token and calling platform-tenancy's
API from inside the cluster. X2 (PLAN-TENANT-HOSTS §6) does exactly that, with a script.
`paridyum.in` serves a static placeholder. The backend for creating a tenant is deployed: orgs, tenant
requests, the saga, routing and deployment. No person can reach it, though.

- **Who is affected:** every prospective tenant, and the operator. ADR-0128 D3 requires Paridyum's own
  tenant to be created "through the portal, as anyone would". The cutover (S8) cannot happen without a
  portal.
- **What happens if nothing is done:** tenants exist only by staff action. That contradicts the
  direction (PLAN-PARIDYUM-IN: "nothing hand-spun"), and S8 stays blocked.
- **How we know it is fixed:** a synthetic person signs in at paridyum.in, creates an org and a tenant,
  watches it reach `active`, and its products answer on its host. No staff step and no script are
  involved. This is S7's finish line in the plan.

## Constraints

- **ADR-0128 D1 (carrying ADR-0118 D1):** the portal is the platform's own surface, at paridyum.in. It
  runs in no tenant, and **`web-dashboard` becomes the portal.** Its README and AGENTS still describe
  the retired tenant-admin console (ADR-0100). That is stale text, not a competing decision.
- **RFC-0056:** the flows U1–U12, and nothing provisioned before an address is proven (B). Its abuse
  controls fail closed (D). The personal-data gate comes first (F). The copy is honest (G).
- **ADR-0129:** signup may open to real people in region `in`. It opens nothing else; the tenant data
  gate stays shut.
- **ADR-0119 D1, ADR-0047 D5:** the portal signs people in at the region's portal instance
  (`in.paridyum.in`). It forwards the person's own token; it never uses a service account for a
  person's action.
- **ADR-0039 D1:** platform-tenancy owns the saga and calls routing and deployment. **ANTI-0003:** the
  portal reads no other service's database. **ANTI-0001:** one writer per table.
- **ADR-0121 D4:** an own domain never blocks activation. The default host lives under an app zone
  that is a public suffix. **No app zone has been bought yet (P1)**, so today every tenant must give
  its own domain.
- **ADR-0070 (and its addendum of 2026-10-09):** the portal's client asks only for the audiences it
  calls.
- **STD-0015, RFC-0028:** `@paridyum/ui`, Next 16.2, software locales from `messages/` (ADR-0122 D1).
  **STD-0007:** each slice is at most 400 reviewable lines.

## Proposal

### 1. What S7 ships, and what it does not

| Flow (RFC-0056) | In S7 | Note |
| --- | --- | --- |
| U1 join | **Sign-in, yes. Open registration, as its own gated slice (§5).** | Synthetic people (portal-token.py's directory users) prove the flow first |
| U2 a team org | **Yes**: create (name, handle), and list the person's orgs | Invitations (C) are a later slice |
| U3 create a tenant | **Yes**: name, slug, products, own domain | Region, locales, timezone and currency come from the org's profile in v1 (§4) |
| Status | **Yes**: the provisioning's public state, from `requested` to `active` or `failed` | |
| Own domain | **Yes**: the TXT record to publish, and a "verify" action | §3 |
| U4–U12 | No | Each is a later RFC-0056 slice. U8's refusal is shown when the API returns it |

### 2. Architecture

- **`web-dashboard` is a backend-for-frontend at paridyum.in.**
  - **Sign-in:** PKCE, through the `web-dashboard` client in the `paridyum-portal` project, which
    already exists. The access token is kept server-side in a session cookie that is host-only,
    `Secure`, `HttpOnly` and `SameSite=Lax`, encrypted with the chart's session secret. The browser
    never sees the token.
  - **Every action** is a server-side call to platform-tenancy (`tenancy.platform.svc:8080`) carrying
    the person's own token. Tenancy introspects it at the portal instance, as it does today.
    `web-dashboard` reads no database and calls no other service.
  - **Writes** are POSTs from forms, protected by an origin check against paridyum.in and a
    per-session CSRF token.
- **Pages** (`/[lang]/…`):
  - `/` lists the person's orgs (and mints the personal org on first arrival, RFC-0056 U1);
  - `/orgs/new` creates a team org;
  - `/orgs/[org]` shows the org and its tenants;
  - `/orgs/[org]/tenants/new` creates a tenant;
  - `/orgs/[org]/provisionings/[id]` shows status, then the own-domain panel.
- **Status** is read by polling `GET /v1/orgs/{org}/provisionings/{id}` (the page refreshes itself
  every few seconds while the provisioning is not terminal). The states shown are ADR-0039 D2's public
  lifecycle, in plain words, each with a translation in `messages/`. The page never shows a step name
  or an error code (ANTI-0008).
- **The old control-plane pages are removed:** the tenant list from platform-identity, and
  `tenants/new`. They predate the org model and call a service the portal must not.

### 3. Verifying an own domain: tenancy fronts routing

The portal needs two things: the challenge to show, and a way to ask for verification. **platform-tenancy
adds `GET` and `POST /v1/orgs/{org}/tenants/{tenant}/domains/{host}`** (the challenge, and the request
to verify). It checks that the caller is a member of the org that owns the tenant, then calls routing
**as the saga**, exactly as the saga's `bindings` step does. Routing still has one caller (its API.md),
and the membership check stays where the membership data lives.

### 4. The tenant's region and languages in v1

RFC-0056 U3 lets a tenant choose its own region, content locales, locale, timezone and currency.
platform-tenancy's request carries none of them today; a tenant inherits the org's profile (ADR-0122
D2's seed). **v1 shows those values, read-only, as "from your organisation".** Region `in` is the only
live region (RFC-0051), so nothing a person could choose is withheld. Making them choosable needs the
API to accept them, which is a separate tenancy change before a second region exists.

### 5. Opening registration is its own slice, and it is gated

Registration is Zitadel's own login UI at `in.paridyum.in`: credential, passkey or password, and
address proof by mail through `mail.paridyum.in`. It is closed twice today: by the portal instance's
login policy, and by the edge (`/ui/v2/login/register` answers 404). **It opens only when every
RFC-0056 D control exists:**

- edge rate limits on registration, address proofs and tenant creation;
- a bot challenge on registration, whose vendor is chosen in its own ADR (ADR-0066, REQ-0012);
- a published abuse contact and a takedown runbook, which RFC-0056 D5 requires before the first
  non-Paridyum tenant is served;
- `max_tenants`, which already exists.

Until then the portal is exercised by synthetic people. ADR-0129 is what allows real people when it
opens.

### 6. The app zone

**ADR-0136: the app zone is a dedicated low-cost domain, as configuration.** The form makes the own
domain optional, and the default host `<slug>.<zone>` serves at activation. The zone is a value
(`SAGA_APP_ZONE`, `ROUTING_APP_ZONE`, the edge's wildcard row), so changing it later is configuration
(ADR-0136 D3). Whether a zone is configured is read from tenancy, so the form follows it without a
release.

### 7. Slices (each one PR, ≤ 400 lines, tested)

| # | Repo | What |
| --- | --- | --- |
| S7.0 | web-dashboard | README, AGENTS and ARCHITECTURE re-scoped to the portal (docs only) |
| S7.1 | web-dashboard | Sign-in at the portal instance; the session; the old control-plane pages removed |
| S7.2 | web-dashboard | Home: the person's orgs; the personal org on first arrival |
| S7.3 | web-dashboard | Create a team org |
| S7.4 | web-dashboard | An org's tenants, and the create-tenant form |
| S7.5 | web-dashboard | The provisioning status page |
| S7.6 | platform-tenancy | The domain challenge and verify routes (§3), with tests against routing's contract |
| S7.7 | web-dashboard | The own-domain panel |
| S7.8 | web-dashboard, infra-edge | The chart's staging values for paridyum.in; the edge proxies paridyum.in to the portal instead of the placeholder |
| S7.Z | platform-tenancy, platform-routing, infra-edge | The app zone (ADR-0136): `SAGA_APP_ZONE` and `ROUTING_APP_ZONE` set to the operator's domain; the proxied wildcard row and the edge's wildcard site. Configuration only: neither service needs code for a zone outside paridyum.in |
| S7.9 | infra-kubernetes | `prove-tenant-hosts.py` gains a portal stage: the same journey, through paridyum.in |
| S7.R | infra-edge, infra-kubernetes, knowledge | Registration opens (§5). Gated; not part of S7's finish line |

Order: S7.0 → S7.1 → S7.2 → S7.3 → S7.4 → S7.5 → S7.8. S7.6 runs in parallel with S7.1–S7.5. Then S7.7
and S7.9.

## Alternatives considered

1. **The portal calls routing directly for verification, with the person's token.** Rejected: routing
   would have to accept portal tokens and decide org membership. That is either a second caller
   identity with its own role vocabulary, or reading tenancy's membership data (ANTI-0003). Tenancy
   already holds membership and the saga's identity.
2. **A single-page app calling tenancy from the browser.** Rejected: the person's token would live in
   the browser, and tenancy would need a public ingress and CORS. Today it is ClusterIP and reachable
   only from inside the cluster (ADR-0118 D6's "neither side holds a path into the other" is easier to
   keep with one server-side caller). The BFF is also the pattern the tenant sites already use.
3. **A new repository for the portal instead of re-scoping web-dashboard.** Rejected: ADR-0128 D1
   names web-dashboard. Its sign-in client in the portal project already exists and carries the
   paridyum.in callback, and a second repository would leave web-dashboard as an orphan.
4. **Open registration in S7 with the controls following.** Rejected: RFC-0056 D's controls fail
   closed by design, and a signup page live before them is the unprotected window D exists to prevent.

## Consequences

- **web-dashboard loses its old pages and gains a public audience.** It becomes the first page of
  Paridyum that strangers sign up on, so its copy falls under C-0006 and STD-0021 as a public claim.
- **platform-tenancy gains two routes and a second reason to call routing.** It is still one caller
  identity (the saga's), but now acting on a person's request. Its audit events must say so.
- **Without the app zone, the first experience is slow:** a tenant is created, then waits on DNS that
  its owner publishes. This is honest, but it is friction, and P1 removes it.
- **Status by polling** costs one tenancy request every few seconds per open status page. It is
  bounded: polling stops at a terminal state.
- **Until S7.R, nobody outside the operator can join.** S7 can be finished while signup stays closed.

## Rollout

The slices land behind nothing. paridyum.in keeps serving the placeholder until S7.8 switches the edge,
and S7.8 is reverted by re-applying the edge's previous site definition. The portal's Release is a
digest, as every Release is (ADR-0060), so rolling it back is a `helm rollback`. S7.6's routes are new;
removing them breaks only the own-domain panel.

## Open questions

1. **The app zone's name** (ADR-0136: the operator buys it), and its Public Suffix List submission,
   which the first non-Paridyum tenant waits for.
2. **The bot-challenge vendor** for S7.R. That is its own ADR.
3. **Retrying or retiring a failed tenant** (RFC-0056 E). The API offers neither yet, so the status page
   can only explain a failure. Which comes first?
4. **The two test tenants X2 left `failed`** (`tenant-a`, `tenant-b`). They hold those slugs for good, by
   design. Should synthetic tenants use a reserved slug prefix so they never take a name a customer
   wants?
