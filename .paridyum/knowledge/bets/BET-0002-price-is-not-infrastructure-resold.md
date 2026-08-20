---
id: BET-0002
title: Price is charged per running business, never as infrastructure resold
type: bet
status: superseded
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: BET-0007
scope: [org]
applies_to: ["*"]
tags: [strategy, pricing, unit-economics, margin, packaging]
breaking: false
trigger: >
  Infrastructure cost for the median active tenant exceeds the floor price at a plan the
  target segment will actually pay, and no density work closes the gap — measured once two
  hundred tenants are live. At that point the flat shape is subsidising consumption it
  cannot see, and either the price shape or BET-0001's segment has to change.
review_date: 2026-12-01
enforced_by: []
---

# BET-0002 — Price is charged per running business, never as infrastructure resold

## Status

**ACCEPTED** as of 2026-08-07. Never `locked` (ADR-0022) — a price shape is the most
frequently falsified claim a company makes.

## The wager

**A customer pays a flat monthly price per running business, with usage allowances generous
enough that most never think about them. Infrastructure consumption is a guardrail on abuse,
not a line item, and never a source of margin.**

C-0006 anti-goal 2 states the prohibition. This bet states the positive shape and what would
kill it.

## Why not infrastructure resold

The obvious model — pass cloud costs through with a markup, so revenue scales with customer
success — is the one to refuse, for four reasons that compound:

1. **It makes the customer's bill scale with our inefficiency.** Every unoptimised query and
   every oversized container becomes revenue. That is an incentive pointed exactly backwards,
   and it survives long after anyone remembers choosing it. A margin that improves when
   engineering gets worse is a structural defect, not a pricing decision.

2. **It invites comparison against a published price list.** Cloud providers post their rates.
   The moment a bill is itemised as storage, bandwidth and compute, the customer can compute
   our markup exactly, and the conversation becomes about that number instead of about the
   product. We would be competing on transparency against companies whose scale we cannot
   match.

3. **It charges most at the moment a customer is winning.** A tenant whose traffic just
   tripled receives a bill that tripled, in the same week they are least able to absorb it.
   That is the worst possible timing for a surprise, and it converts our best outcome into
   their worst experience.

4. **BET-0001's segment cannot absorb variance at all.** A solo founder with no revenue and a
   bill that moves unpredictably churns on the variance itself, independent of the amount.
   Predictability is worth more than fairness to this customer.

## Why flat-per-business could be right

- **It prices the thing we actually claim to sell.** C-0006 says we sell the runtime a
  business runs on. One running business, one price. The unit of value and the unit of billing
  are the same object, which is the property that makes a price easy to explain and hard to
  resent.
- **It is forecastable on both sides.** The customer knows their bill; we know our revenue per
  tenant. At BET-0001's ACV, forecastability is worth more than upside capture.
- **It rewards efficiency in the correct direction.** Every unit of density improvement is
  our margin, not our lost revenue. That aligns the pricing model with ADR-0024, where density
  is already an existential constraint rather than an optimisation.
- **Expansion happens on a count the customer controls and understands.** More businesses, or
  a higher tier — not a meter they cannot predict.

## Costs accepted

1. **We capture none of the upside from a tenant that becomes very large.** A customer doing
   enormous volume on a flat plan is pure margin compression. This is a real and deliberate
   giveaway, accepted because BET-0001's segment makes that outcome rare enough to be a
   problem worth having, and because tiers can be introduced later without changing the
   *shape*. Adding a tier is a repricing; adding a meter is a rewrite of the promise.

2. **A flat price at the low end must cover an idle tenant, so the floor cannot be zero and
   probably cannot be very low.** BET-0001 cost 3 is the same constraint from the other
   direction: schema, DNS, certificate, backups and monitoring share all accrue whether anyone
   logs in or not. This bet therefore forbids the growth tactic of unlimited free tenants. A
   free tier, if one ever exists, must be time-boxed or capacity-boxed rather than open-ended,
   because an unbounded free tier under a flat model is a commitment to pay for other people's
   abandoned projects indefinitely.

3. **Generous allowances are a promise we cannot yet price.** "Most customers never think
   about limits" requires knowing the distribution of consumption, and there are no tenants.
   The first allowances will be guesses. They must be set from measured data as soon as data
   exists, and the honest expectation is that the first numbers are wrong.

4. **Abuse must be handled by the guardrail rather than by the bill.** Under consumption
   pricing, a tenant mining cryptocurrency pays for it. Under this model they do not, so
   quota enforcement and egress limits become correctness features rather than billing
   features — they move from the finance side of the system to the platform side, and they
   have to exist before the price does.

5. **Some genuinely fair usage will be refused.** A legitimate tenant will eventually hit a
   ceiling and be told no, where a metered competitor would simply have charged them. That
   conversation is the cost of predictability, and it will not always be the right trade.

## Alternatives rejected

**Per-seat.** The standard SaaS answer, and wrong here: BET-0001's customer has no seats. One
person, sometimes zero other users for months. A seat-priced product bills this segment
approximately nothing, which is not a discount strategy but an absence of a business model.
Worth revisiting if BET-0001 dies and the agency segment is adopted, where seats are real.

**Percentage of the tenant's revenue or transaction volume.** Excellent alignment — we win
when they win — and rejected on three counts. It requires being in the payment flow, which is
a large regulatory surface. It is unenforceable for the many templates that process no
payments at all. And it makes our revenue a function of their success in the segment least
likely to have any, so it prices at approximately zero exactly where BET-0001 places us.
The strongest candidate if the wedge moves to commerce.

**Free platform, paid marketplace revenue share.** Requires third-party supply that does not
exist and cannot exist before the runtime contract is stable (BET-0003). This is a plausible
*second* revenue line and an impossible first one, because it depends on the ecosystem that
depends on the product working.

## Related

- C-0006 — anti-goal 2, the prohibition this bet elaborates
- BET-0001 — cost 3 there is the floor constraint here
- ADR-0024 — density is what makes a flat price viable
- REQ-0012 — vendor and compliance governance; a payment-flow model would fire its trigger
- ADR-0022 — why this is a bet rather than an ADR
