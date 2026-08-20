---
id: RFC-0029
title: A derivation is an artifact, and an estimate is writable only if it is labelled
type: rfc
status: proposed
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [strategy, unit-economics, pricing, artifact-type, registry, assumptions, adr-0048, adr-0035, adr-0022, c-0007]
breaking: false
enforced_by: []
---

# RFC-0029 — A derivation is an artifact

## Problem

**No number exists anywhere in this repository, and the absence is structural rather than
lazy.**

The observable facts, all of them current on 2026-08-19:

- `BET-0007` fixes the price *shape* — a flat floor plus a royalty — and states no floor and no
  royalty rate. Its trigger clause 1 is *"the flat fee fails to cover fully-loaded per-tenant
  cost at the median tenant"*, which cannot be evaluated because neither side of the comparison
  is written down.
- `BET-0008` trigger clause 1 requires substrate cost per tenant to be *"at least half the
  managed-cloud equivalent"*. Same defect: a ratio with no baseline.
- `BET-0009` trigger clause 3 sets a sixty-percent gross margin floor per node. Nothing records
  what the current or projected margin is.
- `ROADMAP.md` §6 is titled *"The measurement that unblocks pricing"* and states the diagnosis
  directly: *"BET-0008 open question 1 and T-0.7 are the same missing number: the fully-loaded
  cost of one tenant."*
- The Phase 0 exit gate — which blocks the first runtime commit — reads *"floor and royalty
  set"*. It is open.

**Who experiences it:** every agent and every human who has to price, size, or sequence
anything. `CLAUDE.md` instructs agents to *never infer architecture, retrieve it*. An agent
that cannot retrieve the cost model will infer one, and then encode that inference in a plan
table or a quota default. RFC-0005 already made this argument for strategy: **the absence of a
written answer is not neutral here; it is an instruction to guess.**

**How often:** every time a bet's trigger is evaluated, which is the mechanism by which this
repository's strategy is supposed to be falsifiable at all. Nine bets carry triggers; four of
them are arithmetic against quantities that have no home.

**Why the numbers are absent, stated as a structural claim rather than a complaint.** The three
business artifact types divide cleanly by their load-bearing field, and none of them will accept
a derived quantity:

| Type | Load-bearing field | Answers |
| --- | --- | --- |
| `market` | `sources` | What is true of the world, and how do we know? |
| `bet` | `trigger` | What are we wagering, and what would kill it? |
| — | — | **What follows arithmetically from those, and under which assumptions?** |

A cost-per-tenant figure is not an observation: it is not checkable against the world today,
because it depends on a density this company has never measured. It is not a wager either: it
has no falsification date, it has a *calculation*. Written as an `MKT-` it would be an opinion
with frontmatter — the failure that type's own registry notes name. Written as a `BET-` it
would be a wager whose trigger is itself the number, which is circular.

**So the number is not written down at all**, and the honest description of the current state is
that the company's unit economics exist only as arithmetic nobody has performed twice the same
way.

**How we will know it is fixed:** `T-0.7` can be closed with a citation rather than a meeting,
and each of the four trigger clauses above names an artifact holding the quantity it compares
against.

## Constraints

Locked and accepted artifacts that bind this design. Each is cited because it removes an
option, not as decoration.

**`ADR-0048` — a value that changes on a cadence we do not control is registry data.** This is
the tightest constraint and it partly dictates the answer. Its rule 2 gives the test as one
question: *does this change on a schedule I do not set?* Applied here, the quantities split
three ways, and conflating them is the mistake this RFC most needs to avoid:

| Kind | Example | ADR-0048 says |
| --- | --- | --- |
| Externally cadenced input | Hetzner €/TB, aggregator fee percentage | registry data, or dated evidence |
| Parameter we choose | target margin, the floor we set | artifact — we set the cadence |
| The derivation itself | how cost-per-tenant follows from density | artifact — it is a rule |

Its rule 3 also binds the body of any artifact this RFC creates: a value may appear *only as an
illustration that names its source*. `STD-0003`'s `channel = "1.90.0"` is the worked example of
what happens otherwise, and the erratum appended to it *labels the trap and cannot remove it*.

