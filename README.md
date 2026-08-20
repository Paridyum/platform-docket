# tool-docket

Paridyum's internship document system. Fourteen documents, one typesetting
authority, one home for every fact, and a register that makes an issued
document verifiable years later.

Rust, class `tool`, binary `docket` (C-0007). Requires TeX Live with
`pdflatex` on `PATH`.

## One verb

    $ docket next asha-verma
    1. Issue 12 document(s): OFR INS AGR NDA IPA ISP PSH COC GRV DPN ONB EMC
       why: owed at stage "preboarding"
       run: docket onboard asha-verma

Run it, do what it says, run it again. The ordering is the workflow: a record
that cannot lawfully be issued against blocks everything above it, documents
come before packs, packs before chasing signatures, signatures before the stage
gate. Exit 0 only when there is nothing left, so it can drive a loop.

It does **not** suggest advancing out of a stage with no signature gate. An
empty gate means nothing blocks the move, not that you should make it — and
suggesting it from `active` would tell you to offboard someone on their first
day.

## Enrolling an intern

    cargo build --release
    export DOCKET_OPERATOR=rohan     # who is doing this; recorded on every write
    docket new asha-verma intern -i  # asks the questions, writes the record
    #   or, without -i: copies the template for you to edit by hand
    docket check                     # catches an incomplete or over-collected record
    docket onboard asha-verma        # issues everything the stage owes
    docket pack asha-verma           # builds the folder you hand over

Then, as signed copies come back:

    docket sign PAR/NDA/2026/0001 --method wet-ink --on 2026-08-25 --scan nda.pdf
    docket advance asha-verma        # refuses while any agreement is unsigned

`SOP.md` has the whole sequence with real output and a failure table per step.

## What to send, and when

`registry/communications.yaml` holds the seven messages Paridyum sends — offer,
pack, guardian, institution, access, offboarding, completion — each naming the
documents it carries **by code**. `docket emails <slug>` renders the ones due at
that person's stage, with the placeholders filled from the record and every
attachment resolved to a real serial:

    [pack]  stage preboarding  ->  intern
    WHEN     On receiving the signed acceptance. Not before: ...
    TO       asha.verma@example.com
    SUBJECT  Your Paridyum internship documents — please read before signing
    ATTACH   NDA  Non-Disclosure Agreement       PAR/NDA/2026/0001
    ...

An attachment that has not been issued is listed as `NOT ISSUED` and the command
exits 1 — the operator is about to attach it to a real email, so hiding it would
be the wrong kindness.

**It does not send mail.** Sending is outward-facing, with its own credentials,
failure modes and audit trail, and none of that belongs in a document register.
The bodies are plain text to be pasted into a mail client by a person.

`HR-CHECKLIST.md` is the one-page version for whoever actually does this.

## The folder you hand over

`docket pack` groups documents by **what the person must do**, not by type,
because "which of these must I sign and send back" is the only question someone
opening the folder has:

    PAR-P-000001-X/
      00_Read_Me_First.pdf          generated index: what is here, what returns
      01_Sign_and_return/           both parties sign; a signed copy returns
      02_Complete_and_return/       fill in and send back
      03_For_your_records/          theirs to keep
      04_Optional/                  declining changes nothing
      MANIFEST-sha256.txt           verifies with plain `shasum -a 256 -c`

The grouping is derived from `lifecycle.yaml`, so a new document lands in the
right folder without anyone editing a list. `00_Read_Me_First.pdf` is generated
from the register, so it cannot describe a pack it was not built from — and it
says, in as many words, not to sign anything unread because it arrived in a
folder of eleven documents.

The pack is a **copy**. The store stays authoritative; rebuild or delete a pack
freely.

`onboard` issues every document the person's current stage owes, allocates a
real serial for each, writes the PDF into the store, and appends a
tamper-evident register entry. Running it twice issues nothing twice.

Move the person's `internship.stage` forward (`offered` → `preboarding` →
`active` → `offboarding` → `alumni`) and run `onboard` again; each stage
owes its own documents, cumulatively.

## Commands

