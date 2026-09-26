---
id: STD-0015
title: The visual identity is one mark and one palette, and the signal is never type
type: standard
status: accepted
created: 2026-08-22
locked_at: null
supersedes: null
superseded_by: null
scope: [org, library, web, console]
applies_to: ["*"]
tags: [brand, design-system, identity, accessibility, contrast, lib-ui, wcag, std-0002]
breaking: false
enforced_by:
  - check: ui.brand_accent_is_not_a_text_colour
    severity: error
    rationale: >
      #35ecde on #f9f5ec is 1.36:1. The failure is invisible in review — the colour looks
      correct, the token name looks correct, and the text is unreadable only to the person
      who has to read it. This is the same shape as the chrome-band defect that reached
      production in console-docket and was found in a screenshot rather than by any gate.
  - check: ui.tokens_are_not_locally_defined
    severity: error
    rationale: >
      §4 has relied on this check by name since this standard was written and never declared
      it, which is ADR-0014's failure in the file that names it: a control cited in prose is
      not a control. Declared 2026-09-05 alongside ADR-0062, which moves the functional
      palette into lib-ui and needs this to keep the consoles' copies deleted.
  - check: ui.mark_is_not_redrawn
    severity: error
    rationale: >
      A logo diverges by re-export and by optimisation, not by decision. Byte-identity of
      the path data against the source board is mechanically checkable and needs no
      judgement about whether a curve moved enough to matter.
  - check: ui.theme_blocks_agree
    severity: error
    rationale: >
      The dark palette is written twice — once under prefers-color-scheme and once under
      data-theme — because plain CSS has no way to write it once. Two copies of one fact
      drift, and the drift is invisible: each theme looks right in the state its author
      tested, and wrong in the other.
  - check: ui.brand_assets_carry_no_ground
    severity: error
    rationale: >
      The source boards are 1920x1080 artboards with a full-bleed background rect. An asset
      that keeps one paints a cream or teal box behind itself on every surface it is placed
      on, and does it silently.
---

# STD-0015 — The visual identity

## Status

**`accepted`, and lockable.** The checks named in `enforced_by` are declared in `check-registry.yaml` with
`owner: tool-forge` and are therefore not yet a build break estate-wide. Each is
implemented and defect-tested today in `libs/lib-ui/brand/tools/check.py`, which plants each
violation and watches it fail. It is created `accepted` and locked in a separate commit per
the `new-decision` skill §5b — ADR-0029 carries two errata because that ordering was not
followed there.

**On the number.** `docs/superpowers/specs/2026-08-22-web-surface-and-design-system-design.md`
§3.6 anticipated STD-0015 for *the design system* — its tier model, contribution model and
SemVer policy. That document is prose and states that it binds nothing. The number was
allocated here under the `new-decision` rule that ids are issued next-unused with no gaps. The
design-system standard takes the next free number when it is written; it is a different
artifact from this one and always was.

## Context

Paridyum had no visual identity in code. Before 2026-08-22 the string `#35ecde` appeared in no
file in the estate, and the console wordmark was the word "Paridyum" set in the chrome band's
type. Five logo boards arrived from the operator on that date, and with them three problems
that would otherwise each be discovered separately and late.

**The boards are not assets.** Each is a 1920×1080 Illustrator artboard with a full-bleed
background rect. Dropping one into a header paints a cream rectangle across it. Something has
to derive consumable assets, and if nothing owns that derivation each consumer does it once,
differently.

**The accent cannot do the job a brand accent is normally asked to do.** `#35ecde` against the
cream ground is 1.36:1 — below every threshold WCAG defines, for any text at any size. It is
not a marginal call to be settled per surface. Every board uses it correctly, as a ground the
mark sits *on*; nothing in the boards tells a developer that, and the natural reading of "the
brand accent" is "the colour links are".

**The estate has already been bitten by exactly this class of defect.** `console-docket`'s
chrome band shipped white text on pale periwinkle. Lint passed, types passed, and
`palette.test.ts` passed while asserting the contrast of a palette that was never served. It
was found by looking at a screenshot. A second theme, with a lower-contrast accent than the
first, reintroduces that bug with better odds.

