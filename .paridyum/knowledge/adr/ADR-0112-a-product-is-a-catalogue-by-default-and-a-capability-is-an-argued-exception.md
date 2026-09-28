---
id: ADR-0112
title: A product is a catalogue by default, and a capability is an argued exception
type: adr
status: accepted
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, tool, agent]
applies_to: ["*"]
tags: [product, registry, catalogue, capability, classification, c-0008, c-0006, adr-0050, bet-0004, bet-0005, marketing, docket, hiring, reel, mkt]
breaking: false
enforced_by:
  - check: product.capability_names_its_exception
    severity: warn
    rationale: >
      The citation is the mechanically checkable half — every `capability` entry states its
      ground or its owed re-review. Whether the stated ground is true is judgement and no
      check approximates it (ADR-0002's reason); warn rather than error because a citation
      can lag a review, and a failing-error check on a judgement call is the false
      confidence this estate keeps refusing.
---

# ADR-0112 — A product is a catalogue by default, and a capability is an argued exception

## Status

**ACCEPTED**, 2026-09-27, on the operator's direction. Not `locked`: the checkable half is
declared as `product.capability_names_its_exception` and unbuilt, and the truth of a stated ground
is judgement. This ADR changes a default; it re-opens nothing else.

## Context

**ADR-0050 gave products two kinds that matter here, and the registry makes them different
weights, deliberately:**

- **`catalogue`** — a vertical application Paridyum operates and later franchises (BET-0004,
  BET-0005). It claims a market, so it owes an **MKT-** naming external, non-tenant-derived evidence
  before it is registerable, and **entry to `building` is C-0006 anti-goal 6's gate** — no second
  vertical before the first has a paying customer who renewed.
- **`capability`** — a runtime capability a tenant can buy. It claims **no market**, owes the
  tenant-facing interface instead of market evidence, and meets no anti-goal 6 gate.

**The asymmetry is a drift, and the registry's own history shows the direction it runs.** The
`capability` door is cheaper on every axis, so every product that *can* be argued either way will
be argued capability — not by anyone deciding wrongly, but because it is the path that moves.
Docket and hiring were classified capability on a test that distinguishes nothing: *"a tenant gets
it by holding a tenant."* That sentence is true of every product on the platform, and both entries
describe something that now looks, from the outside, like a service with end customers — RFC-0044
says it outright: *"to sell docket — a tenant spins up under its own domain."* Nothing was decided
wrongly against a rule that existed; there was no rule, and the default ran.

**Reel is the first registration where the route was argued rather than defaulted** — the operator
put the question directly, saw both doors and their costs, and chose `catalogue` with the gate
accepted. That conversation is the one this ADR makes permanent, because its useful property was
that the default did **not** decide.

**The operator's direction, 2026-09-27:**

> *"Prioritise everything as a catalogue; if it's absolutely hard, then only think of going to
> capability."*

## Decision

**1. Catalogue is the default.** A product registers as `catalogue` unless decision 2 applies.
The burden of proof is on the exception, and the exception is stated in the entry itself.

**2. Two grounds qualify, and they are closed. The entry must say which.**

- **(a) A runtime primitive** — a product that *is* the runtime rather than a business on it:
  identity, tenancy, routing, entitlement. There is no separate market to enter, no end customer
  to serve, and no business to operate; C-0008 makes it a product anyway, and the `capability`
  kind is what keeps that honest. These are the set C-0008 itself names.
- **(b) No end-customer business is possible** — the product has no end customers of its own and
  exists to make other products possible. The entry must say **why** in one sentence. This is the
  ground most likely to be pressed and it is closed at exactly this: *"only us, for now"* is not
  it (ADR-0049 D5 already refuses that reading), and neither is a revenue line that has not been
  imagined yet.

**3. "Absolutely hard" means the ground is impossible — never inconvenient.** A market that is
hard to evidence is a market to *measure*; MKT-0005 exists because reel's was measured in a day of
work with a search tool. An MKT- is effort; the anti-goal 6 gate is waiting; and neither is a
reason to take the other door. A ground that is merely effortful is a mis-registration.

**4. Every capability entry names its ground, and the existing entries are re-reviewed.**

- The entry's notes cite this ADR and state **(a)**, **(b)**, or that a re-review is owed under
  this decision. `product.capability_names_its_exception` checks the citation; the truth of it
  stays judgement.
- **entitlement, identity, routing, tenancy — ground (a)**, recorded in their entries. Not
  reclassified; they are the runtime.
- **docket and hiring — re-review owed.** Both have plausible end customers and an operating
  story already written (RFC-0044; the public hiring surface at careers.paridyum.com), and both
  predate this rule. Whether each is a vertical Paridyum operates is a decision **with its own
  artifact**, taken deliberately — not a kind field edited in passing, and not taken here.

**5. Nothing is re-gated retroactively.** docket's and hiring's `building` stage stands as earned
under the rule that existed; the re-review changes what their promotion to `operated` must answer,
not what their history was. C-0006, anti-goal 6, and
`product.sequencing_respects_second_vertical_rule` are untouched — and now bite more often, which
is the intent.

## Consequences

**Accepted costs**

1. **Almost nothing new can be built.** With hosting still pre-renewal, a catalogue-by-default
   registry means the next candidate cannot enter `building`, and the exceptions cover only the
   runtime's own furniture. This is anti-goal 6's cost, moved from a case-by-case surprise to a
   house posture — deliberately.
2. **Every product now starts with an MKT- it must survive.** Some candidates will die at that
   step. Dying there is cheap and honest; dying as an unbuilt tool nobody examined is neither.
3. **The second ground will be pressed at its edges.** "No end-customer business is possible" is a
   judgement dressed as a fact, and the check verifies the citation, never the truth.
4. **Two entries now carry visible debt** (docket, hiring), where before they carried a quiet
   assumption — the registry's honesty improves by becoming less flattering.

**Gained**

- The cheap door stops being the default door. Breadth is no longer free, and the registry's
  counting rule gains a default pointing the same way C-0006 does.
- Every classification decision says, in the artifact, which it is and why retrieval no longer
  depends on remembering the argument.
- C-0008 and C-0006 compose without strain: primitives are products tenants buy; everything with
  an end customer is a business to operate, sequence, and evidence.

**Rejected alternatives**

- **Per-case judgement, no default.** The status quo that produced the drift. The person deciding
  is always the person who wants to ship, and the cheaper door always looks like the obvious one.
- **Everything catalogue, no exception.** The runtime's own primitives would be registered as
  markets or forced into a kind that mis-describes them — corrupting the registry to remove a
  judgement it needs. C-0008 already forces primitives to be products; this ADR keeps their kind
  true.
- **Re-classifying docket and hiring in this change.** Honest classification is a decision about a
  business, not a YAML edit. Doing it here, silently, is the drift wearing the rule's clothes.

## Related

- **ADR-0050** — the product axis and registry this extends; it carries a pointer amendment
- **C-0008**, **ADR-0049** — every runtime capability is a product; the buyer is named first
- **C-0006** — anti-goals 1 and 6, whose sequencing this makes the daily default
- **BET-0004**, **BET-0005** — the catalogue, and operate-before-franchise
- **`product-registry.yaml`** — the entries decision 4 re-reviews; **MKT-0005** — the evidence
  standard the exception must not dodge
- **RFC-0044**, **`web-hiring`** — the operating stories that make docket's and hiring's re-review
  real rather than theoretical
- **`check-registry.yaml`** — `product.capability_names_its_exception`, declared with this ADR
