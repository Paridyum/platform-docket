---
id: ADR-0065
title: Diagrams are rendered by archify, and the architecture model stays in manifests
type: adr
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, web, console, tool, library, service, spec]
applies_to: ["*"]
tags: [diagrams, visualisation, archify, mermaid, adr-0040, c-0004, req-0012, req-0014, std-0002, std-0015, documentation]
breaking: false
enforced_by:
  - check: docs.diagram_ir_is_valid
    severity: error
    rationale: >
      A committed IR that does not validate produces no HTML at all, and the failure surfaces
      whenever someone next regenerates rather than when it was introduced. `archify validate
      --json` already emits path-prefixed diagnostics with rule codes, so this is the cheapest
      check here and the only one that needs no judgement.
  - check: docs.diagram_source_is_present
    severity: error
    rationale: >
      A rendered HTML with no IR beside it is an un-regenerable file, which C-0004 forbids by
      making generated output regenerable and which nothing would otherwise detect — the HTML
      opens perfectly, and it is the source that is missing.
  - check: docs.diagram_is_current
    severity: error
    rationale: >
      The failure mode of generated output committed to a repository is staleness rather than
      incorrectness, and a stale diagram is more dangerous than an absent one because it looks
      authoritative. The same reasoning `docs.system_view_is_current` records for ADR-0040.
  - check: docs.architecture_ir_is_hand_authored
    severity: error
    rationale: >
      ADR-0040's `docs.no_hand_authored_architecture_model` is keyed on file shape and names
      three extensions that predate this format. An archify architecture IR is a modelling
      language the existing check cannot see, and it is the sixth instance of the defect that
      check exists to prevent.
  - check: docs.diagram_is_self_contained
    severity: error
    rationale: >
      The output's whole value here is that it is a file rather than a hosted page. An HTML that
      reaches the network at open time is a page pretending to be a file, and it breaks both the
      offline-readable posture of ADR-0001 and REQ-0012 §4.
---

# ADR-0065 — Diagrams are rendered by archify, and the model stays in manifests

## Status

**ACCEPTED** as of 2026-09-05. Not locked — per the `new-decision` skill §5b an artifact is
created `accepted`, its facts are checked, and it is locked in a separate commit.

Designed in RFC-0040, which holds the evidence. This artifact decides; that one argues.

## Context

STD-0002 has required one Mermaid diagram per repository since 2026-08-06, and that is the
estate's entire visual capability. Mermaid renders a container sketch adequately and renders a
seven-lane workflow, a retry lifecycle or a token exchange badly — as an unreadable tangle whose
layout nobody controls. The gate's own section ordering, the artifact lifecycle and the tenancy
chain are all things this repository explains in prose because there was no way to draw them.

`archify` was proposed as the fix. It is MIT, it has no runtime dependencies, it never touches
the network while rendering, and it emits a single self-contained HTML file — which is the only
shape acceptable here, because the standing rule is that deliverables are files in the repository
and nothing sits on a vendor's servers.

**And adopting it naively breaks ADR-0040.** Its `architecture` renderer consumes a document
declaring `components`, `boundaries` and `connections`. ADR-0040 decision 2 forbids exactly that
file — *"no `workspace.dsl`, `.puml`, `.d2` or equivalent declaring systems, containers or
relationships"* — because `manifest.yaml` already owns those facts and a second copy of an
enumeration is the defect this repository has been caught by five times.
`docs.no_hand_authored_architecture_model` is keyed on **file shape** and names three extensions.
A `.architecture.json` is not one of them. The violation would have passed every gate.

Two facts decide the shape of the answer:

1. **ADR-0040 decision 1 already anticipated this.** *"One model, many views. Adding a view is a
   renderer, never a new source of facts."* Archify is a renderer. The error would be letting it
   become a model.
2. **Four of the five renderers have no model to be second to.** `manifest.yaml` declares
   ownership, exposure and dependency. It declares no ordering, no state, no time and no branch.
   ADR-0040 could not have forbidden a second copy of a workflow, sequence, data flow or
   lifecycle, because there is no first copy.

The supply chain was checked and is worse than the licence suggests: the package is
`"private": true` and is **not on npm**, and its newest tag is `v2.9.0` against a shipped version
of `2.17.0-dev.1`, so **no release tag corresponds to the code anyone would run**. RFC-0040
finding 4 holds the evidence.

## Decision

**Archify renders the estate's diagrams. It never holds a fact that a manifest already holds.**

1. **Hand-authored IR is legal for `workflow`, `sequence`, `dataflow` and `lifecycle`, and for
   nothing else.** These four describe ordering, state, time and branching, which no
   `manifest.yaml` field declares. For them the committed IR is the **first** copy and is
   authored source, not generated output.

2. **Architecture-kind IR is generated from the set of manifests and is generated output under
   C-0004.** It is ADR-0040 decision 1's renderer, arriving four weeks later with a name.
   A hand-authored `.architecture.json` outside a generated directory is forbidden and
   `docs.architecture_ir_is_hand_authored` fails on one. **This forbids more than it enables
   today**, because the generator does not exist — see accepted cost 1.

