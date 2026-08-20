---
id: ADR-0014
title: A declared check id is a tracked commitment, not a build failure
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: [ADR-0002]
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [governance, ci, ai-agents, enforcement, bootstrap]
breaking: true
enforced_by:
  - check: knowledge.frontmatter_valid
    severity: error
    rationale: An artifact the gate cannot parse is an artifact the gate cannot enforce.
  - check: knowledge.locked_artifacts_immutable
    severity: error
    rationale: Editing a locked artifact in place erases the audit trail; supersede instead.
  - check: knowledge.check_ids_resolve
    severity: error
    args: { registry: "check-registry.yaml" }
    rationale: Resolution is now against the manifest, not the implementation set, so a typo still hard-fails while an unbuilt check does not.
  - check: knowledge.declared_checks_manifest_valid
    severity: error
    rationale: A manifest entry is the only thing separating a commitment from a typo; if it can be malformed or empty the distinction collapses.
  - check: knowledge.no_check_id_demotion
    severity: error
    rationale: Moving an id from implemented back to declared is how a check that started failing gets quietly switched off.
  - check: knowledge.orphan_check_declarations
    severity: warn
    rationale: A declaration no artifact cites is a commitment to enforce nothing, and turns the manifest into a junk drawer.
  - check: repo.ci_runs_knowledge_check
    severity: error
    rationale: The gate must run in every repo, or it binds only the repos that opted in.
  - check: repo.precommit_hook_installed
    severity: warn
---

# ADR-0014 — A declared check id is a tracked commitment, not a build failure

## Status

**LOCKED** as of 2026-08-06. Supersedes ADR-0002.

Accepted by a human under ADR-0013 stage 3. ADR-0002 carries the legal supersession
edit — `status: superseded`, `superseded_by: ADR-0014` — and nothing else; its body is
unchanged and still reads `LOCKED`, because ADR-0002's own immutability rule permits
only those two frontmatter fields to move. The frontmatter is the machine-readable
truth; the body is the record of what was argued at the time.

## Context

ADR-0002 established the rule this platform rests on: a decision is not locked until
something executable rejects its violation. That holds, and this ADR does not weaken
it. What follows is a defect in one clause of it, discovered by trying to build the
gate ADR-0002 specifies.

### The arithmetic

The knowledge base currently declares **100 distinct check ids** across **26
namespaces**, cited by **23 artifacts**. `tool-forge` does not exist, so **0 are
implemented**.

ADR-0002 point 4 says:

> An unknown check id is a **hard failure**, never a skip. A rule that silently
> enforces nothing is more dangerous than an absent rule, because it manufactures
> false confidence.

Read against the current state, that clause says: the first time `forge knowledge
check` runs, it fails on all 23 artifacts. It fails on ADR-0002 itself, because
`knowledge.check_ids_resolve` is one of the 100 unimplemented ids. There is no
first green run, and no incremental path to one — every check must exist before any
check can pass. ADR-0002 forbids the only order in which its own gate can be built.

### Why the clause is wrong, and it is not the guarantee

The clause is not wrong about the danger. It is wrong because it asks a single lookup
table to answer a question that table cannot answer.

An id absent from `tool-forge`'s registry is one of two entirely different things:

- a **typo** — `repo.ci_runs_knowlege_check`. Nothing will ever run. The artifact reads
  as enforced and is not. This is precisely the false confidence ADR-0002 names, and it
  must hard-fail.
- a **commitment not yet built** — `pr.significant_change_references_design`, named by
  ADR-0013 in full knowledge that it does not exist yet. This is a tracked debt.

Both present identically: *not in the registry*. One lookup cannot distinguish them,
so ADR-0002 conflated them and applied the correct treatment for the first to both.

