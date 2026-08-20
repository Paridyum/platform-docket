---
id: BET-0008
title: The estate is cost of goods, and infrastructure is never a metered line item
type: bet
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [strategy, infrastructure, proxmox, unit-economics, cost-of-goods, rfc-0019, c-0006]
breaking: false
trigger: >
  Any ONE of the three kills the bet; they are ordered by how early each can fire.
  (1) DENSITY — fully-loaded substrate cost per tenant on owned or rented metal is not at
  least half the managed-cloud equivalent for the same workload, measured once fifty tenants
  exist. Owning the substrate then buys nothing but operational burden, and the honest move
  is managed Kubernetes and an end to being a hardware business.
  (2) GUARDRAIL — the quota and egress enforcement that BET-0007 inherits from BET-0002
  cost 4 proves unbuildable, or a single tenant is demonstrated able to consume enough
  substrate to threaten the floor without tripping it. Unmetered is a promise that rests
  entirely on the guardrail; without one the promise is not conservative, it is unfunded.
  (3) OPERATIONAL — running the estate consumes more than four engineer-days per month
  before the first catalogue product is live. At that rate the hardware business is eating
  the software business, which is the failure RFC-0019 accepted cost 1 named and this bet
  is the commercial half of.
review_date: 2027-02-01
enforced_by: []
---

# BET-0008 — The estate is cost of goods

## Status

**ACCEPTED** on 2026-08-18, proposed 2026-08-17. A bet, not a decision: per ADR-0022 this artifact can never be
`locked`, and the `trigger` above is the condition under which it should be killed.

**This closes RFC-0019 open question 1**, which has been open since 2026-08-12 and which
RFC-0019 decision 3 says must be answered before that decision means anything:

> *If those other virtual machines are **sold** rather than run for ourselves, that is
> infrastructure resold, and C-0006 anti-goal 2 is explicit… Which of the two is intended has
> to be stated before this decision means anything.*

It is answered here as **for ourselves**, and BET-0009 handles the part that is sold, on
separate metal, under a different unit of sale. The two bets are deliberately separate
artifacts because they can be killed independently.

## The wager

**The Proxmox estate exists to make BET-0007's flat floor structurally defensible. Tenant
workloads run on it as cost of goods. No tenant is ever shown a vCPU, a gigabyte, or a
transfer figure, and no revenue line is derived from one.**

C-0006 anti-goal 2 states the prohibition. BET-0007 carries it forward. **This bet states why
the estate exists at all given that prohibition**, which no artifact currently does — RFC-0019
builds the datacentre and records the tension as unresolved, and its accepted cost 2 says so
in as many words: *"both are true; the tension is recorded rather than resolved."*

## The argument anti-goal 2 has been missing

C-0006's own amendment note of 2026-08-15 states the defect this section repairs:

> *An artifact that outsources its argument to a lower-precedence one inherits that artifact's
> mortality. A constitution citing a bet must still be able to state its own reason.*

Anti-goal 2 currently delegates its entire reason to BET-0007, which is a bet and therefore
mortal. Below is a reason that does not depend on any bet, because it is arithmetic.

**You cannot resell infrastructure at a markup, because the retail price of rented compute is
at or below your own wholesale cost.**

**MKT-0001 is the measurement, and this bet deliberately does not restate it.** Its finding, in
one line: a retail vendor will sell your prospective customer a gigabyte of memory for less than
an Indian colocated rack costs you to produce one.

⚠️ **Do not copy a figure out of MKT-0001 into this file.** Those are a competitive market's
published prices on one day — MKT-0001 records one vendor repricing three times during 2026 — and
a number restated here is a snapshot that cannot be superseded when the evidence is. That
separation is what ADR-0050 created the `MKT-` type for.

There is no markup available. Anti-goal 2 is therefore not a moral position about pricing; **it is
a description of a market you would lose in**, and it now has a reason of its own that survives
BET-0007 being killed.

**Two corollaries worth stating, because both are counter-intuitive.**

1. **Owning metal does not buy price. It buys jurisdiction.** MKT-0001 finds Indian colocation and
   rented European metal within noise of each other, with the European side marginally cheaper.
   Owning only beats renting *in India*, and the only reason to be pinned to India is RFC-0014's
   residency boundary. **This is the strongest available argument for RFC-0014 and the weakest
   available argument for RFC-0019 as a cost measure**, and both should be said plainly rather
   than one of them quietly assumed.

