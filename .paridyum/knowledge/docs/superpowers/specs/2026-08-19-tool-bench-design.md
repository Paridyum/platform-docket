# `tool-bench` — the D2 baseline harness

**Written 2026-08-19.** Design for the first runtime repository Paridyum builds: a benchmark
harness that measures three naive multi-tenancy implementations *before* ADR-0024's mechanism
exists, and that becomes `rust-ci.yml`'s first integration-testing caller while doing it.

This document is prose in `docs/`, not a knowledge artifact. It binds nothing. Where it and a
knowledge artifact disagree, **the artifact wins.**

**Closes:** T-0.4 (D2 baseline harness, expiring) · most of T-2.1 (watch `rust-ci.yml` fail).
**Feeds:** T-0.3 (D1 thesis, needs the metric) · T-0.7 and BET-0008 open question 1 (the
fully-loaded cost of one tenant, whose denominator is density).

---

## 1. Why this exists, and why now

RFC-0013 D2: *"A benchmark harness capturing latency, memory, throughput and density lands before
the optimised path does. A claim of improvement requires a measured predecessor, and once the naive
implementation is gone it cannot be recovered."*

That is the whole argument. It is not a performance-engineering exercise — it is evidence
collection with an expiry date. Every day this waits, the probability rises that someone writes
ADR-0024's schema first and the predecessor becomes something we reconstruct from memory rather
than something we measured.

**The claim being baselined.** C-0006 non-commodity item 2 asserts *provable isolation at two
levels*, and ADR-0024 is the mechanism: one Postgres database, one schema, nested tenancy as two
columns and two `FORCE`d RLS predicates, operator scope as an explicit positive grant, no
per-tenant infrastructure. The claim has two halves, and they need different baselines:

| Half of the claim | Naive predecessor that half improves on |
| --- | --- |
| **Density** — no per-tenant infrastructure | `db-per-tenant`, `schema-per-tenant` |
| **Provability** — isolation the runtime guarantees rather than the application remembering | `shared-schema-app-filter` |

A single-arm baseline can only support one half. That is why all three arms are built.

---

## 2. What this is not

- **Not the mechanism.** The ADR-0024 RLS arm is deliberately **not** implemented here. D2 requires
  the baseline to land first, and building both at once destroys the ordering that makes the
  comparison evidence rather than assertion. It is added later as a fourth implementation of the
  same trait, run by the same driver, against the same workload.
- **Not `platform-tenancy`.** No code here is a precursor to the real runtime. Anything that looks
  reusable should be viewed with suspicion — this repository's arms are all things Paridyum has
  decided not to build.
- **Not a CI benchmark.** Benchmark runs are manual, on a pinned host, recorded with provenance.
  Numbers from shared CI runners cannot support a patent claim and should never be recorded as
  though they could.
- **Not throwaway.** The user asked for production quality, and D2's own reasoning requires it: a
  measurement that cannot be re-run in a year, against a fourth arm, by someone re-deriving the
  claim, is not evidence. The harness must still work when the mechanism arrives.

---

## 3. Repository identity

| | |
| --- | --- |
| **Name** | `tool-bench` |
| **Class** | `tool` (`class-registry.yaml`: developer-facing executables, `stacks: [rust]`, `deployable: false`, `ci: true`) |
| **Binary** | `bench` — the class rule is that the binary name is the bare capability |
| **Path** | `~/Developer/paridyum/tools/tool-bench`, beside `tool-forge` |
| **Remote** | `git@github.com:Paridyum/tool-bench.git` |
| **Toolchain** | `1.97.1`, from `toolchain-registry.yaml` (ADR-0048 — the registry is the authority, not the template) |
| **Lifecycle** | `alpha` |

**Class `tool` rather than a new class.** The harness is a developer-facing executable that is not
deployed. Nothing about it needs a class that does not exist, and inventing one to hold a single
repository is the sprawl RFC-0018 is about.

---

## 4. Architecture

One binary, one trait, three implementations, one driver, one results format.

```
                    ┌─────────────┐
  workload.rs ─────▶│   driver    │◀──── provenance.rs (host, toolchain, image digest)
  (the fixed        │  (ramp,     │
   query mix)       │   measure)  │
                    └──────┬──────┘
                           │  dyn Tenancy
              ┌────────────┼────────────┐
              ▼            ▼            ▼
      db-per-tenant  schema-per-tenant  shared-schema-app-filter
              └────────────┼────────────┘
                           ▼
                  Postgres (testcontainers, pinned digest)
                           │
                           ▼
                    results/*.json  +  results/*.md
```

### 4.1 The `Tenancy` trait

The seam that makes the arms comparable. Every arm implements exactly this, and the driver knows
nothing else about them.

