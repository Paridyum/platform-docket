---
id: RFC-0034
title: Test evidence is attributed to a declared type, collected by pull, and read from one place
type: rfc
status: accepted
created: 2026-08-21
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [testing, evidence, observability, ci, test-registry, std-0008, adr-0029, adr-0014, anti-0008, req-0006, req-0001, adr-0004, adr-0055, dashboard]
breaking: false
enforced_by: []
---

# RFC-0034 — Test evidence is attributed, collected, and read from one place

## Status

**ACCEPTED** 2026-08-21, reviewed by rohanbatrain, who answered the two questions
that gated it:

- **Open question 1 — a read-only CI credential is out of REQ-0001's scope.**
  Step 5 proceeds. REQ-0001 has been updated to say so in its own words rather than
  leaving the boundary to be re-argued from this document.
- **Open question 3 — the dashboard is a new `console-` repository.** ADR-0055 has
  been updated to say that `console` covers Paridyum's own operators generally, not
  the HR department specifically.

Questions 2, 4, 5 and 6 remain open and are not blocking; each is answered by the
step that reaches it.

It was written before any code, deliberately (ADR-0013 §4): the decisions here were
cheap to change while proposed and expensive once repositories depend on an ingest
pipeline.

## Problem

**`test-registry.yaml` declares a catalogue of test types. Nothing can say which
of them run.**

Some carry `status: required` — the registry is the count, and
`report-test-coverage.sh` prints it. For each required type, these questions have
no answer today and cannot be given one without reading every workflow file by hand
and guessing:

- Does any repository run it?
- Which repositories run it, and which are silently exempt?
- When did it last pass, on which commit?
- Is it flaky, and how often?

The same holds for the `recommended` types, and for the `deferred` ones the moment
their trigger fires.

This is not a gap in vocabulary. ADR-0029 made test types registry data and
STD-0008 schedules them into its tiers with an `evidence` field per type — the
description of what should exist is complete and locked. What is missing is any
connection between that description and what actually happens.

Four observable consequences, each seen in this estate:

1. **Evidence expires and is per-repository.** `tool-docket` uploads
   nextest JUnit; `console-docket` uploads Vitest JUnit. Both go to that run's
   artifact store with a 90-day retention. After 90 days the answer to "did that
   pass on that commit" is gone, which is precisely the question STD-0008's
   Evidence section says a log cannot answer.
2. **Some evidence is produced nowhere.** `console-docket`'s Playwright suite
   covers the write path — enrol, issue, refuse, sign, advance — and runs on a
   laptop only, because issuing renders documents and no runner has a typesetter.
   Its report exists on one machine and is overwritten by the next run.
3. **A test type cannot be attributed to a job.** No workflow declares which
   declared type it implements, so the mapping is inferred, and an inference is
   exactly as good as the person making it. This is ADR-0014's finding — a
   declared check id is a tracked commitment, not a build failure — applied to a
   registry that has had no equivalent report.
4. **REQ-0006 §6 cannot be satisfied.** It requires "CI pass rate, flake rate"
   among effectiveness metrics, and states the reason plainly: "without these
   there is no evidence any of this helps, only the belief that it does." No
   number in that list is obtainable today.

The count is deliberately not written here. ADR-0035 decision 5 forbids it, and
this document would be the worst place to break that rule: a number in a design
about measuring things decays exactly as fast as the thing it measures.

**How we will know it is fixed.** A single view answers, for every declared test
type: does it run, where, when did it last pass, and how often does it flake — and
for anything it cannot determine, it says so rather than reporting a zero.

## Constraints

These are retrieved, not assumed. Each one eliminates a design that would
otherwise be obvious.

