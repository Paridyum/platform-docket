---
id: BET-0009
title: Hosting may be sold, by the workload and never by the gigabyte, on separate metal
type: bet
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [strategy, hosting, revenue, proxmox, isolation, residency, dpdp, c-0006, rfc-0019]
breaking: true
trigger: >
  Any ONE of the four kills the bet; they are ordered by how early each can fire.
  (1) SCHEDULE — Phase 3a slips by more than six weeks against the date recorded when the
  first hosting customer is signed, and hosting is a contributing cause. This fires earliest
  and matters most: the company has no running software, and a revenue line that delays the
  product is not funding it. Kill on this clause even if the margin is excellent.
  (2) ABUSE — support, abuse handling and incident response exceed two hours per paying
  customer per month, averaged over any rolling quarter after the tenth customer. That is
  the load the cited evidence says closes small providers, and it scales with customers
  while the price does not.
  (3) MARGIN — gross margin per node below sixty percent after six months of selling, where
  the numerator excludes any engineer time. Below that the workload framing has failed and
  what is being sold is gigabytes with extra steps.
  (4) COMPARISON — more than one in four evaluating customers asks to be quoted per vCPU or
  per GB and declines when refused, measured over the first forty evaluations. The whole
  bet is that the unit of sale can be moved off the resource; this is the direct test.
review_date: 2027-02-01
enforced_by: []
---

# BET-0009 — Hosting is sold by the workload, not the gigabyte

## Status

**ACCEPTED** on 2026-08-18, proposed 2026-08-17. A bet, not a decision: per ADR-0022 this artifact can never be
`locked`, and any one of the four `trigger` clauses kills it.

**`breaking: true`, and the breakage was constitutional.** This bet could not be accepted while
C-0006 anti-goal 2 stood unamended. **That amendment was made on 2026-08-18** — anti-goal 2 now
forbids selling infrastructure *by the resource* rather than selling it at all, and carries an
argument of its own rather than delegating to BET-0007.

**The door this bet opened can be closed again.** C-0006's amendment note says so directly: if any
of the four triggers below fires, the honest response is to close anti-goal 2 back up rather than
leave the permission standing without the conditions that justified it.

## The wager

**Paridyum may sell hosting as a revenue line, on three conditions that hold together or not
at all: it is priced per running workload and never per resource; it runs on physically
separate metal from the platform; and it dies on a schedule trigger rather than a margin one.**

BET-0008 answers what the estate is for. This answers whether any of it may be sold, and the
answer is a qualified yes with the qualifications doing all the work.

## Why the unit of sale is the whole bet

The arithmetic in BET-0008 is fatal to selling gigabytes and irrelevant to selling workloads.
**MKT-0001 measures the capacity side and MKT-0002 the workload side; neither is restated here.**

What they establish together: a rented Indian node sold as raw capacity earns less than retail
charges for the same memory, and the same node sold as managed applications — at the per-app
prices MKT-0002 records across five vendors — puts the metal at roughly **seven percent of
revenue**.

**Everything that makes selling capacity hopeless stops applying**, and it stops applying for a
reason rather than by accident: the low-cost vendor sells a bare machine the customer must then
*operate*. The operating is the product. A customer weighing a VPS price against a per-app price
is comparing a machine to an outcome, which is the comparison you want and the opposite of the one
anti-goal 2 warns about.

⚠️ **The seven percent is derived, not observed.** MKT-0002 is explicit that no margin figures for
that vendor group are public; the ratio combines their published prices with MKT-0001's cost
measurement. It is arithmetic on two observations, not a third observation.

**This is why trigger clause 4 exists.** If customers insist on being quoted per vCPU, the
framing has failed empirically and the bet should die rather than be defended.

## Decision 1 — separate metal, and this one is not negotiable

**Hosting customers' workloads run on physically separate machines from the platform's.** Not a
separate VM, not a separate Proxmox node in the same cluster, not a separate Kubernetes namespace.
Separate metal.

> **Refined 2026-08-18 by RFC-0025 decision 1.** As originally written this clause read as
> forbidding *anything* shared, including the provisioning system. That was broader than the
> argument supports and would have forced two complete estates built twice for one operator.
>
> **The boundary is the data plane, not the automation.** The isolation guarantee C-0006 rests on
> is about where a workload *executes*. A control plane that provisions both fleets and runs
> workloads on neither crosses nothing that guarantee names. What stays absolutely forbidden is
> unchanged: a hosting customer's container and a tenant's data on the same physical machine.

RFC-0019 decision 3 already says this and gives the reason. **Its reasoning gets stronger under
this bet, not weaker**, and the reason is worth stating because the cheap option is genuinely
tempting:

