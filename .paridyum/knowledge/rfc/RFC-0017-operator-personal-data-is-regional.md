---
id: RFC-0017
title: An operator's personal data lives in the operator's region, not the control region
type: rfc
status: proposed
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [residency, operator, personal-data, gdpr, dpdp, identity, adr-0047, req-0003]
breaking: true
enforced_by: []
---

# RFC-0017 — An operator's personal data lives in the operator's region

## Status

**PROPOSED** as of 2026-08-11. Feedback deadline **2026-09-10**.

**Fourth of four.** RFC-0014 defines the structure, RFC-0015 how the planes behave,
RFC-0016 the edge. This one closes a hole the other three left open, and it **contradicts ADR-0047
decision 1**, which is `accepted` rather than `locked` — so an RFC is the right instrument.

## The question

RFC-0014 and RFC-0015 place *tenant* data correctly and say nothing about the *operator*.

An operator is not an identifier. GLO-0001: *"the party that holds a Paridyum subscription and runs
one or more tenants. The operator is our customer."* A customer has a name, a billing contact, an
email address and staff who log in. All of it personal data under the regimes ADR-0041 puts in
phase 1.

So "operators are global, they live in the control plane" places a German customer's personal data
in an Indian database — an untreated cross-border transfer, in the system whose selling point is
provable residency.

**A search of the estate returns nothing on operator residency.** ADR-0047, ADR-0041, REQ-0003 and
GLO-0001 are collectively silent. This was never decided and never rejected. It was missed, and it
was missed because the word *operator* appears in a sentence about staff.

## Decision — split two populations ADR-0047 treats as one

ADR-0047 decision 1 says the `paridyum`
instance *"holds staff **and** operators"*, and gives one reason: *"N regional copies of a staff
directory means N joiner processes and N places to forget to deprovision."* That argument is sound
**about staff** — they are ours, they are few, and deprovisioning them is a security control. It was
never an argument about **operators**, who are customers, are many, and are wherever their business
is. Lumping the two together is where the hole came from.

| Population | Who | Where their personal data lives |
| --- | --- | --- |
| **Staff** | Paridyum's own people | One directory, control region. ADR-0047's argument holds unchanged. |
| **Operators** | Our customers | Their own region, beside the tenants they run. |

**What stays in the control plane is a skeleton with no person in it:** operator id, which tenants
it owns, which region each is in, which plan applies. Identifiers and relationships, no names, no
addresses, no billing. This is the same stripping rule the rejected entitlement projection needed —
except here it is not an exception to "one fact, one home", it is that rule applied to a fact whose
home was wrong.

**This contradicts ADR-0047 decision 1 as written**, which is `accepted` rather than `locked`, so an
RFC is the right instrument. The ADR this resolves into must state the narrowing explicitly: the
`paridyum` instance holds **staff**, and the word *operators* comes out of that sentence.


## Consequences

**Accepted costs**

1. **An operator's own login now depends on their region, not the control region.** A customer whose
   region is unreachable cannot sign in to administer it. Arguably correct — there would be nothing
   there to administer — but it is a real coupling and it is new.

2. **Two identity populations to reason about, permanently.** Staff authenticate against the control
   region's `paridyum` instance; operators against their own region's. Every tool that touches both
   has to know which is which, and the two will be confused at least once.

3. **ADR-0047 decision 1 has to be narrowed**, which means an ADR revision rather than a clean
   addition. The word *operators* comes out of one sentence; nothing else in that ADR moves.

4. **Billing spans the split.** A subscription is a control-plane fact and an invoice carries a
   billing address, which is personal. Where the boundary falls inside billing is open question 1,
   and `platform-billing` is deferred (ADR-0042 decision 3), so it is cheap to leave open now and
   will not be later.

**Gained**

- The residency guarantee becomes true for every person in the system, not only for a tenant's end
  users. Before this, the customer's own data was the one place it leaked.
- ADR-0047's deprovisioning argument survives intact for the population it was actually about.
- "One fact, one home" stays a rule with no exceptions — this is that rule applied to a fact whose
  home was wrong, not a carve-out from it.

## Alternatives considered

- **A control plane per continent**, so each continent's operators are held locally. The intuitive
  answer, and it fails on ADR-0047's own argument: N control planes means N staff directories, N
  joiner processes, N places to forget a deprovision — the exact cost ADR-0047 refused. It would
  also make an operator with businesses on two continents unrepresentable, and multiply RFC-0015
  decision 3's jurisdiction-loss risk by the number of continents rather than containing it in one.

- **Keep operators global and rely on transfer mechanisms** — standard contractual clauses, adequacy
  decisions. Legally available and rejected on posture: C-0006 sells *"provable isolation"*, and a
  guarantee that holds for a tenant's customers but not for the customer themselves is the kind of
  distinction that reads as a loophole the first time anyone asks.

- **Hold no operator personal data at all**, keeping only identifiers and pushing name and billing to
  a payment vendor. Genuinely attractive — the least data is the least risk — and rejected because
  staff still need to contact a customer, which means a name and an address exist somewhere we
  control. Worth revisiting when `platform-billing` is designed.

## Open questions

1. **Where does billing sit?** A subscription is global; an invoice has a billing address and is
   personal. The likely split is that the subscription is control-plane and the invoice is regional,
   but `platform-billing` is deferred and this should be decided with it rather than before it.

2. **Which Zitadel instance holds operator staff users?** The regional instance already exists per
   ADR-0047 and holds tenant organizations. Whether an operator's own administrators become an
   organization in that instance, or something beside it, is an identity-model question RFC-0012's
   successor owns.

3. **Does this change what `platform-tenancy` owns?** ADR-0042 gives it the whole operator → tenant →
   account hierarchy. After this decision the operator's *identifier* is control-plane and the
   operator's *person* is regional, so the context straddles the split more sharply than RFC-0014
   decision 3 already noted.

## Reversal conditions

- **Reverses if operator personal data turns out to be legally exempt** in the regimes ADR-0041
  phase 1 names — which would need a written legal opinion, not an engineering judgement.
- **Reverses toward option 3 above** if `platform-billing`'s design shows that no operator personal
  data needs to exist in our systems at all. That is a better outcome than this decision, not a worse
  one, and it is why option 3 is recorded rather than dismissed.

## Related

- ADR-0047 — decision 1, the sentence this RFC narrows; its deprovisioning argument, which survives
- GLO-0001 — the operator is our customer, which is why their data is a customer's data
- ADR-0041 — phase 1 regimes, and `tenant.region` as the contractual fact this extends upward
- REQ-0003 — item 8 owns residency; personal-data classification at the migration that creates it
- RFC-0014 — the plane split this corrects the control-plane row of
- RFC-0015 — "one fact, one home", which this applies rather than excepts
- ADR-0042 — `platform-tenancy` owns the hierarchy that now straddles the split (open question 3)
- C-0006 — "provable isolation" as the product, which is what makes the alternative a posture problem
