---
id: ADR-0028
title: A control maps outward to frameworks; a framework is never the source of a rule
type: adr
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, compliance, frameworks, ssdf, cis, csf, control-library]
breaking: false
enforced_by:
  - check: knowledge.framework_ids_resolve
    severity: error
    rationale: >
      A mapping's entire value is that it answers "what covers this obligation". An id
      that does not exist answers that question confidently and wrongly, and no reviewer
      has SSDF task ids or CIS safeguard numbers memorised — so review cannot be the
      control here. Registered framework, and an id matching that framework's own
      id_pattern, or the mapping does not land.
---

# ADR-0028 — A control maps outward to frameworks; a framework is never the source of a rule

## Status

**LOCKED** 2026-08-08.

## Context

`check-registry.yaml` already held 132 controls, each with a falsifiable spec, an owner, and
the artifacts it blocks. What it did not hold was any statement of how those controls relate
to the external frameworks anyone will eventually ask about — NIST SSDF, CIS Controls, NIST
CSF, OWASP ASVS, SLSA.

That question arrives from outside and it arrives suddenly: a customer security
questionnaire, an enterprise procurement review, a certification scope. The expensive way to
answer it is the common way — read a framework, write a document per framework, and discover
that the documents describe controls nobody implemented.

There are two failure modes available here and they pull in opposite directions.

**Writing policy to satisfy a framework.** A NIST policy, a CIS policy and a SOC 2 policy
become three documents that drift, and the drift is invisible because each is internally
complete. Worse, rules start existing because a framework named them rather than because
something here needs them, which is how a governance system becomes theatre — and this
repository's entire premise is that a rule with no enforcement is worse than an absent one.

**Refusing to map at all.** Then the questionnaire arrives and someone answers it from
memory, under time pressure, about 132 controls they did not write. That answer will be
wrong in the generous direction.

A third failure sits underneath both: a mapping is unusually easy to fabricate. `PW.4.1` and
`PW.4.2` look equally plausible on a line, a reviewer cannot tell them apart, and a wrong id
reads as coverage forever. This is the same shape as every defect this repository has found
so far — the secret scan reporting `postgres` as the secret, the identity gate examining an
empty commit range, three checks verifying less than their names implied. Each reported
success while enforcing nothing.

## Decision

**A Paridyum control is the source of truth. It maps outward to framework identifiers, and
never the reverse.**

1. **`framework-registry.yaml` is the id-space**, in the same relationship to mappings that
   `check-registry.yaml` has to `enforced_by` (ADR-0014) and `class-registry.yaml` has to
   repository classes (C-0005). Each entry carries an `id_pattern`; an id that does not match
   it is a hard failure. The pattern lives in the registry and is not restated in
   `check-registry.schema.json`, because two copies of a rule are a drift generator.

2. **`frameworks:` on a control is optional and expected to be sparse.** Of 132 controls, 74
   carry a mapping. The other 58 govern naming, layout, formatting and documentation, and
   map to nothing. Leaving them unmapped is the correct output, not incomplete work.

3. **Map only where the control demonstrably implements the named task.** Where a mapping is
   arguable, it is omitted. An absent mapping is recoverable by reading the spec; a wrong one
   is not, because nobody re-derives a mapping that already looks answered.

4. **A mapping asserts contribution to an outcome, never achievement of it.** Most framework
   outcomes need several controls plus operational evidence this repository does not collect
   (REQ-0012). **No compliance percentage may be computed from this data.** That number would
   be the most confidently wrong artifact in the system, and it would be quoted externally.

5. **A framework may be registered before anything maps to it, if it declares
   `mapping_deferred_to`** naming the requirement that will close the gap. `asvs` and `slsa`
   are registered this way today. Without that field, an unmapped framework warns — a
   registry whose only job is describing coverage must not itself overstate coverage.

6. **No rule is created because a framework names it.** A gap a framework reveals is
   evidence for an RFC, not an instruction. The RFC is judged on whether Paridyum needs the
   rule.

## Consequences

**Good.** A security questionnaire is answered by querying the registry rather than by
recollection. The gaps the mapping exposes are visible and specific — the concentration of
mappings under SSDF `PS.1.1` and CSF `PR.AA`, versus nothing at all under `RV.*`, is a
readable statement that this system governs how code is written and protected far better
than it governs what happens after a vulnerability is found. That is true, and REQ-0014 and
REQ-0015 now record it rather than leaving it to be discovered during an incident.

**Costly.** Every new control now invites a mapping decision, and the honest answer is
usually "none". Reviewers must be comfortable leaving the field off. The counter-pressure —
mapping everything so the file looks thorough — is precisely what item 3 forbids, and it will
recur.

**Deliberately excluded.** ISO 27001 and SOC 2 are not registered. Both are certification
regimes rather than control catalogues; mapping to them without an audit in scope produces
a document whose only reader is its author. REQ-0012 holds that trigger.

**Not a compliance claim.** This ADR creates a mapping, not an attestation. Paridyum is not
SSDF-conformant, CIS-aligned, or SLSA-anything, and nothing in this repository may be quoted
as saying otherwise.

## Alternatives considered

**A separate mapping file, keyed by framework id.** Reads better for "what covers CIS 16.12"
and is the shape most compliance tooling expects. Rejected: it puts the mapping somewhere the
control author is not looking, so it goes stale the moment a control changes. Co-locating it
with the spec means the two are edited in the same diff. The framework-first view is a query,
and a query can be generated; a stale file cannot be un-staled.

**Mapping every control, including the layout and naming ones.** Rejected under item 3. It
would raise coverage from 74 to 132 and lower the value of every mapping, because the reader
could no longer assume a present mapping was considered.

**CSF subcategory precision (`PR.AA-05`) rather than category (`PR.AA`).** Rejected for now:
subcategory ids are easy to misremember and the extra precision buys nothing until an
assessment is actually in scope. `framework-registry.yaml`'s `id_pattern` admits both, so
tightening later is an edit rather than a migration.

**Waiting until a customer asks.** Rejected. The mapping is cheap now, while the person who
wrote the controls still remembers why each exists, and expensive later, when it must be
reconstructed from specs under a deadline. This is the same reasoning ADR-0018 applies to
deferred requirements.
