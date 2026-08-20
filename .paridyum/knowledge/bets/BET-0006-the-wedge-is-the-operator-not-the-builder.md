---
id: BET-0006
title: The wedge is defined by wanting to run a business, not by being one person
type: bet
status: accepted
created: 2026-08-15
locked_at: null
supersedes: [BET-0001]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [strategy, icp, wedge, franchise, go-to-market, retention]
breaking: false
trigger: >
  REPLICATION — of the first forty franchised tenants, fewer than one in four reaches half the
  end-customer revenue that Paridyum's own tenant on the same product had reached at the same
  age, measured from each tenant's go-live. Paridyum's own operation is the control group, which
  is the measurement BET-0005 makes available and BET-0001 could not have had. If the playbook
  does not replicate at that rate, what is being sold is software with a story attached rather
  than a business in a box, and the franchise premise is wrong regardless of how many sign up.
review_date: 2026-12-01
enforced_by: []
---

# BET-0006 — The wedge is the operator, not the builder

## Status

**ACCEPTED** as of 2026-08-15, superseding BET-0001. A bet, not a decision: per ADR-0022 this
artifact can never be `locked`.

**BET-0001 was not falsified. Its supporting argument expired underneath it and nobody re-ran
the derivation.** That is recorded plainly below, because the failure is more instructive than
the conclusion.

## Why BET-0001 needed superseding

BET-0004 withdrew BET-0003 on 2026-08-08 and re-justified two bounded contexts the same day
(RFC-0009 Revision 2). It said of the wedge only: *"BET-0001 — the wedge; unchanged, and a
catalogue may reach it differently."* That is a note, not a derivation. Checking BET-0001's four
reasons against a catalogue:

| # | BET-0001's reason | Under BET-0004 and BET-0005 |
| - | --- | --- |
| 1 | "The only segment that produces learning at volume" | **Void.** The entire argument was BET-0003's contract-coverage risk. |
| 2 | "The only segment that tolerates an immature runtime" | **Void as a differentiator.** BET-0005 proves the runtime by operating it first. |
| 3 | "The generated application layer is worth most to someone who cannot write it" | Survives reworded: a *catalogue* product is worth most to someone who cannot build it. |
| 4 | "Acquisition can cost approximately nothing" | Survives intact. |

Reason 1 did not merely die — **it inverted.** Its argument was that five hundred solo founders
produce a diverse distribution of application specifications while *"five agencies produce five
deep, correlated opinions."* Under a catalogue nobody is designing a contract to cover
everything; the task is choosing a handful of verticals to build deeply. **Deep correlated
opinions are now the more useful input.** The exact property that disqualified agencies is the
property the catalogue wants.

BET-0001 rejected agencies on two grounds — *"on learning rate… and because they will not accept
an immature runtime."* Reason 1's inversion voids the first. BET-0005 voids the second, since a
vertical is operated by Paridyum before it is franchised. **Both rejection grounds are gone**, and
BET-0001 already conceded agencies are *"better on every revenue metric."*

## The wager

**The first franchised tenant is someone who wants to run a business on a proven playbook rather
than build software — and the distinguishing property is motivation, not headcount.**

A solo operator and a five-person agency are the same customer with different capacity. BET-0001
said so in its own beachhead section: *"an agency or a team is the same shape with more
members."* Segmenting on team size was the error; segmenting on **operate versus build** is the
correction.

Who this is:

- an individual who wants to run a software business and has no intention of writing one;
- a freelancer or consultant who wants a product line rather than billable hours;
- an agency or operator already running businesses for clients, for whom one relationship is N
  tenants and their org chart is our data model.

Who this is **not**: the small business or freelancer who simply needs invoicing software. That
person is real, wanted, and served — as an `account` inside **Paridyum's own tenant** under
BET-0005, not as a tenant. They are a customer of the catalogue, not a franchisee of it. Keeping
those two straight is the whole reason this bet exists.

## This is a beachhead, not a destination

Carried forward from BET-0001 unchanged in substance, because it was right and remains the most
frequently misread section in `bets/`.

**The intended end state is many markets. This bet is about which one is first, not which one is
only.** C-0001's Vision is multi-market by definition. The constraint is **sequencing**, never
ambition, and C-0006 anti-goal 6 should be read that way.

**Design the contract for breadth; sell to one segment.** The contract may not assume:

- one operator per business — an agency is the same shape with more members;
- one business per paying account — BET-0007 prices per business, plural;
- that the business has no end users of its own, or that it has them;
- **that money flows the way it does in a subscription product** — BET-0007 adds a commission on
  the tenant's own transactions, and a contract that assumed pure subscription would have to be
  reopened to carry it.

Each is cheap to avoid now and structural to remove later.

## Why this could be right