**`ADR-0035` — counts in prose are snapshots, not claims.** A derivation is made almost entirely
of numbers in prose, which makes this the artifact type most exposed to that ADR. The gate
already reports `48 numeric claim(s) about registry-computed things` as a review list. Any new
type must not inflate that list with numbers that are *not* registry-computed, or the warning
loses its meaning.

**`ADR-0022` and `C-0007` — the grammar is constitutional, the list is data.** A new artifact
type is added through `artifact-registry.yaml` and nowhere else. Fourteen of fifteen hardcoded
sites failed silently the last time this was done by hand.

**`ADR-0002` and `ADR-0014` — `locked` requires a check that rejects the violation**, and a name
in `enforced_by` is a commitment rather than a control.

**`ADR-0050`** established the `product` axis, which is live and, as of 2026-08-19, referenced by
zero artifacts. Anything created here should use it or explain why not.

**`scripts/lib/validate-schema.py` implements neither `if`/`then` nor `allOf`**, and reports
unknown keywords as unenforced. This is not a decision but it is a hard constraint, and it is
the reason `requires_sources` is a registry field rather than a schema conditional. Any per-type
requirement introduced here must follow the same route or it will advertise a constraint that
nothing applies — the defect `market`'s registry notes record at length, discovered by planting
a sourceless `MKT-` and watching it validate clean.

## Proposal

**1. A new artifact type `model`, prefix `MOD-`, directory `models/`,** declared in
`artifact-registry.yaml` and nowhere else:

```yaml
  - type: model
    prefix: MOD
    directory: models
    lockable: false
    deferrable: false
    requires_trigger: never
    requires_sources: never
    requires_assumptions: always
    immutable_when_locked: true
    description: A derivation — what is computed, from which assumptions, and what it decides.
    declared: 2026-08-19
```

**`lockable: false`**, for the reason `bet`, `rfc` and `market` each carry it, reached here from
a fourth direction: `locked` means must-not-be-contradicted, and a model exists precisely to be
contradicted by measurement. A locked model would be a spreadsheet with constitutional
authority, and it would outrank the first real number that disagreed with it.

**`immutable_when_locked: true`** is unreachable through `locked` and deliberately still true,
exactly as `bet` and `market` record. It fires on `superseded`, which is how a model ends. **A
model that turned out wrong is the most valuable one this company will own and the easiest to
quietly rewrite** once the price it set is already being charged — which is the same argument
`market` makes about evidence that selected a vertical already being built.

**`requires_sources: never`**, which is a genuine distinction and not an oversight. A model's
external inputs are cited *through* the `MKT-` artifacts its assumptions name, so requiring
`sources` on the model itself would produce either duplication or a second home for the same
fact, and `RFC-0015` says a fact has one home.

**2. A new registry field `requires_assumptions`**, taking `always` or `never`, and a new
schema property `assumptions`:

```yaml
assumptions:
  - name: memory_cost_per_gb_month
    value: "EUR 0.60"
    basis: MKT-0001
    feeds: fully-loaded tenant cost — the denominator T-0.7 is blocked on
  - name: tenants_per_node
    value: "40"
    basis: estimate
    feeds: density; BET-0008 trigger clause 1
```

`basis` takes an `MKT-` id, a registry key of the form `<registry>.<path>`, or the literal
string `estimate`.

**3. `basis: estimate` is the load-bearing decision of this RFC**, and it is the part most
likely to be argued with, so it is argued for here.

Today an unfounded number cannot be written into any artifact in this repository, because every
container for a number demands a provenance the number does not have. The intended consequence
was rigour. **The actual consequence is that the number is not written down, and is therefore
guessed silently and repeatedly, differently each time.** Making a guess writable *and
mechanically visible as a guess* is the same bargain `ADR-0018` struck for requirements: a
`deferred` requirement with a `trigger` is honest, and the alternative was never a scheduled
requirement — it was a `TODO` nobody could see.

An estimate is therefore permitted, labelled, countable, and reportable. It is not permitted to
be invisible.

**4. Three checks**, declared in `check-registry.yaml` with `check-knowledge.sh` as owner:

