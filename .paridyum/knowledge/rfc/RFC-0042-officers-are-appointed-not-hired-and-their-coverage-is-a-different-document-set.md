---
id: RFC-0042
title: Officers are appointed rather than hired, and their document coverage is a different set signed by someone else
type: rfc
status: accepted
created: 2026-09-06
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, tool]
applies_to: ["tool-docket"]
tags: [governance, officers, directors, signatories, docket, ip-assignment, incorporation, dpdp, c-0003, c-0004, adr-0051, adr-0063, rfc-0041]
breaking: false
enforced_by: []
---

# RFC-0042 — Officers are appointed, not hired, and their coverage is a different document set


> **Amended by ADR-0077 (2026-09-09).** Decision 5's rule that the Director's own documents are
> executed by the CEO held only while they were different people. Priyanshu Sharma now holds both
> offices, so his own documents fall through to the CTO. The substitution order is declared in
> `registry/signatories.yaml` rather than stated here, so a future collapse of two offices is data
> rather than another amendment. Officer documents are also now scoped to a company.

## Status

**Accepted**, 2026-09-06. Written before any registry is edited, because one of the changes it
describes is governance-altering under C-0004 and because the other is a legal structuring
question this RFC deliberately does not answer.

## Problem

**Paridyum's officers are covered by nothing.** Rohan Batra (CTO), Ruhani Khurana (CEO) and
Priyanshu Sharma (Director) appear in `registry/operators.yaml` and `registry/signatories.yaml`
and nowhere else in the estate. Neither file is a record of a person; one says who may run the
tool, the other says who may sign on Paridyum's behalf. Consequently:

- **No officer has signed an NDA, an IP assignment, or the code of conduct.** Every intern
  Paridyum onboards will sign all three. `docket next` will not let an internship proceed without
  them.
- **The asymmetry runs the wrong way.** Interns will have assigned their work product to Paridyum;
  the three people who write most of it have not. For a company whose entire argument to customers
  is that its estate is governed and provable, the founders being the unlogged exception is the
  weakest point in the claim.
- **A fourth officer is being appointed** — Aditya Shrivastava as COO — and there is no defined way
  to do it. `signatories.yaml` already carries the scar: a name was recorded as "Founder /
  Promoter" and later removed with the note *"that third name was not connected to Paridyum"*.
  Adding a name to the file that decides who may bind the company has gone wrong here once.
- **`tool-docket` has no class for an officer.** `person-classes.yaml` declares `intern`,
  `employee` and `contractor`. Nothing else can be recorded, so today the only way to cover an
  officer is to record them as something they are not.

**Observably fixed when** every officer, including a new one, has a person record and a signed
document set appropriate to holding office, and appointing or removing an officer follows a
written procedure that leaves evidence.

## Constraints

- **C-0004 (constitution)** — irreversible and governance-altering operations require a human.
  Editing `signatories.yaml` decides who may bind Paridyum. It is governance-altering by any
  reading, and nothing may automate it.
- **C-0003** — every commit carries a real, signed, attributable identity. The reason officer IP
  matters at all: their attribution is already in the git history.
- **`org.yaml`** — `legal_status: Incorporation in Process`, `cin: null`, `incorporated_on: null`.
  **This is the binding constraint on everything about IP below.**
- **Internship Agreement §13** — "Paridyum is currently being operated by **Priyanshu Sharma**
  (Director) and may later be operated through a registered company", with transfer to that future
  entity contemplated and IP left to the separate IPA.
- **`person-classes.yaml`** — a field not needed for *this person's class* must not be collected at
  all (DPDP purpose limitation). A new class is a new statement about what may be held.
- **ADR-0051 (locked)** — person identifier opaque and permanent; engagement identifier carries
  class and period. An officer gets both on the same terms as anyone else.
- **`signatories.yaml` vs `operators.yaml`** — signing and administering are already separate, and
  the file says why: the first person to do HR administration without being an officer would
  otherwise force an untangling.

## Proposal

### 1. An officer is a class, and it is a small one

`person-classes.yaml` gains a fourth class:

```yaml
  - class: officer
    prefix: OFC
    description: >
      A person holding office at Paridyum -- director, or an officer
      designated by the directors. Appointed by instrument, not engaged by
      offer. Not an employee by virtue of office alone; where an officer is
      also employed or contracted, that is a SEPARATE engagement recorded
      under its own class.
    permits:
      - identity
      - contact
      - engagement          # office held, from when, under what instrument
      - professional
      - device
      - access
    requires:
      - identity.legal_name
      - contact.email
      - engagement.role
      - engagement.start_date
    forbids:
      - guardian            # an officer is not a minor
      - education           # irrelevant to holding office; not collected
      - statutory
      - compensation
    unpaid: true
```

