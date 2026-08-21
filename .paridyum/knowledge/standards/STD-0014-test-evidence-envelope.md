---
id: STD-0014
title: A test run reports one envelope, naming what ran, where, and which declared type it implements
type: standard
status: accepted
created: 2026-08-21
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [testing, evidence, ci, junit, envelope, test-registry, std-0008, adr-0029, anti-0008, rfc-0034]
breaking: false
enforced_by: []
---

# STD-0014 — The test evidence envelope

Implements RFC-0034 §2. STD-0008 requires every blocking tier to publish a
machine-readable artifact; this says what that artifact is, so the same reader can
consume Rust, TypeScript, Python and anything after them.

## Why an envelope rather than the JUnit file

JUnit XML is what every stack already produces and it is the wrong unit on its own.
It describes *which assertions ran*. It says nothing about the four things any
question about test evidence actually needs:

- **which repository and commit** — a JUnit file is anonymous
- **which tier** it ran in (STD-0008 has six, and they mean different things)
- **which declared test type** it implements, so `test-registry.yaml` can be
  answered against
- **whether it came from CI or a laptop**, which decides whether it can be trusted

So the envelope wraps the JUnit rather than replacing it. The JUnit stays the
evidence; the envelope is the label on the box. Nothing in this standard parses
test *names* or *messages* — those are prose, and reading them would be ANTI-0008.

## The shape

One JSON object per test job. UTF-8, no trailing newline requirement, written to
`reports/envelope-<slug>.json` where `<slug>` distinguishes jobs within a run.

```json
{
  "schema": "paridyum.test-envelope/1",
  "repository": "Paridyum/console-docket",
  "commit": "97f05b0e1c2d3f4a5b6c7d8e9f0a1b2c3d4e5f60",
  "ref": "refs/heads/main",
  "tier": "pr",
  "implements": ["TST-0001", "TST-0002"],
  "stack": "typescript",
  "provenance": "ci",
  "run": { "id": "32492213670", "attempt": 1, "url": "https://github.com/..." },
  "started_at": "2026-08-21T14:27:12Z",
  "duration_ms": 29700,
  "outcome": "passed",
  "totals": { "tests": 48, "passed": 48, "failed": 0, "skipped": 0, "errored": 0 },
  "artifacts": [
    { "kind": "junit", "path": "reports/junit-unit.xml", "sha256": "…" }
  ],
  "skipped_reason": null
}
```

### Every field, and why it is there

| Field | Required | Rule |
| --- | --- | --- |
| `schema` | yes | `paridyum.test-envelope/<major>`. A consumer that does not recognise the major **refuses the envelope** rather than reading the fields it happens to know. |
| `repository` | yes | `owner/name`, as GitHub spells it. |
| `commit` | yes | Full 40-character SHA. Not abbreviated: an abbreviation is ambiguous across a long-lived ledger. |
| `ref` | yes | Full ref. `refs/pull/N/merge` is distinguishable from `refs/heads/main`, and they mean different things about what was tested. |
| `tier` | yes | One of STD-0008's tiers, exactly as `test-registry.yaml` spells them. |
| `implements` | yes | TST- ids from `test-registry.yaml`. **May be empty**, and an empty list is a claim: this job implements no declared type. Never omitted to mean "unknown". |
| `stack` | yes | From `class-registry.yaml`'s `stacks` vocabulary. |
| `provenance` | yes | `ci` or `local`. See below. |
| `run` | when `provenance: ci`, and **forbidden** when `local` | Identity of the run, so the envelope can be traced to logs that outlive it. A local envelope carrying one is a trace nobody can follow. |
| `started_at` | yes | RFC 3339, UTC, `Z`. Not local time — a ledger spanning machines cannot order local timestamps. |
| `duration_ms` | yes | Integer milliseconds. |
| `outcome` | yes | `passed` · `failed` · `errored` · `skipped` · `timed_out`. `errored` is the job failing to run its tests, which is different from tests failing, and collapsing them hides infrastructure decay. |
| `totals` | yes | Counts. Must sum to `tests`; a consumer that finds otherwise refuses the envelope rather than guessing which number is wrong. |
| `artifacts` | yes | Where the raw evidence went, with a digest so a later copy can be checked against what was reported. May be empty. |
| `skipped_reason` | when `outcome: skipped` | Free text **for a person**, never branched on. A skip with no reason is indistinguishable from a pass to everything downstream, which is how coverage silently disappears. |
| `submitted_by` | when `provenance: local` | Operator identifier. `tool-docket/registry/operators.yaml` is the only operator registry in the estate; it belongs to another repository, so this value is **recorded and not verified** here. Naming an author who might be wrong still beats an anonymous claim. |

