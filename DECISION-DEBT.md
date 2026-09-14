# Decision debt — tool-docket

Open questions that block issuing real documents. None is a code problem.

## D-1 — Governing law forum  (RESOLVED, pending confirmation)
The `.docx` originals all read "courts in [City, State, India]". Now set to
`Dehradun, Uttarakhand, India` in `registry/org.yaml`. **Confirm this is the
intended forum.** `docket check` and `docket onboard` both refuse to run if
it is ever unset again.

## D-2 — Who Paridyum's officers are  (RESOLVED 2026-08-19)
The source `.docx` set gave three different answers. It now has one home,
`registry/signatories.yaml`, and the underlying fact has been settled:

- **RUHANI KHURANA** — Chief Executive Officer
- **ROHAN BATRA** — Chief Technology Officer
- **PRIYANSHU SHARMA** — Director

`Vikas Darmora, Founder / Promoter`, who appeared on the business
communication letter in the source material, is **not connected to Paridyum**
and has been removed. Confirmed by Rohan on 2026-08-19.

The Internship Agreement's future-incorporation clause and the business
communication letter both now name the Director from the registry rather
than a literal. Incorporation is in process, so `director` is a director
designate until a CIN exists and `org.yaml` records it.

## D-3 — POSH policy  (DRAFTED, needs counsel review)
Written as `templates/policy/posh-policy.tex` and owed at `preboarding`.
The POSH Act 2013 applies to every workplace in India regardless of
headcount, and interns count toward the ten-person Internal Committee
threshold. Below ten, complaints route to the district Local Committee, which
`org.yaml` `posh.local_committee` names and the template routes to.
**Two things to confirm:** the correct Local Committee designation and
address for Dehradun district, and the whole document's wording with
counsel.

## D-4 — DPDP notice  (DRAFTED, needs counsel review)
DPDP Rules notified 2025-11-13, phased. Section 7 makes "purposes of
employment" a legitimate use, covering the intern's own data without
consent. It does not obviously cover the **emergency contact**, who is a
Data Principal in their own right and is not Paridyum's employee. The
source's Privacy Notice box was a courtesy paragraph, not a DPDP notice.

Now addressed two ways: `templates/policy/data-protection-notice.tex` covers
the Intern's own data, and the Emergency Contact Form carries a notice
written for the third-party position, plus a confirmation that the Intern
told the named people. Both need counsel review.

## D-5 — Unpaid status  (ENFORCED, legal review still owed)
Unpaid student internships are lawful in India when genuinely educational,
time-bound and linked to study. The Onboarding Form collects exactly those
facts (§4) and the Internship Agreement never references them, while
asserting "unpaid" alongside "other reasonable tasks", a 7-day notice
period and full IP assignment. Bind the agreement to recorded student
status rather than asserting unpaid in isolation.

Now enforced: the Internship Agreement and the Offer both state the basis
conditionally on `education.currently_enrolled`, and `docket onboard`
REFUSES to issue without it. That makes the basis visible and consistent. It
does not make the arrangement lawful — the underlying question of whether
this internship's actual duties look like employment is one for counsel.

## D-6 — The .docx originals are fuller than the .tex versions
Measured 2026-08-19. The LaTeX NDA compressed a 16-item enumerated
definition of Confidential Information into one prose sentence; the
InfoSec policy is 822 words in Word against roughly 450 in LaTeX. The
`.docx` files in `.intake/` are the source of truth. Regenerating from the
`.tex` would ship an abridged NDA with a narrower confidentiality
definition.

## D-7 — Logo  (RESOLVED 2026-08-19)
Rebuilt as vector geometry in `latex/assets/paridyum-logo.tex`, which
compiles to the PDF the class uses. Vector rather than the supplied raster
so it stays sharp at the 1.9cm the letterhead sets it at, and carries no
raster artefacts into a legal document. Source is committed, so it is
regenerable and editable.

Replace by dropping a real `paridyum-logo.pdf` (or `.png`) into
`latex/assets/`; the class prefers `.pdf` and falls back to `.png`.

## D-7a — Superseded note: no logo asset  (RESOLVED, see D-7)
`latex/assets/paridyum-logo.pdf` does not exist. The class draws a
placeholder box rather than failing the build. A vector (PDF or SVG)
is wanted, not the PNG — it is set at 1.9cm and must stay crisp in print.

## D-8 — Verification cannot rely on re-rendering across toolchain versions
Output is byte-reproducible on a *fixed* TeX Live. A future TeX Live will
change glyph positioning and therefore the hash. So the register must
verify a presented PDF against the **stored hash of the issued file**, and
treat re-rendering only as a CI reproducibility check on a pinned
toolchain. Recorded here because the opposite design is the intuitive one
and it silently stops working in about two years.

## D-9 — The offboarding checklist had no intern signature  (RESOLVED)
Section 5 of `OffBoarding.docx` has Paridyum tick a box confirming the
intern "has been reminded that confidentiality obligations continue", with
no acknowledgement from the intern. Nothing evidenced the reminder, which is
that section's entire purpose. An intern acknowledgement block was added.

## D-10 — The CLI is Python, and `tool-` implies Rust  (RESOLVED 2026-08-19)
`docket` is now a Rust crate: `manifest.yaml` declares `class: tool`,
`stack: rust`; the binary is the bare capability per C-0007; the workspace
lints, `clippy.toml`, `deny.toml`, `rustfmt.toml` and `rust-toolchain.toml`
are the estate's. Clippy is clean under `pedantic` with `unwrap_used`,
`expect_used`, `panic` and `indexing_slicing` denied.

The Python implementation is archived at
`archived/docket-python-2026-08-19/` and is not developed further. Output
parity was verified: the two produce byte-identical PDFs for the same input,
once the epoch matches — see D-13.

## D-11 — Verification depends on the stored hash, not re-rendering
Restating D-8's consequence now that it is implemented: `docket verify`
compares a presented PDF against the SHA-256 recorded at issue. It does NOT
re-render. That is correct and deliberate — a future TeX Live will change
glyph positioning and every re-rendered hash with it. The register also
records `texlive: 2026` so a reproduction attempt knows what to pin.

## D-12 — tool-forge's manifest was schema-invalid  (FIXED 2026-08-19)
`lifecycle: alpha` is not in the schema's enum, and `provides` was missing
entirely. Both corrected in `tool-forge/manifest.yaml`. It is a different
repository, but it is the same estate and the defect was a one-line
schema-compliance error rather than a decision about how tool-forge works.

The more useful half of the finding stands: **nothing catches this.**
`docs.manifest_schema_valid` is owned by tool-forge and is one of the 77
checks awaiting it, so no manifest in the estate is validated today except by
hand. `templates/rust-service/manifest.yaml` had the same `provides` gap and
was propagating it into every scaffolded repository.

## D-12a — Superseded note: tool-forge's manifest  (see D-12)
`knowledge/manifest.schema.json` permits `lifecycle` values
`experimental | beta | production | deprecated`. `tool-forge/manifest.yaml`
declares `lifecycle: alpha`, which is not in that enum. Found while writing
this repository's manifest.

Either `alpha` belongs in the enum, or tool-forge's manifest is wrong. Worth
noting that nothing caught it, which suggests no check validates manifests
against the schema yet — the more useful finding of the two.

## D-13 — The Python implementation had a timezone bug  (FIXED in the port)
It computed `SOURCE_DATE_EPOCH` from **local** midnight, so the same document
issued on the same date produced different bytes on machines in different
timezones — which would have made the register's hashes unreproducible
outside Asia/Kolkata. The Rust implementation uses UTC midnight. Recorded
because the archived code is kept as a reference and this is the one place
it should NOT be copied.

## D-14 — CI workflow  (RESOLVED 2026-08-20)
This entry said `infra-ci` "does not exist yet". **That was wrong when it was
written and stayed wrong for a day**, which is the more useful part of the
record: `infra-ci` existed with its five scripts, and what was actually missing
was `.github/workflows/rust-ci.yml` — the file ADR-0021 names by path, and
without which `uses: Paridyum/infra-ci/.github/workflows/rust-ci.yml@v1`
resolved to nothing for every repository in the estate.

