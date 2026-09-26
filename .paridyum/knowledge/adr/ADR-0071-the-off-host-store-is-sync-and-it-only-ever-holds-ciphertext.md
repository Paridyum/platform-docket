---
id: ADR-0071
title: The off-host store is Sync.com, it only ever holds ciphertext, and recovery is a daily dump not a continuous stream
type: adr
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra]
applies_to: ["*"]
product: [tenancy]
tags: [storage, backup, documents, sync, encryption, age, residency, dpdp, adr-0064, adr-0057, adr-0051, adr-0068, rfc-0039, rfc-0017, req-0003, req-0012, req-0013]
breaking: false
enforced_by: []
sources:
  - url: https://www.sync.com/help/what-are-the-system-requirements-for-sync/
    title: "Sync.com — System requirements"
    retrieved: 2026-09-06
    supports: >
      What the vendor's client supports, which is what makes this decision's daily-dump shape
      necessary rather than chosen. Cited because the decision rests on the vendor's documented
      capability and a capability is a snapshot that decays silently (RFC-0020).
  - url: https://www.sync.com/help/the-sync-desktop-application/
    title: "Sync.com — The Sync desktop application"
    retrieved: 2026-08-31
    supports: >
      The desktop client's documented behaviour, which is the mechanism by which anything reaches
      the off-host store at all. Recorded in research/2026-08-31-d1e099c1.jsonl as a `fetched`
      entry; this citation is the reconciliation RFC-0027 asks for rather than a date from memory.
---

# ADR-0071 — The off-host store is Sync.com, and it only ever holds ciphertext

## Status

**Accepted**, 2026-09-07. Not `locked`: the rules here are checkable in principle — a file
reaching the store without an `.age` extension is a violation a script could catch — but the
store is a folder on an operator's machine, not something the estate's checks can see, so the
enforcement is procedural until a sync-side check exists.

## Context

The operator chose a consumer cloud drive (Sync.com) for storage, twice, and reaffirmed it when
asked directly on 2026-09-07. RFC-0039 §2 recorded the objection and did not bury it: RFC-0017
holds an operator's personal data in the operator's region, `web-paridyum` says publicly that
Paridyum runs in India, and Sync.com stores in Canada with no data-processing agreement a consumer
subscription lets anyone assess. That objection stands against **plaintext** personal data on a
Canadian consumer drive.

Two facts turn the objection from a blocker into a design constraint.

**Sync.com is zero-knowledge, and we hold our own keys.** If everything sensitive is encrypted
*before* it reaches the synced folder — with an age recipient that is ours (ADR-0057), not
Sync.com's — then the drive holds ciphertext it cannot read. The readable personal data never
leaves India; only opaque bytes do. Residency is about where the *data* is, and encrypted-at-rest-
before-upload keeps the data, in any meaningful sense, on the cluster.

**Sync.com cannot be a database backup endpoint, and that is not negotiable by preference.**
ADR-0064 D3 wants continuous WAL archiving to an off-host destination. WAL archiving needs an
object-store API the database streams to; Sync.com presents a folder a desktop agent replicates.
There is no S3 endpoint, so there is no WAL stream. ADR-0068 §6 already recorded that the backup
destination came apart from the document store for exactly this reason.

What Sync.com *can* receive is a file. So the database backup it holds is a **scheduled logical
dump**, not a continuous archive — and the difference is a recovery-point objective the operator
is accepting rather than one being hidden.

## Decision

### 1. The store holds ciphertext, and the extension is the contract

Everything classified `pii`, `confidential`, or `statutory` — every database dump, and every
identity or statutory document — is **age-encrypted before it enters the synced folder**, to an
operator age recipient (ADR-0057). The file lands as `*.age` and Sync.com never holds the
plaintext.

**The `.age` extension is load-bearing, not cosmetic.** A file in the sensitive trees without it
is a plaintext leak of personal data onto a foreign consumer drive, and it is the one thing a
sync-side check will assert when one exists. A file that will not decrypt with an operator key is
equally a defect — it is unrecoverable, which is worse than absent.

**Issued documents are the deliberate exception.** A certificate or reference letter docket
issues is *meant* to be shared and independently verified (ADR-0053's exactness, docket's whole
purpose). Those may sit in plaintext, because their audience is the world, not because the drive
is trusted. The classification in docket's `registry/` decides which tree a document lands in;
nothing in the encrypted trees is ever plaintext.

### 2. Database recovery is daily-granularity, and the window is accepted, not ignored

A nightly `pg_dump` of the `paridyum` database, piped through `age`, written to the store. That is
the whole backup. Its consequence, stated plainly: **a restore loses up to a day.** There is no
point-in-time recovery, because there is no WAL stream, because there is no S3 endpoint.

