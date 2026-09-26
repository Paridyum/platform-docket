---
id: STD-0019
title: A shared component is finished when its worst state is reachable in a story
type: standard
status: accepted
created: 2026-09-02
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, library, service]
applies_to: ["*"]
tags: [design-system, lib-ui, storybook, components, accessibility, rtl, tokens, anti-0004, std-0015, req-0009]
breaking: false
enforced_by:
  - check: ui.every_component_has_a_story
    severity: error
    rationale: >
      §2 is the rule this standard is named for and it was the one nothing checked. The check
      found `Input` exported, styled and published with no story on the run that introduced it —
      three days after the standard was accepted.
  - check: ui.components_declare_no_values
    severity: error
    rationale: >
      §4 and §3. A hardcoded value is a decision made in a component about a theme it cannot
      see, and a physical box property is correct in one direction and wrong in the other —
      invisible to lint, to types, and to everyone not looking at an Arabic page.
---

# STD-0019 — A shared component is finished when its worst state is reachable

## Status

**Accepted**, 2026-09-02.

## Why this exists

Design decisions that must be identical across every Paridyum frontend were stored once per
frontend, by copy. Nothing owned them, nothing versioned them, nothing detected divergence, and
the copies had already diverged in ways that were wrong rather than merely different.

`lib-ui` fixes the storage. This fixes the part storage cannot: **what may go in, what must come
with it, and how a frontend consumes it.** A shared library without those rules becomes a shared
kernel — the thing ANTI-0004 exists to prevent — because the cheapest place to put anything
awkward is the library everybody already depends on.

## The rules

### 1. The tier decides what may live here, and ANTI-0004 supplies the test

| Tier | Contents | Shared |
| --- | --- | --- |
| 1 · Tokens | colour, type, space, radius, motion — semantic names, per-theme values | yes |
| 2 · Primitives | Button, Input, Field, Dialog, Table, Badge — focus, keyboard, ARIA | yes |
| 3 · Blocks | PageHeader, EmptyState, Hero, FeatureGrid, PricingTable, CTA | yes |
| 4 · Product devices | anything encoding a Paridyum rule | **never** |

**The test is ANTI-0004's: *if the business changed its mind, would this file need editing?***
If yes, it is tier 4 and belongs to the product that owns the rule. A component that knows about
entitlements, lifecycle gates or tenancy has been moved out of the repository that decides those
things, and the rule is then owned by nobody and changed by editing a third repository.

The pull toward tier 4 is constant and it always arrives as convenience: *"every product needs
this, so it should be shared."* Every product needing something is evidence it is a rule, not
evidence it is a primitive.

### 2. Every exported component has a story, and the story is the acceptance surface

**A component is not finished when it renders. It is finished when the states that break it are
reachable.** Storybook is where that happens, because an application shows a component in the
state the application happens to produce — which is almost never the state that breaks it.

At minimum, and each as a separate story rather than a toggle:

- **disabled** and **loading** separately, because they are separate contrasts to check
- **long content**, in a language that is not English — a label three times longer is where a
  fixed width or a stray `nowrap` first shows
- **right-to-left**
- **error**, where the component has one

A story that only demonstrates the happy path documents the component without testing it.

### 3. Right-to-left is not a demonstration

**Every component renders in both directions or it is untested rather than merely
undocumented.** Arabic is a shipped locale: `web-paridyum` ships `en` and `ar`, and two locales
is a **security floor** rather than a product choice — CVE-2026-64642 fires on exactly one.

Direction is set on the document, never on a wrapper element. `dir` inherits and CSS logical
properties resolve against it, so a wrapper makes padding behave differently in a story than in
a page — which is precisely the class of bug this surface exists to catch.

Consequently: **logical properties only.** `padding-inline`, `margin-inline-start`, `inset-inline`.
A component using `padding-left` is correct in one direction and wrong in the other, and the
wrongness is invisible to everything except a person looking at it.

### 4. Tokens, never values

