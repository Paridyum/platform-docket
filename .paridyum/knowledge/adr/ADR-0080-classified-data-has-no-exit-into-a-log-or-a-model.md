---
id: ADR-0080
title: Classified data has no exit into a log or a model
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service]
applies_to: ["platform-*", "svc-*"]
tags: [privacy, logging, ai, ml, dpdp, classification, req-0003, adr-0044, adr-0073, anti-0007]
breaking: false
enforced_by: []
---

# ADR-0080 — Classified data has no exit into a log or a model

## Status

**Accepted**, 2026-09-12. Closes the two OWED items in REQ-0003's activation: item 3 (never in
logs/traces) and item 9 (AI/ML use). Not `locked`: the logging half is enforced today by a lint that
forbids raw `console`, which is a proxy — a determined caller can still construct a log line by hand
— and the ML half is enforced by there being no model and no pipeline, which a check cannot assert.
It graduates when a structured-logging check and a data-flow check exist.

## Context

REQ-0003 classifies every candidate column (`public` / `internal` / `confidential` / `pii`,
ADR-0073) and item 1 makes that classification a property of the schema. A classified field then has
exactly the destinations the schema and the application give it: it is written by the profile form,
read by an operator evaluating a candidate, and deleted by the retention clock. REQ-0003 names two
destinations it must **not** acquire, and both were left OWED at activation because neither is a
schema property — they are things code does:

- **The log line (item 3).** "The most common breach is not a stolen database; it is an email
  address in a log line replicated to three vendors." A logger accepts `any`, so `log(profile)`
  type-checks, ships the whole entity to whatever sink is attached, and that sink is routinely a
  third party outside the residency and retention story the rest of REQ-0003 so carefully builds.

- **The training set (item 9).** A candidate's data is the kind that invites a model — score the
  applicant, rank the pipeline, "learn what a good hire looks like." That is automated decision-
  making over `pii` about a natural person, it is a DPDP-regulated use, and REQ-0003 item 9 requires
  it to be answered — *which datasets, and by whom* — before it happens, not after a model already
  holds a copy.

Both are the same shape: a classified field leaking out of the lane the schema drew for it. So they
are decided together.

## Decision

### The log line

1. **A classified field is never a log field.** Nothing classified `internal`, `confidential` or
   `pii` is passed to a log, trace, metric label, analytics event or error tracker — not the value,
   and not an object that contains it.

2. **Logging is structured and typed, and it takes primitives, never entities.** When logging is
   added it goes through one event logger whose signature accepts a stable event name and a map of
   explicitly-named primitive fields (`string | number | boolean`), never a row, a request body or a
   domain object. The safe path is the only path: you cannot hand it a `profile`, so you cannot leak
   one through it.

3. **An error carries a correlation id, not the payload.** ADR-0044 already gives every request a
   correlation id; a failure logs that id and a typed reason, and the operator joins to the request
   out of band. This is why the candidate-facing errors are already generic ("could not record") —
   the diagnostic is the id, not the data.

4. **Raw `console` is forbidden by lint.** `no-console` is an error in the web application. There is
   no console usage today, so the rule costs nothing and prevents the casual `console.log(entity)`
   from ever being the first one. Removing the rule to add a log line is a visible act in a diff.

### The model

5. **Candidate profile and application data is not used to train, fine-tune, evaluate, or benchmark
   any model, and there is no automated decision-making or profiling over it.** No eligibility score,
   no ranking model, no "candidate quality" signal derived by a model from these rows. *Which
   datasets may be used for training or evaluation:* none. *By whom:* no one. That is the current,
   complete answer to REQ-0003 item 9.

6. **Introducing any such use is a new accepted decision, not an implementation choice.** A future
   that wants a model over candidate data supersedes this ADR, states the lawful basis and the DPDP
   notice, states retention and the subject's rights against automated decisions, and passes
   ANTI-0007's controls-before-regulated-data gate. Until that decision exists, a pipeline that reads
   these rows into a model is a governance violation, not a feature.

7. **Operator evaluation by a human is not "AI/ML use."** An operator reading a profile to make a
   hiring judgement is the purpose the data was collected for (ADR-0073 D1). This ADR closes the
   *automated* exit, not the human one.

## Consequences

- The web application's eslint config forbids `console`; the typed event logger is specified here and
  is built when the first log line is actually needed, not before (no unused machinery — C-0006's
  no-capability-without-a-buyer applied to our own tooling).
- REQ-0003 items 3 and 9 move from OWED to MET, with the honest note that both are enforced by a
  proxy and an absence rather than by a check, and each names the check that would lock it.
- Any later analytics, observability vendor, or model integration meets these two decisions first.
  An observability tool that ingests request bodies is not adoptable as-is; a model over candidate
  data is not adoptable at all without superseding decision 5.

## Related

- **REQ-0003** — the requirement whose last two OWED items this closes.
- **ADR-0073** — the per-column classification this depends on; the candidate profile's data lanes.
- **ADR-0044** — the correlation id that a failure logs instead of the payload.
- **ANTI-0007** — regulated data before controls; the gate any future model use must pass.