Installed from `knowledge/templates/infra-ci/rust-ci.yml`, and this repository
now calls it from `.github/workflows/ci.yml`. Running `check-repo.sh` properly
against this repository — rather than against the vendored snapshot, which is
what it inspects without `--root` — then found four real defects here: no
`CLAUDE.md`, no `.claude/settings.json`, no `.gitattributes`, and a
`TESTING.md` missing the three sections STD-0008 requires. All four fixed.

Whether it PASSES on GitHub is a separate and still-open question: see D-28.

## D-15 — Statutory registers  (LIKELY NOT OWED — researched 2026-08-19)
The governing law is the **Uttarakhand Shops and Establishments (Regulation of
Employment and Conditions of Service) Act, 2017**. Two facts decide whether it
binds Paridyum:

1. It applies to establishments employing **ten or more workers**.
2. **"Worker" means a person employed for hire or reward**, and expressly
   excludes an apprentice under the Apprentices Act, 1961.

An unpaid intern works for no hire and no reward. On the face of the
definition they are therefore not a worker under this Act, do not count toward
the ten-worker threshold, and the Act's registers, hours and leave provisions
do not attach to them. Paridyum, engaging only unpaid interns, appears to have
no registration obligation and no statutory register to produce.

**THIS IS THE SAME QUESTION AS D-5, and that is the point.** The conclusion
rests entirely on the internships being genuinely unpaid and genuinely
educational. If an intern were reclassified as an employee — the risk D-5
describes — they become a worker for hire, the threshold starts counting them,
and the registration and registers attach. One finding of fact moves both.

So this is not "a rendering job we have not done". It is a job that is not
owed, for a reason that could stop being true.

**Confirm with counsel, not from this note.** Two specifics are unresolved and
the sources disagree: whether registration is valid for five years and
renewable or requires no renewal, and whether Dehradun district prescribes its
own forms. Neither matters until the ten-worker question changes.

## D-16 — The confidential store is declared but not implemented  (DELIBERATE)
`person-classes.yaml` declares a `confidential` store for medical,
disciplinary, grievance and POSH material, and this tool does not write it.
That is intentional rather than unfinished: a POSH complaint routed through
a CLI that anyone with repository access can run is a confidentiality
failure. The declaration exists so the boundary is written down and nobody
puts that material in the personnel record by default.

If it ever needs a system, it needs a different one, with its own access
control and its own custody rules.

## D-17 — `date_of_birth` for interns  (REVERSED 2026-08-19)
**This entry previously said the opposite, and it was wrong.** It read: "an
intern's engagement turns on enrolment, not age, so there is no purpose that
would justify holding their date of birth."

The DPDP Act defines a **child** as anyone who has not completed eighteen
years (s2(f)), and s9 requires **verifiable consent from a parent or lawful
guardian** before a child's personal data is processed at all. Penalties
reach Rs 200 crore. Paridyum onboards university students; a first-year
student can be seventeen.

Not collecting a date of birth does not remove that obligation. It removes
the ability to SEE it. So `identity.date_of_birth` is now required for the
intern class, a `guardian` field group exists, and `GRD` (Parent / Guardian
Consent) is owed at preboarding whenever the person is under eighteen on
their start date. `docket onboard` refuses to issue for a child with no
guardian recorded, and `docket check` reports guardian data on an adult's
record as over-collection.

## D-18 — Certificates are externally verifiable  (RESOLVED 2026-08-19)
A certificate nobody outside Paridyum can check is a nicely typeset claim.
Issued documents now carry a verification code in the page footer, derived
from the serial, the recipient, the issue date and the hash of the template
as issued. `docket verify` recomputes it from the register.

It is a **lookup check, not a signature**: it proves the document matches a
register entry, not that the holder is the person named. Its forgery
resistance comes from the register being private. A cryptographic signature
would be a different and larger decision.

## D-19 — Signature is recorded, not produced  (RESOLVED 2026-08-19)
`docket sign` records that a document came back executed, in its own
hash-chained ledger at `store/register/signatures.jsonl`.

**Why a separate ledger rather than a field on the register entry.** The
register is append-only so that what was issued cannot be rewritten once
someone has relied on it. Recording a signature by mutating the issuance
entry would invalidate its digest and every digest after it. A signature is
a new fact learned later, so it is a new record.

`Entry.signed` — an `Option<Signature>` that was declared and never populated
— has been removed. It implied the register tracked signature and it did not.
Removing it was proven safe first: a `skip_serializing_if = "None"` field is
absent from the serialised form either way, so no sealed entry's digest moves.

## D-20 — The preboarding access gate is enforced  (RESOLVED 2026-08-19)
`lifecycle.yaml` said "access to any Paridyum system requires ISP and NDA
signed" in a `gate:` prose field that nothing read. It is now
`requires_signed: [NDA, ISP, IPA, AGR]`, and `docket advance` refuses to move
a person out of `preboarding` while any of them is merely issued.

`--force` exists, prints that the gate was not checked, and the move is
recorded in the event log either way. Refusing forever is not a workflow; a
silent override would be worse than both.

## D-21 — The Internship Offer was wrongly treated as needing nothing back
Found while writing the signature tests. `needs_countersignature` keyed only
on `kind`, and the Offer is `kind: issued` because Paridyum produces it — so
it was marked complete the moment it was posted, despite carrying an
acceptance block the person signs. The rule now also honours
`countersigned_by`. Recorded because the same mistake is available to any
future document whose `kind` and `countersigned_by` disagree.

## D-22 — Five documents  (BUILT 2026-08-19)
All five written, registered and issuing:

- **INS** Internship Confirmation for the Institution — addressed to the
  training and placement officer, owed only where
  `education.institution_requires_noc` is set. The onboarding form asked that
  question and nothing acted on it.
- **PUB** Consent to Use Name, Photograph or Likeness — **owed by no stage,
  deliberately.** A consent a lifecycle stage requires is not freely given,
  and a consent that is not freely given is not consent. `check` reports it as
  a warning; that is the intended state. Issue it on request.
- **COC** Code of Conduct — the sections that matter are conflicts of interest
  and outside work, because an unpaid intern is free to work elsewhere and
  Paridyum has to say what that means rather than leave it unstated.
- **GRV** Grievance Policy — routes explicitly around the supervisor, and
  around an officer where the concern is about one. Acknowledged rather than
  merely published: a route the person never saw is not a route.
- **EXF** Exit Feedback — asks about the documentation as well as the work,
  and states that it is not consulted when a certificate or reference is
  written.

COC and GRV also join the preboarding signature gate, so a person cannot move
to `active` without having acknowledged both.

## D-22a — Superseded note: five documents missing  (see D-22)
Not built, in rough order of exposure:

1. **Institution / NOC letter.** The Intern Onboarding Form asks whether the
   institution requires one and nothing produces it. The set opens that loop
   and never closes it.
2. **Consent for use of name, photograph or likeness.** Not covered by the
   DPDP s7 employment legitimate use; needs specific consent. Will come up
   the first time anyone posts an intern on LinkedIn.
3. **Code of Conduct.** InfoSec covers security and POSH covers harassment.
   Nothing covers conduct, availability, conflicts of interest or outside
   work — the last being directly relevant to an unpaid intern.
4. **Grievance route.** POSH tells an intern where to take a harassment
   complaint. Nothing tells them where to take any other one.
5. **Exit feedback form.** The offboarding checklist ticks "intern feedback
   collected" against a form that does not exist.

## D-23 — The test suite is slow because typesetting is real  (ACCEPTED)
`tests/signing.rs` took 51 seconds because each case issued a nine-document
pack and every issuance runs pdflatex three times. The default fixture now
issues one document and the two gate cases ask for four: 10 seconds.