ADR-0002 already draws exactly this distinction one level up. Point 5 requires
`--audit` to list every locked artifact with an empty `enforced_by`, expects that list
to be non-empty, and calls it "an honest inventory of which decisions are currently
only advisory." An artifact that names no check is honest debt. An artifact that names
a check nobody has written is the same debt, one level finer — and ADR-0002 treats it
as a build break. The honesty stops one level short of where it was needed.

The repository already behaves as though this ADR were in force. `README.md` states
that no lock is mechanically enforced today and that "the checks named in
`enforced_by` are a specification for a gate that has not been built." `CLAUDE.md`
says the locks are enforced "by your compliance, not by machinery." Both are accurate
descriptions of a declared-but-unimplemented check set. Neither is expressible in
ADR-0002's vocabulary.

## Decision

**Check id resolution has three states, decided by set membership across two sources of
truth rather than one.**

### The two sources

- **`check-registry.yaml`**, in this repository, alongside `knowledge.schema.json`. The
  complete id-space: every check id that legitimately exists, whether or not anything
  runs it. Owned by the knowledge base, because the artifacts that cite the ids live
  here and travel with it under ADR-0001's vendoring model.
- **`tool-forge`'s check registry**. The subset it can execute. Always a subset —
  `implemented ⊆ declared` is an invariant, not a coincidence.

A second source is what makes the distinction decidable at all. A typo is an id nobody
wrote down twice.

### The three states

| Cited by an artifact | In `check-registry.yaml` | In forge's registry | Verdict |
| --- | --- | --- | --- |
| yes | yes | yes | **implemented** — run it at the artifact's severity |
| yes | yes | no | **declared** — report in `--audit`, do not fail the build |
| yes | no | — | **unresolved** — hard failure, this is the typo case |
| no | yes | no | **orphan declaration** — warn |
| no | no | yes | **unbound implementation** — warn |

`forge knowledge check` fails on ids in neither the manifest nor the implementation
set. Nothing else about ADR-0002 changes.

### What a declaration costs

A manifest entry is deliberately not producible by copy-pasting the id out of the
artifact — if it were, a typo could be legitimised by the same mistake twice. Every
entry requires:

```yaml
- check: pr.significant_change_references_design
  spec: >
    Fails when a PR whose diff matches ADR-0013's significance list contains no
    reference to an RFC or ADR id in its body or commit messages.
  owner: tool-forge
  blocks: [ADR-0013]
  declared: 2026-08-06
```

`spec` is a falsifiable sentence — what the check rejects, not what it is about. It is
the acceptance criterion for the implementation and the thing a reviewer checks the
implementation against. `blocks` lists the artifact ids whose enforcement is waiting on
it, which is what makes the backlog orderable by consequence instead of by id.

Severity is **not** stored here. It stays in each citing artifact's `enforced_by`, and
it is preserved across the declared → implemented transition. The day a check is
written it begins enforcing at the severity that was declared for it, with no second
decision — so a check cannot be slipped in as `warn` and left there.

### The ratchet

Movement between states is one-way per id. `declared → implemented` is the intended
direction and needs no ceremony. `implemented → declared` is a hard failure
(`knowledge.no_check_id_demotion`): removing a working check from forge's registry
while the artifact still cites it is how a check that started failing gets switched off
without a superseding decision. Retiring a check means superseding the artifact that
demands it.

### Bootstrap order

This is the incremental path ADR-0002 foreclosed:

1. Land `check-registry.yaml` with all 100 ids, `spec` written for each. Pure data —
   no `tool-forge`, no Rust. This is the work that converts 100 sentences of intent
   into 100 acceptance criteria.
2. `knowledge.check_ids_resolve` becomes implementable immediately, because it is set
   membership against a file in the same repository. `scripts/check-knowledge.sh` can
   carry it before `tool-forge` compiles.
3. Every subsequent check moves `declared → implemented` individually. The gate is
   green at every step and the declared count only falls.

### What `--audit` must report

