---
id: ADR-0096
title: Localized content in the database is a per-locale keyed JSON object carrying every active locale
type: adr
status: accepted
created: 2026-09-14
locked_at: null
supersedes: []
superseded_by: null
scope: [platform, web, tool]
applies_to: ["*"]
tags: [i18n, l10n, locale, content, database, jsonb, req-0009, glo-0006, adr-0032, adr-0033, adr-0095, std-0010]
breaking: false
enforced_by: []
---

# ADR-0096 — Localized content in the database is a per-locale keyed JSON object

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0122.** **D1 and D2 are amended by ADR-0122 D4.** A localized
> field carries every **content locale of the tenant that owns the row**, checked by a per-tenant
> function — not every estate locale by a literal `CHECK`. `en` and `ar` stay the **software
> locales** of every web product (a security floor), not a content requirement on tenants.

## Status

**Accepted**, 2026-09-14. Prompted by ADR-0095: job postings are the **first authored content the
estate stores in the database in more than one language**. The pattern is decided once, here, rather
than per-table, so the second localized table does not invent a second shape.

## Context

The estate is bilingual (en, ar) and **REQ-0009 binds it**; `web-*` apps render under `[lang]` and
GLO-0006 treats locale as data, not a setting. Three kinds of text have three different homes, and
conflating them is the mistake this ADR prevents:

- **UI chrome** — button labels, headings — lives in the app's message catalogues
  (`messages/en.json`, `messages/ar.json`). Not data; not this ADR.
- **Identifiers, enums, money, dates** — a `posting_id`, an `engagement_class`, a currency code —
  are **locale-independent by ADR-0032/ADR-0033** and stay plain typed columns. Never localized.
- **Authored content** — a job title, a description, a list of responsibilities — is written per
  language by a person and stored in the database. This had no representation, and ADR-0095's
  first cut stored it single-language, which on a bilingual site is a regression.

## Decision

**1. A localized content field is a `jsonb` object keyed by locale code, and it MUST carry every
active estate locale.** Today that is `{"en": …, "ar": …}`. A scalar field is
`{"en": "…", "ar": "…"}`; a list field is `{"en": ["…"], "ar": ["…"]}`. A `CHECK` asserts that each
active locale key is present and non-empty, so a half-translated row cannot be stored — the database
enforces presence (it cannot enforce that the Arabic is *good*, only that it exists).

**2. The active-locale set is a schema fact, and widening it is a migration.** Adding a third
language is a deliberate change — a migration that widens the `CHECK` and backfills existing rows —
not a silently-optional key. This keeps "which languages a claim exists in" answerable from the
schema rather than from whatever happens to be in the data.

**3. Identifiers, enums, money and dates are never localized (ADR-0032/ADR-0033 carried).** Only
human-authored prose and lists take the jsonb-per-locale shape. A locale code that leaks into an
identifier, or a currency rendered as a localized string, is the failure ADR-0032 exists to prevent.

**4. Reads resolve to the request locale (GLO-0006); the whole object is never rendered.** A page
under `[lang]` selects `field[lang]`; an API returns the resolved language or the full object only to
an authoring surface. The fallback when a locale is somehow absent is an explicit decision at the
read site, not a silent empty string — but decision 1's CHECK makes absence not-storable in the
first place.

## Consequences

**Accepted costs.**
- **A CHECK proves presence, not quality.** An operator can paste the English into the Arabic field
  and satisfy the constraint. That is a review problem, not a schema one; ADR-0095's
  `job_posting_events` records who authored each change so a bad translation has an owner.
- **Widening the locale set touches every localized table.** That is the honest cost of making the
  active set a schema fact rather than an open map, and it is the safer direction: a new locale that
  silently defaults to empty on old rows is the bug this prevents.
- **jsonb is less queryable than columns.** Filtering postings by a word in the Arabic description is
  a jsonb path query, not an index-friendly column. Acceptable — localized *content* is read by key
  and rendered, not filtered on; anything filtered on (class, status, id) is a plain column.

**Gained.** One shape for localized content across the estate; a bilingual site whose *data* is
bilingual, not only its chrome; and a schema that refuses a half-translated public claim.

**Rejected alternatives.**
- **A side table `<entity>_content(entity_ref, locale, …)`.** Normalized and the textbook i18n
  shape, but every read joins and every RLS policy on the content table must re-derive the parent's
  visibility (an `EXISTS` subquery against the parent's tenant/status). For content that is always
  read with its parent and never independently, the jsonb-on-the-row shape reuses the parent's one
  RLS policy and freeze/append-only triggers unchanged. Revisit if a localized entity ever needs to
  be queried or secured independently of its parent.
- **A single `text` column plus a `lang` column, one row per language.** Turns every entity into N
  rows and makes "one posting" a join to reassemble; worse than both above.
- **Leave content single-language and localize later.** Rejected — REQ-0009 binds now, and ADR-0095
  is unmerged, so the cost of doing it right is a schema edit, not a data migration.

## Related

- **ADR-0095** — job postings, the first consumer; its localized columns take this shape.
- **ADR-0032** — identifiers are locale-independent; decision 3 keeps them so.
- **REQ-0009, GLO-0006, STD-0010** — i18n is binding, locale is data, interfaces are locale-neutral.
