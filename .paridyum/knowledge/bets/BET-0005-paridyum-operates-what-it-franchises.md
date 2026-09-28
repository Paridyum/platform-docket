---
id: BET-0005
title: Paridyum operates the catalogue it franchises, as a tenant on its own runtime
type: bet
status: accepted
created: 2026-08-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [strategy, franchise, paridyum-as-tenant, catalogue, conflict-of-interest, isolation]
breaking: false
trigger: >
  Any ONE of the three kills the bet; they are ordered by how early each can fire.
  (1) OPERATION — Paridyum's own tenant fails to reach one hundred paying end customers on
  its first catalogue product within nine months of that product going live. The wager is
  that operating the product is what makes franchising it credible; if we cannot run the
  business, we are selling a playbook we do not have, and the catalogue reverts to BET-0004
  standing alone.
  (2) REJECTION — more than one in four founders who evaluate and decline name Paridyum's
  competing operation as a reason, measured over the first forty evaluations. That is the
  direct test of cost 1, which has no commercial mitigation by decision.
  (3) DRIFT — eighteen months after the first catalogue product goes live, revenue from
  Paridyum's own tenant exceeds revenue from franchised tenants by more than five to one.
  At that ratio the company is a vertical SaaS business that also owns a platform, and the
  honest response is to say so rather than to keep describing the platform as the product.
review_date: 2027-02-01
enforced_by: []
---

# BET-0005 — Paridyum operates the catalogue it franchises

## Status

**ACCEPTED** as of 2026-08-15. A bet, not a decision: per ADR-0022 this artifact can never be
`locked`, and the `trigger` above is the condition under which it should be killed.

This is the structural fact BET-0004 did not state, and stating it changes who the customer is,
where first revenue comes from, and what the isolation guarantee is protecting against.

## The wager

**Paridyum runs catalogue products itself, as a tenant on its own runtime, selling directly to
end customers — and licenses the same products to founder-tenants who run them as their own
businesses. Operating a vertical precedes franchising it.**

The runtime is what makes both cheap; the catalogue is what is sold; **operating it is what makes
the sale credible.**

### The franchise analogy, and where it stops

*Added 2026-08-15, because the analogy was doing more work than it had earned.*

**Tenants run under their own brand.** A tenant's end customers need never learn Paridyum exists.
That is the opposite of a franchise, where renting the franchisor's brand is the main thing being
bought — and it means the franchise machinery does not transfer:

- **No territory.** Territorial exclusivity solves physical scarcity; there is exactly one good
  corner in a town and infinite shelf space here. This is a digital business and location-bounded
  exclusivity is meaningless in it.
- **No shared shelf, so no self-preferencing surface.** `paridyum.com` sells to prospective
  *tenants*; a tenant's customers arrive through the tenant's own domain. Paridyum and its tenants
  do not compete for the same visitor, so the platform-ranking conflict that defines app stores
  and marketplaces does not arise here.
- **No brand cannibalisation**, because there is no shared brand to cannibalise.

**The closer analogy is a private-label manufacturer that also sells its own label** — one line
producing supermarket own-brand and its own, which is common, well understood, and nobody calls it
a franchise. What protects the customer there is supply terms and **information separation**, not
territory.

Keep the franchise word only for what it explains well: a proven playbook sold to someone who
wants to operate rather than build. Everything else in it is borrowed from a physical business and
should be checked before use.

## The three levels, stated once

ADR-0024's nested tenancy already provides the structure. This bet names what occupies it.

