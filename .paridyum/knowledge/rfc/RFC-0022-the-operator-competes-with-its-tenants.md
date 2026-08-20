---
id: RFC-0022
title: What must be true for a founder to build on a platform whose operator competes with them
type: rfc
status: proposed
created: 2026-08-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [conflict-of-interest, isolation, data-use, trust, self-preferencing, export, audit]
breaking: true
enforced_by: []
---

# RFC-0022 — The operator competes with its tenants

## Status

**PROPOSED**, 2026-08-15. Opened by BET-0005, which commits Paridyum to operating the same
catalogue products it franchises.

**Four boundary options were considered and rejected the same day**, all for one reason: they are
physical-franchise mechanics that do not transfer to a digital business where **tenants run under
their own brand**.

| Rejected | Why |
| --- | --- |
| Vertical exclusivity — never franchise what we operate | Contradicts BET-0005 outright; nothing proven could ever be franchised |
| Territorial exclusivity | Solves scarcity of location. There is no location, and software territories leak |
| Sunset / hand-over — operate, then exit | Rejected by decision: Paridyum does not intend to exit its own operations |
| Retail price handicap — always price above franchisees | Wrong axis (the conflict is not price), unenforceable across many tenants at many prices, arguably worse than neutral since premium positioning is an advantage, and it edges toward a vertical-restraint question that would need CCI research before it could be written |

**One bound was accepted**, and it is the only one: **nothing tenant-derived informs where
Paridyum competes.** C-0006 non-commodity item 4 was narrowed to say so on the same day.

**White-label removes an exposure that would otherwise dominate this RFC.** `paridyum.com` sells
to prospective *tenants*; a tenant's end customers arrive through the tenant's own domain and need
never learn Paridyum exists. There is no shared shelf, so the platform-ranking self-preferencing
problem that defines app stores and marketplaces **does not arise here**. That is worth stating
because it is the first thing a reader familiar with the Amazon or Apple cases will look for.

## The question

**A founder is deciding whether to run their business on infrastructure operated by a company
that runs the same business. What has to be true for that to be a rational decision?**

This is not a reputational question to be managed with messaging. BET-0005 cost 1 names the
failure — adverse selection on exactly the founders worth having — and with territory,
distribution and exit all ruled out, **the technical guarantee and the data-use commitment are the
entire defence.** This RFC is where that defence is designed, and it must exist before the first
franchised tenant, because a commitment made after the fact is worth a fraction of one made
before.

**The conflict is informational, and once the franchise machinery is stripped away that is all it
is.** Paridyum sees every tenant's customer count, revenue, churn, pricing, feature usage and
growth curve. Tenants see none of ours, and their end customers do not know we exist. We then
compete in the same markets holding that view. Exposure 6 below is therefore not one item on a
list — it is the main event, and the rest of this RFC exists to make it credible that we do not
act on it.

## Why this is urgent precisely because nothing exists

The same argument RFC-0013 makes about intellectual property applies here. The commitments below
are cheap while there are no tenants and no operating history, and they are expensive or
impossible later:

- A data-use commitment made before any tenant data exists is credible. The same words published
  after three years of unrestricted access are a policy change nobody believes.
- An access log that has run since the first row is evidence. One switched on after a dispute is
  an admission.
- Building Paridyum's own tenant through the same code path as everyone else's costs nothing on
  day one and is a migration later.

## What is already decided elsewhere

| Already decided | Where |
| --- | --- |
| Paridyum operates what it franchises; no commercial boundary | BET-0005 |
| Two nested silos, `tenant_id` + `account_id`, RLS forced | ADR-0024 |
| Operator scope is **a grant, never an absence** | GLO-0005 |
| Switching cost is not the retention strategy; export is a sales asset | C-0006 anti-goal 4 |
| Export exists at both levels | REQ-0013 |
| Provable isolation is ours to build; *provable* is the product | C-0006 non-commodity 2 |
| The denial is the assertion | STD-0009 |
| An agent's authority is scoped, its model pinned, its actions auditable | REQ-0007 |

## The threat model, stated as a tenant would state it

Six distinct exposures. Only the first is the one people think of, and it is not the worst.

1. **Direct read of tenant data.** ADR-0024 puts every tenant in one database behind forced RLS.
   An operator with database credentials can bypass RLS. This is the obvious exposure and the
   most tractable.