Three numbers, at every run, not only under `--audit`: locked artifacts naming no check
(ADR-0002 point 5, unchanged), declared-not-implemented ids grouped by namespace, and
orphan declarations. The backlog is worth nothing if seeing it requires choosing to
look.

## Consequences

**Accepted costs**

- **This ADR adds three ids to the backlog it exists to manage** —
  `knowledge.declared_checks_manifest_valid`, `knowledge.no_check_id_demotion`,
  `knowledge.orphan_check_declarations`. 100 becomes 103. Stated plainly because an ADR
  about honest accounting cannot quietly round its own cost down.
- **`declared` can be abused as a parking space.** Nothing here forces an id to ever be
  implemented. The friction is a required falsifiable `spec` and permanent visibility in
  every run's output; the deterrent is that the count is public and only ever compared
  against the previous run. That is weaker than a deadline and it is chosen deliberately
  — see rejected alternatives.
- **A typo written twice is legitimised.** Someone who mistypes the id in the artifact
  and then writes a manifest entry for the mistyped id gets a silently-unenforced rule,
  the exact failure ADR-0002 guards against. The `spec` and `blocks` fields make this
  require deliberate work rather than a paste, which is mitigation, not elimination.
- **Two sources of truth can drift.** `orphan declarations` and `unbound
  implementations` exist to catch drift in both directions, at `warn`. They are the
  seam this design introduces.
- **`locked` now means "committed to enforcement" during bootstrap, not "currently
  enforced."** ADR-0002's reading was stricter. The compensation is that the gap is
  enumerated per-check rather than inferred, which is a stronger position than the
  prose disclaimers in `README.md` and `CLAUDE.md` that carry this today.

**Gained**

- The gate can have a first green run, and therefore can be built at all.
- No silent no-ops. The guarantee ADR-0002 was protecting is intact: an id that
  resolves to nothing still fails the build. Only the *unbuilt* case is reclassified.
- A 100-item backlog becomes a visible, ordered, per-check work queue with acceptance
  criteria attached, instead of a single sentence in `README.md`.
- **Zero edits to the 23 declaring artifacts.** The declared state lives in the manifest,
  not in `enforced_by`, so no artifact frontmatter changes, `knowledge.schema.json` needs
  no change, and no locked artifact is superseded except ADR-0002 itself.
- Writing an ADR that names an unbuilt check stops being dishonest, which means the next
  ADR can name the check it actually wants rather than the check that happens to exist.

**Rejected alternatives**

- *Implement all 100 checks before shipping the gate.* Months of work with enforcement
  off for the duration — the condition ADR-0002 exists to end, extended by the length of
  the project. It also rejects itself: `knowledge.check_ids_resolve` cannot be validated
  against a registry that is still being filled.
- *Demote the checks in all 23 artifacts to match a small initial registry.* 23
  supersessions of locked artifacts, discarding the enforcement intent that is the most
  valuable content in those files, to be re-added later by 23 more supersessions. It
  converts a tracked debt into a lost record.
- *Add a per-entry `status: declared` field to `enforced_by`.* Considered first, and it
  is worse in the same way: the state is per-id, not per-citation, so an id cited by
  four artifacts would carry the flag four times and drift. It also requires editing
  every citing artifact each time a check ships — supersessions triggered by
  implementation progress, which inverts the relationship between decision and code.
- *Expire declarations after N days.* Rejected. A declaration that expires turns the
  whole gate red on a calendar date, which reproduces exactly the all-or-nothing failure
  this ADR removes, only later and less predictably. Deadline pressure on a backlog is a
  planning problem, and encoding it in the gate makes the gate the thing people learn to
  route around.
- *Ratchet the declared count downward — no run may declare more than the last.* Rejected:
  a legitimate new ADR naming a legitimate new check would fail the gate, which
  penalises writing decisions down. The wrong incentive to build into the layer whose
  purpose is getting decisions recorded.
