# Logbook — tool-docket

How this tool got the way it is, recorded as it was built rather than
reconstructed afterwards (RFC-0033). The `CHANGELOG.md` says what changed;
this says what was decided, what was wrong on the way, and what it cost.

Started 2026-09-06, part-way through the tool's life. Everything before that
date lives in `CHANGELOG.md` and `DECISION-DEBT.md`, which were doing this
job less directly.

## 2026-09-06 — officers, and the four templates that could not be shared

The ask was job descriptions and job IDs for the openings, wired to docket.
Two gaps sat behind it, and the second was not the one anybody was looking for.

**The wiring did not exist.** ADR-0063 decision 4 names the seam — a person's
`role` should reference a posting — and explicitly does not perform it.
`engagement.role` was free prose whose stated purpose is what the agreement,
the certificate and the reference letter print. So `engagement.job_id` is a new
optional field beside it rather than a replacement: those are typeset documents
and must not depend on a JSON file in another repository resolving at render
time. RFC-0041 has the argument, including why the posting identifier had to be
frozen before anything could reference it.

**Then the officers.** CEO, CTO and Director existed in `operators.yaml` and
`signatories.yaml` and nowhere else — no person record, no NDA, no IP
assignment, while every intern will sign all three. The instinct behind the
question ("we will hire our existing through this pipeline?") was right about
the gap and wrong about the mechanism: an officer is appointed, not offered, and
running one through `docket onboard` would have issued an Internship Offer and
a Certificate of Internship to a director and had the CEO sign her own offer
letter.

**What implementation found that the design got wrong.** RFC-0042 first claimed
`NDA`, `COC`, `ISP` and `DPN` applied to an officer "unchanged". They do not.
`nda.tex` says "Intern" sixteen times and "internship" six;
`infosec-acceptable-use.tex` twenty-eight and eleven; `code-of-conduct.tex`
states "This internship is unpaid and part-time", which is false of a director.
The party term is one macro away — but "the internship" is the *arrangement*,
not the person, and generalising it would have edited the operative text of
documents interns have already been issued, for the convenience of a different
class of person. So officers got their own two documents and the RFC was
corrected rather than the code bent to match it. The cost is recorded as D-46:
confidentiality prose now exists twice and nothing checks that it agrees.

**Three things had to be class-aware, and the middle one is the one that would
have shipped broken.** Documents (`classes:`, so a false document is refused);
the **stage gate** (`preboarding` requires the internship agreement and intern
IP assignment executed, so an unfiltered gate would have held an officer
forever, waiting on a document nothing would ever produce); and the record
template (`_TEMPLATE.yaml` carries education and a guardian block, both of
which the officer class forbids, so copying it produces a record `docket check`
rejects the instant it is written — which reads as a product defect rather than
as the registry working).

**Nobody signs their own document.** `signed_by_class` routes officer documents
to the Director, and `docket issue` refuses where subject and signatory resolve
to the same person. Verified against the **rendered PDF text**, not the
register: the register can be right while the paper is wrong, and the paper is
what somebody signs. `\paridyumsignblock` was added so the block resolves from
the document rather than the template naming the CEO/CTO pair in twenty-one
places.

**On the IP question, the operator overruled the design twice and the second
version is simpler.** The first draft proposed a founders' agreement with an
assignment-on-incorporation undertaking, then assignment to the Director as
nominee and trustee — both answers to "you cannot assign to a company that does
not exist" (`org.yaml` has `cin: null`; Internship Agreement §13 records
Paridyum as operated by Priyanshu Sharma personally). The operator's decision:
write everything in the company's name and treat it as in effect from
signature, as it would be after incorporation. That is what every intern
document already does, so it adds no new exposure — it extends the existing one
to the founders' own work product. Copyright needs no filing to subsist, so
nothing is lost by not registering early. Recorded in RFC-0042 §3 as an
operator decision rather than a legal finding, and flagged for counsel at
incorporation.

