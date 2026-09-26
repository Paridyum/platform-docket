---
id: C-0006
title: Market posture, category, and the things Paridyum will not build
type: constitution
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [charter, positioning, category, anti-goals, strategy]
breaking: false
enforced_by:
  - check: domain.no_self_serve_path_to_a_dedicated_rung
    severity: error
    rationale: >
      Anti-goal 5 — no enterprise before self-serve works — together with ADR-0024 decision 8's gate
      on a signed contract. A signup flow that provisions a dedicated rung moves a commercial gate
      into a form, where nobody reviews it and nothing records who approved it.
---

# Constitution C-0006 — Market posture

## Status

**`accepted`, deliberately not `locked`,** for the reason C-0001 gives: nothing here can be
mechanically verified. A market position is upheld by judgement. `enforced_by` is empty
because naming a proxy check would be the false confidence ADR-0002 exists to prevent.

**C-0001 outranks this document.** Its own rule — *"if two principles conflict, the
lower-numbered one wins"* — settles any disagreement in its favour. Nothing here contradicts
it; this document **specifies** it. C-0001 states a Vision and a Mission in two sentences,
and two sentences cannot say who the customer is or what the product refuses to be. That gap
is what this fills.

Read this together with the bets in `bets/`. This document holds the parts stable enough to
be a principle. The parts that are wagers — which customer, which price shape, which app
layer — are BET-0006, BET-0007 and BET-0004, and they are separate artifacts precisely
because they are expected to change.

## Amendment — 2026-08-15

**Amended because BET-0003 was withdrawn and this document had delegated two of its arguments
to it without restating them.** Anti-goal 1 and the "actively wrong framings" table both cited
BET-0003 as their entire justification; when BET-0004 superseded it, both were left standing on
a withdrawn artifact and BET-0004 recorded that *"nothing in it is obviously contradicted."*
That was wrong — anti-goal 1 forbade, in its own words, "a catalogue of hand-authored vertical
apps," which is precisely what BET-0004 wagers on.

The failure is worth naming because it is general: **an artifact that outsources its argument to
a lower-precedence one inherits that artifact's mortality.** A constitution citing a bet must
still be able to state its own reason.

Four changes below, all dated here: anti-goal 1 rewritten to bite where it actually bites; the
framings table repaired; non-commodity item 2 extended to isolation *from the operator*, which
BET-0005 makes load-bearing; and non-commodity item 4 flagged as in conflict with RFC-0022.

## Amendment — 2026-08-18

**Anti-goal 2 narrowed from the asset to the unit of sale, and given an argument of its own.**

Two defects, and the first is the one this document already diagnosed in itself on 2026-08-15,
appearing a second time. **Anti-goal 2 delegated its entire justification to BET-0007** and would
have been left standing on a dead artifact had any of that bet's three triggers fired — precisely
the failure the earlier amendment note calls general: *"an artifact that outsources its argument to
a lower-precedence one inherits that artifact's mortality."* It now carries its own reason, which
is arithmetic rather than principle and therefore not mortal in the way a wager is.

The second defect is that **the prohibition was drawn around the asset when it belonged around the
unit of sale.** Every objection it raises is a consequence of an itemised resource; none survives a
price the customer reads as an outcome. RFC-0019 open question 1 had been open since 2026-08-12
because the old wording made a hypervisor carrying anything for anyone look like a violation.
BET-0008 closes it.

**This is a real relaxation and should be read as one.** A door that was closed is now ajar, and
the conditions holding it ajar live in BET-0009, which can be killed by any of four triggers. **If
BET-0009 dies, close this door again** rather than leaving the permission standing without the
conditions that justified it.

**What did not change:** anti-goals 1, 3, 4, 5 and 6 are untouched. Anti-goal 3 and ANTI-0007
*tighten* in practice, because a hosting customer is a new party who can put regulated data on the
substrate without being asked. RFC-0019 decision 3 is satisfied by BET-0009 decision 1, not relaxed
by it.

## Category

**Paridyum is a business operating platform.**

That is the external line. It is not a coined acronym, and the absence is deliberate.

