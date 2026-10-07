# Plan — a new tenant's products work on its own host

*Written 2026-10-02. Details S6 of `PLAN-PARIDYUM-IN.md` §3b, which stays the authority for the
sprint's order; `TASKS.md` stays the authority for task state. This document answers **"what stands
between a tenant the saga has just made `active` and a person using that tenant's landing, careers and
dashboard on its host"**, from what the estate is on this date, read rather than assumed.*

## 0. The finish line

Two synthetic tenants, **T1** and **T2**, each created through the API (org → tenant request → saga),
each on its own host. On each host, with no hand step after the request:

1. `/` shows **that** tenant's landing (its name, its sections or its defaults), never the other's.
2. `/careers` lists that tenant's postings; a candidate signs in and applies.
3. The tenant's owner signs in with **"Continue with Paridyum"** and reaches `/careers` admin and
   `/dashboard` as an operator, and is refused on the other tenant's host.
4. An unknown host answers 404 on every path; a non-canonical host of a tenant redirects to its
   canonical one.

## 1. Where the estate stands (read 2026-10-02)

| Piece | State | Source |
| --- | --- | --- |
| `paridyum.resolve_surface(host, product)` | **Live** (0020 applied 2026-09-29). Returns tenant, account, Zitadel org, project, client for an active binding and an active Release, or no row. **Does not return the canonical host.** Executable by `paridyum_app` and `docket_app` | migration 0020 |
| Saga writing bindings, Releases, per-tenant clients | Code in PRs (platform-tenancy #45–#56, routing #4–#13, deployment #4–#16), merging. 0024 applied 2026-10-02 | LOGBOOKs |
| Landing (`web-paridyum`) | **Static content** in Paridyum's brand; reads no database; one env (`PARIDYUM_PARIDYUM_SITE_URL`) | `src/` |
| Careers (`web-hiring`) | One tenant per deployment from `PARIDYUM_ZITADEL_ORG_ID`; **the seam for host resolution is already isolated** (`src/lib/tenancy/resolve.ts`), the tenant is a route segment so the ISR cache cannot cross tenants; one fixed OIDC client; admin needs the `operator` role in the token; served at a host root, no `basePath` | `src/lib/auth/config.ts`, `operator.ts` |
| Dashboard | `web-admin` at `admin.paridyum.com` until `web-console` exists (PLAN-PARIDYUM-IN W1); no `basePath` | `next.config.ts` |
| Docket API | Tenant from the token (ADR-0099) | `tool-docket` |
| Cluster routing | **No ingress controller.** Each app is its own LoadBalancer VIP: landing `.100`, careers `.103`, admin `.104`, docket `.106` | `kubectl get svc` |
| Edge | nginx, one `server` per listed host, each proxied to **one** VIP; one SAN certificate for every listed host by lego **DNS-01 on Cloudflare** | `infra-edge` `edge_proxy_sites`, `lego.service.j2` |
| App zone | **Not bought** (P1: a `.xyz`, name to confirm) | PLAN-PARIDYUM-IN §4 |
| Login host per tenant | ADR-0126: `login.<canonical host>` is standard, but **registering a domain on the tenant instance needs the System API**, confined to ADR-0123 windows; "only a tenant whose login host is already registered can be brought up" | ADR-0126 §Consequences |
| Owner's access | The saga writes the owner's `operator_grants` row, but **no app reads `operator_grants`**; apps authorise on the token's `operator` role, which nobody grants in a new tenant's organization. The per-tenant **hiring project declares no roles** (deployment catalogue) | `saga.rs`, `catalogue.rs` |
| Default content | A Release creates no rows: a new tenant has no `landing_sections`, no postings, no docket configuration (0008/0009) | `platform-deployment` |

## 2. The gaps, in the order they block

1. **Nothing routes a tenant's host to the apps by path.** The edge knows hosts, not paths, and has
   no catch-all.
2. **No certificate exists for a tenant's host,** and lego's DNS-01 works only on zones we hold on
   Cloudflare.
3. **No app resolves its tenant from `Host`** (landing, careers, dashboard), and none serves under its
   path.
4. **No app signs in with the tenant's own client.**
5. **A tenant's people have nowhere to sign in** (login host registration, ADR-0123).
6. **A new tenant's owner holds no role** that any app honours.
7. **A new tenant has no content,** and docket has no configuration.

## 3. Decisions owed (each becomes an ADR or an amendment before its slice)

| # | Decision | Recommended | Why | Blocks |
| --- | --- | --- | --- | --- |
| **H1** | **Buy the app zone now** (P1) and submit it to the Public Suffix List | Yes. Default hosts `<slug>.<zone>` get a wildcard DNS record and a wildcard certificate by DNS-01, once, and every new tenant is reachable with no DNS or certificate step | Without it every tenant needs its own domain, a TXT challenge and a per-domain certificate before anything works | E2, T1 |
| **H2** | **Synthetic tenants until H1:** hosts under a Paridyum-held zone, `tenant-a.paridyum.com`, `tenant-b.paridyum.com` (first-level and proxied: see ADR-0130 D2), recorded as the tenant's **own domain** (DNS verification then runs for real, against a record we write) | Yes, test-only, removed at S8 | Lets the whole path be proven today on zones we already control | test only |
| **H3** | **Path routing lives at the edge,** not in a new ingress controller: one catch-all tenant `server` whose `location`s send `/` → landing, `/careers` → careers, `/dashboard` → dashboard, `/api/docket` → docket, passing `Host` through | Yes. It reuses the VIPs as they are, and the edge already terminates TLS | An ingress controller is a new platform component (an ADR on its own) for routing the edge can already do | E1 |
| **H4** | **Certificates for own domains: per-host HTTP-01 at the edge, issued only for `domain_active` bindings,** the host list rendered from platform-routing (PLAN-PARIDYUM-IN W5) | Yes, after H1; until then H2's hosts are covered by a DNS-01 wildcard | A certificate is requested only for a host the tenant has proven, so the edge cannot be made to request arbitrary names | E3 |
| **H5** | **Login host:** register `login.<canonical host>` on the tenant instance **through Zitadel's instance-level *trusted domains* (admin API), if v4.15 serves the login there** — no System API, so no ADR-0123 window per tenant | Verify first (research R1); if it holds, an ADR amending ADR-0126 §Consequences; if not, the fallback is **interim sign-in at `auth.paridyum.com` scoped to the tenant's organization**, recorded as a time-boxed exception | The ADR's current text blocks every new tenant on a per-tenant window | A3, test step 3 |
| **H6** | **Owner access:** a saga step `owner_access`, after `releases`: create the owner's user in the tenant's organization **linked to their portal identity** (an IdP link: portal provider, portal subject), and grant it `operator` on each product project. "Continue with Paridyum" then signs in **to that user**, and the role is in the token from the first sign-in. Compensation removes the grants and deactivates the user | Yes. It makes ADR-0120 D2's "applied at first sign-in" deterministic and idempotent, with no hook on the login path | Needs the requester's address on the request (a migration), and `operator` declared on the hiring project (catalogue) | P1, P2 |
| **H7** | **Defaults, not seeds:** a tenant with no rows renders defaults (landing: its display name and locale from `tenant_settings`; careers: "no open roles"); docket's per-tenant configuration gets defaults in docket, not rows written by the saga | Yes. No cross-service writes; a tenant edits its content later through its dashboard | Seeding from the saga would write tables the saga does not own (ANTI-0003) | A1, D1 |
| **H8** | **Canonical redirects:** a migration adds `paridyum.canonical_host(host)` (SECURITY DEFINER, the tenant's canonical host for an active binding, or nothing), and each app 308-redirects when the request's host is not it | Yes, additive, same pattern as `resolve_surface` | `resolve_surface` deliberately returns only identifiers; a second narrow function keeps it so | R2 |

## 4. Research (done 2026-10-02; decisions recorded in ADR-0130)

- **R1 found:** trusted domains are on the instance admin API (`POST /admin/v1/trusted_domains`) and need
  `iam.write`, held only by `IAM_OWNER`. **ADR-0130 D4:** new tenants sign in on `auth.paridyum.com`
  scoped to their organization; `login.<host>` waits on a privilege decision.
- **R3 found:** `AddHumanUser` takes a chosen `user_id`, an organization, a verified email and
  `idp_links`; `IAM_ORG_MANAGER` (the saga's provisioner) holds `user.write` and `user.grant.write`.
  **ADR-0130 D5:** the saga provisions the owner already linked.

The original research list:

- **R1 — trusted domains (H5).** Confirm against Zitadel v4.15.3: the admin API's trusted-domain
  endpoints, whether the login UI (v2, per-instance deployment, ADR-0090) serves a trusted domain with
  the instance's branding and the organization's, and what `x-forwarded-host` the edge must pass.
  Prove it on staging with a throwaway trusted domain, then remove it.
- **R2 — Next.js `basePath` with ISR and the proxy** in careers and admin: the cache key, the
  callback URL `https://{host}/careers/api/auth/callback` (the deployment catalogue's), static assets.
- **R3 — IdP links through the management API** (H6): `AddIDPLink`/user creation with a linked
  external identity, and that "Continue with Paridyum" with auto-linking finds that user rather than
  creating a second.

## 5. The slices (each one PR, ≤ 400 lines, tested, a mutant watched failing — STD-0007)

**Edge (`infra-edge`)**

- **E1 — a tenant catch-all with path routing.** One `server` for the tenant zone(s), `location`s per
  H3, `Host` and `X-Forwarded-*` passed, everything else 404 at the edge. *Done when* a request for a
  synthetic host reaches the landing VIP at `/` and the careers VIP at `/careers`, and an unlisted zone
  gets nothing.
- **E2 — the wildcard certificate** for the tenant zone (H2's now, H1's when bought), DNS-01 on
  Cloudflare, in lego's existing unit.
- **E3 — own-domain certificates** (H4), after H1: the edge's host list rendered from platform-routing.

**Routing and the schema**

- **R-a — `canonical_host()`** (H8), migration 0025, probe both ways.
- **R-b — the routing chart and service deployed** (in flight, platform-routing#13).

**Apps**

- **A1 — the landing resolves by host** (`web-paridyum`): a `paridyum_app` pool, `resolve_surface(host,
  'paridyum')` cached with the tenant in the key, `landing_sections` and theme or H7's defaults,
  404 on no row, canonical redirect. Paridyum's own content moves into Paridyum's rows (needed by S8).
- **A2 — careers resolves by host** (`web-hiring`, T-G.12): `tenantForRequest(host)` through
  `resolve_surface(host, 'hiring')`, the proxy writes the tenant segment, `basePath: "/careers"`,
  `PARIDYUM_ZITADEL_ORG_ID` removed.
- **A3 — careers signs in with the tenant's client:** issuer, project and client from the resolved
  surface per request; the login host per H5; redirect URIs as the deployment catalogue registers them.
- **A4 — the dashboard** (`web-admin` under `basePath: "/dashboard"` until `web-console`): host
  resolution, the tenant's client for `docket`, and docket's screens behind the `operator` role.
- **D1 — docket** cross-checks the token's tenant against `Host`'s (`resolve_surface(host, 'docket')`),
  and gives a tenant with no configuration rows its defaults (H7).

**Saga and catalogue**

- **P1 — the request carries the requester's verified address** (migration, regional plane; the
  portal has it from introspection) and the catalogue declares `operator` on hiring.
- **P2 — `owner_access`** (H6), in the deployment service or the saga per R3's finding, with
  compensation and a mock-Zitadel test.

**Deploy and prove**

- **X1 — deploy** tenancy (with `SAGA_*`), routing, deployment; the landing, careers and admin
  redeployed with their new environment.
- **X2 — the end-to-end test** (§6), scripted, so it is rerun after every slice above lands.

## 6. The test (scripted, `infra-kubernetes/scripts/prove-tenant-hosts.sh` or equivalent)

1. Create two synthetic people on the portal instance (verified addresses, admin API).
2. Each: `POST /v1/orgs`, then `POST /v1/orgs/{id}/tenants` with products `paridyum, hiring, docket`
   and own domain `tenant-a.paridyum.com` / `tenant-b.paridyum.com` (H2).
3. Write each tenant's `_paridyum-verify` TXT record (`dns/apply.py`), wait for the saga's `active`,
   then call routing's verify; the binding is `domain_active` and canonical.
4. `curl` both hosts on `/`, `/careers`, `/dashboard`: each shows its own tenant; a third, unbound
   host returns 404.
5. Browser (Playwright on sitar): T1's owner signs in with "Continue with Paridyum", posts a job at
   `/careers` admin, sees it publicly on T1's host and not on T2's; T2's owner is refused T1's admin.
6. A candidate signs in at T1's careers and applies; the application is under T1 only (RLS).
7. Compensation drill: a third request with a forced failure ends `failed`, leaving no binding, no
   active client, no live grant.

## 7. Order and estimate

| Step | Depends on | Size |
| --- | --- | --- |
| Merges in flight, X1 (deploy the three services) | CI | hours (CI-bound) |
| R1, R2, R3 | — | half a day |
| H1–H8 written as ADRs/amendments, operator's reading | R1, R3 | a day, plus the operator's time |
| E1, E2, R-a | H2, H3, H8 | a day |
| A1, A2, A3 | E1, R-a, H5 | two to three days |
| P1, P2 | H6, R3 | a day |
| A4, D1 | A3 | a day to two |
| X2 green | all above | half a day |

**About a week of work** after the decisions, most of it in the apps. H1 (buying the zone) is the one
operator action that removes work rather than adding it.

## 8. What the operator decides

- **H1:** buy the app zone (name to confirm) — recommended now.
- **H2:** done — `tenant-a/-b.paridyum.com` are live at the edge (infra-edge#8).
- **H5:** if R1 fails, accept interim sign-in at `auth.paridyum.com` for new tenants, time-boxed.
- **H6:** the saga creates the owner's linked user and grants — recommended.
- Everything else here follows from accepted artifacts and is the build's to carry.

## 9. Status (2026-10-03)

| Slice | State | Where |
| --- | --- | --- |
| E1, E2 | **Live** for `tenant-a/-b.paridyum.com` | infra-edge#8 (applied) |
| Paridyum's own paths | Prepared, **not applied** | infra-edge#9 |
| R-a, R-b | 0025 in review (platform-tenancy#58); routing **merged** (#8–#13) | |
| Deployment service | **Merged** #12–#16; #17 (hiring declares `operator`) merging | platform-deployment |
| A1–A4, D7 in the apps | **Merged**: landing, careers, dashboard; canonical-host redirects | web-paridyum#5–6, web-hiring#11–15, web-admin#5–8 |
| Saga, P1, P2 | Merging: #38–#53 **merged**, #54–#62 in CI | platform-tenancy |
| Staging config for the saga | In review: Secret keys, `web-admin-secret`, SAGA_* values | infra-kubernetes#26, platform-tenancy#63 |
| Clients' new origins | **Merged**, not applied (`make zitadel-apps-repair`) | infra-kubernetes#25 |
| D1 | Step 2 (no git-registry fallback for a tenant) and step 1 (tenant tokens) in review; step 3 (screens) open | tool-docket#36, #37 |
| X2 | Portal token for a synthetic person **works live** | infra-kubernetes#27 |

**D1 is decided; it is not yet built.** *Correction, 2026-10-03: an earlier version of this section called D1
"a plane question" needing a decision. ADR-0127 D2 (accepted) already decides it.* The console relays the
user's own token to docket's API, through **that tenant's docket client**, so docket accepts tenant-plane
tokens for the tenant's own docket project. Its one-plane pin in `auth.rs` (control plane,
`paridyum-operations`) is code that predates ADR-0127, not a rule standing in the way. What D1 still
needs, in order:

1. **A second verifier path in docket.** The tenant instance's keys. The audience is the docket project
   that `resolve_surface(<forwarded host>, 'docket')` names, and the token's organization must be that
   surface's. `operator` comes from that project's roles.
2. **No git registry for a tenant that is not Paridyum.** ADR-0099 D2's fallback to the git registry is
   Paridyum's own configuration; another tenant falling back to it would see Paridyum's companies and
   signatories. Until the per-tenant tables exist, a tenant without rows gets an empty default
   configuration (H7).
3. **The dashboard's docket screens,** relaying the token (ADR-0127 D2 and D3: `console-docket`'s code,
   moved).
   - **3a, the relay itself: in review** (web-admin#9). A tenant's site keeps its member's token,
     renews it through the site's own client, and `lib/docket.ts` calls docket with it and the host.
   - **3b, the screens: waits on a design call.** console-docket carries its own 437-line Tailwind
     theme (`ink`, `rule-strong`, `paper-raised` and so on) and does not use `@paridyum/ui`, which the
     dashboard is built on. "Moved, not rewritten" therefore needs a choice: bring console-docket's
     theme into the dashboard, or restyle the screens onto `@paridyum/ui` (STD-0015). Either is about
     6,500 lines over roughly 16 PRs.

**The finish line still does not need D1:** the dashboard's operator check uses the token web-admin
verifies itself (A4c).

**The deploy window (X1), in order:**

1. `make zitadel-apps-repair APP=web-hiring` and `APP=web-admin`.
2. The `database_url` key in the `web-admin` and `web-paridyum` Secrets: the same value as web-hiring's.
3. Migration 0025, then tenancy (with `SAGA_*`), routing and deployment.
4. The images: landing, careers, dashboard.
5. `make configure` for infra-edge#9, then `make edge-cache-purge`.
6. X2.

