---
id: ADR-0074
title: The careers index stays on the main site and the application surface is a separate deployable
type: adr
status: superseded
created: 2026-09-08
locked_at: null
supersedes: null
superseded_by: ADR-0086
scope: [org, platform, web]
applies_to: ["*"]
tags: [hiring, careers, web-paridyum, web-careers, adr-0068, adr-0070, adr-0073, adr-0046, adr-0061, adr-0067, c-0002, surface, blast-radius, seo]
breaking: true
enforced_by: []
sources:
  - url: https://roleworth.com/guides/greenhouse-vs-lever-vs-ashby
    title: Greenhouse vs Lever vs Ashby — what job seekers actually experience
    retrieved: 2026-09-08
    supports: >
      The three dominant applicant tracking systems all host the application form on their
      own origin — job-boards.greenhouse.io/{company}, jobs.lever.co/{company},
      jobs.ashbyhq.com/{company} — and the employer's own /careers page links or embeds into
      it. The marketing site indexes; a different origin takes the application. This is the
      split this ADR adopts, arrived at independently by every vendor in the category.
  - url: https://www.cmu.edu/web/best-practices/search-engine-optimization/subdomains-vs-subdirectories.html
    title: Subdomains vs subdirectories — Carnegie Mellon web best practices
    retrieved: 2026-09-08
    supports: >
      Search engines treat a subdomain as a separate site with its own authority and its own
      backlink profile to build from zero, while a subdirectory inherits the parent domain's.
      This is the fact that decides which half of the split keeps the root domain: the index
      is the half with something to rank, so the index is the half that stays.
  - url: https://support.greenhouse.io/hc/en-us/articles/46365908766875-Embed-a-Greenhouse-job-board-on-your-career-site
    title: Embed a Greenhouse job board on your career site — Greenhouse Support
    retrieved: 2026-09-08
    supports: >
      Greenhouse's own documented mechanism for keeping candidates on the employer's site is
      an iFrame embedding the vendor-hosted board. Decision 4 rejects this shape: an iframe
      puts a credentialed, PII-collecting origin inside a page served by the marketing
      surface, which reunites in the browser the two surfaces this ADR separates.
---

# ADR-0074 — The careers index stays on the main site, and the application surface leaves it

## Status

**Accepted**, 2026-09-08. Not `locked`: the rule is mechanically checkable — a route under
`src/app/**` on `web-paridyum` that reads a session or writes a `pii` field is exactly the
defect — but no check asserts it yet, and ADR-0002 reserves `locked` for what the estate can
reject rather than what it intends. It graduates when `check-registry.yaml` carries a check
that fails on an authenticated route in a surface whose manifest declares no candidate
population.

## Context

`web-paridyum` is the estate's front door. Today it also holds
`src/app/[lang]/careers`, `src/app/[lang]/careers/[id]`, `src/app/[lang]/profile`,
`src/app/[lang]/account`, `src/app/api/apply`, `src/app/api/profile` and
`src/app/api/auth/{login,callback,logout}`.

Six of those routes exist to serve one population — job seekers — on a surface whose
declared purpose in `manifest.yaml` is that it *"becomes `production` when the first enquiry
arrives through it"*. **The front door has quietly become an applicant tracking system with a
company page attached,** and the sequencing that produced it is legible: sign-in landed first
(ADR-0038, ADR-0070), then the flow (ADR-0068), then the profile (ADR-0073), each correct in
isolation and each adding a candidate-shaped route to the only web surface that existed.

**`[lang]/account` is not one of the six, and the distinction is the whole shape of this
decision.** That page renders `session.name` and nothing else; its own comment states its
purpose — *"it exists to prove the round trip: no session, no page"*. The front door
authenticates, it will keep authenticating, and this ADR does not take that away. What it
takes away is the **store**: a signed-in screen that reads a display name from a token is a
different risk object from a form that writes a date of birth to a classified column, and an
ADR that conflates them would be removing the wrong thing.

The pressures genuinely compete, and the ADR that only lists one side of them is a
rationalisation.