The competing pressure is real and is not dismissed: a rule that the accent may not be type
makes the marketing palette harder to use than a normal one, and the obvious escape — darken
the brand teal until one token serves everywhere — is cheap and would work. It is rejected
below, but it is not a bad instinct.

## Decision

### 1. The identity is three constants, read off the boards

`--brand-signal` `#35ecde` · `--brand-bone` `#f9f5ec` · `--brand-obsidian` `#000000`.

These are not adjustable in a stylesheet. Every other value in `brand.css` is derived — a
tint, a shade, or a hue-matched darkening — and each carries the note saying so.

### 2. The signal is a ground and a graphic. It is never type.

Not body text, not a heading, not a link, not an icon that carries meaning alone, not a border
that distinguishes a state. It is a surface the mark and black type sit **on** (14.2:1), and
it is fill in a graphic.

Type in the accent is set in `--color-signal-ink` — the signal hue darkened until it clears AA
on bone, `#0a7a72`, 4.78:1.

**In dark this rule does not apply**, because the constraint is the cream ground and nothing
else. `#35ecde` on `#000000` is 14.2:1, so `--color-signal-ink` resolves to the signal itself
below the dark breakpoint. The second token is a cream-ground tax, and it is correct that it
disappears when the ground does.

### 3. The mark is re-anchored, never redrawn

A derived asset carries its board's `d` attributes byte-for-byte and is positioned with a
`transform`. Consequently derived assets **must not be run through an SVG optimiser**: SVGO
and its equivalents rewrite path data, and doing so is indistinguishable — to a gate and to a
reader — from someone having edited the mark.

### 4. A console takes the identity; it keeps its own functional colours

**Revised 2026-08-22, before this standard was first committed, by operator decision.** The
paragraph below replaces one that said the opposite — *"a console takes the mark, and no other
part of this"* — and the reasoning for the reversal is given rather than the old text quietly
deleted, because a rule that flipped within a day of being written is exactly the kind whose
argument the next reader needs.

**The identity is one identity.** A console may take the ground, the ink and the accent from
`libs/lib-ui/brand/brand.css`: bone, obsidian and the signal. There is one company, and an
operator console that shared a mark with the marketing surface and nothing else looked like a
different product wearing the same badge.

**The functional colours are the console's and are not in this standard.** Refusal, outstanding
and settled — `--color-seal`, `--color-pending`, `--color-done` — say what the tool *means*, not
who it belongs to. This standard has nothing to say about them, `brand.css` declares no
equivalent, and a console that borrowed a marketing palette for them would have no way left to
say stop.

**What the original paragraph was protecting is preserved by §2, not by separation.** The stated
worry was a console inheriting a colour it could not read. That risk is real and it is handled by
the two-token split: `--color-signal` is a ground and a graphic, `--color-signal-ink` is the one
to set type from, and the same check reads both surfaces. A console on the obsidian ground gets
the signal at 14.2:1 and the second token collapses into the first, which is §2 working as
written rather than an exception to it.

**The ANTI-0004 test still applies, and it now answers differently for the palette.** If the
business changed its mind about how Paridyum looks, would this file need editing? Yes — and that
is the point: it is *one* file, and it is this one. The previous answer assumed the console's
palette was a product decision. It is not; it is an identity decision that had been made
separately by accident, which is the copy-by-frontend failure §3 of the web-surface design
documents as already in progress.

**Still forbidden, and this is the part that matters:** a console may not redefine the three
constants, and may not set type in `--color-signal`. `ui.tokens_are_not_locally_defined` and
`ui.brand_accent_is_not_a_text_colour` apply to a console exactly as they apply to a marketing
surface. `console-docket` carries its own copy of the second check in `palette.test.ts`; the
duplication is deliberate, because the alternative is inheriting a class of bug and not its
detection.

### 5a. The theme is a **choice**, not only a system preference

**Added 2026-09-15.** Until this date the palette had exactly one switch:
`@media (prefers-color-scheme: dark)`. That is the *operating system's* preference, and a person
had no way to say otherwise. For a marketing page that is defensible. For a console somebody sits
in for hours — and `/dashboard` is one — it is not: the choice to read dark at noon, or light at
midnight, belongs to the reader.

**Three states, and the middle one is the default.**

