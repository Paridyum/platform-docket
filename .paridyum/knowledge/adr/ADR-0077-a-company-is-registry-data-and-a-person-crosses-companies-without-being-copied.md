---
id: ADR-0077
title: A company is registry data, and a person crosses companies without being copied
type: adr
status: accepted
created: 2026-09-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, tool]
applies_to: ["tool-docket"]
tags: [multi-entity, identity, tenancy, signatories, adr-0024, adr-0051, adr-0067, rfc-0042, tool-docket]
breaking: true
enforced_by:
  - check: docket.identifiers_are_never_reused
    severity: error
    rationale: >
      A second company is the first real chance to allocate a person two identities by
      accident. Decision 2 forbids it; this check is what notices if it happens.
---

# ADR-0077 — A company is registry data

## Status

**Accepted**, 2026-09-09. `breaking: true` — the engagement identifier gains a company that was
previously implicit, and `registry/org.yaml` stops being a single object.

## Context

Paridyum is no longer the only entity. **Paridyum Cyber** is owned by Paridyum, and the operator
states that Priyanshu — Paridyum's Director — drives it on paper. The officers moved with it:

| | Paridyum | Paridyum Cyber |
| --- | --- | --- |
| CEO | Priyanshu Sharma (also Director) | Ruhani Khurana |
| CTO | Rohan Batra | Rohan Batra |
| COO | Aditya Shrivastava | — |

docket assumes one company in three places: `registry/org.yaml` is a single object read by every
document for its letterhead, address, jurisdiction and serial format; `registry/signatories.yaml`
is one set of officers; and `PAR/{kind}/{year}/{seq:04d}` hard-codes the entity into every serial.

**The identifier scheme, though, already anticipated this.** ADR-0051 labels the first segment of
`PAR-P-000147-3` as *organisation*, and states plainly that *"one person may hold many
engagements; the archive stays whole across all of them."* Nothing below supersedes it; the
decisions here are the reading of it that a second company forces.

**What does not work today is Rohan.** `personnel::reengage` refuses a second engagement while one
is open — *"separate them first"* — which is correct for a sequential change (an intern becomes an
employee) and wrong for a simultaneous one. Holding office in two companies at once is not
currently representable, and that is the substance of this change rather than the registries.

## Decision

### 1. A company is a registry entry, not a store

One docket, one register, one chain, many companies. `registry/org.yaml` becomes a map keyed by a
company code; each entry carries its own name, address, jurisdiction, legal status, CIN, POSH
routing, verification contact and serial format.

**Rejected: one store per company.** It is the obvious answer and it breaks the thing ADR-0051
exists to protect. Two stores means two ledgers, and a person in both gets two person identifiers
— so Rohan's archive splits down the middle, and the check that identifiers are never reused
cannot see across the boundary it would most need to. It also duplicates a registry that would
then drift, which is the defect `signatories.yaml` was created to end.

### 2. The person identifier is group-wide; the engagement identifier carries the company

`PAR-` on a **person** means the Paridyum group, not the legal entity. One person, one identifier,
for as long as they are known to any company in the group. Rohan is `PAR-P-000001-X` and is not
allocated a second identity by Paridyum Cyber.

The **engagement** identifier carries the company in the slot that already exists for it:

    PAR-OFC-2026-0001      an office held at Paridyum
    PCY-OFC-2026-0001      an office held at Paridyum Cyber

This is ADR-0051's own distinction applied one level out. Class is immutable for an engagement and
mutable for a person; **the company is immutable for an engagement and plural for a person.** It
therefore belongs in exactly the same place class does, and for the same reason.

### 3. Concurrent engagements are permitted across companies and forbidden within one

`reengage` currently refuses while any engagement is open. It is narrowed: a person may hold one
open engagement **per company**, and a second in the same company is still refused.

The refusal was protecting something real — two open internships at one company is a record nobody
can reason about — and that protection survives. What it was also doing, accidentally, was
asserting that a person works for one entity, which was never decided and is now false.

### 4. Signing authority resolves within the engagement's company, with a declared fallback

A document is executed by a signatory of **the company the engagement belongs to**, resolved in a
declared order rather than by a rule in prose. Two consequences the operator has decided:

**Priyanshu's own documents are executed by Rohan as CTO.** RFC-0042 gave the Director's own
documents to the CEO, which worked only while they were different people. They are now the same
person, and `docket issue` refuses a document whose subject and signatory are the same — correctly.
The fallback order is declared per company so the collapse is data, not a special case in code.

**Paridyum Cyber's documents are executed by Priyanshu.** Paridyum owns the entity and its Director
drives it on paper. This is recorded as an operator decision about who acts, not a legal opinion
about who may bind an unincorporated company — the same posture RFC-0042 §3 takes.

### 5. PARIDYUM owns every other company, and that is checked

The operator stated this as a general rule on 2026-09-09, not as a fact about one subsidiary, so
it is an invariant rather than a comment: **exactly one company declares no owner, and every other
reaches it by following `owned_by`, with no chain looping back on itself.**

It is not bookkeeping. Decision 4 resolves who may execute a subsidiary's documents *through* this
relationship — Paridyum's Director signs for the holding company — so a company whose ownership
does not reach the root has no answer to the question of who may sign for it, and would fail only
at its first issuance.

`owned_by` names a company rather than asserting "PAR" everywhere, because ownership may run
through an intermediate: a company owned by Paridyum Cyber is still owned by Paridyum, and the
check follows the chain rather than demanding the root be named directly.

`docket check` also refuses **two companies sharing a serial format**. Each keeps its own per-kind
counter, so a shared format issues `PAR/NDA/2026/0001` twice to different people at different
companies — and the serial is the string a third party verifies a document by.

### 6. Shareholding is not recorded in docket

Who owns what proportion of Paridyum Cyber is a **cap table**, and it belongs to the incorporation
documents. docket issues personnel documents and holds personnel records; a shareholding field
here would be a second home for a fact this system has no purpose for, which is the over-collection
the DPDP purpose limitation forbids and `fields.yaml` already refuses.

What docket records is the **relationship** — that Paridyum Cyber is owned by Paridyum — because
that is what decision 4 resolves signing authority through. The proportion is not needed to answer
any question docket asks, and the operator has said it is still open.

## Consequences

**Accepted costs**

- **Breaking.** Every serial and engagement identifier gains an explicit company. Nothing has been
  issued, so the migration is free exactly once — and this decision is being taken at the only
  moment that is true.
- **`org.yaml` grows a level of nesting**, and every template reading it gains an indirection.
  The coupling is light (five usages across `templates/` and `latex/`), which is what makes this
  affordable now rather than after a hundred documents exist.
- **A person's record must name a company per engagement**, so the record shape changes for
  everybody, including the four already written.
- **The fallback signatory order is a new thing to get wrong.** A company whose order is exhausted
  can issue nothing, which is the correct failure but an obscure one until the error says so.

**Gained**

- Rohan is one person with one archive, holding two offices, which is what is actually true.
- A second company costs a registry entry rather than a second deployment.
- The collapse of CEO and Director into one person stops being a special case and becomes data.