A component may not contain a colour, a font, a radius or a spacing value. It refers to the
token layer, which resolves per theme.

**This is what makes a second theme possible at all.** STD-0015 fixes the visual identity and
the estate already carries two surfaces with different palettes; a hard-coded value is a
decision made in a component about a theme it cannot see.

**The contrast test runs per theme × per mode**, and a new theme cannot ship until it clears
them. The bug that test was written for — a chrome band at 2.1:1, with green lint, green types
and green tests, found only in a screenshot — is exactly the bug a second theme reintroduces.

### 5. The accessible name does not move

A component may change what it looks like in a state; it may not change what it is called.

A button whose label becomes a spinner while loading has lost its name to a screen reader and to
every test that finds it by role and name. Announce the state — `aria-busy`, `aria-disabled` —
and leave the name alone.

### 6. The public surface is enumerated, never inferred

`export *` makes the public API a side effect of what happens to be in a directory. A helper
added for one component becomes something a consumer can import, and therefore something the
library can no longer change without a breaking release.

**Re-export explicitly, one name at a time.**

### 7. Consuming it is a pinned version, not a range

A frontend depends on `@paridyum/ui` at an exact version from GitHub Packages. Not a range: a
design system that changes underneath a frontend without that frontend's commit changing is the
same divergence problem in the other direction, and it appears as a visual regression nobody can
attribute to a change.

### 8. Storybook's static build is a CI artifact, never a deployment

`library` is `deployable: false`, so no class could host it. Inventing one is the
vestigial-concept failure `class-registry.yaml` warns about. It is built in CI and downloadable;
it is not a running service, and it is never committed.

## What this does not decide

**Which components exist.** The tier table names likely ones; a component is added when a second
surface needs it, not in anticipation. One consumer is not evidence of a shared decision — it is
evidence of one product's decision, which is tier 4 wearing tier 2's clothes.

**How a component is tested beyond the story.** REQ-0018 and the test registry own that. A story
is the acceptance surface, not the whole test strategy.

## Alternatives considered

- **A checklist in `lib-ui`'s AGENTS.md rather than a standard.** Cheaper, and it binds one
  repository. The rule that matters most — what may *not* enter the library — has to bind the
  repositories doing the pushing, and those are the frontends.
- **Requiring a visual regression test per component.** Stronger than a story, and rejected for
  now: it needs a baseline store and a rendering environment stable enough that a diff means
  something, and neither exists. The story is what can be required today.
- **Allowing tier 4 with a naming convention** — a `product/` directory inside `lib-ui`.
  Rejected outright: it makes the boundary a convention rather than a location, and ANTI-0004's
  whole point is that the boundary must be one nobody can cross by accident.

## Consequences

**Accepted costs.**

- **Stories are work, and most of them will never be looked at.** That is the same trade as a
  test nobody reads until it fails.
- **The RTL rule will be resented.** It doubles the states to check on components whose authors
  are confident direction does not matter, and they will usually be right.
- **Pinned versions mean upgrades are deliberate**, so a fix in `lib-ui` does not reach a
  frontend until someone moves it. That is the point, and it will feel like friction on the day
  the fix is urgent.
- **~~Nothing enforces any of this.~~** Two of the rules are now checked —
  `ui.every_component_has_a_story` and `ui.components_declare_no_values`, added 2026-09-05,
  which is what §2, §3, §4 and §6 are held by. The paragraph below is kept rather than
  rewritten because its prediction was correct: the first check written found a violation
  that had already shipped. What remains unenforced is §5 (the accessible name does not move),
  §7 (pinned versions) and §1 (the tier boundary), and §1 is the one that matters most and is
  the hardest to check — it needs a judgement about what a component knows.

  The original text: a check that every exported
  component has a story is writable and does not exist; so is one that fails on a hex colour in
  a component. Both are owed, and until they exist this is a standard upheld by review.

**Gained.** A component that behaves the same in every Paridyum surface, and a library that
cannot quietly become the place business rules live.
