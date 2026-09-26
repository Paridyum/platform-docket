---
id: RFC-0041
title: A posting identifier is frozen at publication, and a person record references it rather than restating the role
type: rfc
status: superseded
created: 2026-09-06
locked_at: null
supersedes: null
superseded_by: ADR-0095
resolved_by: null
scope: [web, tool]
applies_to: ["web-paridyum", "tool-docket"]
tags: [hiring, careers, docket, identifiers, compensation, personal-data, dpdp, adr-0032, adr-0033, adr-0051, adr-0063, anti-0008, std-0010, rfc-0039]
breaking: false
enforced_by: []
---

# RFC-0041 — A posting identifier is frozen at publication, and docket references it

## Status

**Accepted**, 2026-09-06. Written before the change, per ADR-0013 and ADR-0019.

## Problem

Three observable facts, none of them a preference:

1. **`jobs/` contains no postings.** The only file is `_EXAMPLE-platform-engineering-intern.json`,
   underscore-prefixed and `status: draft`, so `openJobs()` returns empty and
   `/[lang]/careers` renders a page with nothing on it. Paridyum has openings it cannot
   publish through the surface built to publish them.

2. **A person's role is prose.** `registry/fields.yaml` declares `engagement.role` with the
   purpose "Stated on the agreement, the certificate and the reference letter", and
   `people/_TEMPLATE.yaml` carries `role: FILL-ME`. Nothing connects a hire to the posting
   they answered. "Who did we hire for this posting", and its inverse "what were the terms
   we advertised to this person", have no answer that does not involve reading email.
   ADR-0063 decision 4 names this seam and explicitly does not perform it.

3. **The posting identifier is not designed to be referenced yet.** The example uses a bare
   slug, `platform-engineering-intern`. It is simultaneously the filename, the public URL
   path, and — under decision 4 — the thing a person record would point at. Nothing says it
   may not be renamed, and a rename silently breaks every reference and every inbound link.

If nothing is done: postings continue to be advertised outside the estate, the careers page
stays empty while claiming to be the hiring surface, and the seam gets wired later against an
identifier that was never promised to be stable — which is the more expensive order.

**Fixed when** a posting can be published by merge, a person record can name the posting it
came from, and a rename of a published posting is impossible rather than merely discouraged.

## Constraints

- **ADR-0051 (locked)** — "A slug is a filename, never an identity. Nothing keys off it." This
  is the binding constraint and the reason this RFC exists rather than a one-line schema edit.
  Its logic is about *mutability*, not about slugs: the person identifier is opaque because
  class and year go stale for a person, and the engagement identifier carries class and year
  because those facts are permanent *for an engagement*. Its ban on keying off a slug is
  justified by a slug being "a filename that may be corrected".
- **ADR-0063 (accepted)** — a JD is declared data, the repository is the admin surface for
  Phase A, an admin UI belongs to Phase B, and nothing in Phase A may be shaped to make Phase B
  feel closer than it is.
- **ADR-0032** — machine-readable identifiers are locale-independent. A posting is bilingual
  (`copy.en`, `copy.ar`); its identifier must not be derived from either title.
- **STD-0010** — locale-neutral interfaces.
- **C-0006 anti-goal 6** — this is internal tooling, not a second vertical. No tenancy
  machinery.
- **ADR-0038 / ADR-0064 / RFC-0039** — applications are Phase B and blocked on a store. This
  RFC touches no personal data beyond a field on a record that already exists.

## Proposal

### 1. A posting identifier is `<slug>-<year>`, and publication freezes it

    platform-engineering-intern-2026
    │                          └── the year the posting opened, four digits
    └───────────────────────────── slug: lowercase ASCII, digits, hyphens

The year is not decoration and it is not for humans. It is the same reasoning ADR-0051 applies
to an engagement identifier: **a posting's year is immutable for that posting.** Re-advertising
the same role next season is a new posting with new terms, not an edit of last season's, and
without the year the two collide on one file, one URL and one reference.

**A posting identifier is frozen the moment `status` first becomes `open`.** Before that it is a
draft filename and may be corrected freely. After that it may never be renamed and never be
reused — including after the posting closes, and including for an identical role in a later
year, which the year suffix already prevents.

