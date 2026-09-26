---
id: ADR-0068
title: An applicant is not yet a person, so applications go to a tenanted store and docket stays loopback
type: adr
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, tool, infra]
applies_to: ["*"]
tags: [hiring, docket, tenancy, applications, adr-0049, adr-0055, adr-0063, adr-0064, adr-0024, adr-0038, req-0003, req-0008, c-0008, tool-docket, web-paridyum]
breaking: false
enforced_by: []
---

# ADR-0068 — An applicant is not yet a person, and the store that holds one is tenanted

## Status

**Accepted**, 2026-09-07. Not `locked`: decision 1 is already enforced in code — `tool-docket`'s
`require_loopback` refuses to bind anything outside `127.0.0.0/8` and says why — but that is a
crate's own test, not a check in `check-registry.yaml`, and ADR-0002 reserves `locked` for what
the estate can mechanically reject. It graduates when a check asserts the boundary from outside
the crate that implements it.

## Context

`tool-docket` grew an HTTP adapter. It now serves `GET /api/form/{class}`, which returns the
fields a class of person may be asked for, and `POST /api/person`, which enrols one. Those two
endpoints are, functionally, a self-enrolment portal. The careers page on `web-paridyum` needs a
self-enrolment portal.

**The obvious move is to point the second at the first, and it is the wrong one.** ADR-0049 D4
names this case in advance: *"If tenant data can reach it, it is in a request path and this ADR
binds — including anything shaped like a support console or an internal dashboard, which is the
case most likely to be argued and most likely to be wrong."* A candidate's date of birth
arriving over the internet is tenant data in a request path, whatever the crate that receives it
is called.

Three things make the join look closer than it is, and each is worth stating so the argument is
not had again from a position of enthusiasm:

**docket's store is a filesystem on the operator's machine.** `people/*.yaml`, an append-only
register, and issued PDFs. A candidate on the internet cannot write to it, and the versions of
that sentence that sound like solutions — a tunnel, a sync, a queue that replays into the
filesystem — are all descriptions of exposing the operator's laptop to strangers.

**docket's writes resolve an operator first.** Every write path in the router calls
`operator::resolve` and refuses without one; the comment says this is also what stops a
cross-site form forging one. That is the correct design for a console and exactly the wrong
authentication model for a member of the public, who is not an operator and never will be.

**docket's first lifecycle stage is `offered`.** ADR-0063 established this. A person who has
applied and not been offered anything has no stage in docket's model, and giving them one would
mean every count, every retention clock and every `docket check` invariant now spans a
population docket was not designed to reason about.

Against that: the apparatus docket already has is genuinely the right apparatus. `fields.yaml`
gives every field a DPDP purpose. `person-classes.yaml` forbids collecting a PAN from an unpaid
intern rather than merely not requiring it. `retention.yaml` measures from `engagement_end`
rather than issuance. **None of that should be rebuilt, and none of it should be reached over
the internet.**

## Decision

### 1. docket is not exposed, and the exposure is not made conditional

`tool-docket` binds loopback. It is a `tool-` under ADR-0049 D5 and a console under ADR-0055 —
*"an operator console runs where the operator is"* — and remote access to it is SSH port
forwarding, which is what the crate already prints on startup.

No reverse proxy in front of it, no NetBird exception, no "just the form endpoint". A boundary
with one hole in it is a boundary someone has to remember the shape of.

### 2. Applications live in Postgres, under tenancy, and not in docket

The applications store is a tenanted table in the one database ADR-0024 fixes — it supersedes
ADR-0023 and is the locked decision; ADR-0023 is cited here only where the record needs it — with
the two columns ADR-0024 requires, RLS `ENABLE`d **and** `FORCE`d, and session context set per
transaction from a verified Zitadel token per ADR-0038, failing closed.

This is not a second store built out of preference. It is where ADR-0063 §5 already pointed:
*"Phase B inherits that rather than inventing a scheme."*

### 3. The seam is the offer, and crossing it is an operator action

An applicant becomes a person in docket when an offer is made — ADR-0063's `offered` stage, now
doing double duty as the boundary between the two stores.

The crossing is an operator running docket locally against the tenanted store, not the tenanted
store calling docket. **Direction matters more than mechanism here:** the public system never
holds a credential that reaches the operator's machine, so a compromise of the careers site
yields applications and not the personnel record, the register, or the signing chain.

`identity.legal_name`, `contact.email` and the rest carry the same names on both sides.
`fields.yaml` is the definition of those fields for both stores; the Postgres schema references
it rather than restating it, and an applications column with no entry there is over-collection
by the same rule that governs docket.

### 4. Tenancy is promoted to `building`, and its buyer is hosting

`tenancy` moves from `proposed` to `building` in `product-registry.yaml`. At `building` a
`capability` owes a `REQ-` and the `tenant-facing-interface` slot, which is C-0008 stated as a
dossier obligation.