**Left undone, deliberately.** `COC`, `ISP` and `DPN` do not exist as officer
documents (D-46) — an officer holds the most access in the estate and has
agreed to no acceptable-use policy. `engagement.job_id` is validated by nothing
(D-47). ADITYA SHRIVASTAVA is recorded as COO with `appointed_on` and
`instrument` both `null`, because the appointment was stated and the instrument
was not produced; C-0004 and RFC-0042 §5 both want one, and the legal-name
spelling has not been checked against it.

Verified with `scripts/gates.sh` — fmt, clippy, 313 tests, `docket check` —
all green, run through the script rather than by hand, because the changelog
entry for that script exists precisely because piping `cargo fmt --check` into
`head` returns the exit code of `head`.

Skills used: `paridyum-platform:design-change` for the change lifecycle.
RFC-0041 and RFC-0042 were written, reviewed and accepted before any code in
this repository was touched.

## 2026-09-06 — the bootstrap, and what running it found

Four officers to onboard, run by one operator. Written as `scripts/bootstrap-officers.sh` in two
passes: open the records, stop; then, once a human has filled them in, check and issue. It does
not invent personal details, for the reason `_TEMPLATE.yaml` carries placeholders at all.

**Running it in a sandbox found two defects that reading the code did not.**

`blockers` refused every officer for `education.currently_enrolled is false` and
`engagement.end_date is unset`. Both are the intern class's preconditions and say so in their own
message text; `person-classes.yaml` forbids `education` for an officer and requires no end date,
so the refusals were **unsatisfiable** — no edit clears them without breaking a different rule.
Scoped to the intern class, with the DPDP s9 child check deliberately left unscoped because it is
about processing a child's data rather than about internships.

And the refusal for a self-signed document, correct as a rule, left the Director with no
documents at all. Substitution replaced it: the first active signatory in registry order who is
not the subject. Registry order rather than anything cleverer, because the substitute has to be
the same person every time — who signed a legal document must not depend on hash order.

**The sandbox mattered for a second reason.** `docket new` allocates a person identifier that is
never reused (ADR-0051), so a test run against the real store would have burned four of them.
`DOCKET_PEOPLE` and `DOCKET_STORE` made a throwaway store possible; the real run is the
operator's.

Verified: all eight documents issued in the sandbox, the register recording `signed_by=['director']`
for three officers and `['ceo']` for the Director, and `pdftotext` confirming **RUHANI KHURANA,
Chief Executive Officer (CEO)** in the Paridyum block of the Director's own NDA with PRIYANSHU
SHARMA countersigning as Officer. Sandbox removed; nothing was written to the real store.

## 2026-09-06 — the three officer policies, and the two that were inverted

D-46's open half: `COC`, `ISP` and `DPN` did not exist for officers, so the four people with
the most access in the estate had agreed to no acceptable-use policy. Written as `COO`, `ISO`
and `DPO`.

**The interesting part is that a word-swap would have been wrong in both directions.** Going
through the intern text section by section, four differ in substance and two are inverted:

*Speaking publicly* — the intern code says they may not speak on Paridyum's behalf. An officer
may; that is part of the office. Copying the intern clause would have forbidden a director from
doing their job, so the officer version is about care instead: a claim must be true when made,
not merely intended, and the officers' names are on the public site, which is what makes it
matter more for them than for anyone else.

*Access* — the intern policy grants access to what an assignment requires. An officer's access
is administrative and broad by necessity, so the boundary cannot be drawn by somebody else. The
officer version makes least privilege a discipline they impose on themselves: **being able to
reach something is not a reason to.** Copying the intern clause would have described a
permission model the estate does not have.

Two routes could point at their own subject, which is the same defect the signature
substitution exists to prevent, in a different place. `org.yaml` routes security incidents to
the CTO — so an incident involving the CTO had nowhere to go. And the Grievance Policy is
`classes: [intern]`, so it cannot be the route for a concern about an officer. Both now name
another officer, and the ISO says why in one line.