2. **Aggregate and derived data — and this one contradicts a live artifact.** C-0006's
   non-commodity list, item 4, states as an *asset*:

   > **Aggregate operational data.** What a business looks like when it is working, across
   > thousands of them. Benchmarks, defaults, and eventually models. Available to nobody who does
   > not host the runtime.

   Written under BET-0001 and BET-0003, that was aggregation across customers who were not
   competitors. **Under BET-0005 it is competitive intelligence gathered from businesses Paridyum
   competes with.**

   **Resolved 2026-08-15** — item 4 was narrowed rather than kept or dropped: aggregate data may
   improve the runtime and set product defaults, and **may never inform where Paridyum competes.**
   D2 carries the mechanism. This was the sharpest unresolved conflict in the strategy and it is
   now a decision rather than a caveat.

3. **Roadmap self-preferencing.** A catalogue feature that reaches Paridyum's own tenant first
   gives it a head start in a market it shares with franchisees. Invisible from outside and easy
   to do accidentally, since Paridyum's tenant is the natural place to test.

4. **Pricing self-preferencing.** Paridyum's own tenant pays neither floor nor royalty (BET-0007).
   Its unit economics are therefore structurally better than any franchisee's in the same
   vertical, which is a permanent competitive advantage rather than an accounting artefact.

5. **Support and operational priority.** An incident affecting Paridyum's own revenue and one
   affecting a franchisee's compete for the same on-call engineer.

6. **Catalogue selection informed by tenant performance.** Paridyum can observe which franchised
   verticals perform, then enter them. With no boundary (BET-0005), nothing prevents this — and
   it is the exposure most likely to be discovered by a franchisee after the fact and experienced
   as betrayal.

## Proposal

### D1 — Paridyum's own tenant is a tenant, with no privileges

It is provisioned through the same saga, stored in the same silos, reached through the same code
path, and granted no operator scope. GLO-0005 already makes scope a grant; this says the grant is
never made to Paridyum's operating tenant.

**This is testable, and STD-0009 already specifies how**: the denial is the assertion. A test that
Paridyum's own tenant cannot read another tenant's rows is the single most valuable test in the
system, because it is the one a prospective franchisee would ask for.

### D2 — A published data-use commitment, written before there is data to use

Naming what Paridyum may and may not derive from tenant data. The proposed shape:

- **No per-tenant data leaves the tenant boundary** for any purpose other than operating the
  platform for that tenant.
- **Aggregate statistics only above a stated k-anonymity threshold**, and never in a vertical
  where Paridyum operates a competing tenant.
- **Tenant performance may never inform where Paridyum competes** — not which vertical to enter,
  not which segment to target, not when. This is exposure 6, which is the main event.

**C-0006 non-commodity item 4 was narrowed to match on 2026-08-15**, so the artifacts now agree.
Aggregate data may improve the runtime and set product defaults; it may not select a market.

### D2a — the part of D2 that cannot be checked, made evidentiary instead

**A decision about which market to enter is not a code path.** No check sees it, and pretending
otherwise would be the false confidence ADR-0002 exists to prevent. Data *access* is testable
(D1, D5); *acting on what you know* is not.

The proposal: **every vertical Paridyum enters is recorded in an artifact naming the evidence that
selected it, and that evidence may not be tenant-derived.** A check verifies the artifact exists
and that its cited sources are external — public market data, Paridyum's own operating experience,
customer conversations outside the tenant base. It cannot verify that the *real* reason matches
the stated one; it can make an unstated reason conspicuous, and it makes the commitment auditable
by the counterparty rather than only by us.

This is prose-with-frontmatter that a gate can read, which is this repository's founding argument
applied to the one commitment that resists direct testing. **Its limit must be stated wherever it
is claimed:** it raises the cost of quietly breaking the promise; it does not make breaking it
impossible.

### D3 — Parity commitments on roadmap and price

- **No catalogue capability reaches Paridyum's own tenant before franchised tenants**, beyond a
  stated and bounded canary window.
- **Paridyum's own tenant is billed internally at the same floor and royalty as any franchisee.**
  The money is circular, but the accounting is not: it keeps unit economics comparable, and it is
  what makes BET-0005 trigger clause 3 measurable rather than arguable.

### D4 — Export moves from a deferred requirement to a launch commitment

C-0006 anti-goal 4 already says a tested export path is a sales asset rather than a concession.
Under this RFC it becomes the guarantee of last resort: **the credible ability to leave is what
makes staying rational when the landlord competes with you.** REQ-0013 is `deferred` on the
trigger *"the first tenant-authored row"* — which now fires at the first franchised tenant, and
should be treated as firing then rather than discovered later.

