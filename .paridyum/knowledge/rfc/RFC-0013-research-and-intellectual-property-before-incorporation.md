---
id: RFC-0013
title: How research becomes intellectual property before there is a company
type: rfc
status: accepted
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [ip, patents, trade-secrets, research, disclosure, india, governance]
breaking: false
resolved_by: null
verified_by: []
enforced_by: []
---

# RFC-0013 — How research becomes intellectual property before there is a company

## Status

**ACCEPTED** on 2026-08-18, closing T-0.2. Proposed 2026-08-11.

**All five decisions are adopted.** D1–D4 are live now; D5 stays deferred on its own trigger, which
this acceptance does not fire.

| | Decision | Owner | State on acceptance |
| --- | --- | --- | --- |
| D1 | Technical-effect thesis precedes the runtime | Rohan Batra | **Owed.** One page, before the first runtime commit |
| D2 | Baseline measured before the mechanism exists | Rohan Batra | **Owed.** Benchmark harness lands before any optimised path |
| D3 | Signed commits and a conception log from the first commit | Rohan Batra | **Already satisfied for signing** — verified 2026-08-18, every commit in this repository carries a good signature. The conception log is owed |
| D4 | Present assignment at incorporation, DPIIT before filing | Rohan Batra | **Owed at incorporation.** Wording drafted under T-0.8 |
| D5 | Docket machinery deferred | — | Deferred. Fires at the first candidate, or the first non-founder with access |

**Owner is one person for all four, and that is a finding rather than a formality.** D3's
conception log and D4's assignment both exist to survive a second contributor arriving. Neither is
tested until one does, and the first non-founder access is also D5's trigger — so the day this
system is first exercised is the day it is first relied upon, with no rehearsal. Recorded here
because acceptance is the moment that becomes a known risk instead of an unexamined one.

**What acceptance costs, stated plainly:** one page and roughly one day of benchmark harness. D3
was already true. That the cost is this small is the whole argument of the section below — every
mechanism here is nearly free while the estate is empty, and several are impossible afterwards.

## The question

C-0006 claims *provable tenant isolation* on its not-commodity list. BET-0004 stakes the company
on one shared multi-tenant runtime beneath a catalogue. RFC-0007 and ADR-0024 establish that
tenancy is nested — operator → tenant → account — and that isolating the inner silo is ours to
do rather than the application's.

Together those artifacts assert that Paridyum's defensibility lives in a mechanism. Nothing in
this repository says **who owns that mechanism, when it may be shown to anyone, or what protects
it once it is shown.** There is no `patent`, no `trade secret`, and no `intellectual property` in
any artifact — the grep returns empty.

So: **what governs the path from research to protected invention, and what must be decided before
the first line of runtime code exists?**

## Why this is urgent precisely because nothing exists

The estate is zero services, zero code, nothing published, one person, and an incorporation in
flight. That is not a reason to defer. It is the entire argument for acting now.

RFC-0010 made the same trade in the opposite direction: it skipped a feedback window because the
migration set was empty and *"the window exists to protect something; here there is nothing yet to
protect."* The inverse applies here. Every mechanism proposed below is nearly free while the
estate is empty and either expensive or **impossible** once it is not:

- A baseline measurement cannot be taken after the naive implementation has been replaced.
- A contemporaneous conception record cannot be manufactured afterwards and remain evidence.
- A public disclosure cannot be withdrawn. There is no undo, in any jurisdiction, ever.
- An assignment signed at incorporation is a formality; the same assignment sought from a
  departed contributor in diligence is a negotiation.

Nothing here is on fire. That is the point. This is the only window in which all of it is cheap.

## What the law actually requires

Four findings constrain the design. All are India-specific and all are load-bearing.

### Section 39 fixes the order of filing, on pain of criminal liability

A resident of India may not file a patent application outside India first. Either an Indian
application is filed at least six weeks earlier, or a Foreign Filing Licence is obtained on Form
25 (₹1,600 for a natural person, startup or small entity; the Controller must respond within
twenty-one days under Rule 71). Violation carries imprisonment up to two years under §118, and
under §40 the Indian application may be deemed abandoned or the patent revoked.

This inverts the sequencing in essentially all publicly available startup IP advice, which is
written for US residents and begins with a US provisional. **India first is not a preference. It
is a criminal statute.**

### Section 3(k) grants on measurable technical effect, and the guidelines changed in 2025