| Artifact | State | What it binds here |
| --- | --- | --- |
| **ADR-0029** | locked | Test types are registry data. A new list of types, anywhere, is a second source of truth. |
| **STD-0008** | locked | Six tiers; every blocking tier publishes a machine-readable artifact; retention is stated per repository in `TESTING.md`. |
| **ANTI-0008** | accepted, `applies_to: ["*"]` | A value a machine reads must not be a sentence written for a person. **This forbids the obvious implementation.** |
| **ADR-0004** | locked | GitHub Free, private repositories. No org-level Actions secrets; branch protection unavailable. |
| **REQ-0001** | deferred | Established by testing on 2026-08-07: org-level Actions secrets do not work for private repos on Free, so *"every repository's secrets must be set by hand, in that repository, forever."* |
| **ADR-0021** | locked | CI is a reusable workflow per stack, called by every class declaring `ci: true`. Emission belongs there, not in each caller. |
| **ADR-0055** | accepted | An operator console runs where the operator is: `deployable: false`, loopback, no identity system of its own. |
| **ADR-0020** | locked | Kubernetes for every class declaring `deployable`. Anything deployable here owes probes, limits and a chart. |
| **ADR-0014** | locked | A declared id is a tracked commitment. The precedent for reporting declared-versus-implemented rather than hiding it. |
| **ADR-0035** | locked | Counts in prose are snapshots. This document cites the registry rather than restating its cardinality. |
| **REQ-0006** | accepted | §6 requires effectiveness metrics including CI pass rate and flake rate. This RFC partially discharges it. |

### The constraint that decides the architecture

`REQ-0001` records that org-level Actions secrets do not work for private
repositories on the Free plan, and that the consequence is per-repository secrets
set by hand forever. Repository-level secrets **do** work.

That single fact rules out the design most people would reach for first:

**Push is rejected.** If each repository sends its results to a central store, each
repository needs a write credential for that store. That is one hand-set secret per
repository, growing with the estate, of exactly the invisible per-repository drift
REQ-0001 warns about — a repository with a stale credential stops reporting and
nothing notices, because absence of evidence looks identical to absence of tests.
It is also a worse security posture: a write credential in every repository means
any repository can write any other repository's results.

**Pull is chosen.** One repository fetches from the GitHub API. That is exactly one
hand-set secret, in one place, read-only. No repository can forge another's
results, because no repository can write at all. A leak of that credential exposes
read access to things the holder could already read.

`rust-ci.yml` records the same lesson learned the hard way: five jobs once tried to
check out `infra-ci` to reach its scripts and every one failed with
`remote: Repository not found`, because the default `GITHUB_TOKEN` carries
`Contents: read` for the **calling** repository only. Cross-repository access is
not a detail to be arranged later; it is the shape of the problem.

## Proposal

Four parts. Parts 1 and 2 are governance and cost nothing to run. Part 3 works
with no stored credential at all. Part 4 is the only part that needs a human to
approve a secret, and it is sequenced last for that reason.

### 1. A job that runs tests declares which type it implements

The mechanism that turns a guess into a fact.

`test-registry.yaml` gains one field per type, machine-readable and additive:

```yaml
- test_type: TST-0001
  name: unit
  evidence: junit.xml            # unchanged: prose, for a person
  artifacts:                     # NEW: machine-readable
    - kind: junit
      path: reports/junit-unit.xml
```

`evidence` is left exactly as it is and **is never parsed**. It is a sentence
written for a person — `"junit.xml, trace and video artifacts"`,
`"capacity model with the measurements behind it"` — and a collector that read it
would make that English load-bearing, which is ANTI-0008 by its own definition. The
new `artifacts` field is the machine contract; the prose stays documentation.

The reusable workflows then emit, per test job, one **run envelope**: a single JSON
object naming the repository, the commit, the tier, the TST- ids the job
implements, the outcome counts, and where the raw artifact went. The envelope is
uploaded as an ordinary CI artifact alongside the JUnit it summarises.

Emission lives in `rust-ci.yml` and `nextjs-ci.yml` (ADR-0021), so no caller
repository implements it and no caller can implement it differently.

### 2. One envelope shape, one converter, every language

JUnit XML is the only format every stack already produces. What was actually
verified on 2026-08-21, stated at the strength it was verified:

