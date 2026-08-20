---
id: RFC-0026
title: The covenant vehicle — where the tithe goes, and what holds it when the sums are small
type: rfc
status: proposed
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [covenant, vehicle, section-8, trust, affdf, equity, incorporation, legal, c-0009, mkt-0004]
breaking: false
enforced_by: []
sources:
  - url: https://vinodkothari.com/2023/03/comparative-analysis-between-social-sector-entities-section-8-companies-and-trusts/
    title: Comparative analysis between social sector entities — Section 8 companies and Trusts — Vinod Kothari Consultants
    retrieved: 2026-08-18
    supports: >
      THE DECIDING FACT FOR THIS RFC, and it is about exit rather than about setup. A Section 8
      company is a body corporate with perpetual succession, distinct from its members, prohibited
      from distributing dividend in any form and from returning surplus assets to members on
      dissolution — and it CANNOT BE VOLUNTARILY STRUCK OFF: dissolution requires IBC liquidation
      or licence revocation by the Central Government. A charitable trust under the Indian Trusts
      Act 1882 or state public trust law is not itself a legal entity (the trustees are), has no
      central registration authority, carries a far lighter compliance burden, and can be wound up
      by deed modification with assets transferring per the trust instrument and no regulatory
      involvement. Compliance for a Section 8 company includes MGT-7/7A, AOC-4, DIR-8, DIR-3KYC
      and statutory audit under s.139.
      SECONDARY SOURCE — a professional firm's analysis, not the statute. Read as orientation for
      a conversation with counsel, never as the basis for a filing.
  - url: https://ksb.gov.in/armed-forces-flag-day-fund.htm
    title: Armed Forces Flag Day Fund — Kendriya Sainik Board
    retrieved: 2026-08-18
    supports: >
      Option C's target. AFFDF constituted 1993, administered by Kendriya Sainik Board, Managing
      Committee chaired by the Defence Minister, funding medical treatment, mobility equipment,
      home loan interest subsidy, education for children of the fallen and vocational training for
      widows. The page offers a contribution route but states neither 80G status nor account
      details, which is open question 3 below.
      PRIMARY SOURCE for what the fund is; silent on the mechanics this RFC needs.
  - url: https://ca2013.com/135-corporate-social-responsibility/
    title: Section 135 — Corporate Social Responsibility, Companies Act 2013 (Integrated Ready Reckoner)
    retrieved: 2026-08-18
    supports: >
      Establishes that no statutory CSR obligation applies to Paridyum at any threshold — net worth
      ₹500 crore, turnover ₹1,000 crore, net profit ₹5 crore. The vehicle is therefore being chosen
      for durability and for the beneficiaries, not to discharge or anticipate a legal duty.
      SECONDARY SOURCE — the Act as published is the authority.
---

# RFC-0026 — The covenant vehicle

## Status

**PROPOSED, and deliberately not an ADR.**

C-0009 is a decision and it is taken. This is the implementation of one part of it, and the
implementation requires a chartered accountant and a lawyer. Writing it as an ADR would record a
decision nobody qualified has taken. **Nothing in this document should be filed, signed or acted on
without professional advice**, and the recommendations below are written to be the agenda for that
conversation rather than a substitute for it.

**This RFC does not gate C-0009.** The obligations are true from 2026-08-18. This decides only
where the money goes and how the share is held. If this RFC stalls, obligation I accrues as a
liability and is disbursed when the route exists — it does not lapse.

## The question

C-0009 creates one thing that needs somewhere to be:

- **Obligation I** — 1% of revenue, ex-GST, annually, from first revenue.

**Obligation II — the founding equity share — was considered and not undertaken on 2026-08-18.**
C-0009 slot II carries that decision, the argument for the obligation, the cost of deferring it,
and a recurring trigger that returns it at every cap-table event. **This RFC therefore designs a
vehicle that receives money and does not hold shares**, and the difference matters: it removes the
one requirement that would have forced a decision before incorporation.

## Why the timing no longer dominates, and what it dominates instead

An earlier revision of this document opened by arguing that timing was the constraint on
everything below, because founding equity is nearly free before incorporation and expensive at
every later date. **That argument is unchanged and is now C-0009 slot II's rather than this
document's.** It is not repeated here, and this RFC does not re-argue a decision the constitution
has recorded.

What follows from the change is narrower and worth being explicit about: **nothing in this RFC has
a deadline any more.** With no share to hold, the vehicle question is purely about where money goes
once money exists, and money does not exist. That is a genuine relaxation and should be read as
one — a document that kept its urgency after losing its reason would be manufacturing pressure.

**One consequence survives the relaxation.** Option A's case rested partly on a Section 8 company
being able to hold the equity in its own name rather than through a trustee. That argument is now
inert. Option A still wins, on the exit argument alone, and it is worth noticing that it would
have won either way — a recommendation that changes when one of its two supports is removed was
resting on the wrong one.

## Options considered

### Option A — Section 8 company

A company licensed under §8 of the Companies Act 2013, with the covenant as its object.

**For.** A body corporate with perpetual succession, distinct from its members — so it can hold
property in its own name rather than through a trustee. Prohibited from distributing dividend in
any form, and from returning surplus assets to members on dissolution.
MCA-registered and publicly verifiable. And the fact that decides this RFC: **it cannot be
voluntarily struck off.** Dissolution requires IBC liquidation or licence revocation by the Central
Government.

That last property is the point. C-0009's amendment barrier makes repealing the covenant a written
confession; a Section 8 vehicle makes dismantling the recipient a process with a regulator in it.
**The two barriers are independent, which is why they are worth having together** — the document
barrier binds the people, the corporate barrier binds the assets, and neither is defeated by
defeating the other.

