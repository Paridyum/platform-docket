---
id: BET-0001
title: The wedge is the solo founder launching a software product
type: bet
status: superseded
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: BET-0006
scope: [org]
applies_to: ["*"]
tags: [strategy, icp, wedge, go-to-market, retention]
breaking: false
trigger: >
  Of the tenants that reach a running application on a custom domain, fewer than one in
  twenty ever collects a payment from an end user of their own — measured on the first
  two hundred such tenants. That is the direct test of whether software was their
  bottleneck, and if it was not, this wedge is the wrong one no matter how many sign up.
review_date: 2026-12-01
enforced_by: []
---

# BET-0001 — The wedge is the solo founder launching a software product

## Status

**ACCEPTED** as of 2026-08-07. A bet, not a decision: per ADR-0022 this artifact can never
be `locked`, and the `trigger` above is the condition under which it should be killed.

Chosen over two alternatives that scored better on revenue quality and worse on learning
rate. The reasoning is below, including the parts that argue against it.

## The wager

**The first customer is one person building one software product, with no team, no agency,
and no existing client base — and they will pay for a running production application before
they have any users of their own.**

Self-serve. No sales call. Signup to a deployed application on their own domain, without
writing the application.

This is the segment C-0001's Mission names in its own words — *"enable anyone to launch
production-grade software businesses in minutes"* — so it needs no constitutional amendment
to adopt. That is a convenience, not an argument, and it is listed here so it is not mistaken
for one.

## This is a beachhead, not a destination

**The intended end state is many markets. This bet is about which one is first, not which one
is only.** C-0001's Vision — *the operating system for businesses* — is multi-market by
definition; an operating system serving one segment is an application. So the constraint here
is **sequencing**, never ambition, and C-0006 anti-goal 6 should be read that way: it forbids
a second market before the first has a renewed customer, and forbids nothing after.

Recorded explicitly because the omission was already causing a misreading. Without this
section BET-0001 reads as a permanent choice, and an agent following CLAUDE.md would retrieve
"solo founders" as the answer to *who is this for* and design accordingly. That produces the
one mistake that is expensive to undo:

**Breadth as the end state changes an engineering constraint today, and it constrains
BET-0003.** A runtime contract fitted to the needs of one segment makes market number two a
rewrite rather than a configuration. The contract must therefore be designed for a *class* of
businesses from the first line, even while only one segment is sold to — which is C-0001
principle 1, *platform before product*, applied to the market axis rather than the code axis.
Concretely, the contract may not assume:

- one operator per business — an agency or a team is the same shape with more members;
- one business per paying account — BET-0002 already prices per business, plural;
- that the business has no end users of its own, or that it has them;
- that money flows the way it does in a subscription product.

Those four assumptions are exactly the ones a solo-founder-only design would bake in, and
each is cheap to avoid now and structural to remove later.

**What this section does not license.** It is not permission to build for many markets now.
Anti-goal 6 stands, the costs below are still accepted, and breadth attempted before the first
market works is the most attractive way to avoid finding out whether anything works. Design
the contract for breadth; sell to one segment.

## Why this could be right

1. **It is the only segment that produces learning at volume.** BET-0003 wagers that the
   application layer is generated against a runtime contract. The binding risk there is
   coverage: nobody knows yet what that contract must express. Five agencies produce five
   deep, correlated opinions. Five hundred solo founders produce a distribution of
   genuinely diverse application specifications, which is the only input that tells us
   where the contract is thin. For this product, at this stage, breadth of signal is worth
   more than quality of revenue.

2. **It is the only segment that tolerates an immature runtime.** Their alternative is
   nothing at all. An agency has client commitments and will not accept a platform with
   unknown failure modes; a solo founder with no users has almost nothing at risk and will
   forgive a great deal. That tolerance is a real, temporary asset and it expires.

3. **The generated application layer is worth most to someone who cannot write it.** The
   person who benefits most from BET-0003 working is the one for whom the alternative is not
   "hire a developer" but "give up". Highest perceived value, lowest switching cost from the
   status quo, because the status quo is an empty directory.