```rust
#[async_trait]
trait Tenancy {
    /// Stable identifier recorded in results. Never derived from the type name.
    fn arm(&self) -> Arm;

    /// Create whatever this arm needs to exist before any tenant does.
    async fn migrate(&self, pool: &PgPool) -> Result<()>;

    /// Bring one tenant into existence. The cost of this call IS the density story
    /// for two of the three arms, so it is measured, not just awaited.
    async fn provision_tenant(&self, t: TenantId) -> Result<()>;

    /// Every read and write goes through a scope. An arm cannot see a query
    /// without also seeing who is asking — which is the property the
    /// app-filter arm is about to fail to enforce.
    async fn execute(&self, scope: &Scope, op: &Op) -> Result<OpOutcome>;

    async fn teardown(&self) -> Result<()>;
}
```

`Scope` carries `tenant_id` and an optional `account_id`. It is a distinct type with private
fields and no `Default`, so an arm cannot silently construct an empty one — the type-level echo of
ADR-0024 decision 5. **This is deliberately stricter than the arms deserve.** The point is that the
app-filter arm must *choose* to drop the predicate rather than fall into it, so the leak we
demonstrate is the one a real developer writes, not one the harness made easy.

### 4.2 The three arms

**`db-per-tenant`** — one Postgres database per tenant, created on provision, one connection pool
per database with a bounded LRU of open pools. Isolation is total and trivially provable. The cost
is per-tenant infrastructure: catalog entries, per-database background overhead, connection
multiplication, and a `provision_tenant` that runs a full migration.

**`schema-per-tenant`** — one database, one schema per tenant, `search_path` set per connection.
The common middle answer. Cheaper than a database each; still per-tenant DDL, still linear growth
in `pg_class`, and planning time degrades as the catalog grows.

**`shared-schema-app-filter`** — one database, one schema, `tenant_id` and `account_id` columns,
and **every predicate applied in application code**. No RLS. This is what almost everyone actually
builds, and it is the arm whose density will be indistinguishable from the mechanism's. It exists
to fail the *other* axis, and §6 is how that failure is recorded.

### 4.3 The schema

Deliberately the example from `templates/rust-service/manifest.yaml`, so the harness and the
service template describe the same imaginary product:

| Table | ADR-0024 category |
| --- | --- |
| `regions` | platform-owned — no tenancy column |
| `plans` | tenant-scoped — `tenant_id` |
| `invoices` | account-scoped — `tenant_id` **and** `account_id` |
| `invoice_lines` | account-scoped — `tenant_id` **and** `account_id` |

Including a platform-owned table matters: it is the control. A query with no tenancy predicate
should cost the same in all three arms, and if it does not, the harness is measuring something
other than tenancy.

### 4.4 The workload

Fixed, versioned, and changed only by a version bump that invalidates prior results.

| Op | Shape | Why it is here |
| --- | --- | --- |
| `W1` | List invoices for one account, paginated | The hot path. Account-scoped, two predicates, the query the mechanism must not slow down |
| `W2` | List plans for one tenant | Tenant-scoped, one predicate |
| `W3` | Insert one invoice with four lines, in a transaction | The write path, and the one where the app-filter arm must remember the columns |
| `W4` | Look up a region by code | Platform-owned control — no tenancy predicate in any arm |

Mix: 70% `W1`, 15% `W2`, 10% `W3`, 5% `W4`. Chosen to be read-dominated because a runtime's
tenancy cost is paid on reads; recorded as a constant in one place so a future run can restate it.

---

## 5. Metrics, and what "density" means

Four metrics, per arm, per tenant-count step.

| Metric | Definition |
| --- | --- |
| **Latency** | p50 / p95 / p99 per op type, from an HDR histogram, measured client-side at a fixed offered concurrency |
| **Throughput** | Sustained ops/sec over a fixed measurement window after a discarded warm-up |
| **Memory** | Postgres container working-set bytes, sampled through the run; reported as steady-state and peak |
| **Provisioning** | Wall-clock time of `provision_tenant`, per tenant, as a distribution. For two arms this runs DDL and is the density story directly; for the third it is an insert. A tenant that takes minutes to create is a product constraint even when steady-state latency is fine |
| **Density** | See below |

### The degradation condition

**Density is not a single number and must not be recorded as one.** The harness ramps tenant count
geometrically (1, 2, 4, … up to arm failure or a cap) and records the **full curve** of every metric
against tenant count. Density is then *derived* from the curve at a stated threshold:

> **Density = the largest tenant count at which p99 `W1` latency stays within 3× the same arm's
> single-tenant p99, and no hard failure has occurred.**

Two properties of that definition are load-bearing:

1. **It is relative, not absolute.** REQ-0016 exists to stop numbers derived from taste being
   shipped as commitments, and an absolute millisecond target here would be exactly that. A
   multiple of the arm's own single-tenant performance is a claim about degradation, which is what
   the D1 thesis actually needs.