`additionalProperties` is false. An unknown field is a producer and a consumer
disagreeing, and the disagreement should surface at the boundary.

## `provenance`, and what a local envelope may not do

`ci` means a workflow produced it and its `run` can be inspected by anyone with
read access. `local` means a person ran it on a machine nobody else can see.

Both are accepted. A local envelope is the only way the estate's most valuable
suite is visible at all — `console-docket`'s Playwright tests cover the write path
and cannot run in CI, because issuing renders documents and no runner has a
typesetter (`tool-docket` D-35).

Three rules follow, and they are the whole point of the field:

1. **A `local` envelope may never satisfy a gate.** Not a policy preference: a gate
   satisfied by an unverifiable claim is not a gate.
2. **A `local` envelope may never be the sole reason a `required` type reports as
   running.** A dashboard showing a required type as covered on the strength of one
   laptop run is asserting something nobody can check.
3. **Provenance is displayed wherever a number derived from it is displayed.** A
   view that blends the two silently is misrepresenting one of them.

Rejecting local runs outright was considered and is worse: the write path would
stay invisible, which is the condition RFC-0034 exists to end.

## Three rules the schema cannot enforce, and one it now can

`test-envelope.schema.json` is the machine contract, and it does not carry
everything this standard requires. Stated here rather than discovered, because a
contract believed to check more than it does is worse than one known to check less.

Verified 2026-08-21 by mutating a real envelope built from a live CI run. Of ten
deliberate corruptions, **eight are rejected** by the vendored validator:
an abbreviated commit, an unrecognised schema version, `implements` omitted, an
unknown field, a tier outside STD-0008, a malformed TST- id, a non-UTC timestamp,
and a negative duration.

**Three are not, and all three are cross-field:**

1. **`totals` summing to `tests`.** Not expressible in JSON Schema at any version —
   the language has no arithmetic. The converter computes these from one JUnit
   parse, so they agree by construction there; a **consumer** must check them
   anyway, because an envelope may arrive from anywhere.
2. **`run` required when `provenance: ci`, `submitted_by` when `local`, and `run`
   forbidden when `local`.** Expressed correctly in the schema with
   `allOf`/`if`/`then`. The estate's vendored validator does not implement those
   keywords and — to its credit — says so out loud rather than passing quietly:
   `NOTE unsupported keyword, not enforced — <root>: allOf`. So a full JSON Schema
   validator enforces it and ours announces that it will not.
3. **`skipped_reason` required when `outcome: skipped`.** Now also expressed, and it
   was not when this standard was first written — the section said "two rules" and
   claimed the schema expressed them, and neither was true of this one. Found by the
   converter's author reading the schema against the prose rather than trusting it.
   A skip with no reason is indistinguishable from a pass to everything downstream,
   so this was the omission with the largest consequence.

The consequence is a rule, not a caveat: **a producer validates against the schema
and then checks all three by hand, and a consumer does the same.** An envelope that
satisfies the schema is not thereby valid.

Two related corrections, both from the same reading:

- **`artifacts[].sha256` is now required.** The field table said "with a digest so a
  later copy can be checked", which reads as mandatory, and the schema had it
  optional.
- **`format: uri` was removed from the schema** in favour of a pattern. The estate's
  validator implements only `format: date` while listing `format` as known, so
  `format: uri` advertised a constraint nothing checked. `validate-schema.py` now
  reports an unimplemented format value per value — which immediately surfaced a
  second instance in `knowledge.schema.json` that had never been enforced.

## What a producer does when JUnit does not say

JUnit's `timestamp` and `time` attributes are both optional and both are genuinely
absent in practice — this repository's own gate-suite JUnit has neither.

