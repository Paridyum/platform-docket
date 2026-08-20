---
id: STD-0001
title: Testing standard for Rust services and libraries
type: standard
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [platform, service, library, tool]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [testing, quality, coverage, ci]
breaking: true
enforced_by:
  - check: test.nextest_configured
    severity: error
    args: { path: ".config/nextest.toml" }
  - check: test.coverage_meets_threshold
    severity: error
    args: { line: 70, patch: 80 }
    rationale: Patch coverage is stricter than total; new code carries the burden, not the backlog.
  - check: test.no_ignored_tests_without_reason
    severity: error
    rationale: An #[ignore] with no reason is a permanently disabled test nobody will revisit.
  - check: test.integration_tests_use_testcontainers
    severity: warn
    args: { applies_to: ["svc-*", "platform-*"] }
  - check: test.doc_tests_enabled
    severity: error
    rationale: Doc tests are the only tests that fail when documentation goes stale.
---

# STD-0001 — Testing

Tests exist to make change safe. A suite that is slow, flaky, or unclear about what it
covers does the opposite: it gets skipped, and the skipping becomes habit.

## The stack

Fixed, so every repository looks the same and CI is one template (ADR-0005).

| Layer | Tool | Why this one |
| ----- | ---- | ------------ |
| Runner | **`cargo-nextest`** | Process-per-test isolation, so one panic cannot take out a whole binary's results. Faster, and its JUnit output feeds CI annotations. |
| Coverage | **`cargo-llvm-cov`** | Source-based, accurate, and integrates directly with nextest. |
| Property | **`proptest`** | For invariants over generated input — parsers, encoders, state machines. |
| Snapshot | **`insta`** | For output whose shape matters but whose text is long: JSON, CLI output, error rendering. |
| Integration | **`testcontainers`** | Real Postgres and Redis in Docker. Never mocks for the database. |
| Mutation | **`cargo-mutants`** | Scheduled, not per-PR. Answers "do the tests actually assert anything?" |
| Benchmarks | **`criterion`** | Only where a performance claim is being made. |

`cargo test` still runs doc tests — nextest does not execute them — so CI runs both.
This is the one place two runners are correct rather than redundant.

## The pyramid

```
       ╱ mutation ╲          scheduled weekly, not blocking
      ╱─────────────╲
     ╱  integration  ╲       real Postgres/Redis via testcontainers
    ╱─────────────────╲      one per endpoint or workflow
   ╱     component     ╲     module boundary, in-process, no network
  ╱─────────────────────╲
 ╱         unit          ╲   pure logic, no I/O, microseconds
╱─────────────────────────╲
```

**Unit** — `#[cfg(test)] mod tests` beside the code. No I/O, no sleeps, no network.
If a unit test needs a database, it is an integration test wearing the wrong label.

**Component** — `tests/` directory, exercising a module through its public API with
in-memory fakes. This is where most behaviour should be pinned, because it is fast
enough to run on every save and stable enough not to flake.

**Integration** — `tests/it/` with real dependencies in containers. One per HTTP
endpoint and per background workflow. Slow, so kept few and meaningful.

**Mutation** — weekly. A surviving mutant means a line is executed but nothing asserts
on it, which coverage percentage cannot tell you.

## Rules

**No mocking the database.** `testcontainers` starts a real Postgres. Mocked SQL tests
assert that the code calls the mock, which is a tautology; they pass while the query
is invalid. This is the single most common source of "tests green, production broken".

**Every test is deterministic.** No wall-clock dependence, no `sleep` to wait for
async work, no reliance on test ordering, no shared mutable global state. Inject a
clock. `sleep` in a test is a race condition with a timeout.

**`#[ignore]` requires a reason and a ticket:**
```rust
#[test]
#[ignore = "flaky under load — PDY-142"]
```
An ignored test with no reason is a deleted test that still costs compile time.

**Test names state the behaviour, not the function:**
```rust
#[test] fn rejects_expired_refresh_token() { }      // yes
#[test] fn test_refresh() { }                        // no
```
The name is what appears in the CI failure. It should say what broke.

**Assert on behaviour, not implementation.** A test that breaks when a private
function is renamed is a maintenance tax with no safety benefit.

## Coverage

| Metric | Threshold | Enforcement |
| ------ | --------- | ----------- |
| Patch (changed lines) | **80%** | blocks CI |
| Total (whole crate) | **70%** | blocks CI |

Patch coverage is stricter deliberately: new code carries the burden, not the
backlog. A repository inheriting low coverage can still merge well-tested changes,
and the total rises on its own.

Coverage is a floor, not a goal. 100% coverage with no assertions is worthless, which
is what `cargo-mutants` exists to detect. Chasing the number by testing getters is
how a suite becomes expensive without becoming useful.

## Layout

```
crate/
├── src/
│   └── thing.rs          #[cfg(test)] mod tests  — unit
├── tests/
│   ├── component/        public API, fakes
│   └── it/               integration, testcontainers
├── benches/              criterion, only where perf is claimed
└── .config/nextest.toml  profiles: default, ci, integration
```

## nextest profiles

```toml
# .config/nextest.toml
[profile.default]
retries = 0                      # locally, a flake should be visible immediately
failure-output = "immediate"

[profile.ci]
retries = { backoff = "exponential", count = 2, delay = "1s" }
failure-output = "immediate-final"
fail-fast = false                # report every failure in one run, not the first
status-level = "skip"
[profile.ci.junit]
path = "junit.xml"

[profile.integration]
test-threads = 4                 # container-bound, not CPU-bound
slow-timeout = { period = "30s", terminate-after = 4 }
```

CI retries exist to absorb infrastructure noise, not test flakiness. A test that only
passes on retry is failing; `cargo-nextest`'s flaky-test reporting surfaces it, and it
gets fixed or ignored with a reason.

## What is not tested

State it explicitly in the repository's `TESTING.md` rather than leaving a silent gap.
Generated code, third-party wrappers with no logic, and `main()` wiring are legitimate
exclusions. An undocumented gap reads as an oversight to whoever finds it next.

## Errata

Appended under [ADR-0031](../adr/ADR-0031-errata-are-additive-and-dated.md). Additive and dated;
the text above is unchanged and is what this standard requires.

### 2026-08-08 — the `#[ignore]` rationale is truncated at parse time

**Says**, on line 24 of the frontmatter:

```yaml
rationale: An #[ignore] with no reason is a permanently disabled test nobody will revisit.
```

**Wrong:** the `#` is unquoted, so YAML reads it as the start of a comment and the parsed value
is the two-character string `"An"`. A human reading the raw file sees the intent; every tool that
parses this frontmatter sees `An`. The `enforced_by` entry is otherwise valid, so nothing failed
and nothing warned.

**Intended:** `rationale: "An #[ignore] with no reason is a permanently disabled test nobody will
revisit."` — the same sentence, quoted.

**Still wrong after this erratum, and that is the point.** ADR-0031 §2 puts errata below the body,
and frontmatter is above it, so this annotation cannot repair the parsed value — only make the
defect visible to a reader who would otherwise never learn of it. ADR-0031 cost 4 records
frontmatter errata as unsolved. Correcting the value needs supersession.

REQ-0002 known instance 1. Found 2026-08-07; recorded here 2026-08-08, having lived until then
only in `check-registry.yaml`'s note on `test.no_ignored_tests_without_reason` — where no reader
of this standard would ever have encountered it, which is REQ-0002 criterion 5 in one sentence.