3. **ADR-0040 is neither superseded nor errata'd.** Nothing it decides changes. This artifact
   applies its decision 1 to a format its decision 2's check cannot see, and extends the
   file-shape list rather than the rule.

4. **Mermaid remains STD-0002's floor and is not replaced.** The requirement of one Mermaid
   diagram in `ARCHITECTURE.md` is unchanged. Mermaid renders inside a GitHub diff and a pull
   request review; a self-contained HTML renders there as a blob a reviewer must clone to see.
   An archify diagram is an addition to a repository's documentation, never a substitution for
   the block `docs.diagram_parses` gates.

5. **The pin is a commit sha, and the install is a `git clone`.** Not a version number — no tag
   matches the shipped code — and not `npx skills add`, which is a third-party CLI performing an
   unpinned clone. The sha lives in `tool-registry.yaml` (ADR-0066), never in this artifact,
   because ADR-0048 forbids a value on a cadence we do not control from living where correcting
   it is heavier than the correction.

6. **The IR is the source and the HTML is the output.** Both are committed: the IR because it is
   what a human edits, the HTML because a reader must be able to open it from a checkout with no
   toolchain. The HTML is generated output under C-0004 — regenerated, never hand-edited —
   and `docs.diagram_is_current` compares it against a regeneration.

7. **A rendered diagram must be self-contained and must not reach the network when opened, and
   the vendor's output is not.** This was found by checking rather than by reading the README,
   and it corrects what this decision said in its first draft. `archify`'s template links
   **JetBrains Mono from `fonts.googleapis.com`** with a `preconnect` to `fonts.gstatic.com`.
   The load is asynchronous behind `media="print"` and the stack falls back to system monospace,
   so a diagram renders correctly with no network at all — the vendor's offline story is sound.
   What it is not is *silent*: opening a committed governance diagram issues a request to a third
   party and discloses that someone read it.

   **The render step is therefore two commands, not one.** `scripts/render-diagram.sh` runs
   `deliver` and then removes those three references, and refuses to write a file that still
   reaches an external origin. That is generation, not hand-editing: the pipeline is ours, it is
   deterministic, and C-0004 is satisfied because the output is still produced by running a
   command rather than by someone opening it. The cost is the typeface — see accepted cost 6.

   `archify brands capture` is the only *command* that fetches, it is explicit, and its results
   are digest-pinned. It **must never be pointed at Paridyum's own mark**: STD-0015's
   `ui.mark_is_not_redrawn` requires byte-identity against the source board and a capture is a
   re-export by definition.

8. **STD-0015 does not govern a diagram.** `visual_preset` is a closed vendor enum —
   `classic`, `signal-flow`, `blueprint`, `editorial` — and the estate's tokens cannot be applied.
   A diagram is engineering output, not a brand surface. Said explicitly because the alternative
   is a standard that silently fails on every diagram, which is the shape of rule nobody obeys.

9. **A diagram is never the only home of a fact.** It renders what an artifact, a manifest or a
   script already decides. RFC-0015 is the rule; a picture is the most attractive place to break
   it, because a diagram cannot fail a build and is read more often than the thing it describes.

## Consequences

**Accepted costs**

1. **Decision 2 forbids something nothing can yet produce.** The architecture generator is
   `forge`, which does not exist. Until it does, the estate has four usable diagram kinds and one
   that is legal only in a directory nothing writes to. The alternative — permitting hand-authored
   architecture IR "for now" — is how the second copy would arrive, and a temporary permission on
   a path already walked five times is not temporary. Recorded as an open question in RFC-0040
   rather than softened here.

2. **Every check this artifact names runs today, and each one skips where archify is absent.**
   Each is implemented in `check-repo.sh` and defect-tested in `scripts/test-checks.sh`.
   `docs.diagram_is_current` was declared against `tool-forge` first, on the honest ground that
   byte-reproducibility was the vendor's claim rather than a measured property; the claim was then
   measured — six diagrams re-render byte-identically, and the same IR delivered to two different
   filenames produces identical bytes — and the check was written. The declaration and the
   measurement are two different acts, and only the second earns the `owner` column.

   **The remaining hole is the skip, and it is deliberate.** A repository whose contributor has
   not installed archify at its pinned sha gets a *warning* and no IR validation and no currency
   comparison. A hard failure there would make every contributor install a Node tool to commit an
   unrelated change, which is a gate people bypass — ADR-0002's argument applied to the cost of
   *running* a check rather than the cost of writing one. The skip is printed, never silent.

   **And if the renderer's output moves between shas, all six diagrams report stale on the day
   the pin advances.** That is arguably the correct signal and it will be a surprising one; the
   remedy is to re-render in the same change that advances the pin.

