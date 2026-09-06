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
