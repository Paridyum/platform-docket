---
id: REQ-0015
title: A design that accepts untrusted input states what could go wrong, and the org keeps one list of accepted risk
type: requirement
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [threat-model, stride, risk-register, risk-acceptance, asvs, security-design]
breaking: false
trigger: >
  FIRED 2026-08-08. On "is designed", which this requirement deliberately wrote to fire
  early. RFC-0008 §1 designs Server Actions as publicly callable, unauthenticated-by-default HTTP
  endpoints and cites a CVSS 10.0 advisory; RFC-0009 §3 designs `platform-identity` and
  `platform-routing`. The LLM clause fired with REQ-0007 clause 3, on the same event.
  Non-obvious consequence: the model this produces is the second opinion RFC-0009 Q1 explicitly
  asks for, so this and that open question are one piece of work.
  ORIGINAL: The first component that accepts input from outside the trust boundary is designed —
  an HTTP handler, a webhook receiver, a file upload, a queue consumer fed by anything
  other than our own services, or an LLM prompt assembled from user-supplied text.
review_date: 2027-02-01
verified_by:
  - TST-0026
  - TST-0027
enforced_by: []
---

# REQ-0015 — A design that accepts untrusted input states what could go wrong, and the org keeps one list of accepted risk

## Status

**DEFERRED** as of 2026-08-08. There is no trust boundary to model. This repository contains
markdown, shell and YAML; it accepts input from one person on one machine. Threat-modelling
it would produce a document about a threat surface that does not exist, and the presence of
that document is itself a hazard — it is the shape of governance without the substance, and
it would be cited later as evidence that threat modelling happens here.

The risk register is deferred for the adjacent reason: a register listing risks nobody has
accepted, for systems nobody operates, decays into a wish list. Both halves are recorded
together because they are the same discipline at two altitudes — one per design, one per
organisation — and because they will trigger within weeks of each other.

## Requirement

**A design that crosses a trust boundary shall state what an attacker would try, what stops
them, and what is knowingly not stopped — and every knowingly-not-stopped item shall appear
in a single organisation-wide register with an owner and a review date.**

## Numbered items — threat modelling

1. **Trigger on the trust boundary, not on the ticket size.** ADR-0013 already computes
   significance from the diff rather than accepting a self-declaration; the same logic
   applies here, and for the same reason. "Is this worth threat-modelling?" asked by the
   person who wants to ship is not a control. The boundary is the trigger.

2. **STRIDE as the prompt, not as the deliverable.** Spoofing, Tampering, Repudiation,
   Information disclosure, Denial of service, Elevation of privilege — six questions that
   reliably surface what an unstructured discussion misses. The output is a short list of
   concrete attacks against *this* design, not a filled-in template. A template returned
   complete with generic entries is worse than nothing: it consumes the review slot that
   would otherwise have found something.

3. **Every identified threat gets one of three dispositions — mitigated, transferred, or
   accepted — and accepted goes to the register (§7).** A threat with no disposition is the
   default failure mode of threat modelling: the list exists, nobody disagreed with it, and
   nothing changed.

4. **Multi-tenancy is the standing threat for this platform.** Nested tenancy (GLO-0004)
   means cross-tenant access has two ways to fail, not one, and `db.no_tenant_id_from_client_input`
   plus `db.row_level_security_enabled_on_tenant_tables` are the mitigations that already
   exist. A tenant-facing design that does not consider a tenant as the attacker has not
   been threat-modelled.

5. **Agent-authored code is in scope as an attack surface, not merely as a quality
   question.** REQ-0007 governs what an agent may do; this governs what an attacker can make
   an agent do. Prompt injection reaching a tool call is elevation of privilege with a
   different vocabulary, and it is the threat most specific to what Paridyum is building.

6. **Adopt OWASP ASVS as the requirement catalogue for the first service.** ASVS states
   application-layer verification requirements at three levels; picking a level and mapping
   to it is cheaper and more complete than deriving controls from first principles per
   service. `framework-registry.yaml` registers `asvs` with `mapping_deferred_to: REQ-0015`
   for exactly this moment.

