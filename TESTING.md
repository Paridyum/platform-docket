# Testing — platform-docket

    cargo test              # 96 tests
    cargo clippy --all-targets
    cargo fmt --check
    cargo run -- check      # validate the checked-in registry

## Tiers

**Unit (68)** — `domain/` and `adapters/clock`, in-process, no filesystem, no
TeX. These carry the rules: escaping, serial allocation, chain integrity,
stage obligations, refusal conditions.

**Integration (28)** — `tests/issuance.rs` renders real PDFs with pdflatex
against a throwaway store; slower (~15s) because typesetting is real, which
is the point since the guarantees are about actual output bytes.
`tests/records.rs` covers identifiers, class gating, events and erasure, and
runs in milliseconds because none of it needs TeX.

## What the tests are for

Each one is either a defect that reached a built PDF during development, or
a guarantee the register makes. Named so a failure says what broke:

- `backdating_an_entry_is_detected`
- `removing_an_entry_is_detected`
- `issuing_the_same_document_twice_is_byte_identical`
- `verify_matches_the_issued_document_and_rejects_a_modified_one`
- `issuance_is_refused_when_the_unpaid_basis_is_not_evidenced`
- `no_template_restates_a_fact_the_registry_owns`
- `backslash_is_not_re_escaped`
- `an_empty_list_still_produces_a_valid_itemize_body`
- `nulls_define_nothing_so_the_build_fails_loudly`
- `collecting_a_statutory_identifier_for_an_unpaid_intern_is_refused`
- `the_same_identifier_is_permitted_for_an_employee`
- `erasure_deletes_the_record_but_keeps_the_identifier`
- `an_erased_identifier_is_never_reissued`
- `every_chain_still_verifies_after_an_erasure`
- `an_erased_person_stays_on_the_roster_as_a_tombstone`
- `a_record_created_by_hand_is_reported_rather_than_trusted`
- `every_declared_field_states_a_purpose`

## Determinism (STD-0001)

`clippy.toml` forbids `SystemTime::now`, so the clock is a trait. Tests use
`FixedClock` and a pinned issue date; no test result depends on when it ran.

`expect`/`unwrap` are denied crate-wide and allowed only inside `#[cfg(test)]`
modules, with the reason stated at each site: a test that cannot assert an
expected value is not a test.

## Requirements

TeX Live with `pdflatex` on `PATH` for the integration tests. The unit tests
need nothing.

## Not tested

Deliberately uncovered, with the reason:

| What | Why |
|---|---|
| `src/main.rs` argument parsing and printing | `clap`'s own behaviour plus formatting. The exit codes ARE tested, through the binary, in `tests/issuance.rs` — those are contractual because CI reads them. |
| `pdflatex` itself | A third-party binary. What is tested is that our output is byte-reproducible and that a failed pass surfaces as an error rather than a silent empty PDF. |
| The legal sufficiency of any document | Not a testable property. `docket check` verifies structure, attribution and custody; counsel verifies the words. |
| `tools/extract_docx.py` | Archived with the Python implementation. A one-off migration concern, run by hand if the `.docx` originals ever change. |

## Quarantine

No test is flaky, ignored, or tolerated.

**Rendering tests SKIP where no typesetter is installed**, which is the case on
a GitHub runner. They print `SKIP <test>: no typesetter installed` so a green
run is not mistaken for coverage. This is a real gap, not a quarantine: it means
rendering, determinism and the PDF/A claim are verified on a developer machine
and not in CI. Recorded as DECISION-DEBT D-35 with the two ways to close it.

The integration suites are slow rather than flaky — they render real PDFs
because the guarantees they make are about output bytes, and a test that mocks
the typesetter cannot make them. `tests/signing.rs` was cut from 51s to 10s by
issuing one document per case instead of a nine-document pack; the remaining
cost is the point.

## Evidence

| Tier | Artifact | Where | Kept |
|---|---|---|---|
| `pr` | nextest JUnit XML | `target/nextest/ci/junit.xml`, uploaded by the shared workflow | 90 days, GitHub default |
| `pr` | coverage lcov | `lcov.info`, summarised against the thresholds in `.github/workflows/ci.yml` | the run |
| `nightly` | mutation report | `mutants.out/` | the run |
| local | rendered PDFs under test | a `tempfile::TempDir`, deleted when the test ends | not kept |

A test that renders a document writes into a throwaway store, never into
`store/`. That is what lets the suite run against a repository that holds real
records without touching them.
