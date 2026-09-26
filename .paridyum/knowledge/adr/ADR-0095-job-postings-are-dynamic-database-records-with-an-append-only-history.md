---
id: ADR-0095
title: Job postings are dynamic database records with an append-only history, not git-declared files
type: adr
status: accepted
created: 2026-09-14
locked_at: null
supersedes: [RFC-0041]
superseded_by: null
scope: [platform, web, console, tool]
applies_to: ["*"]
product: [tenancy]
tags: [hiring, careers, job-postings, web-hiring, console-docket, tool-docket, dynamic, cms, database, rls, audit, adr-0044, adr-0051, adr-0063, adr-0073, adr-0092, adr-0093, rfc-0039, rfc-0041]
breaking: true
enforced_by: []
---

# ADR-0095 — Job postings are dynamic database records with an append-only history

## Status

**Accepted**, 2026-09-14. **Supersedes RFC-0041** and **reverses ADR-0063 decision 2** (a job
description is git-declared data, the repository its admin surface). A human direction change
(recorded per C-0004): the operator wants to post and manage roles live, without a commit and a
deploy. `breaking: true` — it moves the source of truth for a public claim.

This is not a repudiation of ADR-0063 §2 / RFC-0041. Their reasoning was correct: a public claim
about pay, duration and terms deserves an author, a review and a history, and a git file gives all
three for free. **The reversal only holds if those three properties are preserved in the new
store** — so this ADR's load-bearing decision is not "move to a database", it is "move to a
database *with an append-only history and a publish gate that reproduce what git gave*." Without
decision 4 below, this would be the regression RFC-0041 warned against.

## Context

`web-hiring` exists and is deployed: the careers pages, a candidate profile, `api/apply`,
`api/profile`, OIDC. Applications and profiles already persist to the shared tenancy Postgres via
account-context/RLS (ADR-0073, ADR-0068) — the platform is the system of record, not docket, and a
public portal never reaches docket (C-0008, RFC-0039). Only one thing is still static:
`web-hiring/lib/jobs.ts` reads JD files from `jobs/` at **build time**, so posting a role needs a
commit and a redeploy.

RFC-0039 and ADR-0093 have since put a tenant-scoped Postgres with RLS under the estate and moved
docket onto it. The infrastructure a dynamic posting store needs now exists; the only question left
is whether to use it, which is what this ADR answers.

## Decision

**1. A job posting is a row in `job_postings` in the shared database, not a file.** Tenant-scoped
(ADR-0023), classified per ADR-0073/`fields.yaml` conventions. `web-hiring` reads it; it is no
longer built into the image. The `jobs/` directory and `lib/jobs.ts`'s filesystem read are retired.

**2. The identifier and its freeze survive unchanged (RFC-0041 §1 carried over).** A posting id is
`<slug>-<year>`, lowercase ASCII/digits/hyphens, the year immutable for that posting. **It is
frozen the moment `status` first becomes `open`** — before that a draft may be corrected freely;
after that the id may never be renamed and never be reused, enforced by constraint and trigger
rather than by a filename convention. This is exactly ADR-0051's test for an identifier (a frozen,
never-reused string encoding only immutable facts), and it is why `engagement.job_id` can point at
a posting for a decade.

**3. A posting is never deleted; the table is a monotonic register.** A `closed` posting stays a
row forever — that is the only thing that keeps a `job_id` on a person record resolvable in 2036
(RFC-0041 §1). Deletion is not a code path. `status` moves `draft → open → closed` and stops; a
role advertised again next season is a new row with a new year, never an edit of the old one.

**4. Every change is an append-only event, and publication is a gated transition (this replaces
git).** A companion `job_posting_events` table records who did what and when — created, each edit
(with the changed fields), published, closed — append-only (the `forbid_mutation` trigger,
ADR-0044). This is git's author, diff and history moved into the database, and it is the condition
on which superseding RFC-0041 is defensible. Publication (`draft → open`) is an explicit operator
action that freezes the id and stamps the publisher; it is the review gate git's merge used to be.

