---
id: BET-0003
title: The application layer is generated against a contract, not authored as templates
type: bet
status: superseded
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: BET-0004
scope: [org, platform, service, spec, template, web]
applies_to: ["*"]
tags: [strategy, architecture, generation, templates, runtime-contract, moat]
breaking: false
trigger: >
  More than one in three applications that reach production needed hand-written code
  outside the spec contract to get there, measured over the first hundred. At that rate the
  contract is not the product — it is a scaffold around bespoke work, and the economics
  revert to the per-vertical linear cost this bet exists to avoid.
review_date: 2027-02-01
enforced_by: []
---

# BET-0003 — The application layer is generated against a contract, not authored as templates

## Status

**ACCEPTED** as of 2026-08-07. Never `locked` (ADR-0022). This is the largest of the three
bets and the one most likely to be judged wrong in its details while right in its direction.

## The wager

**One opinionated business runtime, one machine-readable specification format, and vertical
applications composed against that contract by a model — rather than a catalogue of
hand-authored templates maintained per vertical.**

The runtime is the product. The specifications are the input. The generated application is
output, and per C-0004 output is regenerated, never hand-edited.

## Why not a template catalogue

The alternative — build CRM, store, LMS, clinic and helpdesk as real products and let
customers pick one — is the obvious shape, and it has been the obvious shape since roughly
2014. WordPress multisite, Odoo, Sharetribe and Bubble all occupy it. Three problems:

1. **Cost is linear in verticals and depth is divided among them.** Every template is a
   product with its own domain model, its own edge cases and its own support burden. Ten
   templates is ten products maintained by whoever maintained one.

2. **Shallow loses to the incumbent's free tier, in every vertical, individually.** A
   competent-but-shallow CRM is compared against HubSpot's free tier, not against the effort
   of building a CRM. We would be entering ten fights simultaneously, in each of which we are
   the least-resourced participant.

3. **It contradicts what C-0001 says we are.** An operating system ships system calls, not
   applications. A catalogue makes us an app store — a legitimate business, and a different
   one from the one C-0001 describes.

## Why generation could be right now, having been wrong before

Templates were the right answer when the vertical layer had to be written by a person. That
constraint is the one that changed. The wager is narrow and specific:

**A model can author and maintain the vertical layer, but only against a contract rigorous
enough that it cannot drift.** Without that contract, generated business software is a demo:
it looks right, and its permissions, tenancy and billing semantics are subtly wrong in ways
no reviewer catches until a customer does.

Which is why the asset already exists and was built first, largely by accident of sequencing:

- **Machine-validated artifacts.** Every rule carries frontmatter checked against a schema,
  and `locked` means a check rejects violations (ADR-0002, ADR-0014).
- **A live deny hook.** ADR-0007's `PreToolUse` layer does not advise an agent, it fails the
  tool call. CLAUDE.md's note is the operative one: *"this is not advice you are expected to
  remember — the tool call fails."*
- **Specification-to-code already a first-class path.** The `spec-` and `sdk-` classes exist
  in `class-registry.yaml`, with `stacks: [generated]` and no hand-written source in any
  language.
- **A written rule that green means nothing until watched failing.** The most valuable
  sentence in README.md, and the one that separates a generation pipeline from a demo.

**The governance machinery is not overhead on the way to the product. It is the part
competitors cannot copy quickly, and it is why this bet is available to us and not to a team
that started with the templates.** That claim is the reason this artifact exists; if it is
wrong, this is an ordinary code-generation product with a hosting bill.

## Costs accepted

1. **The contract must be substantially right before anything ships, and nobody knows what it
   must contain.** This is the inversion of the template approach's risk: templates ship
   early and scale badly, this ships late and scales well. If the contract is wrong, every
   generated application inherits the error simultaneously — the blast radius of a bad
   abstraction is the whole customer base rather than one vertical. ADR-0019 already forbids
   implementation before decomposition; this bet raises the cost of getting the decomposition
   wrong.

