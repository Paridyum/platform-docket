---
id: STD-0010
title: Locale-neutral interfaces — how locale, time, text and money are stored and carried
type: standard
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, spec, sdk]
applies_to: ["platform-*", "svc-*", "spec-*", "sdk-*", "lib-*"]
tags: [i18n, l10n, locale, postgres, api, events, unicode, money, observability]
breaking: false
verified_by:
  - TST-0059
enforced_by:
  - check: db.locale_columns_are_bcp47_text
    severity: error
    rationale: >
      `char(2)` is where "locale is not language" stops being advice and becomes
      unrecoverable. The column physically cannot hold `en-IN`, so the region is discarded at
      write time and no later change recovers which of `en-IN`, `en-US` and `en-GB` each row
      meant.
  - check: db.timestamps_are_timestamptz
    severity: error
    rationale: >
      C-0005 states the rule and the only check citing its column rules is `partial`, covering
      the pluralisation half and not this one. A `timestamp` without a zone is a wall-clock
      reading whose instant nobody recorded, and every later conversion invents one.
  - check: db.no_non_unicode_encoding
    severity: error
    rationale: >
      The one item on this list whose remediation is not a migration. Changing a database's
      encoding is a dump and restore of production, so the only affordable moment to reject
      SQL_ASCII or LATIN1 is before the database exists.
  - check: db.money_columns_declare_currency
    severity: error
    rationale: >
      ADR-0033's column shape is a storage convention, so this standard is where it is
      written down, and the check is cited from both.
  - check: openapi.error_responses_declare_a_code
    severity: error
    rationale: >
      The error envelope in §5 is the shape SDK consumers depend on. An error schema with a
      message and no code is the defect, and it is decidable from the document.
  - check: domain.events_carry_no_rendered_text
    severity: error
    rationale: >
      An event is durable and its parameters cannot be recovered after publication, so the
      envelope in §6 has to be rejected at the schema rather than corrected later.
---

# STD-0010 — Locale-neutral interfaces

## Status

**ACCEPTED** as of 2026-08-08. Four checks implemented in `check-migrations.sh` and proven
against planted defects by `scripts/test-checks.sh`; two declared for `tool-forge`. Created
`accepted` per the `new-decision` skill §5b; it locks in a separate commit.

## Why this exists

ADR-0032 decides that machine-readable identifiers carry no locale, ADR-0033 decides how money is
modelled, ADR-0034 decides how a locale is chosen. None of them fixes a syntax, which is the right
scope for a decision and leaves the same gap STD-0006 was written to close: a convention that
exists only inside a gate script is a rule nobody agreed to that nonetheless binds everyone.

This is that syntax. Nothing here is a new decision.

## 1. The four fields, stored

Each of GLO-0006's fields is its own column, and none is derived from another.

```sql
    language          text        NOT NULL,   -- BCP 47 primary subtag: 'en', 'hi'
    locale            text        NOT NULL,   -- BCP 47 tag:            'en-IN', 'sr-Latn'
    timezone          text        NOT NULL,   -- IANA zone id:          'Asia/Kolkata'
    currency          char(3)     NOT NULL,   -- ISO 4217, uppercase:   'INR'
```

**`text`, never `char(n)` or `varchar(n)`, for `locale`, `language` and `timezone`.** The check
rejects the narrow-fixed-width shape specifically, because it is the one that destroys data
silently: `char(2)` truncates `en-IN` to `en` at write time and there is nothing left to migrate.
`sr-Latn`, `zh-Hant-HK` and `az-Cyrl-AZ` are all legal tags and all longer than the two- and
five-character widths people actually reach for.

`currency` is the exception and is `char(3)` deliberately: ISO 4217 alphabetic codes are exactly
three characters and the width is part of the standard rather than a guess.

**A tenant additionally carries `supported_locales`**, per ADR-0034 decision 7. Resolution never
yields a value outside it.

## 2. Time

`timestamptz`, always, in UTC. C-0005 owns the rule; this adds only that the check now exists.

```sql
    created_at        timestamptz NOT NULL,
    scheduled_at      timestamptz NOT NULL,
```

Any `*_at` column is an **instant**. `timestamp` without a time zone is rejected.

`date` is legal and means a calendar date with no instant — a birth date, an invoice period. It is
not a cheaper timestamp, and using it for something that happened at a moment loses the moment.

**A future local appointment is two columns, not one — and neither of them is an instant.** Store
the local date-time it was booked as, plus the IANA zone id, and resolve to an instant at read
time. A zone's offset can change by legislative act between booking and occurrence, and an instant
computed under the old rules then renders at the wrong wall clock: "3pm on the 14th" becomes 2pm.
The instant is precisely the thing that does *not* move — the appointment is. Such a column is
deliberately not an `*_at` instant and is outside the rule above.

## 3. Text and encoding

- **The database is UTF-8.** `SQL_ASCII` and `LATIN1` are rejected. `SQL_ASCII` is not "ASCII
  only" — it is *no encoding at all*, storing bytes uninterpreted, so it accepts every insert and
  corrupts on the first non-trivial comparison.
- **`text` for user-supplied content**, not `varchar(n)`. Where a limit is genuinely required it
  is a `CHECK` on `length()`, so the limit is a stated business rule rather than a number somebody
  sized for ASCII.