The remaining cost is deliberate. `tests/issuance.rs` renders real PDFs
because the guarantees it makes are about output bytes, and a test that mocks
the typesetter cannot make them.

## D-24 — Retention measured from the wrong date  (FIXED 2026-08-19)
`retention.yaml` states every period relative to the END of the engagement --
"8 years after the internship ends", "deleted within 30 days of the last
working day" -- and the first implementation measured all of them from the
document's ISSUE date.

Wrong by the whole length of the engagement, and for `emergency_contact`
actively harmful: 30 days after issue falls in the middle of the internship,
while the person is still working and the contact is still needed. The tool
would have advised deleting it.

Each class now declares `from:` explicitly (`engagement_end`, `issue`, or
`never`), the clock runs from the separation date, and a document belonging to
a still-open engagement is reported as not-yet-started rather than compliant.
A test asserts that any class whose prose says "ends" or "working day" declares
`engagement_end`.

## D-25 — The pack is a copy, not the record  (DELIBERATE)
`docket pack` copies rather than moves, and a pack is cleared and rebuilt
rather than updated. The store stays the record; a pack is a distribution
artefact that can be deleted without consequence.

The alternative — packing being where documents live — would make a folder
someone has already emailed part of the archive, and would put the erasure
guarantee inside a directory anyone may have copied.

## D-26 — `04_Optional` is how consent stays consent  (DELIBERATE)
The pack's optional folder is not a convenience. A document owed by no
lifecycle stage lands there by construction, which is what keeps the publicity
consent a genuine choice: a consent that appears in `01_Sign_and_return`
alongside the NDA is not freely given, and a consent that is not freely given
is not consent.

`docket check` reports PUB and LTR as "declared but owed by no stage". That
warning is the mechanism working, not a gap to close.

## D-27 — This tool does not send mail  (DELIBERATE)
`docket emails` renders a message and stops. Sending is an outward-facing
action with its own credentials, its own failure modes and its own audit
trail, and none of that belongs in a document register — a register that can
also email is a register whose failure modes now include "sent an intern's
agreement to the wrong address".

The bodies are plain text for a person to paste. That person is the audit
trail, and their sent folder is the evidence.

## D-28 — CI is configured but has never run  (CLOSED — it runs, and the entry was stale)

**Superseded by reality on 2026-09-12.** CI has run many times; this entry described the state on
2026-08-20 and nobody came back to it. `gh run list` shows runs from 2026-09-06 onwards, and the
workflow file itself cites measurements from real runs ("First real run measured 64.81%").

**What the entry got right is the warning it ended with**, and it was worth more than the status
line above it: *"do not treat a green local gate as evidence that CI passes."* On 2026-09-12 that
was exactly the situation -- four gates green locally, three consecutive CI runs red, and nobody
looking. The failure was one test, and D-45 holds it.

The lesson kept rather than the entry: a red CI is not visible from this checkout unless somebody
asks, and `gates.sh` does not ask. `scripts/ci-parity.sh` closes the common half of that gap.

## D-29 — Nothing records what has already been sent  (OPEN)
`docket next` lists messages as "ready to send", not "outstanding", and says
so. There is no sent-flag because this tool does not send mail (D-27) and a
flag it could not verify would be worse than none — it would assert something
false the first time someone sent from their own client without telling it.

The honest fix, if this ever matters, is `docket event <slug> message_sent
--detail <id>`, recorded like every other event, by the person who sent it.
That is a real workflow step rather than a guess, and it is deliberately not
added until someone actually wants it.

## D-30 — The Tectonic render path is unverified end to end  (CLOSED 2026-09-12, by finding it broken)

**Closed by verification, and the verification failed.** Tectonic was installed and pointed at
this repository for the first time. Two defects, in order:

`TEXINPUTS` was set for both engines and **Tectonic does not read it** -- it searches its bundle
and the input file's directory and nothing else. Every render failed with
`File 'paridyum.cls' not found`. Fixed: the invocation now passes `-Z search-path` once per
directory, which is the engine's documented equivalent and which nothing had ever exercised.

With that fixed, the real incompatibility appeared: `paridyum.cls` uses **`pdfx` for PDF/A**, and
under Tectonic's XeTeX engine it fails with `CreationDate is not properly supported`. 7 of 13
issuance tests pass; 6 cannot render at all.

**So the engine docket advertised as its supported alternative does not work**, and had not
worked at any point, because no machine on the estate had it installed to find out. The search
path fix stays -- it is correct and was plainly a bug -- and D-57 holds what remains.

## D-31 — The interview cannot ask for a nested field  (OPEN, small)
`identity.pronouns` is a single declared field whose value is three sub-keys,
so the interview asks for `she/her/her` and expands it. Any future field with
that shape needs the same special case in `enrol::scalar`, which does not
generalise.

The honest fix is for `fields.yaml` to declare the shape — a `kind:` on the
field — rather than for the code to know which paths are special. Not done
because there is exactly one such field, and inventing a type system for a
sample size of one is how registries acquire concepts nobody needs.

## D-32 — Optional fields are answered by editing the record  (DELIBERATE)
The interview offers to stop after the required questions and most operators
will. The other 38 fields are then filled in by editing the file, which is what
the generated comments are for.

Asking all 50 would be worse: an operator who is told fifty things are needed
stops reading the purposes, which are the only reason the prompts are worth
showing at all.

## D-33 — The TUI, and why it has no dependency  (RESOLVED 2026-08-20)
Built as `docket --tui`, with **no new dependency**, after `ratatui` could not
be fetched on a machine with under 2 GB free.

That turned out to be the better answer rather than a workaround. A read-only
view needs three things: clear the screen, print, read input. The first is two
ANSI escapes. The only part that genuinely needs a library is reading a single
KEYPRESS, which needs raw mode, which needs `termios`, which needs `unsafe` --
and `unsafe_code` is `forbid` crate-wide. So the dependency was never available,
and line-based navigation (a number, then Enter) is not a compromise but the
only shape the lint policy permits.

Fourteen transitive crates avoided, and the binary stays installable on a
machine with no toolchain, which was the point of D-2 in the first place.

## D-33a — Superseded note: the TUI was blocked on disk  (see D-33)
Requested and agreed: CLI stays the default, `--tui` as an alternate. Not
built, because adding `ratatui` failed with `No space left on device`.

The machine is at 199 GB of 228 GB with well under 2 GB free. `ratatui` pulls
fourteen transitive crates and their build artifacts, which is not a
responsible thing to add to a volume in that state — and a dependency that
cannot be fetched leaves `Cargo.toml` naming something nobody can build. The
Cargo.toml entry was reverted for exactly that reason; the repository builds.

**This is a machine problem before it is a project problem.** `tool-docket` and
`tool-forge` between them held 1.3 GB of regenerable `target/` output, which
was reclaimed to get the build working again, and that is a rounding error
against 199 GB. Something else is using the disk.

When there is room, the scope agreed was deliberately narrow: a READ-ONLY
dashboard — roster, and the selected person's `next` steps — because that is
the one thing the CLI genuinely cannot do well (seeing everyone at once), and
because a read-only view cannot corrupt a record while it is half-finished.
Data entry is already solved by `docket new --interactive`.

## D-34 — `requires` is about lawfulness, not about rendering  (RESOLVED 2026-08-20)
Found by running the wizard end to end: a record answering only the eight
fields the `intern` class requires could not issue the Internship Offer, which
failed with `Undefined control sequence`.

Both lists are correct and they are not the same list. `person-classes.yaml`
`requires` says what must be true for the engagement to be lawful — enrolment,
an end date, a date of birth. A template's needs are whatever it happens to
print, which for the offer includes the supervisor and the weekly hours.

Conflating them would be wrong in both directions: making every printed field
class-required would refuse records that are perfectly lawful, and dropping the
class requirements would let an unlawful one through.

So they are checked separately. `issue::unrenderable` renders the definitions a
document WOULD get, scans the template for commands dereferenced without a
`\dkfill` fallback, and reports the difference as record paths --
`engagement.supervisor`, not `\dkPEngagementSupervisor`. `docket next` shows it
before the issue step, because issuing would fail.

