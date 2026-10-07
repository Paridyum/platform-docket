# Plan — paridyum.in brings every tenant up; nothing is hand-spun

*Written 2026-09-29. Supersedes the build order of `PLAN-TENANT-FLOW.md` §4, whose flow the org
model already re-scoped (ADR-0113 to ADR-0122). `TASKS.md` remains the authority for task state;
this document answers **"how does paridyum.in bring a tenant's products up, and in what order is
that built"**.*

## 0. The operator's direction, 2026-09-29

> paridyum.in will spin up auth, landing, docket and everything else for tenants — what they
> enabled — but they are wired in and working **separately** from paridyum.in. We are focused on
> spinning them up from the paridyum.in API instead of hand-spinning them.

Three rules follow, each already carried by an accepted artifact:

1. **paridyum.in provisions; it does not serve a tenant.** The portal and the saga are the control
   plane (ADR-0118 D1, ADR-0116 D1). A tenant's landing, careers, docket and sign-in run in the
   tenant plane, on the tenant's host.
2. **The portal being down never takes a tenant down.** Nothing on a tenant's request path calls
   paridyum.in. Identity is split by plane (ADR-0119): a tenant's people sign in at the tenant
   instance, never at the portal's.
3. **Nothing a tenant runs is set up by hand, Paridyum's included.** Genesis provisions only the
   platform: instances, shared deployments, database, edge. **paridyum.com is created through
   paridyum.in** like any tenant (ADR-0128, superseding ADR-0118 D3's adoption).

## 1. What "spinning up" is: data, identity and routing on shared deployments

**A tenant is not a deployment.** At containment rung C0 (RFC-0045), one deployment of each app
serves every tenant and resolves the tenant from the request. Docket already works this way: it
derives the tenant from the token (ADR-0099). Bringing a tenant up is four things, none of which
installs software:

| Step | What it creates | Where it lives today |
| --- | --- | --- |
| **Identity** | the tenant's Zitadel organization in the region's tenant instance, and its owner | `infra-kubernetes/zitadel/provision-tenant.py`, run by hand |
| **Records** | the `tenants` row, owned by the requesting org (`owner_org_id`), and the owner's first grant (ADR-0120 D2) | `provision-tenant.py` plus migrations 0005, 0011 and 0017 |
| **Releases** | one `releases` row per product the tenant enabled (RFC-0049, migration 0013), with that product's default rows: landing sections (0016), docket configuration (0008, 0009) | seed scripts, run by hand |
| **Routing** | the tenant's canonical host: a route at the edge, DNS and a certificate (ADR-0121) | `infra-edge` inventory and `dns/apply.py`, edited by hand |

**Dedicated per-tenant deployments are rungs C2 and C3**, which are sold only when built and
rehearsed (ADR-0103). They are not part of v1.

## 2. Every product, mapped

| Product (`product-registry.yaml`) | Surface or repository | Runs in | Resolves its tenant today | Gap to "provisioned by the portal" |
| --- | --- | --- | --- | --- |
| **paridyum** (platform) | `web-dashboard` as the portal at paridyum.in (ADR-0118 D1) | control plane | n/a (serves orgs) | Still the pre-org-model control plane: it lists tenants on paridyum.com. Re-scope to orgs, its own portal instance, and joining (RFC-0056). |
| **tenancy** | `platform-tenancy` | both planes | n/a | The schema exists (orgs 0001/0017, tenants, releases 0013). **No org or tenant API** a portal can call. |
| **identity** | Zitadel: staff `id.paridyum.com`, tenant `auth.paridyum.com`, portal instance (not yet); `platform-identity`; `web-admin` | tenant plane, except staff | `platform-identity` classifies by issuer | Portal instance absent. `provision-tenant.py` is a script, not a step a service can run. |
| **routing** | `infra-edge` (nginx and lego), Cloudflare DNS | edge | by `server_name`, per hand-written site | Every host is an inventory edit and a certificate reissue. **The app zone is not bought** (ADR-0121), so no wildcard is possible. |
| **entitlement** | none yet (`max_tenants` per org, ADR-0114 D5) | control plane | n/a | Proposed. v1 needs only the per-org tenant count, enforced in the saga. |
| **docket** | `tool-docket` (API), `console-docket` (HR console) | tenant plane | **yes**: from the token (ADR-0099) | Configuration rows per tenant (0008, 0009) are seeded by hand. Its screens move into each tenant's console at `/dashboard` (ADR-0127); no standalone console is served. |
| **hiring** | `web-hiring` (careers) | tenant plane | **no**: the Zitadel organization is fixed in `PARIDYUM_ZITADEL_ORG_ID` | T-G.12: resolve the tenant from the host, so one deployment serves every tenant's careers. |
| **landing** (RFC-0048 §E; registered under `paridyum` today) | `web-paridyum` | tenant plane | **no**: static content in Paridyum's brand | Read `landing_sections` per tenant, and a per-tenant **theme** (palette, type, logo) on top of `lib-ui`'s tokens. Paridyum's site becomes one tenant's configuration of it. |
| **hosting**, **veer-entitlements**, **reel** | none | n/a | n/a | Catalogue, `proposed`. Outside v1. |

**The landing is the template every tenant customises.** It is built in Paridyum's brand because
Paridyum's tenant is its first configuration, but the brand is data: `lib-ui` already expresses the
palette and scale as tokens (STD-0015), so a tenant's theme is a row that overrides those tokens,
and its content is `landing_sections`. Nothing in the component tree may name Paridyum.

## 3. The build, in waves

Each wave has a done-when that is observed on the running estate, not asserted.

### W0 — the estate stays up

On 2026-09-28 the host ran out of memory while three network-lab guests ran beside `pve-staging`,
and the hypervisor guest hung. Every tenant surface went down with it.

- **Rule:** nothing else runs on the estate host while the estate is served from it, or it runs
  under a hard memory cap sized so that `pve-staging` keeps its full allocation.
- **Done when:** the rule is written into infra-substrate's SOP, and the host shows free memory with
  `pve-staging` at its full allocation.

### W1 — one host per tenant, products by path, and every surface resolves its tenant

This is the step that makes provisioning a data operation. Until it lands, a new tenant still needs
a new deployment, a new environment variable, or a new subdomain.

**The shape (ADR-0121 D2, taken as the build shape on 2026-09-29).** A tenant has exactly one
canonical host, and its products are **paths** on it, routed at the edge to the shared deployments:

| Path on the tenant's host | Deployment | Resolves the tenant from |
| --- | --- | --- |
| `/` | `web-paridyum`, the landing | `Host` |
| `/careers` | `web-hiring` | `Host` |
| `/dashboard` | `web-console` (not yet created), holding each enabled product's screens — docket's from `console-docket` (ADR-0127); `web-admin` serves it until then | `Host`, then the session's grant |
| `/api/docket` | `tool-docket` | the token (ADR-0099), cross-checked against `Host` |

- **Every tenant gets this shape** at `<slug>.<app zone>` or its own domain. Provisioning never
  mints a subdomain or a certificate per product.
- **Paridyum's pre-model hosts become permanent redirects:** `careers.paridyum.com` →
  `paridyum.com/careers`, and `admin.paridyum.com` → `paridyum.com/dashboard`.
- **There is no `console.paridyum.com`** (ADR-0127): docket's screens are part of each tenant's
  console at `/dashboard`.

**Host bindings, product configuration and sign-in, as ADR-0126 decides.**

- A host maps to exactly one tenant through a **domain binding**, owned by `platform-routing`
  (ADR-0121 D4). It is declared in the one migration stream as 0013 declares `platform-deployment`'s
  table.
- The binding has a state (`awaiting_domain` → `domain_active`) and marks the one canonical host.
- The default host is derived from `tenants.slug` and needs no binding.

**Resolution.** Each surface calls `paridyum.resolve_surface(host, product)` once per request,
cached with the tenant in the key (T-G.1's rule). It returns the tenant, the product's account and
that tenant's sign-in configuration for the product: organization, project and client. An unknown
host answers 404; it never falls back to Paridyum.

**Sign-in.** Each tenant signs in through its own PKCE client per enabled product (ADR-0126 D4).
Paridyum's tenant gets its clients the same way, recorded now and adopted by the saga.

**Per surface.**

- **`web-paridyum`:** renders `landing_sections` and the tenant's theme. Paridyum's content moves
  into Paridyum's rows; no component names Paridyum.
- **`web-hiring`:** drops `PARIDYUM_ZITADEL_ORG_ID` and serves under `/careers` (T-G.12).

**Done when:** one deployment of each surface serves two tenants on two hosts with different content
and themes, the old Paridyum hosts redirect, and a request for an unknown host returns 404.

### W2 — the org and tenant API

- **`platform-tenancy` gains the org context's API** (ADR-0114, ADR-0116): create an org, list its
  tenants, and create a tenant for it within its entitlement (`max_tenants`, ADR-0114 D5).
- It writes only its own tables (ANTI-0003). It is called by the portal's BFF and the saga, and by
  nothing on a tenant's request path.
- **Done when:** its API tests prove org isolation both ways, and that a request over the entitlement
  is refused.

### W3 — the saga, in `platform-tenancy`, calling the two owners it needs

> **Corrected 2026-09-29.** This section first put the saga in `platform-deployment`. **ADR-0039 D1
> says `platform-tenancy` owns it:** the thing being created is a tenant, so ownership follows the
> entity (ANTI-0001). `platform-deployment` owns **Releases** (PLAN-TENANT-FLOW D1, RFC-0049), and
> the saga asks it for them, as it asks `platform-routing` for host bindings (ADR-0121 D4). The
> operator chose to build both owners properly (2026-09-29) rather than let the saga write their
> tables (ANTI-0003).

- **The saga (`platform-tenancy`, ADR-0039).** `POST /v1/orgs/{id}/tenants` answers `202` with the
  tenant id. Its public lifecycle is `requested → provisioning → active`, or `→ compensating → failed`
  or `compensation_failed`. The state is durable in `tenant_provisionings` (platform scope, control
  plane). Every remote id is derived (UUIDv5 over the provisioning id), so a retry discovers rather
  than duplicates. The steps:
  1. the `tenants` row (`requested`), owned by the requesting org, within its `max_tenants`;
  2. the Zitadel organization in the region's tenant instance;
  3. the owning org's portal instance linked as its identity provider (ADR-0119 D4);
  4. **platform-deployment:** one account and Release per enabled product, each with its project and
     PKCE client (ADR-0126 D4);
  5. **platform-routing:** the tenant's default host binding, and its own domain if one was given
     (`awaiting_domain`);
  6. the creator's first grant (ADR-0120 D2): the `operator_grants` row now, applied as a role on the
     linked user at the first sign-in;
  7. `active`, the only path to it (ADR-0039 D8).
- **`platform-deployment`:** writes `accounts`, `releases`, `release_events` and `sign_in_clients`,
  and creates the Zitadel project and client for a Release. An internal API, called by the saga only.
- **`platform-routing`:** writes `domain_bindings`, and verifies a domain before it goes
  `domain_active` (ADR-0121 D4). An internal API, called by the saga and later by the portal.
- **Done when:** a synthetic tenant is created end to end through the API, a failed run converges on
  retry, and a forced failure compensates to `failed` with nothing left serving.

### W4 — the portal at paridyum.in

- **The region's portal instance** of Zitadel (ADR-0118 D3 step 5, ADR-0119).
- **Joining, per RFC-0056:** a person proves an address in a region they chose, and nothing is
  provisioned before that.
- **`web-dashboard` re-scoped to orgs:** org profile, members and invitations (0017), the org's
  tenants, and "create a tenant", which calls W2 and starts W3.
- **Done when:** a stranger joins at paridyum.in, creates an org and a tenant, and reaches that
  tenant's landing and careers at its host, with no step taken by Paridyum's staff.

### W5 — routing without hand edits

- **Wildcard DNS and certificate on the app zone:** a new tenant's default host needs no DNS or
  certificate work (ADR-0121).
- **The edge's host table** is rendered from `platform-tenancy`, not hand-edited in the inventory.
- **Custom domains:** a certificate per domain, issued once the tenant proves control of the domain.
- **Done when:** a tenant created in W4 is reachable at `<slug>.<app zone>` within a minute of
  creation, and adding a custom domain needs no commit.

## 3b. The sprint: paridyum.in spins up paridyum.com

**The finish line.** The operator joins paridyum.in, creates the Paridyum org, and creates Paridyum's
tenant with `paridyum.com` and its products. `paridyum.com/`, `/careers` and `/dashboard` are then
served by the shared deployments for the tenant the saga created. The rows made by tooling on
2026-09-28 are gone (ADR-0128 D3). The slices run in this order, and each depends on those above it.

| # | Slice | Done when (observed, not asserted) |
| --- | --- | --- |
| S1 | **The personal-data gate** (RFC-0056 §F): **ADR-0129**, the operator's risk acceptance for signup only, closing on a real off-host restore, review 2026-10-29. The tenant data gate (careers, docket) stays closed | **done 2026-09-29**: ADR-0129 `accepted` |
| S2 | **Migration 0020 applied** (platform-tenancy#29) | `resolve_surface` answers on the cluster; `verify-isolation` passes both runtime roles |
| S3 | **The portal instance for `in`** (ADR-0119 D1, ADR-0121 D6): a virtual Zitadel instance through an ADR-0123 window; its sign-in host **`in.paridyum.in`**, the region's code from `region-registry.yaml`, which generates it; self-registration with a verified address, passkey-first (RFC-0056 answers); mail from paridyum.in's sending subdomain | a synthetic person joins, verifies their address, and receives a token from the portal instance |
| S4 | **Org and tenant API** (W2) in `platform-tenancy`, called with portal tokens: create an org (registry row plus `org_profiles`/`org_members`), list an org's tenants, create a tenant within `max_tenants` | the API tests prove org isolation both ways, and that a request over the entitlement is refused |
| S5 | **The saga** (W3) in **`platform-tenancy`** (ADR-0039 D1; corrected 2026-09-29), calling **`platform-deployment`** (Releases, sign-in clients) and **`platform-routing`** (host bindings), both built as owners (operator, 2026-09-29) | a synthetic tenant is created end to end, a failed run converges on retry, and a forced failure compensates to `failed` |
| S6 | **Apps resolve by host** (W1): the landing renders the resolved tenant's `landing_sections` and theme; careers drops its fixed organization; both sign in with the tenant's own client; the edge routes paths on each bound host | two synthetic tenants served by one deployment of each, with an unknown host answering 404 |
| S7 | **The portal UI** (W4): `web-dashboard` re-scoped to orgs — join, org, create tenant (domain, products), tenant status | a synthetic person goes from joining to a working tenant with no staff step |
| S8 | **Cutover** (ADR-0128 D3): the operator creates the Paridyum org and tenant through the portal; `paridyum.com` binds to it; the tooling-made rows and Zitadel organization are removed; `careers.` and `admin.paridyum.com` redirect | `paridyum.com/`, `/careers` and `/dashboard` serve the saga's tenant; the old tenant id resolves nowhere |

**What each slice needs from the operator:**

- **S1:** done, ADR-0129. Closing it needs a real off-host destination (P8).
- **S3:** the System-API window that creates the portal instance (ADR-0123). The host name is settled: `in.paridyum.in`.
- **S5:** creating `platform-deployment` (P7) and `platform-routing`: both approved 2026-09-29.
- **S8:** the operator's own signup.

## 4. Decisions only the operator can make

| # | Decision | Recommended | Blocks |
| --- | --- | --- | --- |
| P1 | **Buy the app zone** and submit it to the Public Suffix List | a short, product-neutral domain | W5 (W1 to W4 can use hosts under paridyum.com meanwhile) |
| P2 | **RFC-0056 open question 1:** unverified and invite expiry | 7 and 14 days | W4 |
| P3 | **RFC-0056 open question 2:** passkey-first or password-first | passkey-first, with password as a fallback | W4 |
| P4 | **RFC-0056 open question 3:** disposable addresses | allow them; abuse controls fail closed (§D) | W4 |
| P5 | **RFC-0056 open question 4:** bot-challenge vendor | one with no account lock-in, behind an interface | W4 |
| P6 | **RFC-0056 open question 5:** handle namespace | unique per region, as proposed | W4 |
| P7 | **Create `platform-deployment`** (a new service boundary) | yes, when W3 starts | W3 |
| P8 | **A real off-host backup destination.** Today's copy is a second bucket on the same host. | a destination outside `sitar`; options go to the operator with their data-residency reading | the backup gate, and so real applications and personnel records |

**Taken 2026-09-29:**

- **P1:** a `.xyz` zone for now, for example `paridyum.xyz`. The exact name is still to be
  confirmed, and the operator buys it. Only W5 waits on it.
- **P2 to P6:** the recommendations, all accepted. RFC-0056 is `accepted`, with each answer recorded
  under its open questions.
- **The host model:** paths on one canonical host (W1), as ADR-0121 D2 reads.
- **Per-tenant configuration and sign-in clients** on shared deployments: ADR-0126.
- **No standalone HR console:** docket lives in each tenant's console (ADR-0127, superseding ADR-0098).

**Still open:** P7, when W3 starts, and P8.

## 5. What is hand-made today, and becomes the saga's

Each is correct for Paridyum's tenant and is adopted, not redone:

- Paridyum's org and tenant rows, and its Zitadel organization (`provision-tenant.py`).
- The shared deployments: docket and its console, careers, landing, admin.
- The edge sites and DNS records for paridyum.com and its subdomains.
- Docket's and the landing's seeded rows.
