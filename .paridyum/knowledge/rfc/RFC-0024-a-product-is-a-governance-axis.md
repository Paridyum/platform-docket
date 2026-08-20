---
id: RFC-0024
title: Where product-level business knowledge lives, and what addresses it
type: rfc
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0050
scope: [org]
applies_to: ["*"]
tags: [governance, product, strategy, artifact-types, market, scope]
breaking: true
enforced_by: []
---

# RFC-0024 — Where product-level business knowledge lives, and what addresses it

## Status

**ACCEPTED**, resolved by ADR-0050.

## The question

C-0008 was written on 2026-08-17 and states that **every runtime capability Paridyum builds is
a product a tenant can buy**. BET-0004 says the catalogue is the product. BET-0005 says
Paridyum operates what it franchises. Three artifacts, all live, all agreeing that this company
is a set of products rather than a single one.

**Nothing in the governance model can address a product.**

`scope` selects repository classes (ADR-0015). `applies_to` narrows to repository names. Both
address the estate by how it is built. Neither can express *the artifacts binding the invoicing
product*, and no third field exists. So the question is not whether products are first-class —
C-0008 already decided that for the runtime — but why the thing that governs the runtime cannot
name them.

This surfaced from the other end. Asked where per-product market research, pricing and
go-to-market documentation should live, the honest answer was that there was no *where* to
propose, because there was no addressing scheme to hang one off.

## What is already decided and must not be rebuilt

| Already decided | Where |
| --- | --- |
| Commercial strategy belongs in this repository, not a sibling | RFC-0005 |
| A wager is a `BET-`, with what would kill it and when it is judged | ADR-0022 |
| Market posture, category, and six anti-goals | C-0006 |
| Every runtime capability is a product; Paridyum is tenant zero | C-0008, ADR-0049 |
| The catalogue is the product; Paridyum operates what it franchises | BET-0004, BET-0005 |
| A product's technical contract is a `spec-` repository | RFC-0009 Revision 2 |
| Lists that must be extended live in registries | ADR-0014, ADR-0015, C-0005, ADR-0022 |
| External facts are cited and dated | RFC-0020, ADR-0035 |
| A fact has one home | RFC-0015 |

RFC-0005 is the closest precedent and it answered the same question one level up: *does
commercial strategy belong here, and as what?* It concluded yes, and as a new artifact type.
This RFC is that question asked again once the company turned out to have more than one product,
and it reaches a compatible answer by a different route — because the gap this time is an
addressing gap, and a new type alone would not have closed it.

## The obligation nothing could satisfy

C-0006 contains this sentence:

> every vertical Paridyum enters must be recorded in an artifact naming the evidence that
> selected it, and that evidence may not be tenant-derived.

**There was no artifact type that sentence could name.** A `BET-` is the closest and it is the
wrong shape: a bet's load-bearing field is `trigger`, the evidence that would *kill* it, and
this obligation is about the evidence that *selected* something. A `REQ-` describes what the
platform must do, not what the market was observed to be. An `RFC-` is a question.

So the requirement was live, written into the constitution, and binding on nobody — not through
neglect but because the container did not exist. That is the same defect RFC-0005 found when
positioning had nowhere to go, and it has the same consequence CLAUDE.md names: **the absence
of a written answer is not neutral, it is an instruction to guess.**

The ROADMAP's phase 3 exit gate reads *"vertical one chosen"*. It is unmet, and when it is met
the evidence would have had nowhere to be written down.

## Options considered

### 1. Add `product` to the `scope` enum

The cheapest-looking option. `scope: [org, service, invoicing]`.

**Rejected, and it would have been a real defect rather than an inelegance.** `scope` selects
repository classes, and a product is orthogonal to class in both directions: one product spans
`platform-*`, `spec-*` and a web repository, while `platform-tenancy` serves every product
there will ever be. Putting both in one enum makes `scope` a union of two incompatible
selectors, and `forge context <repo>` could no longer resolve it — given `invoicing`, it has no
class to match and no product to match either, because it was handed a repository.

artifact-registry.yaml's header already names this failure in the general case: *"three
independent axes, and conflating them is what made the old rules special-cased."* This would
have been the fourth axis folded into the first.

### 2. A repository per product

`product-invoicing`, each carrying its own knowledge subtree and its own vendored snapshot.

**Rejected on RFC-0005's argument, which transfers without modification.** C-0001 principle 4
requires every rule be discoverable from a single repository checkout, offline. A per-product
knowledge subtree means an agent reasoning about two products needs two checkouts, and an agent
reasoning about whether a *third* product should exist — the C-0006 anti-goal 6 question — needs
all of them plus the ability to count. ADR-0001 vendors knowledge redundantly rather than
centralising it precisely so one checkout suffices.

It also duplicates the governance machinery per product, which is how the machinery drifts.

### 3. A `products/<slug>/` subtree of free-form documents