**5. docket stays decoupled — the id is a recorded fact, never a render-time lookup (RFC-0041 §3
carried over, and it now matters more).** Even though postings and docket share one database,
`tool-docket` MUST NOT read `job_postings` to render a document, and there is **no foreign key**
from a person/engagement to `job_postings`. `engagement.job_id` remains a recorded string so an
issued agreement or certificate renders identically in 2036 when the posting is long closed, and so
a person record does not break if a posting is absent. The shared database does not license a
coupling ADR-0001 and RFC-0041 §3 forbade.

**6. Reads are public for open postings; writes are operators only.** The careers page is public
and has no account, so the RLS policy admits a `SELECT` of `status = 'open'` rows for any session
that has set the Paridyum tenant context, while insert/update require operator scope (ADR-0038,
GLO-0005). Public traffic reads open postings; only an authenticated operator (ADR-0092) creates,
edits, publishes or closes them.

**7. The operator posting surface is an operator-gated section of `web-hiring`, authenticated by
ADR-0092.** Postings are the hiring domain and live in the platform database — which `web-hiring`
already reaches (account-context/RLS) and already writes (applications, profiles), behind an OIDC
session. `console-docket` was the intuitive choice ("beside the roster") but it is wrong here: it
talks to *docket's* API, and postings are decoupled from docket (decision 5), so putting posting
management there would invent a cross-domain coupling this ADR exists to avoid. web-admin is the
tenant identity/membership surface, a different domain again. So the public careers pages
(anonymous read) and the operator posting admin (role-gated write) are two sections of one
people-platform app, separated by the `operator` role in the token — not two apps.

**8. Public postings are cached, because they are public and low-churn.** `web-hiring` renders the
careers list and a posting page with a short revalidation window (ISR / `revalidate`), so the
common case is served from cache — the "fast" property the static build gave, kept, now refreshed
without a redeploy. A publish/close invalidates or is picked up on the next revalidation.

Compensation stays declared and an unpaid posting must still name what it gives instead (RFC-0041
§5), and the engagement classes (`intern`/`employee`/`contractor`) and document provides
(`CRT`/`REF`/`EQP`) it mirrors from docket are carried as columns/constraints.

## Consequences

**Accepted costs.**
- **A public claim now lives in a database, not a diff.** The mitigation is decision 4 —
  `job_posting_events` — but a database audit is not a code review, and an operator can publish
  without a second pair of eyes in a way `git` merge did not allow. The publish gate records *who*;
  it does not require *another*. A four-eyes publish is a later refinement, noted as debt.
- **The reversal is `breaking`.** `web-hiring`'s job read changes shape, `jobs/` is retired, and a
  new migration + operator UI are owed.
- **Two more tables under the backup gate.** Postings are not personal data, but the events table
  and the whole store still ride on ADR-0068 §5 backups being real before anything load-bearing.

**Gained.** Roles are posted and closed live by HR without a deploy; the careers page is dynamic
and still cache-fast; and the posting register gains an author-and-history trail in the same place
everything else about a person's journey now lives.

**Rejected alternatives.**
- **Keep git-declared postings (ADR-0063 §2 / RFC-0041).** The status quo; rejected on the
  operator's explicit direction for live posting, having weighed that it trades review/history for
  immediacy — a trade decision 4 substantially buys back.
- **Hybrid: draft in the DB, publish to git.** Keeps the git register but doubles the machinery and
  the surface area; rejected as more moving parts than the append-only events table, which achieves
  the same history in one store.
- **A foreign key from the person record to the posting.** Rejected — decision 5; it would couple
  document rendering to a live row and break RFC-0041 §3's decade-resolvability.

## Related

- **RFC-0041** — superseded; its identifier-freeze and reference semantics are carried over
  (decisions 2, 3, 5).
- **ADR-0063 §2** — reversed; the rest of ADR-0063 (hiring is the front half of docket;
  applications wait for a store) stands.
- **ADR-0044** — append-only audit, the mechanism decision 4 uses.
- **ADR-0051** — the identifier test the frozen posting id satisfies.
- **ADR-0073, ADR-0068, RFC-0039** — the candidate/application half of the people platform this
  posting store sits beside; the platform, not docket, is the system of record.
- **ADR-0092, ADR-0093** — operator authentication and the served backend the console posting
  surface authenticates against.
