---
id: RFC-0043
title: web-careers is carved out of web-paridyum in four independently revertable slices
type: rfc
status: proposed
created: 2026-09-08
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, web, infra]
applies_to: ["web-paridyum", "web-careers"]
tags: [hiring, careers, web-careers, web-paridyum, adr-0074, adr-0070, adr-0073, adr-0068, adr-0046, adr-0020, adr-0061, c-0002, c-0010, anti-0007, migration]
breaking: true
enforced_by: []
---

# RFC-0043 — `web-careers` is carved out of `web-paridyum` in four independently revertable slices

## Problem

One deployable serves two populations with two risk profiles.

`web-paridyum` currently holds the estate's public marketing pages *and* the routes that
authenticate a job seeker and write their date of birth, institution and legal name to a
classified store: `[lang]/profile`, `api/profile`, `api/apply`, `api/auth/*`.

Observably, this means:

- **Every content edit is a release of the surface that holds regulated personal data.** A
  locale string change, a copy fix or a weekly dependency bump ships the PII-writing code path.
  This is the coupling REQ-0003's classification work does not model and cannot see.
- **A compromise of the marketing surface yields a candidate session and the applications
  table.** ADR-0068 §3 established that a careers-site compromise must not reach the personnel
  record; it did not extend the reasoning to the marketing site reaching the applications.
- **The blast radius grows with the front door's change rate**, which is the highest in the
  estate and will stay highest, because marketing content is what changes most.

Who experiences it: every future candidate, on first contact with the estate; and the operator,
at the moment of any incident on the front door.

If nothing is done, the coupling accumulates. It already has — six candidate-shaped routes
arrived on the front door one individually-correct step at a time.

**Fixed when:** no code path reachable from a `web-paridyum` request can read or write a column
carrying `paridyum:class: pii` or `confidential`, and the two surfaces deploy, roll back and
rotate credentials independently.

## Constraints

| Artifact | What it binds here |
| --- | --- |
| **ADR-0074** | The decision this implements. Index stays on the root domain; profile, apply and the candidate session move to `careers.paridyum.com`. `[lang]/account` stays. Outbound link, never an iframe or a reverse-proxy path. |
| **ADR-0068 §5** | **The gate.** Schema, RLS, both surfaces and this whole migration may be built against synthetic data. No real applicant until an off-host backup destination exists and a restore has been rehearsed. |
| **ADR-0073** | The candidate profile's field set, `paridyum:class` directives, retention clock and delete button. Not reopened — this RFC moves where that code deploys. |
| **ADR-0070 §1** | Two deployables, two OIDC clients, never shared; staging and production are separate applications. Four registrations. |
| **ADR-0070 §2** | Both clients sit in project `paridyum-hiring`. A different population on `[lang]/account` would be a new project and a new ADR. |
| **ADR-0046** | TypeScript and Next.js. `web-careers` is the same stack, not an opportunity to try another. |
| **ADR-0020** | A `deployable: true` class-`web` repo owes probes, resource limits and a Helm chart at creation, not deferred. |
| **ADR-0024, ADR-0038** | One Postgres; RLS `ENABLE`d and `FORCE`d; session context set per transaction from a verified Zitadel token, failing closed. |
| **ADR-0061** | Both hostnames terminate at the rented tier, then our node. `careers.paridyum.com` is one more name on an existing path, not a new tier. |
| **C-0002** | Repository name `web-careers`. |
| **C-0010** | There is no prototype tier. `web-careers` is the production careers surface from its first commit. |
| **ANTI-0007** | Regulated data before controls. The classified store moves *with* its classification directives and its retention entries, or it does not move. |
| **ADR-0022 / registries** | `web-careers` needs entries in `class-registry.yaml`'s consumers, `product-registry.yaml` under `tenancy`'s dossier, and a `manifest.yaml`. |

Nothing here contradicts a locked artifact; ADR-0074 is `accepted` and this RFC is its
implementation, not its supersession.

## Proposal

### Slice 1 — `web-careers` exists, deployable and empty

New repository, `class: web`, `stack: typescript`, `lifecycle: experimental`, mirroring
`web-paridyum`'s skeleton: Next.js App Router, `[lang]` segment and the same `messages/`
locale shape, `healthz`/`readyz`, Helm chart, Dockerfile, vitest, CI.

