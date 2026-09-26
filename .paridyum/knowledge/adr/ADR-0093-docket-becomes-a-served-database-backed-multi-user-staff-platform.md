---
id: ADR-0093
title: docket becomes a served, database-backed, multi-user staff platform
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: [ADR-0072]
superseded_by: null
scope: [platform, tool, console, infra, org]
applies_to: ["*"]
tags: [docket, tool-docket, console-docket, database, postgres, concurrency, rfc-0039, adr-0023, adr-0055, adr-0064, adr-0068, adr-0071, adr-0072, adr-0092, hash-chain, backups]
breaking: true
enforced_by: []
---

# ADR-0093 — docket becomes a served, database-backed, multi-user staff platform

## Status

**Accepted**, 2026-09-13. **Supersedes ADR-0072** (one authoritative node, loopback, `ssh -L`,
single-writer lock). Advances **RFC-0039** decision 1 from proposal to a committed architecture, and
resolves the class tension **ADR-0055** left when a console became something served. `breaking: true`.
Not `locked`: the migration's invariants (hash-chain preserved, RLS on every docket table, real data
gated on backups) are checkable and must become checks before this locks.

This is a **human direction change** (recorded per C-0004): docket must let *everyone in Paridyum*
create and maintain HR entries, and the current shape cannot. It is not a repudiation of what was
built — ADR-0072's single-writer/loopback/SSH design was correct *for a filesystem store on one
laptop*, and each of its three pillars was argued on its own merits. What changed is the store.

## Context

The ask: a served HR platform all Paridyum staff use to create and maintain people records —
RFC-0039's "one platform with several capabilities … candidates and staff both get accounts."

The current docket cannot be that, by construction:

- **The store is YAML on a filesystem**, and RFC-0039 D1 already named the wall: *"two people
  editing a YAML file on one operator's laptop is not a concurrency model."*
- **ADR-0072** makes exactly one node authoritative, binds loopback, is reached over `ssh -L`, and
  serialises writes behind an `O_EXCL` lock. Its own words: *"a second writing node would invalidate
  all three."* Multi-user is precisely a second (and third) writer.
- **ADR-0055** classed the interface a `console` — *"runs where the operator is … not served from
  the cluster."* A platform every staff member reaches is served.

Three misconceptions were corrected on the way here, and they shape the decision:

1. **Concurrency comes from the database, not object storage.** S3 is files, not concurrent record
   editing. The concurrency model is Postgres — row-level tenancy (ADR-0023), transactions.
2. **S3 is a later, separate question** — the document store and the *backup destination* have come
   apart and are open (ADR-0068 §6); today the off-host store is Sync.com, ciphertext-only, manual
   (ADR-0071). Neither is a prerequisite for multi-user; both sit behind the person store.
3. **Real people data is gated on backups** — ADR-0068 §5: nothing real is stored until an off-host
   destination exists and a restore has been rehearsed end to end. That gate binds regardless of how
   docket is hosted or authenticated.

## Decision

