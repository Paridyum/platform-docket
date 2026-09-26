---
id: ADR-0087
title: Governance authority is personal and ordered; signing authority belongs to an office
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [org, tool]
applies_to: ["tool-docket"]
tags: [governance, officers, removal, succession, c-0004, rfc-0042, adr-0077, tool-docket]
breaking: false
enforced_by: []
---

# ADR-0087 — Governance authority is personal; signing authority is an office

## Status

**Accepted**, 2026-09-13. `enforced_by` is empty and that is honest: nothing docket can check
knows who authorised a change to `signatories.yaml`, because that file is hand-edited and the
authorisation happens in a conversation. What is checkable — that a ceased officer stops signing,
and that every officer has a possible countersignatory — is enforced and is not this decision.

## Context

RFC-0042 decided how an officer is **appointed**: a dated instrument, a registry entry citing it,
and a human approving the change under C-0004. It decided nothing about **removal** beyond the
shape of the record — `active: false` plus `ceased_on`, never deletion.

The asymmetry is visible the moment the question is asked. An appointment that reaches the
register without an instrument is a `docket check` finding. **A removal that reaches it without one
is indistinguishable from a correct removal.** The operator asked how firing a CEO is decided, and
the honest answer was that nothing decided it.

Two facts settled it, both stated by the operator on 2026-09-13: **Rohan Batra holds absolute
authority and Priyanshu Sharma holds it second**, and **Paridyum is the main brand**, with
Priyanshu continuing to act for the owner in Paridyum Cyber.

## Decision

### 1. Governance authority is personal and ordered: Rohan, then Priyanshu

Appointing an officer, removing one, and changing what a company is are authorised by **Rohan
Batra**; where he cannot act, by **Priyanshu Sharma**. This is a person-level authority and it does
not move when an office does.

### 2. Governance authority is not signing authority, and conflating them would be wrong

**Signing authority belongs to an office.** `registry/signatories.yaml` says who executes a
document on behalf of a company, resolved by office and fallback order (ADR-0077 D4). Rohan is
`cto` there — third in Paridyum's order — and that is correct and unchanged by decision 1.

The distinction is load-bearing rather than pedantic. A document is executed by whoever holds the
office it is routed to, because that is what makes it verifiable years later: the register can say
who held an office on a date. **Who may decide that somebody stops holding an office is a
different question**, asked far less often, and answering it by reordering the signatory list
would have made Rohan sign everything — which is not what was decided and would have changed
every document Paridyum issues.

### 3. A removal owes what an appointment owes

RFC-0042 §5's three requirements extend to removal: a dated instrument, a registry entry citing
it (`active: false` plus `ceased_on`), and a human approving the change under C-0004 — who, by
decision 1, is Rohan or Priyanshu.

**`ceased_on` is what makes this answerable rather than merely recorded.** A register that cannot
say *when* an officer ceased cannot say whether they held office on the date a given document was
issued, which is the only question a removal makes anyone ask later.

### 4. Paridyum is the holding company and Priyanshu acts for it in Cyber

Unchanged from ADR-0077 D4 and reaffirmed: Paridyum Cyber's documents are executed by Priyanshu as
Paridyum's Director, signing for the owner. "For now" is the operator's word and is recorded as
such — this is who acts, not a permanent structure.

**No second holding-company signatory is added**, because the question that prompted it was based
on a claim that turned out to be false. DECISION-DEBT D-59 asserted that removing Priyanshu would
leave nobody to countersign Ruhani's documents. Measured, the chain survives: her documents fall
to Rohan as `cyber-cto` and his fall to Ruhani as `cyber-ceo`. Cyber becomes unable to issue only
if two of its three signatories go at once.

## Consequences

**Accepted costs**

- **This is an operator decision, not a legal opinion**, and the distinction matters more here
  than elsewhere in this repository. After incorporation, removing a director is governed by the
  Companies Act and the articles regardless of what any register here says; an internal ordering
  cannot displace a statutory procedure. RFC-0042 §3 takes the same posture and this decision
  claims no more than that one does.
- **Nothing enforces decision 1.** `signatories.yaml` is hand-edited, and docket cannot know who
  approved an edit. The control is C-0004's — a human, deliberately — and the evidence is the
  instrument, not a field.
- **Two authorities deep is thin.** If neither Rohan nor Priyanshu can act, nothing here says what
  happens. That is acceptable while the estate is four people and is the first thing to revisit
  when it is not.

**Gained**

- The question "how is a CEO fired" has an answer that is written down, which it did not on
  2026-09-12.
- Signing authority stays with offices, so every document issued to date remains explicable by the
  register that issued it.
- The removal path is now symmetrical with the appointment path, which is what makes an
  unauthorised removal detectable at all: a missing instrument.