**The declared tenant-facing consumer is `hosting`, not hiring.** That distinction is the whole
of this decision's compliance with ADR-0049 D3. Hosting is vertical one, promoted to `building`
by operator decision on 2026-08-23, and it cannot run a second customer's workload without
provable isolation — C-0006 non-commodity item 2. Tenancy is built because hosting needs it and
would need it if nobody were ever hired.

Hiring is then the **first workload on it**, as tenant zero, which ADR-0049 D2 permits
explicitly: *"Paridyum's own use is a tenant… granted no scope a customer could not also be
granted."* That ordering is C-0008 and BET-0005 working as designed rather than a loophole —
the estate runs on the runtime it sells, and the first thing to run on it proves the isolation
rather than assuming it.

The registry's own note on `tenancy` predicted the failure this decision avoids: it called
tenancy *"the one most likely to be built as an internal mechanism first."* It is being built
for a buyer, with an internal consumer, and those are not the same shape.

### 5. Nothing real is stored until backups leave `sitar`

ADR-0064 D3 and ADR-0063 §1 set the same gate from two directions, and this ADR does not move
it. Schema, migrations, RLS policies and the session-context accessor may all be built and
tested against synthetic data. **The first real applicant waits for an off-host destination and
a rehearsed restore.**

### 6. The backup destination and the document store have come apart, and that is now open

ADR-0064 D3 assumed one decision would serve both: *"whatever object storage holds candidate
documents is the natural destination for `barman` archives, which is one more reason for it to
be S3-compatible."*

The operator has since chosen a consumer cloud drive for documents, recorded with the objection
in RFC-0039. Stated from the side that can be checked rather than from a vendor claim:
**CloudNativePG archives WAL to an object store it can address as an endpoint, and a consumer
sync client presents a folder on a machine.** Those are different things, and a folder that a
desktop agent replicates when it happens to be running is not a WAL archive destination.

So **ADR-0064's assumption no longer holds and the backup destination is an independent open
decision** rather than a consequence of one already taken. It is named here because an
assumption that quietly stops being true is how a gate gets passed without anyone deciding to
pass it.

The evidence position is worse than that, and it is recorded rather than smoothed over. The two
Sync.com pages the document-store choice was researched against — logged under session
`d1e099c1` — no longer resolve: one returns 404 and the other 403 as of 2026-09-07. **The
document-store decision currently rests on nothing this repository can re-read**, which is the
condition RFC-0020 exists to make visible. Re-establishing it is owed before the choice is acted
on, and is separate from the backup destination this section opens.

## Consequences

**Accepted costs**

- **Two stores hold personal data, and REQ-0003 must be satisfied twice.** Docket's registries
  discharge it on one side; the migrations discharge it on the other. Reusing `fields.yaml` as
  the single definition keeps this to one source of truth, but it is two enforcement paths and
  the Postgres one does not exist yet.
- **The offer crossing is manual and will stay manual for some time.** An operator moves an
  accepted applicant into docket by hand. That is slower than a webhook and it is the price of
  decision 3's direction rule.
- **Hiring cannot ship before tenancy exists.** Applications are now blocked on RLS, session
  context and migrations rather than on a form. This is a real delay measured in weeks, and the
  alternative was a store that would have to be rebuilt the first time a tenant existed.
- **`tenancy` at `building` owes artifacts that are not written.** A `REQ-` and a `spec-`
  contract for the tenant-facing interface. Promoting the entry creates the debt visibly rather
  than deferring the promotion to hide it.
- **REQ-0008 fires at the first migration file**, and its status still reads `deferred`.

**Gained**

- The operator's machine, the personnel records, the append-only register and the signing chain
  are unreachable from the internet, and remain so by construction rather than by configuration.
- The first real workload forces the isolation model into existence instead of describing it.
  What `web-paridyum` claims about provable isolation becomes a thing with a test.
- ADR-0063's Phase A/Phase B line survives contact with a docket that suddenly looked ready. The
  line held because it was drawn at the personal-data store rather than at a feature.
- The backup assumption in ADR-0064 is caught while it is still a gate, not after something was
  stored behind it.

**Rejected alternatives**

- **Expose docket's HTTP API to the careers page.** The shortest path, and the one ADR-0049 D4
  describes. It puts an unauthenticated-by-design console on the internet, replaces
  `operator::resolve` with something that has to trust a stranger, and drags docket's lifecycle
  invariants over a population that has no stage in them.
- **Applications by email.** ADR-0063 already rejected this and it remains the honest fallback:
  it ships this week and defers tenancy, migrations, RLS and the backup gate entirely. Rejected
  again because it defers exactly the work that vertical one needs anyway, so the deferral buys
  time against a bill that arrives regardless.
- **A separate applications database.** Avoids RLS by giving applications their own instance.
  Rejected by ADR-0023, and it would mean the first workload on the runtime deliberately does
  not use the isolation model the runtime is sold on.
- **Build tenancy with hiring as its declared consumer.** Simpler to justify in the moment and
  it fails ADR-0049 D3, which requires a tenant-facing consumer rather than an internal one.
  D5 is explicit that *"only us, for now"* is a decision to build a `tool-` instead — and the
  `tool-` in question already exists and is called docket, which is the point.
