---
id: REQ-0020
title: Covenant eligibility is an attribute of the tenancy model, not a discount applied to a price
type: requirement
status: deferred
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [covenant, tenancy, eligibility, entitlement, veterans, c-0009, glo-0010, adr-0024]
breaking: false
trigger: >
  The first migration that creates the tenant table in `platform-tenancy`, or the first artifact
  that fixes the shape of a tenant record — whichever comes first. This requirement must be
  answered before that shape is fixed, not after, and the trigger is deliberately the schema
  rather than the pricing work, because the pricing work is where it is too late.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0020 — Covenant eligibility is a tenancy attribute

## Status

**DEFERRED** as of 2026-08-18. `platform-tenancy` does not exist, no tenant record has a shape,
and there is no price for anything to be zero.

Deferred, not dormant. The trigger is the migration that creates the tenant table, and the reason
it is that early is the whole content of this requirement.

## Why this is written now

C-0009 obligation III grants free capacity to covenant beneficiaries permanently. Nothing runs, so
nothing can implement it. The question is why it is written today rather than when there is a
billing system to exempt someone from.

**The same argument C-0006 makes about isolation.** *Provable isolation at two levels* is
non-commodity because it cannot be retrofitted — a runtime that did not have nested tenancy in its
first migration does not acquire it later without rebuilding. A permanent zero-price tenant class
has the same property, one level down.

Arriving late, this becomes a discount: a `discount_percent` column, a coupon record, a flag on a
subscription. Each of those is a **revocable pricing artifact**, editable by anyone with database
access or an admin console, invisible in the tenancy model, and indistinguishable from a
promotion that expires. The obligation it implements is constitutional and permanent. **Encoding a
permanent constitutional obligation as a mutable pricing field is a category error**, and it is
the specific mechanism by which C-0009 would be honoured on paper and dropped in practice — with
nobody deciding anything, which is C-0009's own account of how this fails.

ADR-0019 puts the general form of this in one line: decomposition precedes implementation. This is
that rule applied to a commitment rather than to a feature.

## The requirement

1. **Covenant eligibility is an attribute in the tenancy model**, carried on the tenant record
   itself, at the level ADR-0024's two nested silos already establish. Not a pricing field, not a
   subscription flag, not a coupon.

2. **It is set by verification against state-issued documents and never by self-declaration.**
   GLO-0010 names them: the ex-servicemen identity card, the Zila Sainik Board or Rajya Sainik
   Board record, the Pension Payment Order, the service pension record, the discharge book.
   Paridyum verifies against what the Indian state already issues and builds no parallel registry.

3. **It records the tier.** GLO-0010's tiers are a priority order that binds under scarcity,
   and an attribute that collapses them to a boolean cannot express the rule it exists to serve.

4. **It is removable only by the same path that set it** — an audited change with an actor, under
   C-0003 — and never by a routine pricing or plan operation. A plan change, a migration to a new
   price book, or a billing-system replacement must not be capable of silently clearing it.

5. **It grants price and nothing else.** Isolation, quota class, export path (REQ-0013), support
   path and data handling are identical to a paying tenant's. C-0008's rule that no party is a
   privileged shape cuts both ways: a covenant tenant is neither advantaged nor degraded in any
   dimension except what is charged.

6. **The verification evidence is regulated-adjacent and is treated as such.** A PPO, a discharge
   book and a disability record identify a person, their service history and in tier 2 their
   medical status. ANTI-0007 forbids regulated data before controls exist. **This requirement does
   not authorise storing those documents**; the design that satisfies it must decide between
   verifying-and-discarding, verifying through a third party, or waiting for controls — and that
   decision belongs in the ADR written when the trigger fires.

## Open questions for the ADR that closes this

Named rather than answered, because answering them today would be inference.

- **Where the attribute sits under nested tenancy.** A beneficiary may be a `tenant` running a
  business on Paridyum, or an `account` inside a tenant's product (GLO-0004). Obligation III speaks
  of a business *owned or operated by* a beneficiary, which is the tenant level — but the account
  case will arise the first time a franchisee has a veteran customer, and the answer there is not
  obvious. It is not Paridyum's obligation to make a franchisee's pricing free, and it may be
  Paridyum's obligation not to charge the franchisee for that account.
- **Re-verification cadence, if any.** Eligibility under GLO-0010 does not expire. Documents and
  their holders' circumstances do.
- **Whether obligation III is metered.** C-0009 accepts an uncapped cost and writes no bound. Even
  unbilled consumption should be *measured*, or the accepted cost is unknowable and the decision to
  bound it later cannot be made on evidence.
- **Whether tier is visible to the tenant.** A person told by software which tier of loss they fall
  into is a design problem, not only a schema one.

## Related

- C-0009 — obligation III, the constitutional source of this requirement
- GLO-0010 — the tiers, and the documents verification rests on
- ADR-0024 — nested tenancy, the model this attribute lives in
- C-0008 — no party on the runtime is a privileged shape
- ANTI-0007 — regulated data before controls; the bound on requirement 6
- REQ-0013 — the export path a covenant tenant gets identically
- C-0003 — the audited actor requirement 4 depends on
- ADR-0019 — decomposition precedes implementation, which is why this is written now