## Numbered items — risk register

7. **One register, org-wide, in this repository.** Not per-service, not in a spreadsheet, not
   in an issue tracker. Risk accepted in three places is risk nobody can total.

8. **Each entry states: what could happen, roughly how bad, what is being done, who owns it,
   and when it is next looked at.** Deliberately not a 5×5 heat map with computed scores.
   A likelihood number invented to populate a matrix is precision with no accuracy, and the
   matrix then gets treated as measurement. Prose plus an owner plus a date beats a colour.

9. **An accepted risk expires.** Acceptance is a decision made under conditions, and the
   conditions change. On expiry it is re-accepted with a fresh date or it is fixed —
   silence is not renewal. This is the same mechanic ADR-0017 already applies to scan
   exceptions and REQ-0006 §1 applies to waivers, and it should be the same machinery
   rather than a third implementation of expiry.

10. **The register is where deferred requirements go to be counted.** Thirteen requirements
    are currently deferred, each with a trigger. That is a well-organised list of known
    exposure, and at present nothing sums it or asks whether the total is acceptable.
    `repo.decision_debt_ledger_present` and `knowledge.deferred_requirements_have_triggers`
    are the mechanics; the register is the reading.

11. **Entering the register requires naming who accepted it.** An organisation of one still
    needs this, because the value is not accountability today — it is knowing, in two years,
    whether the person who accepted it knew something that is no longer true.

## Why this is deferred rather than started small

A threat model for a system with no attack surface, and a risk register with no operated
system, both produce artifacts that look like the real thing. This repository has repeatedly
found that the dangerous failure is not an absent control but a control that reports success
while enforcing nothing — a green check nobody watched fail. An empty-but-present threat
model is that failure in prose form, and it is harder to detect because nothing runs it.

The trigger is written to fire early — at *design* of the first boundary-crossing component,
not at its deployment — so the model exists before the code it describes.

## Framework position

Related: NIST SSDF `PW.1.1` (use risk modelling to assess security risk in the design) and
`PW.2.1` (qualified review of the design); CIS v8.1 `16.14` (conduct threat modeling) and
`16.10` (apply secure design principles); NIST CSF 2.0 `ID.RA` (risk assessment), `GV.RM`
(risk management strategy) and `GV.OC` (organizational context); OWASP ASVS as the
requirement catalogue named in §6.

`pr.significant_change_references_design` and `agent.service_creation_escalates` already map
to `PW.1.1`/`PW.2.1` — they force a design artifact to exist and be reviewed. What they do
not do is require that artifact to contain a threat analysis. That is the gap this closes,
and it is why the mapping on those two controls is honest as it stands: they contribute to
the outcome without achieving it.

## Partially discharged

**ADR-0045** (2026-08-08) is a threat model for the identity platform — assets, nine threats, and
four accepted risks. It covers **one subsystem**, deliberately: the catalogue products, the
generated layer and the supply chain have no design to model yet. This requirement stays
`accepted` until they do.

## Related

- [REQ-0003](REQ-0003-data-and-privacy-governance.md) — what data exists determines what an
  attacker wants; the two documents are read together or neither is useful.
- [REQ-0007](REQ-0007-agent-authority-governance.md) — §4 on prompt injection and egress.
  This requirement treats that as a threat to model rather than a rule to follow.
- [REQ-0006](REQ-0006-governance-of-the-governance.md) — §1 waivers with expiry. The risk
  register should reuse that expiry machinery, not invent a parallel one.
- [REQ-0012](REQ-0012-vendor-and-compliance-governance.md) — vendor concentration is already
  named there as a risk with no register to hold it. This is that register.
- [ADR-0017](../adr/ADR-0017-scan-exceptions-are-reviewable.md) — the working precedent for
  an exception that lapses rather than persists.
- [ADR-0024](../adr/ADR-0024-nested-tenancy-is-two-columns-in-one-database.md) and
  [GLO-0004](../glossary/GLO-0004-nested-tenancy.md) — the standing threat in §4.