The distinction between a bare command and one inside `\dkfill` is load-bearing:
a form wants a ruled line where a value is absent, and an agreement wants the
build to stop.

## D-35 — CI does not exercise rendering  (CLOSED 2026-09-12)

A `render` job in `tool-docket/.github/workflows/ci.yml` installs TeX Live and runs the four
suites that produce PDFs, plus a step that **fails if any of them skips** -- a skip is printed to
stderr and would otherwise leave the job green while testing nothing, which is the shape of
D-45's defect.

**TeX Live rather than Tectonic, and that ordering is the finding rather than the preference.**
Tectonic was tried first for the obvious reason -- a single 10MB binary against a multi-hundred-MB
apt install -- and cannot render these documents (D-30, D-57). More decisively, pdflatex is what
every developer machine runs and what produced every document this estate has issued: a CI job
rendering with a different engine would be verifying something nobody ships.

`texlive` is now declared in `tool-registry.yaml`. It had been a critical dependency of the
estate's only user-facing output since the beginning and was in no registry, because it arrived
as "whatever the developer already had" rather than as a decision.

## D-36 — Coverage in CI measures the no-typesetter subset  (OPEN, and narrowed)

Coverage is 60% rather than 70% because every `cmd_*` path that issues a document needs a
typesetter, and the runner measuring coverage has none. `main.rs` sits at 14% with 940 of 1098
lines uncovered; every `domain/` module is 95-99%.

**Narrowed on 2026-09-12, and the narrowing is the useful part.** D-35 is closed: a `render` job
now installs TeX Live and typesets for real. Coverage did not move, and the reason is that they
are **two different jobs** — rendering is docket's own job in its own workflow file, coverage is
the shared `infra-ci` workflow's job in another repository. Only one of them gained a typesetter.

So what is left is not "CI cannot render" but "the job that measures coverage cannot", and closing
it means either teaching the shared workflow to install a typesetter for one consumer — which is
the wrong shape, since no other repository typesets anything — or measuring coverage in docket's
own render job and having the shared one stop asserting a total. The second is probably right and
is a change to `infra-ci`, not here.

Until then the number stays at 60 with the cause stated, and `ci.yml` carries the correction so
the next reader does not repeat the inference that closing D-35 would raise it.

## D-37 — No class describes an internal operator application  (OPEN, recorded not closed)
`class-registry.yaml` has ten classes and none fits a graphical interface used
by Paridyum's own staff:

- **`web`** — *"A user-facing web application, served from the cluster"*,
  `stacks: [typescript]`, `deployable: true`. A locally served interface is not
  cluster-served and need not be TypeScript.
- **`tool`** — *"Developer-facing executables"*, `stacks: [rust]`,
  `deployable: false`. Everything fits except who it faces, and HR is not a
  developer.

**Deliberately not closed.** The served HR platform
(`docs/superpowers/specs/2026-08-21-served-hr-platform-design.md`) lives inside
`tool-docket` as a `docket serve` subcommand, so nothing new is being classed
and the gap does not block it. Adding a class to describe one subcommand would
be inventing a concept before its first instance — the empty-glossary state
`artifact-registry.yaml` records this repository having paid for already.

It becomes real the first time an operator interface needs its own repository.
`class-registry.yaml`'s own header says that addition should be cheap, and it
is: a registry entry plus an ADR, which is the mechanism C-0005 created.

## D-38 — The HR platform has no per-operator authorisation  (OPEN, by design for now)
The served platform binds loopback only and is reached over an SSH tunnel, so
SSH keys are the authentication and the operating system is the identity. The
application has **no login of its own**, which is what keeps ADR-0047 from
engaging: there is no identity system present to be non-compliant.

The cost is stated rather than hidden: **anyone who can tunnel in has full
write access.** There are no roles, and `--by` names who is acting but does not
constrain what they may do.

ADR-0047 point 1 already describes *"a separate `paridyum` instance in the home
region [that] holds staff and operators"* — exactly these users. When
`platform-identity` exists, OIDC becomes a second authentication adapter beside
loopback-only, added rather than swapped (ADR-0054). Until then, treat SSH
access to the HR workstation as equivalent to full access to every record.

## D-39 — RESOLVED 2026-08-21 — the API writes, and every write names its author

Every route is a `GET`. Enrolment, issuance, signing, advancing a stage and pack
assembly all remain CLI-only, so an HR operator still needs a terminal for
anything that changes a record.

That was the sequencing the design chose (§10), and the reason holds: writes need
operator resolution first, because a write that cannot name who made it is not
auditable — and this process has no way to name one. The CLI takes `--by` and
resolves it against `registry/operators.yaml`; over HTTP the equivalent is the
SSH account, which the process cannot see from inside a loopback socket.

**Resolved.** Four write routes — enrol, onboard, sign, advance — each resolving an
operator from an `X-Docket-Operator` header against `registry/operators.yaml` and
refusing with 403 when it is absent, blank or unknown.

**What this is:** attribution exactly as strong as the CLI's `--by`. A claimed
identifier, checked against the registry so it must name a real operator, not
proven. SSH is what proves anything: the server binds loopback only, so reaching
it means holding a key to the machine.

**What it is not:** authorisation. Anyone who can reach the socket may claim to be
any operator in the registry — which is the CLI's position too, where anyone with a
shell can pass any `--by`. Per-operator authorisation still waits for
`platform-identity`, and remains D-38.

## D-40 — RESOLVED 2026-08-21 — the operator header is the CSRF defence

Loopback plus SSH lowers the risk and does not remove it: a page open in the
operator's browser is reachable by script from any other page in that browser.
The original design specified a CSRF token on every form; there are no forms now
that the interface is a separate application, so the shape changes — requiring a
custom request header is sufficient, because a cross-site HTML form cannot set
one.

**Resolved, and it landed with the first write route rather than after it.** A
cross-site HTML form cannot set a custom request header, so requiring
`X-Docket-Operator` on every write means a page in the operator's browser cannot
forge one. The header is mandatory, so a form post can never satisfy it — the
defence is the same mechanism as the attribution, which is why there is no separate
token to expire or store.

## D-41 — one connection at a time is correct and undocumented as a limit  (DOCUMENTED 2026-09-12, still a limit)

The accept loop handles connections serially, so one slow request blocks the next. That is
deliberate — it is what keeps four append-only hash chains safe without file locking — and at
three operators it is invisible.

It stops being invisible if a route ever becomes slow. Issuing twelve documents through
`pdflatex` takes seconds, and as a write route that would block every other operator's screen for
its duration with no indication why.

**The "undocumented" half is closed.** `docket serve` now says it on startup, in the place an
operator is standing when a screen appears to hang: one request at a time, a slow route blocks
the others, that is waiting rather than a crash, and the process holds the store lock so CLI
writes are refused while it runs. A limit stated only in a ledger is not stated to the person who
meets it.

**The limit itself stands, and it is not a documentation problem.** It clears when either the
slow work moves off the request — a job the console polls — or the writer is separated from the
readers so reads can be concurrent. Both are real changes to the server's shape, and neither is
worth making for three operators.

## D-42 — RESOLVED 2026-08-21 — an unfilled record is refused

`docket new <slug>` copies `people/_TEMPLATE.yaml`, which carries a realistic
example name (`preferred_name: Asha`, and a full name to match). Until someone
edits the record, that name is what the roster shows, what a person's page shows,
and what a document would print.

Found while testing `docket serve`: two freshly enrolled people, `asha` and
`vikram`, both displayed as the same person, and nothing anywhere said the record
was untouched. `docket check` does not flag it — every required field is present,
which is exactly the problem: the template satisfies the schema.

The consequence is not cosmetic. `docket onboard vikram` would issue twelve
documents naming somebody else, each with a real serial, entered in the register
and sealed into the chain. Correcting it means re-issuing under new serials,
because an issued document cannot be edited.