| Check id | Rejects |
| --- | --- |
| `knowledge.model_artifacts_declare_assumptions` | a `MOD-` with an empty or absent `assumptions` — the sourceless-`MKT-` failure, pre-empted rather than discovered |
| `knowledge.model_assumption_basis_resolves` | a `basis` naming a dangling, retired or superseded `MKT-`, or a registry key that does not resolve |
| `knowledge.model_estimate_ratio_is_reported` | nothing; it reports estimates against cited bases per model, so that a model which is nine-tenths guesswork says so on every run |

The third blocks nothing by design. `ADR-0002`'s bar is a check that rejects a violation, and
"too many estimates" is a judgement rather than a violation — naming it as enforcement would be
the false confidence ADR-0002 exists to prevent.

**5. Every check is verified by planting the defect it should catch**, before it is claimed to
work. This repository's most expensive lesson was that checks reported success while
enforcing nothing, and its own conclusion was: *treat a green check as unproven until you have
watched it fail.*

**6. `MOD-` artifacts declare `product:`.** `ADR-0050`'s axis has no references today; a cost
model is naturally per-product, and this is the type that should populate it first.

## Alternatives considered

**A. A registry only — `model-registry.yaml`, no new artifact type.** Rejected, though it is the
most `ADR-0048`-native option and was close. A registry holds a list; a model is an argument —
what it computes, what it deliberately ignores, and why the structure is the right one. There is
nowhere in a registry to record *"this model excludes support cost because BET-0009 clause 2
measures it separately"*, and that sentence is more valuable than any figure in the model.
Decisively: registry entries cannot be superseded, and a superseded model is the most
instructive artifact in the set. `immutable_when_locked` has no analogue in YAML.

**B. A type carrying everything, vendor figures included.** Rejected as a direct violation of
`ADR-0048` rule 1. Every vendor price inside the body would rot, and correcting it would require
supersession — heavier than the correction, which is the exact trap `STD-0003` is in and cannot
get out of.

**C. Extend `market` to carry derived numbers.** Rejected. It re-merges the distinction that type
was created to draw. `market`'s registry notes give the test — *whether the sentence could be
checked against the world today (MKT-) or only judged later (BET-)* — and a derivation answers
neither: it is checked against its own assumptions. Overloading `sources` to sometimes mean
"where I read this" and sometimes "what I computed this from" would put two meanings on one
field, which is the failure `GLO-0002` records for the word `tenant`.

**D. Extend `bet` with an `assumptions` field.** Rejected. A bet's `trigger` is its falsification
condition, and several triggers *compare against* the model. A bet holding its own assumptions
would let the wager move its own goalposts silently — the artifact would supersede itself into
correctness, which is precisely what `immutable_when_locked` on `bet` exists to prevent.

**E. Do nothing; put the numbers in `ROADMAP.md` §6.** Rejected, and worth stating because it is
the path of least resistance and the status quo. `ROADMAP.md` has **zero references in
`check-registry.yaml`** as of 2026-08-19. Numbers placed there are unvalidated prose in a
repository whose founding argument is that unenforced prose is a suggestion, and they would
carry no provenance, no supersession and no estimate labelling.

## Consequences

**What this makes worse**, required by ADR-0013 and not softened:

**1. A twelfth artifact type, and type proliferation has a real cost.** Every type is one more
thing an author must choose between correctly, and the `MKT-`/`BET-` boundary already needs a
written test to be usable. `MOD-` adds a third neighbour to that boundary. The mitigation is a
test stated in one sentence and put in the registry notes: **an observation is checked against
the world, a wager is judged later, a derivation is computed from named inputs.** It is a
mitigation and not a fix; some artifacts will be filed wrong.

**2. `requires_assumptions` is the second field of its shape, and there is now a pattern
forming** — `requires_trigger`, `requires_sources`, `requires_assumptions`. A third is a
generalisation waiting to be found, and adding this one without finding it may make the eventual
generalisation more expensive. This is accepted deliberately rather than pre-solved: two
instances is not enough to see the shape, and inventing the abstraction now is the error
`ADR-0022` diagnosed from the opposite direction.

**3. `basis: estimate` will be abused.** It is the path of least resistance for an author who
does not want to find a source, and nothing in this design prevents a model composed entirely of
estimates. `knowledge.model_estimate_ratio_is_reported` makes that visible and blocks nothing.
**This is a genuine weakening of the repository's standard of evidence, accepted knowingly**,
because the alternative in force today produces zero numbers rather than honest ones.

