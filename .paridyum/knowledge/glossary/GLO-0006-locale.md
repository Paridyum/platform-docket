---
id: GLO-0006
title: Locale — four independent fields, not one setting called "language"
type: glossary
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org, platform, service, web, mobile, sdk, spec]
applies_to: ["*"]
tags: [i18n, l10n, locale, vocabulary, bcp47, currency, timezone]
breaking: false
enforced_by:
  - check: db.locale_columns_are_bcp47_text
    severity: error
    rationale: >
      The definition's load-bearing half is that a locale is not a language, and the column
      shape is where that stops being a distinction and becomes unrecoverable: `char(2)`
      truncates `en-IN` to `en` on write. The check cannot enforce the concept, only the one
      place the concept is destroyed silently.
---

# GLO-0006 — Locale

## Definition

**A locale is a BCP 47 language tag identifying the conventions by which values are rendered
for a reader. It is one of four independent fields, and collapsing them into one is the
defect this entry exists to prevent.**

```
language   en              what words are written in
locale     en-IN           which conventions render dates, numbers and sort order
timezone   Asia/Kolkata    which instant maps to which wall-clock reading
currency   INR             which unit a monetary amount is denominated in
```

Four fields. Not one field with four meanings, and not one field the other three are derived
from.

## Why four and not one

The obvious reading — store `region: "India"` and derive the rest — fails on the first real
user, and it fails silently rather than loudly:

| Combination | Who this is |
| --- | --- |
| `en` / `en-IN` / `Asia/Kolkata` / `INR` | the modal Indian user |
| `en` / `en-IN` / `America/New_York` / `USD` | the same person, on assignment, billed in dollars |
| `hi` / `hi-IN` / `Asia/Kolkata` / `INR` | reads Hindi, everything else identical |
| `en` / `en-GB` / `Asia/Kolkata` / `USD` | British spelling, Indian clock, dollar invoices |

Every row is legitimate and no two fields covary reliably. A derivation rule is correct for the
first row and wrong for the rest — and wrong in the direction nobody reports, because a date
rendered in the wrong convention is still a date.

## What it is confused with

**Language.** `en` is a language. `en-IN` is a locale. `en-IN`, `en-US` and `en-GB` are the same
language and three different sets of conventions for dates, number grouping, spelling, week
start and sort order. A system storing only `en` has already lost the distinction and cannot
recover it — which is why the storage rule in STD-0010 forbids the column shape that makes this
easy to do by accident.

**Region or country.** A country is not a locale. It is not a timezone either — several
countries have more than one, and the mapping changes by legislative act rather than by
software release. Deriving a timezone from a country is wrong for the United States, Canada,
Brazil, Mexico, Russia, Australia and Indonesia, which each span several. It is wrong in the other
direction across South Asia, where every neighbour of India runs a *single* zone at its own offset
— +4:30, +5, +5:30, +5:45, +6 and +6:30 within a few hundred kilometres — so a regional guess
there is confidently wrong rather than merely ambiguous.

**Writing direction.** Direction is a *property* of a locale, not a synonym for one. `ar-SA` is
right-to-left; so is `he-IL`; `fa-IR` is right-to-left with Extended Arabic-Indic (Persian) digits, a different Unicode block
from the Arabic-Indic digits `ar-SA` uses. Testing
`locale == "ar"` to decide layout direction is the frontend form of the same collapse, and it is
why REQ-0009 item 9 treats right-to-left as a layout property rather than a translation setting.

**Script.** `sr-Latn` and `sr-Cyrl` are one language in two scripts. BCP 47 carries this in the
tag; a two-letter column cannot.

**Currency.** A currency is a property of a *transaction*, not of a reader. The same invoice is
denominated in one currency regardless of who looks at it, and the reader's locale decides only
how that amount is *rendered*. ADR-0033 is the consequence.

## The instant is not a field here

Timezone appears above because it is a rendering convention. The stored value is not: C-0005
already requires `timestamptz` in UTC, so what is stored is an **instant**, and timezone is the
function applied at the presentation boundary to turn it into a reading. A wall-clock time
stored without a zone is not a locale problem, it is missing data.

## Nested tenancy makes this four fields at five levels

Paridyum's tenants are themselves multi-tenant (GLO-0004), so each of the four fields has a
value at the platform, operator, tenant, account and user level, and a request may override —
six steps in ADR-0034's chain. Which one
wins is not obvious and is not left to each service to decide — ADR-0034 fixes the order,
including the case GLO-0005 creates, where an operator with scope over a tenant is reading data
belonging to someone whose conventions are not theirs.

## Related

- ADR-0032 — machine-readable identifiers never carry locale
- ADR-0033 — currency is modelled with the amount, never inferred from the reader
- ADR-0034 — how the four fields are resolved across the tenancy chain
- STD-0010 — how the four fields are stored, carried and rendered
- GLO-0004, GLO-0005 — the tenancy chain and the operator-scope hazard
- C-0005 — `timestamptz`, always UTC, which is what makes timezone a rendering concern
- REQ-0009 — the presentation half, deferred until a frontend framework exists