- **Vitest — demonstrated.** `console-docket` already writes
  `reports/junit-unit.xml`, and a real file from it was converted successfully.
- **Playwright — demonstrated.** `--reporter=junit` accepted, valid JUnit produced.
- **cargo-nextest — declared, not demonstrated.** `tool-docket`'s
  `.config/nextest.toml` carries `[profile.ci.junit]`, so the configuration is
  there and CI has produced JUnit from it. `cargo nextest` is **not installed on
  this machine**, so no file was converted locally. An earlier draft of this section
  said "verified against what is actually installed" and that was wrong about this
  one.
- **pytest — neither.** `--junit-xml` is standard and there is no Python repository
  in the estate to try it against. Python is anticipated by construction; the first
  Python repository is where the claim gets tested.

The distinction matters because the design rests on it. If a stack turns out not to
emit JUnit, "one converter for every language" becomes "one converter plus an
emitter per exception", which is alternative F below, rejected.

So the converter reads JUnit and writes the envelope, and it is **one
implementation** in `infra-ci/scripts/`, language-agnostic, rather than one per
stack.

That is the whole answer to "Python, Rust, TS or whatever language": the envelope
is the interface, JUnit is the adapter, and a new stack is a new CI job that emits
JUnit — not a new collector, not a new schema, not a new dashboard.

Playwright is included by the same route. It emits JUnit when asked, so a local run
can produce an envelope too — see §4 on what that means for trust.

The envelope shape is specified in a companion standard rather than here, because
it is a format that will change and this document should not have to be superseded
to add a field.

### 3. The dashboard reads live state with the operator's own credentials

A `console-` repository under ADR-0055 — class `console`, `deployable: false`,
loopback bind, no identity system of its own, exactly like `console-docket`.

It answers the questions in the Problem section by querying the GitHub API for
workflow runs and their envelopes, **using the operator's own `gh` credentials**.
No stored secret, no ingest, no store, nothing to deploy, and it works the day it
is written.

It renders, per declared test type: runs / does not run / **cannot determine** —
and the third is a first-class outcome shown as prominently as the other two. A
dashboard that renders an undeterminable row as a zero is the failure this whole
document exists to prevent, one level up.

Its limitation is stated rather than discovered: artifacts expire at 90 days, so it
shows the recent past only, and it cannot answer anything about a run whose
artifacts have gone.

### 4. Durable history, by pull, into an append-only ledger

The only part needing a credential, and therefore the last.

A scheduled workflow in one repository fetches envelopes via the GitHub API with a
**read-only, fine-grained** token (Actions: read, Contents: read, metadata: read,
across the organisation) held as that one repository's secret. It appends each
envelope to `ledger/YYYY-MM.jsonl`, append-only and hash-chained in the manner
`tool-docket` already uses for four ledgers: each entry carries `prev_sha256` and
`entry_sha256`, so a rewritten history is detectable rather than merely
discouraged.

The dashboard then reads the ledger for history and the API for the live view, and
says which is which. A number that came from a ledger and a number that came from
a live query have different trust properties, and a view that blends them silently
is lying about one of them.

**Local runs are accepted and labelled, never trusted.** A Playwright envelope
produced on a laptop carries `provenance: local` and the operator who submitted it;
one produced by CI carries `provenance: ci` and its run id. A local result may
never satisfy a gate and may never be the reason a required type shows as running.
Rejecting local runs outright would mean the write path — the most valuable suite
in the estate — remains invisible; accepting them unlabelled would mean the ledger
cannot be trusted at all. This is the same shape as the operator attribution in
`console-docket`: record the claim, record that it is a claim, and never let it
carry more weight than it has earned.

## Alternatives considered

**A. Push from each repository to a central store.** Rejected on REQ-0001's
finding: one hand-set write credential per repository, growing with the estate,
with silent failure when one goes stale, and any repository able to forge any
other's results. Discussed at length under Constraints because it is the design
most likely to be proposed again.

