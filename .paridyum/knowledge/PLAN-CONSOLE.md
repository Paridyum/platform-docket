# Plan — the tenant console, `web-console`

*Written 2026-10-10. Details **ADR-0143**, which names the console's repository and its shape, and
carries **ADR-0128 D4** and **PLAN-TENANT-FLOW D4**. `PLAN-PARIDYUM-IN.md` stays the authority for the
sprint's order and `TASKS.md` for task state; this document answers **"what stands between the console
the estate designed and a tenant's people using it on their own host"**, from what the estate is on
this date, read rather than assumed.*

## 0. The finish line

A tenant's own people, on the tenant's canonical host, with no hand step:

1. `https://<host>/dashboard` is **that** tenant's console — its enabled products, and nothing of
   another tenant's.
2. Its docket screens show **that** tenant's roster; an operator enrols a person there, and the write
   lands under that tenant alone.
3. Sign-in is with **that tenant's own client**; a session minted on one tenant's host is refused on
   another's.
4. An **unknown host answers 404**, never Paridyum's console; a tenant's non-canonical host answers 308
   to the canonical one.
5. `admin.paridyum.com` is a **permanent redirect** to `paridyum.com/dashboard`.
6. The portal (paridyum.in) shows the org's businesses and **links into** each one's console, and never
   renders a tenant's insides.

> **Status 2026-10-10:** **C1 has landed** — `web-admin` is renamed `web-console` across the
> estate (repository, package, manifest, chart `console`, image `ghcr.io/paridyum/web-console`,
> `PARIDYUM_CONSOLE_SITE_URL`, the OIDC client, and the live cross-references). C2–C5, E1 and K1
> are open.

## 1. Where the estate stands (read 2026-10-10)

| Piece | State | Source |
| --- | --- | --- |
| `web-admin` **is** the host-resolved console | **In code**: `/dashboard`, `resolve_surface(host,'docket')`, per-tenant client, canonical 308, docket token relay | `web-admin` #5–#9 |
| `web-admin` deployed | **Once**, on `admin.paridyum.com` (VIP `.104`); `web-admin` OIDC client with redirects for `admin.paridyum.com` and `paridyum.com/dashboard` | `zitadel/applications.yaml` |
| `web-admin`'s name and manifest | **Stale**: *"tenant-admin surface … BFF in front of platform-identity"*, `product: [identity]`, `depends: [platform-identity]` | `manifest.yaml` |
| `console-docket`'s screens | **Not moved in**; the relay (`lib/docket.ts`) is, the screens are not | `web-admin/src/lib`, `console-docket/src/app` |
| `paridyum.resolve_surface(host, product)` | **Live** (migration 0020); returns tenant, account, Zitadel org, project, client; unknown host → no row | ADR-0126 D3 |
| The app zone | **Bought**: `paridyum.xyz` (ADR-0136) | |
| Per-tenant clients | **Created by the saga** (ADR-0126 D4); Paridyum's are recorded | ADR-0126, ADR-0128 D6 |
| Edge: a tenant host's `/dashboard` | **Not routed** — the edge knows hosts, not paths, and has no catch-all | PLAN-TENANT-HOSTS §2 gap 1 |
| The portal's link into a console | **Built**: `dashboardUrl(t)` → `https://<default_host>/dashboard` | `web-dashboard/src/lib/portal.ts` |
| ADR-0118 D4's open question | **Answered by ADR-0143**: `web-admin` keeps a role on tenant hosts, and is renamed `web-console` | ADR-0143 |

## 2. The gaps, in the order they block

1. **The console has the wrong name and a description its code has outgrown** (`web-admin`), so no
   reader or registry agrees with what it is.
2. **Its manifest declares the wrong product and dependencies**, so the estate's own model of what the
   console calls is wrong.
3. **A tenant's docket screens are not in it** — only the relay is.
4. **Nothing routes a tenant host's `/dashboard` to it** (PLAN-TENANT-HOSTS E1 owns the edge catch-all).
5. **`admin.paridyum.com` still serves a second copy** of a surface that must have one origin.
6. **No end-to-end proof exists** that a synthetic tenant's console is that tenant's alone.

## 3. The slices (each one PR, ≤ 400 lines, tested, a mutant watched failing — STD-0007)

Each slice is **vertical** and its sentence has no "and". Slices live in the repository being changed
(STD-0007 §1); this document is the plan, not the slice files.

### `web-console` — the repository