**Resolved.** `people/_TEMPLATE.yaml` now ships `FILL-ME` wherever a person's own
details go. `domain::person::unfilled` finds it at any depth including inside
sequences, `check` reports each one as a blocking error, and `issue` refuses.

**`--force` does not reach it.** The flag exists for judgement calls an operator
may legitimately overrule; there is no situation in which someone means to issue
documents naming nobody, so `issue::unforceable` is checked before `force` is even
looked at. Both the `onboard` and single-`issue` paths go through it.

The values left as ordinary defaults are the ones that are not personal data —
class, stage, department, supervisor, work arrangement, device type — and the
template says so, so nobody has to guess which of the two kinds a value is.

## D-44 — RESOLVED 2026-08-21 — `unrenderable` names fields, not LaTeX macros

When a record was missing something a template needed, the refusal read
`IPA needs: dkPPreExistingIp`. That is the macro name, not the field: the operator
had to know that `dkPPreExistingIp` means `pre_existing_ip` to act on it.

It was a real improvement on what it replaced — the same condition used to reach
pdflatex and come back as `Undefined control sequence` with a 500 — but it stopped
one step short of useful.

**Resolved.** `issue::unsupplied_fields` now returns record paths. The same
refusal reads `IPA needs these record fields: pre_existing_ip`.

**The mapping is built forwards and read backwards, and that is the whole point.**
`domain::template::paths_by_command` derives the command each declared field in
`registry/fields.yaml` would define — `dkP` plus `escape::command_name` of the
path — and indexes the paths by it. Taking the macro apart instead was considered
and rejected: `command_name` drops every separator, so `dkPIdentityLegalName` is
an equally faithful rendering of `identity.legal_name`, `identity_legal.name` and
`identity.legal.name`. The declared paths are the only surviving record of where
the dots were.

The `dkP` prefix is now a named constant used both to absorb the record and to
invert it, because two separate literals could drift and the symptom would be
every field reported as unmapped.

**A command no declared field derives is named as itself and marked unmapped** —
`\dkPFoo (unmapped: not declared in fields.yaml)`. Dropping it would hide a real
blocker and leave the build to fail at issuance; inventing a path from the name
would send someone to edit a field that does not exist. Every `\dkP` command in
every shipped template maps today, so the form is not currently reached.

**Two declared paths deriving one command is refused, not resolved.** That is a
registry defect rather than a reporting problem: `Definitions::absorb` would
define the command twice and the later value would silently win on a signed
document. Picking one here would hide it.

**Known limit — a sub-key of a field declared at a higher level.**
`identity.pronouns` is one declared field, so `dkPIdentityPronounsSubject` has no
declaration of its own and would report as unmapped. It does not arise today: the
templates reach pronouns through the aliased `dkPPronouns*`, which always carry a
neutral default and so are never missing. Recovering the sub-key would mean
guessing its casing back out of the macro, which is exactly what this change
exists to stop doing.

`docket next` had been doing this translation privately for its own step text
while the API and every other caller printed the macro; the mapping now lives in
one place and its local copy is gone. D-34's claim that the difference is reported
as record paths is now true of every caller, not just that one.

## D-45 — a skip guard was written on the wrong signal, and CI caught it  (CLOSED 2026-09-12)

**It happened a second time, and the second time is what closed it.** CI was red on
`the_keystore_is_never_in_what_gets_uploaded`, a test that reads two paths and asserts about a
layout. It failed because `Sandbox::new` called `crypt::generate_identity()` unconditionally, so a
fixture demanded `age-keygen` for a test that needs no cryptography at all. Green on every
developer machine, red on every runner.

**The guard belongs on the resource the test uses, not on the fixture that happens to build it.**
The sandbox now generates a key only where `age` exists; every test that uses one already calls
`skip` first.

**And the general fix is `scripts/ci-parity.sh`**, which runs the suite with `PATH` stripped to
the Rust toolchain. Both instances of this defect were discovered by pushing and reading a CI log;
that now takes seconds locally. It does not replace `gates.sh` -- green there means the code is
correct, green here means nothing depends on a tool the runner lacks.

## D-46 — Adapted templates are pinned, because deduplicating them was measured and rejected

Five officer documents were adapted from five intern ones. The concern was drift: the intern
document is amended, the officer one is not, and nobody notices until somebody signs the stale
half.

**The obvious fix was to deduplicate, and the measurement says not to.** Broken into sections
and compared across the pairs:

| | sections | byte-identical | differ only by the party term | genuinely different |
| --- | ---: | ---: | ---: | ---: |
| nda | 14 | 3 | 5 | 6 |
| code-of-conduct | 11 | 2 | 2 | 7 |
| infosec-acceptable-use | 17 | 0 | 10 | 7 |
| data-protection-notice | 9 | 0 | 1 | 8 |
| **total** | **51** | **5** | **18** | **28** |

**More than half of all sections differ in substance**, and two are inverted outright — an intern
may not speak for Paridyum and an officer may; an intern accesses what an assignment allows while
an officer's access is administrative. Even introducing a party-term macro would make only 45%
shareable, and it would put `\dkPartyTerm` into prose a person signs, so no reviewer could read
the document without mentally evaluating it.

A line-level count had suggested 71% duplication and was misleading: it counted boilerplate
(`\begin{itemize}`, shared phrasing) inside sections that genuinely differ.

**Across the whole template set, only seven duplicate section copies existed.** Five were
`Governing Law` — the same clause in five documents, two of which differed only in where the line
wrapped. That one has been extracted to `templates/clauses/governing-law.tex`. The rest are not
worth the indirection.

**So the drift is detected rather than designed out.** Each adapted template carries a
`% adapted-from: <source>@<sha256>` header, and `tests/repository.rs` recomputes it. Changing the
intern document fails the suite, naming the officer file to review and the digest to paste once
it has been. It does not compare content — the two are supposed to differ — it makes changing one
half silently impossible.

**Still open**, and smaller than it was: the review is a human one. The pin proves somebody
looked; it cannot prove they looked carefully.

## D-47 — Nothing validates `engagement.job_id` against a posting that exists

`engagement.job_id` records which posting an engagement answered. Nothing resolves it:
`tool-docket` does not read `web-paridyum`, by design — the repositories have no shared
build under ADR-0001, and a document issued today must render identically in 2036 when the
posting file is long gone.

The consequence, stated in RFC-0041 rather than hidden: **a typo in `job_id` is a wrong
recorded fact and nothing complains.** A weaker check is possible sooner — a script in the
console reading both working copies — and was not built, because a check that only runs
where both repositories happen to be cloned reports absence as failure.

Closes when postings and person records live in one store, which is ADR-0063 Phase B and
blocked on persistence.

## D-49 — Nothing checks that a class template is not gitignored  (CLOSED 2026-09-12)

`docket check` now asserts that every class in `person-classes.yaml` **with a template on disk**
has a matching `!people/_TEMPLATE-<class>.yaml` negation. A class using the generic record is not
a finding; only a template that exists can be lost to the ignore rule.

**Asserted against the `.gitignore` text rather than by shelling out to `git check-ignore`**, and
that distinction is the one this entry was originally written about. `git check-ignore` answers
whether a path is ignored *right now*, and a file already tracked is reported as not ignored
whether or not the negation exists — which is exactly how the first mutation test of this rule
passed while proving nothing. The question worth asking is whether the line was written.

The glob negation stays rejected for the reason first recorded: `!people/_TEMPLATE*.yaml` would
admit anything a person happened to name that way, and this directory is the one place in the
repository where a false negative is personal data in git history. Per-template lines,
deliberately — and this check is what makes the deliberate step a required one rather than a
remembered one.

## D-48 — Nothing detects a stale `team.json`

`docket team` exports the published officer register for `web-paridyum` to commit. The two
repositories have no shared build (ADR-0001), so if somebody edits `signatories.yaml` or
`team.yaml` and does not re-export, the public page keeps naming the old office and nothing
complains.

