---
id: RFC-0033
title: Every deliverable carries a logbook, and how it was built is captured rather than remembered
type: rfc
status: proposed
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [documentation, provenance, logbook, agent-harness, hooks, knowledge-capture, std-0002, rfc-0027, rfc-0032, c-0010]
breaking: true
enforced_by: []
---

# RFC-0033 — Every deliverable carries a logbook

## Status

**PROPOSED** as of 2026-08-19. Feedback deadline **2026-09-18**.

Governance of the governance, so it sits inside REQ-0006's scope, beside RFC-0020, RFC-0027 and
RFC-0032.

## Problem

Six documents are required or proposed for a repository in this estate, and between them they
answer six questions. **None of them answers how the thing came to be what it is.**

| File | The question it answers |
| --- | --- |
| `README.md` | What is this, can I run it in 60 seconds |
| `ARCHITECTURE.md` | How is it built, and why that way |
| `CHANGELOG.md` | What changed, and when |
| `manifest.yaml` | What is it, machine-readably |
| `SOP.md` (RFC-0032, proposed) | I have this job — what exactly do I do |
| `RUNBOOK.md` (svc/platform) | It is 3am and this is paging |

The missing question is **provenance**: what built this, under which rules, using which
capabilities, against what evidence, and what was tried and abandoned on the way. Every one of
the six above describes the artifact in its finished state. None records the process that
produced it, and in this estate that process is the larger part of the fact.

**This is not a general documentation complaint. It is specific to how this estate is built.**
`tool-docket` — 26 source files, a five-stage engagement machine, a hash-chained register and
DPDP erasure — was built in a small number of agent sessions. The knowledge of which skills were
invoked, which model produced which subsystem, which approaches were tried and discarded, and
which artifacts were consulted exists in a transcript. STD-0011 already states what a transcript
is worth: *"an agent starting tomorrow begins with what is in `knowledge/` and nothing else."*

C-0010 sharpens it. There is no prototype tier, so every repository is a production system
someone must operate, audit and defend later. **A production system whose construction is
unrecorded is a system that cannot be audited** — not for correctness, not for licence
provenance, not for the "which of these decisions was actually deliberate" question that arrives
with the first customer security review.

### The mechanism this replaces is an agent remembering

RFC-0027 has already had this argument, on the narrower subject of web sources, and reached the
answer this RFC generalises:

> Between reading a vendor's documented limit on a web page and writing it into frontmatter sits
> exactly one mechanism: an agent remembering.

Its response was to capture at the moment of the act, before anyone decides whether it mattered.
`log-web-sources.sh` does that for `WebSearch` and `WebFetch`. **Every other thing a session does
is uncaptured** — which skill was invoked, which subagent was dispatched, which model was in
use, which gate passed, which artifact was created.

### The evidence that capture-by-memory fails is in this repository

Two findings from this session, both recorded because they are the argument rather than an
aside:

1. **`research/` holds one file, dated 2026-08-18.** The hook that writes it is present in
   `plugins/paridyum-platform/hooks/` and **absent from the installed plugin snapshot at
   `~/.claude/plugins/cache/paridyum/paridyum-platform/0.1.0/hooks/`**, which carries five hooks
   where the repository carries six. This is exactly the condition
   `repo.plugin_snapshot_matches_head` warns about (ADR-0027) — *correct in git, stale on disk,
   and invisible either way.* A hook that is committed is not a hook that is running.
2. **`log-web-sources.sh` records its own capture failure in a comment**: a session started in
   the parent of the knowledge repository finds nothing and captures nothing, *"found by using
   this hook rather than by reading it."* Both defects are defects of a capture mechanism, and
   both were invisible until something went looking. Neither would have been visible at all
   under capture-by-memory, because there would have been nothing to compare against.

**Who experiences it:** every session after the one that built the thing, plus every future
audit, licence review and incident that asks *why is it like this*. **What happens if nothing is
done:** the estate accumulates production systems whose construction is recoverable only from
transcripts that are compacted, discarded, or held by a vendor.

**How we know it is fixed.** Someone who has never seen the repository can state, from the
repository alone, what built it, when, under which rules, and what was rejected — and can tell
the difference between a decision and an accident.

## Constraints