> *A virtual machine for another project is outside that model entirely — no `tenant_id`, no
> policy, no session context, nothing the migration gate can see. If it shares a hypervisor, a
> Ceph pool and a corosync ring with the platform, then the isolation guarantee rests on the
> hypervisor boundary rather than on anything this repository can check.*

RFC-0019 wrote that imagining **our own side projects** in those virtual machines. This bet
puts **paying strangers running arbitrary container images** there instead. That is a
categorically larger threat than the one decision 3 was defending against, arriving through the
same door.

C-0006 names provable isolation as one of four things that are not commodity and therefore ours
to build, and its 2026-08-15 extension makes it the *entire mitigation* for the operator-tenant
conflict:

> *The guarantee is therefore no longer only "we isolate tenants from each other" — it is **we
> isolate tenants from us.*** … *With no commercial boundary, this item is the entire
> mitigation.*

A container escape on shared metal reaches every tenant's data, and the explanation would be
that they were put there to save rent. **ANTI-0007 is the only anti-goal C-0006 says can end
the company in a single incident; this is the only technical decision in this bet with the same
property.**

### The shared-cluster variant, recorded rather than argued away

*Recorded because it was proposed on 2026-08-17 and because a rejected alternative that is not
written down gets re-proposed.*

**The variant:** run hosting customers on the same Proxmox cluster as the platform, isolated as
separate VMs or a separate Kubernetes environment.

**What it saves:** approximately ₹3,000–4,000 per month, being the cost of one rented Indian
node at stage 1.

**What it costs:** the isolation guarantee stops being checkable and becomes a hypervisor
assumption. RFC-0019 decision 3 would need superseding rather than satisfying. C-0006
non-commodity item 2 — the thing the franchise model rests on — would rest on a boundary no
check in this repository can see.

**The judgement:** ₹4,000 a month is not a saving worth that, and it is the cheapest insurance
premium in this document. Recorded here so that if it is adopted anyway, it is adopted as a
decision with its cost stated, which is the whole point of the artifact.

## Decision 2 — rent until roughly 830 GB, then buy

**The transition from rented to owned metal is a computed trigger, not a preference.**

**MKT-0001 carries the crossover table and the arithmetic behind it.** Rented capacity in India
scales linearly; a colocated half rack is a step function; they meet at roughly **830 GB of
committed memory**, about twenty-six 32 GB nodes.

**Below that, renting is strictly cheaper and carries no capital risk. Above it, owning is
strictly cheaper and residency becomes sellable.**

⚠️ **The threshold moves with an assumption, and MKT-0001 marks which one.** Colocation capex is
modelled rather than quoted — it is the single input in that artifact that is not a published
price. Re-derive the crossover against a real quotation before any hardware is bought; the method
is what this decision commits to, not the number.

**This rule exists to protect against the growth assumption, not to express doubt about it.**
If growth is fast, rented capacity absorbs it the same week and the rack is bought with revenue
in hand. If growth is slow, ₹11.5 lakh was not spent. A robust design on paper does not
de-risk capital, because the risk being carried is demand risk and no amount of architecture
addresses it.

## Decision 3 — it cannot start on `sitar-as-in-01-ar`

RFC-0023 allocates that machine down to roughly 12 GB of Talos worker memory, from which
Kubernetes overhead must still come. More decisively, RFC-0023 accepted cost 3:

> *No availability of any kind. One machine, one power supply, one network connection, in a
> residence. Everything on it is disposable and must be treated that way.*

**Selling hosting from it means selling an availability commitment that the governing artifact
says does not exist.** Stage 1 begins at the first rented node. The entry cost of this business
is therefore ₹3,000–4,000 per month, which is also the largest sum at risk before the first
trigger can fire.

## Why this could be right

1. **First revenue that needs no payment aggregator.** BET-0007's royalty depends on split
   settlement through a licensed aggregator, which the ROADMAP lists as unresearched and which
   its own trigger clause 3 says may prove unavailable. Hosting is billed directly, B2B, in
   advance. It is the only revenue line currently designed that has no regulatory dependency.

2. **The marginal cost is near zero because the work is already required.** Proxmox, Talos,
   Terraform provisioning, tenancy and isolation are all being built for the platform under
   RFC-0019, RFC-0023 and ADR-0024. Selling workloads reuses them rather than adding a stack.

3. **Residency is the one thing that cannot be undercut.** A European provider structurally
   cannot sell "in India, under DPDP." It is also the only justification the cost model
   supports for owning metal at all, which means this bet and BET-0008 want the same rack for
   the same reason.

4. **It is the cheapest possible test of the substrate.** Real customers on real metal produce
   the density and abuse-rate numbers that BET-0007 trigger clause 1 and BET-0008 open question
   1 both need and neither can currently compute.

## Costs accepted