- **A length limit counts characters, not bytes and not what the author imagined.** `length()` in
  Postgres counts characters; `😀` is one character and four bytes, and `👨‍👩‍👧` is one grapheme
  the user sees, **five** code points — three emoji joined by two zero-width joiners — and a
  validator that says otherwise.
- **Test fixtures include non-Latin text.** A fixture suite that is entirely ASCII proves the
  ASCII path. At minimum: Devanagari, Arabic, CJK and an emoji with a zero-width joiner.
- **No ASCII assumptions in validation.** Do not reject a `display_name` for containing non-ASCII;
  `रोहन` and `ロハン` are names. Identifiers under C-0005 stay ASCII — that is a rule about
  machine identifiers (ADR-0032 decision 4) and it does not extend to content.
- **Normalise before comparing.** Comparing user text for equality or uniqueness requires a stated
  Unicode normalisation form, applied on write. Two visually identical strings in NFC and NFD are
  different byte sequences and a `UNIQUE` constraint will accept both.

## 4. Money

ADR-0033's shape, restated here because it is a storage convention:

```sql
    amount_minor      bigint      NOT NULL,
    currency          char(3)     NOT NULL,
```

The `_minor` suffix is mandatory. Floating-point types are rejected for stored amounts.

## 5. The error envelope

```json
{
  "error": {
    "code": "PASSWORD_TOO_SHORT",
    "params": { "minimum": 8 },
    "message": "Password must contain at least 8 characters"
  }
}
```

`code` — `SCREAMING_SNAKE_CASE`, stable forever, the only field a client may branch on.
`params` — everything a renderer needs. Part of the API surface, versioned under REQ-0010.
`message` — **optional, non-authoritative, English, for operator debugging.** May change without
a version bump. No client branches on it; no test asserts its text.

An error with a `message` and no `code` is rejected at the schema.

## 6. The event envelope

```json
{
  "event_type": "TodoAssigned",
  "occurred_at": "2026-08-08T06:28:00Z",
  "actor_id": "usr_789",
  "todo_id": "todo_456"
}
```

`event_type` is `<Subject><PastTenseVerb>` in PascalCase. C-0005 fixes that and
`manifest.schema.json` enforces `^[A-Z][A-Za-z0-9]*$` on every declared event; this standard adds
nothing to it. **No `message` field, no exceptions** — an event is durable and its parameters
cannot be recovered after publication.

## 7. Server-rendered boundaries

Six outputs are rendered by a backend because there is no client to render them: **email, SMS,
push notification, PDF, CSV or spreadsheet export, and scheduled report.** These are the
presentation layer, and ADR-0032 does not apply to their *content*.

It still applies to their *inputs*. The rendering takes a template id, a parameter map, and a
resolved locale — never a pre-rendered sentence handed over by a domain service.

```
domain service          notification service
──────────────          ────────────────────
 semantic event    →     recipient  →  resolved locale (ADR-0034 §6)
 + parameters            template   →  rendered output
```

Resolution is against the **recipient**, at send time. Not the actor, not the request.

## 8. Logs and metrics

Structured events with stable identifiers. A log line is `{"event": "user.authenticated",
"user_id": "…", "locale": "es-MX"}` — the locale is a *field*, never the language the line is
written in. A metric name is `auth.login.success`, never derived from user-facing text: when that
text is translated the old series stops receiving points and no alert fires, because nothing
errored.

## 9. Formatting

Every call that renders a date, a number or a currency for a person takes an explicit locale
argument. No call relies on an ambient or default locale — ADR-0034 decision 2 is why, and the
check rejects the call shape rather than the output, because both outputs are plausible dates.

Manual formatting is forbidden. Neither `format!("{}/{}/{}", d, m, y)` nor `amount / 100.0`
produces a correct result outside the author's own conventions, and the second is wrong for JPY
and KWD in every locale.

## Costs accepted

1. **Four columns where most products have one.** Every table holding preferences is wider, and
   the middle levels of ADR-0034's chain will hold duplicate values for a long time.

2. **The checks read migration text, not a live catalogue.** Same limitation STD-0006 records and
   for the same reason: the pre-commit gate has no database. A table created outside a migration
   is invisible to all of them, and the encoding check can only see an encoding a migration
   *states* — the common case where the database is created by infrastructure rather than by SQL
   is out of its reach, and is the largest hole in this standard's enforcement.

3. **Normalisation on write is easy to omit** and there is no check for it. It is here as prose,
   which by this repository's own standard makes it a suggestion until something rejects it.

4. **`char(3)` for currency contradicts §1's own argument** against fixed widths. Stated rather
   than smoothed over: the width is defensible only because ISO 4217 fixes it, and if a future
   requirement needs a non-ISO settlement unit this column is the thing that breaks.

## Related

- ADR-0032, ADR-0033, ADR-0034 — the three decisions this gives a syntax to
- GLO-0006 — the four fields
- ANTI-0008 — the shape §5 and §6 exist to make unavailable
- STD-0006 — migration conventions; these checks run in the same gate
- C-0005 — identifier naming and `timestamptz`
- REQ-0005, REQ-0010 — event and API evolution, which govern `code` and `params`
- REQ-0019 — search, sorting and collation, deliberately deferred
- REQ-0009 — the presentation half, deferred