3. **A committed HTML is a large binary-shaped file in a text repository.** A showcase-quality
   render is hundreds of kilobytes of inline SVG, it diffs uselessly, and it will dominate the
   byte count of any repository that carries several. Committing it anyway is deliberate: the
   alternative is a diagram nobody can open without installing a Node tool at a pinned sha, which
   makes the documentation conditional on the toolchain and defeats the reason the format was
   chosen.

4. **A single-maintainer, pre-1.0, unpublished dependency is now in the documentation path.**
   `2.17.0-dev.1` is a development version by its own label, and there is no npm provenance to
   verify. Mitigated rather than solved: the sha is content-addressed, the IR is ours, and
   STD-0002's Mermaid floor is a working fallback rather than a theoretical one. That is the whole
   of REQ-0012 §5's exit plan and it is cheap only because it was written while the integration
   was small.

5. **The estate now owns a render step because the vendor's output was not acceptable as
   shipped.** `scripts/render-diagram.sh` exists only to delete three lines archify writes, and
   it is a small permanent maintenance burden that moves whenever the upstream template does —
   which is why it is anchored on the two font hosts by name rather than on `<link>`, so a
   *different* external reference appearing later is caught by the verification instead of being
   silently swallowed. The alternative was to weaken decision 7 to match the tool, and a rule
   rewritten to match what a vendor happens to do is not a rule.

6. **Diagrams render in system monospace rather than the typeface they were designed against.**
   A visible difference, and the direct cost of decision 7. Embedding the font as a data URI, or
   upstreaming a flag, would remove it — MIT permits the fork and the change is four lines — but
   a forked renderer is a second thing to pin and it is not worth that yet. `DECISION-DEBT.md`.

7. **Two diagram tools instead of one.** Every repository now has a Mermaid block and may have
   archify HTML, and a reader must know which answers which question. STD-0020 exists to make that
   choice mechanical rather than a matter of taste, and it is a standard that would not be needed
   if the estate had one renderer.

**Gained**

- Ordering, state, time and branching become drawable at all. They were not.
- The rendered output is a file in the repository, which is the only posture permitted here.
- `archify validate --json` makes a diagram **checkable**, which no Mermaid block has ever been
  beyond parsing.
- ADR-0040's decision 1 acquires a concrete renderer, which it has lacked since 2026-08-09.
- The `compare` subcommand produces a machine receipt of added, removed and rerouted
  relationships between two architecture snapshots — a reviewable diff of a picture, which
  nothing in the estate has ever had.

**Rejected alternatives**

**Keep Mermaid only.** Rejected: it is the status quo, and the status quo cannot draw four of the
five kinds legibly. Mermaid's layout is chosen by the tool, so the author's control ends at the
topology — which is precisely wrong for a workflow whose point is which lane a step belongs to.

**Adopt archify fully, including hand-authored architecture IR.** Rejected: it is ADR-0040
decision 2 violated by a file extension that check cannot see, and RFC-0040 finding 1 shows the
violation is exact rather than technical. The convenience is real and the cost is a second copy
of the estate's dependency graph that no gate would ever compare against the manifests.

**Supersede ADR-0040.** Rejected: nothing it decides is wrong. Superseding a correct artifact to
make room for a tool is the shape of drift ADR-0052 names, and it would discard the argument that
made this decision available.

**Adopt Structurizr or D2 instead.** Rejected for the reason ADR-0040 already gave, unchanged:
they duplicate manifest facts. Neither draws a lifecycle or a sequence better than archify, and
both are the specific tools decision 2 names.

**Install via `npx skills add tt-a1i/archify -g`, as documented.** Rejected: it delegates the
clone to a third-party CLI at no pinned revision, which is an unpinned dependency fetched by an
unpinned dependency. REQ-0014 asks for provenance and this offers less than a plain `git clone`
at a sha, which is content-addressed and needs nothing.

**Vendor archify into `knowledge` and ship it with the snapshot.** Rejected: ADR-0001 vendors
*knowledge* into every repository, and it is a snapshot of rules rather than a bundle of binaries.
A Node tool in that tree would be copied into every checkout by `forge knowledge sync` and would
make the rule snapshot a distribution channel for third-party code.

**Render on demand and commit only the IR.** Rejected: it makes reading the documentation
conditional on having Node and the pinned sha installed, so the diagram is unavailable to exactly
the reader who has just cloned the repository. Accepted cost 3 is the price of avoiding that.

## Related

- RFC-0040 — the design and the evidence; findings 1 and 2 are what decisions 1 and 2 rest on
- ADR-0040 — the model is `manifest.yaml`; applied here, not amended
- ADR-0066 — the tool registry that holds the sha this artifact refuses to state
- ADR-0048 — why the pin is not in this file
- C-0004 — generated output; governs the HTML and the architecture IR
- STD-0002 — the Mermaid floor, unchanged
- STD-0015 — does not govern a diagram; decision 7 names the one command it forbids
- STD-0020 — how to author one
- RFC-0015 — a fact has one home; decision 9
- REQ-0012 · REQ-0014 · REQ-0007 — vendor approval, provenance, and agent egress