**A producer must refuse rather than substitute.** Not `now()`, which records when
the file was converted rather than when the tests ran, and not `0`. A ledger whose
ordering is built from invented timestamps cannot answer the question it exists for.
Where the values are known from outside the file — the workflow knows when its step
started — they are passed in explicitly.

**Zero test cases is `skipped`, and needs a reason.** A JUnit with no `<testcase>`
has no natural outcome: nothing passed and nothing failed. It maps to `skipped`,
which forces the reason above, which forces a human sentence. Reporting it as
`passed` would let an empty suite look like a green one, which is the failure mode
the whole standard is arranged against.

**`timed_out` cannot come from the converter.** A job killed by a timeout writes no
JUnit to convert. That value exists for a producer that knows the job died — the
workflow itself — and a converter that emitted it would be guessing.

## What a producer must do

## What a producer must do

1. Run the tests, producing JUnit as it already does.
2. Convert: read the JUnit, emit the envelope. **One shared converter** lives in
   `infra-ci/scripts/`; a stack does not write its own, because four converters is
   four places for the totals to be computed differently.
3. **Validate the envelope it just wrote**, against the schema *and* against the two
   rules above. A producer that emits an invalid envelope has produced a lie about a
   test run, and the cheapest place to catch it is the process that wrote it.
4. Upload both the envelope and the JUnit as artifacts, with the retention that
   repository's `TESTING.md` states (STD-0008).

Emission belongs in the reusable workflow per stack (ADR-0021), never in a caller.
A caller that emits its own envelope is a caller that can emit a different one.

## What `test-registry.yaml` gains

An `artifacts` list per type, machine-readable, additive:

```yaml
- test_type: TST-0001
  evidence: junit.xml        # unchanged; prose, for a person
  artifacts:
    - kind: junit
      path: reports/junit-unit.xml
      stack: typescript
    - kind: junit
      path: target/nextest/ci/junit.xml
      stack: rust
```

`path` is **optional and stack-dependent**, which the first draft of this standard
did not allow for and which wiring the workflows immediately exposed: TST-0001 lands
at `reports/junit-unit.xml` under TypeScript and `target/nextest/ci/junit.xml` under
Rust, and one list per type could not say so. A type used by one stack names an exact
path; a type spanning stacks either omits the path — declaring only that the artifact
kind exists — or gives an entry per stack with `stack:` naming which is which.

Only `kind` is required. The registry's job is to say what a type produces; where it
lands is the producing job's business, and pretending otherwise puts a repository's
directory layout into a locked org-scope registry.

**`evidence` is never parsed.** It is a sentence — `"junit.xml, trace and video
artifacts"`, `"capacity model with the measurements behind it"` — and a machine
reading it would make that English load-bearing, which ANTI-0008 forbids by name.
The two fields coexist on purpose: one is documentation, the other is an interface,
and the mistake this standard exists to prevent is treating the first as the second.

`artifacts` is optional. A type that produces nothing collectable — a manual
review, a drill — omits it, and that omission is a fact a report can state rather
than a gap it must guess at.

## What this standard does not cover

- **Where envelopes are stored.** RFC-0034 §4 proposes a ledger; this standard is
  storage-agnostic so that changing the store does not change the format.
- **How a flake rate is computed.** RFC-0034 open question 5.
- **Whether a type not running is a defect.** A `deferred` type for a service that
  does not exist is not a gap. RFC-0034 open question 6.
- **Coverage percentages.** STD-0001 owns those, and they are a different artifact
  with a different lifetime.

## Related

- [RFC-0034](../rfc/RFC-0034-test-evidence-is-collected-and-attributed.md) — the design this implements
- [STD-0008](STD-0008-test-tiers-and-gates.md) — the tiers named in `tier`, and the Evidence rule
- [ADR-0029](../adr/ADR-0029-test-types-are-registry-data.md) · [test-registry.yaml](../test-registry.yaml) — the source of `implements`
- [ANTI-0008](../anti-patterns/ANTI-0008-rendered-text-in-a-machine-contract.md) — why `evidence` is not parsed and `skipped_reason` is not branched on
- [ADR-0021](../adr/ADR-0021-ci-is-a-reusable-workflow.md) — where emission lives
- [STD-0001](STD-0001-testing.md) — how a test is written; unchanged and assumed here
