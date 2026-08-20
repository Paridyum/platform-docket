---
id: REQ-0008
title: A schema change is migration-driven, reversible and deployable without downtime
type: requirement
status: deferred
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