**Against splitting.** ADR-0070 §1's own test — *"if two things can be deployed, revoked or
rotated independently, they are two applications"* — cuts both ways, and a candidate today
signs in once and moves between listing, profile and apply as one flow on one session. Split
carelessly and that becomes a cross-origin session problem the estate does not have. A second
`web-` repository is also a second Helm chart, a second image, a second set of probes and a
second CI pipeline, under ADR-0020, for an estate that has not yet served its first visitor.

**For splitting.** Three things, and the third is the one that decides it.

*Blast radius.* ADR-0068 §3 was careful that a compromise of the careers site yields
applications and not the personnel record. That reasoning stops one seam too early. The
marketing surface is the most-changed, most-publicly-linked, most-dependency-churning artifact
in the estate — content edits, locale additions, an npm tree that moves weekly — and it is
currently the same deployable that holds a signed-in candidate's date of birth. Every content
change is a release of the PII surface, and REQ-0003's classification work is undone by
deployment coupling it never modelled.

*Population.* ADR-0070 §2 already refused to put candidates and operators in one Zitadel
project, on the ground that they are two populations with two role vocabularies. The identical
argument applies one layer out and was not made there: **an anonymous visitor reading about the
company and an authenticated candidate holding a `paridyum-hiring` token are two populations,
and they are being served by one deployable.** `web-paridyum` holds a client for a project
whose population it mostly does not serve.

*The industry arrived here independently.* Greenhouse, Lever and Ashby — three vendors, three
codebases, no shared constraint — all host the application on their own origin and leave the
employer's `/careers` page as the index that links into it. Amazon runs `amazon.jobs`, Meta
runs `metacareers.com`, Apple runs `jobs.apple.com`. Google is the instructive exception: it
keeps careers as a *subdirectory* at `google.com/about/careers`, which is the same split
resolved the other way on the URL question while still being a separately operated surface.
The convergence is not on the URL. **It is on the surface: nobody ships the application form
in the same deployable as the marketing site.**

The SEO literature explains why the URL question splits while the surface question does not.
A subdomain starts with its own authority and its own backlink profile to build from zero; a
subdirectory inherits the root domain's. That is a real cost, and it lands entirely on the
half of the split that has something to rank — the job listings. It lands on the application
form not at all, because a form behind a login has nothing to rank and wants no traffic it did
not send itself.

## Decision

### 1. `web-paridyum` keeps the careers *index*, and keeps its own signed-in surface

`/[lang]/careers` and `/[lang]/careers/[id]` stay on the root domain, as `paridyum.com/careers`.
They are public, unauthenticated, cacheable marketing content — role descriptions, team, what
it is like to work here — and they are the half of the split that benefits from the root
domain's authority, per the subdirectory argument above.

They may read job data at build time from the same source the application surface uses. They
may not read it per-request with a credential.

`/[lang]/account` also stays, along with the `api/auth` routes that make it work. It is the
front door's own signed-in surface — today a name and a sign-out, later whatever the front
door legitimately owes a signed-in visitor. **It is not the candidate profile and must never
grow into one:** its ceiling is decision 5, not its current line count.

### 2. The application and the candidate profile move to `web-careers`, a separate deployable

A new repository, `web-careers`, class `web` under `class-registry.yaml`, named per C-0002.
It owns what leaves `web-paridyum`:

| Leaves `web-paridyum` | Becomes |
| --- | --- |
| `src/app/[lang]/profile` | the candidate profile of ADR-0073, unchanged in field set |
| `src/app/api/profile` | its writer |
| `src/app/api/apply` | the application submission of ADR-0068 §2 |
| `src/app/api/auth/{login,callback,logout}` | **copied, not moved** — `web-careers` gets its own, against its own client (decision 3). `web-paridyum` keeps its set for `[lang]/account`. |

**ADR-0073 is not reopened.** Its field set, its `paridyum:class` directives, its retention
clock and its delete button are the specification of what `web-careers` stores. This ADR moves
where that code is deployed, not what it collects.

### 3. Its origin is `careers.paridyum.com`, and it gets its own OIDC client

A subdomain, not a separate registrable domain. ADR-0067 fixes a person's address on the root
domain and the estate has one brand; `amazon.jobs` is a decision for an employer with a hiring
volume this estate will not have for years, and a second registrable domain is a second
certificate story, a second reputation to establish and a second thing to renew.

