---
id: ADR-0094
title: Garage is the in-cluster S3 object store, docket's document store, and the estate's WAL/backup endpoint
type: adr
status: accepted
created: 2026-09-14
locked_at: null
supersedes: []
superseded_by: null
scope: [infra, platform, tool]
applies_to: ["*"]
tags: [storage, s3, garage, object-store, backup, documents, docket, cnpg, wal, longhorn, adr-0064, adr-0068, adr-0071, adr-0093, rfc-0039, req-0001]
breaking: false
enforced_by: []
---

# ADR-0094 — Garage is the in-cluster S3 object store and the estate's backup endpoint

## Status

**Accepted**, 2026-09-14. **Resolves the open destination ADR-0068 §6 left** ("the backup destination
and the document store have come apart, and that is now open") and supplies the S3 endpoint
**ADR-0064 D3** assumed and **postgres/backup.yaml** was left waiting for (its `REPLACE` bucket/endpoint
placeholders). A human direction (recorded per C-0004): the operator chose an S3-compatible object
store, self-hosted, and named Garage.

## Context

Three things needed an S3-compatible endpoint and none existed:

- **docket's document store** (ADR-0093 §1, ADR-0068 §6). docket keeps issued PDFs, packs and signed
  scans; ADR-0093 moved its *records* to Postgres and left the *blobs* on a filesystem as an explicit
  interim, because there was no object store to put them in.
- **CloudNativePG WAL archiving** (ADR-0064 D3). `postgres/backup.yaml` carries the barman-cloud
  config with `REPLACE` placeholders for a bucket and endpoint; the infra LOGBOOK records this as
  *"genuinely blocked — an S3-compatible object store off `sitar`."* Until it exists the cluster's
  recovery is the ~24h logical dump (ADR-0071), not point-in-time.
- **The backup-gate rehearsal** (ADR-0068 §5). No real person record is stored until a restore has
  been rehearsed end to end; a rehearsal needs a real destination.

**Verified before deciding** (this is the correction that produced the decision): the live staging
cluster was inspected directly. There is no Garage, MinIO, or any object store deployed — only
Longhorn (block storage) and CloudNativePG. The premise that "we already have Garage" did not hold;
Garage is deployed *by* this decision, not assumed by it. Recording that here so the next reader does
not inherit the mistaken premise.

## Decision

**1. Deploy Garage in-cluster as the estate's S3-compatible object store.** Garage runs in the
`storage` namespace, backed by Longhorn (the same storage class Postgres uses), exposing the S3 API
on the cluster network. It is self-hosted, single-region, and holds ciphertext or
already-non-sensitive artifacts only (see decision 4). It is not exposed off-cluster.

**2. Garage is docket's document/blob store** (ADR-0093 §1, ADR-0068 §6 resolved). Issued PDFs, packs
and scans that ADR-0093 left on a filesystem move to a Garage bucket addressed by the S3 API. The
*record* that a document was issued is already in Postgres (the register chain); this gives the
*bytes* a shared home so a served instance and a break-glass CLI see the same documents. docket's
existing at-rest encryption is unchanged — what is encrypted to a person's or the estate's age key
(ADR-0069/0071) is encrypted before it reaches the bucket.

**3. Garage is the S3 endpoint CloudNativePG archives WAL to** (ADR-0064 D3). Filling
`postgres/backup.yaml`'s bucket/endpoint with the Garage service upgrades the database's recovery from
the ~24h logical-dump window to continuous WAL archiving and point-in-time recovery — the named
trigger ADR-0071 recorded for "the day PITR is enabled."

**4. The whole Garage store is itself backed up off-host, manually, as ciphertext** (ADR-0071 stands).
Garage in one cluster is not its own disaster recovery. The off-host copy remains the estate's
existing mechanism: a scheduled or manual dump encrypted to the estate age recipient, landing as
`*.age` off-host (Sync.com today, ADR-0071). Garage holds the live/working copy and the WAL stream;
the off-host `.age` dump is the copy that survives losing the cluster. Nothing readable leaves the
cluster unencrypted.

**5. The restore rehearsal (ADR-0068 §5) runs against Garage, and the gate still binds.** The
`storage-restore-rehearsal` path rehearses recovery using Garage as the endpoint. `PARIDYUM_BACKUPS_
CONFIRMED` — an off-host destination AND a rehearsed restore — remains the gate before the first
**real** person record. Deploying Garage satisfies the *destination*; the *rehearsal* is run before
the gate is declared met. This ADR does not declare the gate met; it makes meeting it possible.

## Consequences

**Accepted costs.**
- **A stateful workload the estate now operates.** Garage is durable state on Longhorn with its own
  capacity, upgrade and key-management concerns; it is a thing that can fill up or fail and must be
  monitored, not fire-and-forget.
- **Off-host DR is still manual.** Decision 4 keeps ADR-0071's manual/scheduled dump rather than
  continuous off-host replication; the recovery-point for a whole-cluster loss is the last off-host
  `.age` dump, not the last WAL segment. Accepted at this stage, as ADR-0071 accepted it.
- **Access keys are secrets under REQ-0001.** Garage's S3 access key/secret for each consumer
  (docket, CNPG) are SOPS-encrypted and created as k8s secrets by a Makefile target, never inline,
  never printed — the pattern `postgres-backup-secret` already uses.

**Gained.** docket's documents get a shared home; Postgres gains point-in-time recovery; the
backup-gate rehearsal has a destination to rehearse against; and ADR-0064 D3's assumption is made
true by construction instead of left as a placeholder.

**Rejected alternatives.**
- **A managed cloud S3 (AWS/B2/R2).** Rejected for this stage on residency and cost/control grounds
  (RFC-0017, the same posture that put Postgres and Zitadel in-cluster); Garage keeps the bytes on
  the estate's own substrate. The barman config remains portable (swap endpoint/keys) if that changes.
- **MinIO.** A heavier operational surface than the single-binary Garage for a single-region,
  modest-scale store; no feature here needs it.
- **Keep docket blobs on a filesystem PVC.** Rejected — it does not give CNPG a WAL endpoint, does not
  give a served instance and a CLI a shared document store, and leaves ADR-0068 §6 open.

## Related

- **ADR-0068 §5/§6** — the backup gate, and the open document-store/backup-destination question this
  resolves.
- **ADR-0064 D3** — the WAL-archiving assumption this supplies the endpoint for.
- **ADR-0071** — the off-host, ciphertext-only, manual-dump posture, which stands as decision 4.
- **ADR-0093** — docket's served/DB-backed move, whose deferred blob store this places in Garage.
- **RFC-0039, REQ-0001** — the people-platform program; runtime secret management for the access keys.
