---
id: REQ-0009
title: A user interface is accessible, translatable and locale-correct from its first screen
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [accessibility, i18n, l10n, wcag, frontend]
breaking: false
trigger: >
  The first user-facing interface is written — the first screen, page or component a
  person outside the organisation will look at.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0009 — A user interface is accessible, translatable and locale-correct from its first screen

## Status

**DEFERRED** as of 2026-08-07. Paridyum has no user interface. `pd-coming-soon` is an
archived-estate landing page outside the greenfield scope (ADR-0009).

This requirement exists **because** it is deferred. Accessibility and internationalisation
are the two disciplines most reliably discovered too late, and both get dramatically more
expensive with every screen shipped without them. Retrofitting them is a rewrite of the
view layer; doing them from the first component is nearly free.

Recording it now is deliberate: the reasoning is loaded today and will be cold when the
first UI is written. That is the entire argument for a trigger-based requirement, and it is
a better argument than the one originally used to leave this out.

## Requirement

**Every user-facing interface shall meet a stated accessibility conformance level, contain
no hardcoded user-facing strings, and render correctly for locales other than the author's
— checked automatically, from the first component rather than before the first release.**

## Numbered items

1. **A stated WCAG level.** Pick one — 2.2 AA is the common floor and the one most
   procurement and legal regimes reference. An unstated level means every reviewer applies
   a different one.
2. **Automated checks in CI.** `axe` or equivalent on every component and page. Automated
   tooling catches roughly a third of real issues, which is worth having and must not be
   mistaken for conformance.
3. **Keyboard and screen-reader paths.** Every interactive element reachable and operable by
   keyboard alone, with a visible focus indicator. This is the single highest-value manual
   check and the one automation misses most.
4. **Semantic HTML before ARIA.** A `<button>` beats a `<div role="button">` with four
   handlers. Most ARIA in the wild makes accessibility worse.
5. **Colour and contrast.** Contrast ratios enforced, and no information conveyed by colour
   alone.
6. **No hardcoded user-facing strings.** Every string goes through the translation layer
   from the first component. This is the item that is nearly free at the start and a
   full-codebase sweep later — and the reason this requirement is worth writing before the
   first screen rather than after.
7. **Translation key standards.** Naming, namespacing, and what happens to a missing key —
   fall back visibly in development, never render the raw key to a user.
8. **Pluralisation and interpolation via the library.** String concatenation to build a
   sentence is broken in most languages; plural rules vary from one form to six.
9. **RTL support considered from layout.** Retrofitting right-to-left into a layout built
   on physical directions is a rewrite; logical properties cost nothing up front.
10. **Timezone and locale correctness.** Store UTC (C-0002 already mandates `timestamptz`),
    render in the user's zone. Dates, numbers and currency formatted by locale, never by
    string formatting.
11. **A supported browser and device matrix**, stated, so "works on my machine" has a
    definition.

## Why this is not startable today

There is no interface, no framework chosen, and no `template-web` to put the tooling in.
Items 1 and 11 are decidable now but would be decided without knowing the audience,
jurisdiction or stack, which is how a conformance target becomes a number nobody can
justify.

The right moment is the creation of the first frontend template, where items 2, 6 and 7 can
be built into the scaffold rather than mandated as prose — which is the difference between
a rule and a default.

## What this requirement does not cover

Added 2026-08-08. This requirement is titled *"a user interface is…"*, and that framing turned
out to leave out the expensive half. Internationalisation is not a property of the view layer: an
API returning `"Pending"`, an event carrying `"Rohan created a task"`, a `char(2)` locale column
and an `amount` with no currency are all locale defects, all in Rust and SQL, and none of them is
reachable from a requirement about screens. They are also the ones that cannot be fixed later —
a rendered sentence in a durable event has already discarded the parameters that would have
allowed it to be re-rendered.

That half is now ADR-0032, ADR-0033, ADR-0034 and STD-0010, `accepted` and enforced rather than
deferred, because none of it depends on a frontend framework existing.

**This requirement keeps the presentation half unchanged**, and its trigger is unchanged: the
first user-facing interface. Items 1–9 and 11 are frontend concerns. Item 10 is now split — the
storage rules it names are enforced by STD-0010 today, and the rendering rules remain here.

## Related

- ADR-0009 — the archived estate, which is why the existing landing page is out of scope
- C-0002 — UTC timestamps, which item 10 depends on
- REQ-0003 — privacy; personal data and locale data frequently overlap
- ADR-0013 — the lifecycle a frontend template would be designed through
- GLO-0006 — locale, language, timezone and currency as four independent fields
- ADR-0032 — the locale-neutral core, which is the half this requirement's title omitted
- ADR-0033 — money as minor units and a currency code
- ADR-0034 — locale resolution across the tenancy chain, which item 10's rendering half needs
- STD-0010 — storage and contract conventions, already enforced
- REQ-0019 — search, sorting and collation, deferred separately