| | |
|---|---|
| `docket new <slug> <class>` | allocate identifiers and open a record |
| `docket reengage <slug> <class>` | new engagement, same person: intern → employee |
| `docket pack <slug>` | assemble the folder to hand over |
| `docket emails <slug>` | the messages to send, filled in, with attachments |
| `docket index` | regenerate the archive index and payload manifest |
| `docket retention` | documents past their retention period |
| `docket next <slug>` | **the single next thing to do**, and the command that does it |
| `docket status <slug>` | stage, what is issued, what is **executed**, what is outstanding |
| `docket sign <serial>` | record that a document came back executed |
| `docket advance <slug>` | move to the next stage — refuses on an unmet signature gate |
| `docket record <slug>` | everything held: identity, documents, access, history |
| `docket roster` | everyone engaged, with current standing |
| `docket event <slug> <kind>` | record what happened: access granted, equipment issued |
| `docket separate <slug>` | end the engagement; documents and NDA survive |
| `docket erase <slug> --confirm` | DPDP erasure; identifier survives as a tombstone |
| `docket onboard <slug>` | issue everything the current stage owes |
| `docket issue <slug> <CODE>` | issue one document with a serial |
| `docket render <slug> <CODE>` | draft: watermarked `DRAFT`, `NOT ISSUED`, unregistered |
| `docket verify <serial> [--file X]` | is this PDF the one that was issued? |
| `docket check` | validate registries, templates, records, register chain |

Exit codes are meaningful: `0` clean, `1` outstanding or integrity failure,
`2` refused or misuse. Safe to put in CI.

## Editing the facts

Everything editable lives in `registry/`. Nothing in `latex/` or
`templates/` may restate any of it — `docket check` fails the build if a
template hard-codes the address, the email or an officer's name.

| file | owns |
|---|---|
| `org.yaml` | address, email, website, legal status, CIN, governing-law forum, notice period, security and POSH contacts, serial format |
| `person-classes.yaml` | intern / employee / contractor: what each may hold, must hold, and must not |
| `fields.yaml` | every field, its purpose, its store. 54 of them |
| `signatories.yaml` | who may sign, and under what designation |
| `lifecycle.yaml` | the 13 documents, and what each stage owes |
| `retention.yaml` | how long each class is kept, **and why** |

Change the office address in `org.yaml` and every one of the fourteen
documents picks it up on the next build. That was the point: the address
used to appear verbatim in eleven files.

## The 14 documents

Ten are transcribed from the `.docx` originals, which are the source of
truth for content — the extractor used for that migration is archived with
the Python implementation, since reading Word is a one-off concern rather
than something the production system does. Four are additions.

**Letters** — Internship Offer *(new)*, Business Communication
**Agreements** — Internship Agreement, NDA, IP Assignment
**Policies** — Information Security & Acceptable Use, POSH *(new)*, Data Protection Notice *(new)*
**Forms** — Intern Onboarding, Emergency Contact, Equipment Acknowledgement, Offboarding Checklist
**Certificates** — Certificate of Internship, Letter of Reference

Forms are dual-purpose: fields the person record knows are pre-filled,
fields it does not are ruled lines to complete by hand. Nobody retypes what
Paridyum already holds.

## Where things live

    src/domain/          rules: escaping, serials, the chain, obligations
    src/application/     use cases: issue, status, verify, check
    src/adapters/        filesystem, clock, pdflatex
    src/main.rs          CLI
    latex/paridyum.cls   ALL layout. The letterhead, boxes, footer, page
                         numbering and signature blocks live here once.
    templates/           Document content only. No layout, no facts.
    registry/            The facts.
    people/              person records          [GITIGNORED]
    store/               register + issued PDFs  [GITIGNORED]

See ARCHITECTURE.md for the layering and the two decisions that shape it.

## Guarantees, and how they are enforced

**Reproducible.** The same document issued twice is byte-identical.
`SOURCE_DATE_EPOCH` pins the dates and `\pdftrailerid` pins the PDF trailer
`/ID`, which pdfTeX otherwise derives from the wall clock and which was the
sole remaining source of drift. Tested.

**Verifiable.** Each register entry records the PDF's SHA-256 and the
content hash of the template *as issued*, so a document stays tied to the
wording it was issued under. `docket verify` rejects a modified PDF.