The web side asserts the export's shape — names upper-case as documents print them, a summary in
both locales, and that no officer appears in their own countersignature list. **It cannot assert
freshness**, because freshness is a fact about a file in another repository.

Closes with a check that runs where both are cloned, or with an export step in whatever
publishes the site. Not with a test on either side alone.

## D-50 — The estate key has no rehearsed restore, and losing it loses every vault

ADR-0069 D3: every per-person vault key is `age`-encrypted to the estate identity, so that one
key is what stands between Paridyum and its whole archive. **Losing it is the same mechanism as
erasure, invoked by accident** — and the documents it would destroy include ones with
`retention: permanent`, whose entire purpose is to be producible decades later.

RUN-0004 says where the key lives (one identity, in the password manager, `600` at
`~/.config/sops/age/keys.txt`). It does not say how a restore is proved to work, and nothing has
ever done one.

**Partly closed 2026-09-07.** `docket vault backup` puts the register, ledger, event log,
signatures and tombstones into the uploaded path encrypted to the estate key, and a test
rehearses the recovery: decrypt with nothing but that key and confirm the serial and hash come
back — which is exactly what `verify` compares a presented file against. **So the estate key
alone restores the ability to verify every document ever issued.**

**What is still missing is a rehearsal against the real thing.** The test decrypts a local file.
Nobody has fetched the ciphertext back from Sync onto a machine that has never seen the store.
Until that has been done once, "we have a backup" is a belief about the transport rather than
about the cryptography.

**Closed 2026-09-07, by making the trade explicit rather than solving it.** Crypto-shredding
needs key material that can genuinely be destroyed, so any backup of it weakens erasure by
exactly as much as it improves durability — there is no scheme that gets both, and one claiming
to has picked a side quietly. `docket vault escrow <path>` is therefore a command an operator
runs deliberately: without it erasure is provable and a lost laptop loses every vault, with it
the laptop is survivable and erasure becomes asserted. ADR-0069 D6 records both columns.

**What remains open is the operator's part**, and it is genuinely theirs: choosing whether to
hold an escrow, and if so destroying or re-exporting it after every erasure. `docket check`
warns after a shredding, and that warning is the only one in the system about a thing docket
cannot observe.

REQ-0001 (runtime secrets) is the artifact this belongs to.

## D-51 — Nothing checks that only the vault is uploaded

**Closed 2026-09-09 by `docket store push`, in the place this entry said the check belonged.**

The entry's own words were that the guard could not live in docket because docket cannot see what
Sync received, and that what could exist is *"a guard on the other side, in whatever performs the
upload, refusing any path outside `store/vault/`"*. That was right about the network and wrong
about the boundary: the thing performing the upload is a person copying files into a watched
folder, and filling that folder is an operation docket can perform and therefore guard.

`docket store push <dir>` copies `store/vault/` and nothing else, and refuses five ways:
a destination inside the store; a never-list directory **already at the destination**; an
unclassified path; a file that is not `.age`; a register with no bundle behind it. RUN-0007 is the
procedure.

**The second refusal is the one this entry was actually about.** Every other check looks at what
is being sent; that one looks at what is already there, and so it is the only thing that can catch
a `store/keys/` uploaded once, by hand, months ago — after which no erasure this system performs
is real and nothing would have reported it.

**Two defects were found by the tests rather than by review**, both worth recording because both
produced a refusal that lied. The plaintext check bailed inside the copy loop while telling the
operator "Nothing was uploaded", which was false for every file already written; validation now
runs over every file before any file is copied. And the inside-the-store guard compared paths with
`starts_with`, which is purely lexical, so a relative `store/uploads` walked straight past it and
wrote into the directory it existed to protect — and a `..`-laden path defeated the first fix too.
Destinations are now normalised lexically before the comparison.

**What remains is not a check.** The vendor's leg still cannot be observed from here: `push` fills
the folder, and whether Sync replicated it is the client's business. RUN-0007 says so in those
words rather than letting "uploaded" imply "arrived", and D-53 holds the one manual rehearsal that
would prove the transport.

## D-52 — The node registry and the off-authority write refusal are not built

ADR-0072 decides five things and code now enforces four: the store lock (D2), the identifier
uniqueness assertion (D3), the classification of every store path (D5), and restore-and-verify
(D6). **Decisions 1 and 4 are prose only.**

What that means concretely: the lock stops two writers *on one machine*, and nothing stops a
second docket being started against a restored store on the spare. That is the exact ADR-0051
collision the lock exists to prevent, reachable by the one operation a spare exists for. The
uniqueness check would find it after the fact, on the next `docket check` — which is detection,
not prevention.

The missing pieces are a node registry naming which machines may hold a store and which one is
authoritative — `tabla-*` under `device-registry.yaml`, which already has the family — and a
refusal in `writes()` when this node is not the authoritative one. The lock's holder record is
already the natural place to put the node name.

**Not built now because it needs a decision this repository cannot make**: which machine is
authoritative is estate inventory, and RFC-0021 (which governs hostnames) is still `proposed`.
Building a registry against a proposed naming scheme would bind docket to a name that may change.

## D-53 — The restore is rehearsed across machines, and not against Sync itself

ADR-0072 D6 adopts ADR-0071 §3 — *"a backup that has never been restored is a belief, not a
control."*

**What is now proven.** `rehearsal/rehearse.sh` runs four containers on a private network with no
shared filesystem between the nodes. Node A creates people, backs up, pushes the bundle to an
off-host container, and is then **deleted along with its volume**. Node B — which has never held
the store — pulls the bundle over HTTP, restores, verifies every chain, and continues the
identifier sequence at `PAR-P-000004-4`. Thirteen assertions, including four negatives: a
stranger's estate key cannot open the bundle, a second node on a shared store is refused by name,
no identifier is allocated twice, and the API answers on its own loopback while being unreachable
from another container.

This is the leg `tests/node.rs` structurally cannot reach: those tests share a kernel and a disk,
so they prove the cryptography and not that the spare needs nothing from the lost machine.

**What is still not proven, and cannot be from here: the Sync.com leg.** RFC-0038 records that the
vendor's client is Windows and Mac only, so it can never run in a Linux container. The `offhost`
container carries the same bytes over the same kind of hop, which is a stand-in and not the
vendor. One manual rehearsal remains: push a bundle to Sync, pull it down on a machine that has
never held the store, restore from that copy.

**Two assertions in the harness were vacuous when first written**, and both are recorded because
the failure mode is a green run that proves nothing. The plaintext check used `grep -v`, which is
line-based and therefore flags binary ciphertext; it now inspects the first eighteen bytes, and a
deliberately injected plaintext file makes it fail. The reachability check curled a container that
was not serving at all, so it was measuring an absent listener rather than `require_loopback`; the
server must now answer on its own loopback before its unreachability from elsewhere is counted.

## D-54 — A person record holds one engagement block, and a person may hold two

**Half closed 2026-09-12.** The resolution defect is fixed; the record shape is not.

**What was actually wrong was worse than the record shape, and it was live.**
`personnel::identify` took the LAST ledger row for a slug — correct while a person holds one
engagement at a time, and silently wrong the moment they hold two. Every per-person command read
through it, so Rohan resolved to Paridyum Cyber and his Paridyum office was unreachable by
`status`, `next`, `record`, `pack` and `onboard`. `issue` derives the company from the engagement,
so **his Paridyum NDA would have been issued on Cyber's letterhead under a `PCY/` serial** — and an
issued document is corrected only by reissuing under a new one.

Ambiguity is now refused rather than resolved by recency: `identify` errors naming both
engagements, a global `--company` selects one, and a company the person holds no office at is an
error rather than a silent fallback to the other. `docket record` lists every engagement.
Regression test in `tests/officers.rs`.