2. **The contract must be designed for a class of businesses, not for BET-0001's segment.**
   BET-0001 is a beachhead and many markets is the end state, so a contract fitted to solo
   founders makes market number two a rewrite. That bans four assumptions from the outset —
   one operator per business, one business per paying account, and either presence or absence
   of the business's own end users, and a subscription-shaped money flow. Designing for
   breadth while selling to one segment is more expensive now and is the cheaper of the two
   errors; BET-0001's beachhead section carries the list.

3. **Expressiveness has a ceiling, and we cannot see it from here.** Every specification
   language eventually meets a requirement it cannot state. The `trigger` above measures
   exactly this and is the honest place to lose the bet. The failure will not announce itself
   as *"the contract is inexpressive"*; it will arrive as a series of reasonable one-off
   exceptions, each individually justified, and the escape-hatch rate is the only way to see
   it early.

4. **Nobody can debug what nobody wrote.** When a generated application misbehaves, the
   customer cannot read the code, and we did not write it either. Support becomes a matter of
   reading generated output against a spec, which is a genuinely harder skill than reading
   code. This has to be designed for rather than discovered — it argues for generated output
   that is boring, conventional and readable over output that is clever.

5. **It couples the company to model capability and model cost.** A vertical layer that a
   model composes is a vertical layer whose economics move when inference prices move and
   whose quality moves when a model is deprecated. Templates, whatever else is wrong with
   them, do not regress overnight.

6. **The escape hatch is harder here, and it is not optional.** REQ-0013 requires a credible
   export path. Exporting generated code is easy; exporting code the customer can then
   *maintain* is not, since the thing they actually depended on was the runtime and the
   generator. This bet makes C-0006 anti-goal 4 more expensive to honour, and honouring it is
   still required.

7. **There is no impressive early demo.** A template catalogue demos on day one. A runtime
   contract demos when it works. For a company that needs to show progress, this is a
   sustained period with nothing to show, and the temptation to hand-author "just the first
   template" will be constant and will be reasonable every single time.

## What this permits, and what it forbids

**Permitted.** A small number of specifications maintained as *reference inputs* — proof the
contract can express real applications, and the corpus the generator is tested against. These
are test fixtures with a marketing use, not products.

**Forbidden.** Hand-authored applications presented to customers as products, with their own
roadmaps and their own support burden. That is the catalogue, arrived at one reasonable
exception at a time. C-0006 anti-goal 6 is the guard: no second vertical until the first has a
customer who renewed.

The distinction is ownership of the roadmap. A reference spec changes when the contract
changes. A template changes when its customers ask.

## Alternatives rejected

**Build one vertical for real, extract the runtime afterwards.** The Rule of Three honestly
applied, and the least speculative option available. Rejected because it contradicts C-0001
principle 1 — *"platform before product... a capability built once in the platform is worth
more than the same capability built well three times"* — which outranks this artifact. If the
extraction path is ever preferred, it needs an ADR amending that principle, not a bet
quietly working around it.

**A template catalogue now, generation later.** Superficially the safe hedge, and actually the
worst of both: the catalogue's linear cost is incurred immediately, and its existence removes
the pressure that would make the contract rigorous. The templates would become the product by
default, because they would be the thing with customers.

## Related

- C-0006 — anti-goal 1 forbids the catalogue; the not-commodity list names the contract
- C-0001 — principle 1 rejects the extract-afterwards alternative and outranks this
- BET-0001 — the segment chosen to generate the diverse specs this bet needs
- ADR-0019 — decomposition precedes implementation; cost 1 raises its stakes
- ADR-0007 — the deny hook that makes a contract binding rather than advisory
- C-0004 — generated output is regenerated, never edited
- REQ-0013 — the export path cost 6 complicates
- ADR-0022 — why this is a bet rather than an ADR
