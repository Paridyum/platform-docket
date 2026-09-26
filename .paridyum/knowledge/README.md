# Paridyum — Knowledge

The source of truth for every rule that binds Paridyum code, infrastructure and agents.

This repository has no prefix. Every other repository is `svc-*`, `lib-*`, `tool-*`,
`platform-*`, `infra-*`, `spec-*` or `template-*` (C-0002). The absence of a prefix is
the signal: this is not a member of a class, it is the thing that defines the classes.

**Agents: read [`CLAUDE.md`](CLAUDE.md) first.**

## Why this is not called `docs`

Documentation is written for humans and is advisory. Knowledge is consumed by humans
**and machines**, and parts of it are enforced. Every artifact here carries frontmatter
validated against [`knowledge.schema.json`](knowledge.schema.json), and artifacts marked
`status: locked` name executable checks that fail CI when violated.

The distinction is the whole point. Prose that nothing enforces is a suggestion, and a
suggestion is exactly what a long-running or freshly-started AI agent will drift from.

## Layout

```
constitution/   Foundational, org-wide, rarely changed. Highest precedence.
adr/            Architecture Decision Records — one decision, its context, its cost.
standards/      How to write things: Rust, APIs, testing, logging, database.
patterns/       Preferred implementations, to be copied rather than reinvented.
anti-patterns/  Forbidden shapes, with the reason they are forbidden.
runbooks/       Operational procedures, including workstation provisioning.
rfc/            Open questions and design proposals, before a decision exists.
requirements/   What the platform must eventually do, including deferred work (ADR-0018).
bets/           Falsifiable commercial wagers, each stating what would kill it (ADR-0022).
markets/        Dated market and customer evidence, with its sources (ADR-0050, MKT-).
glossary/       One term, one meaning, org-wide (ADR-0022, GLO-).
scripts/        bootstrap.sh — provisions a workstation per RUN-0001.
```

Alongside the directories sit the registries — the lists that must be extended, moved out of
the places where extending them was expensive:

```
artifact-registry.yaml   which artifact types exist, and which statuses each may take
check-registry.yaml      every check id, its spec, its owner, and what it blocks
class-registry.yaml      which repository classes exist, and which stacks bind them
framework-registry.yaml  which external frameworks a control may map outward to
product-registry.yaml    which products exist, their lifecycle, and what each owes
test-registry.yaml       which test types exist, on which axis, in which tier
tool-registry.yaml       which tools the estate uses, why, under what licence, and the way out
```

Each has a JSON Schema beside it and a section of `scripts/check-knowledge.sh` that verifies
every "generates" claim in both directions. ADR-0014, C-0005, ADR-0022, ADR-0028, ADR-0029 and
ADR-0066 are the decisions that put them there, and they are the same decision each time: a list
that must be extended, hardcoded in a place extending it is expensive. `tool-registry.yaml` is the
instance where the list was hardcoded in **no place at all**, which is the version that leaves
nothing to audit.

`toolchain-registry.yaml` sits beside them and is deliberately not in that list: it pins one
compiler or runtime per stack, has no schema and no check, and its own header records that as
TASKS T-2.7. It owns version numbers; `tool-registry.yaml` owns adoption facts and owns no version
where that file already holds one.

That list is **prose, and nothing checks it.** The authoritative one is
[`artifact-registry.yaml`](artifact-registry.yaml), which generates the type enum and the
four id patterns in `knowledge.schema.json`, the directories this gate scans, the id
prefixes its cross-reference scanner resolves, and which statuses each type may take. Per
ADR-0022 the type list was previously hardcoded at fifteen sites, fourteen of which failed
silently — a directory missing from the scanned set was not scanned at all, and the gate
still printed `Knowledge check passed`. Three of the fifteen were found already stale, one
of them a hole in the `PreToolUse` guard that let a locked requirement be edited in place.
Ten sites are now generated from the registry and three more are hardcoded-and-compared.
This block and the one in `CLAUDE.md` are the only two that are neither, because a generated
table dropped into the middle of an argument reads worse than a stale one. If they disagree
with the registry, the registry is right.

## Why strategy lives here

