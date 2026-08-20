---
id: ADR-0040
title: The system view is generated from manifests rather than drawn alongside them
type: adr
status: accepted
created: 2026-08-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [architecture, diagrams, c4, manifests, generated, documentation]
breaking: false
verified_by: [TST-0061]
enforced_by:
  - check: docs.diagram_parses
    severity: error
    rationale: >
      STD-0002 requires one Mermaid diagram in ARCHITECTURE.md and nothing has ever parsed it.
      A diagram with a syntax error renders on GitHub as a grey box, which is indistinguishable
      from a rendering failure and is the one documentation defect that looks like someone
      else's bug.
  - check: docs.system_view_is_current
    severity: error
    rationale: >
      A generated view committed to a repository is generated output under C-0004, and the
      failure mode of generated output is staleness rather than incorrectness. Regenerating and
      comparing is the only claim worth making about it.
  - check: docs.no_hand_authored_architecture_model
    severity: error
    rationale: >
      The whole decision is that the model has one home. A `.dsl`, `.puml` or `.d2` file
      declaring systems and relationships is a second copy of manifest data, and a second copy
      is what this repository has been caught by five times.
---

# ADR-0040 — The system view is generated from manifests

## Status

**ACCEPTED** as of 2026-08-09. Not locked — per the `new-decision` skill §5b, an artifact is
created `accepted`, its facts are checked, and it is locked in a separate commit. ADR-0029's two
errata exist because that ordering was not followed.

## Context

STD-0002 requires every repository to carry `ARCHITECTURE.md` with **one Mermaid diagram**, and
`docs.required_files_present` verifies the file exists. That is a per-repository, container-level
view. Nothing renders the estate as a whole, so there is no system-context view of Paridyum, and
the question *"what talks to what"* has no answer that is not read out of a dozen repositories by
hand.

The obvious fix is to adopt an architecture-as-code modelling language — Structurizr DSL, C4
notation, a `workspace.dsl` under `architecture/` — and hand-author the model. It was proposed in
exactly that form.

**It is the wrong fix here, and the reason is a pattern this repository has already paid for five
times.** `manifest.yaml` already declares, per repository: `name`, `class`, `owner`,
`owns.tables`, `owns.events`, `exposes.http` with its port and OpenAPI path, `exposes.events`,
`depends.services`, `depends.infra`, `depends.events`, `stack`, `lifecycle` and `slo`. That is a
C4 Level 2 container model with typed edges, and STD-0002 already says `forge docs generate`
assembles the dependency graph from it.

A hand-authored `workspace.dsl` would restate those facts in a second place. The second copy
would drift, and it would drift silently, because a diagram cannot fail a build. This is the same
defect as a `spec:` field restating an enumeration the implementation derives — found in
`knowledge.no_duplicate_id_namespaces` on 2026-08-08, where the prose listed seven artifact
prefixes and the code had always read ten from the registry.

Two things genuinely blocked generating the view, and only one of them was hard:

1. **`depends` could not name anything outside the estate.** Its three keys were `services`,
   `infra` and `events`. A payment processor, a mail provider or an identity provider had
   nowhere to be declared. C4 Level 1 is *precisely* the diagram that shows external systems and
   human actors, so it was underivable — while Level 2 was fully derivable and simply never
   generated.
2. **Nothing validated the diagram that does exist.** `docs.required_files_present` proves
   `ARCHITECTURE.md` is on disk. It says nothing about whether the Mermaid block inside it parses.

## Decision

**The architecture model is `manifest.yaml`. Every cross-repository architecture view is
generated from the set of manifests, is treated as generated output under C-0004, and is never
hand-authored.**

1. **One model, many views.** System context, container, dependency graph and ownership views are
   renderings of manifest data. Adding a view is a renderer, never a new source of facts.

2. **A hand-authored architecture model is forbidden.** No `workspace.dsl`, `.puml`, `.d2` or
   equivalent declaring systems, containers or relationships. `docs.no_hand_authored_architecture_model`
   fails on one. This is not hostility to the notation — Mermaid C4 syntax is a fine *output* —
   it is the single-source rule applied to architecture facts.

3. **`depends.external` and `actors` are added to `manifest.schema.json`**, closing the Level 1
   hole. `external` carries `name`, `purpose` and optional `data_shared`; `actors` carries `name`
   and `interacts_via`. Neither participates in cycle detection: an external system cannot call
   back through a declared edge, so it cannot close a cycle.