**A closed posting's file is kept**, `status: closed`, and is never deleted. That is what makes a
`job_id` recorded on a person record still resolvable in 2036, which is the only reason to record
it; `closed` already exists in `STATUSES` and `openJobs()` already filters on it, so the public
page is unaffected. `jobs/` grows monotonically and that is the intended shape — it is the
register of what Paridyum has advertised about itself, and a register you can delete from is not
one.

This is what reconciles the change with ADR-0051. That ADR forbids keying off a slug **because a
filename may be corrected**; the freeze removes the premise. A frozen, never-reused string that
encodes only facts immutable for the thing it names is an identifier by ADR-0051's own test, and
this RFC asks for its `id` field to be read that way rather than for an exception to it.

Rejected within this option: minting a separate opaque posting identifier
(`PAR-JOB-2026-0001`) beside the slug. See alternatives.

### 2. `engagement.job_id` is a new optional field, and `engagement.role` stays

`registry/fields.yaml` gains one declaration:

```yaml
  - path: engagement.job_id
    group: engagement
    store: personnel
    purpose: >
      Names the published posting this engagement answered, so the terms advertised
      are recoverable from the record. Not rendered on any document.
    optional: true
```

`engagement.role` is unchanged and stays the human-readable title, because it is what the
agreement, the certificate and the reference letter print, and those are typeset documents that
must not depend on a JSON file in another repository resolving at render time.

**Optional, and it stays optional.** A hire who was not recruited through a posting — a direct
approach, a returning intern, the founder — has no posting to name, and a required field would
be satisfied with a fiction. `docket check` must not demand it.

### 3. Nothing resolves the reference at render time, and that is deliberate

`job_id` is a recorded fact, not a lookup. `tool-docket` does not read `web-paridyum`, does not
validate the id against `jobs/`, and does not fail when a posting file is deleted. Two reasons:
the repositories are separate under ADR-0001 with no shared build, and a document already issued
must render identically in 2036 when the posting is long gone.

Consequence, stated rather than hidden: a typo in `job_id` is not caught by anything in Phase A.
§5 says what catches it and when.

### 4. Phase A ends at "a posting can be published and referenced"

No admin UI, no application form, no candidate record, no status transitions beyond the three
`STATUSES` already declared. ADR-0063's split holds.

### 5. Compensation is declared, and an unpaid posting must name what it gives instead

Most Paridyum internships are unpaid; a stipend is rare. Both must be expressible, and **the
absence of a compensation field must not be how "unpaid" is said** — a posting silent on pay
reads as paid, and letting silence carry the meaning is how a public claim becomes accidentally
untrue.

So compensation is required and tagged, never inferred:

```json
"compensation": { "kind": "unpaid", "provides": ["CRT", "REF", "mentorship", "ownership"] }
"compensation": { "kind": "stipend", "amount": 2500000, "currency": "INR", "period": "monthly" }
```

`kind` is a closed set of exactly `unpaid` and `stipend`. A stipend is an integer of minor units
plus a currency code, per **ADR-0033** — no float, no formatted string. Rendering it for a reader
is the page's job in the reader's locale, per STD-0010.

**`provides` is the whole design, and it is deliberately not prose.** An unpaid posting is
positioned on what the intern actually receives, and the estate's own answer to "what do they
receive" is not a marketing sentence — it is documents `tool-docket` is going to issue, under
signature, with permanent retention:

- `CRT` — Certificate of Internship. `kind: issued`, `signed_by: [ceo, cto]`,
  `retention: permanent`, and its register entry never expires so it stays verifiable whenever it
  is presented.
- `REF` — Letter of Reference. Same shape, `required_if: reference_requested`.

Every entry in `provides` is either **a document code that exists in
`tool-docket/registry/lifecycle.yaml`** — checked by a test, because a code with no document
behind it is a promise nothing keeps — or one of a closed set of non-document values
(`mentorship`, `ownership`, `equipment`) whose meaning is fixed here rather than per-posting:

- `mentorship` — a named supervisor. `engagement.supervisor` already exists on the person record,
  so this one is also verifiable after the fact.
- `ownership` — the intern's work ships and is attributed to them. C-0003 already requires every
  commit to carry a real, signed, attributable identity, so this is a consequence of a locked rule
  rather than a claim being invented for a job ad.
- `equipment` — Paridyum provides the hardware. `EQP`, the equipment acknowledgement, is the
  document that proves it.

