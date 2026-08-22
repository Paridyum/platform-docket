# Changelog — tool-docket

Format: [Keep a Changelog 2.0.0](https://keepachangelog.com/en/2.0.0/) ·
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **A document can declare itself `on_request`**, meaning it is issued when somebody asks
  rather than owed by a lifecycle stage. `docket check` no longer warns about one, and now
  **refuses** a document that is both `on_request` and owed by a stage.

  PCK, LTR and PUB were each already carrying a `note` saying the warning was intended — so
  `check` reported three warnings on every clean run, for years of runs to come. A warning
  list that is never empty is one an operator learns to scroll past, which is exactly how the
  next genuine orphan would have hidden among the three known ones. It reports 0 errors and
  **0 warnings** now.

  The new refusal is a real control, not tidiness: PUB is consent to use a person's name or
  photograph, which the DPDP s7 employment legitimate use does not cover. A consent a stage
  *requires* is not freely given, so a stage quietly coming to owe it is the mistake worth
  blocking.

- **`fields.yaml` declares `choices` and `default` per field**, and both interfaces
  offer them. A field whose declared set is exactly `true` and `false` is a flag, which
  is how the record decides to write a bare boolean rather than the string `"true"`.

  These two facts already existed — as `is_flag` and `default_for`, two hardcoded
  `match` arms in `application::enrol`. They were a second statement of the registry's
  vocabulary and only the CLI could read them: `docket new --interactive` offered a
  default and a true/false answer, while the console's enrolment wizard, generated from
  the same catalogue, rendered a free-text box for the same field. Both matches are gone
  and both interfaces read the registry, so they cannot drift again.

  `GET /api/form/{class}` sends both. It stays silent about types it could only infer —
  `engagement.start_date` looks like a date and the API says nothing, because whether
  "on joining" is a valid answer belongs here and not to a browser.

- **`docket check` refuses two ways a declared set goes quietly wrong.** A `default`
  outside its own `choices` is blocking — the interview writes it on a blank answer, so
  every record opened that way would be wrong from the first field. And
  `engagement.stage` offering a stage `lifecycle.yaml` does not declare is blocking,
  because a record opened at it would sit at a stage no gate can ever match. The reverse
  — a declared stage nobody offers — is a warning, since not being able to open a record
  at `alumni` is a reasonable thing to decide on purpose.

### Added

- **Write routes on `docket serve`,** so the console can do the job rather than
  only describe it: `POST /api/person` (enrol), `/onboard`, `/sign`, `/advance`,
  plus `GET /api/form/{class}` for the enrolment questions.

  **Every write resolves an operator or is refused with 403.** An
  `X-Docket-Operator` header, checked against `registry/operators.yaml`. This is
  attribution exactly as strong as the CLI's `--by` — a claimed identifier that
  must name a real operator, proven by SSH rather than by us — and it is refused
  rather than defaulted, because a default is what made every event in the log say
  "docket". It doubles as the CSRF defence: a cross-site form cannot set a custom
  header, so the token the original design specified is unnecessary.

  **A refusal is 409, not 400 and not 500.** 400 means the request was wrong, 500
  means we are broken, 409 means it was understood and the domain declined. Only
  the third is information an operator can act on, and flattening them is how a UI
  comes to render a refusal as an error and invite a retry. Refusals carry the same
  `blockers` shape the person page renders, so a refused write and a held person
  read identically.

  **The routes do not expose `--force`.** `onboard` cannot be forced at all from
  here; §8 forbids an interface that offers a way past a refusal, and a console
  button that issues defective documents is worse than no button. `advance` accepts
  `force` because the CLI has it and the boundary mirrors the CLI, but it defaults
  off, so a console that does not send it can never skip a gate.

  `sign` checks the serial was issued to the person in the URL. Without that a
  request could file a signature against anybody's document through anybody's URL,
  and the signature ledger would be correct while the audit trail lied.

- **`docket serve` — the operator API `console-docket` consumes.** Read-only:
  `/api/health`, `/api/roster` (with `?standing=`), `/api/person/{slug}`,
  `/api/check`, `/api/retention`, `/api/emails/{slug}`. Every route calls the
  same `application::` function the CLI calls, so the console and the command line
  cannot disagree about what a person owes, and **no route writes anything** —
  asserted by a test over every route rather than argued from the absence of write
  handlers.

  **Binds `127.0.0.1` and refuses anything else**, as a function with tests rather
  than a default. There is no login: SSH keys are the authentication and the
  operating system is the identity (ADR-0047 gives this tool no identity system of
  its own, so it must not be reachable without one). Verified live that the
  listening socket is `127.0.0.1` only and that the LAN address refuses the
  connection — the property `next dev` in `console-docket` fails.

  The HTTP implementation is ours, on `std::net`, adding **no dependency**. The
  obvious crate for the job last published in 2022, and the needed surface is one
  client over loopback with no TLS and no keep-alive. It is strict by choice:
  chunked encoding, absolute-form targets, HTTP/2 and unparsable `Content-Length`
  are refused rather than guessed at, with bounds on the request line, header
  count and body size, and a read/write timeout so a silent client cannot hold the
  accept loop.

  Connections are handled **one at a time, deliberately**. This process is the only
  writer to four append-only hash chains, and that is what makes them safe without
  file locking; a thread per connection would make two concurrent writes possible
  and a torn chain the failure.

  Wire types live in `adapters::http::dto` rather than `#[derive(Serialize)]` on
  the `application::` types, so renaming an internal field cannot silently rename a
  field in a published API.

- `docket` CLI in Rust: `status`, `onboard`, `issue`, `render`, `verify`, `check`.
- `latex/paridyum.cls`, the single typesetting authority. Absorbs the
  letterhead, boxes, footer, page numbering and signature blocks that were
  duplicated across eleven separate `.tex` files.
- Fourteen document templates. Ten transcribed from the `.docx` originals,
  which are the source of truth for content.
- Four documents that did not exist: POSH policy, Data Protection Notice,
  Internship Offer, and an intern acknowledgement on the offboarding
  checklist.
- `registry/` — `org`, `signatories`, `lifecycle`, `retention`. One home per
  fact; `check` fails any template that restates one.
- Tamper-evident register: hash-chained JSONL, outside git so that erasure
  remains possible.
- Personnel record keeping: `docket new`, `record`, `roster`, `event`,
  `separate`, `erase`. Permanent identifiers (`PAR-INT-2026-0001`), never
  reused, including after erasure.
- `registry/person-classes.yaml` and `registry/fields.yaml`: 3 person
  classes, 54 declared fields, each with a stated purpose and a store.
- Class-gated collection. An unpaid student intern is not an employee, so
  no statute requires their PAN, Aadhaar, UAN or bank account; the `intern`
  class forbids them and `check` refuses a record that holds one.
- Hash-chained identity ledger and personnel event log, alongside the
  document register. The chain logic is one implementation shared by all
  three.
- DPDP erasure that actually deletes: the record file and the person's event
  detail go; the identifier, class and dates survive as a tombstone so
  issued serials still resolve.
- 96 tests.
- Vector logo, built from source at `latex/assets/paridyum-logo.tex`.
- `identity.date_of_birth` required for interns, a `guardian` field group,
  and `GRD` (Parent / Guardian Consent) owed whenever the person is under
  eighteen on their start date — DPDP s2(f) and s9.
- Verification code in the footer of every issued document, derived from
  serial + recipient + issue date + template hash. `docket verify`
  recomputes it. A lookup check, not a signature.
- The offboarding checklist now names the systems the event log records as
  still granted, and lists the documents on record with their serials. Both
  derived, so the paper cannot disagree with the register.
- **Two identifiers (ADR-0051).** `PAR-P-000001-X` is opaque, permanent and
  names the archive folder; `PAR-INT-2026-0001` carries class and year and
  names the engagement. Class is mutable for a person and immutable for an
  engagement, which is the whole distinction.
- ISO 7064 MOD 11-2 check character on the person identifier, because it is
  typed into folder names and read aloud. Catches every single-digit error
  and every adjacent transposition; both properties are swept in tests.
- `docket reengage` — an intern who is hired keeps their person identifier
  and gets a new engagement identifier, so the archive stays whole.
- The register keys on `person_id` + `engagement_id`. A slug is a filename
  and nothing keys off it.
- Issued PDFs are stored under the person identifier, not the slug: a folder
  named after a person discloses who Paridyum engages from the listing alone.
- **PDF/A-2b (ISO 19005-2)** via `pdfx`, with the sRGB OutputIntent the
  standard requires. Output is still byte-identical.
- **PDF metadata** — `/Title`, `/Author`, `/Subject`, `/Keywords` written from
  a generated `.xmpdata`. A file pulled out of the store is no longer
  anonymous.
- `docket index` — generated `roster.csv`, `documents.csv`, `README.txt` and a
  BagIt-style `manifest-sha256.txt`. The archive verifies with plain
  `shasum -a 256 -c`, with no Paridyum tooling. Generated output under C-0004;
  `docket check` fails if it is stale (ADR-0040's pattern).
- `docket retention` — reports documents past their period, and reports classes
  whose prose period it cannot parse rather than assuming them compliant.
- `docket erase` now names what it could NOT reach: the cloud folder, backups,
  mail, printouts. A tool that reports a complete erasure it did not perform
  is worse than one that states its boundary.
- `access.cloud_folder` field, so erasure can name the remote folder.
- `SOP.md` (STD-0013), six tasks with real pasted output.
- `provides` block in `manifest.yaml` — required by `manifest.schema.json` and
  absent before.
- `docket sign <serial>` — records that a document came back executed, in its
  own hash-chained ledger. Never by mutating the issuance entry: the register
  is append-only so what was issued cannot be rewritten, and a signature is a
  later fact.
- Refuses wet-ink with no scan (the scan is the only evidence that method
  leaves), a signing date before issuance, a received date before signing, a
  second signature for the same serial, and an unissued serial.
- Scans are copied into the store and hashed. `docket check` reports a scan
  that has gone missing or no longer matches its hash.
- `docket advance <slug>` — moves a person to the next stage and **refuses**
  while the stage's `requires_signed` documents are merely issued. That gate
  was prose in `lifecycle.yaml` and nothing checked it, which is how access
  gets granted before the NDA comes back. `--force` overrides and says so.
- `docket status` now distinguishes issued (`[~]`) from executed (`[x]`) and
  lists what is awaiting return. "Nothing outstanding" previously meant
  *issued*, which is not the question onboarding turns on.
- Signature state is a column in `documents.csv`.
- `tests/signing.rs`, 16 cases.
- `docket pack <slug>` — assembles a person's documents into a folder grouped
  by **what they must do with each**: sign and return, complete and return, for
  your records, optional. Derived from `lifecycle.yaml`, so a new document lands
  in the right folder without editing a list. Ships a `MANIFEST-sha256.txt` that
  verifies with plain `shasum -a 256 -c`, and is a copy — the store stays
  authoritative.
- **PCK** Read Me First — a cover sheet generated from the register, so it
  cannot describe a pack it was not built from. Says explicitly not to sign
  anything unread, and lists where to take each kind of question including the
  one that must not go through the supervisor.
- **INS** Institution Confirmation, **PUB** Publicity Consent (owed by no stage,
  deliberately — a consent a stage requires is not freely given), **COC** Code of
  Conduct, **GRV** Grievance Policy, **EXF** Exit Feedback.
- COC and GRV join the preboarding signature gate.
- `retention.yaml` classes now declare `from:` — the epoch a period is counted
  from.
- `.github/workflows/ci.yml` calling `infra-ci`'s reusable workflow, and the
  vendored `.paridyum/knowledge/` snapshot ADR-0001 requires and the workflow
  reads. `check-repo.sh` now passes.
- `CLAUDE.md`, `.claude/settings.json`, `.gitattributes`, and the `Not tested`
  / `Quarantine` / `Evidence` sections STD-0008 requires in `TESTING.md` — all
  four were missing and all four were caught by the repository check.
- `registry/communications.yaml` — the seven messages Paridyum sends, each
  naming its documents by code, so a message cannot reference a document the
  lifecycle does not have. `docket check` verifies it.
- `docket emails <slug>` — renders the messages due at a person's stage with
  placeholders filled and attachments resolved to serials. Reports an
  attachment that has not been issued and exits 1. Does not send mail.
- `HR-CHECKLIST.md` — one page, 34 checkboxes, every step a command.
- **Operator identity.** `--by <id>` or `DOCKET_OPERATOR`, validated against
  the new `registry/operators.yaml`. Every write records who made it; a write
  without an operator is refused rather than defaulted. Before this, every
  event in the log said `recorded_by: "docket"`, so the trail could say access
  was granted and never by whom — and that is not fixable in retrospect.
- **Tectonic as a second typesetter (ADR-0054).** `pdflatex` is preferred when
  present; Tectonic is used when it is not. Neither replaces the other. Each
  register entry now records `engine` and `engine_version`.
- `docket next <slug>` — the single next thing to do and the command that does
  it, ordered so that following it from the top requires no knowledge of the
  lifecycle. Exit 0 only when nothing remains.
- `tests/engines.rs` — verifies the Tectonic invocation with a shim, since
  Tectonic is not installed here: a wrong flag would otherwise surface only on
  the machine that has no pdflatex, which is the machine least able to debug it.
- `docket new --interactive` — asks the questions instead of handing over a
  YAML file. Prompts are generated from `registry/fields.yaml`: only what the
  class permits, only what belongs in the personnel store, the stated purpose
  as help, required first with an offer to stop there. Writes a
  self-documenting record that passes `check` unedited.
- `adapters/console` — prompt and confirm, with the closed-input case handled
  so a required question cannot spin forever.
- `docket --tui` — full-screen, read-only: roster, then a person's documents
  with signature state and their `next` steps. **No new dependency.** Reading a
  keypress needs raw mode, which needs `unsafe`, which is `forbid` here, so
  line-based navigation is the only permitted shape — and it avoids fourteen
  transitive crates. Read-only by construction: no input reaches a write path.
- `adapters/screen` — clear, heading, and character-counted fit/pad so a
  non-ASCII name is not cut mid-codepoint or thrown out of column.
- `issue::unrenderable` — reports fields a document prints that the record does
  not supply, as record paths rather than TeX commands, before issuing is
  attempted. Surfaced in `docket next`.

### Changed

- Person records now use the field paths declared in `fields.yaml`
  (`identity.legal_name`, `engagement.role`) rather than the ad-hoc shape
  the first version grew. Templates and the record template were migrated
  together.

### Fixed

- **An unedited record was issuable, and looked filled in.** `docket new` copied a
  template carrying realistic example values, so two freshly enrolled people
  displayed as the same person and `docket check` reported nothing — every required
  field was present. `docket onboard` would have issued twelve documents naming
  somebody else, each with a real serial sealed into the chain, correctable only by
  re-issuing under new serials.

  The template now ships `FILL-ME` wherever a person's own details go;
  `domain::person::unfilled` finds it at any depth, including inside sequences;
  `check` reports each as blocking; and `issue` refuses. **`--force` does not reach
  it** — the flag is for judgement calls, and nobody means to issue documents
  naming nobody. Values that are not personal data keep ordinary defaults, and the
  template says which is which so nobody has to guess.

- **`enrol::to_yaml` silently discarded answers to top-level fields.** Any field
  whose path has no dot — `pre_existing_ip` is the one that exists — was dropped by
  a `split_once('.')` with a bare `continue`. An operator answered the question in
  `docket new --interactive`, the answer went nowhere, and the only symptom arrived
  much later as `IPA needs: dkPPreExistingIp` from pdflatex: a macro name for a
  field they had in fact supplied. So the single field on the IP assignment
  agreement stating what an intern already owns was unanswerable through the
  interview.

- **A record generated by `docket new --interactive` could not be advanced.**
  `personnel::advance` searched for the literal substring `stage: {from}`, and
  `enrol::to_yaml` writes the stage quoted — `stage: "preboarding"` — so the search
  missed and the first `advance` failed with "could not find `stage: offered`",
  which was true and useless. A record with no stage line at all, which is what
  skipping the question produces, failed the same way.

  Now rewritten line by line, handling plain, quoted and trailing-comment forms and
  writing the line when it is absent. Fixing only the absent case would have been
  worse than fixing neither: inserting into a record that already had a quoted
  stage left **two** `stage:` keys in one mapping, and the record then meant
  whichever the parser kept. That is not hypothetical — it is how the bug was
  found, with the signing suite failing on `duplicate entry with key "stage"`.
  The trailing comment is preserved, because it lists the valid stages.

- **Onboarding an incomplete record reached pdflatex and returned a 500.** The API
  now runs `issue::unrenderable` before rendering anything and refuses with 409
  naming what is missing, rather than surfacing `Undefined control sequence` as a
  server error. Found by enrolling through the API with a partial answer set, which
  is exactly what a half-completed form sends.

- **Issuance was never recorded as an event.** `Kind::DocumentIssued` existed
  and was emitted nowhere, so a person's history had a hole where their
  documents should be.
- **`Entry.signed` was dead weight.** Declared `Option<Signature>` and never
  populated, so the register implied it tracked signature and did not. Removed;
  proven first that a `skip_serializing_if = "None"` field cannot change an
  existing entry's digest.
- **The Internship Offer was treated as needing no signature back**, because
  the rule keyed only on `kind: issued`. It carries an acceptance block the
  person signs. `needs_countersignature` now also honours `countersigned_by`.
- `templates/rust-service/manifest.yaml` in the knowledge repository was
  missing the required `provides` block — so the knowledge gate was failing and
  every repository scaffolded from it inherited an invalid manifest.
- `tests/signing.rs` took 51s because each case issued a nine-document pack.
  The default fixture now issues one document; the two gate cases ask for four.
  10s.
- The class's `draft` option leaked to `graphicx` as a global option, so a
  draft rendered a box containing the filename instead of the logo. Loaded
  with `[final]`.
- Pronouns recorded in a person record were silently ignored: the record
  nests them under `identity.pronouns`, generating `\dkPIdentityPronouns*`,
  while templates read `\dkPPronouns*`. The neutral default always won, so a
  record saying `she/her` printed "their". Aliased before defaulting.
- `identity.date_of_birth` rendered as ISO while every other date was long
  form, so a legal document carried two date formats.
- The offboarding checklist asked for a last working date the record already
  held, and broke lines so checkboxes were orphaned from their labels.
- Non-deterministic output: the PDF trailer `/ID` is derived from the wall
  clock by pdfTeX. `SOURCE_DATE_EPOCH` does not cover it; `\pdftrailerid`
  does.
- `SOURCE_DATE_EPOCH` now uses UTC midnight rather than local midnight, so
  output no longer depends on the operator's timezone. The archived Python
  implementation had this bug.
- Multi-page documents had no letterhead and no page numbering after page 1,
  so a page could be inserted or removed unnoticed. Every page now carries
  `Page X of Y` and the reference number.
- The `Page X of Y` line rendered at 845.8pt on an 845.0pt page and was
  silently dropped. The footer block is now anchored to grow upward.
- `\titleformat` targeted the numbered `\section`, which no document uses.
- **Retention measured from the wrong date.** Every period is stated relative to
  the END of the engagement — "8 years after the internship ends", "30 days of
  the last working day" — and all of them were measured from the document's
  ISSUE date. Wrong by the length of the engagement, and for the emergency
  contact actively harmful: 30 days after issue falls mid-internship, so the
  tool would have advised deleting a contact that was still needed.
- `tool-forge/manifest.yaml` declared `lifecycle: alpha`, which is not in the
  schema's enum, and had no `provides` block. Both required.
- **A wizard record could not issue.** Answering only the eight fields the
  `intern` class requires left the Internship Offer failing with `Undefined
  control sequence`, naming a TeX command rather than a field. The two lists
  are both correct and are not the same list — `requires` is about lawfulness,
  a template's needs are whatever it prints — so they are now checked
  separately. See DECISION-DEBT D-34.
- **The storage split was declared and unenforced.** `person-classes.yaml`
  declares four stores and the reason for each, yet third-party emergency
  contact details could sit in the personnel record and `check` passed with
  zero errors. `validate` now reports a field declared to live elsewhere.
  Forbidden is still checked first: for an intern's PAN, "no statute requires
  this" is the reason worth telling someone, not "wrong file".
- The interview asked date of birth before name, because paths sorted
  alphabetically. Small, and the kind of wrongness that makes a tool feel like
  a form.
- `personnel::create` refused to run after `--interactive` had written the
  record. The ledger decides whether a person is new, not the file.

Supersedes the Python implementation, archived at
`archived/docket-python-2026-08-19/`. Output verified byte-identical.