4. **`data_shared` is deliberately more than a diagram needs.** REQ-0003 (data and privacy) and
   REQ-0012 (vendor governance) both need to know what leaves the estate, and neither had a place
   to ask. A field that serves a diagram and two deferred requirements is worth more than one that
   serves a diagram.

5. **The hand-written diagram in `ARCHITECTURE.md` stays, and stays hand-written.** It answers
   *why* — the boundary this service owns, what was deliberately not built. No generator infers
   intent, and STD-0002's requirement is unchanged. Only the *cross-repository* view is generated.

6. **Every committed diagram must parse.** `docs.diagram_parses` covers the hand-written Mermaid
   in `ARCHITECTURE.md` and any generated view. It is the cheapest of the three checks and closes
   a gap that has existed since STD-0002 was locked.

## Consequences

**Accepted costs**

1. **Three more checks on a backlog that is already the larger half.** Most declared checks
   await `tool-forge`, which does not exist; these three join them, and
   `scripts/check-knowledge.sh` prints the split on every run rather than this artifact
   asserting a figure that goes stale (ADR-0035). Until then this ADR is
   upheld by compliance rather than machinery, which CLAUDE.md's *honest current state* already
   names as the normal condition. Declaring them anyway is ADR-0014's bargain: the id exists so
   the rule can be cited, and the gap is visible in the count rather than hidden.

2. **A generated view is only as true as the manifests.** `domain.no_service_dependency_cycles`
   already records the limit — *"a dependency reached by hard-coded URL and not declared here is
   invisible to cycle detection, which is how a dependency graph becomes false."* Generating the
   picture from declarations makes that failure mode **more** consequential, because a false
   graph now looks authoritative. The mitigation is that manifests are already checked against
   grants by `domain.entity_has_single_owner`; there is no equivalent for `depends`, and there
   should eventually be one.

3. **`actors` and `external` will be under-filled at first.** A required field nobody needs yet
   is a field people copy from a template without thinking. Neither is required by the schema,
   which is honest but means an empty system-context view is indistinguishable from an accurate
   one — the same shape as ADR-0030 cost 3.

4. **Forbidding a modelling language forecloses what those tools are genuinely good at.**
   Structurizr's view-definition language is better than anything derivable from manifests will
   be in its first version. The judgement is that one true model with a worse renderer beats two
   models with a better one, and it should be revisited if the renderer stays poor.

**Gained**

- A system-context view becomes possible at all, which it was not before item 3.
- The architecture picture cannot disagree with the estate, because it is a function of it.
- C-0004 governs the output for free: generated, regenerated, never hand-edited.
- `docs.diagram_parses` closes a hole open since 2026-08-06 in which a required diagram could be
  syntactically broken and every check stayed green.

**Rejected alternatives**

**Adopt Structurizr DSL as the source of truth.** Rejected: it duplicates manifest facts. The
proposal that raised it also assumed a monorepo layout (`architecture/model/`, `architecture/views/`),
which ADR-0001 forbids — this is a polyrepo with knowledge vendored into every repository, so
there is no single tree for a model to live in. Generation from manifests is the polyrepo-shaped
answer.

**Put the model in the `knowledge` repository.** Rejected: `knowledge` holds rules, not facts
about the estate. A service list here would go stale exactly as ADR-0009 records the previous
estate going stale, and `knowledge` is sui generis precisely so it does not accumulate
descriptive data.

**Add a `spec-architecture` repository.** Rejected: a third place for facts that already have a
home, and it inherits every drift problem of the DSL option while also needing a class, a
standard and a workflow.

**Do nothing and keep only per-repository diagrams.** Rejected: it is the status quo, and the
status quo has no answer to *what talks to what* that survives the second service.

## Related

- STD-0002 — requires the hand-written diagram; unchanged by this, and deliberately not superseded
- ADR-0001 — polyrepo with vendored knowledge; why a single `architecture/` tree cannot exist
- C-0004 — generated output is regenerated, never hand-edited; governs the generated views
- ADR-0030 — the verification bridge; `verified_by: [TST-0061]` is that bridge for this artifact
- ANTI-0001 · ANTI-0002 · ANTI-0003 — the architecture rules TST-0061 verifies
- REQ-0003 · REQ-0012 — the two deferred requirements `data_shared` also serves
- ADR-0014 — why a declared-but-unimplemented check id is worth declaring