Under ADR-0070 §1 this is a second deployable surface, so it is a second Zitadel application
with its own `client_id` and its own redirect URIs — staging and production separately again.
It sits in project `paridyum-hiring`, which is the project ADR-0070 §2 created for exactly this
population.

**`web-paridyum` keeps its own client, and the two are never the same client.** ADR-0070 §1
forbids sharing a `client_id` between deployables — *"if two things can be deployed, revoked or
rotated independently, they are two applications"* — and after this ADR they demonstrably can
be. So: two Web applications, two `client_id`s, two redirect allowlists, and staging and
production split again within each. Four registrations where there is one today.

**Both sit in `paridyum-hiring`, and that is deliberate rather than lazy.** ADR-0070 §2 makes a
project a *role vocabulary and an audience*, and notes that all applications within a project
share roles. The humans who sign in at `paridyum.com/account` today are the same population as
the ones who sign in to apply — candidates, and staff acting on applications. One population,
one role vocabulary, two surfaces. That is exactly the case §1 solves with separate clients and
§2 does *not* require a separate project for.

This is the seam to watch. If `[lang]/account` ever serves a population that is not
`paridyum-hiring`'s — a customer, a partner, a tenant admin — that is a new project under
ADR-0070 §2 and a new ADR, not a role added to this one.

### 4. The link between the two is an outbound link — never an iframe, never a reverse-proxy path

Greenhouse's documented answer to *keep the candidate on your site* is an iFrame. This estate
does not take it.

An iframe puts a credentialed origin that collects `pii` inside a document served by the
marketing surface, and the browser is then the place where the two surfaces are joined back
together — a `frame-ancestors` mistake, a clickjacked consent, or one relaxed CSP directive
away from the separation being decorative. A path on the root domain reverse-proxied to the
careers surface is the same defect with the seam moved to the edge, and ADR-0068 §1 already
rejected the equivalent move for docket in the sentence *"a boundary with one hole in it is a
boundary someone has to remember the shape of."*

Applying to a role is a navigation to `careers.paridyum.com`. The candidate sees the origin
change, which is the property that makes the boundary real to the only party a phishing attack
targets.

### 5. `web-paridyum` may hold a session; it may not hold a classified store

The line is the store, not the session. Stated so a check can be written against it and a
reviewer needs no interpretation — on `web-paridyum`:

- **Permitted.** Reading the session cookie, and rendering claims the token already carries
  (name, subject, locale). ADR-0038 puts the account and its email on the identity's side of
  the boundary, so displaying them stores nothing.
- **Forbidden.** Any code path that reads or writes a column carrying
  `-- paridyum:class: pii` or `confidential`, and any request-time query against the
  applications or candidate-profile tables. No exceptions for read-only, and none for
  "just the name" — ADR-0073 classifies `legal_name` as `pii` and the profile row is the
  object being separated.

The practical test for a reviewer: **if the page would still render for a signed-in visitor
with an empty candidate profile, it belongs on the front door.** If it needs the row, it
belongs on `web-careers`.

This is the rule that keeps the decision from decaying. The failure mode is not a deliberate
reversal — it is one small authenticated thing added to the front door eighteen months from
now because that is where the layout already lives, which is precisely how the current state
arose.

### 6. The gate of ADR-0068 §5 does not move

Schema, migrations, RLS policies, both surfaces and the split itself may be built and tested
against synthetic data. **The first real applicant still waits for an off-host backup
destination and a rehearsed restore.** Splitting a surface is not progress against that gate
and must not be reported as such.

## Consequences

**Accepted costs.**

- **A second deployable, before the first has served a visitor.** A second Helm chart, image,
  probe set, CI pipeline and manifest under ADR-0020, and a second thing to keep patched. This
  is real overhead bought against a hiring volume that is currently zero.
- **The job listings and the application form can now disagree.** Two surfaces reading one
  source will drift — a role closed in one, still listed in the other. Single-deployable
  today makes that impossible by construction; after this ADR it is a class of bug that
  exists.