**Against.** Real ongoing cost. MGT-7/7A, AOC-4, DIR-8, DIR-3KYC, statutory audit under s.139,
directors, board meetings, minutes. On a fund receiving one percent of pre-revenue or early revenue
this is administratively heavier than the sums moving through it, and a vehicle whose compliance
cost approaches its disbursements is not obviously serving anyone.

### Option B — Public charitable trust

A trust under the Indian Trusts Act 1882 or the applicable state public trust legislation.

**For.** Cheap and quick. No central registration authority, no mandatory statutory audit unless
the trust deed requires it, materially lighter filings.

**Against, and it is the whole case.** A trust is not itself a legal entity — the trustees are, and
they hold property in that capacity. Winding up is a deed modification, with assets transferring
per the instrument and **no regulatory involvement at all.** For a covenant whose entire design
premise is that the exit should be visible and expensive, choosing the vehicle that can be closed
quietly by the people who set it up inverts the premise. Trustee-held property also carries a
succession question that a body corporate with perpetual succession does not — currently moot,
and live again the moment C-0009 slot II is ever undertaken.

### Option C — Route to AFFDF via Kendriya Sainik Board, and stand up the vehicle later

Contribute obligation I directly to the Armed Forces Flag Day Fund — constituted 1993, administered
by Kendriya Sainik Board, Managing Committee chaired by the Defence Minister — and defer the
vehicle.

**For.** Zero setup cost, zero compliance, and the money reaches an established distribution network
that is unambiguously better at reaching beneficiaries than a new one-person fund would be. MKT-0004
is explicit that this estate does not have a funding-network gap. In year one, one percent of
Paridyum's revenue routed through a century-old structure does more good than the same amount minus
audit fees routed through a company Paridyum built to feel like it was doing something.

**Against.** It discharges obligation I and nothing else — it does nothing for obligation III or
IV. It also gives up the direct relationship with beneficiaries that
obligation III eventually requires — Paridyum needs to be able to verify eligibility (REQ-0020),
and a fund it contributes to anonymously builds none of that.

## Recommendation

**C then A, with the transition condition written down now.**

1. **At incorporation: nothing is required of this RFC.** With C-0009 slot II not undertaken, the
   incorporation documents carry no covenant clause and no share is committed. Recorded as a step
   so its emptiness is visible rather than inferred — an earlier revision made this step one and
   called it the only item with a deadline.
2. **From first revenue:** obligation I is paid to AFFDF through Kendriya Sainik Board, recorded in
   `COVENANT.md` with the date, the amount, the revenue base it was computed on, and the
   acknowledgement reference.
3. **Transition to a Section 8 company when the annual tithe first exceeds ₹5,00,000**, or when
   obligation III has more than 25 active beneficiary tenants, whichever comes first. Either
   condition means the covenant has a real balance sheet or a real beneficiary relationship, and at
   that point the compliance cost is proportionate and the direct relationship is needed.
4. **Option B is rejected outright**, on the exit argument, and should not be revisited on cost
   grounds.

The transition condition is written now rather than left to judgement because "when it makes sense"
is how a temporary arrangement becomes the permanent one.

## Open questions

Each needs a named answer before this RFC can resolve to an ADR.

1. **PARKED — no longer blocking.** *Can pre-incorporation founding equity be committed to an
   entity that does not yet exist, and what is the cleanest instrument?* Express trust declared by
   the founder, a subscriber clause in the MOA, an irrevocable undertaking, or issue-on-formation.
   **Counsel.** Kept in place and kept numbered because C-0009 slot II's trigger returns at every
   cap-table event, and this is the question that must be answered when it does. It carried a
   deadline until 2026-08-18 and no longer does.
2. **What is the tax treatment of the tithe in each configuration** — deductible business expense,
   80G donation, or neither — and does the answer change the recommended routing? **CA.** Note
   C-0009's rule: where the efficient structure and the faithful one diverge, take the faithful one
   and record the cost.
3. **Does AFFDF carry 80G status, what are the contribution mechanics, and does it acknowledge
   contributions in a form that can be recorded in `COVENANT.md`?** The KSB page offers a
   contribution route and states neither. **Answerable by asking KSB directly**, and it should be
   asked before first revenue rather than at it.
4. **Can a contribution be earmarked by tier?** GLO-0010 puts Tier 1 first. If AFFDF pools all
   contributions, obligation I's routing does not honour the priority order — which may be an
   acceptable cost of using a better network, and should be an acknowledged one rather than an
   unnoticed one.
5. **MOOT, and the reason is worth reading.** *What happens to the one percent of equity on a
   liquidity event where the vehicle does not yet exist?* There is no equity, so there is nothing
   to strand — but the underlying exposure did not go away with the question. **On an acquisition
   today, the covenant receives nothing at all**, because obligation I follows revenue and revenue
   stops being Paridyum's. C-0009 accepted cost 2 records this; it is restated here because this is
   the document someone reads when an acquisition is actually on the table.
6. **FCRA.** If any foreign contribution is ever received, or if a foreign investor's presence
   affects the vehicle's status, FCRA registration or prior permission is engaged. Not live today;
   flagged so it is not discovered late.
7. **Who governs the vehicle**, and specifically: is a beneficiary on its board? A covenant for a
   community, governed entirely by people outside it, is a familiar and avoidable failure.

## Related

- C-0009 — the covenant; obligation I is what this RFC implements, and slot II is what it no longer does
- GLO-0010 — the beneficiary definition, and the tier order open question 4 concerns
- MKT-0004 — the estate, AFFDF's role in it, and the CSR thresholds establishing no statutory duty
- REQ-0020 — the verification relationship option C does not build
- RFC-0013 — the ordering argument this RFC borrows: cheap at the first commit, structural after
- `COVENANT.md` — where every disbursement under this RFC is recorded