1. **It is a second business before the first one exists.** C-0006 anti-goal 6 is about
   verticals and so is not literally engaged, but its reason is: *"Breadth is the most
   attractive way to avoid finding out whether anything works."* There are zero lines of
   runtime Rust. **Trigger clause 1 is this cost written as a falsification condition, and it
   is ordered first deliberately** — a hosting line with excellent margins that delays the
   platform has failed at the thing it was for.

2. **An abuse desk, and eventually a 24/7 one.** The cited evidence is that this is what closes
   small providers rather than pricing. Managed workloads attract materially less abuse than
   raw compute — there is no root shell to sell — but "less" is not "none", and clause 2 bounds
   it in hours rather than incidents.

3. **The residency claim is narrower than it will be tempting to state.** MKT-0003 records the
   position and leads with the negative finding: there is no blanket localisation requirement
   under DPDP. **Selling "required by law" would be false today.** Sell residency as a property,
   never as a compliance obligation — and note that MKT-0003 flags its own notification citation
   as secondary and requires replacement with the primary Gazette notification before any
   customer-facing or contractual claim rests on it.

4. **ANTI-0007 tightens rather than relaxes.** A hosting customer may put regulated data on the
   substrate without asking. Terms and an acceptable-use policy have to exist before the first
   customer, not after — this is the anti-goal C-0006 says can end the company in one incident,
   and hosting is a new way for it to be violated by someone who is not us.

5. **Two operational estates, not one.** Separate metal means separate patching, monitoring,
   backup and on-call. Decision 1 buys the isolation guarantee and this is its recurring price.

## Alternatives rejected

**Raw VPS or container hosting priced per resource.** The arithmetic kills it. MKT-0001 finds
retail capacity priced below Indian colocation cost per gigabyte, and MKT-0002 records this as the
one group with a documented, repeatable failure mode — DDoS and abuse load against margins too
thin to fund defence — and no small-scale success to point at.

**GPU and AI inference hosting.** The highest-margin adjacent business by a distance, and rejected
on capital grounds rather than strategic ones. MKT-0002 records the Indian existence proof and the
scale it rests on: megawatts and thousands of GPUs. It is an existence proof for the market and a
counter-example for the entry cost. Revisit only if the crossover in decision 2 has already been
passed.

**Managed Kubernetes or bare-metal-as-a-service for agencies.** Higher contract values, far
higher support load, and a sales motion rather than self-serve. Rejected for stage 1 as
contradicting the low-cost test; a candidate for stage 3 if trigger clause 2 stays comfortable.

**Backup and disaster recovery as a service.** The dark horse and the one worth revisiting.
Proxmox Backup Server already sits in RFC-0019's sizing table; storage attracts almost no abuse
and no noisy-neighbour risk; retention is structurally high. Rejected for stage 1 only because
it earns less per node than managed workloads and does not exercise the platform's own
isolation model.

## Open questions

1. **What is the workload catalogue?** "Managed apps" is a shape, not a list. Whether it is a
   general catalogue in the Elestio mould or a narrow set adjacent to the first vertical is
   undecided, and it determines whether this competes with the catalogue product or feeds it.
2. **Does a hosting customer get a `tenant` row, or is this outside ADR-0024 entirely?** If
   they are tenants, RFC-0022's no-privilege guarantee and access log extend to them and the
   commitment gets larger. If they are not, there are two customer models and ANTI-0006's
   warning about merged concepts applies in reverse.
3. **Which entity sells it?** RFC-0013 fixes the jurisdiction as India and the incorporation
   path is scheduled but not taken. Selling hosting before incorporation is a different
   question from selling software before it.

## Related

- **MKT-0001** — cost per gigabyte and the rent-versus-buy crossover
- **MKT-0002** — the workload-seller group this bet copies, and the capacity-seller group it refuses
- **MKT-0003** — India residency under DPDP, and the bound on what may be claimed
- **BET-0008** — the estate as cost of goods, and the arithmetic this bet depends on
- **RFC-0019** — decision 3, which decision 1 here satisfies rather than supersedes; open
  question 1, closed by BET-0008
- **RFC-0023** — the substrate that exists, and decision 3 here explaining why it cannot serve
- **C-0006** — anti-goal 2, which must be amended before this can be accepted; anti-goal 6,
  whose reason trigger clause 1 encodes; non-commodity item 2, which decision 1 protects
- **RFC-0022** — the operator-tenant conflict, which open question 2 may extend
- **ANTI-0007** — regulated data, and a new way for a stranger to violate it on our metal
- **BET-0007** — the floor and royalty, whose regulatory dependency this line does not share
- **ADR-0022** — why this is a bet with a trigger rather than a decision
- **RFC-0025** — how this is built: a shared control plane, separate fleets, and Paridyum as
  customer zero. Refines decision 1 and is where the real scope of stage 1 is recorded
