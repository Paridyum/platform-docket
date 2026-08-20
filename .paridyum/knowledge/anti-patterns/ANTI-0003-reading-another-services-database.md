---
id: ANTI-0003
title: Reading another service's database directly
type: anti-pattern
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [platform, service, infra]
applies_to: ["platform-*", "svc-*", "infra-*"]
tags: [data-ownership, coupling, hidden-dependency, database]
breaking: false
enforced_by:
  - check: domain.no_foreign_database_access
    severity: error
    rationale: A read against someone else's schema is a dependency that appears in no manifest, so nothing else in the system can see it.
---

# ANTI-0003 — Reading another service's database directly

## The shape

```
svc-reporting  ──SELECT──▶  billing_db.invoices
```

No write, so ANTI-0001 does not fire. No API call, so nothing appears in
`depends.services`. The dependency exists and is invisible to every tool.

The usual justifications are good ones: the API does not expose the field, the join would
need three calls, a read cannot break anything.

## Why it is forbidden

**It is a dependency that no manifest declares, which makes it worse than a synchronous
call, not better.**

- **The dependency graph is now wrong.** `domain.no_service_dependency_cycles` reads
  declared dependencies. An undeclared read edge means the graph the org reasons about is
  not the graph that exists — so cycle detection, blast-radius analysis and ownership
  reports are all quietly false.
- **The owner cannot refactor.** The whole point of owning a schema is being free to change
  it. A reader nobody declared means the owner's tests pass, their contract is unchanged,
  and something else breaks in production. The owner is now constrained by a coupling they
  cannot see and did not agree to.
- **It leaks the internal model.** An API is a deliberate projection — it omits the
  denormalised column, the soft-delete flag, the half-finished migration. Reading tables
  couples you to all of it, including the parts that exist only because of how the owner
  happens to store things today.
- **"A read cannot break anything" is false.** It cannot break the *writer*. It breaks the
  *reader*, silently, at a time chosen by someone else's deploy.

This is the most common way a set of services becomes a distributed monolith, because it is
the one that never shows up in a diagram.

## What to do instead

**Ask the owner for a contract, or subscribe to their events and keep your own copy.**

```
svc-reporting  ──GET /v1/invoices?since=──▶  svc-billing
```

or, for reporting and analytics — usually the right answer:

```
svc-billing  ──publishes──▶  InvoiceIssued  ──▶  svc-reporting  (own read model)
```

The projection is the point. `svc-reporting` shapes the data for its own queries, is not
affected by billing's schema changes, and keeps working when billing is down. That is
strictly better than the direct read it replaces — the direct read only *looks* cheaper
because its costs arrive later.

**If the API lacks the field, that is the conversation to have.** A missing field is a
five-minute request to the owner. A hidden schema dependency is a year of coupling.

## How this is detected

`domain.no_foreign_database_access` fails when a service's configuration or code references
a database, schema or connection string it does not own per its `owns:` block.

Be honest about the limit: **static detection here is weak.** A connection string assembled
at runtime, read from a secret, or pointed at a shared host with per-schema separation may
not be visible to any scan. Expect this check to catch the careless case and miss the
determined one.

**The real control is the same as ANTI-0001's: one database user per service, granted access
only to what it owns.** A foreign read then fails at connect time. The check is the
design-time signal; the grant is the enforcement. That is the pattern the service template
should ship, and it is tracked in REQ-0001 alongside runtime secret delivery, since a
per-service credential is exactly what needs delivering.

## The legitimate exception

**A read replica the owner deliberately publishes as a contract**, declared in the owner's
`manifest.yaml` under `exposes` and documented as such. That is not a foreign read — it is
an interface that happens to speak SQL, versioned and owned like any other.

The distinction is agreement. The owner publishing a replica has taken on the obligation not
to break it. A reader reaching into a schema has imposed an obligation nobody accepted.

Not exceptions: an ad-hoc analytics query, a one-off backfill, "just for the migration",
or a cron job. The last is the worst, because it is a hidden dependency on a schedule.

## Related

- ANTI-0001 — two writers; this is the read-side sibling and shares its enforcement
- ANTI-0002 — the dependency graph this invisibly falsifies
- STD-0002 — `manifest.yaml`, `exposes` and `depends`
- REQ-0001 — per-service credential delivery, which is what makes grants practical
- RFC-0002 — proposes the `owns:` declaration this check reads
