---
id: RFC-0005
title: Where commercial strategy lives, and what shape it takes
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0022
scope: [org]
applies_to: ["*"]
tags: [governance, strategy, artifact-types, positioning]
breaking: true
enforced_by: []
---

# RFC-0005 — Where commercial strategy lives, and what shape it takes

## Status

**ACCEPTED**, resolved by ADR-0022.

## The question

C-0001 states a Vision — *"to become the operating system for businesses"* — and a Mission
— *"enable anyone to launch production-grade software businesses in minutes."* Both are one
line long, and neither says who the customer is, what the product charges for, or what would
prove either wrong.

Everything downstream of those two sentences is currently unwritten and undecided: the
customer segment, the pricing shape, whether the vertical application layer is authored or
generated, and what isolation guarantee a tenant is owed. Those decisions constrain the
schema, the deployment topology and the first service. They are being made informally, in
chat, right now.

So: **does commercial strategy belong in this repository, and if so as what?**

## Why this is not obvious

The repository's founding argument, stated in the first paragraph of README.md, is that
documentation is advisory and knowledge is enforced — that *"prose that nothing enforces is
a suggestion, and a suggestion is exactly what a long-running or freshly-started AI agent
will drift from."*

Commercial strategy is the hardest possible case for that argument. No check can verify a
market position. Nothing mechanically rejects a wrong guess about who will pay. By the
README's own test, a folder of go-to-market prose is precisely the `docs/` directory this
repository was built to not be.

And yet the alternative is worse, for a reason specific to how this repository is consumed.
CLAUDE.md instructs every agent to *"never infer architecture, retrieve it"*, and ranks
existing code as the **lowest** authority. An agent that cannot retrieve who the customer is
will infer one. It will then encode that inference in a schema, a pricing table, a tenancy
boundary — and per ADR-0002's own reasoning, once it ships it has to be undone. The absence
of a written answer is not neutral here. It is an instruction to guess.

## Options considered

### 1. A separate strategy repository

A sibling repo with looser conventions. Keeps this one purely engineering.

Rejected. C-0001 already holds the Vision and Mission, so positioning would live in one
repository and its own premise in another. An agent following CLAUDE.md would retrieve half
the answer and never learn the other half existed. That is C-0001 principle 4 — *"every rule
must be discoverable from a single repository checkout, offline"* — failing in the exact way
the principle describes. ADR-0001 vendors knowledge redundantly rather than centralising it
specifically so that one checkout is sufficient; a second repository undoes that.

### 2. Constitution amendments only

Fold positioning and pricing posture into C-0001 or a sibling constitution, and push
everything else through the existing REQ/RFC/ADR lifecycle. No schema change.

Rejected, but it is the closest call and it survives partially — ADR-0022 adopts the
constitution half of it. It fails alone because a constitution is the wrong container for a
claim you expect to be wrong. C-0001 is stable because principles are; *"agencies will pay
more than solo founders"* is a wager with a shelf life. Filing it at the highest precedence
level in the repository, where CLAUDE.md tells agents it *"must not be contradicted"*, is
how a guess acquires the authority of a decision.

An ADR is closer but not right either. C-0001 principle 8 is *"decide once, in writing, with
the cost stated"* — an ADR records a decision already taken. A market wedge is not decided,
it is bet on, and the interesting field is not the cost accepted but the evidence that would
end it. An ADR has nowhere to put that. REQ- has exactly the field, `trigger`, and ADR-0018's
argument for it transfers without modification.

### 3. A new artifact type

Add `bet` to the type enum: a falsifiable commercial wager carrying the `trigger` and
`review_date` fields REQ- already defines, and forbidden from ever being `locked`.

**Chosen.** It is the only option where the strategic claim and its own expiry live in the
same file, and where the gate can reject a claim that fails to state what would kill it.

## The complication that shaped the answer

The type list is not in one place. Adding `bet` naively means editing **fifteen** sites:

