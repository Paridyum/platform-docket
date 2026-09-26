---
id: RFC-0039
title: The people platform is one program, and the order it must be built in is forced rather than chosen
type: rfc
status: proposed
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra, platform, web, console, tool]
applies_to: ["*"]
tags: [hiring, onboarding, docket, identity, zitadel, persistence, storage, dpdp, personal-data, adr-0019, adr-0023, adr-0045, adr-0055, adr-0063, req-0001, req-0003, rfc-0017]
breaking: false
enforced_by: []
---

# RFC-0039 — The people platform is one program, and its order is forced

## Status

**Proposed**, 2026-09-05. Written before any of it is built, because ADR-0019 requires
decomposition to precede implementation and because four of the pieces below cannot be built in
the wrong order at all.

## What was asked for

A public landing surface that says what Paridyum does at a bird's-eye level; a job portal;
candidate and staff self-enrolment onto profiles they maintain themselves; documents those people
upload and we hold; and `tool-docket` wired to all of it so onboarding runs through the estate
rather than through an operator's terminal.

Operator decisions taken 2026-09-05, recorded here because they are the constraints everything
below is designed against:

- **One platform with several capabilities**, built so a group framing can be added later without
  a rewrite. Not a portfolio of separate services — C-0006 anti-goal 1 forbids breadth without
  coherence, and this stays inside it.
- **Candidates and staff both get accounts.**
- **Zitadel**, as ADR-0045 already decided, rather than a smaller identity.
- **Documents on a consumer cloud drive**, S3 a secondary path.
- **It runs on `sitar` as it stands.**

## The forced order

This is the part that matters. Each item below is blocked by the one above it, not by preference:

1. **Storage.** The cluster has no StorageClass, no PersistentVolumeClaim and no StatefulSet.
2. **A database.** ADR-0023 already fixes the shape — row-level tenancy in one database — and it
   cannot exist without (1).
3. **Runtime secrets (REQ-0001).** Zitadel needs a masterkey and the database needs credentials.
   REQ-0001's trigger fired before this program started, and ADR-0045 risk R3 expired at the
   first deployment on 2026-09-02.
4. **Zitadel.** Needs (2) and (3).
5. **Profiles and applications.** Need (4) for a login and (2) for somewhere to put a person.
6. **Documents.** Need (5), because a document belongs to a person.
7. **docket joined to it.** Needs (5), because the person must exist before docket can issue
   against them.

**Only the landing surface sits outside this chain**, and it is therefore the only piece that can
be built today. That is not a scheduling observation, it is the reason to start there.

## Decisions this RFC proposes

### 1. The database is the system of record for a person; docket keeps its domain and loses its filesystem

`tool-docket`'s store is not a port. It is a module of free functions over `Paths` — 198 lines of
`fs::` — called as `store::person(paths, slug)` from roughly ten places in `application/`. Staff
self-service makes that untenable: two people editing a YAML file on one operator's laptop is not
a concurrency model.

**What must survive the move is the register's hash chain.** `domain/chain.rs` seals entries so an
issued document is verifiable years later, and a naive port to a database table would quietly
destroy exactly the property docket exists to provide. The chain moves across with its semantics
intact — sealed, append-only, verified on read — or the move does not happen.

So: extract a store port, keep the file adapter, add a Postgres adapter. `docket` stays a CLI of
class `tool`, `console-docket` stays unserved, and **ADR-0055 is not touched** — the portal is the
only thing deployed.

### 2. The document store is a port before it is a vendor

Documents land behind an interface with one implementation to begin with. The vendor choice is
then reversible at the cost of an adapter rather than a migration.

**The operator chose a consumer cloud drive, and this RFC records the objection rather than
burying it.** RFC-0017 holds that an operator's personal data lives in the operator's region;
`web-paridyum` states publicly that Paridyum runs in India on infrastructure it operates itself;
REQ-0012 fired on the organisation half in August and a consumer subscription offers no data
processing agreement to assess. Staff and candidate identity documents on a Canadian consumer
drive is the narrowest possible reading of that promise.

**And the alternative is nearly free.** Item (1) of the forced order is a StorageClass, which this
program needs for the database regardless; MinIO on top of it is S3-compatible, keeps the
documents in India on hardware we operate, and makes "S3 support" the only code path rather than
a second one. The recommendation is MinIO. The decision is the operator's, and if it stands, it
is recorded as an accepted risk with those three sentences attached rather than as a default.