Serves one page — a placeholder index at `/[lang]` — and answers its probes. No auth, no
database, no candidate routes. `manifest.yaml` declares `deployable: true` and, like
`web-paridyum`, **carries no `slo` block** for RFC-0016's reason.

Revertable by deleting a repository nothing points at.

### Slice 2 — the OIDC registrations split

Four Zitadel Web applications in project `paridyum-hiring`, per ADR-0070 §1:

| Application | Surface | Environment |
| --- | --- | --- |
| `web-paridyum` (staging) | `[lang]/account` | staging |
| `web-paridyum` (production) | `[lang]/account` | production |
| `web-careers` (staging) | candidate session | staging |
| `web-careers` (production) | candidate session | production |

Each with its own `client_id`, its own redirect allowlist containing exactly one hostname, and
Authorization Code + PKCE per ADR-0070 §3's `web` row. `web-paridyum` is re-pointed at its new
client and `[lang]/account` re-verified before anything moves; the existing client is retired
rather than reused, because a client whose allowlist once held both hostnames is the typo
ADR-0070 §1 warns about, already made.

Revertable by re-pointing `web-paridyum` at the old client.

### Slice 3 — the candidate routes move

`[lang]/profile`, `api/profile` and `api/apply` are deleted from `web-paridyum` and created on
`web-careers`, against the `web-careers` client. `api/auth/{login,callback,logout}` is
**copied**: `web-careers` gets its own set, `web-paridyum` keeps its set for `[lang]/account`.

The database access moves with them — the ADR-0038 session-context accessor, the RLS-scoped
queries, and ADR-0073's classification and retention entries travel as one unit with the code
that uses them, which is ANTI-0007's requirement rather than a tidiness preference.

`web-paridyum` loses its database dependency entirely in this slice. That is the observable
signal that the carve-out worked: a front door with no connection string.

Revertable, but this is the slice where revert is a restore rather than a `git revert` if it
lands after the ADR-0068 §5 gate opens. **It must land before.**

### Slice 4 — the seam is a link, and the front door is fenced

`[lang]/careers/[id]` grows an *Apply* control that navigates to
`careers.paridyum.com/{lang}/apply/{id}` — a plain outbound navigation, per ADR-0074 §4.

`web-paridyum` sets `frame-ancestors 'none'` and `web-careers` sets `frame-ancestors 'self'`,
so neither can be embedded in the other even by mistake. No reverse-proxy path on
`paridyum.com` resolves to the careers surface; the edge config is asserted, not assumed.

Job data for the index is read **at build time** from the same source, never per-request with
a credential (ADR-0074 §1).

`web-paridyum`'s `manifest.yaml`, `CONTENT.md`, `ARCHITECTURE.md` and `README.md` stop
describing candidate flows. Both repositories get a `LOGBOOK.md` entry and `web-careers` gets
an `SOP.md` at completion.

### The check that graduates ADR-0074 to `locked`

Proposed for `check-registry.yaml`, owner `check-repo.sh`:

```
web.surface_without_candidate_population_touches_no_classified_column
  severity: error
  For a class-web repo whose manifest declares no candidate population, fail on any
  source path referencing a table or column that fields.yaml classifies pii or
  confidential, and on any connection string in its deployment config.
```

This asserts ADR-0074 §5's *store* half mechanically. The *session* half stays a review
matter, honestly, because "did `[lang]/account` stay small" is a judgement.

## Alternatives considered

**One big-bang PR.** Loses on revertability. Slice 3 alone touches the auth path, the database
path and two repositories; a single PR mixing repository creation, four OIDC registrations,
route moves and edge configuration cannot be reverted in pieces when one of the four is wrong,
and the OIDC half is the half most likely to be wrong on first attempt.

**Move the routes first, split the OIDC clients later.** Cheaper to sequence and rejected
because it means `web-careers` runs in production against `web-paridyum`'s `client_id` for
some interval — the exact shared-client state ADR-0070 §1 forbids, adopted deliberately and
called temporary. Temporary shared credentials are how permanent ones start.

**Move the whole careers area including the listings.** Rejected in ADR-0074's own alternatives
on SEO grounds; restated here only because it is the shape most likely to be re-proposed by
someone reading the repository name `web-careers` and assuming it owns careers.

**Keep one repository, two Next.js apps in a pnpm workspace, deployed as two images.** This
satisfies the letter of the split — two deployables, two clients, two blast radii — at less
repository overhead, and it genuinely was close. Rejected on ADR-0001: the estate is a
polyrepo, `check-repo.sh` and the manifest system are per-repository, and a workspace with two
manifests is a shape no check in the registry understands. Not wrong in principle; wrong
against this estate's tooling.