`bets/` is the newest directory and the one that most needs justifying, since a market
position is the one thing in this repository no check can verify.

It is here because of how this repository is consumed. `CLAUDE.md` instructs every agent to
*never infer architecture, retrieve it*, and ranks existing code as the lowest authority. An
agent that cannot retrieve who the customer is will infer one, and then encode that
inference in a schema, a price table or a tenancy boundary. **The absence of a written
answer is not neutral here; it is an instruction to guess.** RFC-0005 has the full argument,
including why a separate strategy repository and a strategy-tagged ADR were both rejected.

`glossary/` became an artifact type on the same day and for a related reason. It had existed as
a directory, was listed above, was empty, and was absent from every artifact type — so its
contents would have been unvalidated prose in a repository whose founding argument is that
unenforced prose is a suggestion. What made that expensive was concrete: `tenant` was used
thirty-six times across binding artifacts with no definition anywhere, while Paridyum's tenants
turned out to be *themselves multi-tenant* (GLO-0004), so the word meant two different isolation
boundaries depending on who read it. ADR-0024 could not be written until the vocabulary was.

A bet is not an ADR. An ADR records a decision already taken and states the cost accepted;
a bet records a wager and states the evidence that would end it. So a `bet` carries
`trigger` and `review_date` — the two fields REQ- already defines — and **may never be
`locked`**. `locked` means must-not-be-contradicted, and a bet exists precisely to be
contradicted by evidence on a date already written down. That rule is a check, not a
convention.

`markets/` arrived on 2026-08-17 and completes that pair. A bet is a wager and its
load-bearing field is `trigger` — what would kill it. An `MKT-` is an observation and its
load-bearing field is `sources` — where it came from and when it was read. The two get
confused, and the test is whether the sentence could be checked against the world today or
only judged later. What forced the type was C-0006 requiring that every vertical Paridyum
enters name *"the evidence that selected it"*, with no artifact type that sentence could
name: the requirement was live, constitutional, and impossible to comply with. ADR-0050 has
the argument, along with why a product needed a governance axis of its own before any of this
could be addressed at all.

## The lifecycle

```
REQ (what, eventually) → RFC (how) → accepted → ADR → (if foundational) Constitution
                                                 ↓
                                             Standard → Template → Generated code
                                                 ↓
                              enforced_by ── rejects the wrong diff        (ADR-0002)
                              verified_by ── demonstrates the right behaviour (ADR-0030)
                              frameworks  ── maps the whole thing outward   (ADR-0028)
```

The three bridges at the bottom answer different questions and none substitutes for another.
A check reads a repository, so it can see that a policy was written and never that the policy
is correct — which is a property of running software. `verified_by` names the test type that
closes that distance; `evidence` in `test-registry.yaml` says what the run leaves behind.

A requirement outlives the designs that satisfy it, which is why it is a separate
artifact rather than an RFC (ADR-0018). A requirement that is real but deliberately
unscheduled is `deferred` and must name the **trigger** that would make it current — a
deferred requirement without one is a `TODO` with frontmatter.

A decision only becomes `locked` once something can mechanically reject its violation
(ADR-0002). Rules that cannot be checked — judgement calls, boundaries, taste — stay
`accepted`. This keeps `locked` a promise rather than a wish.

## Current state

A hand-maintained table here went stale within a week. Its replacement — a count in
prose — went stale within a day. So there is no number here either. Ask the repository:

```sh
scripts/check-knowledge.sh                    # artifact count, and what is enforced
grep -rl '^status: locked' constitution adr standards anti-patterns requirements
grep -rl '^status: accepted' constitution adr standards rfc   # binding on judgement only
```

Anything stated here as a figure will be wrong before you read it. That is not a
criticism of whoever wrote it; it is a property of hand-maintained counts, and the third
time it happened the numbers were removed rather than corrected again.

`C-0001` (platform philosophy) is `accepted` rather than `locked` — it states the
principles that generate the enforceable rules, and nothing can check a principle.
Binding on judgement, not on CI. `ADR-0002` is `superseded` by `ADR-0014`; its body is
unchanged and still reads LOCKED, because only frontmatter may move in a supersession.

## The enforcement ladder