**Tamper-evident.** The register is a hash chain: each entry carries the
digest of the one before it. Editing an entry in place — backdating an
issue date, say — is detected by `docket check`. Tested.

**Loud.** A template referencing a value the data does not define fails the
build. No signed document ever carries a silently blank field.

**Refuses defective output.** `onboard` will not issue when the
governing-law forum is unset, or when the unpaid basis is not evidenced by a
current enrolment. `--force` overrides and says the documents will be
defective.

## Looking at everything at once

    docket --tui

A full-screen, read-only view: the roster, and — on selecting someone — their
documents with signature state, and their `next` steps. Navigation is a number
then Enter.

**No dependency, and that is not a compromise.** Clearing a screen is two ANSI
escapes; the only part needing a library is reading a single keypress, which
needs raw mode, which needs `unsafe` — and `unsafe_code` is `forbid` here. So
line-based navigation is the only shape the lint policy permits, and it saves
fourteen transitive crates.

**Read-only by construction.** No input in the browser reaches a write path, so
a half-drawn view cannot leave a record in a bad state. Writing is
`docket new -i`, `sign`, `advance` — each of which says what it did.

## The interview

`docket new <slug> <class> --interactive` asks instead of handing you a YAML
file. The questions are **generated from `registry/fields.yaml`**, so they
cannot drift from the schema:

- only fields the class permits are asked — an unpaid intern is never asked for
  a PAN, because asking is already the failure;
- only fields declared to live in the `personnel` store are asked, so
  third-party emergency contact details never land in the wrong file;
- the field's **stated purpose is the help text**, first sentence at the prompt
  and in full as a comment beside the value in the record;
- required questions come first, and it offers to stop there. Fifty prompts is
  an interrogation, not a wizard.

The record it writes is self-documenting and passes `docket check` as written.
`docket new` without `-i` still copies the template — removing a working path to
add one is what ADR-0054 forbids.

## Who did it

Every write records an operator, resolved from `--by <id>` or `DOCKET_OPERATOR`
and validated against `registry/operators.yaml`. A write with no operator is
**refused**, not defaulted — a default is exactly what made every event in the
log read `recorded_by: "docket"`, and an audit trail that cannot name a person
is not one. Reads need no operator.

Operators are separate from signatories: signing a document and running the tool
that issues it are different acts, done today by the same people.

## Two typesetters (ADR-0054)

`pdflatex` when it is present, Tectonic when it is not. **Neither replaces the
other.** Every existing register entry was produced by pdflatex and its
reproducibility claim is against it; swapping would have invalidated all of them
to spare an install only a new operator experiences.

Each entry records `engine` and `engine_version`, so a durable artifact says
what produced it. Entries written before two engines existed carry none and read
as `pdflatex` — safe precisely because the alternative did not exist then, and
proven not to change their sealed digests.

## Two identifiers (ADR-0051)

    PAR-P-000001-X      the person.     Opaque, permanent, names the folder.
    PAR-INT-2026-0001   the engagement. Class and year, names the documents.

The person identifier encodes **nothing** about the person, because an
identifier carrying a mutable fact stops being true — an intern who is hired
would otherwise keep a folder labelled `INT` forever, or get a second
identifier and a split archive. `docket reengage` gives them a new engagement
and keeps the person, so one folder holds the whole relationship.

The engagement identifier *does* carry class and year, and that is correct:
class is mutable for a person and immutable for an engagement. An engagement
that was an internship was an internship permanently.

The person identifier ends in an ISO 7064 MOD 11-2 check character, because it
is typed into folder names and read aloud. Every single-digit error and every
transposition of adjacent digits is detected.

Folders are named by identifier, never by name: a folder called `asha-verma/`
in a shared drive discloses who Paridyum engages to anyone who can read the
listing, without opening anything.

## Personnel records

A person is created with `docket new`, which allocates both identifiers and
opens a record. Neither is ever reused, including after erasure, so a
certificate issued in 2026 still names the same person in 2036.