| Where | Sites | Fails how |
| ----- | ----- | --------- |
| `knowledge.schema.json` | `type` enum, and the id pattern repeated at `id`, `supersedes.items`, `superseded_by`, `resolved_by` | loudly |
| `check-knowledge.sh` | `ART_DIRS`, the id validator, the cross-reference scanner, the immutability directory filter | **silently** |
| `check-registry.yaml` | the `knowledge.frontmatter_valid` spec text | silently |
| `check-registry.schema.json` | the `blocks` id pattern | silently |
| `guard-locked-decisions.sh` | the immutable-artifact directory list — **ADR-0007 layer 1** | silently |
| `capture-open-loops.sh` | the ledger's id scanner — ADR-0011 | silently |
| `README.md`, `CLAUDE.md` | the layout and precedence blocks | silently |

Fourteen of the fifteen fail silently, and one is severe: a directory missing from
`ART_DIRS` **is not scanned at all** — no error, no warning, an entire artifact class
unenforced while the gate prints `Knowledge check passed`. This is not hypothetical; it is
the same defect class the ADR-0014 audit found four separate times in checks that were
reporting success while enforcing nothing.

**The count was twelve until the last three rows were found, and that is the strongest
argument in this document.** Two of the three had drifted already, and one of them was a
real hole in the strongest enforcement layer there is:

- **`guard-locked-decisions.sh` omitted `requirements/`.** `check-knowledge.sh` section 3
  has protected that directory since REQ- existed, so ADR-0007's layer 1 and layer 2
  disagreed about which artifacts are immutable — and layer 1 is the one CLAUDE.md
  describes as *"a hook that denies a tool call cannot be reasoned around."* A locked REQ-
  could be rewritten in place, in-session, and the guard permitted it. Reproduced against
  the pre-fix hook with synthetic tool-call JSON before it was fixed.
- **`capture-open-loops.sh` carried the old prefix set**, so a dangling reference to any
  new type would never have reached the ADR-0011 ledger — the mechanism whose entire
  purpose is that a forgotten loop gets put back in front of the next person.

Two further defects were found in `knowledge.schema.json`, both present for the life of the
repository:

- The four id patterns in `knowledge.schema.json` are independent copies. Nothing required
  them to agree. A prefix accepted by `id` but absent from `supersedes` yields an artifact
  that validates and **can never be superseded** — a one-way door built by a typo.
- `knowledge.frontmatter_valid`'s spec enumerated seven directories and omitted
  `requirements/`, which had been scanned since REQ- existed. The spec was narrower than the
  check it described.

Also confirmed, and it changed the implementation: **`scripts/lib/validate-schema.py`
implements neither `$defs` nor `$ref`, and its `unsupported()` reporter skips every key
beginning with `$`.** Factoring the repeated id pattern into `$defs` would therefore have
been silently ignored, leaving those fields with no constraint whatsoever — the validator's
own docstring says *"a constraint that is silently ignored is worse than one that is
absent"*, and this would have been an instance of it. The duplication has to stay; only a
check can hold it together.

## Recommendation

Adopt option 3, and land the type through a registry rather than a fifteen-site edit — the
fourth application of a move this repository has already made three times (ADR-0014 for
check ids, ADR-0015 for scope resolution, C-0005 for repository classes). Same defect, same
fix, and this instance is worse than the three that preceded it because almost all of its
failure modes are silent and three had already fired.

Where a site genuinely cannot be derived — the two hooks must stay fast and dependency-free,
since a YAML parse on every tool call is a hook people disable — it stays hardcoded **and
gets compared**. That is the bargain ADR-0014 struck for check ids: hardcoding is
acceptable, hardcoding with nothing comparing it is not.

Resolved by ADR-0022.

## Related

- ADR-0022 — the decision, and the registry
- ADR-0018 — requirements are first-class; the `trigger` argument this reuses
- ADR-0014, ADR-0015, C-0005 — the three prior instances of list-in-code
- C-0001 — the Vision and Mission this exists to make specific
- C-0006 — the market posture adopted once the type existed
