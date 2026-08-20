# Architecture — tool-docket

Issues Paridyum's internship documents, and keeps a register that makes an
issued document verifiable years later.

## Layering (STD-0004)

Dependencies point inward only.

    main.rs        CLI. Argument parsing, printing, exit codes. No rules.
      ↓
    application/   Use cases: issue, status, verify, check.
      ↓
    domain/        Rules that hold regardless of storage or rendering.
      ↑
    adapters/      Filesystem, clock, pdflatex. The outside world.

`domain/` does no I/O and reads no clock, so every rule in it is testable
without a temp directory or a TeX installation. That is why 40 of the 53
tests run in under a millisecond.

| module | responsibility |
|---|---|
| `domain/escape` | rendering arbitrary text safely into LaTeX source |
| `domain/chain` | sealing records into a tamper-evident chain |
| `domain/identity` | allocating and retiring the identifier that names a person |
| `domain/personnel` | what may be recorded about a person, and what must be |
| `domain/event` | the history of what happened to an engagement |
| `domain/definitions` | flattening registry and person data into LaTeX definitions |
| `domain/serial` | allocation and formatting of serial numbers |
| `domain/register` | the tamper-evident record, and its hash chain |
| `domain/lifecycle` | which documents a person owes at each stage |
| `domain/person` | one person's record, and what must hold before issuing |
| `adapters/registry` | reading the registry files |
| `adapters/store` | person records, the register, issued PDFs |
| `adapters/typeset` | producing a PDF from a template |
| `adapters/clock` | supplying today's date, injectably |
| `application/personnel` | creating, describing and retiring a record |
| `application/roster` | the register of everyone engaged |

## Three chains, one implementation

    store/register/issued.jsonl   documents issued
    store/register/people.jsonl   identifiers allocated, standing changes
    store/register/events.jsonl   what happened to an engagement

Separate files because they answer separate questions; one `domain::chain`
implementation because the integrity argument is identical. Each record
carries the digest of the one before it.

## What a person record may hold

`person-classes.yaml` declares classes; `fields.yaml` declares fields, each
with a purpose and a store. Validation is the intersection: a record may
hold a field only if its class permits the field's group, must hold the
fields the class requires, and must not hold what it forbids.

This exists because of a finding, not a preference. Paridyum's people are
unpaid student interns, who are not employees under Indian law: EPF, ESI,
gratuity and TDS do not apply to them, so nothing requires a PAN, an
Aadhaar, a UAN or a bank account. Collecting one anyway is processing
personal data for which no purpose exists. The `intern` class therefore
forbids the whole `statutory` group, and `employee` permits it — the same
field, lawful in one class and not the other.

## Two decisions that shape everything

**The store is not the repository.** Person records and the register live
under `DOCKET_STORE`, outside git. Data committed to git cannot be erased,
and DPDP §12 erasure rights would then be unhonourable.

That removes git as a source of integrity, so the register supplies its own:
each entry carries the SHA-256 of the entry before it. Editing an entry in
place is detected. This is strictly better than relying on git immutability,
because the chain survives erasing the person underneath it.

**Verification compares stored hashes, never re-renders.** A register entry
records the hash of the issued PDF and the hash of the template *as issued*.
`verify` hashes the file presented and compares. It deliberately does not
re-render: a future TeX Live positions glyphs differently, so a re-rendering
verifier would report forgery on every genuine document the day the
toolchain moved. The entry records `texlive` so a reproduction attempt knows
what to pin.

## Reproducibility

Two things make a rendered PDF byte-stable:

- `SOURCE_DATE_EPOCH` at **UTC midnight** of the issue date, plus
  `FORCE_SOURCE_DATE=1`, which pins the embedded dates. UTC rather than local
  midnight, so the output does not depend on the operator's timezone.
- `\pdftrailerid` in `paridyum.cls`, which pins the PDF trailer `/ID`.
  pdfTeX otherwise derives it from the wall clock, and it was the sole
  remaining source of drift after the dates were fixed.

`Definitions` uses a `BTreeMap`, so the generated data file is byte-stable
regardless of insertion order.

## Failure posture

**Missing data fails the build.** Templates reference `\dkPNameLegal` and
friends; the class loads the generated definitions last. A value the data
does not define stays undefined and pdflatex stops. No signed document ever
carries a silently blank field.

**Defective output is refused, not warned about.** `onboard` will not issue
when the governing-law forum is unset or the unpaid basis is unevidenced.
`--force` overrides and says so.

**Exit codes are contractual.** `0` clean, `1` outstanding or integrity
failure, `2` refused or misused. `check` is safe to run in CI.
