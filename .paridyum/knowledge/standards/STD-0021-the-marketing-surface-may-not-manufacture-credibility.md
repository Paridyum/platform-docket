---
id: STD-0021
title: The marketing surface may not manufacture credibility it has not earned
type: standard
status: accepted
created: 2026-09-06
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, web]
applies_to: ["web-paridyum"]
tags: [marketing, landing-page, design, honesty, conversion, c-0006, adr-0049, std-0015, std-0019, rfc-0020]
breaking: false
enforced_by: []
sources:
  - url: https://github.com/bear2u/my-skills/blob/master/skills/landing-page-guide-v2/SKILL.md
    title: "Landing Page Guide V2 — an agent skill for building landing pages"
    retrieved: 2026-09-06
    supports: >
      The "11 essential elements" a landing page is said to require, in order: keyworded URL,
      logo, hero title and subtitle, primary CTA, social proof in the hero, media, 3-6 benefits,
      4-6 testimonials with photos, 5-10 FAQ items, a final CTA, and a footer. Also the
      anti-generic design rules adopted in decision 2, the prohibition on generic fonts, the
      ShadCN/Framer stack, and the conversion advice rejected in decision 3 — statistics that
      "build credibility", scarcity and countdown timers, and testimonials with faces.
  - url: https://www.tasteskill.dev/
    title: "Taste Skill — a design framework for AI agents"
    retrieved: 2026-09-06
    supports: >
      Brief inference before implementation — read industry, audience, mood, motion depth and
      layout family before choosing anything. Audit-first redesign with preservation rules,
      dark-mode parity by default, and a pre-flight checklist before output ships. Its framing
      that this is "decision architecture" rather than prescriptive styling is the reason it is
      adopted more completely than the guide above.
---

# STD-0021 — The marketing surface may not manufacture credibility it has not earned

## Status

**Accepted**, 2026-09-06.

## Why this exists

Two external guides were brought in to make the landing page better. Both are useful and one of
them, followed literally, would put things on `paridyum.com` that are not true.

That is not a criticism of either source. **A conversion playbook is written for a company that
has customers**, and its advice is sound for one. Paridyum has no customers, no testimonials, no
usage statistics and nothing running on the platform — C-0006 states it, ADR-0049 requires that
nothing ships without a buyer, and a test in `web-paridyum` asserts the product registry agrees.

So the guides cannot be applied as checklists. This standard says which parts are adopted, which
are refused, and why — so the next person who reads a landing-page guide does not have to relitigate
it, and so a section that appears on the page can be traced to a decision rather than to a template.

## Decision

### 1. The honesty floor outranks every conversion technique

**A section that requires a fact we do not have is not built.** Not with placeholders, not with
"coming soon", not with an illustrative example, not with a number rounded from nothing.

This is the whole standard, and everything below is a consequence.

### 2. Adopted from both guides, because they improve the page without asserting anything

From Taste Skill, adopted nearly whole, because it is decision architecture rather than styling:

- **Brief inference before implementation.** Read industry, audience, mood and layout family
  first. For this surface the brief is already written and is not a matter of taste — C-0006
  fixes the category, BET-0006 fixes the reader, and `CONTENT.md` traces every claim to the
  artifact that owns it. **Read those before touching copy.**
- **Audit-first redesign with preservation rules.** A redesign states what it is keeping before
  it states what it is changing. The order of the argument on this page is C-0006's, not a
  designer's, and it survives redesigns by being named as preserved.
- **Dark-mode parity by default.** Already true here and now required rather than incidental:
  `brand.css` defines both modes and the contrast pass runs per mode.
- **A pre-flight check before it ships.** Section 4 below.

From the landing-page guide, the design rules that make a page look considered rather than
generated:

- **Break the symmetric centred column.** Its criticism of "cookie-cutter three-column feature
  grids" lands on the grid this estate shipped on 2026-09-05, and it is fair.
- **Hierarchy through dramatic scale**, not three sizes within 20% of each other.
- **Backgrounds beyond flat**, within STD-0015: the signal is a ground and a graphic and never
  type, so texture is spent on surfaces rather than on text.
- **Motion that respects `prefers-reduced-motion`**, and only on `transform` and `opacity`.
- **Its "does this look AI-generated?" test**, which is a better question than most rubrics.

### 3. Refused, with the reason each one fails here

