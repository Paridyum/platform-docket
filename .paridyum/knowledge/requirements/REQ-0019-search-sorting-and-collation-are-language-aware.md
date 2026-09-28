---
id: REQ-0019
title: Search, sorting and comparison over user text are language-aware, not byte-aware
type: requirement
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web]
applies_to: ["platform-*", "svc-*", "web-*"]
tags: [i18n, search, collation, sorting, unicode, postgres]
breaking: false
trigger: >
  The first feature that searches user-supplied text, or orders it for display, across more
  than one row — a search box, a sorted list of names, a uniqueness constraint on a title.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0019 — Search, sorting and comparison over user text are language-aware

> **Accepted 2026-09-28 by ADR-0122 D9 — its trigger has fired:** the portal sorts user text — tenants, members, orgs — across more than one row. What follows
> binds from today; the trigger below is kept as the record of why.

## Status

**DEFERRED** as of 2026-08-08. No service, no search feature, and no search engine chosen.

This is the part of the Track A internationalisation set that is genuinely not startable, and
saying so is the point. STD-0010 §3 fixes encoding and normalisation because those are
unrecoverable later. Everything below depends on choices — which engine, which analyser, which
collation provider — that would be inference rather than retrieval if made today.

Recorded now because the reasoning is loaded now and will be cold when a search box appears. That
is the same argument REQ-0009 makes about itself — and REQ-0009 turned out to be *wrong* about
its own scope: its title left out the expensive half, which is why ADR-0032, ADR-0033, ADR-0034
and STD-0010 had to be written separately.

## Requirement

**Every comparison, ordering and search over user-supplied text shall use a stated, language-aware
collation and analysis strategy, rather than the byte ordering or the case rules that a default
happens to provide.**

## Numbered items

1. **A stated collation provider and version.** Postgres supports libc and ICU collations, they
   order differently, and a libc collation can change with a glibc upgrade — which silently
   corrupts every index built on it. ICU with a pinned version is the usual answer. State it, and
   state the version, because an index is ordered data.

2. **Ordering for display is not ordering for storage.** An index has one collation; readers have
   many. `ORDER BY name` is correct for one locale and wrong for the rest — German, Swedish and
   Turkish disagree about where the same letters go. Decide where display ordering happens and
   accept the cost of it.

3. **Case folding is language-dependent.** Turkish dotless `ı` is the standard example and it is
   not academic: `lowercase("I")` differs between `en` and `tr`, so a case-insensitive lookup
   written for English fails for a Turkish user on a letter that exists in both.

4. **Normalisation before comparison.** STD-0010 §3 requires a normalisation form on write. This
   item is the query side, which has to use the same one, including in `UNIQUE` constraints and
   deduplication.

5. **Accent and diacritic sensitivity is a product decision.** Whether `café` matches `cafe` is a
   choice, and it differs by field: probably yes for a search box, probably no for an identifier.
   Made once, written down, not per-query.

6. **Tokenisation is language-specific and defaults are wrong.** Chinese, Japanese and Thai have
   no whitespace word boundaries. Devanagari and Arabic have their own segmentation rules. A
   whitespace tokeniser returns one enormous token for a Thai sentence and nothing errors.

7. **Stemming and stop words per analysed language**, and a stated behaviour for text whose
   language is unknown or mixed — which is the common case, not the exception.

8. **A stated strategy for multi-language corpora.** One column holding several languages needs
   either a language discriminator per row or an analyser that does not assume one. Both are
   legitimate; neither is a default.

9. **Search results respect tenancy.** Whatever engine is chosen, the two isolation boundaries of
   GLO-0004 apply inside it, and STD-0009's rule holds: the **denial** is the assertion. A search
   index is a second copy of the data outside Postgres RLS, which makes it the most likely place
   for an isolation defect to appear and the least likely place for anyone to look.

10. **A test corpus that is not English.** Fixtures in Devanagari, Arabic, CJK and Latin-with-
    diacritics, with expected orderings written down. Without them every item above is untested
    prose.

## Why this is not startable today

Items 1 and 5 look decidable now, and deciding them would mean deciding them without knowing
whether search is Postgres full-text, an external engine, or both — and those have different answers for collation, analysis
and item 9. Item 9 in particular cannot be specified before the engine exists, because it is the
one item where the wrong answer is a cross-tenant leak rather than a bad sort order.

The right moment is the first search feature, where items 1, 4 and 9 go into the schema and the
query layer rather than into prose after the fact.

## Related

- STD-0010 §3 — encoding and normalisation, which is the part that could not wait
- GLO-0006 — locale, and why sort order is one of its conventions
- GLO-0004, STD-0009 — the two isolation boundaries item 9 inherits
- ADR-0032 — machine identifiers, which are exempt from all of this and stay ASCII
- REQ-0009 — the presentation half of internationalisation
- STD-0006 — migration conventions, where a collation pin on a column would be written; REQ-0008
  governs how that migration ships