| Artifact | What it binds here |
| --- | --- |
| **STD-0002** (locked, breaking) | The required-file set. Not superseded — see alternative 4. |
| **ADR-0036** | Check parameters come from `args`, not prose. A second standard adds a required file without creating a second prose list. RFC-0032 establishes this move; this RFC reuses it rather than inventing a second mechanism. |
| **RFC-0027** (proposed) | Owns web-source capture and the append-only-log posture. This RFC generalises its mechanism and must not duplicate its subject. |
| **RFC-0032** (proposed) | Owns the operating question and `SOP.md`. Adjacent, not overlapping. |
| **RFC-0015** · **ADR-0040** | A fact has one home; cross-repository views are generated. Forbids restating in a logbook anything `manifest.yaml`, `CHANGELOG.md` or a knowledge artifact already owns. |
| **C-0004** | Generated output is never hand-edited, and a credential in history cannot be removed by a later commit. Both bind the ledger. |
| **C-0010** | No prototype tier. Every repository is a production system whose construction is auditable. |
| **ADR-0002** | `locked` requires a check that rejects the violation. |
| **ADR-0007** · **ADR-0027** | Hooks come from the versioned plugin, never copied per-repo; and reachability is checked because declaring a hook is not running one. |
| **ADR-0015** point 6 | `knowledge` is class-less and carries no `manifest.yaml`, so any rule keyed on class must say what it does here. |
| **ADR-0043** | The Stop hook commits the working tree to a per-session branch. The ledger is committed by it for free — and is therefore subject to its secret-blocking rule. |

## Proposal

### 1. `LOGBOOK.md` is required in every classed repository

A sixth required file, added the way RFC-0032 adds the fifth: a new standard citing
`docs.required_files_present` again with its own `args`, so locked STD-0002 is never contradicted
and no second prose list exists.

**`knowledge` carries one too**, notwithstanding ADR-0015 point 6. It is the repository with the
most construction history in the estate and the one where "why is this rule here" is asked most
often; exempting it because it has no class would be the sui-generis escape hatch used to avoid
the rule rather than to describe the repository.

### 2. The name, and why not the two obvious ones

**`dossier` is taken.** `product-registry.yaml` uses it for what a product owes at each stage —
an obligation set, not a record. **`docket` is taken.** `tool-docket` is the people register.
Reusing either would put two meanings on one word inside a governance system whose first rule is
that names are the cheapest coordination mechanism and the most expensive thing to change
(C-0007).

`LOGBOOK.md` is a ship's log: dated, append-only, and a record of what happened rather than of
what is owed. It matches the estate's single-uppercase-noun root-file convention.

### 3. Two halves that never mix

The logbook is **rendered**, from a generated half and an authored half, and regeneration can
never destroy authored text.

**Generated** — pulled from the one home of each fact, never hand-typed: identity and `provides`
from `manifest.yaml`; the rules that bind this repository from the knowledge artifacts whose
`scope`/`applies_to`/`product` select it; checks owed and their implemented-versus-declared split
from `check-registry.yaml`; test tiers from `test-registry.yaml`; dependencies and pins from the
lockfile; release history from `CHANGELOG.md`; age, commit count and authorship from git.

**Authored** — in `logbook/`, embedded on render, because none of it has another home:

- `logbook/origin.md` — why this repository exists as its own repository, and what was true when
  it was started.
- `logbook/decisions.md` — what was tried and rejected here that never earned an ADR. The
  sub-ADR-threshold record, which is currently lost entirely.
- `logbook/incidents.md` — what broke, what it cost, what changed afterwards.

### 4. The ledger is written by a hook, and never by an agent

`logbook/ledger/YYYY-MM.jsonl`, append-only, one JSON object per line, written by a PostToolUse
hook at the moment of the act. This is RFC-0027's mechanism with its subject widened, and it
inherits every lesson that hook paid for: `.jsonl` rather than `.log` because `.gitignore`
excludes the latter and it would exist on one disk only; built with `jq` so a quoted string
cannot corrupt a record; exits 0 unconditionally, because a provenance record that can break the
work it is recording is worth nothing; and the repository root is named explicitly rather than
guessed downward.

Six event kinds:

