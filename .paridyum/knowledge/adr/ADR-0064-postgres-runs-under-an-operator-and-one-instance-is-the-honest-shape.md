---
id: ADR-0064
title: Postgres runs under an operator, and one instance is the honest shape on one hypervisor
type: adr
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [postgres, database, cloudnativepg, backup, restore, persistence, longhorn, adr-0023, adr-0038, rfc-0023, rfc-0039, req-0001, req-0003, req-0008]
breaking: false
enforced_by: []
---

# ADR-0064 — Postgres runs under an operator, and one instance is the honest shape

## Status

**Accepted**, 2026-09-05. Discharges the first item RFC-0039 left open.

## Context

**The engine is not in question.** ADR-0023 decided "one Postgres database, one schema, row-level
security" and ADR-0038 builds session context on top of it. What nothing decided is *how Postgres
runs here* — and RFC-0039 named that as the thing blocking Zitadel, profiles, applications,
documents and docket.

The substrate is now able to hold it: Longhorn is installed, `longhorn` is the default
StorageClass, and a volume has been written, detached and read back from another node. Two
replicas per volume, one on each of two workers.

The constraint that decides most of this ADR: **those two workers are virtual machines on one
hypervisor.** `sitar` is the failure domain. Nothing arranged inside the cluster changes what
happens when that machine stops.

## Decision

### 1. CloudNativePG, not a StatefulSet and not a Helm chart of Postgres

An operator, because the operations that matter are the ones nobody does by hand correctly:
failover, switchover, a base backup, point-in-time recovery, a major-version upgrade. A
StatefulSet gives a running Postgres and no answer to any of them, and the day one is needed is
the day it is least available.

CloudNativePG specifically: it is CNCF, it declares clusters as custom resources rather than
templating a chart, and its backup and PITR story is built in rather than assembled. Its
`Cluster` object is the same shape here and on metal, which is what RFC-0023 decision 4 asks of
everything in this substrate — one design, a variable where it must differ.

### 2. One instance, and it is a variable

`instances: 1`.

**HA inside one hypervisor is theatre.** A primary and a replica on two VMs on `sitar` survives a
node reboot and does not survive the thing that actually takes this estate down, which is the
host. Running two would double the storage on 12 GiB data disks, halve what is left for
everything else, and buy a failover drill against a fault we can already trigger by other means.

What protects the data here is Longhorn's two replicas, and that is a different guarantee: the
volume survives a worker, not the hypervisor.

`instances` is a value in a file. On metal it becomes 3. That is the same scale-down RFC-0023 D7
made for replica count, and it is expressible for the same reason.

### 3. Backups go off the host before personal data goes on

**A backup on the same hypervisor as its source is not a backup.** It is a second copy of a file
that dies with the first, and calling it a backup is worse than having none, because it ends the
conversation.

So: continuous WAL archiving and scheduled base backups are configured now, to a destination
**outside `sitar`**. Until such a destination exists, the cluster runs with local volume
snapshots only, and **that state blocks personal data rather than merely being noted.** ADR-0063
already holds applications behind a store; this is the second half of the same line — the store
existing is not the same as the store being safe to put a person in.

This is also where the document-storage decision lands: whatever object storage holds candidate
documents is the natural destination for `barman` archives, which is one more reason for it to
be S3-compatible.

### 4. The superuser is disabled and the application gets a role

CloudNativePG creates a database, an owner role and a secret. `enableSuperuserAccess: false`, so
there is no `postgres` password to leak and no ambient superuser for an application bug to reach.

**This is where REQ-0001 stops being deferrable.** Its trigger — "the first service needs a
credential at runtime" — fired before this ADR was written, and the credential this creates is
the one that matters: a database password for a system holding other people's identity documents.
The operator-generated secret in-cluster is where it starts and not where it ends.

### 5. Restore is proven before anything depends on it

RFC-0039 records that `etcd` snapshots are taken by hand, on no schedule, and have never been
restored. The same must not become true of the database.

**A backup that has never been restored is a belief, not a control.** A restore drill runs after
this cluster exists and before anything real is stored in it, and its result is written down —
including how long it took, because a restore whose duration nobody knows is one nobody can plan
around.

## What this does not decide

- **Schema, migrations or RLS policy.** ADR-0023, ADR-0024 and ADR-0038 own those, and REQ-0008
  fires at the first migration file.
- **Where off-host backups go.** Named as blocking above; decided with the object-storage choice.
- **Connection pooling.** CloudNativePG ships PgBouncer as a separate resource; nothing needs it
  at one instance and no clients.

## Alternatives considered

- **A StatefulSet and the official image.** Fewest moving parts, and no operator to learn. It
  gives a running database and nothing else — no base backup, no PITR, no supervised failover,
  no major-version upgrade path. Each of those becomes original work at the worst moment.

- **Zalando's postgres-operator.** Mature and widely run. CloudNativePG is CNCF, has the
  clearer backup and PITR model, and does not require Patroni's understanding of etcd on a
  cluster where etcd is already the thing whose restore is untested.

- **A managed Postgres from a cloud provider.** Solves backup, HA and restore in an afternoon,
  and puts the tenancy database outside the estate — contradicting the residency argument
  `web-paridyum` makes publicly and RFC-0017's position on operator data. Rejected on the same
  reasoning as the ATS in ADR-0063.

- **Two instances anyway, for the practice.** Genuinely tempting: a failover drill has value even
  against a fault that cannot happen here. Rejected because the storage cost is real on 12 GiB
  disks and because a two-instance cluster on one host invites the belief that the database is
  highly available. A false belief about availability is more expensive than an unpractised
  failover.

- **Waiting for off-host backup before running Postgres at all.** The most cautious order, and it
  blocks Zitadel and every profile behind a procurement decision. Rejected because the database
  can exist and be exercised while holding nothing that matters — and decision 3 is what keeps
  that distinction honest rather than aspirational.

## Consequences

- **The chain moves.** Zitadel, profiles and applications are unblocked in that order.
- **Nothing real may be stored until backups leave the host.** That is a gate this ADR sets on
  itself, and the point at which it is quietly ignored is the point this decision failed.
- **A restore drill is owed**, with its duration recorded.
- **REQ-0001 and REQ-0003 both come due now**, and neither can be discharged by this ADR.