The persuasion, in other words, is `copy.<locale>.summary`'s job and it may say whatever is true;
`provides` is the machine-readable list of things that can be held against us. **Nothing may appear
in `provides` that no document or locked rule backs**, which is the difference between portraying
unpaid work as valuable and overstating it.

**A posting with `engagement.class: intern` may not declare a stipend, and a test refuses it.**
This is not a policy invented here — `person-classes.yaml` declares the intern class `unpaid: true`
and `forbids: compensation.salary`, on the stated ground that an unpaid student intern is "not an
employee under Indian law" and that the statutory apparatus of employment therefore does not
attach. Paying one is plausibly the fact that changes that analysis, which is why the class forbids
it rather than merely omitting it. A posting that advertised a paid internship would advertise
terms `tool-docket` is built to refuse to record, and the contradiction would surface at the offer,
after the promise was public.

`kind: stipend` is therefore available to `employee` and `contractor` postings, both of which
declare `unpaid: false` and permit a `compensation` group. Changing the intern class is a labour
and DPDP question, not a schema one, and it is out of scope here.

`provides` is required and non-empty when `kind` is `unpaid`, and forbidden when `kind` is
`stipend` — a paid posting does not need to argue for itself, and letting it list perks turns the
field back into marketing copy.

## Alternatives considered

- **Bare slug, no year** (what the example ships). One less thing to explain and a cleaner URL.
  Rejected because re-advertising a role is the normal case, not the edge one, and the first
  repost forces either a colliding identifier or an ad-hoc `-2` suffix invented under time
  pressure. Deciding the shape once, before there are any postings to migrate, is free; deciding
  it after is a migration of live URLs.

- **A separate opaque posting identifier, `PAR-JOB-2026-0001`, with the slug as URL only.**
  Most faithful to ADR-0051 read literally, and it keeps the URL free to be corrected. Rejected
  because ADR-0051's opacity requirement is argued from two premises that do not hold here: a
  posting is not a person, so no listing of postings discloses personal data — they are published
  deliberately — and a posting has no mutable class to go stale. It would add an allocator, a
  check character and a second identifier to explain, to protect against a disclosure that does
  not exist. ADR-0051 also already accepts a non-opaque identifier for the engagement, on exactly
  this reasoning.

- **Replace `engagement.role` with `engagement.job_id` and derive the printed title from the
  posting.** The cleanest end state, no duplicated title, one source of truth. Rejected for now:
  every LaTeX template prints `role` directly, a typeset document would acquire a cross-repository
  dependency at render time, and a hire with no posting would have no title at all. Left as a
  possible successor once `job_id` is populated in practice.

- **No compensation field at all**, pay discussed with the candidate only. Smallest schema, and
  it is what the example ships today. Rejected because it makes silence mean unpaid, and ADR-0063
  names pay as one of the three public claims the merge review exists to check — a claim that is
  not in the file cannot be reviewed in the diff.

- **Compensation as a free-text line in `copy.<locale>`.** Fastest to write and the most
  persuasive, because it can be phrased. Rejected under **ANTI-0008** — rendered text in a machine
  contract — and because it must then be written twice, in English and Arabic, with nothing
  keeping the two from disagreeing about the money.

- **`provides` as free-text bullet points** rather than codes. Rejected: it is where an unpaid
  posting would quietly grow claims nobody has to honour. Codes make the promise resolvable to a
  document with a signatory and a retention class, and an unbacked promise fails a test instead of
  being published.

- **A `paid: true|false` boolean.** Rejected: it cannot carry an amount, and the first stipend
  forces a second field beside it that the boolean can then contradict.

- **Do nothing in docket, ship postings only.** Half the ask, and it is the half that leaves the
  original question unanswerable. Rejected, though it is a legitimate stopping point if §5's
  ordering slips.

- **Wait for Phase B and model postings in Postgres.** Shares a store with applications and
  gets an admin UI for free. Rejected by ADR-0063 alternative 4 already: it makes content that is
  not personal data wait for infrastructure that only personal data requires.

## Consequences

**What this makes worse.**

- **Two places state the role title** — `copy.<locale>.title` on the posting and
  `engagement.role` on the person — and they can disagree. Accepted as the price of documents
  that render without a network.
- **A frozen identifier means a typo in a published slug is permanent.** The freeze has no
  escape hatch by design; the mitigation is that a posting is published by pull request and the
  review is the point.