A coined category — *Business Infrastructure as a Service*, or any four-word phrase reduced
to initials — is not free positioning. Establishing one costs years of marketing spend, and
leading with an initialism nobody has heard signals a company that has not found its wedge
and is describing its architecture instead. Categories are earned in retrospect. Salesforce
did not launch as *CRM as a Service*; it launched as **No Software**, and the category was
named later by other people.

The internal North Star stays as C-0001 wrote it: *the operating system for businesses.* That
is the correct ambition and the wrong sentence to sell with, because it is unfalsifiable and
abstract to the person deciding whether to sign up. **Do not use it as external copy.** An
operating system's credibility comes from what runs on it, and nothing runs on this yet.

Adjacent framings that are actively wrong, and why:

| Framing | Why it is rejected |
| ------- | ------------------ |
| no-code platform | Sets the ceiling at what a non-programmer can assemble, and puts us in a category whose buyers churn hardest. |
| website builder | Names the least valuable layer we own. |
| white-label SaaS | Describes a billing arrangement, not a capability. **Note 2026-08-15:** the *model* is white-label — tenants run under their own brand and their end customers need never learn Paridyum exists (BET-0005). This row rejects it as external **positioning**, not as a description of what happens. |
| app store / template marketplace | Describes third-party supply we do not have and do not want. Paridyum authors and **operates** its catalogue (BET-0004, BET-0005); a marketplace is a venue for other people's software. |

## What we actually sell

**The runtime a business runs on, and the software that runs on it.**

The claim is about the *layer*, not the breadth. C-0001's *"in minutes"* is a claim about
time-to-running-software. It must never be presented, internally or externally, as a claim
about time-to-revenue. Those are different quantities and conflating them is the single
easiest way to acquire customers who cannot succeed — see BET-0006, which commits to the
segment where that gap is widest and states the cost. **Sharper under BET-0007**: a franchisee
whose payment onboarding is still in KYC has running software and cannot yet be paid, so the two
quantities are separated by days in the product itself, not only in the marketing.

## What is not commodity, and what is

This distinction decides where engineering effort goes, so it is a principle rather than a
bet.

**Commodity.** Authentication, payment processing, object storage, transactional email, CDN,
web hosting, error tracking, product analytics. Each is a mature vendor category with a free
tier. A competent developer assembles all of them in a weekend. **We will not position on
these, price on these, or describe them as our value.** We depend on them; depending on
something well-supplied is not a differentiator, and reselling it at a markup is BET-0007's
subject. **Payment processing is on this list, and BET-0007 cost 1 is where the line sits:** a
royalty on a franchisee's own business is a share of something we supplied; a percentage of a
small business's own trade is a markup on rails, and is refused.

**Not commodity, and therefore ours to build:**

1. **Business semantics as one inherited model.** Organisation, membership, role, permission,
   entitlement, plan, quota, usage meter, audit trail — coherent, consistent across every
   application, and inherited rather than re-implemented. Every vendor above solves one slice
   for one app. Nobody sells the whole model, and it is the part every application needs and
   every founder rebuilds.
2. **Provable isolation at two levels.** Not "we use separate schemas" but a guarantee with a
   check behind it — and the guarantee is *nested*: Paridyum isolates businesses from each other,
   and each business inherits the isolation of its own customers from the runtime rather than
   implementing it (GLO-0004). The second level is the larger part of this item. It is the
   hardest thing every founder rebuilds, the one whose failure is unrecoverable, and the clearest
   difference between selling infrastructure and selling a runtime. ADR-0024 is the mechanism;
   the *provable* is the product.

   **Extended 2026-08-15.** Under BET-0005 Paridyum operates tenants on this runtime alongside
   the tenants it sells to, in the same verticals, with no commercial boundary. The guarantee is
   therefore no longer only *we isolate tenants from each other* — it is **we isolate tenants
   from us.** That is a third level and the hardest one to make credible, because the party
   making the promise is the party it constrains. GLO-0005's rule that operator scope is a grant
   and never an absence stops being hygiene and becomes the commitment the franchise model rests
   on. RFC-0022 designs it. **With no commercial boundary, this item is the entire mitigation.**
