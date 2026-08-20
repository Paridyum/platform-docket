---
id: C-0009
title: The Veer Covenant — what Paridyum owes those who served, before it owes anyone a return
type: constitution
status: accepted
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [covenant, veterans, armed-forces, veer-nari, charter, non-profit, obligation, glo-0010, mkt-0004, rfc-0026, req-0020]
breaking: false
enforced_by:
  - check: covenant.disbursement_record_current
    severity: error
    rationale: >
      The failure mode of a commitment like this is never repeal. It is silence — a year
      passes, revenue is tight, nobody decides anything, and the obligation is simply not
      discharged. No check can verify that the right amount reached the right people. A check
      CAN verify that a dated entry exists and is under twelve months old, which converts
      silence into a red gate and a false entry into a deliberate lie rather than a drift.
      That is the honest ceiling of mechanical enforcement here, and it is stated as the
      ceiling rather than sold as more.
---

# Constitution C-0009 — The Veer Covenant

## Status

**`accepted` as of 2026-08-18, and it will not stay that way.**

This article is created `accepted` rather than `locked` for two reasons, one procedural and one
substantive.

The procedural one is the `new-decision` rule that no artifact is created `locked` — ADR-0029 was
frozen mid-authoring by the ADR-0025 checkpoint hook before its author had finished counting, and
both of its errata exist because of that ordering.

The substantive one is ADR-0002. `locked` requires a named check, and the check named in
`enforced_by` above **has no subject yet**: Paridyum has no revenue, so there is nothing to
disburse and no ledger entry that could be stale. A check with no subject is indistinguishable
from a check that does nothing, and the CHANGELOG already records this repository shipping two
such checks and mistaking their silence for success.

> **This article is promoted to `locked` on the day Paridyum recognises its first rupee of
> revenue.** Not at incorporation, not at first customer — at first revenue, because that is the
> day obligation I becomes payable and the day `covenant.disbursement_record_current` acquires
> something to be right or wrong about.

Recording the promotion condition here is the point. A commitment whose enforcement is deferred
to an unnamed future is the shape this repository exists to refuse.

## Why this article exists

Every other artifact in `knowledge/` answers a question about how to build or sell something.
This one does not, and the difference should be visible rather than smoothed over.

Paridyum owes this to nobody. Statutory CSR under §135 of the Companies Act 2013 binds companies
above ₹500 crore net worth, ₹1,000 crore turnover, or ₹5 crore net profit. Paridyum is at none of
those thresholds and, at the date of writing, is not yet incorporated. **There is no law, no
customer, no investor and no regulator requiring a single line of this document.** That is not a
weakness in the argument; it is the argument. A duty assumed while it is still optional is the
only kind that tells you anything about the company assuming it.

The reason it is in the constitution rather than in a mission statement, a bet, or a page on a
website is the reason RFC-0005 gave for putting strategy here at all: `CLAUDE.md` instructs every
agent to *never infer, retrieve*, and ranks existing code as the lowest authority. **An
obligation that is not written where decisions are retrieved from is an obligation that will be
inferred away** the first time it is inconvenient — not maliciously, but by an agent or a person
reading a repository in which it does not appear.

## The founder's statement, dated

Recorded in the first person and dated because RFC-0013 D3 established that a conception log
cannot be reconstructed after the fact, and because a covenant with no author is a policy.

> *I want a part of this business that exists for the men and women of the Indian armed forces
> and the paramilitary — for the families of those who did not come back, and for those who came
> back and had to start again. Not as marketing, not as a tax line, not for profit. I am writing
> it into the constitution before there is any revenue to share, because I know what happens to
> intentions that wait for a convenient year.*
>
> — Rohan, 2026-08-18, before incorporation.

## Definitions

**Beneficiary** means what GLO-0010 says it means, in the priority order GLO-0010 sets. This
article deliberately does not restate the definition. One term, one meaning, one home — RFC-0015.