*Breach* — an office is vacated or removed by the directors. It is not terminated, and there is
no Internship Agreement to point at, so the intern clause was simply describing a thing that
cannot happen.

**`DPO` differs in what it says is collected, not only in wording.** The intern notice lists
education details, emergency contact and a supervisor; the officer class forbids `education` and
`statutory` and does not permit `emergency_contact`. A data protection notice that overstates
what is held is a defect in the notice — its entire purpose is to be an accurate account of the
processing. So section 1 is derived from what the class actually permits, and it adds the one
thing an officer has that an intern does not: two facts about them are *published*, not merely
held.

Verified by rendering all five officer documents and reading the PDF text, not the source. The
remaining "intern" strings in the output are deliberate cross-references — "differs from the
copy given to an intern", "the Grievance Policy is an intern document" — plus one false positive
on "internal business information".

Left open and narrowed rather than closed: D-46 is no longer a coverage gap, it is a
duplication. Five officer documents now carry text adapted from five intern ones, and nothing
compares them. Diffing fails immediately because they are supposed to differ.

Gates green: fmt, clippy, tests, `docket check` — 26 documents, 0 errors.

## 2026-09-06 — addresses, and the subdomain that would have repeated a locked mistake

Four officers need email, interns follow, and nothing said what an address looks like. C-0007
governs repositories, crates, API paths, tables, environment variables and images — not
mailboxes. Three function addresses existed by custom: `legal@`, `hello@`, `security@`.

The shape proposed was `vivaswan.pdol001.paridyum@gmail.com`, with `intern.paridyum.com`
floated for the domain half.

**The subdomain is the part worth recording.** `intern.paridyum.com` is ADR-0051's locked
mistake in a worse place. That ADR rejected `PAR-INT-2026-0001` as a person identifier because
`INT` is a class and class is mutable for a person — "the predictable case is the one that
breaks it: an intern is hired." An address is a far worse place to make it than a folder name,
because most copies are not yours: they are in other people's address books, on signed
documents, in university records, and in commit authorship that C-0003 requires be real and
attributable. A folder can be renamed by one person. An address cannot.

The honest counter is sending reputation, and it has a standard answer that does not put a
mutable fact in someone's identity: isolate bulk and automated sending on its own subdomain.
That is about what sends, not who sends.

**`pdol001` could not be decoded and was replaced rather than guessed at.** It is not the
person identifier and not the engagement identifier. Whatever it meant, docket already mints
something with exactly the properties a Gmail disambiguator needs — unique across Paridyum
forever, never reissued, and carrying a check character. Adding a second hand-kept counter
would have been a second thing that must never be reused, which ADR-0051 singles out as the
failure that silently reattaches one person's records to another.

**The Gmail account name keeps the given name, and that is a deliberate departure from
ADR-0051's opacity rule.** That rule is argued from a directory listing disclosing who Paridyum
engages to anyone who can read it. A Gmail account name is not a listing anybody browses; what
it has to be is findable in an account switcher when an operator holds twelve of them.

**The operator chose free Gmail long-term rather than Workspace, and the cost lands on
something docket was built to do.** `access.company_email`'s stated purpose is "so offboarding
can prove it was removed", and a consumer account cannot be suspended, audited or recovered by
Paridyum. Rather than let the record imply a proof that does not exist, the field now means the
published alias specifically and `access.mailbox_email` is separate: **severing an alias is
provable; recovering a consumer mailbox is a request.** Splitting them is also the mitigation —
Paridyum can end someone's ability to act as Paridyum without needing access to their mailbox.

Derivation is a domain function with six tests rather than a documented convention, including
the two cases a convention gets wrong: a three-part name keeps every part, and a name with no
ASCII yields a visibly wrong local part instead of an invented romanisation — permanent, under
a rule that forbids renaming.

`gates.sh` earned itself again: `cargo build` was clean and clippy was not (`-D warnings`), so
building and eyeballing would have recorded a failing gate as green.
