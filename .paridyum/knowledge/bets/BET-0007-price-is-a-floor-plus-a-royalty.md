---
id: BET-0007
title: Price is a flat floor per running business plus a commission on what that business earns
type: bet
status: accepted
created: 2026-08-15
locked_at: null
supersedes: [BET-0002]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [pricing, commission, franchise-royalty, payments, regulatory, entitlement]
breaking: false
trigger: >
  Any ONE of the three kills the bet.
  (1) FLOOR — the flat fee fails to cover fully-loaded per-tenant cost at the median tenant,
  measured once fifty tenants exist. A floor that does not cover an idle tenant means growth in
  signups is growth in losses, which is the failure BET-0002 cost 2 named and this bet inherits.
  (2) ROYALTY — fewer than one in three tenants active for six months generates commission
  exceeding a quarter of their own flat fee. Below that the commission is theatre: it carries a
  large regulatory surface and returns a rounding error, and the honest response is to drop it
  and reprice the floor.
  (3) REGULATORY — split settlement through a licensed payment aggregator proves unavailable,
  uneconomic, or is withdrawn, and direct authorisation is not viable at our stage. The
  commission is then unbuildable rather than unwise, and BET-0002's shape returns unchanged.
review_date: 2026-12-01
enforced_by: []
---

# BET-0007 — Price is a floor plus a royalty

## Status

**ACCEPTED** as of 2026-08-15, superseding BET-0002. Never `locked` (ADR-0022) — a price shape
is the most frequently falsified claim a company makes.

**BET-0002 explicitly evaluated and rejected the commission half of this bet.** That reversal is
argued below rather than glossed, because a superseding artifact that does not engage its
predecessor's reasoning is just a newer opinion.

## The wager

**A tenant pays a flat monthly price per running business, and Paridyum takes a commission on
the transactions that business processes. The floor covers the cost of existing; the commission
is a franchise royalty on what the franchisee earns. Infrastructure consumption is a guardrail
on abuse, never a line item and never a source of margin.**

BET-0002's prohibition is carried forward whole. What changes is the addition of a second line,
and the reason it is now coherent.

## Which level pays what

Under BET-0005 the revenue model follows the level in the tenancy tree, which makes the rule
structural rather than a policy anyone has to remember:

| Who | Pays | Why |
| --- | --- | --- |
| A franchised **tenant** | flat floor + commission on their GMV | They run a business Paridyum supplied. The commission is a royalty on that business. |
| An **account** inside Paridyum's own tenant | a direct price for the software | They are a customer, not a franchisee. **No commission is taken on their own trade.** |
| Paridyum's own tenant | — | You do not charge yourself a royalty. Its revenue is direct end-customer revenue, which BET-0005 trigger clause 3 measures separately. |

**That second row is the important one.** A commission on a small business's own invoices to its
own clients is a percentage levied on trade they conducted themselves — which is
payment-processing markup, and C-0006 lists payment processing on the commodity side: *"We will
not position on these, price on these, or describe them as our value."* It would also invite
direct comparison against a licensed aggregator's published rate, which is BET-0002's rejection
reason 2 arriving through a different door.

A commission on a *franchisee's platform GMV* is not that. It is a share of revenue that exists
because the product exists. The distinction is not rhetorical: it is the difference between
charging for rails and charging for a business.

## Why the reversal is justified

BET-0002 rejected *"percentage of the tenant's revenue or transaction volume"* on three counts,
and closed with **"the strongest candidate if the wedge moves to commerce."** Under BET-0006 the
wedge *is* commerce — a franchisee runs a transacting business. Taking the three counts in turn:

1. **"It requires being in the payment flow, which is a large regulatory surface."** Still true,
   and now accepted as cost 1 rather than treated as disqualifying. Not answered — *paid for*.

2. **"It is unenforceable for the many templates that process no payments at all."** Answered by
   the hybrid. Products that process nothing pay the floor and generate no royalty, which is the
   correct outcome rather than a gap.

3. **"It makes our revenue a function of their success in the segment least likely to have any,
   so it prices at approximately zero exactly where BET-0001 places us."** Answered by the floor.
   Commission is upside on a base that already covers cost. BET-0002's own framing — *"adding a
   tier is a repricing; adding a meter is a rewrite of the promise"* — is why the floor had to
   come first and why this is a supersession rather than a fresh start.

## Why this could be right

- **It prices both things we actually sell.** The floor prices the running business; the royalty
  prices the business *doing well*. C-0006 says we sell the runtime a business runs on and the
  software that runs on it — one price for each half.
- **It is the natural shape of the thing BET-0005 says we are.** Franchises take a royalty. In
  that frame the commission needs no defending and reads as normal rather than extractive.
- **It aligns the company with replication.** BET-0006's trigger measures whether franchisees
  replicate Paridyum's own results; the commission means we are paid exactly when they do. This
  is the answer to BET-0001 cost 1 — *"software is not this customer's bottleneck"* — that a flat
  fee alone could never give.
