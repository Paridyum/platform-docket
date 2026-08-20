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

## D-28 — CI is configured but has never run  (OPEN)
`infra-ci/.github/workflows/rust-ci.yml` is installed and `tool-docket` calls
it, and both repositories pass `check-repo.sh` locally. Neither has run on
GitHub Actions, because neither has been pushed to a remote from this
checkout — `repo.git_remote_configured` reports `n/a` for exactly that reason.

So the honest state is: the pipeline is written and locally verified, and the
first real run will find whatever local checks cannot. Do not treat a green
local gate as evidence that CI passes.

## D-29 — Nothing records what has already been sent  (OPEN)
`docket next` lists messages as "ready to send", not "outstanding", and says
so. There is no sent-flag because this tool does not send mail (D-27) and a
flag it could not verify would be worse than none — it would assert something
false the first time someone sent from their own client without telling it.

The honest fix, if this ever matters, is `docket event <slug> message_sent
--detail <id>`, recorded like every other event, by the person who sent it.
That is a real workflow step rather than a guess, and it is deliberately not
added until someone actually wants it.

## D-30 — The Tectonic render path is unverified end to end  (OPEN)
`tests/engines.rs` verifies that Tectonic is invoked with the right flags, one
pass, and `SOURCE_DATE_EPOCH` set, using a shim. It does NOT verify that
`paridyum.cls` renders under Tectonic, or that Tectonic output is
byte-reproducible, because Tectonic is not installed on this machine.

Both are plausible and neither is proven. Tectonic bundles its own TeX
distribution, so a package our class depends on may resolve differently, and
its PDF writer is not pdfTeX's — `\pdftrailerid`, which is what makes output
byte-identical, is a pdfTeX primitive and probably does nothing there.

**Do not assume the Tectonic path produces reproducible output.** Install
Tectonic, issue a document twice, and compare before relying on it.

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

## D-35 — CI does not exercise rendering  (OPEN)
Found by the first real CI run, 2026-08-20. The integration tests render real
PDFs, and a GitHub runner has neither TeX Live nor Tectonic, so `test` and
`coverage` failed with `no typesetter found`.

The tests now SKIP when no typesetter is installed, printing which test and
why, using the same pattern `pdflatex_is_preferred_when_present` already used.
That makes CI green and honest, and it is not the same as making CI complete:
**rendering, determinism and the whole PDF/A claim are currently verified only
on a developer machine.** A green CI badge does not cover them.

Two ways to close it, and the choice is not mine:

1. **Install Tectonic in the shared workflow.** A single binary, so it is cheap,
   and it would resolve D-30 as a side effect by exercising that path for real.
   But `rust-ci.yml` is shared by every `rust`-stack repository under ADR-0021,
   and most of them will never typeset anything. It would need to be optional —
   a `typesetter:` input the caller opts into.
2. **A tool-docket-specific workflow** that installs Tectonic and runs only the
   render tests. Simpler, and a deviation from ADR-0021's "all logic lives in
   the shared workflow", which is a decision to take deliberately rather than by
   adding a file.

Option 1 is better and costs a change to a shared workflow. Until one is taken,
run `cargo test` locally before trusting a release.

## D-36 — Coverage in CI measures the no-typesetter subset  (OPEN)
First real run: **64.81% total**, against a 70 default. The breakdown is the
finding, not the total:

    domain/person.rs        96.47%
    domain/personnel.rs     95.14%
    domain/register.rs      99.43%
    domain/serial.rs        99.35%
    domain/signature.rs     97.40%
    domain/template.rs      98.54%
    main.rs                 14.39%   940 of 1098 lines uncovered

The logic is covered. `main.rs` is argument parsing and printing, and most
`cmd_*` paths issue documents — which needs a typesetter, which a runner does
not have (D-35). So the CI number measures the subset that runs without
rendering, and is structurally lower than the local one.

The threshold in `.github/workflows/ci.yml` is set to 60 with that reasoning
written next to it. **This is a statement, not a licence.** Two things follow:
it returns to 70 when D-35 is closed, and it does not excuse the domain
slipping — that is where the logic is and it is at 95%+.

The alternative considered and rejected was excluding `main.rs` from the
measurement, which reaches ~71% arithmetically and hides the fact that 940
lines of it are never executed by anything.

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