**4. A model can be internally consistent and entirely wrong.** No check can evaluate whether a
derivation is *sound* — only that its inputs resolve. `verified_by` cannot close this distance
either, because there is no running software to test. Until density is measured, a `MOD-` is a
structured guess, and calling it a model risks it acquiring authority its evidence does not
support. The status field is the only defence and it is a weak one.

**5. Implementation touches the generated type-enum sites**, which is where `ADR-0022` found
fourteen silent failures. The work is mechanical but the blast radius is the whole gate.

## Rollout

1. `artifact-registry.yaml` — declare the type and the `requires_assumptions` field. Extend
   `artifact-registry.schema.json` for the new field.
2. `knowledge.schema.json` — add the `assumptions` property. Regenerate the type enum and id
   patterns from the registry rather than editing them.
3. `mkdir models/` — created populated, not empty. `glossary/` sat listed-but-empty and
   unvalidated for weeks precisely because nobody noticed an empty directory.
4. `check-registry.yaml` — declare the three ids with falsifiable specs and `check-knowledge.sh`
   as owner.
5. Implement the checks in `check-knowledge.sh`. **Plant the defect each should catch and watch
   it fail before claiming it works.**
6. Verify the three synchronised sites the gate itself checks: the `PreToolUse` guard's
   immutable directories, the open-loop ledger's `ART_DIRS`, and the schema/registry type
   agreement. All three are asserted by existing checks; all three have been wrong before.
7. Write `MOD-0001` — fully-loaded cost of one tenant — closing `T-0.7`'s blocking half.

**Backwards compatibility:** additive. No existing artifact acquires an obligation, because
`requires_assumptions` is `never` for all eleven existing types.

**How it is undone:** remove the registry entry, the schema property, the check ids declared
here and `models/`. Nothing else references them. This is materially easier to reverse than the four
prior type additions, because no existing artifact will cite a `MOD-` until one is written.

## Open questions

**1. Does `MKT-0001` violate `ADR-0048` rule 1, and does the answer change this design?**
`MKT-0001` holds Hetzner and RackBank prices *inside an artifact body* — values on a cadence
Paridyum does not set, which rule 1 appears to forbid. The defensible reading is that `MKT-` is
dated evidence meant to be superseded by better evidence, not a pin meant to stay current, and
that rule 1 governs pins. **These artifacts have never been read against each other.** If that
reading fails, the `basis: MKT-####` mechanism inherits the problem, and external inputs need a
registry instead. This blocks acceptance.

**2. Is `estimate` one value or a graded set?** `estimate` conflates "an informed figure from an
adjacent market" with "a placeholder". A graded `basis` — `estimate:informed` against
`estimate:placeholder` — carries more signal, at the cost of a judgement call on every
assumption and an argument about the boundary. Deliberately left at one value pending a real
`MOD-0001`; the shape should be chosen against a written model rather than in advance.

**3. Should a `MOD-` carry `review_date`?** `bet` and deferred `requirement` both do. A model
built on estimates decays and nothing here forces a re-look. The argument against is that
`review_date` on a non-`trigger` type may weaken what the field means on `bet`, where it is
paired with a falsification condition.

**4. Does `MOD-0001` belong to `product: [paridyum]` or `product: [hosting]`?** Fully-loaded
tenant cost is measured *by* Track B hosting and consumed *by* platform pricing. This is the
first real test of the `ADR-0050` axis and the answer is not obvious.

**5. What stops two models disagreeing?** Nothing proposed here. Two `MOD-` artifacts may derive
different costs from different assumptions and both validate. Whether that needs a check, a
convention, or nothing at all is unresolved.

## Related

`ADR-0048` (registry data), `ADR-0035` (counts in prose), `ADR-0022` (the type list),
`ADR-0050` (the product axis), `ADR-0018` (deferred requirements need triggers),
`ADR-0002` / `ADR-0014` (locked requires a check), `C-0007` (grammar and list),
`RFC-0005` (why strategy lives here), `RFC-0015` (a fact has one home),
`RFC-0020` (external facts are cited and dated),
`BET-0007`, `BET-0008`, `BET-0009` (the four trigger clauses with no baseline),
`MKT-0001` (open question 1), `ROADMAP.md` §6, `T-0.7`.
