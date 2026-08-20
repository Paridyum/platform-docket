---
id: RFC-0027
title: Research is captured where it happens, because provenance does not survive a memory
type: rfc
status: proposed
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, provenance, sources, research, rfc-0020, adr-0011, adr-0014]
breaking: false
enforced_by:
  - check: knowledge.research_log_is_append_only
    severity: error
    rationale: >
      The checks that warn report on what a person did or did not write down, and RFC-0020 is
      right that failing a build for that gets suppressed. This one reports on whether the
      record is trustworthy at all — and a reconciliation reading a log the same run could have
      rewritten enforces nothing, which is the ADR-0014 defect class inside the mechanism
      written to close it.
  - check: knowledge.fetched_sources_are_reconciled
    severity: warn
    rationale: >
      Most fetches are dead ends. A gate that fails a session for reading a page it decided not
      to use teaches people to stop reading pages. The uncited list is a review queue, which is
      ADR-0011's posture and RFC-0020's.
  - check: knowledge.researched_artifacts_cite_sources
    severity: error
    rationale: >
      Not a review item: the type opted in, and the log proves the session read something. An
      artifact of an obliged type resting on a fact that exists only in a log is the exact
      condition RFC-0020 was written to end. No type sets it yet, deliberately.
sources:
  - url: https://code.claude.com/docs/en/hooks
    title: Claude Code — Hooks reference
    retrieved: 2026-08-19
    supports: >
      Open question 1, resolved. `hookSpecificOutput.additionalContext` IS documented for
      PostToolUse, with the exact shape decision 1 implements, so the nudge lands and
      consequence 2 no longer describes an unverified assumption. Read twice: a first pass
      inferred from the exit-code-2 table that PostToolUse cannot inject context, which the
      literal text contradicts. Recorded because the wrong answer was the more plausible one.
  - url: https://pve.proxmox.com/wiki/Cluster_Manager
    title: Proxmox VE — Cluster Manager
    retrieved: 2026-08-12
    supports: >
      The worked example throughout this RFC. Carried here because RFC-0019 rests on this page's
      5 ms corosync figure and RFC-0020 cites the same source — this RFC's own claim is that
      such a fetch must leave a trace, and an RFC making that claim while citing nothing would
      be the argument refuting itself.
---

# RFC-0027 — Research is captured where it happens

## Status

**PROPOSED** as of 2026-08-18. Feedback deadline **2026-09-17**.

Governance of the governance, so it sits inside REQ-0006's scope, beside RFC-0020.

## The question

RFC-0020 settled what a citation looks like: an external fact is declared in `sources`, with
`url`, `title` and the `retrieved` date that turns an assertion into a dated observation. Both of
its checks are implemented. The format is not the problem.

**What carries a fact from a web page into that block is an agent remembering to write it.**

That is the whole mechanism. Between reading corosync's 5 ms requirement on a wiki page and the
`sources` entry that dates it sits one act of recall, performed by a process that will be
compacted, interrupted, or simply finished. RFC-0020 concedes the gap in its own consequences —
its checks "catch the mechanical half only", and "an artifact that states a regulatory position
with no link at all passes every check in this RFC."

ADR-0011 has already had this argument in its general form. Its subject was decisions rather than
facts, and its answer was not to ask people to remember harder:

> a decision left unrecorded is put back in front of the next person to work here — whether or
> not anyone remembers it exists.

**An external fact read and never cited is the same shape.** The difference is that a forgotten
decision at least leaves the reasoning in the transcript, while a forgotten source leaves an
artifact asserting a vendor's limit in the confident present tense with nothing to re-read.

## Decision 1 — every web fetch is logged as it happens

A `PostToolUse` hook on `WebSearch|WebFetch` appends one JSON object per line to
`research/YYYY-MM-DD-<session>.jsonl`, before anyone has decided whether the page mattered:

```json
{"ts":"2026-08-18T04:12:33Z","session":"728efb92","tool":"WebFetch","kind":"fetched",
 "url":"https://pve.proxmox.com/wiki/Cluster_Manager","title":null,
 "query":"corosync latency limit"}
```

**It records rather than obliges.** The hook exits 0 unconditionally and never blocks a fetch. A
provenance mechanism that can break the work it is recording will be removed by whoever it breaks.

## Decision 2 — `fetched` and `surfaced` are different kinds, and the split is load-bearing

`fetched` is a page this session actually read. `surfaced` is a URL a search *result* offered and
nothing opened.