2. **The saving is real against managed cloud, and it is an order of magnitude, not a margin.**
   MKT-0001 measures rented dedicated metal against on-demand hyperscaler compute at the same
   vCPU and memory. That gap is what funds a low floor. It is not a gap you can resell — the
   customer can see both published prices — but it is entirely a gap you can *absorb* and price
   against, which is what this bet wagers on.

## Why this could be right

1. **The unit of value and the unit of billing stay the same object.** BET-0002 named this
   property and BET-0007 inherited it. One running business, one price. Infrastructure moving
   into cost of goods is what keeps that true as the substrate grows.

2. **Density becomes margin rather than lost revenue.** ADR-0024's nested tenancy already makes
   density an existential constraint rather than an optimisation. Under a metered model, every
   density improvement is revenue given away; under this one, it is margin kept. The pricing
   model and the isolation model point the same direction, which is rare enough to be worth
   protecting.

3. **The estate has no line-item surface, so it cannot be compared.** Anti-goal 2's second
   objection — *"invites comparison against a published price list"* — is structural rather
   than presentational. A bill with no resources on it cannot be compared to one that has them.

4. **It keeps the residency claim honest.** RFC-0014 makes a region a legal boundary. A tenant
   who buys "a running business in India" and is never billed for compute has no reason to
   discover, or care, which physical machine served them — which is exactly the property that
   lets datacentres be added and replaced as operational acts.

## Costs accepted

1. **No revenue from idle capacity.** Spare substrate earns nothing under this bet. That is a
   real giveaway and it is the pressure BET-0009 exists to answer under strict conditions,
   rather than by quietly relaxing this one.

2. **Guardrails become correctness features and must exist before the price does.** Inherited
   from BET-0002 cost 4 and restated because it is the load-bearing one: under a metered model
   a tenant mining cryptocurrency pays for it, and under this one they do not. Trigger clause 2
   is this cost written as a falsification condition.

3. **This is a hardware business, commercially as well as operationally.** RFC-0019 accepted
   cost 1 records the operational half. The commercial half is that capacity must be bought
   ahead of demand, so a forecast error becomes idle capital rather than an unused reservation
   somebody else owns. Trigger clause 3 bounds it in engineer-days rather than rupees, because
   at this stage attention is the scarcer input.

4. **The order of magnitude may compress.** The measurement this bet rests on is a dated
   observation of a competitive market, and MKT-0001 records a vendor repricing three times in a
   single year. **This artifact carries no `sources` of its own by design** — the evidence lives
   in MKT-0001, which can be superseded when the market moves without touching the wager. The
   `review_date` is here because the wager may outlive the numbers that justified it.

## Alternatives rejected

**Meter consumption and bill for it.** The obvious model, rejected by C-0006 anti-goal 2 and
BET-0002 on four grounds that still hold, and now additionally by the arithmetic above: there
is no markup available to charge.

**Buy managed Kubernetes and stop owning hardware.** Genuinely tempting, and it is what trigger
clause 1 converts to if this bet dies. Rejected now because it forfeits the order-of-magnitude
gap that funds a low floor, and because RFC-0012's per-region Zitadel and RFC-0014's residency
boundary make provider selection a legal question in every region rather than a purchasing one.

**Own metal because it is cheaper.** Rejected as a *reason*, though the conclusion may stand.
The cost model says owned Indian colocation and rented European metal are within noise of each
other. Anyone justifying a rack purchase on price should be shown the table above first.

## Open questions

1. **What is the fully-loaded cost of one idle tenant?** BET-0007 trigger clause 1 needs this
   number and no artifact has it. It cannot be computed before Phase 2b runs, because the
   denominator is density and density is unmeasured.
2. **Does the guardrail live in `platform-metering` before that context's trigger fires?**
   RFC-0009 §7 fires `platform-metering` on *"the first resource whose overuse costs us money"*.
   Under this bet that is every resource, from the first tenant — so the trigger may already
   have fired, in the same way RFC-0009 open question 2 suspects `platform-audit`'s has.

## Related

- **MKT-0001** — the cost measurement this bet rests on, and the only home for its figures
- **MKT-0002** — how infrastructure is actually monetised, and the failure mode of selling capacity
- **RFC-0019** — open question 1, which this closes; decision 3, which BET-0009 depends on
- **BET-0009** — the part that is sold, on separate metal, by the workload
- **BET-0007** — the floor this bet exists to make defensible; superseded BET-0002
- **C-0006** — anti-goal 2, which this supplies an independent argument for
- **RFC-0014** — a region is a residency boundary; the only thing owning metal actually buys
- **RFC-0023** — the substrate that exists today, and its stated lack of any availability
- **ADR-0024** — nested tenancy, where density is already existential
