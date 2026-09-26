---
id: ADR-0086
title: Hiring is one deployable, web-hiring, and web-paridyum is the landing
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: [ADR-0074]
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
tags: [hiring, careers, web-paridyum, web-hiring, landing, surface, adr-0074, adr-0070, adr-0068, adr-0046, adr-0067, seo, blast-radius]
breaking: true
enforced_by: []
---

# ADR-0086 — Hiring is one deployable, web-hiring, and web-paridyum is the landing

## Status

**Accepted**, 2026-09-12. **Supersedes ADR-0074.** Not `locked`: like ADR-0074 the rule is
mechanically checkable (a hiring route or a `pii` store on `web-paridyum` is the defect) but no
check asserts it yet; it graduates when `check-registry.yaml` carries one.

This is a **human direction change** (recorded per C-0004: a person chose it). ADR-0074 split
hiring in two — the careers *index* stayed on `web-paridyum` for SEO, and only the
application/profile left to `web-careers`. This ADR collapses that split: **all of hiring is one
deployable, and `web-paridyum` holds none of it.** It is ADR-0074's explicitly-rejected
alternative ("move the whole careers area, listings included"), chosen deliberately with its cost
understood.

## Context

ADR-0074's reasoning still holds on its own terms — a subdomain builds SEO authority from zero,
so the half with something to rank (the listings) was the half it kept on the root. What changed
is not that argument but the stage: `web-paridyum` is becoming a **landing / coming-soon** surface
first, and hiring is not live. Keeping a careers index on the root while the root is a coming-soon
page optimises SEO for listings that are not yet published, at the cost of two surfaces reading
one job source that can disagree (ADR-0074's own accepted cost) and a split that must be reasoned
about before either surface has served a visitor.

The simpler shape — one hiring deployable, one landing — is the one chosen. The SEO authority of
the root domain is conceded **for the job listings specifically**, and that concession is this
ADR's principal cost.

## Decision

**1. `web-paridyum` is the landing, and holds no hiring.** It is the company's front door:
marketing content, and for now a **coming-soon page**. It carries **no careers index, no
application, no candidate profile, and no candidate session.** The `api/auth` routes and
`[lang]/account` that ADR-0074 kept for the front door are removed with the rest — nothing on the
landing is authenticated, so it needs no OIDC client (ADR-0074's "strip authentication entirely"
alternative, now correct because the population that signed in has fully left).

When the landing later owes a signed-in visitor something — a tenant-marketing surface that links
to `web-admin`, say — that is a new client and a new decision under ADR-0070, not a reopening of
this one.

**2. The whole hiring experience is `web-hiring`, one deployable.** A new repository, `web-hiring`,
class `web` (C-0002), owning everything that was hiring on `web-paridyum`:

| Moves off `web-paridyum` | Into `web-hiring` |
| --- | --- |
| `[lang]/careers`, `[lang]/careers/[id]` | the public job index and listings |
| `[lang]/profile`, `api/profile` | the candidate profile (ADR-0073, unchanged field set) |
| `api/apply` | the application submission (ADR-0068 §2) |
| `api/auth/{login,callback,logout}`, `[lang]/account` | hiring's own auth, against its own client |

**ADR-0073 is not reopened** — its field set, `paridyum:class` directives, retention clock and
delete button are still the specification of what `web-hiring` stores. This ADR moves where the
code is deployed, not what it collects. The careers *index* that ADR-0074 protected for SEO moves
with it (decision's cost, below).

**3. Its origin is `careers.paridyum.com`, and it gets its own OIDC client.** The candidate-facing
URL stays `careers.paridyum.com` — "careers" is the word job-seekers search and the conventional
ATS subdomain (ADR-0074's sources) — even though the repository is `web-hiring`; a repo name and a
public origin need not match, and the friendlier URL is kept. A subdomain, not a second
registrable domain (ADR-0067: one brand, one root). Under ADR-0070 §1 it is a second deployable,
so its own `client_id` and redirect allowlist, staging and production separate. It sits in project
`paridyum-hiring` — the project ADR-0070 §2 created for this population.

**4. `web-paridyum` and `web-hiring` are linked by an outbound link, never an iframe or a
reverse-proxy path.** ADR-0074 decision 4 is retained verbatim in intent: an iframe or a proxied
root path reunites a credentialed, PII-collecting origin with the landing in the browser, which is
where the attacks are. The landing links out to `careers.paridyum.com`; the candidate sees the
origin change.

**5. The gate of ADR-0068 §5 does not move.** Schema, migrations, RLS, both surfaces and the split
may be built and tested against synthetic data. **The first real applicant still waits for an
off-host backup destination and a rehearsed restore.** Splitting or renaming a surface is not
progress against that gate.

## Consequences

**Accepted costs.**

- **The job listings build SEO authority from zero on `careers.paridyum.com`**, and the root
  domain no longer hosts the indexable recruiting content. This is the cost ADR-0074 weighed and
  declined to pay; it is paid here for the simpler one-hiring-deployable shape, while hiring is not
  yet live so the authority being forgone is prospective. Mitigation if it matters later: the
  landing can carry a thin, indexable "careers" link/section that points into the subdomain.
- **A second `web-` deployable** — Helm chart, image, probes, CI, manifest (ADR-0020) — before the
  first visitor. Same cost ADR-0074 accepted; it does not change.
- **Migration work on a green surface.** `web-paridyum` loses six-plus routes, its OIDC client and
  its session; `web-hiring` is scaffolded and receives them; ADR-0070's registration for the
  current `web-paridyum` client becomes the `web-hiring` client. `breaking: true`.

**Gained.**

- One mental model: `web-paridyum` is the landing, `web-hiring` is hiring. No two-surfaces-read-
  one-source drift between a root index and a subdomain application.
- The blast-radius goal of ADR-0074 is met *more* completely: `web-paridyum` holds no session and
  no classified store at all, so a compromise of the most-changed, most-linked surface yields
  nothing of a candidate's — it is now static.
- ADR-0070 §1's per-deployable client rule is honoured: the landing has no client, `web-hiring`
  has its own.

**Rejected alternatives.** ADR-0074 itself — keeping the index on the root — is the alternative
this supersedes; its SEO argument is sound and is the reason this is a deliberate cost rather than
an oversight. A separate registrable domain (`paridyum.jobs`) is rejected as ADR-0074 rejected it.

## Related

- **ADR-0074** — superseded by this; its split-in-two reasoning and its sources are the record of
  why keeping the index on the root was the better SEO answer, now traded for one hiring surface.
- ADR-0070 — one client per deployable; a project is a role vocabulary (hiring stays `paridyum-hiring`).
- ADR-0068, ADR-0073 — what hiring collects and the applicant gate, neither reopened.
- ADR-0067 — one brand, one root domain; why a subdomain and not a second registrable domain.