**B. A service with a database, and a real dashboard over it.** The eventual right
answer and wrong now. It is `deployable`, so locked ADR-0020 obliges Kubernetes
probes, limits and a Helm chart; there is no cluster (RFC-0019 is still proposed);
and it needs runtime secrets, which is REQ-0001, deferred. Building it now means
building the cluster first. The envelope in §2 is deliberately storage-agnostic so
that this becomes a migration rather than a redesign.

**C. Parse `evidence` and skip the new field.** Rejected as ANTI-0008: the field is
prose written for a person, and parsing it makes the English load-bearing —
renaming `"junit.xml, trace and video artifacts"` would then break a collector.
Cheaper today, and it converts a documentation string into an interface nobody
declared.

**D. A third-party test-reporting service** (Datadog CI Visibility, BuildPulse,
Trunk, Currents). Genuinely good products that solve this exact problem. Rejected
on REQ-0012's exitability test — the same test that rejected Terraform in ADR-0052
and Sidero Omni in RFC-0025 — and because they need the credential problem solved
anyway, plus personal data in test names leaving the estate for a pre-incorporation
company with no DPA in place. Worth revisiting after incorporation, at which point
§2's envelope is what feeds it.

**E. Nothing; keep reading workflow files by hand.** The status quo, and it should
be said out loud that it is survivable at four repositories. It is rejected because
the cost is not the reading — it is that nobody does it, so nobody knows, and every
required type has been unverified since the day it was declared.

**F. One test-result format of our own, instead of JUnit.** Rejected: every stack
already emits JUnit, so inventing a format means writing an emitter per language
and maintaining four of them. JUnit is poor — it carries no tier, no commit, no
type — and the envelope's whole job is to add exactly those without replacing what
already works.

## Consequences

Required section, and the honest list.

**This makes some things worse:**


- **A new field in a locked registry's schema.** `test-registry.schema.json`
  changes, and every declared type eventually needs an `artifacts`
  entry or an explicit statement that it produces nothing collectable. Until then
  the registry is in two states at once, which is precisely the condition ADR-0014
  exists to make visible rather than tidy away.
- **The dashboard can be wrong in a new way.** Today nobody knows which types run.
  Afterwards there is a screen that says, and a screen that says is trusted. Its
  `cannot determine` state is load-bearing and will be the least-loved part of the
  interface.
- **One more repository, one more CI pipeline, one more thing to keep green.**
- **A read-only organisation-wide token exists** where none did. Its blast radius
  is read access to private repositories — source, workflow logs, artifacts. That
  is not nothing, and it is the single item in this RFC that a human must weigh
  rather than an agent.
- **Emission adds time to every test job.** Small — reading a JUnit file and
  writing JSON — but it is on the critical path of the `pr` tier, which STD-0008
  holds to a ten-minute budget.
- **Local envelopes create a category of second-class evidence.** Two kinds of
  number in one system is a permanent explanation cost, paid every time someone
  reads the dashboard.

**It makes these things better, and they are the point:** every required type
becomes verifiable; evidence outlives its 90 days; the write path stops being
invisible; and REQ-0006 §6 becomes reachable rather than aspirational.

## Rollout

Ordered so that each step is useful alone and revertable alone.

1. **The inventory report** — a read-only script answering "which declared types
   appear to run", by heuristic, saying per row that it is a heuristic. Needs no
   decision from this RFC and makes the problem visible with real numbers. *This
   step is not significant under ADR-0013 and may proceed immediately.*
2. **The envelope standard** — a `STD-` artifact specifying the shape, and
   `artifacts:` added to `test-registry.schema.json`.
3. **Emission** in `rust-ci.yml` and `nextjs-ci.yml`, plus the shared converter.
   At this point envelopes exist as CI artifacts and nothing consumes them.
4. **The dashboard**, reading live state with the operator's credentials. Useful
   without steps 5–6 and requires no secret.
