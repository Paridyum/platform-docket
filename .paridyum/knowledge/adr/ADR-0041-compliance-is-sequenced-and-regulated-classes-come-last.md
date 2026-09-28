---
id: ADR-0041
title: Compliance is sequenced — process frameworks first, regulated data classes last and gated
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [compliance, soc2, gdpr, dpdp, hipaa, pci, audit, residency, anti-0007]
breaking: false
enforced_by:
  - check: data.regulated_class_requires_control_evidence
    severity: error
    rationale: >
      Already implemented in `check-migrations.sh` and already the mechanism this ADR relies
      on: a migration declaring `phi`, `pci` or `financial-record` fails unless a control
      framework is declared in `manifest.yaml`. This decision does not add a gate — it states
      the sequence that gate was always going to enforce, before someone meets it by surprise
      in a pull request.
  - check: db.timestamps_are_timestamptz
    severity: error
    rationale: >
      An audit record whose instant nobody recorded cannot evidence when access occurred, which
      is the question every one of these frameworks asks first.
  - check: db.region_is_chosen_at_provisioning_and_immutable
    severity: error
    rationale: >
      Decision 3 calls region contractual and immutable. This is that decision made mechanical: a
      DEFAULTed region places a tenant in a jurisdiction nobody chose, which decision 3 names as a
      residency breach every existing check reports as green.
  - check: domain.a_named_rung_is_an_operable_rung
    severity: error
    rationale: >
      Decision 3's companion for containment. A rung named in an attestation that nothing can
      actually operate is a claim, and decision 4 forbids claiming what is not true.
---

# ADR-0041 — Compliance is sequenced

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0117.** D3's rule — a region chosen explicitly and never
> changed except by migration and contract amendment — **extends to an org's region** (ADR-0117 D1).
> A tenant's region and its owner org's region are chosen independently.

## Status

**ACCEPTED** as of 2026-08-08. Created `accepted` per `new-decision` §5b.

## Context

Asked which compliance regimes Paridyum targets, the answer was *"every one of them."* As an
ambition that is coherent — a multi-tenant identity and application platform sells into
procurement, and procurement asks. As a plan it is not, for a reason already written down and
locked.

**ANTI-0007 — "regulated data before controls" — is locked**, and `CLAUDE.md` singles it out as
*"the one anti-pattern that can end the company in a single incident."* It is already enforced:
`check-migrations.sh` hard-fails any migration declaring `paridyum:class phi`, `pci` or
`financial-record` unless `manifest.yaml` declares a control framework. So the regulated classes
are not a target anyone sets — they are a gate, and it is already closed.

The frameworks also differ in kind, and treating them as one list hides it:

- **SOC 2, ISO 27001** certify *how you work*. Evidence is process: change control, access
  review, monitoring, incident response.
- **GDPR, DPDP** regulate *what you do with personal data*. Evidence is design: lawful basis,
  residency, deletion, portability.
- **HIPAA, PCI-DSS** regulate *specific data classes*. They impose scope boundaries, contractual
  instruments (BAAs), and segmentation obligations that reshape architecture.

The first two groups are largely satisfied by disciplines this repository already has. The third
is a step change, and taking it early means carrying its cost before any customer pays for it.

## Decision

**1. Two phases, and the boundary is ANTI-0007's gate.**

| Phase | Frameworks | Starts |
| --- | --- | --- |
| **Now** | SOC 2 Type II, GDPR, DPDP | design inputs from today |
| **Gated** | HIPAA (PHI), PCI-DSS | only when a customer contract requires it |

**2. No `phi`, `pci` or `financial-record` column exists until its control framework is declared
and real.** ANTI-0007's rule, restated so the sequence is explicit rather than emergent. The gate
already refuses; this says that refusal is intended, not an obstacle to route around.

**3. Five things become design inputs immediately**, because each is far cheaper now than
retrofitted:

- **Audit logging is a first-class output, not a feature.** Append-only, retained to a stated
  period, exportable per tenant. It spans Zitadel's event log *and* our own records, so a shared
  correlation id must exist from the first request — retrofitting one across two systems means
  reconstructing history that was never written.
