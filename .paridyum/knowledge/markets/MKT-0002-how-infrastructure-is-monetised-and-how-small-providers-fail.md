---
id: MKT-0002
title: How infrastructure is monetised, and how small providers fail
type: market
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [hosting, business-model, repatriation, paas, ddos, abuse, e2e-networks, unit-of-sale]
breaking: false
enforced_by: []
sources:
  - url: https://www.datacenterdynamics.com/en/news/37signals-claims-it-saved-almost-2m-last-year-from-cloud-repatriation/
    title: DataCenterDynamics — 37signals claims it saved almost $2m from cloud repatriation
    retrieved: 2026-08-17
    supports: >
      ~$2M annual saving after repatriation from AWS, against roughly $700,000 spent on Dell
      R7625 servers. The repatriation group's headline figure.
  - url: https://www.theregister.com/2025/05/09/37signals_cloud_repatriation_storage_savings
    title: The Register — 37signals is completing its on-prem move
    retrieved: 2026-08-17
    supports: >
      ~90 servers per datacentre across eight 48U cabinets in two Deft facilities; 18 PB of
      Pure Storage flash at ~$1.5M upfront with operating cost under $200,000/year.
  - url: https://hackernoon.com/how-to-save-70percent-on-cloud-costs-re-architecting-aws-to-proxmox-lxc
    title: HackerNoon — re-architecting AWS to Proxmox LXC
    retrieved: 2026-08-17
    supports: >
      A SaaS moved off EKS, RDS and Elasticache to colocated Proxmox LXC with HAProxy and
      Keepalived, cutting ~$12,000/mo to ~$3,500/mo including colocation and bandwidth. A
      secondary source and the smallest documented Proxmox repatriation found.
  - url: https://www.whalesbook.com/news/English/technology/E2E-Networks-Q1-Revenue-Jumps-334percent-Amid-GPU-Expansion/6a619cfdfe1c9287f47c9db2
    title: E2E Networks Q1 FY27 — revenue up 334% amid GPU expansion
    retrieved: 2026-08-17
    supports: >
      Q1 FY27 revenue ₹1,568 million (+334% YoY), 75.2% EBITDA margin, on roughly 5,100 GPUs
      and 10 MW of capacity in Delhi NCR. Indian infrastructure sold profitably, and the
      capital scale at which it became profitable.
  - url: https://dev.to/vikasprogrammer/i-compared-6-platforms-for-deploying-self-hosted-apps-in-2026-3j8
    title: A comparison of six platforms for deploying self-hosted apps, 2026
    retrieved: 2026-08-17
    supports: >
      Elestio ~$17/month per managed application across a 400+ catalogue; PikaPods ~$2–4/month
      per app across ~200; Coolify free self-hosted under Apache 2.0 with cloud at $5/month;
      Dokploy managed at $4.50 per server; Cloudron ~$22/month. The per-workload price points.
  - url: https://www.nexusguard.com/blog/the-costly-dilemma-small-isps-and-ddos-protection
    title: Nexusguard — the costly dilemma, small ISPs and DDoS protection
    retrieved: 2026-08-17
    supports: >
      Small providers run margins too thin to fund scrubbing hardware, bandwidth
      over-provisioning or 24/7 incident staffing, and the combination forces closures.
  - url: https://blog.cloudlinux.com/scaling-hosting-in-2026-where-growth-meets-its-limits-and-how-hosting-providers-respond
    title: CloudLinux — scaling hosting in 2026
    retrieved: 2026-08-17
    supports: >
      Growth is no longer fuelled by low-cost plans or oversold resources; price competition
      named the top threat to profitability by 29% of hosts and rising costs by 28%.
---

# MKT-0002 — How infrastructure is monetised

## What this is

**An observation of what operators actually do**, gathered 2026-08-17. It records who makes money
from owned metal, at what scale, and what the recorded failure mode is for those who do not. What
should be concluded from it is BET-0009.

## Three groups that make money, and one that does not

### 1. Repatriators — they prove the saving, not a business

