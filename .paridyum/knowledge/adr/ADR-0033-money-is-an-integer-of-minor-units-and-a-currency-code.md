---
id: ADR-0033
title: Money is an integer of minor units and an explicit currency code, never a bare number
type: adr
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org, platform, service, spec, sdk]
applies_to: ["platform-*", "svc-*", "spec-*", "sdk-*"]
tags: [i18n, money, currency, billing, postgres, data-modelling]
breaking: false
enforced_by:
  - check: db.money_columns_declare_currency
    severity: error
    rationale: >
      A monetary column with no currency beside it is the defect, and it is invisible in
      review because the schema looks complete. It stops being recoverable the moment the
      table holds rows from two currencies, because nothing in the data says which is which.
  - check: db.money_amounts_are_integral
    severity: error
    rationale: >
      Binary floating point cannot represent 0.10, so a total accumulated in `double
      precision` is wrong by an amount that grows with the number of line items. The failure
      is silent, off by fractions, and surfaces first in a customer's reconciliation rather
      than in a test.
---

# ADR-0033 — Money is minor units and a currency code

## Status

**LOCKED** as of 2026-08-08, with two checks implemented in `check-migrations.sh` and proven
against planted defects by `scripts/test-checks.sh`.

Created `accepted` and locked in a separate commit, per the `new-decision` skill §5b. The gap
between the two was not ceremony: an audit of every countable claim in this artifact found six
factual errors, including a `SUM()` overflow argument placed on a single value seven orders of
magnitude inside `bigint`, and a `financial-record` line that would have led readers to classify
ordinary invoice tables and hard-fail the migration gate. All six were corrected before this
transition.

It locks ahead of ADR-0032, ADR-0034 and STD-0010 on a deliberate line: **ISO 4217 already froze
this content.** Minor units and three-letter currency codes are not ours to get wrong. The other
three encode designs we invented and nothing has yet consumed, and DECISION-DEBT.md records the
condition under which they follow.

## Context

This repository contained the defect until 2026-08-08, in the one place a schema exists at all.
The migration fixture that the tenancy gate treats as *correct* declared:

```sql
CREATE TABLE invoices (
    ...
    amount      bigint NOT NULL,
```

`amount` of what, denominated in what, at what scale. The column is `bigint`, which is the right
type by accident, and there is no currency anywhere on the table. That fixture is what a new
service will be pattern-matched against, and CLAUDE.md's warning applies exactly: *do not pattern
match off a neighbouring file and assume it is correct.*

The decision is forced now rather than when billing exists, because money is the one thing on the
Track A list whose remediation is not a code change. Adding a currency column later means
answering "what currency were the existing rows in" for rows nobody recorded it for, and the
honest answer is a guess applied to financial records. BET-0002 puts pricing at the centre of the
commercial model, so those records will exist and will matter.

## Decision

**1. A monetary value is two columns and two fields, always adjacent.**

```sql
    amount_minor  bigint      NOT NULL,
    currency      char(3)     NOT NULL,   -- ISO 4217, uppercase
```

```json
{ "amount_minor": 100000, "currency": "INR" }
```

`100000` `INR` is ₹1,000.00. The value is an **integer count of the currency's minor unit**, and
the exponent that turns it into a decimal is a property of the currency (ISO 4217), never of the
column, the API or the reader.

**2. The `_minor` suffix is mandatory on the amount.** The suffix is the whole mitigation. A
field named `amount` holding minor units is indistinguishable from a field named `amount` holding
a decimal, and the difference is a factor of one hundred applied to somebody's invoice. The name
is the only place the scale can live where every consumer sees it — an SDK field, a JSON key and
a column all carry it for free.

**3. Never floating point.** Not `real`, not `double precision`, not `float`, and not Rust's
`f32`/`f64`. `numeric` is permitted only for a rate or a ratio, never for a stored amount.

**4. Zero-decimal and three-decimal currencies are not special cases.** JPY has exponent 0, so
`amount_minor: 1000` `JPY` is ¥1,000. KWD has exponent 3. Any code that divides by 100 to display
is wrong for both, which is why the exponent is looked up from the currency and never written as
a literal.

