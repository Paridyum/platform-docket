---
id: ADR-0078
title: A surrogate key is a time-ordered UUID, generated once by the schema
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [database, migrations, identity, uuid, uuidv7, postgres, req-0008, adr-0024, std-0006]
breaking: false
enforced_by: []
---

# ADR-0078 — A surrogate key is a time-ordered UUID, generated once by the schema

## Status

**Accepted**, 2026-09-12. Closes REQ-0008 item 6 ("identity strategy decided once, in an ADR").
Not `locked`: it graduates when a migration check asserts that every surrogate primary key
declares `DEFAULT uuidv7()` and that no migration or application code calls `gen_random_uuid()`
for a stored key. Until that check exists the rule binds by review.

## Context

REQ-0008 item 6 named a real gap: every surrogate key in the tenancy schema was a UUIDv4 from
`gen_random_uuid()`, and it was generated in **four places** — `paridyum.resolve_account` minted
both a principal `id` and an `account_id`, and the application typed `gen_random_uuid()` into the
`INSERT` for `applications` and again for `profiles`. Four call sites is not a strategy; it is the
same unstated choice repeated until someone changes one of them and not the others. The tables
carried no column default at all, so the schema did not even record what a key was supposed to be.

A surrogate key is expensive to change once rows exist and once other systems (docket, a backup,
an export) have copied the values, so REQ-0008 correctly lists it among the decisions that must be
made early and in writing. This is that writing.

Three candidates were on the table, the ones REQ-0008 item 6 lists:

**BIGINT identity.** Compact, sequential, perfect index locality. Rejected for a multi-tenant RLS
estate for one decisive reason: a monotonic integer visible across a tenant boundary leaks volume
and ordering. `application_id = 4021` tells any tenant roughly how many applications the whole
platform has taken and lets two rows be compared for which came first across silos. It also needs
a central sequence, which is a coordination point the nested-tenancy model (ADR-0024) is built to
avoid. GLO-0002/GLO-0003 keys must not carry cross-silo information, and a shared counter does.

**UUIDv4 (the status quo).** Globally unique, coordination-free, carries nothing. Its cost is
physical: a random 128-bit key is inserted at a random point in the primary-key B-tree on every
row, so the index never fills pages in order — it splits them, and every split is written to the
WAL and shipped to the nightly dump (ADR-0071). At candidate-portal volumes this is invisible; as
a default that every future table inherits it is a standing tax paid for nothing, because there is
a strictly better option on the Postgres version we run.

**UUIDv7 (chosen).** A UUID whose leading 48 bits are a Unix-millisecond timestamp, so values
generated in time order sort in key order. It keeps everything UUIDv4 gives — 128 bits, globally
unique, no central sequence, drop-in `uuid` column type so no schema-wide type change — and adds
sequential insert locality, which turns the random B-tree churn back into append-mostly. Postgres
18 (the version the cluster runs, confirmed 18.1) ships `uuidv7()` as a built-in, so this needs no
extension and no application library.

UUIDv7's one disclosed property is that the key **embeds its creation time** to the millisecond.
That is accepted here, not overlooked: every table that gets a v7 key already stores `created_at`,
so the timestamp is not newly exposed; and these keys are surrogate row identities, never secrets
or capability tokens (a bearer value that must be unguessable does not come from this default —
that is a different decision with a different generator). Where a future value must *not* reveal
its creation order — a public-facing reference a tenant could mine for volume — that column opts
out of this default explicitly and says why, the same way STD-0006 makes tenancy a declared
property rather than an inferred one.

## Decision

1. **Every surrogate primary key is `uuid` typed and defaults to `uuidv7()`.** The default lives on
   the column, in the schema — `id uuid PRIMARY KEY DEFAULT uuidv7()` — so the key is generated in
   exactly one governed place. This is the whole point: the generator is a property of the table,
   not a string repeated at every call site.

2. **No `INSERT` names the surrogate key.** Application code and functions omit `id` from the column
   list and let the default fire. An `INSERT` that supplies its own `id` is reintroducing the fifth
   call site this ADR exists to remove, and is a review defect.

3. **A minted non-PK surrogate (an `account_id`) uses `uuidv7()` too.** `resolve_account` assigns a
   new inner-silo `account_id` when a subject is first seen; it is not a primary-key default but it
   is the same kind of value, so it comes from the same generator for the same reasons.

4. **`gen_random_uuid()` is retired for stored keys.** It remains legitimate only for a value that
   is deliberately unordered and non-temporal and is *not* a stored surrogate key — none exist in
   the tenancy schema today. Any reappearance in a migration or in application `INSERT` text is a
   regression against this decision.

5. **BIGINT and application-generated UUIDs are closed.** No table adopts an integer identity, and
   no key is generated in application memory when the schema can default it. A client that needs the
   id it just wrote uses `INSERT … RETURNING id`.

## Consequences

- Migration `0004` adds `DEFAULT uuidv7()` to the four existing surrogate keys (`tenants.id`,
  `principals.id`, `applications.id`, `profiles.id`) and recreates `resolve_account` to mint its
  `account_id` with `uuidv7()`. The tables are empty (data is gated behind ADR-0071 D3), so there is
  no backfill and no rewrite — only the default changes and the next inserted rows are ordered.
- The application's `INSERT` statements for `applications` and `profiles` drop the `id` column and
  its `gen_random_uuid()` literal. The positional parameters are unchanged because the removed value
  was a literal, not a bound parameter, so this is a text edit with no behavioural risk, and it runs
  only behind the same gate.
- New tables inherit the rule by convention now and by a migration check later. The check that would
  lock this ADR asserts (a) every table with a surrogate PK declares `DEFAULT uuidv7()` and (b) the
  string `gen_random_uuid` does not appear in `migrations/` or in application SQL.
- Keys sort by creation time. Anything that must not — a public reference number — is a column that
  opts out explicitly, and the opt-out is where the reason is written.

## Related

- **REQ-0008** — item 6 is the requirement this closes; item 4 (reversible migrations) is settled
  separately.
- **ADR-0024 / STD-0006** — the nested-tenancy silos whose keys must not leak across a boundary,
  which is why BIGINT is closed.
- **ADR-0071** — the backup gate that keeps the tables empty, which is why `0004` needs no backfill.