- *Amend ADR-0002 in place.* Not available and should not be. Point 4 was load-bearing
  for six months of reasoning about what `locked` means; a reader of code written under
  it needs to be able to retrieve the rule as it stood. Supersession preserves that,
  which is the whole argument of ADR-0002 point 1 applied to ADR-0002.

## Carried forward from ADR-0002

Supersession in this knowledge base is whole-artifact — `superseded_by` is a single id,
and there is no mechanism for amending one clause. So this ADR restates ADR-0002's
decision in full. Points 1, 2, 3 and 5 are carried forward **unchanged in substance**;
only point 4 is replaced.

1. Every artifact with `status: locked` carries an `enforced_by` list naming checks,
   each of which must appear in `check-registry.yaml` (schema:
   `knowledge.schema.json`).
2. `forge knowledge check` resolves and runs those checks against a repository.
3. It runs in three deliberately redundant places: the pre-commit hook, CI on every
   pull request and push to `main`, and `forge audit` sweeping the org on a schedule.
4. **Replaced.** An id in neither `check-registry.yaml` nor forge's registry is a hard
   failure. An id in the manifest but not yet implemented is `declared`: reported, not
   fatal. See *The three states*.
5. `forge knowledge check --audit` lists every locked artifact with an empty
   `enforced_by`. That list is the enforcement backlog, is expected to be non-empty, and
   is an honest inventory of which decisions are currently only advisory.

ADR-0002's other sections survive intact and are restated by reference rather than
re-argued: prose carries the *why* and the check carries the *what*; not every rule is
checkable, and unchecked rules stay `accepted` rather than `locked`; and the GitHub Free
limitation on required status checks under ADR-0004 remains a real gap in this ADR's
guarantee, tracked in ADR-0004 and ADR-0012.

## Known limitations on the current plan

Two gaps, both real, both narrower than the one this ADR closes.

**Supersession is not checked for symmetry.** Nothing in the 103 ids validates that an
artifact naming `supersedes: [X]` is matched by `X` naming `superseded_by`. The
acceptance procedure at the top of this document is therefore a two-step edit whose
second step nothing enforces — if it is skipped, ADR-0002 and ADR-0014 both read as
in-force, on the one question where they disagree, and the gate stays green. This ADR
does not add a check for it because that is a separate decision about the supersession
model rather than about check resolution, and bundling it would hide a second change
inside this one. It is the most immediately consequential item this ADR leaves open.

**The GitHub Free limitation still applies.** Under ADR-0004 private repositories cannot
require a status check to pass before merge, so CI reports red while the merge button
stays enabled. The enforceable layers remain the pre-commit hook and the `forge audit`
sweep, both bypassable by a determined actor. Unchanged from ADR-0002 and tracked in
ADR-0004 and ADR-0012 — noted here because a reader arriving at this ADR for the
resolution rules should not conclude the gate is mandatory.

## Errata

Appended under [ADR-0031](ADR-0031-errata-are-additive-and-dated.md). Additive and dated; the
text above is unchanged and is what this ADR decided.

### 2026-08-08 — "0 are implemented" is true of tool-forge and false of enforcement

**Says**, in *The arithmetic*: *"`tool-forge` does not exist, so **0 are implemented**."*

**Wrong:** as a statement about enforcement. **35 checks were already implemented** when this was
written — in the bash gates, in the `PreToolUse` hooks and in CI — and the audit that established
that figure is what exposed the claim.

**Intended:** *"`tool-forge` does not exist, so 0 are implemented **in its registry**."*

**The decision is unaffected**, and the distinction is worth keeping: no id resolved against any
registry, so the gate could not have had a first green run, which is the argument this ADR makes.
What the sentence does is understate the enforcement that existed, and it is quotable in a way
the surrounding paragraph is not.

REQ-0002 known instance 2. Found 2026-08-07; recorded here 2026-08-08, having lived until then
only in `check-registry.yaml`'s `owner` fields and header.