4. **Acquisition can cost approximately nothing.** Content, community, and a product whose
   output is publicly visible on the customer's own domain. A company with no sales team and
   no revenue cannot buy its way into a segment that requires one, and choosing a segment
   whose motion we cannot execute is a slower way to fail.

## Costs accepted

C-0001 principle 8: *an artifact that lists only benefits is incomplete and probably wrong.*
These are not hypothetical risks. Three of them are near-certain.

1. **Software is not this customer's bottleneck, and we are selling software.** Businesses
   fail for want of customers, not for want of a CRM. We are choosing the segment where the
   gap between what we sell and what the customer actually needs is at its widest. Most of
   these founders will fail whatever we ship, and their failure will look like our churn.

2. **Retention and satisfaction will decouple, and this is a metric trap.** A founder can
   love the product, recommend it, and cancel — because their business ended, not because
   ours did. High NPS beside terrible logo retention will read as a product-market fit signal
   and will be nothing of the kind. Any retention number reported for this segment must
   separate *churned because the business closed* from *churned because Paridyum was wrong*,
   or it is not a number worth reading. The `trigger` above is written to sit on the right
   side of that line.

3. **Low willingness to pay meets a per-tenant cost that never sleeps.** An idle tenant is
   not free: a schema, a DNS record, a certificate, backups, and its share of monitoring all
   cost money continuously, whether anyone logs in or not. Thousands of idle low-ACV tenants
   is the specific way this wedge becomes structurally unprofitable, and it is the reason two
   downstream artifacts are constrained rather than open:
   - **ADR-0024** must be maximum-density from day one, at both levels. Per-tenant infrastructure is
     unaffordable at this ACV, which is why the isolation escalation path is deferred behind
     a revenue trigger rather than offered as a tier.
   - **BET-0002** cannot price on consumption at the low end. The floor price must cover the
     cost of an idle tenant, or growth in signups is growth in losses.

4. **Self-serve removes the human who would have caught a bad fit.** No call means no chance
   to say *"you do not need this yet."* We will onboard, host and bill customers we could
   have told at the outset would not succeed. This is a cost of the motion, not a defect in
   it.

5. **Adverse selection on enthusiasm.** The people most excited by *"launch a business in
   minutes"* skew toward those with no demand to validate, because someone with demand
   already has a reason to be specific about what they need. The message that converts best
   may select the customers who retain worst.

## Alternatives rejected

**Agencies and operators running many client businesses.** Better on every revenue metric:
demand already exists, budget already exists, and one customer is N tenants — their org
chart is our data model. Rejected for now on learning rate (cost 1 above, inverted) and
because they will not accept an immature runtime. **This is the most likely successful
migration path once the runtime is proven, and choosing it later costs less than choosing it
now.** If this bet dies at its trigger, this is where to look first.

**Internal tools for existing small businesses.** Demand and budget both proven, and depth in
one vertical would beat breadth. Rejected because it reframes the Mission from *launch a
business* to *run the one you have*, which would require amending C-0001 — a larger act than
picking a first customer, and the wrong order to do it in.

## What would confirm it

The `trigger` states what kills this. The opposite, for symmetry — none of these is a vanity
metric, and all three must hold together:

- Tenants whose end users are paying them, at a rate that makes the trigger clearly unmet.
- Retention that survives the separation in cost 2: churn dominated by businesses closing,
  not by Paridyum being replaced.
- Application specifications diverse enough to have found real gaps in the runtime contract.
  If five hundred founders all wanted the same four things, BET-0003's premise is weaker than
  it looks and the honest response is a narrow product, not a platform.

## Related

- C-0006 — the market posture this sits inside; anti-goals 5 and 6 both follow from this bet
- C-0001 — the Mission this segment is named by, and the "in minutes" claim C-0006 qualifies
- BET-0002 — the pricing floor cost 3 forces
- BET-0003 — the generated application layer this segment is chosen to inform
- ADR-0024 — the density constraint cost 3 forces
- ADR-0022 — why this is a bet rather than an ADR
