---
id: ADR-0062
title: The functional palette is shared storage, and the split from the identity is a file
type: adr
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org, library, web, console]
applies_to: ["*"]
tags: [design-system, lib-ui, palette, contrast, accessibility, std-0015, std-0019, anti-0004, rfc-0015, wcag]
breaking: false
enforced_by:
  - check: ui.status_palette_clears_contrast
    severity: error
    rationale: >
      A refusal that cannot be read is a refusal that is not delivered, and the failure is
      invisible in review — the colour looks correct, the token name looks correct, and the
      text is unreadable only to the person who has to read it. This is the shape of the
      chrome-band defect that reached production in console-docket and was found in a
      screenshot rather than by any gate.
  - check: ui.tokens_are_not_locally_defined
    severity: error
    rationale: >
      The whole point of moving these values is that they stop being redeclared. A console
      that keeps a local `--color-seal` after this ADR has the divergence problem back with
      an extra file in it.
---

# ADR-0062 — The functional palette is shared storage, and the split from the identity is a file

## Status

**Accepted**, 2026-09-05.

## Context

`lib-ui` gained an appearance in 0.2.0, and three of its rules refer to `--color-seal`: Button's
danger variant, the border of an invalid input, and the text of a `Field` error. **The library
does not define that token.** It resolves in `console-docket`, which declares it, and resolves to
nothing everywhere else — so a danger button outside docket renders without its ground.

The token was not invented for those rules. It already exists, twice:

- `console-docket` declares `--color-seal: #a3242b` — "lac, shellac, what a seal is actually
  made of" — with `--color-pending` and `--color-done` beside it, each with a wash, each with
  its own contrast reasoning, in both modes.
- `console-testing` declares the same three, at the same values, in its own 255-line stylesheet.

Two surfaces, one decision, stored twice. That is the copy-by-frontend failure `lib-ui` exists to
end, and it is the reason the base tokens were moved: those two files also redeclare
`--color-paper`, `--color-ink` and `--color-rule` at byte-identical values, 692 lines between
them, with nothing checking that they still agree.

**STD-0015 §4 already ruled on where these do not go, and it is emphatic.** The functional
colours "are the console's and are not in this standard"; `brand.css` "declares no equivalent";
and a console that borrowed a marketing palette for them "would have no way left to say stop".
That paragraph was itself a same-day reversal, so its argument is load-bearing rather than
incidental.

So the question this ADR answers is not *may they be shared* — the duplication settles that —
but **how they are shared without dissolving the line STD-0015 draws.**

## Decision

### 1. The functional palette moves into `lib-ui`, in a file of its own

`styles/status.css`, exported as `@paridyum/ui/status.css`. It declares refusal, outstanding and
settled — `--color-seal`, `--color-pending`, `--color-done` and their washes — in both modes, and
nothing else.

### 2. It is a separate file from `brand.css`, and that is the entire mechanism

STD-0015 §4 says the identity and the functional colours are different kinds of fact. This ADR
does not weaken that; it makes it something you can point at.

**`brand.css` still declares no equivalent.** A surface that wants the identity imports
`brand.css` and gets exactly the three constants and what derives from them. A surface that wants
to say *stop* imports `status.css` as well, deliberately, by name.

Merging the two files would have been fewer imports and would have made the distinction a
paragraph in a standard again — which is what it was when both consoles quietly reimplemented it.
**A boundary that is a sentence gets crossed by accident; a boundary that is a file has to be
crossed on purpose.**

### 3. The values are docket's, adopted rather than re-derived

`#a3242b`, `#8a6a1f`, `#1b6b57` and their washes, in both modes, moved across unchanged.

They carry reasoning already — the naming is argued, the contrast is stated, and they have been
in front of an operator. Picking new ones because the file is new would discard that for nothing,
and re-deriving a palette is how a "move" becomes a redesign nobody asked for.

### 4. Contrast is checked per mode, and the check is what makes this shippable

`ui.status_palette_clears_contrast`: every functional colour is checked against the ground it is
rendered on, in light and in dark, at 4.5:1 for text and 3.0:1 for a border or a graphic.

This is the same obligation STD-0015 §2 places on the identity, for the same reason. It runs in
`lib-ui` where the values now live, so a value edited here fails here rather than in whichever
console notices first.

### 5. `components.css` may refer to these tokens, and says so

The library's appearance layer already refers to `--color-seal`. That becomes legitimate rather
than a dangling reference, and `@paridyum/ui/components.css` documents that a consumer importing
it must import `status.css` too.

**Not solved by making `components.css` import `status.css` itself.** A stylesheet that pulls in
a palette a surface did not ask for would put the functional colours back into every surface by
default, which is decision 2 undone through the back door.

### 6. The consoles delete their copies

`console-docket` and `console-testing` drop their local declarations of both the base tokens and
the functional ones, and import them. `ui.tokens_are_not_locally_defined` is what keeps them
deleted.

## What this does not decide

**Whether the marketing surface should use functional colour at all.** `web-paridyum` renders no
`Field`, no `Input` and no danger button today — it is four sections of prose — so none of the
three rules that refer to `--color-seal` fire on it, and the import buys it nothing right now.

It imports `status.css` anyway, and the reason is the failure mode rather than the present need:
the day somebody adds a contact form, the error text renders in the inherited colour and looks
deliberate. An import that is dead weight is visible in a diff; a missing one is visible only to
whoever cannot read the error. If the page never grows a form the import can go — that is a
question about a page, not about a palette.

**Anything about `--color-chrome`.** Docket declares it and testing does not; one surface is not
evidence of a shared decision, which is STD-0019's rule and it applies here unchanged.

## Alternatives considered

- **Put them in `brand.css`.** Fewest files, one import, and it contradicts STD-0015 §4 in the
  paragraph that standard revised on purpose. Rejected on the standard's own reasoning: a console
  whose refusal red comes from the marketing palette has nothing left that means stop.

- **Leave them in the consoles and remove the references from `components.css`.** Honest, and it
  costs the library its error states: no danger button, no invalid border, no error text — the
  three things every console needs and would then write itself. That is where this started.

- **Component-level indirection** — `--ui-danger`, bound per surface to whatever that surface
  calls its red. Genuinely more flexible, and it buys flexibility nobody asked for: there is one
  refusal colour in this estate and two surfaces that already agree on it. It also puts a
  binding step between a token and its check, which is where a contrast guarantee goes to die.

## Consequences

**Accepted costs.**

- **Another import line in every surface.** Deliberate, per decision 2.
- **A surface that forgets `status.css` gets unstyled danger states** — the current bug, narrowed
  from "always" to "when someone skips a documented import". A check for it is not written.
- **The values are now shared, so changing one changes every surface.** That is the point, and it
  means the next change to the refusal red is a version bump rather than an edit.
