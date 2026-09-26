---
id: ADR-0105
title: A second failure domain is declined for now, and every attestation states the honest RPO
type: adr
status: accepted
created: 2026-09-24
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [failure-domain, hypervisor, rpo, attestation, backup, sitar, adr-0064, adr-0094, rfc-0045, rfc-0046, rfc-0047, t-g.0b, t-g.7]
breaking: false
enforced_by:
  - check: docs.attestation_rpo_matches_failure_domain
    severity: error
    rationale: >
      An attestation (RFC-0045 §Proposal E) that states an RPO without naming that the backup and
      the primary share a hypervisor is the specific claim ADR-0041 decision 4 forbids — a
      capability stated as better than it is. The check exists so a future second region does not
      get folded into the same number by omission.
---

# ADR-0105 — A second failure domain is declined for now, and every attestation states the honest RPO

## Status

**ACCEPTED** as of 2026-09-24 by the operator. Answers **T-G.0b**: *"a decision exists on whether a
second hypervisor is bought, rented, or declined — and if declined, the attestation's RPO field
states the worse number honestly."* This is the decline.

## Context

ADR-0064 already named the constraint plainly: `sitar` is the failure domain, and *"nothing
arranged inside the cluster changes what happens when that machine stops."* That ADR decided
Postgres runs as one instance on it, deliberately, because two replicas on the same hypervisor is
*"theatre"* — no genuine availability gain, real storage cost.

T-G.0b asks the question ADR-0064 left adjacent to but did not answer: is a **second** hypervisor —
a second physical or rented machine, giving the estate an actual second failure domain — bought,
rented, or declined. RFC-0045 slice 7 (containment class C2/C3), all of RFC-0046 (BYOK), and every
stamp RFC-0047 designs are undeliverable without one; T-G.7's off-host backup (ADR-0094) narrows
the blast radius of losing `sitar` but does not create a second place to run from.

## Decision

**1. Declined, for now.** No second hypervisor is bought or rented at this stage. The estate
remains one failure domain, per ADR-0064, and this ADR extends that same reasoning one level up:
buying a second machine before there is a tenant whose contract requires it would be building
capacity against a guess, the same shape of cost RFC-0025 rejected for cluster-per-customer.

**2. The trigger is a funded one, not a calendar one.** A second failure domain is bought or rented
the day a specific commercial need requires it — a C2/C3 containment sale (ADR-0103 decision 3: a
rung is named only once it is built and rehearsed), or a franchisee whose contract states a
recovery objective this shape cannot meet. Until then, evaluating "should we buy a second machine"
on a schedule produces the same drift T-2.12 found in the Rust pin: a decision revisited from
scratch each time rather than answered once with a named trigger.

**3. Every attestation (RFC-0045 §Proposal E) states the real number, not an aspirational one.**
Concretely: RPO and RTO fields report against `sitar` as the sole failure domain — the off-host
backup (T-G.7, once it lands) bounds data loss to the backup interval, but recovery still means
standing up a new primary from that backup, on infrastructure that does not yet exist as a warm
target. **No field may read as though a warm or hot second site exists.** ADR-0041 decision 4 —
no capability claimed before it is real — governs, and this is the field it governs most directly:
an isolation attestation is exactly the document a prospective tenant reads to decide whether to
trust the platform with their recovery objective.

**4. This does not weaken T-G.7.** Off-host backup with a rehearsed restore is still owed and still
⛔ — it is what keeps a single-failure-domain estate from losing its backups in the same event as
its primary. This ADR says only that "off-host" does not mean "a second live failure domain";
those are different guarantees and only the second is declined here.

## Consequences

**Accepted costs**

1. **RFC-0045 slice 7, RFC-0046 (BYOK) and RFC-0047 in full stay undeliverable.** Known and already
   recorded as each RFC's own blocker; this ADR is what makes the block a decision rather than an
   unanswered question sitting in front of it.
2. **An honest RPO is a worse number than a competitor with two regions can quote.** Accepted under
   C-0006 anti-goal 5's position: a customer who needs that guarantee today is the right customer
   to lose for now, rather than the right one to lie to.

**Gained**

- T-G.0b is closed rather than sitting open in front of Wave 0.
- The attestation mechanism (T-G.18) has an unambiguous instruction for its RPO/RTO fields before
  it is built, rather than a judgement call made silently inside the implementation.

**Rejected alternatives**

- **Rent a second machine now, ahead of any funded need.** Rejected — no tenant yet, and RFC-0025's
  argument against building capacity ahead of a customer applies identically one layer up.
- **State a target RPO/RTO as if the second site existed, since it's "coming."** Rejected outright
  by ADR-0041 decision 4; this is precisely the claim it forbids.

## Related

- **ADR-0064** — one instance is the honest shape on one hypervisor; this ADR is the same argument
  one layer up, for the estate rather than for Postgres alone
- **ADR-0094** — the off-host backup endpoint; bounds data loss, does not supply a second failure
  domain
- **ADR-0041** — decision 4, no capability claimed before it is real, governing decision 3 here
- **ADR-0103** — decision 3, a named rung is an operable rung; the trigger this ADR's decision 2
  gates against
- **RFC-0045, RFC-0046, RFC-0047** — each blocked by this decline, named in decision 1
- **T-G.0b** — the Genesis task this closes; **T-G.7** — the off-host backup this does not weaken;
  **T-G.18** — the attestation this constrains before it is built