| Root attribute | Resolves to |
| --- | --- |
| `data-theme="light"` | light, whatever the OS says |
| `data-theme="dark"` | dark, whatever the OS says |
| *absent* | follow `prefers-color-scheme` |

**Written as three blocks, in this order, and the order is load-bearing:**

1. `@theme { … }` — the complete **light** palette. Every token gets its value here and nowhere
   else; a token whose only definition lives inside a media query or an attribute selector has no
   value in the third state.
2. `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }` — the dark
   overrides. The `:not()` is what lets an explicit light choice win over a dark OS.
3. `:root[data-theme="dark"] { … }` — the same dark overrides again, so an explicit dark choice
   wins over a light OS.

Blocks 2 and 3 carry **identical** values. That duplication is the cost of doing this in plain CSS
with no preprocessor, and it is a drift risk rather than a theoretical one — someone will edit one
and not the other. `ui.theme_blocks_agree` asserts they match, which is the same bargain §2 strikes
for the accent: the rule survives because a check reads it, not because a comment asks nicely.

**`@theme` still may not appear inside `@media`.** The Tailwind v4 hoisting trap the header of
`brand.css` documents — and which shipped dark to every `console-docket` operator regardless of
their system — is unchanged by this. Blocks 2 and 3 override custom properties on `:root`; neither
is a `@theme` block.

**The choice is remembered per reader and is never sent anywhere.** It is a display preference, not
tenant data: it lives in that browser and does not reach a tenant's row, a log, or a model
(ADR-0080). It must also be applied **before first paint** — a theme resolved after hydration is a
flash of the wrong ground on every navigation, which is worse than having no toggle at all.

**Contrast is asserted in both themes, not one.** `--color-ink-faint` is the case that proves why:
4.04:1 on bone, 5.65:1 on obsidian. The same token is below AA in one theme and above it in the
other, so a check that reads one theme would call the palette clean.

### 5. Clear space is one bar-width of the mark

Expressed as `0.22em` against the mark's rendered height so it scales with the logo, and
applied as padding rather than margin so an adjacent margin cannot collapse it.

## Consequences

**Accepted costs.**

1. **Two accent tokens instead of one, and a rule to remember.** Every developer touching a
   marketing surface must know that `--color-signal` is not the one you set type in. The check
   is what makes this survivable; without it the rule is a sentence in a file nobody reads.
2. **Derived assets carry coordinates in board space.** `mark.svg` has a `viewBox` starting at
   0 and a path whose numbers are in the 700–1100 range, offset by a transform. It looks wrong
   until you read the header. This is the direct price of byte-identity being checkable.
3. **No SVG optimisation.** The assets are larger than they need to be — a few hundred bytes
   each, over a handful of files, which is not a real cost, but it is a real constraint on the
   build and it will surprise someone.
4. **The gate is Python inside what will be a TypeScript package.** Ported when `lib-ui`
   becomes a package; a wart until then.
5. **The typeface is not decided here.** The boards fix the colour half of the identity and
   not the type half, so this standard governs less than its title suggests.

**Gained.** The identity exists in exactly one place. The failure that reached production in
`console-docket` is mechanically rejected in the second theme rather than found in a
screenshot. The derivation from board to asset is repeatable by a command instead of by hand.

**Rejected alternatives.**

- **Darken the brand teal so one accent token serves.** The cheapest fix and the wrong one:
  it changes the identity to suit a stylesheet. The boards are the identity, and a rule that
  edits them to be more convenient is not governing the brand, it is replacing it.
- **Let each surface decide whether the accent is legible enough.** This is the current
  estate-wide practice by default and it is what produced the chrome-band defect. "Legible
  enough" is a judgement made by someone who already knows what the text says.
- **Put the assets in `console-docket/public/`.** It is the only frontend that exists and it
  would work today. It is also precisely the copy-by-frontend failure the web-surface design
  documents as already in progress — `console-testing/src/app/globals.css` is a byte-level
  copy of docket's, header comment included.
- **Store the identity only as a knowledge artifact.** Governance without a consumable asset
  means every frontend renders the mark from its own copy, which is the same failure one layer
  down.
- **Ship the boards as the assets.** They carry a full-bleed ground and a 1920×1080 viewBox.
  There is no configuration in which this works.