| Event | Records |
| --- | --- |
| `session` | start and stop: model, branch, HEAD, duration |
| `skill` | every skill invoked: name, plugin, version |
| `agent` | every subagent dispatched: type, model, task |
| `artifact` | knowledge artifacts created, accepted, locked or superseded |
| `gate` | every gate run and its verdict |
| `command` | slash commands invoked |

**Web fetches are deliberately absent.** RFC-0027 owns them and `research/*.jsonl` is their one
home (RFC-0015). The logbook cites that log; it does not copy it.

**Arguments are recorded as presence, not content.** The skill name and a bounded argument
summary are logged; a full prompt is not. C-0004 makes a credential in history unremovable, and
ADR-0043's Stop hook commits this file automatically — so an unbounded prompt capture is a
credential leak with an automatic committer attached.

**The record is evidence, so nothing rewrites it.** RFC-0027 establishes the shape: a later
process appends a derived record rather than flipping a field on an existing one.

### 5. What the ledger cannot do, stated before anyone relies on it

The ledger begins on the day the hook is installed. **Every repository in the estate predates
it.** The backfilled logbooks state their own start date and say plainly that construction before
it is recoverable only from git and from the artifacts the sessions produced. A record that
implies completeness it does not have is worse than a short one.

## Enforcement

| Check | Severity | Owner | State |
| --- | --- | --- | --- |
| `docs.required_files_present` cited again with `args: { files: ["LOGBOOK.md"] }` | error | `check-repo.sh` | implemented with the standard |
| `logbook.ledger_is_append_only` — a line already present in `logbook/ledger/*.jsonl` is changed or removed, or is not a parseable JSON object | error | `check-knowledge.sh` | declared; mirrors `knowledge.research_log_is_append_only` |
| `logbook.hook_is_registered` — the ledger hook is reachable by some path | error | `check-repo.sh` | declared; ADR-0027's reasoning, and finding 1 above is why |
| `logbook.generated_half_is_current` — the rendered half matches a fresh render | warn | `tool-forge` | declared, owed |

**The authored half is enforced by nothing**, and that is stated rather than proxied. Whether
`decisions.md` records what was actually rejected is a judgement no check can make; a
word-count heuristic would be the false confidence ADR-0002 exists to prevent.

## Alternatives considered

1. **Extend `CHANGELOG.md`.** Rejected: a changelog is deliberately user-facing and notable-only
   (STD-0005 — *"do not paste a git log"*). Provenance is neither, and the two audiences would
   fight over one file until the notable half lost.
2. **Extend `ARCHITECTURE.md`.** Rejected: it answers how the thing is built now. A record of
   what was tried and abandoned inside a document describing the current design makes the current
   design harder to read, which is the one job that document has.
3. **A central `provenance/` tree in `knowledge/`.** Rejected on RFC-0032's third ground:
   ADR-0007 vendors this repository into every other, so one tool's construction record would be
   copied into every unrelated checkout in the estate. STD-0002 also makes the repository the
   unit of documentation.
4. **Supersede STD-0002 with a merged documentation standard.** Correct in the abstract and
   rejected for RFC-0032's reason — the full supersession of a locked `breaking: true` standard
   with seven `enforced_by` entries, to add one row. Named as the eventual tidy-up. **If RFC-0032
   and this RFC both land, that tidy-up is owed twice and should be done once**, which is an
   argument for sequencing them together rather than for either doing it alone.
5. **Ask the agent to write the ledger at the end of a session.** Rejected: it is the mechanism
   RFC-0027 exists because nobody trusts, and ADR-0011 before it. A session that is compacted or
   interrupted writes nothing, and those are the sessions whose record matters most.
6. **Extend `log-web-sources.sh` rather than adding a second hook.** Rejected, narrowly. Its
   subject is RFC-0027's and its destination is `knowledge/research/`; the ledger's subject is
   every session and its destination is the repository being worked in. One script serving two
   scopes and two destinations is how a hook acquires a branch nobody tests. They share a
   library, not a file.
7. **Log every tool call, not six event kinds.** Rejected: a ledger nobody finishes reading is a
   ledger that stops working, which is the failure `log-web-sources.sh` names as the one its
   design cannot survive. Every `Read` and `Bash` call is already in git's result.

