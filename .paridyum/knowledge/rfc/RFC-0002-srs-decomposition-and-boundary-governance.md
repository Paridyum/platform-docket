---
id: RFC-0002
title: Decomposing a specification into bounded contexts before any code exists
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, spec]
applies_to: ["*"]
tags: [architecture, requirements, bounded-context, ownership, ai-agents, process]
breaking: false
resolved_by: ADR-0019
enforced_by: []
---

# RFC-0002 — Decomposing a specification into bounded contexts before any code exists

## Status

**ACCEPTED** as of 2026-08-07, resolved by ADR-0019. Closed well inside its 2026-09-07
feedback deadline.

Accepted rather than left open because **all five** recommended steps were already built
and in use — the count was twice misreported as four of five before an audit corrected it.
An RFC is a proposal under discussion; this one had nothing left to propose, and leaving it
open would only have made the record lag the reality.

Two of its three remaining open questions were settled in practice: the `domain.*` checks
landed as `declared`, and ADR-0018 fixed one `REQ-` per bounded context rather than one per
line item. The third — governing bulk repository creation — is **not** answered, and is
carried into ADR-0019 as a known limitation rather than quietly dropped.

Accepting also closed the gap recorded below: the service-creation guard now has a check
id, `agent.service_creation_escalates`, cited by ADR-0019.

`enforced_by` is empty because an RFC decides nothing — checks arrive with the ADRs this
RFC would generate, if it is accepted.

### Implemented ahead of acceptance — and what that costs

All four recommended steps were built as a trial before this RFC was accepted, at the
author's direction. That is a deliberate inversion of ADR-0013 and it has one honest cost,
recorded here rather than hidden:

| Step | State | Check id |
| --- | --- | --- |
| `ANTI-0001`…`ANTI-0006` | shipped | 3 declared, 3 correctly none |
| `manifest.schema.json` with `owns:` | shipped | `docs.manifest_schema_valid` — partly implemented |
| `db/grants.sql` in the service template | shipped | folded into `domain.entity_has_single_owner` |
| `decompose-spec` skill | shipped | none — a skill is invoked, not enforced |
| Service-creation guard (`ask`) | shipped | **none — see below** |

**The creation guard had no check id, and that was the gap.** A check id must be cited by
some artifact's `enforced_by`, and the artifact that would cite this one was the ADR this
RFC becomes on acceptance. Minting an id earlier would have left an orphan declaration;
citing it from an RFC would have claimed a proposal enforces something. So the guard was
live enforcement the registry could not see — the precise condition that hid four broken
checks for weeks before an audit found them.

**Closed on acceptance:** ADR-0019 cites `agent.service_creation_escalates`.

## Problem

Given a large specification, an agent's default behaviour is to begin implementing it.
The failure is not bad code; it is a **plausible implementation of the wrong
decomposition**, produced faster than anyone can review it, after which the code has
momentum and rejecting it costs more than accepting it.

ADR-0013 already establishes design-before-code and computes significance from the diff.
But its lifecycle **starts at stage 1 "Define — issue or RFC §Problem"**, which assumes
the problem arrives already atomic. A specification containing a hundred requirements
across nine actors is not atomic, and there is currently no step that makes it so.

