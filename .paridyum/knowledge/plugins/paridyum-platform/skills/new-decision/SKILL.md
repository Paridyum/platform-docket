---
name: new-decision
description: Write a new Paridyum ADR, constitution entry, or standard with valid frontmatter. Use when recording an architecture decision, locking a rule, or superseding an existing decision.
---

# Record a Paridyum decision

Creates a knowledge artifact that validates against `knowledge.schema.json` and slots
into the precedence order. Argument, if given, is the decision to record: `$ARGUMENTS`

## 0. The type list is registry data, not a fixed set

`artifact-registry.yaml` decides which types exist, which directory each lives in, which id
prefix it uses, and which statuses it may take (ADR-0022). Ten types today.

**Do not hand-edit the `type` enum or the four id patterns in `knowledge.schema.json.`** They are
generated from the registry, and `knowledge.artifact_registry_valid` fails when they disagree in
either direction. To add a type, edit the registry first — that is the whole point of it, and it
already caught three sites carrying a stale list on its first unassisted run.

Two types have restrictions worth knowing before you pick one:

- **`bet`** — a falsifiable commercial wager. **May never be `locked`**, because `locked` means
  must-not-be-contradicted and a bet exists to be contradicted by evidence. It must carry
  `trigger` (what would kill it) and `review_date` (when it gets judged anyway). Both are checked.
- **`glossary`** — one term, one meaning. Reach for this when two artifacts use a word to mean
  different things; `tenant` meant two different isolation boundaries before GLO-0002 existed.

## 1. Choose the type

| Type           | Directory        | Prefix | Use when                                          |
| -------------- | ---------------- | ------ | ------------------------------------------------- |
| Constitution   | `constitution/`  | `C-`   | Foundational, org-wide, generates other rules     |
| ADR            | `adr/`           | `ADR-` | A specific decision with alternatives and costs   |
| Standard       | `standards/`     | `STD-` | How to write something, not whether to            |
| Anti-pattern   | `anti-patterns/` | `ANTI-`| A shape that must never be built                  |

## 2. Allocate the id

Next unused number in that series. **Never reuse a number**, even one whose artifact
was superseded — the id is the permanent handle for the decision in git history, code
comments and other artifacts.

```sh
ls adr/ | sort -V | tail -3
```

## 3. Decide the status — this is the important step

`locked` is **only** for rules something can mechanically reject. Before writing
`status: locked`, name the check that enforces it and put it in `enforced_by`.

- Can you write a check that fails CI or blocks an edit? → `locked` + `enforced_by`
- Is it a judgement call, a boundary, a principle? → `accepted`, `enforced_by: []`

Getting this wrong is the failure ADR-0002 exists to prevent: a rule marked `locked`
that nothing enforces manufactures false confidence. `accepted` is not a weaker
commitment — it is an honest one about *how* the rule is upheld.

## 4. Write it

```markdown
---
id: ADR-00NN
title: <imperative statement of what was decided, 8-120 chars>
type: adr
status: locked | accepted
created: <YYYY-MM-DD>
locked_at: <YYYY-MM-DD or null>
supersedes: null
superseded_by: null
scope: [org | platform | service | library | tool | infra | spec]
applies_to: ["*"]
tags: []
breaking: <true if existing repos must change>
enforced_by:
  - check: <namespace>.<check_name>
    severity: error | warn
    rationale: <why this check, not just what>
---

# ADR-00NN — <title>

## Status
**LOCKED** as of <date>.  (or: `accepted`)

## Context
What forced the decision. State the competing pressures honestly — a context section
that only supports the chosen option is a rationalisation, not a record.

## Decision
What was decided, stated so an agent can act on it without interpretation.

## Consequences

**Accepted costs** — what this makes worse. Required. An ADR with no costs listed
is incomplete and probably has not been thought through.

**Gained** — what this makes better.

**Rejected alternatives** — each with the reason it lost. This is the section
future-you will actually read.
```

## 5a. The locked artifact is not wrong, its text is

Before reaching for supersession, decide which of the two you have found. They have
different costs on purpose (ADR-0031):

- **A wrong fact** — a miscount, a stale figure, a sentence that says the opposite of what
  the paragraph around it argues. Append a dated erratum. The decision is unaffected.
- **A wrong decision** — anything that changes what the artifact *decides*. Supersede it, §5.

An erratum is a `## Errata` section at the **end** of the file, appended never rewritten:

```markdown
## Errata

### 2026-08-08 — cost 2 miscounts the registries it is describing

**Says:** *"A sixth registry…"*
**Wrong:** it is the fifth — <the five, listed>.
**Intended:** *"A fifth registry."*
```

Everything above the heading must stay **byte-identical** to the committed version, and
existing errata are as immutable as the body. `knowledge.errata_are_additive_only` checks
both, so an edit to the artifact's text smuggled in alongside an erratum fails the gate.

**Do not reach for this to avoid a supersession you should be writing.** The original text
sits directly above the erratum, unaltered, which is what makes an erratum that argues with
it visibly wrong to a reader.

## 5b. Do not create an artifact as `locked`

Create it `accepted`, check the facts, then lock it in a separate commit. `status` is one of
the three fields the immutability check permits to move, so the transition is legal by
construction.

The reason is on record rather than theoretical: ADR-0029 was created `locked`, the ADR-0025
checkpoint hook committed and pushed it mid-authoring, and it was frozen before its author had
finished counting. Both of its errata exist because of that ordering.

## 5. Superseding an existing decision

Locked artifacts are immutable; a PreToolUse hook blocks content edits to them — except an
appended erratum (§5a). The legal sequence:

1. Write the new artifact with `supersedes: [ADR-00NN]`.
2. On the old artifact, edit **only** `status: superseded` and `superseded_by: ADR-00MM`.
   The guard permits that specific edit and nothing else.
3. Never delete the old file. The record of what was true when the existing code was
   written is the entire value of the archive.

## 6. Verify

```sh
scripts/check-knowledge.sh
```

Confirms frontmatter validity, that cross-references resolve, and reports any locked
artifact with an empty `enforced_by` — the enforcement backlog.