The operator accepts a ~24-hour recovery-point objective **at the current stage** — a system with
no real applicants yet and a handful expected. This is a weaker posture than ADR-0064 D3
describes, and it is taken with the cost named rather than by letting the gate quietly lapse.

**The trigger to revisit is written so it is recognisable:** the day a customer SLA, a compliance
commitment, or applicant volume makes a day of loss unacceptable, the database moves to an
S3-compatible target (Backblaze B2, Cloudflare R2, or MinIO on a *second* host) with continuous
WAL archiving, and ADR-0064 D3 is met in full. The config for that already exists at
`postgres/backup.yaml`, provider-agnostic, waiting for an endpoint.

### 3. ADR-0064's gate is met by this, and a restore must still be rehearsed

ADR-0064 D3 blocks personal data until backups leave the host. A nightly encrypted dump leaving
for Sync.com's cloud **is** a backup that leaves the host, so the gate is met in the amended form
this ADR sets — with one clause of ADR-0064 that does not bend: *"A backup that has never been
restored is a belief, not a control."*

**No real applicant is stored until a restore has been rehearsed end to end** — dump, sync down on
another machine, decrypt, restore into a scratch database, and confirm the rows. The Sync.com leg
is part of what is rehearsed, because a backup that reached the drive and cannot be pulled back is
the same as no backup.

### 4. Paths carry opaque ids, never names

A person's directory is named by the opaque person identifier (ADR-0051), never by their name.
Two reasons, and the second is specific to this vendor: a name in a path is personal data in a
place retention and erasure are hard to reach, and **file and folder *names* are metadata that a
zero-knowledge drive does not necessarily encrypt** — so a plaintext name in a path can leak even
when the file content is ciphertext.

### 5. Erasure reaches the store, or the store is not compliant

REQ-0003's deletion path includes these files. A person's erasure deletes their document tree and
excludes their rows from the next dump; because dumps are whole-database snapshots, an erased
person persists in older dumps until those age out under the retention schedule, and that
retention window is therefore a stated maximum on how long an erasure takes to become total. This
is the same property REQ-0003 already names for backups; it is written here because the consumer
drive makes it easy to forget which copies exist.

## Consequences

**Accepted costs**

- **Up to a day of data loss on a restore.** The headline cost, accepted for the current stage,
  with a named trigger to end it. A logical dump is also a heavier, longer operation than WAL
  archiving as the database grows, and it locks nothing but competes for I/O.
- **The Sync.com leg is manual to set up and cannot be fully automated from here.** The desktop
  agent runs on an operator machine, not in the cluster. Getting a dump from a cluster CronJob
  into the synced folder is an operator-mediated hop (a pull over the NetBird tunnel, or the
  folder mounted on a reachable host), and it is a runbook step, not a Kubernetes object.
- **Encryption makes the drive's own preview and search useless** for everything sensitive. That
  is the point, and it means the drive is a backup medium, not a working document surface.
- **Restore is only as good as the last rehearsal.** A pipeline with a manual leg rots; ADR-0064's
  "belief, not a control" applies with more force here than to an automated S3 archive.

**Gained**

- The residency objection RFC-0039 raised is answered for everything sensitive: readable personal
  data stays in India, and Canada holds ciphertext. That is a stronger position than either
  "plaintext on a consumer drive" or "no off-host copy at all".
- ADR-0064's gate is met without provisioning or paying for cloud object storage, which unblocks
  the applicant write path at the current stage.
- The whole scheme reuses age and the operator keys ADR-0057 already established; no new secret
  system, no new vendor credential in a file.

**Rejected alternatives**

- **MinIO on the cluster** — RFC-0039 §2's recommendation, and the right answer for *documents*:
  S3-compatible, in-region, operated. Rejected *for backups* because MinIO on sitar is on the same
  host as Postgres, and ADR-0064 D3 is explicit that a copy on the source's hypervisor is not a
  backup. It remains the recommended document store if the operator ever moves off Sync.com, and
  nothing here forecloses it.
- **A cloud S3 target now (B2/R2)** — meets ADR-0064 D3 in full, with PITR, for a few dollars a
  month. Not chosen because the operator chose Sync.com; recorded as the exact upgrade path in
  decision 2 so it is a config change and a credential, not a redesign.
- **Plaintext documents on Sync.com** — what "just put the files there" means read literally.
  Rejected because it is the RFC-0039 objection realised: identity documents of real people on a
  foreign consumer drive. Encryption is what makes the operator's choice defensible rather than a
  documented risk.
- **No off-host backup, keep waiting for S3** — the status quo that blocks all real data. Rejected
  because an encrypted daily dump to a zero-knowledge drive is a genuine off-host backup, and
  refusing it in favour of a perfect one that nobody is provisioning is how the gate becomes a
  permanent block instead of a control.