**1. docket's store moves to Postgres; the hash-chain moves with its semantics intact.** The system
of record for a person, an engagement, a document and the register is the shared database
(platform-tenancy migrations, RFC-0039 D1), not the filesystem. `domain/chain.rs`'s sealed,
append-only, verified-on-read ledger is ported to an **append-only table whose seal is preserved and
verified on read** — the chain moves across with its guarantees or the move does not happen (RFC-0039
D1's own condition). docket keeps its domain; it loses its `fs::`.

**2. Concurrency is the database, and ADR-0072's single-writer machinery is retired.** Row-level
tenancy (ADR-0023) and transactions replace the one-authoritative-node rule, the loopback bind as a
*concurrency* device, and the `O_EXCL` write lock. Correctness that ADR-0072 asserted by other means
is re-asserted in the schema: identifiers `UNIQUE` and never reused (ADR-0051), the ledger append-only
by constraint. ADR-0072 is superseded; its cold-spare/promotion runbook is replaced by the database's
own backup/restore (decision 5).

**3. docket is served from the cluster to authenticated Paridyum staff, and is reclassified
accordingly.** It is no longer run-where-the-operator-is over a tunnel. `tool-docket` keeps its
domain but gains a **deployable served backend** (it stops being `deployable: false`); `console-docket`
is the served staff UI. The ADR-0055 test — "if tenant traffic can reach it, it is a product in
`web-`" — is not yet tripped (only authenticated Paridyum staff reach it), but "not served from the
cluster" no longer holds, so the class must be settled in the class-registry as part of the migration:
a served, staff-authenticated backend is `platform`-shaped, its UI `web`-shaped. **The CLI survives as
a break-glass/admin path against the same database**, not as the primary interface (ADR-0054: added
capability, the old one not removed).

**4. Every staff member authenticates; there is no ambient trust.** The loopback+SSH "OS is the
identity" model is gone with the tunnel, so authentication is **not optional** the way ADR-0055 §5
allowed. Access is a Zitadel token from the control plane, validated in the backend, operator bound by
opaque `sub`, role-gated — **ADR-0092 in full**, now targeting the *served* backend rather than the
SSH one (that ADR's delivery half is realised here). This is the affirmative closure of D-38.

**5. No real person record is created until backups are real — the gate is pulled forward, not
around.** ADR-0068 §5 binds: an off-host destination and a **rehearsed** restore, `PARIDYUM_BACKUPS_
CONFIRMED`. Serving docket to staff for *synthetic* data needs none of it; the first *real* HR entry
needs all of it. The backup destination is decided here as the gate is met — **an S3-compatible target
(B2/R2/MinIO) for continuous/point-in-time recovery** is the strong candidate ADR-0071 and ADR-0068 §6
left open, chosen when this gate is worked, with Sync.com's ciphertext dump as the already-built
fallback. This ADR does not pre-decide S3; it places the decision at the gate where it belongs.

## Consequences

**Accepted costs.**
- **A real migration, not a reshaping.** `tool-docket`'s ~200 lines of `fs::` across ~10 call sites
  become a Postgres adapter; the chain's seal must be proven preserved by test, not assumed. This is
  the single largest change in the estate to date, and it is `breaking`.
- **A class change and a deployable where there was none.** `tool-docket` gains a service and a Helm
  chart it never owed (ADR-0020 now binds it); the class-registry and manifests change.
- **Authentication is now on the critical path.** What loopback+SSH gave for free is now the
  backend's job every request; ADR-0092's verifier must be right or docket is unreachable.
- **The backup gate is now visibly blocking real use**, where before it blocked an applicant nobody
  had. That is the honest cost of inviting all of HR onto it.

**Gained.** HR is usable by every staff member concurrently; a departed operator loses access when
their Zitadel account is disabled; the ledger's guarantees survive on a store that two people can
write at once; and docket joins the one-platform shape RFC-0039 requires instead of a laptop tool.

**Rejected alternatives.**
- **Keep ADR-0072 and add operators to the SSH tunnel.** A tunnel per staff member to one
  serialised writer is not multi-user; it is the same single writer with more clients queued behind
  the lock.
- **Object storage for concurrency.** The category error corrected above: S3 gives files, not
  concurrent editing.
- **Ship served multi-user before backups.** Refused — ADR-0068 §5 exists precisely so that people's
  records are not the thing lost when the gate is skipped for speed.

## Related

- **RFC-0039** — the people-platform program and its forced order; this commits its D1.
- **ADR-0072** — superseded; its single-writer design and the reason it existed.
- **ADR-0023, ADR-0064, ADR-0051** — row-level tenancy, Postgres under an operator, opaque identifiers.
- **ADR-0055, ADR-0054** — the console class this reclassifies; added-not-swapped for the CLI.
- **ADR-0092** — operator authentication, whose served-delivery half this realises.
- **ADR-0068 §5/§6, ADR-0071** — the backup gate and the open destination decision this places at the gate.