Four layers, strongest first (ADR-0007):

| # | Layer | Blocks? | Covers |
| - | ----- | ------- | ------ |
| 1 | `PreToolUse` hook in `paridyum-platform` | **yes, before the edit** | agent sessions |
| 2 | git pre-commit (`scripts/install-hooks.sh`) | yes, bypassable with `--no-verify` | any local edit |
| 3 | CI (`knowledge-check.yml`) | reports red; cannot block merge on GitHub Free | any commit |
| 4 | `forge audit` org sweep | after the fact | anything that reached `main` |

Layer 1 is the practical answer to the ADR-0004 gap: a hook that denies a tool call
cannot be reasoned around, does not decay after context compaction, and does not
depend on the GitHub plan.

**It does depend on being installed, and that is not automatic.** Declaring the marketplace in
`.claude/settings.json` prompts a contributor who trusts the folder; it does not install anything.
An audit on 2026-08-07 found the plugin declared and enabled while the marketplace had never been
added — so layer 1 had never fired, the `SessionStart` context had never been injected, and the
ADR-0025 checkpoint had never run automatically, all while the configuration read as complete.
`repo.paridyum_plugin_is_installed` now warns on that state. Verify with:

```sh
scripts/check-repo.sh | grep -i marketplace   # silent means installed
/plugin marketplace add Paridyum/knowledge    # if not
/plugin install paridyum-platform@paridyum
```

## Installing the harness

```
/plugin marketplace add Paridyum/knowledge
/plugin install paridyum-platform@paridyum
```

Repositories scaffolded by `forge` declare this in `.claude/settings.json`, so the
install is offered automatically when you trust the folder.

## How this reaches other repositories

Per ADR-0001, every repository carries a read-only snapshot at `.paridyum/knowledge/`,
written by `forge` at scaffold time and refreshed by `forge knowledge sync`. The
duplication is deliberate: an agent working in one checkout, offline, in parallel with
others, must be able to read every rule that binds it without a second fetch.

Vendored snapshots are **read-only**. Edits belong here.

## Setting up a workstation

```sh
git clone git@github.com:Paridyum/knowledge.git ~/Developer/paridyum/knowledge
~/Developer/paridyum/knowledge/scripts/bootstrap.sh
```

See [`runbooks/RUN-0001-workstation-setup.md`](runbooks/RUN-0001-workstation-setup.md)
for what it does and why. `bootstrap.sh --check` verifies without changing anything.

## Status

Pre-v0.1. `tool-forge` **exists** as of 2026-08-09 and ships three commands: `forge context`,
which resolves the artifacts binding a repository from its `manifest.yaml`;
`forge knowledge sync`, which refreshes a vendored snapshot as ADR-0001 requires;
and `forge check`, which runs the registry checks it implements. A first few of the checks
that named it as owner now run, and the locked artifacts enforced by nothing fell accordingly
— most, though, are still declared and unbuilt, so `enforced_by` remains largely a
specification. Two limits are worth stating plainly rather than leaving to be
inferred: the five run for `tool-forge` only, because the binary is not distributed and no
other repository's pipeline invokes it; and the repository existing was never the gap
closing. `scripts/check-knowledge.sh` prints the split on every run, so ask it rather than
trusting this paragraph.

Under ADR-0014 that gap is enumerated rather than described. Every check id is *declared*
in `check-registry.yaml` with a falsifiable spec, and the gate prints how many are
implemented on every run rather than failing on the rest. An id in neither the manifest
nor forge's registry is still a hard failure.

The implemented count was believed to be zero until an audit measured it. The same audit
found several checks reporting success while enforcing nothing: the secret scan had four
independent defects, the commit-identity gate had never examined a single commit in CI,
immutability lapsed the moment an artifact was superseded, and three checks verified less
than their names implied. All are fixed, and every fix was verified by planting the defect
it should catch.

**Treat a green check as unproven until you have watched it fail.** That is the most
useful thing learned here, and it is cheap to do.

See ADR-0014 for the resolution rules, ADR-0015 for which artifacts bind which
repositories, ADR-0002 for the superseded original, and ADR-0004 for the second way
enforcement is currently incomplete.