The Indian Patent Office issued new Guidelines for Examination of Computer Related Inventions on
**29 July 2025**, replacing the 2017 guidelines and addressing AI, ML, blockchain and quantum
directly. The four-limb test treats business methods as an **absolute bar** and treats
mathematical methods, algorithms and computer programmes as **qualified** exclusions escapable by
technical effect.

The guidelines require that effect to be concrete and quantified. *"Improves performance"* fails.
*"Reduces average memory access latency by prioritising high-frequency data structures based on
real-time cache-miss analysis"* passes. Metrics are expected in the claim itself.

AI-generated inventions — autonomously produced, no human inventor — are not patentable, since §6
requires a human applicant. AI-assisted inventions, where a human directs and the model is a
tool, are not excluded.

### Section 31's grace period is not a safety net

India's twelve-month grace period covers a paper read by the true and first inventor before a
**learned society** and published in that society's transactions, plus exhibitions gazetted by
government notification. Rule 29A and Form 31, introduced by the Patents (Amendment) Rules 2024,
now require a formal claim with evidence.

A preprint server is not a learned society. A demo is not a learned society. A public repository
is not a learned society. **For every disclosure channel Paridyum is realistically going to use,
the Indian grace period does not exist.** The EPO's is narrower still.

### There is no trade secret statute in India

The Law Commission's 289th Report and the draft Protection of Trade Secrets Bill both date from
March 2024 and remain unenacted as of this writing. Protection rests on contract, equitable
breach of confidence, and the Contract Act — and §27 of that Act makes non-competes largely
unenforceable, so confidentiality and assignment clauses are the only terms doing real work.

A trade secret strategy in India is a **contracts** strategy. If the contracts are not signed,
there is no strategy.

## The finding this RFC is built on

The CRI 2025 guidelines require, for an AI invention, disclosure of: network architecture at
layer-level detail, training dataset characteristics and labelling methodology, the complete
preprocessing pipeline, training parameters, and validation results demonstrating the claimed
effect. The Madras High Court reinforced the standard in *Caleb Suresh Motupalli* (January 2025):
a specification assembling published literature without enabling implementation fails sufficiency.

That is a reproducibility checklist. It is very close to what NeurIPS asks of a submission, and it
is the same instinct STD-0009 already encodes for isolation — that an untested cell is the default
state of every matrix nobody checks, and that a policy without a test has only been proven to
exist.

**The patent disclosure standard, the publication standard, and this repository's own testing
standard are converging on one artifact.** A mechanism documented well enough to satisfy STD-0009
and measured well enough to be believed is most of a filing. That collapse is what makes the
proposal below cost almost nothing: it asks for no artifact that good engineering practice does
not already produce.

The one thing it does add is **system-level measurement** — latency, memory, throughput, density —
alongside correctness. Isolation tests prove behaviour. They do not produce the numbers §3(k)
grants on.

## What is patentable here, and what is not

This deserves stating plainly because the answer is not flattering to the most visible part of the
strategy.

**BET-0004's catalogue is a business method.** A commercial arrangement in which customers select
named vertical products from a shared operator is squarely the absolute bar in §3(k)'s second
limb. No claim drafting rescues it. The catalogue is a good wager and it is not an invention.

**The nested tenancy isolation mechanism is the candidate.** RFC-0007's inner silo, ADR-0024's
decisions, and the enforcement of a complete authorisation matrix across two nested discriminators
form a systems contribution with a plausible measurable effect: tenant and account density per
node, query overhead of the nested discriminator, p99 latency under load, and the cost of proving
isolation rather than asserting it. Those are machine-level improvements, and they are what C-0006
has already claimed as non-commodity.

This asymmetry is the most useful output of this RFC. **The architecture decision is the patent
decision.** A runtime that is genuine systems work has something to protect; orchestration over
someone else's models has nothing to claim, and no process fixes that afterwards.

## Proposal

### D1 — A technical-effect thesis precedes the runtime

Before runtime code is written, one page names the mechanism, the effect it produces, the metric
that demonstrates the effect, and the baseline it improves on. If that page cannot be written, the
finding is about the product rather than the paperwork, and it is better to know before building.

### D2 — The baseline is measured before the mechanism exists

A benchmark harness capturing latency, memory, throughput and density lands before the optimised
path does. A claim of improvement requires a measured predecessor, and once the naive
implementation is gone it cannot be recovered. This is a day of work now and unavailable later.