**What remains is the original entry.** `people/<slug>.yaml` still has a single `engagement:`
block — one role, one start date, one stage. Rohan is CTO of both companies so the one role is
true of both; somebody holding different offices at the two cannot be represented, and whichever
document reads `engagement.role` prints the wrong title on half of them. Nothing enforces that
constraint, which is what keeps this open.

The fix is a schema change with a real choice in it: either the record gains engagements keyed by
company — making every reader of `engagement.role` company-aware, templates included — or the
record splits per engagement and the person-level identity fields need a home that is not
duplicated across them. The second is closer to what the ledger already does.

## D-55 — The officers page names one company

`docket check` warns that `signatories.yaml` has changed since the officers page was exported.
It has, substantially: Priyanshu now holds CEO and Director, Ruhani has moved to Paridyum Cyber,
and three Cyber signatories exist that the public page has no concept of.

`registry/team.yaml` and the exported `team.json` have no company dimension, so re-exporting now
would publish a single flat list mixing officers of two companies — which is worse than the stale
page, because it would read as one company with six officers.

The web surface needs the same decision `org.yaml` just took: a company is data, and the page
groups by it. Held rather than half-done, and the warning is left standing so it is not forgotten.

## D-56 — The console resolves an ambiguous person by refusing, and offers no way through

`docket serve` has no company picker, so the HTTP routes pass `None` and a person holding two open
engagements is refused with an error naming both. That is the correct failure — the alternative is
the recency bug D-54 records — but the console gives the operator no way to answer it, where the
CLI has `--company`.

Nobody is blocked today: one person holds two engagements and the console is used for enrolment,
which opens records at the default company. It becomes a real gap the first time an officer with
two offices is administered from the console rather than the terminal.

## D-57 — Tectonic cannot render these documents, and `detect` still offers it

