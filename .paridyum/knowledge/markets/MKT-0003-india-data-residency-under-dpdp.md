---
id: MKT-0003
title: India data residency under DPDP, as of August 2026
type: market
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [dpdp, india, residency, regulation, sovereign-cloud, rfc-0014, anti-0007]
breaking: false
enforced_by: []
sources:
  - url: https://en.wikipedia.org/wiki/Digital_Personal_Data_Protection_Rules,_2025
    title: Digital Personal Data Protection Rules, 2025 (India)
    retrieved: 2026-08-17
    supports: >
      The DPDP Rules were notified by the Ministry of Electronics and Information Technology in
      November 2025, operationalising the bulk of the Act's delegated provisions. SECONDARY
      SOURCE — see the warning below; the Gazette notification is the authority.
  - url: https://www.mondaq.com/india/privacy-protection/1828534/cross-border-data-transfers-under-indias-dpdp-framework-navigating-the-new-compliance-landscape
    title: Cross-border data transfers under India's DPDP framework
    retrieved: 2026-08-17
    supports: >
      No blanket localisation requirement. True localisation applies only to Significant Data
      Fiduciaries and only to categories of personal data the government specifically notifies;
      as of mid-2026 neither the restricted categories nor the full SDF list has been notified.
      This is the source that BOUNDS the claim.
  - url: https://www.civo.com/blog/definitive-guide-to-choosing-an-indian-sovereign-cloud
    title: Civo — the definitive guide to choosing an Indian sovereign cloud
    retrieved: 2026-08-17
    supports: >
      Sovereign-cloud positioning aimed at Indian SaaS and AI workloads is an active vendor
      category in 2026. Evidence of competition, not of a legal requirement.
  - url: https://acecloud.ai/blog/dpdp-rules-for-cloud-service-providers/
    title: DPDP Rules for cloud service providers — compliance checklist
    retrieved: 2026-08-17
    supports: >
      Implementation is phased, making 2026 a build-and-test year for contracts, evidence
      workflows and incident drills rather than a hard compliance deadline.
---

# MKT-0003 — India data residency under DPDP

## What this is

**A dated reading of a regulatory position**, gathered 2026-08-17. It exists because RFC-0014
makes a region a legal boundary and MKT-0001 finds that residency is the only thing owning metal
measurably buys — so what the law actually requires decides whether that purchase has a reason.

## ⚠️ The most important finding is a negative one

**There is no blanket data-localisation requirement in India.**

True localisation binds only **Significant Data Fiduciaries**, and only over **categories of
personal data the government specifically notifies**. As of mid-2026:

- the restricted data categories have **not** been notified, and
- the full list of designated SDFs has **not** been published.

**Therefore: any claim that Indian hosting is "required by law" is false today.** Residency may be
sold as a property a customer may want. It may not be sold as a compliance obligation, and a
marketing page saying otherwise would be a misrepresentation rather than an overstatement.

This bound is recorded first, before the opportunity, because the opportunity is the part that
will be remembered.

## What is established

| | |
| --- | --- |
| DPDP Rules notified | November 2025, by MeitY — operationalising the bulk of the Act's delegated provisions |
| Localisation scope | SDFs only, over categories not yet notified |
| Cross-border default | Permitted, with transfer-specific contractual mechanisms |
| 2026 in practice | A phased build-and-test year for contracts, evidence workflows and incident drills |
| Vendor activity | Indian sovereign-cloud positioning aimed at SaaS and AI workloads is an active category |

## What follows, and what does not

**Follows.** Residency is a real differentiator that a European provider structurally cannot
match, and there is observable competition for it — so the demand is not hypothetical. It is a
*preference* market today, and would become a *requirement* market if and when categories and SDFs
are notified.

**Does not follow.** That anyone must buy it now. That Paridyum would be early — the sovereign-cloud
category already has entrants. That a residency claim needs owned metal rather than rented Indian
metal; MKT-0001 row B2 satisfies residency at ₹94/GB with no capital at all.

## Source quality, stated plainly

**The notification date rests on a secondary source.** RFC-0020 requires facts cited and dated, and
this one is cited to an encyclopaedia article rather than the Gazette. That is adequate for
planning and **inadequate for anything customer-facing or contractual**. Before a residency claim
appears in marketing, terms, or a sales conversation, replace that citation with the primary
notification and supersede this artifact.

The bounding source — that localisation is SDF-only over unnotified categories — is a law-firm
commentary. Better than the first, still not primary.

## Related

- **MKT-0001** — the cost measurement whose only surviving advantage for owned metal is residency
- **MKT-0002** — the operator evidence; sovereign positioning appears there as competition
- **RFC-0014** — a region is a residency boundary, and adding one is a legal act
- **RFC-0015** — a fact has one home, and nothing crosses a region boundary
- **BET-0009** — cost 3 carries this artifact's warning as an accepted cost
- **ANTI-0007** — regulated data before controls; the anti-goal a hosting customer can violate
  without asking