`compensation` and `statutory` are **forbidden, not merely absent**, on the `employee` class's own
precedent — declared so the model need not be reworked, and collectable only when someone actually
holds a paid office. No officer is paid today. `end_date` is not required: an office is held until
it is vacated, and demanding a date up front would invite a fictional one.

### 2. Documents are scoped to classes, which they are not today

`lifecycle.yaml` declares fourteen documents and **no mapping from class to document set** — the set
is effectively "all of them", which is correct only while every person is an intern. Each document
gains an explicit `classes:` list, and `intern` is written out rather than assumed.

The officer set is deliberately short:

| Code | Document | Why |
|---|---|---|
| `NDO` | Non-Disclosure Agreement (Officer) | An officer sees everything. |
| `IPO` | IP Assignment Agreement (Officer) | Per §3, in the company's name. |
| `COO` | Code of Conduct (Officer) | Conduct, conflicts, and what a breach means for an office. |
| `ISO` | Infosec & Acceptable Use (Officer) | An officer holds the most access in the estate. |
| `DPO` | Data Protection Notice (Officer) | Paridyum processes their personal data too. |

Explicitly **not** issued to an officer: `OFR`, `AGR`, `GRD`, `INS`, `CRT`, `ONB`, `OFB`, `EXF`,
`EMC`, `EQP`, `PSH`, `GRV`. Those describe an internship, and several say so in their body text.
An officer is not offered a position, has no guardian, has no institution to confirm, and is not
issued a Certificate of Internship.

**Correction, found during implementation: `NDA`, `COC`, `ISP` and `DPN` cannot be shared.** An
earlier draft of this section listed all four as applying to an officer "unchanged". They do not.
`nda.tex` says "Intern" sixteen times and "internship" six; `infosec-acceptable-use.tex` says them
twenty-eight and eleven; `code-of-conduct.tex` states "This internship is unpaid and part-time",
which is false of a director. The party term could be swapped for a macro, but "the internship" is
the *arrangement*, not the person, and generalising it would mean editing the operative text of
documents interns have already been issued — a change to live legal prose made for the convenience
of a different class of person.

So an officer gets **its own NDA (`NDO`)**, adapted from the intern one, and the intern's `NDA`,
`COC`, `ISP` and `DPN` stay `classes: [intern]`. **The cost is two copies of confidentiality
prose that will drift**, and it is the smaller cost: the alternative rewrites a signed document's
terms.

**Written 2026-09-06 as `COO`, `ISO` and `DPO`**, in their own right rather than as edits to the
intern set — and the drafting confirmed the correction above rather than merely working around it.
Four sections are substantively different and **two are inverted**: an intern may not speak on
Paridyum's behalf and an officer may; an intern accesses only what an assignment requires while an
officer's access is administrative, making least privilege self-imposed. Two further routes had to
change because they could otherwise point at their own subject — security incidents route to the
CTO, and the Grievance Policy is an intern document — so both name another officer instead.

The officer Data Protection Notice differs in substance, not wording: it lists what the officer
class permits and no more, because a notice that overstates what is held is a defect in the notice.

**The intern `IPA` is not on the list**: it says "Intern" throughout and is scoped to work created during an internship. `IPO` replaces it for officers, and §3 settles its terms.

### 3. Officer IP is not the intern IPA, and this RFC does not decide what it is

The intern IPA reads: *"entered into between **PARIDYUM** ("Paridyum") and \<name\> ("Intern")"*,
and *"the Intern assigns to Paridyum all …"*. Reusing it for an officer fails twice, and the second
failure is not a drafting problem:

1. **It says "Intern" throughout** and is scoped to work product created "during the internship".
   Fixable by writing a template.
2. **Paridyum is not a legal person.** `cin: null`. Per Internship Agreement §13, Paridyum is
   *currently operated by Priyanshu Sharma*. So an officer IPA executed today would in substance
   have the CEO and the CTO assign their intellectual property **to Priyanshu Sharma personally** —
   and Priyanshu cannot assign anything to himself. That is not the arrangement anybody intends,
   and it is not fixed by changing the wording.

**Operator decision, 2026-09-06, reaffirmed: documents are written in the company's name and
treated as in effect from signature, exactly as they would be after incorporation.** Officer
documents name **PARIDYUM** as the counterparty, the same as every intern document already does.
No assignment to an individual, no nominee-and-trustee construction, no separate pre- and
post-incorporation drafting. Registration is filed when the company is registered; nothing waits
on it.