`typeset::detect` tries `pdflatex`, then `tectonic`. The second branch does not work:
`paridyum.cls` uses `pdfx` for PDF/A and Tectonic's XeTeX engine fails it with `CreationDate is
not properly supported`. Established 2026-09-12 by installing Tectonic and running the suite --
7 of 13 issuance tests pass, 6 cannot render.

**The fallback is therefore advertised and broken.** A machine with Tectonic and no TeX Live
detects a typesetter successfully and then fails on the sixth document with an error about a
date, which reads as a docket bug rather than as an unsupported engine.

Three ways out, none free. Drop `pdfx` and lose PDF/A, which ADR-0053's archival case wants.
Keep both engines and have `detect` refuse Tectonic with the real reason, which is honest and
leaves a branch nobody can use. Or find the XeTeX-compatible route to PDF/A -- `pdfx` has
XeTeX-specific configuration this estate has not investigated, and that is the only option that
ends with two working engines.

**Not taken now because nothing is blocked**: every machine that renders has TeX Live, and CI now
installs it. The cost is a misleading error on a machine that does not, and this entry is what
stops that being diagnosed from scratch.

## D-58 — Who may remove an officer is not decided anywhere  (DECIDED 2026-09-13, ADR-0087)

RFC-0042 decided that **appointing** an officer needs a dated instrument, a registry entry citing
it, and a human approving the change (C-0004). It said nothing about **removal** beyond the shape
of the record. The asymmetry was the defect: an appointment reaching the register without an
instrument is a `docket check` finding, and a removal reaching it without one was
indistinguishable from a correct removal.

**ADR-0087 decides it.** Governance authority is personal and ordered — Rohan Batra, then
Priyanshu Sharma — and is deliberately *not* signing authority, which stays with the office. A
removal now owes exactly what an appointment owes.

**What remains is not this entry's.** After incorporation the Companies Act and the articles govern
removing a director regardless of any internal ordering, and the founders' written consent that
would carry this pre-incorporation is the same instrument RFC-0042 §5 has been owed since August.
That is one outstanding document, not two open questions.

**Also unenforced, honestly.** `signatories.yaml` is hand-edited and docket cannot know who
approved an edit. The control is C-0004's — a human — and the evidence is the instrument.

## D-59 — Priyanshu is a single point of failure in both signing chains  (WRONG, and corrected 2026-09-13)

**This entry asserted something false and was written without measuring it.** It claimed that
removing Priyanshu would leave "Cyber with Ruhani and Rohan and nobody to countersign Ruhani's own
documents". Resolved against the real registry, that is not what happens:

    PCY, Priyanshu removed — ruhani: ["cyber-cto"]      (Rohan signs hers)
    PCY, Priyanshu removed — rohan:  ["cyber-ceo"]      (Ruhani signs his)

The chain is two deep and survives losing any one of the three. Cyber becomes unable to issue only
when two of three go at once. The entry was reasoned from the registry ORDER — he is first in both
lists — without following what the substitution actually does when the first entry is vacant.

**The property it was reaching for is now a check** rather than a claim: `docket check` errors
when a company has fewer than two *people* among its active signatories. Counted by person and not
by entry, because Priyanshu holds both `ceo` and `director` at Paridyum — two rows, one human, and
for his own documents both are the subject.

**Kept rather than deleted**, because the correction is the useful part. An entry reasoned from a
list's shape rather than from the code that reads it is a specific way to be wrong, and this one
was published in a summary before it was checked.


## D-60 — Erase is not yet honourable on the Postgres store  (OPEN, needs migration 0007)

`docket erase` rewrites the event log to its skeletal events and re-seals the chain — the one
writer that does not append (CLAUDE.md, port design §6). Behind the `Store` port this is
`reseal_events`. The file store rewrites the JSONL and re-seals; the Postgres store cannot, because
re-sealing means DELETEing chain rows and the append-only trigger (`paridyum.forbid_mutation()`,
platform-tenancy migration 0006) forbids that. `PgStore::reseal_events` therefore returns a clear
error — "erasing on the Postgres store needs migration 0007 (append-only delete exception)" — rather
than silently failing the trigger.

The fix is a **cross-repo** platform-tenancy migration: a narrowly-scoped, `SECURITY DEFINER` delete
exception (or a per-table erase function) that is the only path permitted to remove chain rows. It
must NOT be made from this repo. Until then, an erasure against a Postgres-backed instance is honoured
by running `docket erase` through the **break-glass CLI on the file store**, which still re-seals
correctly. Threaded through the port so the file path keeps working today.

## D-61 — The CLI is file-store only; a Postgres-backed CLI is not wired  (RESOLVED 2026-09-14)

ADR-0093 §3 makes the CLI a break-glass client of the same database: `FileStore::new(&paths)` offline,
a `PgStore` when `DOCKET_DATABASE_URL` is set. The first half was wired earlier; this change wires the
second and closes the debt.

**What changed.** Store construction moved out of the ~25 per-command handlers and into `run()`, which
now builds the backend ONCE and threads a `store: &dyn Store` down through `dispatch` → `route`
(→ `route_person`) → each handler. `open_backend` chooses by environment: with `DOCKET_DATABASE_URL`
set it builds a multi-thread tokio runtime kept alive for the process (as `bin/serve` does), opens the
pool on it, reads the tenant from `DOCKET_TENANT_ID` (a uuid — refused clearly if unset or unparseable
while the DB URL is set, because every row is tenant-scoped by the migration-0006 RLS policies and
guessing the tenant is how one operator's break-glass write lands against another's records), and
constructs `PgStore::new(pool, tenant, handle)`; unset, it is the offline `FileStore` exactly as before.
Either way it prints one stderr line naming the store — and, for Postgres, the host and tenant, with
credentials stripped — so an operator knows they are hitting the cluster, not local files.

**The write lock is file-store-only.** `lock::acquire` is skipped on the Postgres backend: the
database's advisory-locked, atomic seal-append is the concurrency control there (ADR-0093
§decision-2), and a lockfile beside a database store would leave a stray file and serialise nothing.

**What still cannot run on Postgres, and now fails clearly rather than obscurely.** `advance` rewrites
`people/<slug>.yaml` as text to preserve its DPDP field-purpose comments (D-62), and `erase` re-seals
the event log by rewriting it (D-60, needs the cross-repo migration 0007). Both are refused up front by
`unavailable_on_pg` with the reason and the remedy (run offline against the file store), so `advance`
no longer dies deep in an `fs` read and `erase` no longer deletes host-local blobs before its re-seal
fails. Both work unchanged on the file store.

**Proven.** `scripts/gates.sh` green (file mode); `--test pg_store` 7/7 against the throwaway Postgres;
and the CLI itself against that database (a seeded tenant) — `docket roster` returns cleanly and
`docket new asha-verma intern` lands the person in `docket_persons` with its sealed identity and
`created` event in `docket_identities`/`docket_events`, all through the port, no file written to
`people/`; `advance` and `erase` refuse with their D-62/D-60 messages. The break-glass loopback
`docket serve` and `--tui` browse both run on either backend.

## D-62 — `advance` still edits the person body on the filesystem  (OPEN, served stage-change)

`personnel::advance` reads and rewrites `people/<slug>.yaml` as TEXT (via `fs`), deliberately, to
preserve the field-purpose comments the DPDP Act cares about (the record is not re-serialised). That
is correct on the file store and is why `advance` keeps its `&Paths`. On the Postgres store the person
record lives in `docket_persons`, not on disk, so `advance` — and only `advance` — cannot change a
stage there yet: it would look for a file that is not present. Every other person write already goes
through the port (`create_person`/`rewrite_person`). The served backend's stage-change therefore waits
on a port operation that returns the raw person body for a comment-preserving edit (or a decision that
the JSONB record needs no comment preservation). Not exercised by the served happy-path test; called
out rather than hidden. Same family as the blob limitation (port design §8): the served instance
shares the *record* immediately, some host-local edits later.

## D-63 — The loopback `docket serve` is now thread-per-request over the file store  (OPEN, low risk)

ADR-0072's single-request serialisation — which is what kept the four append-only file chains safe
without a file lock — was retired here because ADR-0093 §decision-2 moves write concurrency into the
database: `adapters::http::server` is thread-per-request, and correctness under concurrent writers is
the `Store` port's atomic, advisory-locked seal-append. That reasoning holds for the primary served
backend (`bin/serve`, a `PgStore`). The break-glass loopback `docket serve` (CLI subcommand, a
`FileStore`) now also serves thread-per-request, so two concurrent writers to its file chains are no
longer serialised by the server. It is a single-operator break-glass/dev path over SSH, so the
exposure is small, but it is a real change from ADR-0072 and is recorded as such. The clean fix is for
the loopback path to use the same database (D-61) or to reinstate a file-store write lock behind the
port.

## D-64 — docket is class `platform` but still named `tool-docket`

ADR-0093 §3 reclassifies the served backend as platform-shaped, and `manifest.yaml` now declares
`class: platform` so that ADR-0020's deployable obligations (the Helm chart, probes, limits added
2026-09-14) actually bind. But the repository is still named `tool-docket`, whose `tool-` prefix maps
to the `tool` class under C-0005 and C-0007. The two disagree.

Nothing fails today: `repo.name_matches_scheme` is forge-owned and validates the NAME against the set
of registered prefixes (which `tool-` is), not the manifest `class`; and the class↔stack gate passes
because `platform` permits `rust`. So the disagreement is latent, not breaking.

The clean resolution is to rename the repository to `platform-docket` — but that is a breaking
operation (git remote, the org repo, every vendored `.paridyum/` snapshot, and console-docket's
references to it) that ADR-0093 does not schedule, and it is not something to do as a side effect of
adding a chart. **Reopens when:** the rename is scheduled, or forge ships a check that binds a
manifest's `class` to its repository-name prefix — whichever is first. Sibling debt: console-docket
D-? (its manifest is now `class: web` while named `console-docket`).

## D-65 — API mode needs a Zitadel device-flow public client that does not exist yet  (OPEN, blocks the API path)

ADR-0097 §3 gives the CLI `docket login` over the OIDC device authorization grant. It authenticates as
a PUBLIC client (no secret, RFC 8628), whose id the operator supplies as `DOCKET_OIDC_CLIENT_ID` —
there is deliberately no default, so an unset one is refused up front rather than failing obscurely at
the issuer. That client must be registered in paridyum-operations (control plane, `id.paridyum.com`)
as a device-flow public client, with the `…:aud` scope reserving the paridyum-operations project so the
token it mints is audienced to match `auth.rs` (ADR-0092). **Until it exists, API mode cannot be used**
even against a deployed backend; the file and database modes are unaffected. **Reopens/closes when:**
the client is created and its id is handed to operators (and put in the SOP task). Sibling of the
deployment gate ADR-0097 already records as an accepted cost (the backend must be served at an ingress).

## D-66 — API mode deferrals: refresh-token healing, sign-scan upload, interactive-new validation parity  (OPEN, low risk)

Three things ADR-0097's first slice deliberately does not do:

- **Refresh-token healing.** `docket login` caches a refresh token when the issuer returns one, and
  `token.rs` keeps it, but nothing uses it: a mid-session expiry is met with "run `docket login`", not a
  silent refresh. A later slice can exchange the refresh token for a new access token without a second
  browser round-trip.
- **`sign --scan` over the API.** Recording a wet-ink signature needs the returned scan filed on the
  server's disk; there is no upload endpoint, so `sign --scan` is refused in API mode. Record the scan
  offline (file store or database), or add a multipart upload route later.
- **Interactive `new` validation parity.** `docket new -i` over the API fetches the server's form
  (`GET /api/form/{class}`) and asks its questions, but the answers are only validated when the server
  writes the record — the local interview's inline per-field checks (`console::ask` closed-choice
  enforcement) are not fully mirrored client-side. A partial/invalid set is caught by the server's
  enrol refusal, not at the prompt. Close by having the form carry enough to validate client-side, or by
  a `POST /api/form/{class}/validate` preflight.

**Reopens when:** any of the three is scheduled. None blocks the everyday API path.

## D-67 — Already-issued filesystem PDFs are not migrated into Garage  (OPEN, data move)

The blob port (D-66) makes new issuances land in Garage on the served backend, but every PDF, pack and
scan issued BEFORE the cutover still lives only on whatever filesystem issued it (a break-glass CLI
host, or an earlier served pod's ephemeral disk if any survived). Their register entries already record
the blob KEY (`pdf_path`) unchanged, so a served `verify`/`pack`/`vault seal` over Garage will report
those bytes missing until they are copied in.

This is a one-time DATA move, deliberately separate from the code change (governance precedes
implementation; a data migration is its own reviewed step): enumerate each register entry's `pdf_path`
and each signature's `scan_path`, read the bytes from the source filesystem, and `Blobs::put` them into
the tenant-scoped bucket under the SAME key. It needs a source host that still has the files and the
Garage credentials; it is not runnable from this repo's test loop. **Reopens when:** Garage is deployed
(ADR-0094) and a host holding the historical `store/` is available to run the copy. Until then, the
break-glass CLI on that host (FileBlobs) still reads them; only the served backend cannot.

## D-68 — Sealed vault objects and keys stay on `Paths`, not in the blob store  (OPEN, low risk)

The blob port moved the PLAINTEXT bytes (issued PDFs, packs, scans) to Garage, and `vault seal` now
reads them back through it to encrypt. But the sealed `.age` vault objects and the per-person age keys
still live on the filesystem under `store/vault/` and `store/keys/`, via `Paths`/`fs::` directly. Two
reasons this was left: the key MUST be destroyable locally for `shred` to be real erasure (ADR-0069),
and ADR-0094 §2 says ciphertext is what may sit in the bucket — which is exactly what a `.age` object
is, so moving them is defensible but is a design step, not a mechanical one. Consequence: on the served
backend a vault sealed on one host is not visible on another until this is addressed; today `vault seal`
is a break-glass CLI operation, so the exposure is small. **Reopens when:** vault sealing is offered on
the served backend, or the estate decides `.age` objects belong in Garage (encrypt-before-bucket flow,
ADR-0094 §2), whichever is first.