- **It keeps predictability where predictability matters.** The bill a tenant cannot forecast is
  the infrastructure bill, and that is still refused. A royalty moves with revenue they have
  already received, which is the one variable cost a business does not resent.
- **Density remains our margin.** Unchanged from BET-0002, and still the alignment with ADR-0024
  that makes a flat floor viable at all.

## Costs accepted

1. **The payment flow is a regulatory surface, and it is the largest single commitment in this
   bet.** Collecting funds from end users and settling to tenants while retaining a cut is
   payment aggregation. The two viable paths are **split settlement through a licensed
   aggregator** — where the aggregator holds the licence, performs sub-merchant KYC and settles,
   and card data never reaches Paridyum — or **direct authorisation**, which carries net-worth and
   escrow obligations that a pre-incorporation company cannot meet.

   ⚠️ **The specific thresholds, licence categories and obligations are external facts and are
   deliberately not stated here.** RFC-0020 requires them cited and dated; RFC-0013 already
   establishes that the jurisdiction is India. Recording a half-remembered figure in a binding
   artifact is exactly the failure that RFC exists to prevent. **This must be researched and
   written up before the first rupee moves.**

   Split settlement is the assumed path. It keeps ANTI-0007 and C-0006 anti-goal 3 clean, because
   payment-card data never lands in Paridyum's database. That containment is a design constraint,
   not an aspiration: **no catalogue product may store a card number.**

2. **KYC collides with self-serve, and this is the sharpest operational cost.** BET-0006 inherits
   *"no sales call, signup to a deployed application."* Sub-merchant onboarding requires identity
   and business verification that takes days. **A tenant will have running software in minutes and
   be unable to collect money for days.** C-0006 already forbids conflating *time-to-running-software*
   with *time-to-revenue*, which makes this survivable — but the gap must be designed into the
   onboarding flow explicitly rather than discovered by the first franchisee.

3. **The floor must still cover an idle tenant, and commission does not help.** An idle tenant
   generates no transactions and therefore no royalty, while consuming schema, DNS, certificate,
   backups and monitoring share continuously. BET-0002 cost 2 is inherited unchanged: **the floor
   cannot be zero, and an unbounded free tier remains forbidden.** A free tier, if one ever
   exists, must be time-boxed or capacity-boxed.

4. **Both numbers are guesses until there are tenants.** The floor and the commission rate will
   both be wrong at first, and the commission rate is the harder one because it must clear the
   aggregator's own take without making the total unattractive against running the business
   independently. Set from measured data as soon as data exists.

5. **Two revenue models cost more to operate than one.** Billing, revenue recognition, dunning,
   support and the pricing page all carry both. This is real overhead accepted for the alignment
   in reason 3.

6. **REQ-0012's trigger fires.** BET-0002 said so in its own Related section: *"a payment-flow
   model would fire its trigger."* Vendor and compliance governance is now current, not deferred —
   the aggregator is a critical external dependency that must be *approved, scoped and exitable*.

7. **Abuse is still handled by the guardrail, not the bill.** Unchanged from BET-0002 cost 4.
   Quota enforcement and egress limits remain correctness features on the platform side, and they
   must exist before the price does.

8. **Some genuinely fair usage will still be refused.** Unchanged from BET-0002 cost 5.

## Alternatives rejected

**Flat fee only — BET-0002 unchanged.** The safe answer, and the one this returns to if trigger
clause 2 or 3 fires. Rejected because it leaves the alignment gap open: we would be paid the same
whether a franchisee thrives or stalls, in a model whose entire premise is that we supply a
business rather than software.

**Commission only, no floor.** The easiest sell to someone with no revenue and maximum alignment.
Rejected on cost 3: it charges an idle tenant nothing while costing us continuously, which is the
structural unprofitability BET-0001 and BET-0002 both warned about, walked into deliberately.

**Per-seat.** Rejected for BET-0002's reason, unchanged: this customer has no seats. Revisit only
if the wedge moves to agencies at scale, where seats are real — and note BET-0006 folds agencies
in without changing the price shape.

**Take the commission via our own payment licence.** Maximum margin and full control. Rejected as
premature: it converts a pricing decision into a licensing project, and cost 1's obligations are
unmeetable before incorporation and revenue. **Reconsider once trigger clause 2 is comfortably
unmet**, which is the point at which the royalty is large enough to justify the licence.

## Related

- BET-0002 — superseded; its prohibition on infrastructure markup is carried forward whole
- BET-0005 — the franchise structure that makes a royalty coherent rather than extractive
- BET-0006 — the segment that pays this
- C-0006 — anti-goal 2 (unchanged), anti-goal 3 and the commodity list (both engaged by cost 1)
- ANTI-0007 — why no catalogue product may store a card number
- REQ-0012 — vendor and compliance governance, whose trigger this fires
- RFC-0013 — the jurisdiction; RFC-0020 — why cost 1's figures are not stated here
- ADR-0024 — density is what makes the floor viable
- ADR-0022 — why this is a bet rather than an ADR