A note on vocabulary, because its absence would otherwise be read as indifference. **This article
does not use the word "martyr."** The Ministry of Home Affairs clarified in December 2013 that
"shaheed/martyr is not defined anywhere" and that it was issuing no such order or notification;
the Ministry of Defence informed the Home Ministry in 2015 that the word is not used in reference
to any of the casualties. The word carries a religious connotation the services have declined —
it denotes someone killed for their religious or political belief. Army Headquarters circulated the
terms that are used instead — **Killed in Action, Battle Casualty, Supreme Sacrifice, Veergati
Prapt, Veer, Bravehearts, Fallen Heroes** — and those are the terms used throughout this
repository. MKT-0004 carries the sources. Using an institution's own words about its own dead is
the smallest available form of respect, and it costs nothing.

## The obligations

Four slots. **Two are live** (I and III), **one is triggered** (IV), and **one is considered and
not undertaken** (II). Each states what it costs, including the one not taken.

**The numbering is fixed and slot II is not reused.** An obligation number is a permanent handle,
cited from GLO-0010, MKT-0004, REQ-0020, `product-registry.yaml` and `COVENANT.md`, for the same
reason an artifact id is never renumbered after supersession. Renumbering to close the gap would
also do something worse than break references: it would erase the fact that the question was asked.

### I. The tithe — one percent of revenue

**One percent of Paridyum's revenue, excluding GST and other statutory amounts collected on the
government's behalf, from the first rupee recognised, in every year, whether or not that year is
profitable.**

**The base is revenue and this is the load-bearing choice in the entire article.** Profit is a
residual the company computes: salaries, reinvestment, capitalised development, group structure.
A company can operate lawfully for a decade at zero profit by design, and one percent of zero is
the commitment every organisation would prefer to have made. Revenue is observable, reconcilable
against GST returns by a third party, and cannot be argued down by anyone sitting inside the
company.

**Accepted cost, stated plainly:** in a loss-making year this is paid out of runway. That is not
an oversight in the design. It is the property that distinguishes this from an intention.

### II. The founding share — considered, and not undertaken

**No equity obligation is created by this article.** Decided 2026-08-18, by the founder, before
incorporation.

**What was proposed:** one percent of Paridyum's equity assigned to the covenant vehicle at
incorporation, carried in the MOA, the AOA and the initial cap table from the company's first day.

**Why it was proposed, recorded because the argument does not stop being true:** obligation I
funds the covenant while Paridyum operates and funds nothing if Paridyum is sold. An acquisition
is the single most likely event to end a commitment of this kind — not because anyone repudiates
it, but because the entity that made it stops being the entity that decides. Equity is the only
one of these obligations that pays out in that event.

**Why it was not undertaken:** the founder chose to defer the decision rather than take it before
incorporation. That is a legitimate call and it is recorded as a call rather than as an oversight.

**The cost of deferring, stated rather than softened.** This is the one obligation whose price
rises with delay, and the rise is not gradual. Founding equity written into the incorporation
documents is structural: every later holder acquires a stake in a company that already carries it,
no one is ever asked to approve it, and it appears in diligence as a given. The same one percent
after incorporation is a board resolution, a valuation, a tax event for someone, and a negotiation
with whoever holds the register — conducted, predictably, in the year it is least convenient.
RFC-0013's argument about signed commits is the same shape: cheap at the first commit, structural
afterwards. **Deferring this is not free, and calling it free later would be false.**

**Trigger — the question returns, and it returns often.** This slot is reconsidered at **every
event that alters the cap table**: incorporation, any funding round, the creation of an ESOP pool,
and any issue or transfer of shares. It is also reconsidered no later than **2027-08-18** whether
or not such an event has occurred.

The trigger is written as a recurring one on purpose. "We will see it later" is a decision that
survives only if something makes *later* arrive, and a cap-table event is both the natural moment
to ask and the moment the answer gets more expensive. **Undertaking this obligation at any future
date is an amendment upward** and requires none of the ceremony in the amendment barrier below.

### III. The grant of capacity

**Any business owned or operated by a beneficiary runs on Paridyum at no charge, permanently, with
the identical isolation guarantee a paying tenant receives.**