This is consistent with how the estate already operates: `org.yaml` carries
`legal_status: Incorporation in Process` and every issued document is rendered with it, and
Internship Agreement §13 already contemplates transfer to the entity established to operate
Paridyum. Officer documents inherit that clause rather than inventing a different mechanism.

**Operator sequencing, 2026-09-06, and it shrinks the exposure materially.** Incorporation
completes **before the first intern is hired**. So no intern IP assignment is ever executed
pre-incorporation, and the assumption below applies to exactly four officer documents — held by
the four people who would be party to any correction, all of whom are still in office when the
CIN issues. That is a different risk from the one an earlier draft of this section described,
where every intern would have signed against an entity that did not exist.

**The assumption this rests on, stated so it is not mistaken for a finding:** that an assignment
naming Paridyum before the CIN exists takes effect in the company on incorporation via §13's
transfer. Copyright itself needs no filing — it subsists automatically on creation, and
registration in India is optional and evidentiary — so nothing is lost by not filing early. The
operator's position is that the arrangement between four founders who are all party to it does not
need to be papered as though they were adverse. **It is an operator decision recorded as such, not
a legal opinion.** With the sequencing above it is a question about four documents rather than a
structural one, and it is worth putting to counsel at incorporation alongside the founders'
arrangement — which this RFC does not draft.

**The Director signs for Paridyum on officer documents** (§4), including his own, because the
counterparty is the company and not him personally. That is the simplification this decision buys:
one counterparty, one signatory, one template family, no exceptions to explain.

### 4. The Director executes officer documents, and the subject is substituted out

**Operator decision, 2026-09-06.** For the CEO, the CTO and the COO, the signatory on Paridyum's
side is the **Director**. Below the officers nothing changes: officers sign intern documents as
`signatories.yaml` already declares (`AGR`, `CRT` and `REF` carry `signed_by: [ceo, cto]`).

This satisfies the rule the previous draft asked for — **a document is never signed by its own
subject** — by naming one person rather than leaving `docket` to search.

**The Director's own documents are the exception, and they are substituted rather than refused**
(operator decision, 2026-09-06). Refusing was correct as a rule and unhelpful in practice: his
documents still have to exist. The routed signatory is replaced by **the first active signatory in
registry order who is not the subject**, which in practice is the CEO.

Registry order rather than anything cleverer, because the substitute must be the same person every
time this is asked: who signed a legal document must not depend on a hash order or on how many
documents came before it. The result is written to the register entry, because after substitution
the signatory is no longer derivable from `lifecycle.yaml` and the paper is the only other place it
appears. Refusal survives for the one case no ordering fixes — every eligible signatory being the
subject.

Under §3's decision the counterparty is PARIDYUM rather than the Director personally, so he takes
the same document set as the other three.

### 5. Appointing an officer is a procedure with evidence, not a registry edit

Adding to `signatories.yaml` or `operators.yaml` requires, in order:

1. **An instrument** — a directors' resolution or founders' written consent, dated, naming the
   person and the office.
2. **A registry entry citing it**, carrying `appointed_on` and `instrument` alongside the existing
   fields.
3. **A human approving the change** (C-0004). Never automated, never inferred from a conversation.

**Ceasing is `active: false` plus `ceased_on`, never deletion.** Documents already issued name their
signatory, and a register that cannot say who was an officer in 2026 cannot verify a document
issued in 2026. The one prior removal from this file was a correction of a person who was never
connected to Paridyum — a different act from an officer ceasing to hold office, and the schema
should be able to tell those apart.

### 6. Aditya Shrivastava's appointment as COO is an increment, not part of this design

It waits on §5's instrument and on the full legal name as it will be written on documents. Recorded
here as the change that prompted the RFC, not authorised by it.

## Alternatives considered

- **Record officers as `employee` or `contractor`.** Available today, no new class. Rejected: it
  states something untrue about the relationship, pulls in `compensation.basis` and `statutory.pan`
  as *required* fields for people who are paid nothing, and over-collects under DPDP exactly where
  `person-classes.yaml` exists to prevent it.

- **Issue officers the intern document set.** The pipeline exists and works. Rejected: it produces
  an Internship Offer and a Certificate of Internship for a director, and it has the CEO sign her
  own offer letter. Documents that say something false are worse than no documents.