### D5 — A tenant-visible operator access log

If a Paridyum operator accesses a tenant's data, **that tenant can see it** — not on request, by
default. This is the strongest single trust mechanism available and the cheapest to build early,
since `platform-audit` already exists as a planned context and RFC-0009 open question 2 expects
its trigger to fire during the first wave anyway.

Self-attestation is worth very little. A log the counterparty reads is worth a great deal.

### D6 — The separate legal entity stays deferred, with a trigger

BET-0005 rejected it as premature and named the reconsideration point. Restated here so it is
attached to the conflict rather than only to the bet: **reconsider when BET-0005 trigger clause 2
approaches** — one in four declining founders citing the competing operation.

## Alternatives considered

**Draw a commercial boundary after all.** The standard franchise answer, and it dissolves most of
this RFC. Ruled out by BET-0005 on 2026-08-15. Recorded here because if that decision is revisited,
D2 through D5 shrink considerably, and this is the cheapest available fix to exposure 6.

**Say nothing and rely on isolation being good.** The default path, and the reason it fails is that
a guarantee nobody has published is a guarantee nobody can rely on. C-0006 already makes this
argument about `locked`: an unenforced promise is a suggestion.

**Contractual commitment only, no technical mechanism.** Cheaper and faster. Rejected on the same
ground ANTI-0007 rejects a disclaimer: terms allocate liability between parties, they do not
prevent the access, and they are not what a technically sophisticated founder is asking about.

**Technical mechanism only, no published commitment.** Insufficient in the other direction. D1
prevents the access that goes through the application; it does nothing about exposures 2, 3, 4 and
6, which are all decisions rather than reads.

## Open questions

1. ~~**Does C-0006 non-commodity item 4 survive?**~~ **Answered 2026-08-15: narrowed.** Aggregate
   data may improve the runtime and set product defaults; it may never inform where Paridyum
   competes. A real asset given up, and given up because the alternative is a platform whose
   customers are rationally afraid of it. Kept in this list, struck through, because the reasoning
   matters more than the outcome and a deleted question reads as one that was never asked.
2. **Is a tenant-visible operator access log achievable under ADR-0024?** Shared database with
   forced RLS means operator access is credentialed at a level the application may not see.
   Whether every access path can be logged, or only application-mediated ones, decides whether D5
   is a guarantee or a partial one — and a partial one must be described as partial.
3. **Does the commitment need to be contractual to be worth anything?** A published policy can be
   changed unilaterally. A term in the tenant agreement cannot. The second is much more credible
   and much harder to reverse, which is the point.
4. **Who audits it?** Self-attestation against one's own commitment is close to worthless. A third
   party costs money that does not exist yet. REQ-0012 requires vendor controls to *produce
   evidence*; refusing our own tenants what we demand of vendors is C-0006 anti-goal 4's
   incoherence argument in a new place.
5. **Does D3's roadmap parity survive contact with reality?** Paridyum's own tenant is the natural
   place to test a change before exposing franchisees to it. That is good engineering and a head
   start at the same time, and the canary window is where the tension lives.

## Risks

- **R1 — The commitments are made and then quietly eroded.** The failure mode of every neutrality
  promise. D5's log is the only proposal here that resists it, because it is evidence rather than
  assertion.
- **R2 — D1 is implemented and never tested.** The 2026-08-06 audit found checks reporting success
  while enforcing nothing — ask `scripts/check-knowledge.sh` for the current split rather than
  trusting a figure written here. This is the check that most needs to be watched failing.
- **R3 — The conflict is disclosed well and founders decline anyway.** Honest disclosure does not
  make the conflict acceptable to everyone, and BET-0005 trigger clause 2 exists because that is a
  real outcome rather than a communications failure.

## Related

- BET-0005 — the structure that opens this, and the decision that removed the commercial fix
- BET-0006, BET-0007 — the segment that bears the conflict and the royalty it pays
- C-0006 — non-commodity items 2 and 4, anti-goal 4; item 4 is in direct conflict with D2
- ADR-0024, GLO-0004, GLO-0005 — shared database, nested silos, scope as a grant
- STD-0009 — how D1 is tested; the denial is the assertion
- REQ-0013 — export, which D4 promotes to a launch commitment
- REQ-0007, REQ-0012 — scoped authority and evidence-producing controls
- ANTI-0007 — why a disclaimer is not a mechanism
- RFC-0009 — `platform-audit`, which D5 depends on