3. **A contract strong enough to generate against.** Originally BET-0003's wager, which was
   withdrawn. It survives in changed form: the contract a catalogue **product** declares itself
   in — its identity project, role vocabulary, audience and tenancy scope — which RFC-0009
   Revision 2 moved to a `spec-` repository. The reason this repository's governance machinery is
   a commercial asset and not overhead is unchanged.
4. **Aggregate operational data — bounded, and the bound is the point.** What a business looks
   like when it is working, across thousands of them. Benchmarks, defaults, and eventually
   models. Available to nobody who does not host the runtime.

   **Narrowed 2026-08-15.** This item was written when Paridyum's tenants were not its
   competitors. Under BET-0005 they are, and every tenant runs under **its own brand** rather
   than ours, so we see their customer counts, revenue, churn and growth while they see nothing
   of ours. Unbounded, this asset *is* the conflict.

   The bound, decided rather than deferred: **aggregate operational data may improve the runtime
   and set product defaults. It may never inform where Paridyum competes** — not which vertical
   to enter, not which segment to target, not when. That is a real asset given up, and it is
   given up because the alternative is a platform whose customers are rationally afraid of it.

   Enforcement is partly mechanical and partly not, and the difference must not be blurred.
   Data *access* is checkable (RFC-0022 D1, D5). A *decision* about which market to enter is not
   a code path and no check sees it — so every vertical Paridyum enters must be recorded in an
   artifact naming the evidence that selected it, and that evidence may not be tenant-derived.
   Prose with frontmatter that a gate can read is this repository's entire premise; this is that
   premise applied to the one commitment that cannot be tested directly.

## Anti-goals

The most useful part of this document. Each has an artifact carrying the argument, and each
is a thing we will be repeatedly and plausibly urged to do.

1. **We will not franchise a vertical we have not operated.** *Rewritten 2026-08-15; the
   original forbade a catalogue outright and is preserved in the amendment note above.*

   The original argument had three clauses and only one of them was ever the point. *Costs
   linearly in verticals* is true of the domain layer and BET-0004 accepts it with a trigger.
   *Loses to the incumbent's free tier* turned out to be aimed one level away from where it
   bites — it is a category error against a founder reselling multi-tenant software, and
   literally true of Paridyum's own operating tenant, which is where it now applies.

   **The binding clause was always "shallow in each."** A catalogue of shallow verticals loses,
   and no amount of shared runtime makes anyone a domain expert in five industries. BET-0005
   supplies the only durable answer: **you cannot operate a business you are shallow in, because
   customers leave.** So the prohibition becomes a sequencing rule with a test behind it —
   operate the vertical, acquire real end customers, *then* franchise it. A vertical Paridyum
   will not run itself is one it may not sell to someone else.

   What remains forbidden outright: **breadth without coherence.** A pile of independent
   applications sharing a login is the shape this anti-goal exists to prevent, and non-commodity
   item 1 is the test — one inherited business model across every product, or it is not a
   catalogue, it is a portfolio.

2. **We will not sell infrastructure by the resource.** *Narrowed 2026-08-18; the original
   forbade selling infrastructure at all as a revenue line and is preserved in the amendment
   note below.*

   What is forbidden is the **unit of sale**, not the asset. A price that names a vCPU, a
   gigabyte, a transfer volume or an hour is refused, because every objection this anti-goal
   has ever carried is a consequence of that naming: it makes the customer's bill scale with
   our inefficiency, it invites line-item comparison against a cloud provider's published
   rates, it charges most at the moment a customer is succeeding, and it introduces variance
   into a segment that churns on variance alone.

   **The argument this anti-goal previously delegated to BET-0007, restated so that it
   survives that bet:** there is no markup available to charge. A retail vendor will sell a
   prospective customer a gigabyte of memory for less than our own colocation costs us to
   produce one — MKT-0001 carries the dated measurement and is the only place those figures
   live. Reselling resources is not a strategy we disapprove of; **it is a market we would
   lose in.** Anyone proposing it should be shown MKT-0001 before the argument reaches
   positioning.

   **What is permitted, and the conditions are the point.** Compute may be sold when the unit
   of sale is a running workload rather than a resource, on a data plane physically separate
   from the platform's, under BET-0009's four triggers. The permission is narrow by
   construction: the moment a customer is quoted per gigabyte, this anti-goal is engaged again
   and the answer is no.

   **Unchanged:** infrastructure consumption inside a tenant's own price remains a guardrail
   on abuse, never a line item and never a source of margin. BET-0007's royalty on a tenant's
   transactions remains a share of a business we supplied, never a markup on rails. BET-0008
   holds the general case — the estate is cost of goods.