- **The candidate crosses an origin mid-flow**, and the session does not follow. They sign in
  on `careers.paridyum.com` and that is where their session lives; a signed-in state on the
  marketing site is not a thing that can exist. Some candidates will read the origin change as
  a redirect to a third party — the cost every ATS-hosted form already pays.
- **`careers.paridyum.com` builds SEO authority from zero.** Mitigated, not eliminated, by
  decision 1 keeping the indexable half on the root domain — but any inbound link that points
  at an application URL rather than a listing is authority that does not accrue to
  `paridyum.com`.
- **The front door still holds a session, so it is still a phishing and session-fixation
  target,** and decision 5 is a discipline rather than an impossibility. A store-level check
  can prove the boundary; nothing proves that `[lang]/account` stays small except review.
- **Four OIDC registrations replace one**, and ADR-0070 §1's staging/production split doubles
  with them — more surface for exactly the redirect-allowlist typo that ADR warns about.
- **Migration work on a surface that is already green.** Six routes move, `api/auth` is
  duplicated rather than relocated, ADR-0070's client
  registrations change on both sides, and `web-paridyum`'s `manifest.yaml` and CONTENT.md
  stop describing things it does. `breaking: true` is set for this reason.

**Gained.**

- A content edit to the front door stops being a release of the surface that holds a real
  person's date of birth. The two now fail, deploy and roll back independently.
- The blast-radius argument of ADR-0068 §3 extends one seam further out: a compromise of the
  marketing site now yields no candidate session and no applications, where today it yields
  both.
- ADR-0070 §1's per-deployable client rule is honoured where today one client spans two things
  that deploy independently, and `web-paridyum` becomes a surface that can authenticate a
  visitor without being able to reach a single classified row — a materially smaller thing to
  reason about, and a materially smaller thing to breach.
- The estate matches the shape three independent ATS vendors and every large employer arrived
  at, which is weak evidence taken alone and useful evidence taken against three unrelated
  codebases converging.
- `web-paridyum` can be what it was for: the company's front door, not hiring's landing page
  with a company page attached.

**Rejected alternatives.**

- **Leave it as it is.** Loses on decision 5's failure mode: the coupling is not stable, it
  accumulates, and the current state is the evidence — six of nine routes arrived one correct
  step at a time.
- **Move the whole careers area, listings included, to `careers.paridyum.com`.** This is the
  literal reading of *"careers should be separate"* and it overshoots. The listings are the
  estate's best recruiting-side SEO asset and the half that inherits root-domain authority;
  moving them concedes that for no security gain, since a public listing page holds no session
  and no PII. Google's `google.com/about/careers` is this alternative rejected the same way.
- **A separate registrable domain, `paridyum.jobs` or similar.** Buys nothing over a subdomain
  that this estate can use — the origin separation, which is what decisions 3 and 4 rely on,
  is complete at the subdomain — and costs a second certificate story, a second reputation and
  a second renewal. Reconsider only if hiring volume ever justifies a standalone brand.
- **A third-party ATS — Greenhouse, Lever or Ashby — instead of building it.** Genuinely the
  cheapest path and it is rejected on residency, not on cost or on preference: RFC-0015 fixes
  tenant data in one region under operator choice, ADR-0073 puts a DPDP s9 date of birth in
  the profile, and none of the three vendors gives this estate a checkable answer about where
  that row lives. Revisit if a vendor offers a contractual residency guarantee the estate can
  verify.
- **Strip authentication from `web-paridyum` entirely**, leaving it a purely static front
  door with no client and no session. This was the first draft of decisions 3 and 5, and it is
  wrong: `[lang]/account` is the working proof of the ADR-0038 round trip, the front door will
  owe a signed-in visitor something eventually, and deleting a session that stores nothing buys
  no reduction in the blast radius this ADR exists to reduce. The risk was never the cookie.
  It was the row behind it.
- **Keep one deployable, separate by route group and middleware.** A middleware matcher is a
  correct-looking boundary that one `matcher` regex edit removes, with no deploy-time or
  origin-level signal that it is gone. It is the "boundary with one hole in it" of ADR-0068 §1
  in a different file.
- **Iframe the application form into the root domain**, per Greenhouse's own guidance. Rejected
  in decision 4: it reunites the origins in the browser, which is where the attacks are.