2. **The threshold is applied after the fact, to stored data.** Because the whole curve is
   recorded, anyone who thinks 3× is the wrong number can re-derive density at 2× or 5× **without
   re-running anything.** The measurement is separated from the judgement about it. That separation
   is the single most important design decision in this document — it is what stops a disagreement
   about a threshold from invalidating a baseline that cannot be retaken.

**Hard failures are recorded, not treated as errors.** `db-per-tenant` will exhaust connections or
file descriptors; `schema-per-tenant` will degrade on catalog size. The tenant count and the exact
failure are the most interesting data points in the run and must be captured with the error text
verbatim, never summarised.

---

## 6. The isolation assertions — the other half of the claim

Separate from the benchmark, and run in CI rather than by hand, because correctness is not a
measurement.

For each arm, a test suite asserts the STD-0009 shape at the two boundaries:

- A read scoped to tenant A never returns tenant B's rows.
- A read scoped to account 1 never returns account 2's rows **within the same tenant** — the inner
  boundary, which ADR-0024 decision 3 names as the one every dashboard reports as healthy.
- Operator scope is a positive claim: a `Scope` with a missing `account_id` must **fail**, never
  widen (GLO-0005).

**`shared-schema-app-filter` is then given a deliberate defect and the suite is re-run.** One
predicate is dropped from one query — the `account_id` clause on `W1` — behind a cargo feature.
The expected result is recorded as evidence:

| Arm | Outer boundary | Inner boundary |
| --- | --- | --- |
| `db-per-tenant` | holds structurally | holds structurally |
| `schema-per-tenant` | holds structurally | **application's job** |
| `shared-schema-app-filter` | application's job | application's job |
| `shared-schema-app-filter` + dropped predicate | holds | **leaks, silently, with rows returned** |

That last row is the D1 thesis's technical effect stated as a reproducible test rather than a
paragraph. It is worth more than any latency number in this repository.

---

## 7. Provenance — what makes a number evidence

Every results file carries, captured automatically and never typed by hand:

- Host: CPU model, core count, total memory, kernel, whether the run was on bare metal or a guest
- Postgres image **digest**, not tag, and the server's `version()` string
- Rust toolchain version and the `tool-bench` git commit SHA, with a dirty-tree flag
- Workload version, mix constants, concurrency, warm-up and measurement window
- UTC start and end timestamps

**A run from a dirty working tree is marked `provisional` in the file itself** and is not eligible
to be cited. Signed commits (RFC-0013 D3) make the SHA a dated, tamper-evident anchor for the
measurement; a dirty tree breaks that chain, so the file must say so rather than let a reader
assume otherwise.

Results are committed to `results/` as JSON, with a generated Markdown summary beside them. The
JSON is the record; the Markdown is for reading.

---

## 8. Compliance obligations

Verified against `rust-ci.yml` and `check-repo.sh` rather than assumed:

- `README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `manifest.yaml` — required by the `docs` job.
  `API.md` and `RUNBOOK.md` are **not** required (that branch is `svc-*` and `platform-*` only).
- `TESTING.md` with its four sections — checked by `check-repo.sh` in the `governance` job, not by
  the inline `docs` copy.
- `.paridyum/knowledge/` vendored snapshot present (ADR-0001), via `forge knowledge sync`. Its
  absence fails the `governance` job loudly.
- `rust-toolchain.toml` pinned to `1.97.1` exactly, sourced from `toolchain-registry.yaml`.
- `rustfmt.toml`, `clippy.toml`, `deny.toml`, `cliff.toml`, `release-plz.toml`, `.gitleaks.toml`,
  `CODEOWNERS` — copied from `templates/rust-service` and from `tool-forge`'s working shape.
- Signed commits from commit one (RFC-0013 D3), matching `knowledge`'s existing configuration.
- `manifest.yaml` declares `owns.tables` for the four benchmark tables. They are owned by nothing
  real, but a manifest that lies about ownership is worse than one that is boring.

### The CI caller

```yaml
jobs:
  ci:
    uses: Paridyum/infra-ci/.github/workflows/rust-ci.yml@v1
    with:
      integration: true       # first caller ever to pass this
      coverage-total: 70
      coverage-patch: 80