### D3 — Signed commits and a conception log from the first commit

Git history is a contemporaneous, dated, tamper-evident record of conception. Signed, it is better
evidence than a lab notebook, and it makes the invention record assemble from artifacts the work
produces anyway rather than from a form somebody fills in under deadline. Stage-2 record-keeping
is skipped everywhere it is manual; the fix is to make it not manual.

### D4 — Present assignment at incorporation, DPIIT before filing

Work created before incorporation belongs to its author personally and does not transfer to the
company by operation of forming one. Incorporation documents must carry a **present** assignment —
*hereby assigns*, not *agrees to assign* — of pre-incorporation IP.

Filing sequence: build and measure → incorporate → obtain DPIIT recognition → file. DPIIT requires
incorporation and unlocks the ₹1,600 fee tier, the 80% patent fee rebate and a SIPP facilitator.
Filing before it is a straight loss.

One route is closed and should not be attempted: the PCT's 90% international filing fee reduction
for Indian natural persons is conditioned by the PCT Assembly's 2017 Understanding on the applicant
being sole and true owner **under no obligation to assign** to an ineligible party. Filing
personally to capture it while intending to assign to Paridyum is the precise case that
Understanding forecloses. Budget company rates.

### D5 — Gate and docket machinery is deferred, with a trigger

A registry with zero entries governs nothing. The disclosure ledger, IP docket, triage gate and
CI tripwire are **not** built now. They are built when the first candidate exists — which is also
when the first person other than the founder gains access, because those two events will be close
together and either one alone justifies the machinery.

When built, the docket is a new artifact type and therefore requires an `artifact-registry.yaml`
change under ADR-0022. That is out of scope here and deliberately not pre-empted.

The default disposition at that gate is proposed as **trade secret**, with one sharp exception:
trade secrets fall to independent discovery, which is a complete defence rather than an
infringement. Anything a competent competitor will build anyway on their own timeline should be
filed. Hold what is non-obvious and invisible in the product; file what is non-obvious and
inevitable. Distinguishing those two is the only judgement in the system that cannot be automated.

## The research system

The five decisions above govern output. They do not help produce it. This section proposes the
working half — a place to accumulate research and a protocol for attacking it.

**Claims, not notes.** Material enters `research/inbox/` at zero friction and with no schema.
Promotion out of the inbox means writing a **falsifiable claim**: a statement, what would have to
be true for it to be wrong, and the cheapest measurement that would kill it. A notes system grows
monotonically and becomes a swamp because nothing in it can ever be resolved. A claims system has
a death mechanism.

**The stress pass** is adversarial, scheduled rather than ad hoc, and asks five questions of a
claim:

1. What must be true for this to be wrong?
2. Who has already done this?
3. What is the strongest version of the opposite position?
4. What single measurement would settle it?
5. What is the cheapest form of that measurement?

Question 2 is the load-bearing one, because it is simultaneously the literature review, the related
work section, and the prior art search. Running it once and reading it through three lenses is the
same collapse D2 and the CRI findings rely on: prior art search gets skipped when it is framed as
legal work, and does not get skipped when it is framed as finding out whether the idea is new.

**A claim that survives stress and carries measurements is three things at once**: a validated
design decision eligible to become an ADR or a bet, a paper's thesis, and a patent's technical
effect statement. One object, three readers.

`research/` sits outside `artifact-registry.yaml` and is therefore ungoverned and un-gated. This is
deliberate. Working material subjected to CI would either be sanitised into uselessness or would
break the build; the precedence order in CLAUDE.md already places everything unratified below
everything ratified, and research is an input to RFCs rather than a peer of them.

## Alternatives considered

**Wait until incorporation completes.** The tidiest option, and it forfeits precisely the
irreversible items. Baseline measurement, contemporaneous conception records and the
never-disclosed state all belong to the period *before* the company exists. Deferring costs
nothing that can be bought back later, which is the definition of the wrong thing to defer.

**Build the full governance machinery now** — disclosure ledger, docket, triage gate, CI tripwire.
This was the first shape of this proposal and it was wrong. A registry with zero entries governs
nothing, and ceremony built years before its first use decays before it is needed. D5 defers it
behind a trigger instead, which is the same bargain ADR-0022 struck by making `deferred` a real
status carrying a trigger rather than a synonym for *no*.