| | |
| --- | --- |
| 37signals | ~$2M/year saved against ~$700,000 of Dell R7625s; ~90 servers per datacentre across eight 48U cabinets; 18 PB flash at ~$1.5M upfront, under $200,000/year to run |
| An unnamed SaaS | ~$12,000/mo on EKS + RDS + Elasticache → ~$3,500/mo on colocated Proxmox LXC, colocation and bandwidth included |

**The pattern, and it is the limitation.** Both repatriated an *existing* workload with known
demand. Neither bought capacity speculatively. This group establishes that owned metal is
dramatically cheaper than managed cloud once you have load — and says nothing about acquiring
load.

### 2. Indian infrastructure sellers — they prove the market and the entry price

**E2E Networks**, Q1 FY27: revenue ₹1,568 million, up 334% year on year, **75.2% EBITDA margin**.
That rests on roughly 5,100 GPUs and 10 MW in Delhi NCR.

Both halves are the finding. Indian infrastructure sells at excellent margin, *and* the margin is
a function of capital at a scale measured in megawatts. It is an existence proof for the demand
and a counter-example for the entry cost.

### 3. Workload sellers — the group operating at a scale that is reachable

| Offering | Price | Unit |
| --- | --- | --- |
| Elestio | ~$17/month | per managed application (400+ catalogue) |
| PikaPods | ~$2–4/month | per app (~200 catalogue) |
| Cloudron | ~$22/month | per install |
| Coolify | free self-hosted · $5/month cloud | per control plane |
| Dokploy | free self-hosted · $4.50 | per server |

**The structure is the finding.** The control planes that turn a plain server into an app platform
are free and open source. The managed tier charges anyway, and charges *per application* rather
than per resource. What is being sold is the operating, not the machine — and it is priced on a
unit the customer's own bill from a VPS vendor cannot be compared against.

### 4. The group with no documented success: small sellers of raw compute

Searching for profitable small VPS operations returns abundant material on *starting* one and
almost none on succeeding. What is documented is the failure mechanism, consistently:

- Margins too thin to fund DDoS scrubbing hardware, bandwidth over-provisioning, or 24/7 incident
  staffing — and the combination forces closures.
- Industry commentary for 2026 from the other direction: growth "no longer fuelled by low-cost
  plans, oversold resources, or race-to-the-bottom pricing," with **price competition named the
  top profitability threat by 29% of hosts** and rising costs by 28%.

**This is the strongest negative result in the set.** The business shape with the most obvious
appeal — cheap compute, resold — is the one with a repeatable, documented failure mode and no
small-scale success to point at.

## The observation that ties the groups together

Group 3 charges roughly $17 for an application that group 4 would host on ~₹100 of raw capacity.
The difference is not a markup on the same good; **they are selling different goods.** Group 4
sells a machine the customer must then operate. Group 3 sells the operating.

That distinction is checkable today against five published price lists, which is why it is
recorded here rather than in a bet.

## What this artifact does not establish

- **No margin figures for group 3.** Elestio's and PikaPods' costs are not public; the ~7%
  metal-to-revenue ratio computed in BET-0009 is derived from MKT-0001, not observed from them.
- **No churn, support-load or abuse-rate data at any group.** The DDoS sources describe the
  mechanism, not a rate. BET-0009's two-hours-per-customer trigger is a chosen threshold, not a
  measured benchmark.
- **Selection bias is likely and unquantified.** Failed operators write fewer blog posts than
  successful ones, which cuts against group 4 appearing at all and should if anything strengthen
  the negative result — but it is asserted, not measured.

## Decay

Group 3's price points move with a competitive market. E2E's margin is one quarter in a GPU boom
and should not be read as a steady state. Re-measure before either is load-bearing again, and
supersede rather than edit.

## Related

- **MKT-0001** — the cost of a gigabyte, which sets what group 4 is competing against
- **MKT-0003** — India residency, the one property group 3's European competitors cannot sell
- **BET-0009** — hosting by the workload; group 3 is the shape it copies and group 4 the shape it refuses
- **C-0006** — anti-goal 2, and the commodity list that names web hosting