3. **We will not touch regulated data before the controls exist.** No health, payment-card,
   or regulated financial records — not as a premium tier, not for one willing customer, not
   behind a disclaimer. See ANTI-0007. This is the only anti-goal whose violation can end the
   company in a single incident.

4. **We will not treat switching cost as the retention strategy.** *"To leave, they would
   have to rebuild their company"* describes a hostage, not a moat. It produces the worst
   possible word of mouth, and it is the first objection an evaluating customer raises. A
   credible, documented, tested export path is a **sales asset**. See REQ-0013 — and note
   that REQ-0012 already requires every external dependency be *"approved, scoped, and
   exitable"*; refusing our own customers what we demand of our vendors is incoherent.

5. **We will not sell to enterprises before self-serve works.** One enterprise contract at
   this stage buys revenue and a roadmap owned by someone else. The isolation tiers a large
   customer requires are ADR-0024's deferred escalation path, and their trigger is a signed
   contract, not a conversation.

6. **We will not add a second vertical before the first one has a paying customer who
   renewed.** ADR-0019 already forbids implementation before decomposition; this is the
   commercial form of the same rule. Breadth is the most attractive way to avoid finding out
   whether anything works.

   **This constrains sequencing, not ambition.** Many markets is the intended end state —
   C-0001's Vision is multi-market by definition, since an operating system serving one
   segment is an application. The anti-goal forbids a second market before the first has
   renewed, and forbids nothing after. It also does not license a *narrow* runtime: BET-0006's
   beachhead section records the four assumptions the contract may not bake in, because a
   contract fitted to one segment makes market number two a rewrite. Design for breadth, sell
   to one segment.

   **Note the levels, added 2026-08-15.** Under BET-0005 a small business using a catalogue
   product is an `account` inside Paridyum's own tenant, while a founder running that product as
   their business is a `tenant`. Serving both is **not** a second market and does not engage this
   anti-goal — it is one market at two depths of the same tree. What would engage it is a second
   *vertical*, franchised before the first has a renewed franchisee.

## How to use this document

When a decision looks like a market question rather than a technical one:

1. Check whether it is an anti-goal here. If so, the answer is no and the artifact names why.
2. Check whether a bet in `bets/` already wagers on it. If so, follow the bet, and check
   whether its `trigger` has fired — a bet whose falsification condition has been met is not
   guidance, it is a decision waiting to be written.
3. If it is genuinely open, write a BET-, not an opinion in a chat log. Per ADR-0022 it must
   state what would kill it and when it gets judged.

If an anti-goal here is repeatedly obstructing good work, that is worth knowing. Amend it
deliberately — the point of writing them down is that abandoning one has to be a decision
rather than a drift.

## Related

- C-0001 — the Vision and Mission this specifies, and the higher precedence on conflict
- BET-0006 — the customer segment, and the cost accepted with it (supersedes BET-0001)
- BET-0007 — the price shape: a floor plus a royalty (supersedes BET-0002)
- BET-0004 — the catalogue; BET-0003, which two of this document's arguments cited, is withdrawn
- BET-0005 — Paridyum operates what it franchises; the reason anti-goal 1 and item 2 both moved
- RFC-0022 — the operator-tenant conflict, and the open question against non-commodity item 4
- ADR-0024 — nested tenancy, the mechanism behind "provable isolation"
- REQ-0013 — the export path anti-goal 4 requires
- ANTI-0007 — regulated data, anti-goal 3
- ADR-0022 — why the bets are a separate artifact type from this document
- BET-0008 — the estate is cost of goods; supplies anti-goal 2's independent argument
- BET-0009 — the narrow permission anti-goal 2 now carries, and the four triggers bounding it
- MKT-0001 — the dated cost measurement anti-goal 2's argument rests on, and its only home