**Fields are class-gated.** `person-classes.yaml` declares what an intern, an
employee and a contractor may each hold, and `fields.yaml` gives every field
a stated purpose. `docket check` refuses a record that holds a field its
class forbids — collecting a PAN for an unpaid student intern is not
untidiness, it is processing personal data for which no purpose exists, and
that is what the DPDP Act's purpose limitation forbids. No statute requires
a PAN, Aadhaar, UAN or bank account for an unpaid student placement, so the
`intern` class forbids all of them.

**History is recorded, not just state.** The record holds the current
position; the event log holds how it got there. `docket event` records
access granted and revoked, equipment issued and returned, stage changes and
separation. Access still open is *derived* from that log, so it cannot
disagree with it — which is what makes the offboarding checklist's access
section checkable rather than merely assertable.

**Records are split by who may read them.** `person-classes.yaml` declares
four stores: the ordinary record; emergency contacts, which are third-party
data; statutory identifiers, which stay empty while everyone is an unpaid
intern; and a confidential store for medical, disciplinary and POSH material
that this tool deliberately does not write. A POSH complaint in a personnel
file is readable by whoever can read the file, which defeats the
confidentiality the Act requires.

## Personal data and erasure

`people/` and `store/` are gitignored and must stay that way. Data committed
to git cannot be erased, which would make DPDP erasure rights unhonourable.
Integrity therefore comes from hash chains rather than git immutability —
better anyway, because a chain survives erasing the person underneath it.

`docket erase --confirm` actually deletes: the record file goes, and the
person's event detail with it. What survives is the identifier, the class
and the dates, which name nobody and which the document register still
refers to. The event log is the one place this system rewrites rather than
appends — an append-only log that cannot forget is a log that cannot honour
an erasure request — and the chain is re-sealed so it still verifies.

## Issued is not signed

`docket status` marks a document `[x]` only when it is recorded as **executed**,
`[~]` when it went out and has not come back:

    [x] NDA  Non-Disclosure Agreement    PAR/NDA/2026/0001
    [~] PSH  Policy on Prevention...     PAR/PSH/2026/0001

    9/9 issued and 5 executed, at stage "active"
    awaiting the executed copy back: PSH DPN ONB EMC

A signature is recorded in its **own** hash-chained ledger, never by editing the
issuance entry — the register is append-only precisely so what was issued cannot
be rewritten once someone has relied on it. A signature is a new fact learned
later, so it is a new record.

`docket sign` refuses a wet-ink signature with no scan (wet ink leaves no trail
of its own, so the scan *is* the evidence), refuses a signing date earlier than
issuance, and refuses to record the same signature twice. Scans are copied into
the store and hashed; `docket check` reports one that has gone missing or
changed.

**The stage gate is enforced, not documented.** `lifecycle.yaml` says no system
access until the agreements and the security policy are signed. `docket advance`
now refuses while any of them is merely issued:

    refusing to move asha-verma out of "preboarding": 4 document(s) are not
    recorded as executed.
      - NDA (Non-Disclosure Agreement): issued as PAR/NDA/2026/0001 but not
        recorded as executed
      ...
    Issuing a document is not the same as getting it back.

`--force` overrides and says that it did.

## Archival

    docket index                                    # regenerate
    cd store && shasum -a 256 -c manifest-sha256.txt   # verify, no tooling

Issued documents are **PDF/A-2b (ISO 19005-2)** with embedded fonts, XMP
metadata and an sRGB output intent, so they render without this toolchain.
`store/index/` holds a generated `roster.csv` and `documents.csv`; the store
root holds a BagIt-style `manifest-sha256.txt`. Together they make the archive
self-describing: someone in 2036 with no Rust and no `docket` can read who held
what and verify every file with `shasum`.

The index is generated output (C-0004) and `docket check` fails when it is
stale — ADR-0040's pattern, because the failure mode of generated output is
staleness rather than incorrectness.

`docket retention` reports what is past its period. It never deletes; that is
a decision.

## Tests

    cargo test                    # 219 tests
    cargo clippy --all-targets    # clean under pedantic
    cargo fmt --check
    cargo run -- check            # validate the checked-in registry

See TESTING.md.

## Before anything real is signed

`DECISION-DEBT.md` lists the open items. All document wording — especially
the four new documents — is a **draft for review by qualified counsel**.
This repository provides structure, custody and evidence. It does not make
a document legally sufficient.
