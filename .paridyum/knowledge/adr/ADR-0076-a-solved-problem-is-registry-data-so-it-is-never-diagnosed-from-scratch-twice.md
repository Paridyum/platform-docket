---
id: ADR-0076
title: A solved problem is registry data, so it is never diagnosed from scratch twice
type: adr
status: accepted
created: 2026-09-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [debugging, incidents, knowledge-capture, problem-registry, rfc-0033, adr-0011, adr-0066, rfc-0015, c-0010]
breaking: false
enforced_by: []
---

# ADR-0076 — A solved problem is registry data

## Status

**Accepted**, 2026-09-11. Not `locked`: the obligation is a judgement ("was this non-trivial?")
that no check can make, and `problem-registry.yaml`'s shape is checkable but nothing checks it
today. It graduates when a check asserts that every entry carries the required fields and that its
`related` ids resolve.

## Context

An estate this size solves the same class of problem more than once, months apart, usually by a
different person or a fresh agent session with no memory of the first time. The cost is not the fix
— the fix is often one line — it is the **re-diagnosis**: the hours spent ruling out the wrong
causes again, because the error that shows up almost never names the thing that is actually wrong.

That last sentence is not a mood. It is the single most repeated observation across this estate's
logbooks, stated in almost those words a dozen times:

- A group membership looked "proven" by SSH; the real cause was a `Default` All→All policy carrying
  it, and a host missing from a group.
- Postgres `initdb` was evicted with an error naming a Longhorn **attach timeout on healthy
  volumes**; the cause was a 10 GiB disk and two orphaned `Retain` volumes.
- A Zitadel Helm value was silently ignored and produced *"password authentication failed"* — an
  error about authentication for a key that did not exist.
- A provisioning script got `403 error code 1010` that read as an auth failure; it was **Cloudflare
  banning the `Python-urllib` User-Agent**.
- A login showed *"unknown error occurred"*; the cause was `hasProjectCheck` locking out every
  candidate. Then `token_rejected`; the cause was a missing `resourceowner` scope, not a bad token.
  Then `ERR_TOO_MANY_REDIRECTS`; the cause was the **edge caching an auth redirect**, not the app.

Three of those were the same login, found one at a time because each only failed once the previous
was fixed. Every one of them cost a diagnosis that a written record would have shortcut to minutes.

**RFC-0033 already captures the per-repository half.** `logbook/incidents.md` records "what broke
here, what it cost, what changed afterwards" — the narrative, bound to one repository. What it does
not do, and is not meant to, is give the **next** person facing the same *symptom*, possibly in a
different repository, a reusable walkthrough: this is what it looks like, this is the wrong thing it
appears to be, this is how you confirm the real cause, this is the proven fix. That is a distinct
fact with a distinct home, which RFC-0015 requires it be given rather than smeared across a dozen
logbooks nobody cross-reads.

## Decision

### 1. Every non-trivially-diagnosed problem gets an entry in `problem-registry.yaml`

When a problem took real diagnosis — more than one wrong hypothesis, a misleading error, a
configuration or infrastructure gotcha, or anything that would cost the same time to work out a
second time — it is recorded as an entry in `knowledge/problem-registry.yaml` **at the moment it is
solved**, while the diagnosis is still in hand. Not "when there is time"; there is never time later,
and later is when the memory is gone.

A one-line typo fixed in thirty seconds is not this. A one-line fix found after ninety minutes of
ruling out the wrong causes **is** — the value is in the ninety minutes, not the line.

### 2. The entry is a walkthrough, not a note. Its shape is fixed

Each entry carries, at minimum:

- **`surface`** — what the next person actually sees. The symptom, the exact error string, the HTTP
  status, the behaviour. This is the field someone greps when they hit it, so it is written in the
  words the failure uses, not the words the cause uses.
- **`misleading_signal`** — the wrong thing it looks like. This field exists because "the error
  names the wrong cause" is this estate's defining failure mode, and naming the decoy is what saves
  the next person from chasing it.
- **`root_cause`** — what was actually wrong.
- **`diagnosis`** — the proven method that got from the surface to the cause. The commands, the log
  line, the thing to inspect. This is the walkthrough.
- **`fix`** — what resolved it, concretely, with the command or the artifact.
- **`verification`** — how it was proven fixed, so the next person knows when they are done.
- **`prevention`** — what now stops it recurring, or "none yet" honestly. A check, a test, a config,
  a comment placed where the mistake was made.
- **`related`** — the artifacts that govern the area, by id.

### 3. It does not restate the incident; it generalises the method

RFC-0015's one-home rule binds here. The registry entry is not a copy of `logbook/incidents.md`.
The logbook says *what happened in this repository and what it cost*; the registry says *how to
recognise and fix this class of problem anywhere*. Where an entry would just repeat a logbook, it
links instead. The registry is the reusable half, deduplicated across the estate.

### 4. Prevention is preferred to documentation, and documentation is the floor

An entry whose `prevention` is a real control — a check that fails, a test that catches it, a config
comment at the exact site — is worth more than one that only describes the fix. Recording the
problem is the minimum; the aspiration is that the same problem cannot recur silently. `prevention:
none yet` is an honest and acceptable value, and it is also a small backlog item.

## Consequences

**Accepted costs**

- **A few minutes at the end of every real debugging session**, spent while tired and wanting to be
  done. That is exactly when the memory is freshest and the temptation to skip is highest, which is
  why decision 1 says "at the moment it is solved".
- **Judgement about what counts**, with no check to enforce it. Some genuinely reusable problems
  will be missed, and some trivia will be over-recorded. The failure mode to avoid is the first, so
  when unsure, record it.
- **A registry that must not rot into a second incidents log.** Decision 3 is the guard, and it
  needs attention on each entry: is this the reusable method, or am I just retyping the logbook?

**Gained**

- The same problem is diagnosed once. The second person greps `surface`, reads `diagnosis`, applies
  `fix`, and is done in minutes instead of hours.
- The estate's defining failure mode — the error that names the wrong cause — gets a dedicated
  field (`misleading_signal`) and therefore a growing, searchable index of decoys.
- A fresh agent session with no memory of the first fix has the walkthrough in the repository,
  which is the same reason RFC-0033 and ADR-0011 exist: nothing depends on anyone remembering.

**Rejected alternatives**

- **Rely on `logbook/incidents.md` alone.** It is per-repository and narrative; the same symptom in
  a different repository is not found, and its reusable method is buried in prose. Kept for what it
  is good at (the local narrative); the registry holds the cross-repo playbook.
- **A wiki or an external doc.** Outside the repository is outside the one place a fresh session
  reads, and it drifts from the code it describes. The registry is versioned beside the code and
  travels with the vendored knowledge.
- **Record everything, including trivia.** Drowns the signal; the grep returns noise. The
  non-trivial bar in decision 1 is deliberate.
- **Record nothing; a good engineer remembers.** The premise this whole estate rejects (ADR-0011,
  RFC-0033, C-0010). Nobody remembers a Cloudflare 1010 they saw once, four months later, at
  midnight, in a different repository.