**5. Currency is a property of the transaction, not of the reader.** It is chosen when the amount
is created and does not change because someone with a different locale looks at it. This is the
part GLO-0006 makes explicit and it is the one people get wrong in the pleasant direction: a
tenant's default currency is an input to *creating* a price, not a lens applied at read time.

**6. Rendering is a presentation concern.** `₹1,000.00`, `1.000,00 €` and `¥1,000` are outputs of
locale-aware formatting at a boundary. No service returns a formatted money string as data —
ANTI-0008 forbids it by name, listing `{"price": "₹1,000.00"}` among its forbidden shapes, and
this is the case where the temptation is strongest because a formatted total looks like a value.

**7. Mixed-currency arithmetic is an error, not a conversion.** Summing two amounts of different
currencies must fail rather than silently pick one. A conversion requires a rate, and a rate
requires a timestamp and a source, which is a modelling problem this ADR deliberately does not
solve — it only forbids doing it by accident.

## Consequences

**Accepted costs**

1. **Two columns everywhere, and they can drift.** Nothing in Postgres binds `amount_minor` to
   the `currency` beside it, so a row can be updated in one and not the other. A composite domain
   type would prevent it; it was rejected below.

2. **The exponent table is a dependency.** Displaying any amount needs ISO 4217 exponents, so
   something has to hold them and keep them current. Small, but it is a data dependency that did
   not exist before, and getting it wrong is invisible for USD and wrong for JPY.

3. **`bigint` has a ceiling.** ~9.2 × 10^18 minor units. A single value is never the problem —
   even Zimbabwe's 2008 hundred-trillion-dollar note is 10^16 minor units, two orders inside it.
   A `SUM()` is: an aggregate over many rows of a hyperinflated currency can approach the ceiling
   and overflows silently rather than erroring. Accepted; the alternative is `numeric` and the
   floating-point rule exists to keep that door shut.

4. **The existing fixture had to change.** `tests/fixtures/migrations/pass` violated this and was
   corrected in the same change rather than as a follow-up, because a fixture that models the
   defect is worse than no fixture.

**Gained**

- Financial records are correct to the minor unit by construction, not by care.
- Multi-currency becomes a data question rather than a migration. BET-0002's model can price in
  more than one currency without a schema change.
- The check is implementable today, in a gate that already runs, against a defect that already
  exists in the repository.

**Rejected alternatives**

- **`numeric(19,4)` for amounts.** Exact, arbitrary-precision, and the conventional answer. It
  loses on the scale question: `numeric` says nothing about whether `1000` is a thousand rupees or
  ten, so it needs the `_minor` convention anyway, and then the integer is simpler and faster.
  It also makes floating point look adjacent — the check would have to distinguish `numeric` from
  `real`, and reviewers would not.

- **A Postgres composite type or domain `money_amount`.** Fixes cost 1 properly, and rejected for
  the reason STD-0006 cost 2 rejects a catalogue table: it needs a running database to verify,
  which the pre-commit gate does not have. Worth revisiting alongside STD-0006's own note about live
  catalogue verification, which has the same blocker and would be solved by the same thing.

- **Postgres `money`.** Locale-dependent input and output, single currency per database, fixed
  scale from a server setting. It is the type whose name is the trap.

- **Storing a decimal string.** Preserves scale in the value and pushes parsing into every
  consumer, including the ones that will parse it into a float.

- **Deferring this until billing exists.** Rejected because the fixture above is already wrong and
  is already the thing a new service will copy. The cheap moment is now; the expensive moment is a
  data migration over financial records whose original currency was never recorded.

## Related

- GLO-0006 — currency as one of four independent fields
- ADR-0032 — machine-readable identifiers, which a formatted amount is not
- ANTI-0008 — the forbidden shape a formatted money string falls under, by name
- STD-0010 — the column and field conventions this fixes
- STD-0006 — migration conventions, which the checks extend
- BET-0002 — the commercial model that makes multi-currency likely rather than hypothetical
- STD-0006 §4, ANTI-0007 — the `financial-record` classification, which a monetary table carries
  only when the records are genuinely regulated. It is not a synonym for "holds money": declaring
  it obliges a control framework in `manifest.yaml`, and the migration gate hard-fails without one
