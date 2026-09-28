---
id: ADR-0125
title: Postgres backups run through the Barman Cloud plugin, and cert-manager issues its certificates
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, infra]
applies_to: ["*"]
product: [paridyum]
tags: [postgres, cloudnative-pg, backup, barman, cnpg-i, plugin, cert-manager, mtls, adr-0064, adr-0094]
breaking: false
enforced_by: []
sources:
  - url: https://github.com/cloudnative-pg/plugin-barman-cloud/releases/tag/v0.15.0
    title: "cloudnative-pg/plugin-barman-cloud v0.15.0 — release and manifest"
    retrieved: 2026-09-28
---

# ADR-0125 — Postgres backups run through the Barman Cloud plugin, and cert-manager issues its certificates

## Status

**Accepted** 2026-09-28. The operator's instruction, given when the deprecation below was reported,
was "do it".

## Context

**Every Postgres backup in the estate uses CloudNativePG's native Barman Cloud support.** That covers:

- continuous WAL archiving;
- the nightly base backup;
- the off-host copy's source;
- the restore rehearsal that proved them on 2026-09-28 (ADR-0064 D5, ADR-0094).

**CloudNativePG 1.30 warns on every apply:** *"Native support for Barman Cloud backups and recovery is
deprecated and will be completely removed in CloudNativePG 1.31.0."* An operator upgrade that crossed
1.31 would stop backups. Nothing in the estate reads the Cluster's conditions, so it would stop them
silently.

**The replacement is CNPG-I's Barman Cloud plugin.** It is a separate deployment that CloudNativePG
calls over gRPC with **mutual TLS**, plus a sidecar in the Postgres pod. Its manifest obtains those
certificates from **cert-manager**, which the cluster does not run.

## Decision

1. **Backups move to the Barman Cloud plugin**, `plugin-barman-cloud` v0.15.0. Its manifest is
   vendored in infra-kubernetes, checked against the release's published digest, and its two images
   are pinned by digest.
   - An `ObjectStore` resource carries what `spec.backup.barmanObjectStore` carried: the same
     destination, endpoint, credential, compression and retention.
   - The Cluster names the plugin as its WAL archiver.
   - The `ScheduledBackup` and the restore rehearsal use it too.
   - **The archive stays in place**, same bucket and same server name, so recovery can span the
     change.
2. **cert-manager is installed to issue the plugin's certificates**, at chart v1.21.2 with its CRDs,
   one replica each. The plugin's own manifest declares its `Issuer` and `Certificate`s; cert-manager
   renews them.
3. **Done means proven, not applied.** The move is complete only when:
   - WAL archiving reports success through the plugin;
   - a base backup taken through the plugin completes;
   - the restore rehearsal passes against it.

## Consequences

**Accepted costs.**

- **Two more cluster components** (cert-manager, the plugin) on a small cluster, and a sidecar
  container in the Postgres pod.
- **Switching the archiver restarts the only Postgres instance** (ADR-0064: one instance) — a short
  outage.
- **cert-manager becomes a dependency of backups.** If it stops renewing, the operator can no longer
  reach the plugin once the certificates expire. That fails loudly, as a Cluster condition and a
  failed backup, not silently.

**Gained.**

- The estate can upgrade CloudNativePG past 1.31 without losing backups.
- cert-manager is available for the next thing that needs in-cluster certificates, instead of each
  one hand-rolling a CA.

**Rejected alternatives.**

- **Stay on native Barman and pin CNPG below 1.31.** That defers a forced migration to the day a
  security fix needs 1.31.
- **Hand-made certificates for the plugin** (a self-signed CA in Secrets). Nothing would renew them,
  which is a certificate expiry scheduled into the backups.

## Related

- ADR-0064 — Postgres under an operator, one instance, a drilled backup.
- ADR-0094 — Garage, the backup endpoint.
- ADR-0071 — off-host copy.
- infra-kubernetes `postgres/backup.yaml`, `postgres/restore-rehearsal.yaml` and `SOP.md`.