- **`job_id` correctness is unverified in Phase A.** A wrong id is a wrong recorded fact and
  nothing complains.
- **A posting identifier now carries a year that a reader will mistake for a deadline.** The
  public page must render the title, not the id.
- **Publishing a posting now requires deciding, in writing, what an unpaid intern gets** — and
  `provides` will sometimes be shorter than the summary implies. That friction is the feature, and
  it will occasionally be uncomfortable.
- **`provides` couples `web-paridyum` to `tool-docket`'s document codes.** §3 says the tool does
  not read the web repository; this is the reverse direction, and it means a test in
  `web-paridyum` needs those codes. Pinning them in a small constant beside the test, rather than
  reading across repositories, is the lesser evil — and it goes stale silently if a code is ever
  renamed.
- **A paid internship cannot be advertised**, so if one is genuinely wanted the answer is a
  `contractor` posting or a change to `person-classes.yaml` — and the second is a labour-law
  decision wearing a schema's clothes.
- **A closed posting is kept forever**, so `jobs/` accumulates every role Paridyum has ever
  advertised, including ones it would rather not be reminded of.
- **One more optional field in a registry whose value is that it is closed.** Every optional
  field is a small tax on `docket check`'s ability to say a record is complete.

## Rollout

Three increments, each independently revertable, in this order:

1. **This RFC**, accepted and landed alone.
2. **`web-paridyum`** — the id scheme, the freeze rule written into the example's `$comment`
   and enforced by a test over `jobs/*.json` (id matches `^[a-z0-9-]+-\d{4}$`, id equals
   filename stem, `apply` present, both locales present, `compensation.kind` in the closed set, `provides` non-empty and every code backed when unpaid, `amount` an integer when a stipend), and the real postings.
3. **`tool-docket`** — `engagement.job_id` in `fields.yaml` and `_TEMPLATE.yaml`, plus whatever
   `docket check` needs so an undeclared-field rejection does not fire on it.

**Backwards compatibility.** Additive. Existing person records without `job_id` stay valid.
The example posting is `status: draft` and underscore-prefixed, so renaming it to the new scheme
breaks no URL and no reference.

**How it is undone.** Increment 3 reverts by deleting one declaration; any record carrying
`job_id` would then fail the undeclared-field check, so the revert includes stripping the field
from `people/*.yaml`. Increment 2 reverts by deleting the posting files — but **not** by renaming
a published one, which the freeze forbids; a posting published in error is closed, not unpublished.

## Open questions

- **What catches a wrong `job_id`.** The honest answer is nothing until postings and person
  records live in one store, which is Phase B. A weaker check is possible sooner — a script in
  the console that reads both working copies — and it is not proposed here because a check that
  only runs where both repositories happen to be cloned reports absence as failure.
- **Whether `engagement.job_id` should eventually be plural.** A returning intern answering a
  second posting has a second engagement, and ADR-0051 gives engagements their own identifiers —
  so this is probably a field on an engagement rather than on a person, and today `people/*.yaml`
  holds one engagement inline. Deferred to RFC-0039's item 7.
- **Whether an unpaid internship is lawful in the form advertised.** Not a schema question and
  out of scope here, but publishing "unpaid" on a public page in a named jurisdiction is a
  different act from agreeing it privately, and this RFC is the first thing that makes Paridyum do
  it at scale. Flagged, not answered: it wants a look at the applicable labour position and at
  what the internship agreement (`AGR`) already says about consideration, before the first posting
  goes `open`. The intern class already takes a position on this (`unpaid: true`, "not an employee
  under Indian law"); what is unexamined is whether *publishing* it changes anything.
  **Not a blocker for accepting this RFC**, which decides a field shape.
- **Officers have no person record.** CEO, CTO and Director exist in `operators.yaml` and
  `signatories.yaml` and nowhere else, so none of them has an NDA or an IP assignment while every
  intern will have both. That is a real gap and it is **not** this RFC's — an officer is appointed,
  not hired, and routing one through a hiring pipeline would issue internship documents and have
  the CEO sign her own offer. It wants its own artifact proposing a class with a small document
  set. Named here so it is not lost.
- **Whether `provides` should render on the public page as a list.** It is machine-readable and
  honest, which does not make it good copy; the alternative is that it exists only for review and
  the page argues in prose. Left to the page.