> **Corrected 2026-09-26 by ADR-0109.** The top row of the table below said `Operator | Paridyum`
> and the dogfooding paragraph called Paridyum "a paying operator of its own runtime". Both used
> `operator` in the sense GLO-0001 no longer carries — the commercial party above a tenant — while
> this same bet uses the *runtime* sense eleven lines later ("operator scope is a grant, never an
> absence"). One word, two levels, in one document.
>
> The top row is now **Platform**, which is what Paridyum is in GLO-0004's corrected tree and is not
> a silo. The commercial party is unnamed on purpose: ADR-0024 D9 defers it, and this bet's
> structural claim — that a tenant may be a founder's business or Paridyum's own — is unaffected,
> because it was never about that level.
>
> The `operator-as-tenant` tag went the same way, to `paridyum-as-tenant`: a tag is a search term,
> and one carrying the ambiguity is how the ambiguity keeps getting found and copied.

| Level | Who | How Paridyum earns |
| --- | --- | --- |
| Platform | Paridyum — builds and operates the runtime and the catalogue | — |
| Tenant | (a) founders and agencies running a catalogue product as their business<br>(b) **Paridyum itself**, running catalogue products directly | (a) flat fee + commission, per BET-0007<br>(b) direct revenue from end customers |
| Account | (a) a founder's end users · (b) **Paridyum's own end customers** | — |

**This dissolves a fork that looked real and was not.** A small business or freelancer wanting
invoicing software and a founder wanting to *run* an invoicing business are not competing
segments to choose between — the first is an `account` inside Paridyum's own tenant, the second
is a `tenant`. They sit at different depths of one tree. C-0006 anti-goal 6 is therefore not
engaged: this is one market with two levels, not two markets.

## Why this could be right

1. **Proof of operation is the pitch.** *"Here is a business we run, and you can run one too"*
   is a categorically stronger offer than *"here is a template."* It converts BET-0004's
   catalogue from a shelf of software into a set of demonstrated businesses.

2. **It forces depth, which is C-0006 anti-goal 1's real objection.** The anti-goal's binding
   clause is *"shallow in each."* You cannot operate a business you are shallow in — customers
   leave. Operating before franchising converts the anti-goal from a prohibition into a
   sequencing rule: **do not franchise a vertical you have not operated.**

3. **First revenue does not depend on the wedge being right.** Paridyum's own tenant earns from
   end customers directly. BET-0006 becomes a wager we can afford to test *after* there is
   income, rather than a precondition for having any. This is the single largest risk reduction
   available to the company and it is the main reason to accept this bet.

4. **Dogfooding stops being a practice and becomes structural.** Every failure mode a
   founder-tenant would hit is hit first by Paridyum, as a tenant of its own runtime paying the
   same way, with its own customers affected. BET-0006 cost 2 — that solo founders tolerate an immature
   runtime — stops being load-bearing, because the runtime is proven against a real business
   before anyone else's depends on it.

5. **It defines the persona.** *"Who wants to launch an invoicing business?"* was BET-0004's
   weakest point and had no answer. The answer is **someone who wants to own a software business
   under their own name without building it** — they get a proven playbook and keep the brand
   equity, which makes this a better offer than a franchise rather than a worse one: nothing is
   rented and nothing reverts. A far sharper target than "a solo founder with an idea."

6. **BET-0004's clause-1 measurement becomes direct observation.** Engineer-days per product are
   visible because we build and operate product one ourselves, which is what TASKS T-0.5
   requires and could not otherwise get cheaply.

## Costs accepted

C-0001 principle 8: *an artifact that lists only benefits is incomplete and probably wrong.*
Cost 1 is the one that can kill the company slowly, and it has been made worse by decision.

1. **Paridyum competes with its own tenants, and the conflict is informational.** On 2026-08-15
   the question *"does Paridyum commit to a boundary between what it operates and what it
   franchises"* was answered **no** — no vertical exclusivity, no territory, no first-mover
   claim, no exit. Those were all rejected as physical-franchise mechanics that do not transfer
   to a digital business, and the reasoning is above.

   **What remains, once territory and distribution are removed, is asymmetry of information.** We
   see every tenant's customer count, revenue, churn, pricing, feature usage and growth curve.
   They see nothing of ours, and their end customers do not know we exist. We then compete in the
   same markets holding that view. That is the whole of the conflict and it is sharper for being
   the only part left: it produces adverse selection on exactly the founders worth having, since
   the most capable ones will notice.

   **One bound was accepted on 2026-08-15, and it is the only one:** nothing tenant-derived
   informs where Paridyum competes. Aggregate data may improve the runtime and set product
   defaults; it may never select a vertical or a segment. C-0006 non-commodity item 4 was narrowed
   to say so, and RFC-0022 D2 carries the mechanism. Trigger clause 2 measures whether it is
   believed.

2. **The isolation promise changes what it protects against, and becomes the only mitigation
   for cost 1.** C-0006 lists *"provable isolation at two levels"* as ours to build. Under this
   bet the promise is no longer only *we isolate tenants from each other* — it is **we isolate
   tenants from us.** GLO-0005's rule that operator scope is *a grant, never an absence* stops
   being hygiene and becomes the load-bearing commercial commitment. With territory, distribution
   and exit all ruled out, **the technical guarantee and the data-use commitment are the entire
   defence** — and the second is only partly checkable, since a decision about which market to
   enter is not a code path. RFC-0022 is where both are worked out, and it must exist before the
   first franchised tenant.

3. **This is three companies at once.** Operating a real invoicing business — sales, support,
   marketing, domain expertise — while building a platform while building a catalogue. The
   sequencing below addresses it by making them consecutive rather than concurrent, but capacity
   is the most likely practical failure of this plan and no artifact can engineer it away.

4. **Paridyum's own tenant meets the incumbent's free tier head-on.** C-0006 anti-goal 1's third
   clause — *shallow loses to the incumbent's free tier* — is a category error when applied to a
   founder reselling multi-tenant software, and **literally true** when applied to Paridyum
   operating invoicing against Zoho and Wave. The anti-goal was not wrong; it was aimed one
   level away from where it bites. C-0006 is amended accordingly.

5. **Operating revenue arrives first and is easier, so drift is the default.** Direct end-customer
   revenue needs no founder to be convinced of anything. Franchise revenue needs a second,
   harder sale. A company that takes the easy revenue for eighteen months becomes a vertical SaaS
   company with an unusually good internal platform — which is a fine business and not this one.
   Trigger clause 3 exists because nobody notices this drift while it is happening.

## Sequencing this implies

**Paridyum's own tenant comes first, and it is the product-market-fit instrument.** Operate
product one, acquire real end customers, learn the vertical deeply, and only then offer
*run this yourself*.

Two consequences for the build order:

- **`platform-entitlement` moves earlier than RFC-0009 §7 placed it.** The moment Paridyum runs a
  tenant alongside founder-tenants, *what may this tenant do* is a live question rather than a
  deferred one. Its trigger — *the first plan distinction that must be enforced rather than
  described* — fires on the first franchised tenant.
- **The catalogue order (TASKS T-0.6) gains a real criterion.** Not *which vertical sounds
  attractive* but **which vertical can Paridyum plausibly operate as a real business, in India,
  without touching regulated data.** ANTI-0007 and C-0006 anti-goal 3 are hard filters on that
  list, and they remove more candidates than they first appear to.

## Alternatives rejected

**Franchise only; never operate.** The cleanest answer to cost 1 — no competition with tenants,
no conflict to disclose, no self-preferencing question. Rejected because it discards benefits 1,
2 and 3 together, and benefit 3 is the risk reduction that makes the rest of the plan
financeable. It also leaves anti-goal 1's *shallow in each* objection completely unanswered.

**Operate only; never franchise.** Simpler, faster to revenue, and it is what trigger clause 3
watches for the company sliding into by default. Rejected because it abandons the platform thesis
entirely — C-0001's Vision, C-0006's non-commodity list and the whole of `platform-*` exist to
serve tenants who are not us. Worth revisiting only as an explicit decision, never as a drift.

**Operate through a separate legal entity.** The conventional fix for the conflict: a subsidiary
runs the businesses, the platform stays neutral. Rejected for now as premature — it adds
incorporation and accounting cost before there is revenue, and it does not actually solve the
data-access question, which is technical rather than corporate. **Worth reconsidering when
trigger clause 2 approaches**, because it is the strongest available response to it.

## Related

- BET-0004 — the catalogue this operates and franchises; unchanged, and revised 2026-08-15
- BET-0006 — the wedge, re-derived on this structure
- BET-0007 — the price shape, which this makes coherent
- C-0006 — amended 2026-08-15; anti-goal 1 and the non-commodity list both move because of this
- RFC-0022 — the conflict of the platform competing with its own tenants, which cost 1 and cost 2
  make urgent. **Its title still says "operator" in the pre-ADR-0109 sense**; it is `proposed` and
  outside ADR-0109 D5's correction list, so it is noted here rather than edited in passing.
- ADR-0024, GLO-0004, GLO-0005 — the nested tenancy this occupies, and operator scope as a grant
- ANTI-0007 — the hard filter on which verticals Paridyum may operate
- ADR-0022 — why this is a bet rather than an ADR