**Social proof in the hero, testimonials with photographs, and "specific numbers and statistics
(builds credibility)".** We have no customers. Every version of this is a lie — an invented
quote, a real quote from someone who is not a customer, a statistic counted from nothing, or a
logo strip of vendors we buy from arranged to read as customers. The guide is right that these
convert. That is what makes them dangerous rather than harmless.

**Scarcity, countdown timers and "limited spots".** There is no scarcity. Manufacturing urgency
against unlimited supply is a false statement about availability, and C-0006's segment is
explicitly one that cannot afford to be manipulated into a decision.

**An FAQ of invented questions.** Nobody has asked anything. A FAQ is a record of what people
ask; written before anyone asks, it is copy pretending to be evidence. It becomes buildable the
first time a real question arrives twice.

**ShadCN, and a component library brought in beside `lib-ui`.** STD-0019 §1 puts primitives and
blocks in `lib-ui` and ANTI-0004 exists to stop a second home for the same decision. A second
library is not a shortcut, it is the divergence this estate spent 0.2.0 and 0.3.0 removing.

**The "never use system fonts" rule, for now, and this one is a genuine conflict rather than a
refusal.** `brand/brand.css` ships a system stack because **the Paridyum brand typeface has never
been chosen** — an open loop recorded in the decision-debt ledger since 2026-08-22, because the
logo boards fix the colour half of the identity and not the face. The guide is right that a
generic font reads as generic. The answer is to close that decision under STD-0015, not to pick
a face inside a marketing page, which is exactly how an identity ends up owned by whoever last
edited the CSS.

### 4. The pre-flight check, and it is about claims rather than pixels

Before marketing copy ships, every assertion on the page is one of:

1. **Traceable** — `CONTENT.md` names the artifact it comes from; or
2. **Registry-derived** — read from `product-registry.yaml` at build time, so it cannot outrun
   what has been decided; or
3. **Removed.**

And the three questions:

- **Is any stage described as further along than the registry says?** A test asserts nothing is
  `operated`, because the page says nothing runs yet.
- **Does any element imply a customer, a user count, or a revenue figure?** If yes it is refused,
  however it is phrased.
- **Does the page still say what is not true yet?** C-0006: credibility comes from what runs on
  it. The section that admits nothing runs is the most load-bearing on the page, and it is the
  first thing a conversion rubric would delete.

## What this does not decide

**The brand typeface**, which is STD-0015's and is open.

**Whether the page should convert at all.** It should. Nothing here argues for a worse page — it
argues that the techniques requiring evidence are unavailable until the evidence is, and that
everything else remains on the table.

**When testimonials become buildable.** The day a real customer says something real and agrees to
be quoted. That is a date, not a design decision.

## Alternatives considered

- **Adopt the eleven elements and fill the gaps with placeholders**, replacing them as reality
  arrives. This is what the guide expects and what most teams do. It is refused because a
  placeholder testimonial is indistinguishable from a real one to a visitor, and because the
  estate has already recorded what plausible placeholder data costs: `tool-docket`'s person
  template shipped realistic example values and two freshly enrolled interns displayed as the
  same person, with nothing flagging it.

- **Adopt them with a disclosure** — an "example" label on invented proof. Better, and still a
  page whose design assumes evidence it does not have, sized and positioned to read as proof.

- **Ignore both guides.** Cheapest, and it throws away the half that is genuinely good. The
  criticism of the centred symmetric column is correct about this specific page, and the
  brief-inference discipline is close to what this estate already does everywhere except its
  own marketing.

- **Write this as an ADR rather than a standard.** It reads like a decision, but it binds how a
  thing is written every time rather than settling one question once — which is what STD-0002
  says separates the two.

## Consequences

- **The page will be shorter than a conversion rubric wants**, and will score badly against one.
  That is the cost, and it is accepted for as long as the alternative is asserting things that
  are not true.
- **Sections unlock as reality arrives** rather than being built and backfilled: testimonials at
  the first real quote, an FAQ at the first repeated question, statistics at the first workload.
- **`CONTENT.md` becomes a gate rather than a document.** It was research; §4 makes it the thing
  a claim is checked against.
- **Nothing here is enforced by a check.** `enforced_by` is empty and honestly so — "does this
  imply a customer" is a judgement. The registry-derived rule is the one part machinery can hold,
  and it already does.