Not a discounted tier. Not a sandbox. Not a trial that converts. The same runtime, the same
nested-tenancy guarantee, the same export path REQ-0013 requires — because a degraded tier offered
to this population would be worse than offering nothing, and would be correctly read as a gesture.

This is the obligation that is **Paridyum's specifically to give.** Anyone can send money to the
Armed Forces Flag Day Fund, and many organisations with more money than Paridyum will ever have
already do. Almost nobody can hand a Veer Nari running a business the thing C-0006 names as the
hardest non-commodity asset on the platform: provable isolation, nested, inherited from the
runtime rather than rebuilt. MKT-0004 records that the resettlement estate's self-employment
schemes are physical-asset attachments — coal transport, retail fuel outlets, milk booths,
Janaushadhi pharmacies — with **essentially nothing at the software and operations layer.** That
layer is the one Paridyum owns.

**Accepted cost:** capacity consumed and not billed. Under BET-0008 the estate is cost of goods
rather than a revenue line, so this is a cost-of-goods item and needs no new financial category —
but it is a real cost that scales with uptake and it is not capped here. If it must ever be
bounded, the bound is written as an amendment under the barrier in the next section, in the open,
with a number.

REQ-0020 makes this representable in the tenancy model rather than retrofitted as a discount flag.

### IV. The operated product — triggered, not deferred

**Paridyum builds and operates a product for this community, entering the catalogue under the
same rules as any other vertical.**

Registered in `product-registry.yaml` as `veer-entitlements`, `kind: catalogue`,
`stage: proposed`. Its selecting evidence is MKT-0004, which is what makes the registration
legitimate rather than cheap.

**Trigger for promotion to `stage: building`: the first franchised catalogue vertical has a paying
customer who has renewed.** This is C-0006 anti-goal 6 applied to this product exactly as it
applies to every other, and it is applied deliberately. **A covenant product exempted from the
rules that keep products good would be a worse product**, and this population is served badly by
enough well-meant, under-resourced software already. Anti-goal 1's sequencing rule binds here too:
a vertical Paridyum will not operate itself is one it may not franchise.

The shape MKT-0004's evidence points at is **entitlement navigation** — the KSB / RSB / ZSB /
DESW / ECHS / DGR estate is not primarily underfunded, it is unnavigable, and the documented
failures are eleven-year and thirty-year waits caused by paperwork routed to the wrong authority.
That shape is a finding, not yet a decision. The decision is taken when the trigger fires, in an
artifact written then.

## The amendment barrier

Constitution articles can be superseded. That escape hatch cannot be removed by a document, and
pretending otherwise would be the false confidence ADR-0002 exists to prevent. What this article
can do is make using the hatch expensive and visible.

> **This article may be superseded only by an artifact that increases the obligation.**
>
> **A supersession that reduces, suspends or removes any obligation in it must state, in its own
> text: the sum not paid, the period over which it was not paid, and the beneficiaries who did
> not receive it.** It must be signed under C-0003 by the person deciding, and the superseded
> article stays in the repository unaltered, as every superseded artifact does.

This converts a quiet deletion into a written confession, in git, signed, permanently. It is the
strongest thing prose can do to a future signatory, it is weaker than a law, and it is stated as
weaker rather than dressed up.

**Amending upward requires none of this.** Raising the tithe, widening the grant, or promoting
obligation IV is an ordinary supersession.

## What this is not

**Not a `BET-`, and the distinction is structural rather than stylistic.** Per ADR-0022 a bet is
`lockable: false` because `locked` means must-not-be-contradicted and a bet exists precisely to be
contradicted by evidence on a date already written down. A bet carries a `trigger` naming what
would kill it. **Nothing may kill this.** Anyone proposing to restate the covenant as a bet is
proposing — whether or not they intend it — to make it killable, and this paragraph exists so that
proposal has to be made in the open.