A directory per product holding market research, GTM and pricing as ordinary markdown.

**Rejected.** This is the `docs/` directory README.md's first paragraph exists to reject:
unvalidated prose in a repository whose founding argument is that unenforced prose is a
suggestion. The precedent is concrete rather than theoretical — `glossary/` sat in exactly this
state for the life of the repository, listed in the layout, absent from every artifact type, and
nobody noticed because it was empty. ADR-0024 could not be written until the vocabulary was.

### 4. An artifact type per document family

`MKT-`, `GTM-`, `MET-`, `ICP-`, and as many more as a startup documentation stack names — on the
order of twenty types.

**Rejected, and this is the option a generic answer to the original question would have
produced.** Each type costs a directory, a prefix, a registry entry and a status vocabulary,
and nineteen of the twenty would be empty on the day they were created. The empty-`glossary/`
argument applies to each one individually.

Worse, it puts the *taxonomy* into the type system, where extending it is a schema-shaped act.
The taxonomy is the thing most likely to change as the company learns what it actually needs
to write down. A list that must be extended, held somewhere extending it is expensive, is the
exact defect this repository has now fixed four times.

### 5. A fourth axis, one new type, and the taxonomy as registry data

Three parts:

- **`product:`** — a frontmatter array, orthogonal to `scope` and `applies_to`, resolving by the
  same rule ADR-0015 gave repositories: absent means every product, as a classless repository is
  governed rather than exempt.
- **`MKT-`** — one new artifact type, for dated market and customer evidence, requiring
  `sources`. It exists because C-0006's obligation has no container, and it is the only part of
  the taxonomy that needed a *type* rather than a slot.
- **`product-registry.yaml`** — the product list, the lifecycle, and a `dossier` block naming
  what each kind owes at each stage. Adding a required document kind is one line here.

**Chosen.** It is the only option where the addressing gap and the missing container are closed
by the same change, and where the ~150-item taxonomy that prompted the question becomes data
rather than twenty new types.

## What the answer turns out to be, restated

Most of a startup documentation stack **already has a home here** once the axis exists, and
saying so is more of the value than the new machinery:

| Kind of document | Where it lives under this proposal |
| --- | --- |
| Per-product pricing | `BET-` with `product:` set — BET-0007 decides the shape org-wide, a product bet decides the numbers |
| Per-product requirements, PRDs | `REQ-` with `product:` set |
| Market research, TAM/SAM/SOM, personas, JTBD, competitor analysis | `MKT-` — the new type |
| Product technical contract | the `spec-` repository RFC-0009 Revision 2 already decided |
| Risk register | REQ-0015, product-scoped |
| Threat model, privacy, supply chain, incident response | REQ-0003, REQ-0004, REQ-0014, REQ-0015 — already exist |
| Vision, mission, values, category, anti-goals | C-0001, C-0006 — already exist |
| GTM, sales playbook, support SLA | `slots` in the dossier, owed at `franchised`, no type yet |
| Legal, IP, HR, finance, fundraising | RFC-0013, and deferred until incorporation |

**Executed agreements, cap tables and actual financial records are not knowledge artifacts and
this proposal does not make them one.** They are operational records read by humans and
regulators, not rules that bind code, and a repository vendored read-only into every checkout
is the wrong home for them. The taxonomy that prompted this question does not draw that line;
this RFC draws it explicitly so nobody has to re-derive it.

## The failure mode this introduces, named where it will be read

A product registry makes registering a product cheap. **Cheap registration of verticals is
precisely what C-0006 anti-goal 6 forbids**, and a governance system that made the forbidden
thing convenient while remaining neutral about it would be worse than no system.

So the registry is not neutral. It counts, and
`product.sequencing_respects_second_vertical_rule` reads the count. That converts one clause of
C-0006 from a thing someone must remember into a thing the registry disagrees with out loud —
and the limit of that claim is stated in ADR-0050 and in the check's own note, because C-0006
carries an empty `enforced_by` deliberately and this must not be read as overturning it.

## Recommendation

Adopt option 5. Register Paridyum itself as product zero on the same terms as everything else —
C-0008's principle applied to the governance layer, where an unregistered operator governing a
registry of the things it sells would be the same asymmetry C-0008 forbids in the runtime, and
harder to see.

Resolved by ADR-0050.

## Related

- ADR-0050 — the decision, the registry, and the type
- RFC-0005 — the same question one level up; the rejected-sibling-repository argument this reuses
- C-0008, ADR-0049 — every runtime capability is a product; Paridyum is tenant zero
- C-0006 — the evidence obligation that had no container, and anti-goal 6
- ADR-0015 — the resolution rule this mirrors deliberately
- ADR-0022, ADR-0014, C-0005 — the three prior instances of list-in-code
- RFC-0015, ADR-0040 — why the registry does not list repositories
- RFC-0009 — the `spec-` product contract
