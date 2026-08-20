---
id: MKT-0001
title: The cost of a gigabyte of memory on four substrates, August 2026
type: market
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [cost, infrastructure, proxmox, colocation, hetzner, india, unit-economics]
breaking: false
enforced_by: []
sources:
  - url: https://getdeploying.com/aws-vs-hetzner
    title: getdeploying — Amazon Web Services vs Hetzner price comparison
    retrieved: 2026-08-17
    supports: >
      Hetzner CX43 (8 vCPU, 16 GB) at $19.07/mo against AWS c5.2xlarge (8 vCPU, 16 GiB) at
      $248.20/mo. AWS egress $0.09/GB beyond 100 GB; Hetzner €1/TB beyond a 1–60 TB
      allowance. Object storage 1 TB: AWS $24.63/mo, Hetzner $7.51/mo.
  - url: https://www.ezerhost.com/bare-metal-server/
    title: EzerHost — bare metal dedicated servers, India
    retrieved: 2026-08-17
    supports: 6-core / 32 GB bare metal in India from ₹2,999/month. Row B2.
  - url: https://www.rackbank.in/colocation/
    title: RackBank Datacenters — colocation pricing, India
    retrieved: 2026-08-17
    supports: >
      1U at ₹7,184/month with up to 450 W; 4U at ₹19,520/month with 1 kVA; half rack (21U) at
      ₹54,000/month with 3 kVA. The colocation input to row C.
  - url: https://www.comparevps.com/hosting/contabo
    title: Contabo VPS hosting plans and pricing
    retrieved: 2026-08-17
    supports: 4 vCPU / 8 GB from €4.50/month retail. The reference row that decides the finding.
  - url: https://www.proxmox.com/en/products/proxmox-virtual-environment/pricing
    title: Proxmox VE — subscription pricing
    retrieved: 2026-08-17
    supports: >
      Community €120/socket/year, Basic €370, Standard €550, Premium €1,100. All tiers unlock
      identical features; they differ only in support entitlement. The licence line, and why
      it changes no conclusion here.
  - url: https://www.nobroker.in/blog/commercial-electricity-rate-per-unit-in-maharashtra/
    title: Commercial electricity rate per unit in Maharashtra, 2026
    retrieved: 2026-08-17
    supports: >
      Commercial tariff from ₹5.88/unit, with commercial and industrial users typically paying
      ₹6–15/unit. The power input to row A.
  - url: https://northflank.com/blog/hetzner-cloud-server-price-increases
    title: Northflank — Hetzner cloud server price increases in 2026
    retrieved: 2026-08-17
    supports: >
      Three price increases during 2026. The reason this artifact is dated evidence rather than
      a constant, and the reason it will need superseding rather than editing.
---

# MKT-0001 — The cost of a gigabyte on four substrates

## What this is

**An observation, not a wager.** Every figure below could be checked against the world today by
opening a vendor's price page. What follows from it — that the estate is cost of goods, that
hosting must be sold by the workload — is BET-0008 and BET-0009, and belongs there.

Normalised to **rupees per gigabyte of RAM per month**, because memory is the binding constraint
on tenant density under ADR-0024, and because it is the one unit every substrate publishes.

## The measurement

| Path | ₹/GB/month | Basis |
| --- | --- | --- |
| **A.** `sitar-as-in-01-ar` | ~14 | Sunk hardware; ~108 kWh/mo at ₹6–15 commercial tariff |
| **B1.** Rented dedicated, Europe | ~56 | Hetzner AX41-class 64 GB, auction tier ~€38/mo |
| **B2.** Rented dedicated, India | ~94 | 6-core / 32 GB from ₹2,999/mo |
| **C.** Owned metal, Indian colocation | ~61 | 5 × 256 GB; ₹11.5 L capex ÷ 48 mo + ₹54,000/mo half rack |
| *Reference:* Contabo retail VPS | ~53 | 4 vCPU / 8 GB at €4.50/mo, sold to anyone |
| *Reference:* AWS on-demand | ~13× B1 | c5.2xlarge $248.20/mo vs Hetzner CX43 $19.07/mo, same 8 vCPU / 16 GB |

**Row C's capex is illustrative, not budgeted.** ₹11.5 lakh over 48 months is a modelled figure,
not a quotation, and it is the one input here that is not a published price. It is marked because
the distinction matters: rows A, B1, B2 and both references are observations, row C is an
observation combined with an assumption.

**Proxmox licensing is excluded** because it changes no conclusion. At Community €120/socket/year
across five two-socket nodes it is ~₹8,600/month against a ₹78,000 total — under two rupees per
gigabyte, and zero if the no-subscription repository is used.

## Two comparisons that carry the weight

**1. Retail sits below production cost.** Contabo's ₹53 is beneath row C's ₹61. A vendor will
sell a prospective customer a gigabyte for less than an Indian colocated rack costs to produce
one. Any business whose price names a gigabyte is competing against that number.

**2. Owning is not cheaper than renting; it is cheaper than renting *locally*.** Row C (₹61) and
row B1 (₹56) are within noise of each other, and B1 is the cheaper of the two. Owned metal only
beats row B2 (₹94). Since B1 sits in German jurisdiction, the entire measured advantage of owning
is available only to a business that must be in India — which is a residency argument, not a cost
argument.

## Crossover between renting and owning in India

Rented capacity scales linearly; a colocated rack is a step function.

| Committed RAM | Rented, India | Owned, half rack |
| --- | --- | --- |
| 128 GB | ₹12,000 | ₹78,000 |
| 416 GB | ₹39,000 | ₹78,000 |
| **830 GB** | **₹78,000** | **₹78,000** |
| 1,280 GB | ₹120,000 | ₹78,000 |

**Crossover at ₹94G = ₹78,000 → G ≈ 830 GB**, about twenty-six 32 GB nodes. The arithmetic moves
with row C's capex assumption; the method does not.

## What this artifact does not establish

- **Nothing about density.** Cost per gigabyte is not cost per tenant, and the conversion is
  unmeasured. BET-0008 open question 1 is the gap.
- **Nothing about quality.** Row-to-row comparison assumes fungible gigabytes. Disk class, network,
  noisy neighbours and support are not in the table.
- **Nothing about whether any of this should be sold.** That is BET-0009 and C-0006 anti-goal 2.

## Decay

Every row is a competitive market's published price on one day. Hetzner alone raised prices three
times during 2026. **Treat any figure here older than a quarter as indicative and re-measure before
a purchase decision rests on it.** When re-measured, supersede this artifact rather than editing it
— under ADR-0022's `immutable_when_locked` reasoning, evidence that turned out wrong is the most
valuable kind and the easiest to quietly rewrite.

## Related

- **BET-0008** — the estate is cost of goods; rests on the two comparisons above
- **BET-0009** — hosting by the workload; rests on the retail-below-cost finding and the crossover
- **MKT-0002** — how infrastructure is actually monetised, and how small providers fail
- **RFC-0019** — a datacentre is one Proxmox cluster; its accepted cost 2 records the tension
  this artifact measures
- **RFC-0014** — a region is a residency boundary; the only thing the measurement says owning buys
- **RFC-0020** — external facts are cited and dated