1. **The persona is defined for the first time.** *"Who wants to launch an invoicing business?"*
   had no answer while the offer was software. It has one when the offer is a proven playbook:
   **someone who wants to own a software business under their own name without building it.**

   The brand is theirs, not ours (BET-0005) — tenants run white-label and their end customers
   need never learn Paridyum exists. So this is a better offer than a franchise rather than a
   worse one: nothing is rented, nothing reverts, and the equity they build is their own. That
   population is large, well understood, and reachable by content and community — BET-0001's
   reason 4, which survived intact.

2. **The demonstration is the product.** BET-0005 has Paridyum operating the vertical first, so
   the pitch is a running business with real customers and real numbers rather than a promise.
   This is the reason franchising works generally and it is not available to a pure software
   platform.

3. **Agencies are now additive rather than deferred.** BET-0001 called them *"the most likely
   successful migration path once the runtime is proven"* — and BET-0005 is what proves it.
   Nothing needs to be delayed for them; one relationship is N tenants, N floor fees and N
   commission streams.

4. **The commission aligns us with replication rather than acquisition.** Under BET-0007 we earn
   when a franchisee's business earns. That points the whole company at making tenants succeed,
   which is the property BET-0001's cost 1 said was missing.

## Costs accepted

1. **Software was never this customer's bottleneck, and it still is not.** BET-0001's cost 1
   survives intact and is only partly mitigated. Businesses fail for want of customers. A
   franchise playbook that includes *how to get customers* is a much larger commitment than
   shipping software, and BET-0005's operating experience is the only credible source for it.
   **If we cannot say how a franchisee gets their first ten customers, we are selling software
   and calling it a franchise.**

2. **Retention and satisfaction still decouple, and it is still a metric trap.** A franchisee can
   love the product and close their business. Any retention figure must separate *churned because
   the business closed* from *churned because Paridyum was wrong*, or it is not worth reading.
   The trigger is written to sit on the right side of that line.

3. **Low willingness to pay meets a per-tenant cost that never sleeps.** Unchanged from BET-0001
   cost 3. An idle tenant costs schema, DNS, certificate, backups and monitoring share
   continuously. This is why **ADR-0024 must be maximum-density from day one** and why BET-0007's
   floor cannot be zero. Commission does not rescue an idle tenant; it earns nothing from one.

4. **Self-serve removes the human who would have caught a bad fit.** Unchanged. Worse under a
   franchise frame, where the implied promise is larger — an unsuitable franchisee is a
   reputational cost, not just churn.

5. **Adverse selection on enthusiasm.** The people most excited by *"run a business in minutes"*
   skew toward those with no demand to validate. Unchanged from BET-0001 cost 5, and the franchise
   framing may amplify it.

6. **Franchisees compete with their franchisor, by decision.** BET-0005 cost 1: Paridyum operates
   the same verticals it franchises and has committed to no boundary. This is a cost of *this*
   wedge specifically, because it is the franchisee who bears it.

## Alternatives rejected

**Keep BET-0001 as written.** Rejected: two of four reasons are void and the segmentation axis
was wrong. Leaving it in place would mean CLAUDE.md's retrieval rule returns an argument that no
longer holds, which is worse than no answer — an agent retrieving *"solo founders, for learning
rate"* would design for spec diversity that nothing now consumes.

**Agencies as the sole wedge.** Genuinely defensible now that both rejection grounds are void,
and better on every revenue metric. Rejected because it is not a different wedge — it is the same
wedge at greater capacity, and writing it as a separate segment would recreate the size-based
segmentation this bet exists to remove. Sell to whoever wants to operate; expect agencies to
convert better and let that be an observation rather than a plan.

**Small businesses and freelancers as the wedge.** This is BET-0001's rejected alternative 2 and
it stays rejected *as a tenant segment*, because it reframes C-0001's Mission from *launch a
business* to *run the one you have*. Under BET-0005 it needs no rejection at all: they are served
as accounts inside Paridyum's own tenant, which is a better answer than either accepting or
refusing them.

## What would confirm it

The `trigger` states what kills this. For symmetry, and all three must hold together:

- Franchised tenants collecting real revenue from their own end users at a rate that makes the
  replication trigger clearly unmet.
- Churn dominated by businesses closing rather than by Paridyum being replaced.
- Franchisees choosing verticals Paridyum does **not** operate, which would be the strongest
  available evidence that BET-0005 cost 1 is survivable.

BET-0001's third confirmation — *"application specifications diverse enough to have found real
gaps in the runtime contract"* — is deliberately dropped. It was measurable only under BET-0003
and would have quietly become unfalsifiable.

## Related

- BET-0001 — superseded; its beachhead section is carried forward, its reasons 1 and 2 are void
- BET-0005 — the operator-as-tenant structure that makes this wedge coherent and supplies its control group
- BET-0007 — the price shape this segment pays
- BET-0004 — the catalogue being franchised
- C-0006 — market posture, amended 2026-08-15
- ADR-0024 — the density constraint cost 3 forces
- ADR-0022 — why this is a bet rather than an ADR