Worse, ADR-0013's enforcement point is the pull request — `pr.significant_change_
references_design`. By then the code exists. For a wrong boundary that is too late: the
service is created, the table is owned, and the coupling is already paid for.

## What already exists — do not rebuild this

Establishing this was the first task, and it removed most of the proposed pipeline.

| Concern | Already covered by |
| --- | --- |
| Design before implementation, significance from the diff | ADR-0013 + `pr.significant_change_references_design`, `pr.design_artifact_predates_implementation`, `rfc.has_alternatives_section`, `pr.body_completed` |
| A requirement as a durable artifact, deferrable with a trigger | ADR-0018 (locked), `REQ-` — twelve in use |
| Walking the lifecycle interactively | the `design-change` skill |
| Recording a decision with alternatives and costs | the `new-decision` skill, ADR format |
| One repository = one bounded context | C-0002 |
| Event naming (`<Subject><PastTenseVerb>`), table naming, API paths | C-0002, with `openapi.*` and `db.*` checks declared |
| **Service dependency graph, as data** | `manifest.yaml` (STD-0002): `depends.services`, `depends.infra`, `depends.events` |
| **Published event contracts, as data** | `manifest.yaml`: `exposes.events`, `exposes.http.openapi` |
| Ownership by team, never by person | `manifest.yaml`: `owner` |
| One responsibility per module, no grab-bags | STD-0004 + five `module.*` checks |
| Forbidden-shape artifacts | `ANTI-` prefix and `anti-pattern` type **already in the schema** |

Two things follow. First, the dependency graph and event contracts are **already declared
as data**, so cycle detection and event-subscription validation need no new mechanism —
only a check that reads manifests. Second, `anti-patterns/` and `patterns/` exist and are
first-class in the schema, and at the time of writing contained **zero artifacts** — the
forbidden list had a home and no occupants. That is what step 1 of the recommendation
addressed; `anti-patterns/` now holds ANTI-0001 through ANTI-0006. `patterns/` is still
empty.

## The tension this RFC must resolve

ADR-0002 considered this and set it aside:

> "Service boundaries should follow business capabilities" cannot be mechanically
> verified. Such rules stay `status: accepted`, not `locked`.

That is correct and this RFC does not contradict it. But it conflates two things, in the
same way ADR-0002 conflated a typo'd check id with an unbuilt one:

- **"Is this the right boundary?"** — judgement. Unverifiable. Stays `accepted` forever.
- **"Is the system consistent with the boundary you declared?"** — mechanical, *provided
  the boundary is declared as data.*

Nobody can check that `Invoice` belongs in `svc-billing`. Everybody can check that two
manifests both claim to own it. **Declaration converts a judgement into a checkable
invariant** — the same move ADR-0014 made for check ids, and ADR-0015 for scope.

This is the whole proposal. The human still draws the boundary; the gate holds the system
to it.

## The gaps

1. **No intake step.** The lifecycle begins after decomposition has somehow happened.
2. **No entity-ownership declaration.** `manifest.yaml` has `exposes` and `depends` but no
   `owns`. So "two services own the same table" — the most expensive mistake on the list —
   is undetectable. This is the single highest-value gap.
3. **`anti-patterns/` was empty** when this RFC was written. Nothing recorded the forbidden
   shapes, so nothing could cite them and no agent read them. *(Closed ahead of acceptance
   by ANTI-0001 through ANTI-0006 — see the implementation table in Status. `patterns/` is
   still empty, deliberately: a pattern is worth recording once there is an implementation
   to copy, and there is none.)*
4. **Nothing stops SRS → code at the moment it happens.** ADR-0013 deliberately rejected a
   PreToolUse hook for this: *"an agent cannot know at first file-write whether the
   eventual diff will be significant, so the hook would fire constantly on trivial work
   and be disabled."* That reasoning holds for ordinary edits. It may not hold for the
   narrow act of **creating a new service**, which is knowable at the first write and is
   the irreversible one.
5. **No artifact declares a domain.** C-0002 implies one repo is one context but nothing
   states what a context owns, publishes, or consumes — so `forge context` has nothing to
   reason about above the repository.

## Options

### A. A decomposition skill only

`/decompose` walks extraction → normalisation → entity discovery → context discovery →
ownership matrix, writing `REQ-` artifacts and a proposed context map. Advisory.

Cheap, immediately useful, no new artifact types, no enforcement. An agent that skips the
skill is unaffected — which is the whole objection to advisory process (ADR-0002).

### B. Skill + `ANTI-` artifacts + ownership in `manifest.yaml`

A adds:

- One `ANTI-` artifact per forbidden shape, each carrying a check where one is possible.
  (Deliberately not numbered here — writing a concrete id for an artifact that does not
  exist is a dangling reference, and `repo.no_dangling_decision_references` caught exactly
  that in the first draft of this RFC.)
- An `owns:` block in `manifest.yaml` — entities and tables this service is sole writer of.
- Checks: `domain.entity_has_single_owner`, `domain.no_service_dependency_cycles`,
  `domain.no_cross_service_table_access`, `domain.subscribed_events_are_published`.

Enforceable, and it reuses `manifest.yaml` rather than inventing a registry.

This option was drafted believing `owns:` would need STD-0002 superseded. **Review found
that is not so** — STD-0002 makes no exhaustiveness claim and `manifest.schema.json` does
not exist, so the cost is one new file rather than a supersession. See *Resolved by review*.

Remaining cost: every check here is cross-repository, so it runs in `forge audit`, which
does not exist. And `domain.no_cross_service_table_access` is weak as a static check —
review found the real enforcement is database grants, not metadata.

### C. B, plus a narrow creation guard

A PreToolUse guard that denies **creating a new service or crate** without a referenced
accepted RFC or ADR — not ordinary file writes.

This sidesteps ADR-0013's objection: creating `svc-*/Cargo.toml` or a new `manifest.yaml`
*is* knowable at first write, and it is precisely the irreversible act. Ordinary editing
stays untouched, so the guard fires rarely and does not train `--no-verify`.

Risk: it blocks the exploratory spike, which is a legitimate mode ADR-0013 explicitly
protects when it made `pr.design_artifact_predates_implementation` a warning. Mitigated by
making it `ask` rather than `deny`, consistent with ADR-0008's finding that *"a guard that
denies every dangerous thing is disabled within a week."*

### D. Full fifteen-stage pipeline as artifacts

A directory and artifact type per stage — observations, use cases, context maps, ownership
matrices, event catalogues, service contracts, tickets.

Rejected on the same grounds ADR-0008 rejected a nineteen-file `.governance/` tree and
ADR-0018 rejected `OBS-`/`IDEA-`: *"empty scaffolding is worse than none because it
advertises coverage that does not exist."* Paridyum has zero services. Nine of these
directories would be empty for a year.

## Recommendation

**Option C, reordered after review, with the enforcement of single-writer moved out of the
gate.**

1. **`ANTI-` artifacts first.** Cheapest, highest value per hour, and needs no machinery
   whatsoever — the type and prefix already exist and the directory is empty. An agent
   reading the constitution would then read that two services owning a table is forbidden
   *and why*, which is most of the benefit before a single check is written. Nothing blocks
   this.
2. **`manifest.schema.json`, including `owns:` (tables).** Moved up, because review showed
   it needs no supersession and simultaneously implements `docs.manifest_schema_valid`,
   which has been declared and unimplementable from the start. One file closes an existing
   gap and opens the ownership work.
3. **Single-writer enforced by database grants in the service template**, not by a static
   check. Per-service database user, write grants only on owned tables. The gate's job is
   to verify the declaration and the grants agree — not to police access, which it cannot
   do reliably.
4. **The decomposition skill.** Useful immediately and independent of everything above.
5. **The creation guard, as `ask`.** Last, once the artifacts it would cite exist.

Deliberately **not** proposed: a domain artifact type, an `amends`/`amended_by` relationship,
or a new status. `manifest.yaml` plus C-0002's one-repo-one-context already carries the
domain, and the amendment mechanism turned out to be non-standard and unnecessary.

## Rejected alternatives

- *Generate services from the SRS and review the result.* The failure this exists to
  prevent. Review of a generated decomposition is theatre — the artefact is too large and
  too plausible.
- *Enforce "boundaries follow business capabilities" directly.* ADR-0002 is right that it
  is unverifiable. The proposal deliberately checks consistency with a declared boundary
  instead of the boundary's correctness.
- *A separate ownership registry file.* Rejected: `manifest.yaml` is already the machine-
  readable half of a repository and already feeds the dependency graph. A second registry
  would drift from it, and there would be no rule about which wins.
- *Block all code writes until an accepted design exists.* Rejected on ADR-0013's own
  finding — it fires constantly on trivial work, so it gets disabled, so it protects
  nothing.
- *Do nothing; ADR-0013 already covers it.* Rejected: ADR-0013 enforces at the pull
  request, which is after the boundary is already paid for, and its lifecycle assumes an
  atomic problem statement that a specification is not.

## Resolved by review — 2026-08-07

**Q3, extending a locked standard: the question dissolves.** STD-0002 shows an example
`manifest.yaml` and **makes no exhaustiveness claim** about its fields. `manifest.schema.json`
**does not exist** — which is exactly why `docs.manifest_schema_valid` has been declared and
unimplementable since it was written. So adding `owns:` is not a change to a locked artifact
at all. Writing `manifest.schema.json` *including* `owns:` implements an already-declared
check and extends the manifest in one move, with **zero supersessions**. This is the unlock,
and it inverts the build order below.

A general amendment mechanism was researched as the alternative and **is not standard
practice.** Microsoft's Well-Architected guidance is explicit: *"The ADR serves as an
append-only log. Don't go back and edit accepted records. If a decision changes, write a new
record that supersedes the original and link the two together."* Its status set is
*Proposed, Accepted, Superseded* — no "amended". A secondary source claiming an "amended"
status that preserves `accepted` could not be verified; the authoritative source contradicts
it. What that guidance *does* offer is the real pattern: *"Break one decision into multiple
if an architectural decision is going to result in multiple phases… Log each phase as its
own decision record."* An extension is simply a new record citing the old one. No new
relationship type, no schema change.

**Q1, the unit of `owns:` — and the enforcement is not a check.** Industry does not enforce
single-writer through declarative metadata. It enforces it with **database grants**: a
distinct database user per service, granted write access only on the tables it owns. The
underlying principle is Martin Thompson's Single Writer Principle — data is owned by one
execution context. A declaration is the design-time artifact you review and generate grants
from; the grant is what actually stops the write. Recommendation: declare **tables**, because
that is what a grant is expressed in, and record the aggregate in prose where it belongs.

This relocates most of the value: `domain.no_cross_service_table_access` as a static check is
weak, and per-service credentials with grants in the service template (ADR-0003, ADR-0005)
are strong. The knowledge gate should check that the declaration and the grants agree, not
try to police access itself.

**Q5 confirmed aligned.** Backstage — the reference implementation for service catalogs —
requires `spec.owner` to name a Group, never a User, because an individual owner orphans the
entity when they leave. STD-0002 already says *"`owner` is a team even when the team is one
person."* No change needed.

## Open questions

1. Cross-repository checks need `forge audit`, which does not exist. Do the `domain.*`
   checks land as `declared` now, or wait? (Formerly Q2.)
2. Should the decomposition skill write `REQ-` artifacts directly, given a hundred-
   requirement specification would produce a hundred files? Or a single `REQ-` per bounded
   context, with the atomic list inside it? (Formerly Q4.)
3. A decomposition producing nine services means nine repositories under ADR-0001, and
   nothing governs creating them in bulk. Does the creation guard scale to that, or does
   bulk scaffolding need its own decision? (Formerly Q5, narrowed.)

## Related

- ADR-0013 — the lifecycle this extends at the front
- ADR-0002 — parked boundary rules as unenforceable; this RFC narrows rather than reverses
- ADR-0018 — `REQ-` artifacts, and the precedent for answering a namespace rejection
- ADR-0015 — declaration converting judgement into a checkable invariant
- ADR-0008 — the empty-scaffolding argument that rejects option D
- STD-0002 — defines `manifest.yaml`; locked, which option B must work around
- STD-0004 — one responsibility per module, the intra-repository analogue
- C-0002 — one repository is one bounded context