Only `fetched` reconciles. **One `WebSearch` offers ten results; if those reconciled, a single
search would deposit ten entries into DECISION-DEBT.md** and the ledger would stop being read
within a week — which is the only failure mode ADR-0011 cannot survive. Surfaced URLs are still
logged, because what was looked at and *not* used is the other half of an honest research trail,
and it costs a line to keep.

## Decision 3 — reconciliation appends, and what is uncited becomes an open loop

At `Stop`, `capture-open-loops.sh` compares this session's `fetched` URLs against the `sources`
blocks in every artifact, and appends a session record:

```json
{"kind":"session","session":"728efb92","fetched":2,"cited":1,
 "uncited":["https://example.com/never-cited"],"artifacts":["rfc/RFC-0027-...md"]}
```

**It appends; it never rewrites.** An earlier draft carried a `cited` flag on each fetch for the
Stop hook to flip, and that is a contradiction — the log is evidence, and evidence a later process
edits is not evidence. The outcome is derived from the record rather than written back into it.

Uncited fetches land in the **appended** half of DECISION-DEBT.md, never the regenerated half.
Only a human knows whether a fetch was a dead end or a dropped fact, so these cannot self-heal,
which ADR-0011 already says is the point of that section.

## Decision 4 — `requires_sources` gains `when_researched`

RFC-0020 open question 2 asked whether any type beyond `market` should require sources, and could
not answer it. `always` produces empty blocks on artifacts that rest on nothing external.
`never` is what let RFC-0014 and RFC-0019 each assert an external fact with no provenance at all.

The middle setting is *mandatory when the artifact was written in a session that actually read
something* — and it was inexpressible until decision 1, because nothing recorded whether research
had happened.

**No type sets it yet, and that is the decision, not an omission.** RFC-0020's question is
answerable by counting: after a month, the log says which artifact types are written in sessions
that fetch. Setting the axis now from intuition is the guess this whole mechanism exists to
replace. The value exists so the answer can be applied when it is evidence.

## Decision 5 — the log is data, and it is `.jsonl`

No frontmatter, no id, no entry in `artifact-registry.yaml`. It is read by gates the way
`check-registry.yaml` is.

**The extension is load-bearing.** `.gitignore` excludes `*.log`, and ADR-0025 commits with
`git add -A` — so a research log named that way would never be committed and would exist on one
disk only, which is precisely the loss the checkpoint hook was written in response to. Recorded
here so nobody tidies it later.

## Decision 6 — a wrong entry is voided by appending, never by deleting

A line that should not be in the log — a fixture, a mistaken capture, a URL logged against the
wrong repository — is **corrected by appending a `kind: correction` record naming what it voids
and why**:

```json
{"ts":"2026-08-19T04:11:02Z","session":"parent00","kind":"correction",
 "voids":"every preceding line in this file",
 "reason":"Not session output — a test fixture wrote these..."}
```

This was not designed. It was **forced by the mechanism refusing an author** on 2026-08-19: a
leaked test fixture was staged for deletion and `knowledge.research_log_is_append_only` blocked
the commit at the pre-commit gate. The check was right and the deletion was wrong, and the
repository had already settled the general form of the answer — ADR-0031 says a correction to an
immutable record is **additive and dated**, with the original left byte-identical. An erratum is
to a locked artifact what this is to a research log.

**The alternative is worse than it looks.** An escape hatch for "entries that were never real"
requires someone to judge which entries those are, and that judgement is made by the same party
whose record is inconvenient. A log with a delete path is a log that can be tidied, and a log
that can be tidied proves nothing about what was read.

## Consequences

**Accepted costs**

1. **`title` is null at capture time.** The hook sees a tool payload, not a parsed page. RFC-0020
   requires a title readable *without* following the link, which a synthesised one is not — so
   the field is left blank for the author rather than guessed. The reconciler reports the blank;
   it does not invent one.

2. **The nudge is best-effort, and it is now verified.** The hook returns `additionalContext`
   asking for a citation while the reason for the fetch is still known, and the shape is the one
   the hooks reference documents for PostToolUse (cited above, 2026-08-19). It stays best-effort
   regardless: a harness that does not support it ignores the object, the log is written either
   way, and the reconciliation still runs. The record never depends on the nudge landing.

3. **The ledger grows.** Every dead-end fetch is one line to tick. That is the same bargain
   ADR-0011's appended section already made, and RFC-0020 consequence 3 records the same risk for
   warnings — if the count grows past a handful, the aggregate-plus-baseline treatment applies
   here too.

4. **Scoped to `knowledge/`.** RFC-0020 deferred the reach-beyond question until a service exists
   that asserts a vendor capability, and this does not reopen it.

