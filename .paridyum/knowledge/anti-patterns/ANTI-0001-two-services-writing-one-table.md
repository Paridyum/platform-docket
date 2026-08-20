---
id: ANTI-0001
title: Two services writing the same table
type: anti-pattern
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [data-ownership, coupling, single-writer, database]
breaking: false
enforced_by:
  - check: domain.entity_has_single_owner
    severity: error
    rationale: Two manifests claiming the same table is the one boundary violation that is mechanically detectable, and the one that is effectively permanent once shipped.
---

# ANTI-0001 — Two services writing the same table

## Status

**LOCKED** as of 2026-08-07.

## The shape

Two services both hold write access to one table.

```
svc-billing   ──write──▶  invoices  ◀──write──  svc-reporting
```

It arrives innocently. `svc-reporting` needs one extra column on `invoices`, and adding a
write is a two-line change where calling an API is a week.

## Why it is forbidden

**The schema becomes a shared mutable interface with no owner and no version.** Every
column is now part of a contract between two codebases that neither declares. Renaming a
column requires a synchronised deploy. Adding a `NOT NULL` breaks the other service at
runtime, in production, with no compile-time signal and no failing test.

Martin Thompson's Single Writer Principle is the underlying rule: **data is owned by one
execution context.** Two writers means every consistency and locking problem that principle
exists to avoid, now distributed across two deployables and two release cadences.

It is also **effectively permanent**. Unwinding it means finding every write path, building
the API that should have existed, migrating callers, and removing the grant — while both
services are live. The cost of the mistake is paid once, cheaply, at design time. After
that it compounds.

## What to do instead

**One service owns the table and is its only writer. Everyone else reads through a
contract.**

```
svc-reporting  ──GET /v1/invoices──▶  svc-billing  ──write──▶  invoices
```

or, when the reader must not depend on the writer being up:

```
svc-billing  ──publishes──▶  InvoiceIssued  ──▶  svc-reporting  (own projection)
```

The second is usually right for reporting and analytics: the reader keeps its own copy,
shaped for its own queries, and outages do not propagate. C-0002 already fixes the event
naming.

If the reader genuinely needs a column that does not exist, that is a **request to the
owner**, and it is a feature of the arrangement rather than friction — it forces the
question "whose data is this?" to be answered once, out loud.

## How this is detected

`domain.entity_has_single_owner` compares the `owns:` block across every repository's
`manifest.yaml` and fails when two declare the same table.

**The declaration is not the enforcement.** A manifest is a statement of intent; nothing
stops code writing to a table it did not declare. The actual control is a **distinct
database user per service, granted write access only on the tables that service owns** —
so an undeclared write fails at the database, not at review. The check verifies that the
declarations and the grants agree; the grant is what stops the write.

This is why the check is `error` but not sufficient on its own.

## The legitimate exception

**A migration, bounded and written down.** Moving ownership from one service to another has
a window where both can write. That is permitted when the window is declared — an RFC
naming the two services, the cutover, and the date the second grant is revoked — and it is
not permitted as a steady state.

There is no exception for "it is only a small write" or "reporting is read-mostly." Those
are how the steady state arrives.

## Related

- C-0002 — table naming, and one repository as one bounded context
- ADR-0001 — polyrepo; each service is a separate deployable with its own release cadence
- RFC-0002 — proposes the `owns:` declaration this check reads
- ANTI-0003 — reading another service's database directly, the read-side sibling