### 3. Personal data forces REQ-0003 before a single field is stored

REQ-0003 is `deferred` and its trigger — "the first service stores a field that identifies a
natural person" — fires the moment a profile is saved. `tool-docket` already skirts it: `people/`
holds names, addresses and engagement records for real people and is gitignored precisely because
git history cannot be erased.

Before storage: every field declared with a stated purpose, a retention period per class of
record, and an erasure path that actually deletes — including from the document store and from
backups. `docket`'s `registry/fields.yaml` is the existing standard for this and the new system
adopts it rather than inventing a second one.

**An unsuccessful candidate's application is the hard case**, not the employee's record: nobody
notices it needs deleting, and it is the record with the least reason to exist a year later.

### 4. sitar as it stands is an accepted risk, written down

One hypervisor. Three control-plane nodes that are not high availability. `etcd` backups taken by
hand, on no schedule, never restored. This program puts other people's identity documents and
bank details on that machine.

The operator accepted this. It is recorded here so that the first restore is understood as
untested rather than assumed, and so that **proving a restore is the first thing that happens
after the database exists** rather than the thing nobody got to.

### 5. The landing surface describes one platform, and says what does not run yet

The current page says plainly that nothing runs on the platform yet, and C-0006 observes that an
operating system's credibility comes from what runs on it. A bird's-eye page describing several
capabilities must not become a page implying they are all available: **capability that exists,
capability that is being built, and capability that is named but not started are three different
claims** and the page distinguishes them. ADR-0049 is the test — nothing runs outside a tenant,
and nothing ships without a buyer.

## What this does not decide

- **Which database.** ADR-0023 says one; nothing says PostgreSQL is running it here.
- **How the StorageClass is provided** on a single-node hypervisor.
- **Whether hiring is ever sold**, which C-0006 anti-goal 6 still governs.
- **Retention periods**, which belong with REQ-0003's field registry.

## Alternatives considered

- **Build the job portal on its own store, separate from docket.** This is what ADR-0063 assumed
  three hours ago and it survives right up to staff self-enrolment. Two systems of record for one
  person means a hire is a copy, and a copy is a thing that drifts — the exact failure `lib-ui`
  exists to prevent, applied to people instead of stylesheets. Rejected because the operator asked
  for staff profiles, not only candidate ones.

- **Promote `docket` to a deployed service with an API.** Honest about where a multi-writer system
  ends up, and it is a class change contradicting ADR-0055 plus a rewrite of a tool that works.
  Rejected for now and not forever: if a second console or an integration ever needs docket's
  logic over a network, this becomes the right answer and the store port built here is what makes
  it cheap.

- **Files in, files out — the portal emits YAML an operator drops into `people/`.** Cheapest by a
  wide margin, no database, no identity provider, ships in days. It makes a manual copy the
  load-bearing step of onboarding, and this session has already recorded twice what happens when a
  step that must be remembered is the one that matters. Rejected.

- **A hosted ATS and HRIS instead of building.** Works this week. Puts candidate and staff personal
  data in a vendor while the argument this company makes to its own customers is that isolation is
  provable and data stays where it was promised — a claim we could not repeat while using a
  supplier we would not accept. Rejected on the same reasoning ADR-0063 rejected it for
  applications alone.

- **Start with identity, since everything authenticates.** Intuitive and impossible: Zitadel needs
  a database, the database needs a StorageClass, and the StorageClass is item one. The order in
  this RFC is not a preference and this is the alternative that demonstrates it.

- **Do the whole program before shipping anything.** The landing surface is genuinely unblocked,
  and holding it back would mean the one piece that needs no infrastructure waits on the piece
  that needs all of it. Rejected: ship the front, build the rest behind it.

## Consequences

- **The landing surface ships first and alone**, because everything else is behind a StorageClass.
- **Five deferred requirements stop being deferrable**: REQ-0001, REQ-0003, REQ-0004, REQ-0014 and
  REQ-0018 all have triggers that have already fired, and this program makes three of them acute.
- **`tool-docket` gets a refactor before it gets a feature**, which will feel like a detour and is
  the only way its register survives contact with a second writer.
