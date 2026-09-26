---
id: REQ-0008
title: A schema change is migration-driven, reversible and deployable without downtime
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [database, migrations, schema, zero-downtime, audit]
breaking: false
trigger: >
  The first migration file is written — the moment a schema exists that something else
  will depend on.
review_date: 2027-02-01
verified_by:
  - TST-0037
  - TST-0038
  - TST-0039
enforced_by: []
---

# REQ-0008 — A schema change is migration-driven, reversible and deployable without downtime

## Status

**DEFERRED** as of 2026-08-07. No database, no schema, no migration.

The trigger is the first migration rather than the first deploy, because the decisions
that are expensive to reverse — key strategy, soft delete, audit trail — are made in the
first few migrations and inherited by every table after.

## Requirement

**Every schema change shall arrive as a reviewed migration, be reversible, and be
deployable while the previous version of the application is still running.**

## Status

**ACCEPTED**, 2026-09-11. The trigger — "the first migration file is written" — fired when
`platform-tenancy/migrations/0001_tenancy_foundation.sql` was created, and three migrations now
exist (0001–0003), applied to the live database.

**Honest, item by item, because activation without an inventory is a claim:**

| # | Item | State |
|---|---|---|
| 1 | Migration-driven only; no manual production DDL | **VIOLATED, and this is the first thing fixed.** 0001–0003 were applied by hand with `kubectl exec … psql`, which is exactly the forbidden console DDL. `platform-tenancy`'s migration runner (`scripts/migrate.py`) replaces that with a tracked, ordered, checksummed apply, and the SOP forbids the manual path. |
| 2–3 | Expand→migrate→contract; backward compatible both ways | **Not yet exercised.** Every migration so far is additive (new tables), so the hard case has not arisen; the rule binds the first ALTER on a populated table. |
| 4 | Every migration has a tested rollback | **MET, 2026-09-12.** ADR-0081 decides the hybrid: reversible before a migration takes data, restored after. Each of 0001–0004 carries a `-- migrate:down`; `migrate.py` gained a `down` command that reverses the last-applied migration and **refuses if any `public` table holds a row** (past the first write, recovery is ADR-0071's restore). The checksum is over the `up` section, so adding downs to applied migrations is not drift. Proven live: 0004 round-tripped up→down→up (defaults and `resolve_account` reverted then restored), and the guard refused with one synthetic row present. Graduates to a check when CI runs the round-trip. |
| 5 | No long-lock DDL on a large table | Not yet relevant (empty tables); binds when data exists. |
| 6 | Identity strategy decided once, in an ADR | **MET, 2026-09-12.** ADR-0078 decides the surrogate key: a `uuid` typed `DEFAULT uuidv7()` — time-ordered, coordination-free, generated once by the schema — with BIGINT and UUIDv4 closed and their reasons written. Migration `0004` added the default to `tenants`/`principals`/`applications`/`profiles`, recreated `resolve_account` to mint with `uuidv7()`, and the application `INSERT`s dropped `id` and their `gen_random_uuid()` literals; no `gen_random_uuid` remains in `migrations/` or the app. Verified live: four defaults present, v7 ordering demonstrated. Graduates to a check when a migration gate asserts (a) every surrogate PK declares `DEFAULT uuidv7()` and (b) `gen_random_uuid` is absent. |
| 7 | Soft delete decided once | **OWED.** No table carries `deleted_at`; deletion is hard delete (profiles) — the REQ-0003 erasure path. Decide before the first table that needs history. |
| 8 | Foreign keys and cascade behaviour stated | Partially: `applications.tenant_id`/`profiles.tenant_id` REFERENCE `tenants` with default (restrict). Cascade policy not yet stated as a rule. |
| 9 | Index policy | `applications_tenant_account_idx`, the unique constraints. Reviewed per migration; no written policy yet. |
| 10 | Audit trail | **OWED** — ADR-0044 (correlation id + append-only audit) governs it and is not implemented. |

The items marked OWED are the backlog this activation makes visible rather than the discharge it
is not. `enforced_by` stays empty: the `db.*` migration checks are declared and await tool-forge,
and the live-catalogue assertions (REQ-0022 acceptance criterion 4) are owed.

## What already exists

C-0002 settles **naming**, which is the part most teams argue about and the least
important: tables plural `snake_case`, primary key `id`, foreign keys `<singular>_id`,
timestamps `created_at`/`updated_at`/`deleted_at` always `timestamptz` and UTC, migrations
named `<seq>_<verb>_<subject>.sql`.

ANTI-0001 and ANTI-0003 settle **ownership** — one writer per table, no foreign reads —
and `db/grants.sql` enforces it with per-service roles. C-0004 makes history and production
edits protected operations.

What is missing is everything about **change over time**.

## Numbered items

1. **Migration-driven only.** No manual production DDL, ever. `ALTER TABLE` typed into a
   console is invisible to every other environment and to the next deploy. The migrator
   role in `db/grants.sql` already exists to make this the only path.
2. **Expand → migrate → contract.** Renaming a column in one step breaks the running
   application. Add the new column, write both, read new, backfill, then drop the old — in
   separate deploys. This is the single most valuable item here and the one most often
   skipped under time pressure.
3. **Backward compatible in both directions.** A migration must work with the *previous*
   application version still serving traffic, because during a rollout both are live. A
   rollback must not require a down-migration that destroys data.
4. **Every migration has a rollback, and it is tested.** An untested down-migration is a
   comment.
5. **No long-lock DDL on a large table.** `ALTER TABLE ... ADD COLUMN NOT NULL DEFAULT` and
   naive index creation take locks that stop the service. Concurrent index builds and
   nullable-then-backfill are the safe forms.
6. **Identity strategy decided once, in an ADR.** UUIDv7, ULID or BIGINT, with the reason.
   Each has a different index-locality and leak profile, and mixing them across services is
   a permanent inconsistency. C-0002 says the key is `id`, not what generates it.
7. **Soft delete decided once.** `deleted_at` exists in C-0002's timestamp list, which
   implies soft delete without deciding when it applies. Where it does, **every query must
   filter it** — a soft-delete column with an unfiltered query is a data leak that looks
   like a bug.
8. **Foreign keys and cascade behaviour stated.** Cascade, restrict, or set null, decided
   per relationship rather than by whatever the ORM defaults to.
9. **Index policy.** Which queries justify which indexes, reviewed as part of the migration.
   An unused index costs every write; a missing one costs every read.
10. **Audit trail.** Who changed what, when, and where required the old and new value.
    Distinct from application logs and from observability — this one is evidence.
11. **Per-table metadata.** Owner, PII classification, retention, encryption requirement,
    backup expectation. `owns.tables` in `manifest.yaml` is where this belongs; REQ-0003
    covers the privacy half.
12. **Migration review checklist.** Indexes, foreign keys, rollback, lock duration, data
    loss, backfill strategy, monitoring. The template's PR checklist is the natural home.
13. **Deprecation before deletion.** Mark deprecated, stop writing, stop reading, archive,
    then drop. Dropping a table because nothing appears to use it is how a quarterly job
    discovers it did.

## Why this is not startable today

Items 2–5 and 9 describe operations on a running system with data in it. Items 6–8 are
genuinely decidable now, and are deliberately deferred anyway: deciding a key strategy
before there is a single query pattern to evaluate it against produces a decision made from
taste, locked by ADR-0002, and expensive to reverse. The first migration is early enough.

## Related

- C-0002 — naming, already settled
- ANTI-0001 / ANTI-0003 — ownership and foreign access, already settled and enforced
- C-0004 — production edits and history are protected operations
- REQ-0003 — the privacy half of item 11
- REQ-0004 — deploy ordering and progressive rollout, which item 3 depends on