- **Reuse the intern IPA for officers with the word changed.** Fastest, and it closes the IP gap
  this week. Rejected on §3's second ground — the assignee is an unincorporated entity operated by
  one of the assignors. Speed here buys a document that is void or, worse, effective in a way
  nobody intended.

- **Wait for incorporation and do all of it then.** Cleanest legally: a real company, a real
  assignee, one pass. Rejected as the *whole* answer because interns are being onboarded now and
  will sign IP assignments before any officer does. Adopted in part: §3's IP question genuinely
  does wait, while NDA/COC/ISP/DPN do not.

- **Keep officers out of `people/` entirely and cover them by contract off-system.** The documents
  exist as PDFs somewhere and nothing changes in docket. Rejected: it is the current state, it is
  how the gap arose, and an obligation with no register entry cannot be shown to exist years later
  — which is the entire premise of the register.

## Consequences

**What this makes worse.**

- **`tool-docket` stops being an internship system.** Its README opens "Paridyum's internship
  document system"; a fourth class and per-class document sets make it a people-document system,
  and every future document has to answer "which classes?" — a question nobody had to ask before.
- **The officer set will drift from the intern set.** Four documents maintained in two contexts is
  how a clause gets fixed in one and not the other.
- **Who signs is now partly emergent.** Substitution means the signatory on a document is not
  readable from `lifecycle.yaml` alone; it has to be read from the register entry. Deterministic
  and recorded, but one more indirection on a question — who signed this — that should be direct.
- **Officer documents rest on an untested assumption**, recorded in §3: that an assignment naming
  a company not yet incorporated carries into it on registration via Internship Agreement §13.
  Every intern document already rests on the same assumption, so this adds no new exposure — but
  it does extend it to the founders' own work product, which is the valuable part. Confirm with
  counsel at incorporation.
- **The IP gap stays open and is now written down.** Naming it in DECISION-DEBT makes it visible to
  anyone reading the estate, including people Paridyum would rather not show it to. Accepted: the
  gap exists whether or not it is recorded, and an unrecorded one does not get closed.
- **A legal dependency enters the critical path.** §3 cannot be finished by anyone in this estate.
- **Officer personal data now lives in `people/`**, which is gitignored and therefore backed up by
  nothing described anywhere. Same exposure as intern records, one more reason it matters.

## Rollout

1. **This RFC**, accepted and landed alone.
2. **`person-classes.yaml`** — the `officer` class. Additive; no existing record changes class.
3. **`lifecycle.yaml`** — `classes:` on every declared document, `intern` written out explicitly.
   Behaviour-preserving for every existing record, and it is the increment that must be reviewed
   most carefully because a wrong list silently stops issuing a document that is owed.
4. **§4's countersignature rule** in `docket`, with a test that an officer cannot sign their own.
5. **Officer records and issuance** — `NDO` and `IPO` for all four officers, executed by the
   Director, except his own which the CEO signs.
6. **The COO appointment**, on its instrument.
7. **The public officers page** — `registry/team.yaml` and `docket team`, exported into
   `web-paridyum` and committed there. Not read across repositories: no shared build (ADR-0001),
   and `people/` is gitignored personal data.
8. **At incorporation** — counsel confirms §3's assumption, and the founders' arrangement
   (equity, vesting, leaver terms) is drafted. Tracked in DECISION-DEBT; outside this RFC's scope.

Steps 2–5 are independent of RFC-0041; neither blocks the other.

**How it is undone.** Steps 2–4 revert as registry edits. Step 5 does not revert — an issued
document exists and its register entry is permanent by design.

## Open questions

- ~~Is Aditya Shrivastava also employed or contracted by Paridyum?~~ **Resolved 2026-09-06:
  officer only, as Chief Operations Officer.** One record, class `officer`; `compensation` and
  `statutory` stay forbidden. If he is ever paid, that is a second engagement under its own class
  and a second record, which §1 already allows without rework.
- **Equity, vesting and leaver terms.** Not in this estate, not in docket, and named only so the
  founders' arrangement is not drafted as though they do not exist.
- **Whether a director designate signs as "Director"** before the CIN is issued. `signatories.yaml`
  already flags this, and it is now load-bearing: under §4 every officer document carries that
  designation, so if the title is wrong the whole set is wrong the same way.
- **Whether officers need `INS`-equivalent third-party confirmation** — nothing external confirms an
  officer the way an institution confirms an intern, which may be an argument for the instrument in
  §5 doing that job.
- **Whether `operators.yaml` should be derived from `people/`** once officers have records, or stay
  hand-maintained. Deriving it would remove a duplicate; it would also couple who-may-run-the-tool
  to a gitignored directory.
