---
id: ANTI-0007
title: Accepting regulated data before the controls exist
type: anti-pattern
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [compliance, phi, pci, hipaa, regulated-data, risk, judgement]
breaking: false
enforced_by:
  - check: data.regulated_class_requires_control_evidence
    severity: error
    rationale: >
      Whether a product is "regulated" is a judgement, but whether a migration created a
      column declared to hold protected health information is not — and that declaration is
      where the liability attaches.
---

# ANTI-0007 — Accepting regulated data before the controls exist

## Status

**ACCEPTED**, not `locked`, for ANTI-0004's reason: the central judgement cannot be
mechanically made. Whether an application "handles protected health information" depends on
what a tenant types into a free-text field, and no static analysis sees that.

`data.regulated_class_requires_control_evidence` catches the one decidable part — a migration
that *declares* a column as regulated while no control framework exists. That is a proxy for
the rule, not the rule, and it is worth having because the declaration is the moment liability
attaches. It is also worth nothing until REQ-0003 is current, since REQ-0003 is what makes
classification mandatory in the first place; this is recorded rather than papered over.

This is the only anti-pattern in this repository whose violation can end the company in a
single incident.

## The shape

```
A prospective customer asks for a clinic management template.
They will pay more than everyone else combined.
The platform already has tenants, auth, audit logs and encrypted storage.

  -> "we are basically compliant already"
  -> ship it behind a disclaimer in the terms of service
  -> patient records land in the shared database
```

Every step is individually reasonable. The customer is real, the revenue is real, the existing
controls are genuinely good ones, and the disclaimer was written by someone who read the
regulation. The template ships in a fortnight and works.

Variants, all the same shape: payment card numbers "just for one integration"; regulated
financial records because the customer is a broker; biometric identifiers because the login
flow supports them and someone enabled it.

## Why it is forbidden

**The controls are not the checklist. The controls are the organisation, and there is not one
yet.**

- **Encryption and audit logs are the visible tenth of the obligation.** The rest is breach
  notification within a fixed window, business associate agreements, workforce training,
  designated responsibility, vendor attestations for every subprocessor, documented risk
  assessment, and evidence that all of it has been operating over a period. None of it is
  code, and none of it can be produced retroactively for a period that has already elapsed.

- **ADR-0024 makes shared-database the default at two nested levels, and shared is not a
  defensible home for this data at either.** That decision is correct for the business BET-0001 describes, and its costs are
  stated honestly: the failure mode is silent and unrecoverable. A silent, unrecoverable
  cross-tenant leak of ordinary business records is a serious incident. The same leak of
  patient records is a reportable breach affecting people who never chose to trust us, and
  they cannot be made whole.

- **One incident is terminal at this stage.** A company with no revenue and one breach of
  health records has statutory penalties, mandatory public notification, and no balance sheet.
  There is no version of this that is survivable-but-painful. This is the asymmetry that makes
  it an anti-pattern rather than a trade-off: the upside is one contract, and the downside is
  every contract.

- **The disclaimer does not transfer the obligation.** Terms of service allocate liability
  between us and the tenant. They do not bind the regulator, and they do not bind the patient.
  A clause saying *"do not store protected health information"* on a product built to store
  clinical records is evidence of knowledge, not a defence.

- **It cannot be undone.** REQ-0003 makes this point in its original form: personal data
  written a year ago is in a year of backups. Regulated data accepted before controls existed
  remains accepted before controls existed, permanently, and that fact is discoverable.

- **It arrives disguised as validation.** This customer pays more and asks for more, so the
  request reads as the market pulling us upmarket. It is the highest-risk revenue available,
  and it will feel like the strongest signal we have had.

## Nested tenancy makes this worse, not merely bigger

ADR-0024 added an inner isolation boundary, and regulated data crossing it is a different kind of
incident from regulated data crossing the outer one.

At the outer boundary, one business reads another's records — serious, reportable, and the injured
parties are businesses. At the **inner** boundary, one of an operator's customers reads another's,
and the exposed records belong to people who never chose to trust Paridyum, never chose to trust
the operator's vendor, and have no relationship with us at all. The operator promised them
isolation. We would have broken that promise on the operator's behalf, and there is no version of
that conversation where we are the injured party.

GLO-0005's rule is the specific hazard: operator scope is a session that reads every account of a
tenant. If it is ever represented by an absent `account_id` rather than a positive grant, then any
bug that loses a value turns an ordinary end user into a reader of every patient record in the
tenant, with no error anywhere. A permissive default and protected health information are the two
ingredients of an unrecoverable breach, and ADR-0024 exists partly to keep them apart.

## What to do instead

**Decline the data, not the customer, and say which is which.**

1. **Refuse the regulated field, and be specific about the boundary.** A clinic needs
   appointments, staff rotas, invoices and inventory. Those are ordinary business records. The
   clinical notes are not. A product that manages the operations of a medical practice without
   storing diagnoses is a real product, and it is the one that can be built today.

2. **If the regulated data is the point, decline and record it.** Write down who asked, what
   they needed, and what it would have taken. That record is the input to the decision to enter
   the market deliberately, later, and it stops the same question being re-litigated from
   scratch every quarter by whoever has not heard it before.

3. **When entering deliberately, controls precede data — in that order, with evidence.** The
   framework selected and documented, the subprocessor attestations collected, responsibility
   assigned to a named person, isolation escalated per ADR-0024 decision 8, and an audit period
   elapsed. REQ-0003 and REQ-0012 both have to be current, not deferred. This is a
   company-shaped project, not a sprint.

4. **Isolation is necessary and not sufficient.** A dedicated database for the regulated tenant
   is a prerequisite, and having it does not make the rest optional. The reverse order — hard
   isolation as evidence of compliance — is this anti-pattern with better infrastructure.

## How to spot it in review

1. Does a migration classify a column as `phi`, `pci` or a regulated financial record? → the
   declaration itself is the finding.
2. Is a template named for a regulated profession — clinic, pharmacy, laboratory, brokerage? →
   the operational subset may be fine; establish which fields are actually being asked for
   before anything is built.
3. Does a proposal cite a specific regulation as a *feature* — "HIPAA-ready", "PCI-compliant" —
   with no named framework, no assigned owner and no audit period? → the claim is the risk, and
   making it in marketing is independently actionable.
4. Is the justification the size of one contract? → that is the shape, exactly.
5. Would a free-text field predictably collect regulated data whatever it is labelled? → a
   `notes` column on a patient record is a clinical record, and calling it `notes` changes
   nothing.

## Related

- ADR-0024 — the shared-database default this bounds; decision 8 is the escalation path
- REQ-0003 — data and privacy governance; must be current, not deferred, before entry
- REQ-0012 — vendor and compliance governance; subprocessor attestations
- C-0006 — anti-goal 3, and the only one whose violation is existential
- ANTI-0004 — the same status reasoning: a judgement with a proxy check
- BET-0001 — the segment this revenue would appear to pull us away from