5. **A wrong entry is permanent, and only annotated.** Decision 6 means the log accumulates
   corrections rather than shrinking, and a reader must follow them to know what still stands.
   That is the cost of a record nobody can quietly tidy, and it is the same cost ADR-0031 accepted
   for errata.

6. **Capture needs the repository root, and cannot guess it.** Detection climbs from `cwd`, so a
   session started in a PARENT of the knowledge repository finds nothing and logs nothing — which
   is the commonest way this repository is actually worked on. `PARIDYUM_KNOWLEDGE_ROOT` names the
   root explicitly for that case. Guessing downward was rejected: a parent may hold several
   checkouts, and a hook that picks one files evidence against the wrong repository, silently.
   **A session with neither an upward match nor the variable captures nothing and says nothing**,
   which is a real hole and is recorded rather than closed by a guess.

7. **A squashed or rewritten history breaks the append-only check**, which reads `git diff HEAD`.
   That is a true positive with an inconvenient cause, and the honest remedy is not to rewrite
   history over `research/`.

**Gained**

- The provenance of an external fact no longer depends on an agent remembering, which is the same
  thing ADR-0011 refused to depend on for decisions.
- RFC-0020 open question 2 becomes a measurement rather than an argument.
- An honest record of what was read and *not* used, which no `sources` block can ever contain.

## Alternatives considered

- **A blocking gate on uncited fetches.** Rejected on RFC-0020's own argument: a check that fails
  for something nobody did wrong is suppressed within a month, and most fetches are dead ends.

- **Discipline plus a check, with no hook.** A skill obliging citation as I go, and a gate that
  fails when a session used web tools and produced no source. This is what exists today with the
  gate missing, and it is the trust-the-agent posture the whole RFC is a response to.

- **Log only, no reconciliation.** Cheapest, and it makes provenance *recoverable* rather than
  *enforced* — a searchable record nothing ever forces into the artifact resting on it.

- **In-place `cited` flags.** Rejected under decision 3: it makes the evidence editable by the
  process that reads it.

- **A separate `sources/` directory of citation artifacts.** Already rejected by RFC-0020 on
  RFC-0005's argument, and unchanged here: a fact that lives away from the decision it supports
  is a fact nobody reads while making the decision.

## Open questions

1. ~~**Does `additionalContext` land in the installed harness?**~~ **RESOLVED 2026-08-19.** The
   hooks reference documents it for PostToolUse with the shape decision 1 emits. Kept rather than
   deleted, with its answer and the date: an open question that silently disappears reads as one
   nobody asked.

2. **Should a session that captures nothing announce it?** **PARTIALLY ANSWERED 2026-08-19, and
   the remainder is not closable where it was expected to be.** `session-context.sh` now resolves
   the knowledge base through `PARIDYUM_KNOWLEDGE_ROOT` when the upward walk fails — which fixes
   the same blind spot in the hook that would otherwise have been the place to notice it — and
   confirms in the session summary where fetches will be logged, so an empty `research/` reads as
   "nothing was fetched" rather than "capture was never running".
   **The genuinely unarmed case still says nothing, and cannot.** With no upward match and no
   variable, that hook has already exited: nothing identifies the session as a Paridyum one, and
   warning anyway would fire in every unrelated project on the machine — which is how a warning
   becomes noise and then becomes ignored. The signal has to come from something that knows the
   intent, and the only such thing is the variable. Recorded as a residual hole rather than
   papered over.

3. **Which types should set `when_researched`?** Deliberately unanswered — see decision 4. Revisit
   once the log has a month of session records.

4. **Should `surfaced` URLs expire?** They accumulate forever and reconcile against nothing. A
   retention rule is easy to add and unjustified until the directory is measurably large.

## Reversal conditions

- **Reverses if the uncited queue is routinely ticked without being read**, which would make the
  ledger entries decoration. ADR-0035's baseline check is the precedent for detecting that:
  measure whether the count moves.
- **Decision 2's split reverses** if a surfaced URL is ever found to have carried a decision — that
  would mean pages are being cited from search snippets without being opened, which is a worse
  problem than this RFC addresses.

## Related

- RFC-0020 — where an external fact lands, and the half it admits it cannot catch
- ADR-0011 — the anti-forgetting ledger, whose posture and whose appended section this reuses
- ADR-0014 — declared versus implemented; why an armed check with no subject says so
- ADR-0022 — one list, not fifteen; why the URL extractor is a function with two callers
- ADR-0025 — the checkpoint that commits the log, and the loss that made `.jsonl` load-bearing
- ADR-0050 — `requires_sources` as a registry axis, which decision 4 extends
- REQ-0006 — governance of the governance, whose scope this sits inside