5. **The ledger and the scheduled pull** — the credential decision, taken by a
   human, in its own change.
6. **Local submission** for Playwright and anything else CI cannot run.

**Backwards compatibility.** Steps 1–4 change no existing behaviour: `evidence`
keeps its meaning, existing artifacts keep being uploaded, and a repository that
declares no `artifacts` simply reports as undeterminable.

**How it is undone.** Steps 1–4 are deletions. Step 5 is a deletion plus revoking
one token. Step 6 is a deletion. Nothing here migrates data that cannot be
regenerated, because the envelope is derived from artifacts that CI produces anyway.

## Open questions

An honest list is worth more than a confident guess.

1. ~~**Does a read-only organisation-wide PAT belong under REQ-0001?**~~
   **RESOLVED 2026-08-21: out of scope.** REQ-0001 governs secrets a *running
   service* needs, and its trigger is the first deployment. A read-only CI
   credential that reads only what its holder can already read is a different thing,
   and REQ-0001 now says so directly. Step 5 proceeds.

   What the decision does *not* do is make the token free: it is still one hand-set
   secret in one repository, still the item in this design with the widest blast
   radius, and it is still listed under Consequences.
2. **Which repository holds the ledger?** A new one, `infra-ci`, or `knowledge`.
   `knowledge` is tempting and probably wrong: it is the source of policy, and
   mixing an append-only stream of run data into it would make every governance
   diff noisy.
3. ~~**What class is the dashboard?**~~ **RESOLVED 2026-08-21: a new `console-`
   repository.** ADR-0055's `console` class is now stated to cover Paridyum's own
   operators generally rather than the HR department specifically, which is what it
   always meant — the reachability test it turns on ("if tenant traffic can reach
   it, it is not a console") never mentioned who the operator was.

   It also reuses everything `console-docket` established: the reusable Next.js CI
   workflow, the supply-chain posture, the palette contrast test, and the loopback
   access model. A second class would have duplicated all of it to describe the
   same shape.
4. **Retention of the ledger.** STD-0008 puts retention in each repository's
   `TESTING.md`. A central ledger has no such home, and "keep everything forever"
   is a decision rather than a default — test names can contain personal data.
5. **Does a flake rate need a definition before it can be measured?** REQ-0006 asks
   for one. FLAKY-002 and FLAKY-003 exist in the test registry; whether their
   definition is what the dashboard should compute is unresolved.
6. **What happens when a declared type is legitimately not applicable?** A
   `deferred` type for a service that does not exist is not a gap. Distinguishing
   "not run and should be" from "not run and correctly so" may need another
   registry field, and may be the same shape as ADR-0014's `declared` state.

## Related

- [ADR-0029](../adr/ADR-0029-test-types-are-registry-data.md) · [test-registry.yaml](../test-registry.yaml) — the vocabulary this collects against
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) — the tiers, the gate, and the Evidence rule this implements
- [ANTI-0008](../anti-patterns/ANTI-0008-rendered-text-in-a-machine-contract.md) — why `evidence` is not parsed
- [ADR-0014](../adr/ADR-0014-declared-check-ids.md) — the precedent for reporting declared-versus-implemented
- [REQ-0006](../requirements/REQ-0006-governance-of-the-governance.md) §6 — the effectiveness metrics this partially discharges
- [REQ-0001](../requirements/REQ-0001-runtime-secret-management.md) — the secret constraint that chose pull over push
- [ADR-0004](../adr/ADR-0004-github-org-on-free-plan.md) — the plan limits behind it
- [ADR-0021](../adr/ADR-0021-ci-is-a-reusable-workflow.md) — where emission lives
- [ADR-0055](../adr/ADR-0055-an-operator-console-runs-where-the-operator-is.md) — the class the dashboard would take
- [ADR-0020](../adr/ADR-0020-kubernetes-deployment-target.md) — why alternative B waits
- [REQ-0012](../requirements/REQ-0012-vendor-and-compliance-governance.md) — the exitability test that rejected alternative D