- **C1 — The repository is `web-console`, completely.** Rename `web-admin` → `web-console` (STD-0022's
  checklist: directory and repo, `package.json`, `manifest.yaml`, `AGENTS.md`/`README.md`/`CHANGELOG.md`/
  `LOGBOOK.md`, the Zitadel application and its redirect URIs, `address-registry.yaml`, and every live
  cross-reference in knowledge — ADR-0086, ADR-0089, ADR-0095, ADR-0098, ADR-0099, ADR-0107, ADR-0113,
  ADR-0118, RFC-0048, RFC-0049, RFC-0050).
  - *Done when*: `git grep web-admin` over live artifacts is empty, and the repo builds, tests and
    deploys as `console`.
  - *Out of scope*: the Zitadel client **deletion** (K1), the deploy host (E1).
  - *Budget*: mechanical (a rename) — **`Review-budget-exception:`** with the mechanical reason.

- **C2 — The console's manifest declares the services it actually calls.** `product: [docket, paridyum]`,
  `depends: [platform-identity, tool-docket]`, and a `provides.capability` naming the host-resolved
  tenant console.
  - *Done when*: the manifest's `product` and `depends` match the code's calls, and the capability
    sentence describes the console rather than a BFF in front of one service.
  - *Out of scope*: adding `platform-tenancy` — the console reaches `resolve_surface`, a database
    function, not tenancy's API.

- **C3 — A tenant's operator sees their own docket roster at `<host>/dashboard`.** Move
  `console-docket`'s read screens (the roster, a person, their emails, retention) into the console, over
  the existing token relay.
  - *Done when*: a signed-in operator on a synthetic tenant's host sees that tenant's roster and not
    another's; a person's page renders that tenant's data alone.
  - *Out of scope*: the write path (C4); the standalone `console-docket` deploy, which is retired
    (ADR-0127 D1).

- **C4 — A tenant's operator enrols a person from `<host>/dashboard`.** Move `console-docket`'s write
  screens (enrol, issue, record a signature, advance) in, relaying the operator's own token.
  - *Done when*: an enrolment made from a tenant's host is readable under that tenant alone, and a
    refusal is rendered with its reasons and no override.
  - *Out of scope*: docket's own refusal rules, which live in `tool-docket`.

- **C5 — The console is proven fail-closed.** An unknown host, an unverified binding, and a product the
  tenant has not enabled each answer 404 and never render Paridyum's console; a session minted on one
  tenant's host is refused on another's.
  - *Done when*: the negative is the assertion — a host with no row gets nothing, and a cross-host
    session is refused — and it is watched failing against a deliberately broken lookup.
  - *Out of scope*: `resolve_surface` itself, which ADR-0126 D3 owns and migration 0020 tests.

### `infra-edge` — routing a tenant's `/dashboard`

- **E1 — Every tenant host serves its console at `/dashboard`.** The tenant catch-all's `location`
  sends `/dashboard` to the console VIP, passing `Host` and `X-Forwarded-*` through (PLAN-TENANT-HOSTS
  H3, E1).
  - *Done when*: a request to a synthetic host's `/dashboard` reaches the console and its `/` reaches
    the landing, and an unlisted zone gets nothing.
  - *Out of scope*: certificates for own domains (PLAN-TENANT-HOSTS H4); the landing and careers paths
    (their own slices).

### `infra-kubernetes` — one origin

- **K1 — `admin.paridyum.com` is a permanent redirect to `paridyum.com/dashboard`.** The old host
  answers 308, and the `web-admin` OIDC client is retired (STD-0022: the old name stops existing).
  - *Done when*: `https://admin.paridyum.com` answers a permanent redirect to the path, and no client
    named `web-admin` remains in `applications.yaml`.
  - *Out of scope*: the console's own client, which C1 renames.

## 4. Order and estimate

| Step | Depends on | Size |
| --- | --- | --- |
| C1 rename | — | half a day (mechanical) |
| C2 manifest | C1 | an hour |
| C3 docket read screens | C1, the tenant host serving (E1, PLAN-TENANT-HOSTS) | a day |
| C4 docket write screens | C3 | a day |
| E1 edge `/dashboard` | PLAN-TENANT-HOSTS E1 | half a day |
| K1 admin redirect | C1 | an hour |
| C5 fail-closed proof | C3, E1 | half a day |
| Deploy and prove end to end | all above | half a day |

**About three days of work**, most of it moving docket's screens; C1 and K1 are mechanical.

## 5. What the operator decides

- **The rename is the one call to confirm** (ADR-0143 D1): `web-admin` → `web-console`, a complete
  rename, rather than keeping the interim name. It is the estate's own name for the console
  (ADR-0128 D4) and it retires `admin.paridyum.com`; the alternative is an amendment to keep
  `web-admin`.
- **The identity screens stay ADR-0128 D4's deferred question** — whether a tenant's own members and
  roles are managed in the console or the portal — and are not built here.
- Everything else follows from accepted artifacts and is the build's to carry.
