---
id: REQ-0013
title: A tenant can export everything it owns and leave without asking permission
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [portability, export, lock-in, trust, data-governance, sales]
breaking: false
trigger: >
  The first tenant stores data it created rather than data we seeded — the first row that
  would be lost if the tenant were deleted and that nobody but the tenant can reproduce.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0013 — A tenant can export everything it owns and leave without asking permission

## Status

**DEFERRED** as of 2026-08-07. There are no tenants and no data, so every rule here would
today govern nothing — ADR-0008's test.

Deferred is not unimportant, and the reason is unusual for this repository: **this requirement
is deferred as an obligation and current as an argument.** Nothing needs building yet. But the
promise it makes is already load-bearing in C-0006 anti-goal 4, and it already constrains
ADR-0023, whose per-tenant restore gap is solved by the same machinery. The trigger is
deliberately the *first tenant-authored row*, not the first paying customer, because that is
the moment the obligation becomes real rather than the moment someone asks about it.

## Requirement

**A tenant shall be able to export the complete set of data it owns, in a documented format,
without contacting support, without a contractual negotiation, and without a Paridyum employee
running anything by hand — and the export shall be complete enough that a competent engineer
could stand the tenant's business up elsewhere from it alone.**

## Nested tenancy gives this requirement two readings

ADR-0024 made Paridyum's tenants themselves multi-tenant, so "export everything it owns" is two
different products and this requirement covers both:

- **An operator exporting a tenant** — every account's data, the tenant's own configuration, and
  the account boundaries themselves. The recipient is our customer.
- **An account exporting its own slice** — one account's rows, without any other account's, and
  without the tenant's platform-internal configuration. The recipient is our customer's customer,
  and the obligation is the operator's rather than ours, which means **the runtime must provide
  the mechanism the operator uses to honour it.** An operator who cannot let their customers
  leave inherits C-0006 anti-goal 4 wholesale, and they inherited it from us.

Item 2's completeness rule therefore differs by reading: an account export that included another
account's rows is a cross-tenant leak rather than a generous export, so *complete* and *scoped*
have to be satisfied simultaneously at the inner boundary. That is the one place where this
requirement and ADR-0024 can conflict, and ADR-0024 wins.

## Numbered items

1. **Self-service.** Initiated by the tenant, from the product, at any time, on any plan
   including the lowest. An export that requires asking is a hostage negotiation with extra
   steps.
2. **Complete.** Every row the tenant created, every uploaded file, and the schema needed to
   interpret both. Completeness includes the boring parts — audit trails, historical records,
   soft-deleted rows the tenant can still see. If we would need it to reconstruct their state,
   they need it.
3. **Documented format.** Standard and boring: table-per-file with a schema description, files
   as files. Not a proprietary archive, and not a format whose only reader is Paridyum.
4. **Interpretable without us.** Internal surrogate keys and platform-internal identifiers must
   be resolvable from the export itself, or the export is a puzzle rather than a copy.
5. **Tested by restoration, not by inspection.** The export is verified by loading it somewhere
   that is not Paridyum and confirming the business's data is intact. An export nobody has
   restored is a file of unknown value — README.md's rule about green checks, applied to a
   deliverable.
6. **Available after cancellation, for a stated window.** Export access must outlive the
   subscription by a published period. Cancelling is exactly when people discover they need it.
7. **Its existence is documented publicly**, in the product and in the marketing, before a
   customer thinks to ask.

## Why this is a requirement rather than a courtesy

**The switching cost is not the moat, and treating it as one is C-0006 anti-goal 4.**

The argument that says *"to leave Paridyum a customer would have to rebuild their entire
company, so churn will be low"* describes a hostage rather than a moat. It is true, and it is
the wrong thing to rely on: it produces the worst available word of mouth, it caps the deals we
can close because it is the first objection any evaluating customer raises, and it removes the
pressure that would otherwise keep the product worth staying for.

Three further reasons, in ascending order of how hard they are to argue with:

- **We already demand this of our own vendors.** REQ-0012 requires every external dependency be
  *"approved, scoped, and exitable"*. Refusing our customers what we require of our suppliers
  is not a strategy, it is an inconsistency someone will eventually point out in public.
- **BET-0003 makes it harder and therefore more necessary.** When the application layer is
  generated, the customer never wrote the code and the thing they depended on was the runtime.
  Exporting the generated code is easy; exporting something they can *maintain* is not. That
  cost is recorded as BET-0003 cost 6, and it does not excuse the obligation.
- **ADR-0024 needs the same machinery for a different reason.** Row-level extraction of one
  tenant from a shared database is per-tenant restore and per-tenant export — the same
  operation, pointed in two directions. Building it once serves an operational need that will
  arrive whether or not anyone requests an export.

**And it converts the objection into a sales asset.** A documented, tested, self-service export
is the strongest possible answer to *"what if I want to leave"*, which is a question the honest
answer to is currently *"you cannot."*

## What deferral does not permit

Deferring the obligation does not permit designing in a way that makes it impossible later.
Two specific hazards, both cheap to avoid now and expensive to unwind:

- **Data whose meaning lives only in application code.** A status column of opaque integers
  with no documented mapping is not exportable in any useful sense, and the mapping has to be
  captured when the column is created — REQ-0003's classification argument, applied to
  semantics instead of sensitivity.
- **Cross-tenant surrogate keys with no tenant-local equivalent.** If a tenant's rows can only
  be interpreted by joining against platform-global tables they will never receive, the export
  is structurally incomplete no matter how it is formatted.

## Related

- C-0006 — anti-goal 4, the posture this requirement implements
- REQ-0012 — exitable dependencies; the consistency argument
- ADR-0024 — shares the row-level extraction machinery, now per-tenant and per-account
- BET-0003 — cost 6, why generation makes this harder
- REQ-0003 — the classification-at-creation argument, reused for semantics
- ADR-0018 — why a deferred requirement carries a trigger