```

**Development happens on branches through pull requests, not pushes to `main`.** This is not
process preference — it is what makes the `change lifecycle`, `regression test on a fix` and
patch-coverage paths execute at all. `tool-forge` pushed to `main` and those three jobs have
therefore never run.

---

## 9. T-2.1 — planting the defects

**Correction to the roadmap, dated 2026-08-19.** ROADMAP.md §2 and TASKS.md T-2.1 both state that
`rust-ci.yml` *"has never executed."* This is false as of 2026-08-10: `tool-forge` has five green
runs, the most recent being `31389337904`, in which nine of thirteen jobs ran. The roadmap was
rewritten on 2026-08-18 and did not pick this up. **Both files need correcting, and the corrected
task is narrower and more useful.**

What is actually unexecuted:

| Path | Condition that skipped it |
| --- | --- |
| `mutation (TST-0013)` | `inputs.tier == 'nightly'` — nothing schedules the nightly tier |
| `change lifecycle` | `github.event_name == 'pull_request'` — `tool-forge` pushes to `main` |
| `regression test on a fix` | same |
| `msrv` | `inputs.msrv != ''` — omitted, correctly, for a binary |
| Integration tests | `inputs.integration` — `tool-forge` passes `false` |
| Patch-coverage threshold | PR-only |

And separately: **a job running green proves it executes, not that it rejects.** So T-2.1's real
content stands. Each defect below is planted in `tool-bench` on a branch, the failure observed and
its run URL recorded, then reverted:

| Defect planted | Must be caught by |
| --- | --- |
| Misformatted file | `fmt` |
| A `clippy::all` violation | `clippy` |
| A failing unit test | `test` |
| Coverage pushed below the total threshold | `coverage` |
| A changed line left uncovered | `coverage`, patch threshold — **PR-only, never yet run** |
| A commit with no changelog entry | `changelog` |
| A plausible secret | `secret scan` |
| A broken intra-doc link | `docs` |
| `ARCHITECTURE.md` deleted | `docs`, STD-0002 |
| `TESTING.md` missing a section | `governance` via `check-repo.sh` |
| `.paridyum/knowledge/` removed | `governance` |
| A dependency with a known advisory | `licences and advisories` |

**Any defect that is not caught is a finding, and the finding is the deliverable** — it goes to
`DECISION-DEBT.md` or a `TASKS.md` entry against `infra-ci`, not into a fix that quietly makes the
harness look competent.

`mutation` and `msrv` are out of scope: the first needs a nightly schedule that nothing yet has,
the second is deliberately not passed by binaries. Both are recorded as still-unexecuted rather
than forced.

---

## 10. Testing

| Tier | What runs | Blocking |
| --- | --- | --- |
| `pr` | Unit tests; isolation assertions for all three arms against testcontainers Postgres | yes |
| manual | The benchmark ramp, on the pinned host | no |

The isolation assertions are the part that must never be skipped for speed. They are the evidence.

**Deliberately untested,** to be written into `TESTING.md` rather than left implicit: the Markdown
summary generator's formatting, the LRU pool eviction under memory pressure, and the provenance
collector's behaviour on a host whose CPU model string is unreadable.

---

## 11. Risks and open questions

1. **The `db-per-tenant` arm may hit the host's limits before Postgres's.** 64 GB with 16 GB
   reserved is a real ceiling, and a density number that describes the Arch host rather than
   Postgres is misleading. Mitigation: the failure mode is recorded verbatim, and provenance
   captures the host, so a reader can tell which ceiling was hit. If it is the host's, that is
   stated in the results rather than reported as a Postgres density figure.
2. **Client-side measurement includes client cost.** At high concurrency the harness itself becomes
   a load generator bottleneck. Mitigation: a no-op arm that round-trips `SELECT 1` establishes the
   harness's own floor, recorded alongside every run.
3. **The results are only as good as the workload's realism, and there is no production traffic to
   derive it from.** Accepted openly: the mix is a stated assumption, versioned so it can be
   changed and re-run rather than argued about.
4. **Open — where does this repository's own D1 conception log live?** RFC-0013 D3 owes one, and
   `tool-bench`'s history is the first contemporaneous record of the mechanism's measurement.
   Proposal: a `CONCEPTION.md` in this repository, appended to per session. **Not decided here** —
   it is an RFC-0013 D3 question, and deciding it inside a benchmark spec would be exactly the kind
   of consequential decision made in the wrong place that ADR-0013 exists to prevent.

---

## 12. Done when

- `tool-bench` exists, is on GitHub, has signed commits from commit one, and its CI is green.
- All three arms implement `Tenancy` and pass the isolation assertions in CI.
- The app-filter arm, with its predicate dropped, **demonstrably leaks across the inner boundary**,
  and that test is committed as evidence.
- One full ramp has run on the pinned host and `results/` contains a JSON file with complete
  provenance and a full metric curve for each arm.
- Density is derived from that curve at the stated threshold, and the number exists — which is
  T-0.4's own completion condition and the input T-0.3 and T-0.7 are waiting on.
- The T-2.1 defect matrix has been planted and observed, and every uncaught defect is written down.
- ROADMAP.md and TASKS.md are corrected on the "never executed" claim.