**Not CSR.** CSR is a statutory allocation of profit by companies above a threshold. This is a
pre-statutory allocation of revenue and equity by a company below every threshold. Schedule VII
clause (vi) happens to name this beneficiary class, which is useful when the vehicle is
structured (RFC-0026) and is not the reason the obligation exists.

**Not marketing.** No obligation here is contingent on it being publicised, and none of it may be
made contingent later. The disbursement ledger in `COVENANT.md` is a record kept because records
that only exist when convenient are not records — publishing it is permitted, and the covenant
does not lapse if nobody reads it.

**Not a substitute for paying taxes**, and not to be structured primarily for deduction. Where a
tax benefit follows the vehicle chosen in RFC-0026, take it; where the efficient structure and the
faithful one diverge, take the faithful one and record the cost.

## Consequences

**Accepted costs.**

1. One percent of revenue, unconditionally, including in years with no profit and years with a
   loss. This reduces runway at precisely the moments runway is scarcest.
2. **A gap at obligation II, and it is a cost rather than a saving.** Nothing in this covenant
   pays out if Paridyum is sold, which is the event most likely to end it. Deferring past
   incorporation also converts a free structural act into a priced negotiation. Both are accepted
   as of 2026-08-18 and both are revisited at the trigger written in that slot.
3. Uncapped free capacity under obligation III, scaling with uptake, with no bound written today.
4. A commitment made before revenue exists, by people who cannot yet know what one percent will
   feel like when it is a real number. This is a genuine risk and the barrier above is the
   deliberate response to it: the exit stays open and is made costly to use rather than closed.
5. `enforced_by` claims less than the article obligates. The check sees whether an entry exists
   and is current. It cannot see whether the amount is right, whether the beneficiaries were
   genuinely eligible, or whether the grant of capacity was honoured. **Those are upheld by
   judgement**, exactly as C-0006 says of a market position, and no proxy check will be added to
   make the gap look smaller than it is.

**Gained.**

1. An obligation that cannot be quietly dropped, in the one place every agent and every future
   employee is instructed to retrieve rules from.
2. A recurring trigger on the one question that was left open, so that deferring it costs a
   conversation at every cap-table event rather than costing nothing and being forgotten.
3. A written answer to a question that would otherwise be inferred. Per RFC-0005: the absence of a
   written answer is not neutral, it is an instruction to guess.

**Rejected alternatives.**

- **One percent of profit.** Rejected because profit is computed by the party paying. It reads
  identically on a website and plausibly costs nothing for a decade.
- **A discretionary annual allocation approved by the board.** Rejected because it is what every
  such intention becomes, and because the first bad year is the year it does not happen.
- **A `BET-` with a review date.** Rejected on the structural ground above.
- **Equity only, no revenue line.** Rejected because it defers every rupee to an exit that may
  never occur, and an obligation costing nothing until then is not one.
- **Building obligation IV first.** Rejected because it collides with C-0006 anti-goal 6 and
  C-0008, and because the resulting product would be a shallow vertical built by a company with no
  operating experience — served on the population least able to absorb another one.
- **Armed forces only, excluding CAPF and CPMF.** Rejected in GLO-0010, which carries the
  argument.

## Related

- C-0001 — Vision and Mission; outranks this article, and nothing here contradicts it
- C-0003 — identity and attribution; the signature the amendment barrier requires
- C-0006 — anti-goal 6 and anti-goal 1, which bind obligation IV exactly as they bind hosting
- C-0008 — tenant zero; a covenant tenant is a tenant, never a privileged shape
- GLO-0010 — who a beneficiary is, and the priority order under scarcity
- MKT-0004 — the welfare estate, where it fails, and the sources for every claim above
- REQ-0020 — covenant eligibility as a tenancy attribute rather than a retrofitted discount
- RFC-0026 — the vehicle for obligation I, and what still needs a lawyer; its parked question 1 is slot II's
- BET-0008 — the estate is cost of goods, which is why obligation III needs no new category
- ADR-0022 — why this is not a bet
- ADR-0002 — why this is `accepted` today and `locked` at first revenue
- `COVENANT.md` — the disbursement ledger the declared check reads