**File a US provisional first**, as essentially all publicly available startup IP advice
recommends. Rejected because it is illegal. §39 makes foreign-first filing by an Indian resident a
criminal offence under §118 with revocation under §40. This alternative is recorded rather than
omitted because it is the default any agent or founder will reach for, and the reason it is wrong
is not discoverable from the sources that recommend it.

**File personally to capture the PCT 90% reduction, then assign to Paridyum.** Saves roughly
USD 1,100 per family. Rejected: the PCT Assembly's 2017 Understanding conditions the reduction on
the applicant being under no obligation to assign to an ineligible party, which describes this
plan exactly.

**Trade secret only; never file.** The strongest alternative, and it nearly wins. It needs no
counsel, no fees and no disclosure, and since India has no trade secret statute the contracts must
exist either way. It loses on two counts: independent discovery is a complete defence rather than
an infringement, and nested-tenancy isolation is the kind of mechanism competitors arrive at
independently — so the protection evaporates in the case where it matters. Second, C-0006 claims
provable tenant isolation as non-commodity, and a moat that cannot be asserted against anyone is a
marketing line. D5 keeps trade secret as the *default* disposition while refusing it as the
*only* one.

**Defensive publication only**, via a free platform such as TDCommons. Guarantees freedom to
operate and blocks competitors from patenting the mechanism. Rejected as the primary strategy for
the same reason as above — it buys the right to use, never the right to exclude — but it remains
the correct disposition for candidates that fail the file/hold test, and belongs in the gate when
D5 fires.

**Govern `research/` under `artifact-registry.yaml`.** Rejected: CI applied to working material
either sanitises it into uselessness or breaks the build on half-formed thinking. The precedence
order in CLAUDE.md already ranks unratified material below everything ratified, so research needs
no additional subordination.

## Open questions

- Whether to file on anything. That requires a candidate, which does not yet exist.
- Jurisdictions beyond the lean route: IN provisional → PCT at twelve months → national phase
  decided at thirty. The 2024 Rules cut the Indian request for examination from forty-eight months
  to thirty-one for applications filed on or after 15 March 2024, so month thirty is now a
  collision of decisions rather than a single one. Recorded here, resolved elsewhere.
- Choice of counsel, and whether the International Searching Authority is the IPO (₹10,000) or the
  EPO (~USD 2,237). The lean route's value depends entirely on the quality of evidence available at
  month thirty, which argues for the expensive search on any family taken seriously.
- The docket artifact type, per D5.

## Risks

**The technical-effect thesis may not be writable.** If the runtime turns out to be integration
rather than mechanism, D1 fails and the honest conclusion is that there is no patent strategy —
only trade secret and speed. That is a legitimate outcome and it is better surfaced by a one-page
exercise than by counsel's invoice.

**Deferring the gate risks disclosure before it exists.** D5 accepts this because the founder is
currently the only person with access and nothing is public. The trigger must fire on the *earlier*
of first candidate and first additional contributor, not the later.

**Measurement discipline decays.** D2 is the decision most likely to be quietly dropped under
delivery pressure, and it is also the one that is unrecoverable. It should be enforced by a check
when the machinery of D5 is built.

**This RFC is not legal advice** and is written by a non-lawyer against public sources current to
August 2026. Every statutory claim above should be confirmed by Indian patent counsel before it is
relied on. The sequencing constraints in §39 and the disclosure-is-irreversible constraint are the
two that must be confirmed first, because they are the two whose violation cannot be corrected
afterwards.

## Related

- **C-0006** — market posture. Its not-commodity list names *provable tenant isolation*, which is
  the asset this RFC exists to protect.
- **BET-0004** — the catalogue wager. Its commercial shape is a business method and unpatentable;
  the runtime beneath it is where any invention lives.
- **RFC-0007 / ADR-0024** — nested tenancy and inner-silo isolation. The candidate mechanism, and
  two nested discriminators in one database is the specific thing that would be claimed.
- **STD-0009** — tenant isolation testing. Proves behaviour; does not produce the system-level
  numbers §3(k) grants on, which is the gap D2 fills.
- **ADR-0022** — artifact registry governance. A future docket artifact type changes it.
- **ADR-0013** — significance is computed, never self-declared. The readiness score in D1/D2 is the
  same instinct applied to inventions.