- **`tenant.region` is contractual and immutable.** Multi-region is residency-driven, not a
  latency optimisation. A tenant does not move regions without a migration and a contract
  amendment, and cross-region access by staff is a control that must be evidenced.
- **Key management has a policy from the first deployment.** The Zitadel masterkey is immutable
  once initialised, per region — a compliance artifact, not an operational note.
- **Every grant is attributable and revocable.** GLO-0005's *"a grant, never an absence"* is
  exactly what an access review asks for; operator scope must record who granted it, to whom,
  when, and how it ends.
- **Personal data is classified at the migration that creates the column**, per REQ-0003, so a
  deletion or portability request is a query rather than an investigation.

**4. Certification is not claimed before it is held.** No artifact, README or capability matrix
states a framework as satisfied until an audit says so. ADR-0028 already fixes the direction —
controls map *outward* to frameworks, and a mapping *"asserts contribution to an outcome, never
that the outcome is met."*

## Consequences

**Accepted costs**

1. **Saying no to a regulated deal.** Phase 2 gated behind a paying contract means the first
   healthcare or card-present customer is either declined or delayed while controls are built.
   That is a revenue cost, taken deliberately, and it is the whole point of ANTI-0007.

2. **Audit logging is expensive before it is useful.** Append-only storage, retention and
   per-tenant export are real engineering, built for a customer who has not asked yet. The
   alternative is reconstructing history that was never recorded, which is not an alternative.

3. **Immutable regions constrain operations.** No rebalancing a tenant to a quieter region, no
   consolidating a sparse one. Capacity planning becomes per-region and permanent.

4. **The sequence will be contested by sales.** "We're SOC 2 *and* HIPAA" is easier to sell than
   a phased answer, and the pressure to declare phase 2 early arrives from outside engineering,
   where ANTI-0007's argument is least visible.

5. **This ADR adds no new machinery**, which is a cost as much as a virtue: phase 1 is enforced
   entirely by compliance with prose, since SOC 2 readiness has no check. Only the phase boundary
   is mechanical.

**Gained**

- The most expensive obligations arrive attached to revenue rather than to optimism.
- Phase 1 is nearly free here, and that is not luck: locked decisions, checks named by the rules
  they enforce, a suite that plants defects to prove gates work, and an anti-forgetting ledger
  are most of what a SOC 2 auditor asks to see. Very few pre-revenue companies have any of it.
- The five design inputs land while there is no data to migrate.

**Rejected alternatives**

- **Pursue everything simultaneously.** The stated ambition. It front-loads the costliest
  obligations, and ANTI-0007's gate blocks it anyway — so the practical outcome is either a
  stalled migration or pressure to weaken a locked anti-pattern.

- **Defer all compliance until a customer asks.** Cheapest today. Audit trails cannot be
  backfilled, residency cannot be retrofitted onto placed data, and correlation ids cannot be
  added to requests that already happened. The five inputs in decision 3 are exactly the items
  that are impossible later, which is why they are not deferred with the rest.

- **Pick one framework and ignore the others.** SOC 2 alone omits GDPR and DPDP, which are law
  rather than assurance, and apply the moment a European or Indian person's data is processed —
  which multi-region deployment makes near-certain.

## Related

- ANTI-0007 — locked; the gate this sequences behind, and the only one that can end the company
- ADR-0028 — controls map outward; never claim a framework as met
- REQ-0003 — privacy and personal data classification, which decision 3 depends on
- REQ-0012 — vendor governance, `accepted`; Zitadel and the cloud provider are in scope
- REQ-0015 — threat modelling, `accepted`; compliance turns its output into a dated deliverable
- REQ-0013 — tenant data portability, which GDPR and DPDP make concrete
- GLO-0005 — operator scope as a positive grant, which is what an access review inspects
- RFC-0012 — the identity topology whose regional and audit properties this constrains
- STD-0006 — where a column's data classification is declared