**Use a third-party ATS instead.** Rejected in ADR-0074 on residency. Restated as a constraint,
not reconsidered here.

## Consequences

**What this makes worse.**

- **Two repositories to keep patched, released and green**, doubling ADR-0020's per-repo
  obligations for a hiring volume currently zero. This is C-0010 being paid honestly: there is
  no prototype tier, so the second surface is production-grade from commit one.
- **Four OIDC registrations to keep correct**, and ADR-0070 §1's staging/production hazard now
  has four places to occur rather than two.
- **Listings and applications can drift.** Build-time job data means a role closed on
  `web-careers` stays listed on `paridyum.com` until the next build of the front door. A
  closed-role guard on the application surface is required so the drift produces a clear
  message rather than a submitted application to a closed posting.
- **`api/auth` is duplicated, not shared.** Two copies of the same OIDC round trip, which will
  diverge. A shared `lib-` is the eventual answer and is deliberately not slice 5 — extracting
  a library from one copy is a guess, from two copies it is evidence.
- **Migration on a green surface.** `web-paridyum` currently passes CI and prerenders both
  locales; this breaks that state deliberately for the duration of slices 3 and 4.
- **`careers.paridyum.com` starts at zero SEO authority.** Accepted in ADR-0074; the mitigation
  is decision 1 keeping the indexable half on the root domain.

**What this makes better.** Stated in ADR-0074's consequences; not restated.

## Rollout

**Order is fixed:** 1 → 2 → 3 → 4. Slice 2 must precede slice 3 so no candidate route ever runs
against a shared client. Slice 4 must follow slice 3 so no link points at a route that has not
arrived.

**Backwards compatibility.** There are no real users and no real applicant rows — ADR-0068 §5
has not opened — so there is no data migration and no compatibility window. **This is the
cheapest this change will ever be, and that is the argument for doing it now rather than
after the gate opens.**

**Undo.** Slices 1, 2 and 4 revert cleanly. Slice 3 reverts by restoring the deleted routes to
`web-paridyum` and re-pointing them at the old client, which is a `git revert` only while the
store holds synthetic data.

**The gate does not move.** No slice here counts as progress toward ADR-0068 §5, and none may
be reported as such. The first real applicant still waits on an off-host backup destination and
a rehearsed restore — which ADR-0068 §6 records as itself unresolved.

## Open questions

1. **Does `web-careers` need its own locale set, or does it import `web-paridyum`'s
   `messages/`?** Importing means a shared package and a dependency between two things this
   RFC just separated; duplicating means translations drift. **Settled on inspection as
   duplicate-then-extract, and the reason is stronger than symmetry with `api/auth`:**
   `src/lib/locale.ts` documents `LOCALES` as a *security floor* — CVE-2026-64642 makes a
   single-entry locale array a proxy-check bypass on App Router with Turbopack (RFC-0028). That
   array is a build-time and proxy-path constant. It cannot become a runtime lookup, and
   whatever holds the message bundles must ship inside the build that renders them.
2. ~~**What is the job data source of record?**~~ **Answered on inspection, 2026-09-08.** It is
   already settled and already built: `web-paridyum/jobs/`, read at build time by
   `src/lib/jobs.ts`, per ADR-0063 decision 2 — *"a JD is declared data reviewed and merged like
   every other registry in this estate, and the repository is the admin surface for it."*
   RFC-0041 freezes the `<slug>-<year>` identifier at publication. `web-careers` reads the same
   directory the same way, so slice 4's build-time constraint is the existing behaviour rather
   than a new rule. **The open part is narrower than stated:** which of the two repositories
   holds `jobs/`, given both now read it and ADR-0001 gives them no shared build.
3. **Does the rented tier terminate `careers.paridyum.com` as a second name on the existing
   certificate, or its own?** ADR-0061 does not say, and it is an edge-config question with a
   cost either way.
4. **Where does `[lang]/account` go long term?** ADR-0074 §3 fences it to the
   `paridyum-hiring` population. The first non-candidate signed-in user forces a project
   decision, and it is better to know now what that user would be than to discover it.
5. **Does the closed-role guard belong in slice 4 or its own slice?** It is the one piece of
   genuinely new behaviour in a migration RFC, which is usually a sign it should be separate.