## Consequences

**Accepted costs**

- **A sixth required file, and the second proposed in one day.** Every new repository costs more
  before its first merge, and `templates/` must carry a skeleton or the template fails the gate
  it ships with. RFC-0032 and this RFC should be reviewed as a pair, because each extends the
  required-file set without owning it.
- **The required-file obligation is now stated in three places.** ADR-0036's `args` mean there is
  no second *list*, but there is a third artifact to look in for whether a file is required. This
  worsens the cost RFC-0032 already accepted, and it is the strongest argument for alternative 4.
- **The ledger is satisfiable by an unregistered hook.** Finding 1 above is the proof: the estate
  currently believes it captures web sources and does not. `logbook.hook_is_registered` raises the
  floor; it does not reach *is it firing in the session you are in right now*.
- **A per-repository ledger is a per-repository write on every skill invocation.** Small, but it
  is a hook on a hot path, and ADR-0027 already records that a slow hook is a hook people disable.
- **The authored half will be written thinnest exactly where it matters most** — under time
  pressure, on the repository being changed fastest. Nothing checks it. Same honest limit
  STD-0011 records for itself.
- **`lib-*` and `spec-*` are the weakest fit**, as they are for RFC-0032. Recorded as an open
  question rather than settled.

**Gained**

- The construction of a production system stops depending on a vendor's transcript retention.
- The sub-ADR-threshold record — what was tried and rejected without earning an artifact — has a
  home for the first time.
- Which skills and models built which subsystem becomes a queryable fact, which is what a licence
  review, a security questionnaire and a post-incident *why is it like this* all actually ask for.
- Two live capture defects were found by writing this RFC, before any new mechanism was added.

## Rollout

1. **This RFC lands alone**, per ADR-0013 §4 and `pr.design_artifact_predates_implementation`.
2. **Fix finding 1 first.** `/plugin update`, so the estate's existing capture hook actually runs.
   Adding a second uninstalled hook to an estate with one uninstalled hook records the intent to
   capture and captures nothing.
3. **A new standard**, written `accepted`, carrying the content contract and the ledger schema.
   Its id is allocated when it is written; a forward reference here would fail the gate, and
   `resolved_by` is where the link belongs.
4. **The hook**, in `plugins/paridyum-platform/hooks/`, registered in `hooks.json`, sharing the
   root-detection and emit helpers with `log-web-sources.sh` rather than copying them.
5. **`check-repo.sh`** requires `LOGBOOK.md`; **`scripts/test-checks.sh`** plants the missing-file
   defect and watches it fail, which is what makes `owner` a claim rather than a wish.
6. **Backfill, in the same change as 5**: `knowledge`, `tool-forge`, `tool-docket` and `infra-ci`
   get real logbooks, each stating the date its ledger begins. The estate must not be in
   violation of its own rule at the moment the rule lands.

## Open questions

1. **Should this and RFC-0032 be one artifact?** Both add a required file, both use ADR-0036's
   `args` mechanism, both accept the same cost, and both name alternative 4 as the tidy-up. They
   are separate here because their subjects are genuinely different — operating a thing versus
   how it came to exist — but the *mechanism* change is one change made twice.
2. **Does `spec-*` owe a logbook?** A generated contract repository's construction record may be
   entirely "generated from X at commit Y", in which case the file is a stub and the rule should
   narrow its `scope` rather than tolerate one.
3. **Does the ledger belong in git at all?** `tool-docket` keeps its register outside git so DPDP
   erasure stays honourable, and supplies integrity with a hash chain instead. The logbook ledger
   holds no personal data today. If it ever records a named human operator, that argument arrives
   with it, and the hash chain in `tool-docket`'s `domain/chain` is the answer already built.

## Related

- **RFC-0027** — owns web-source capture; this RFC generalises its mechanism and not its subject
- **RFC-0032** — owns the operating question; adjacent required file, same `args` mechanism
- **STD-0002** — the required-file set this extends without superseding
- **STD-0011** — write what will be needed again, where it will be retrieved
- **C-0010** — why a production system's construction must be auditable
- **ADR-0027** — why a declared hook is not a running hook, and finding 1 is the instance
- **ADR-0043** — commits the ledger automatically, and blocks it on a secret
