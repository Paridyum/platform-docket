---
id: RFC-0040
title: A diagram is a renderer, and the model stays where it already is
type: rfc
status: proposed
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra, platform, web, console, tool, library, service, spec]
applies_to: ["*"]
tags: [diagrams, visualisation, archify, mermaid, adr-0040, adr-0022, adr-0048, c-0004, req-0012, req-0014, std-0002, std-0015, tooling, supply-chain]
breaking: false
enforced_by: []
---

# RFC-0040 — A diagram is a renderer, and the model stays where it already is

## Status

**Proposed**, 2026-09-05. Written before anything was adopted, because ADR-0040 already decides
part of this question and a tool adopted first would have been an amendment made by installation.

Two decisions are proposed and they are deliberately separable: **ADR-0065** adopts a renderer,
**ADR-0066** creates the registry that records which tools the estate uses at all. Either can be
rejected without the other.

## What was asked for

> "Integrate archify and ideate how to use this in every governance — use it wherever possible
> for slicker visualisation. Maintain a tool list in governance that these are the stuff we used."

Two requests wearing one sentence. The first is a rendering question. The second is a
registry question, and it turned out to be the larger of the two.

## What archify is

`github.com/tt-a1i/archify`, MIT, `2.17.0-dev.1`. A Node CLI plus an agent skill. An author
writes a **typed JSON intermediate representation**; the CLI compiles it deterministically to a
**single self-contained HTML file** with embedded SVG.

Five renderers, and the split between them is the load-bearing fact for this document:

| Kind | Structural arrays | Is there already a model of this? |
|---|---|---|
| `architecture` | `components`, `boundaries`, `connections` | **Yes — `manifest.yaml`** |
| `workflow` | `lanes`, `phases`, `groups`, `mainPath`, `nodes`, `edges` | No |
| `sequence` | `participants`, `segments`, `messages`, `activations` | No |
| `dataflow` | `stages`, `nodes`, `flows` | No |
| `lifecycle` | `lanes`, `states`, `transitions` | No |

Verified on this workstation on 2026-09-05 at commit `d8e4daf`: `archify doctor` reports all
fifteen subsystems ok under Node 26.8.1, and all five renderers render their bundled examples.

**Properties that were checked rather than read off the README:**

1. **Zero runtime dependencies.** `ajv`, `parse5`, `saxes` and `simple-icons` are
   `devDependencies`. Nothing is installed to render.
2. **The output is a file.** No server, no hosted viewer, no account. This is the property that
   makes it usable here at all — the standing rule is that deliverables are files in the
   repository and nothing sits on a vendor's servers.
3. **It fails closed on the network.** `schemas/README.md`: *"render and validate never perform an
   unpinned network capture."* Brand marks must be captured by an explicit command and are
   digest-pinned. REQ-0012 §4 asks for egress control and this asks for none.
4. **`additionalProperties: false` at every level.** An unknown field is rejected rather than
   ignored, so a typo in an IR is a failure and not a silently missing node.
5. **`validate --json` emits path-prefixed diagnostics with rule codes.** This is what makes a
   check possible; a renderer that only fails visually cannot be gated.
6. **No custom palette.** `visual_preset` is a closed enum — `classic`, `signal-flow`,
   `blueprint`, `editorial`. STD-0015's tokens cannot be applied. See finding 3 below.

## Finding 1 — adopting archify naively violates ADR-0040, and the violation is exact

ADR-0040 decision 2:

> **A hand-authored architecture model is forbidden.** No `workspace.dsl`, `.puml`, `.d2` or
> equivalent declaring systems, containers or relationships.

`docs.no_hand_authored_architecture_model` is keyed on file shape, *"because deciding whether a
diagram declares a model is not mechanically decidable"*. An
`architecture.schema.json` document declares `components`, `boundaries` and `connections`. That is
systems and relationships in a modelling language, in a hand-maintained file, under a different
extension. It is the **sixth** instance of the second-copy defect ADR-0040 says this repository
has been caught by five times, and it would arrive wearing a tool nobody would think to check.

The reflex is to read this as ADR-0040 blocking a good tool. It is not. ADR-0040 decision 1 already
says what the answer is:

> **One model, many views.** Adding a view is a **renderer**, never a new source of facts.

Archify is a renderer. The mistake would be letting it become a model.

## Finding 2 — ADR-0040 says nothing about four of the five kinds, and that is not an oversight

`manifest.yaml` declares `owns.tables`, `owns.events`, `exposes.http`, `exposes.events`,
`depends.services`, `depends.infra`, `depends.events`, and — since ADR-0040 item 3 —
`depends.external` and `actors`. That is a C4 Level 2 container model with typed edges.

It declares **no ordering, no state, no time and no branch**. There is no manifest field from which
the gate's section order, the artifact lifecycle `proposed → accepted → locked → superseded`, a
token exchange, or the path a row takes through the tenancy chain could be derived. ADR-0040 could
not have forbidden a second copy of those facts, because there is no first copy to be second to.

So the boundary is not a compromise between the tool and the rule. It is where the rule's own
argument stops:

> ADR-0040 forbids a second copy of **architecture facts**, which `manifest.yaml` owns. A workflow,
> a sequence, a data flow and a lifecycle have no model behind them, and a hand-authored IR for
> those is the **first** copy, not the second.

## Finding 3 — three things archify does not replace, and one it must not touch

**Mermaid stays.** STD-0002 requires one Mermaid diagram in `ARCHITECTURE.md` and
`docs.diagram_parses` gates it. Mermaid renders **in a GitHub diff and in a pull request review**;
a self-contained HTML file renders as a binary-looking blob that a reviewer must clone to see.
Whatever archify is better at, it is worse at the one place a reviewer actually reads. Mermaid is
the floor, not the loser.

**STD-0015 does not apply.** The palette is a closed vendor enum. `ui.brand_accent_is_not_a_text_colour`
and `ui.tokens_are_not_locally_defined` govern product surfaces built from `lib-ui` tokens, and a
diagram is not one. Stating this matters because the alternative is a standard that silently
fails on every diagram, which is the shape of rule nobody obeys.

**The mark must not be captured.** `archify brands capture <url>` fetches and digest-pins a logo.
STD-0015's `ui.mark_is_not_redrawn` requires byte-identity of the Paridyum mark against the source
board, and a capture is a re-export by definition. Pointing that command at Paridyum's own mark is
the exact defect that check exists to catch. External vendor marks are unaffected.

## Finding 4 — the supply chain is worse than the licence suggests, and it is still acceptable

REQ-0012 §1 wants approval before adoption and §5 wants an exit path. REQ-0014 wants provenance.

- **MIT.** Vendorable, forkable, no exit obstacle in the licence.
- **`"private": true`.** The package is **not published to npm**. The documented install is
  `npx skills add tt-a1i/archify -g`, which is a third-party CLI cloning a GitHub repository.
  There is no registry signature, no npm provenance attestation, and no immutable published
  artifact.
- **The tags are stale.** Latest tag `v2.9.0`; shipped version `2.17.0-dev.1`. **No release tag
  corresponds to the code anyone would actually run.** A version-number pin would be a pin to
  something that does not exist.
- **Single maintainer, pre-1.0 in substance.** `2.17.0-dev.1` is a development version by its own
  label.

The conclusion is not "reject". It is that **the pin must be a commit sha and the install must not
go through the `skills` CLI**. A sha is content-addressed and is the only thing here that is
immutable. `git clone` at a sha is the whole install, because there is no build step and no
dependency tree to resolve.

**Exit path, stated while the integration is small (REQ-0012 §5):** the IR is ours — plain JSON in
our repositories, against a schema we hold a copy of. Losing archify costs the *renderer*, not the
*content*, and the fallback is the Mermaid that STD-0002 already requires. That is why the Mermaid
floor in finding 3 is also the exit plan, and why removing it to "avoid duplication" would trade a
small redundancy for an unbounded switching cost.

## Finding 5 — the tool list has nowhere to live, and this is the sixth instance of a known pattern

The second request. `toolchain-registry.yaml` was checked first and it is **not** the home:

> "THE PINNED COMPILER OR RUNTIME FOR EACH STACK. One entry per stack, and the entry is the
> authority."

One entry per stack. A tool is not a stack, and the file already shows the strain — `talosctl`,
`helm`, `kubectl`, `cilium`, `sops`, `age` and `ansible-lint` are parked under a `tooling:` key
inside the stack that happened to need them first. The file says so itself about `talosctl`:
*"A stack is a thing you write code in, and nobody writes Talos."* That is a correct sentence
solving the wrong problem: it explains why `talosctl` is not a stack without providing anywhere
else for it to be.

Everything else has no entry at all. `gh`, `ripgrep`, `jq`, `python3` and its schema validator,
`mermaid`, the agent harness plugins — the estate depends on all of them and none is declared.

This is the pattern ADR-0014, ADR-0015, C-0005, ADR-0022 and ADR-0048 have each recorded once:
**a list that must be extended, hardcoded in a place extending it is expensive** — here, in no
place at all. Its consequence is documented three times in `toolchain-registry.yaml`'s own notes:
with no entry to derive from, the first repository pins whatever the workstation had and the next
copies it. That is how an end-of-life Node runtime spread across three repositories.

**The proposal is a `tool-registry.yaml` that owns adoption facts and owns no version number.**
Purpose, licence, install source, the artifact that adopted it, the exit path, the criticality
tier. For a tool already pinned in `toolchain-registry.yaml`, the entry carries
`pin_home: toolchain-registry` and **no version**, because RFC-0015 gives a fact one home and a
version copied into a second registry is the identical defect one level sideways.

**Migrating those pins is explicitly not proposed here.** It would change behaviour while moving
it, which is the lesson `artifact-registry.yaml` records about ADR-0010's `ci: false`. It goes to
`DECISION-DEBT.md`.

## Alternatives considered

**Keep Mermaid and add nothing.** The honest baseline, and the one this document had to beat
rather than assume. Mermaid is already installed nowhere — GitHub renders it — so it costs no
supply chain, no pin and no exit plan, and every argument below is an argument for paying those.
Rejected because Mermaid chooses the layout, which means the author's control ends at the
topology. For a workflow whose whole point is which lane a step belongs to, the layout *is* the
content. The gate's section ordering and the tenancy chain are explained in prose today because
of this, not because prose was preferred.

**Structurizr DSL or D2.** Rejected for the reason ADR-0040 already gave and this document does
not improve on: they duplicate manifest facts, and they are two of the three extensions
`docs.no_hand_authored_architecture_model` names. Neither draws a lifecycle or a sequence better
than archify, so adopting one would accept ADR-0040's cost without gaining the four kinds that
motivated the change.

**Excalidraw or a drawing tool.** Rejected: the output is not derivable from anything, cannot be
validated, and diffs as a binary. It is the fastest way to a picture and the fastest way to a
picture that quietly disagrees with the estate — and a diagram that cannot fail a build is
already the most dangerous document type here.

**Adopt archify without restriction, including hand-authored architecture IR.** Rejected in
finding 1: it is ADR-0040 decision 2 violated by a file extension that check cannot see. The
convenience is real — an architecture diagram is the one people ask for first — and the cost is
a second copy of the estate's dependency graph that no gate compares against the manifests.

**Adopt archify and supersede ADR-0040.** Rejected: nothing ADR-0040 decides is wrong. Superseding
a correct artifact to make room for a tool is the drift ADR-0052 names, and it would discard the
argument in decision 1 that made this adoption available in the first place.

**Put the tool list in `toolchain-registry.yaml`.** Rejected in finding 5: its header states one
entry per stack and its authority is the pin. Widening it means inventing a fake stack for `gh`
and `jq`, or nesting them under an unrelated one — which is what the `tooling:` key already is,
adopted deliberately rather than by accident.

**Put the tool list in a `TOOLS.md` table.** Rejected: it is the list-in-prose shape C-0005 moved
repository classes out of, and `artifact-registry.yaml` names README and CLAUDE.md prose as the
only two places its list can still rot unobserved. Another such place is not an improvement.

**Generate the tool list by scanning a workstation.** Rejected: it records the machine rather
than the decision, so it cannot hold `purpose`, `adopted_by` or `exit` — the three fields the
request was actually about. It also makes whatever someone happened to install authoritative,
which is the mechanism this proposal exists to close.

## What is proposed

1. **ADR-0065** — archify is the estate's diagram renderer. Architecture-kind IR is **generated
   from manifests** and is generated output under C-0004. Hand-authored IR is legal only for
   `workflow`, `sequence`, `dataflow` and `lifecycle`. Mermaid remains STD-0002's floor. The pin
   is a commit sha. ADR-0040 is **not** superseded and **not** errata'd — it is applied.
2. **ADR-0066** — a tool the estate uses is registry data. Creates `tool-registry.yaml` and
   `tool-registry.schema.json`.
3. **STD-0020** — how a diagram is chosen, authored, located, named, validated and regenerated.
4. **RUN-0006** — the operating procedure for producing one.
5. **Checks** — one implemented and defect-tested now, four declared against `tool-forge`.

## Open questions

1. **Nothing generates the architecture IR yet.** ADR-0065 makes architecture-kind IR generated
   output, and the generator is `forge`, which does not exist. Until it does, the estate has four
   usable kinds and one forbidden one. This is honest and it is also a real gap: the C4 Level 1
   and 2 views ADR-0040 made *possible* in August 2026 are still not *produced*. Naming archify as
   their eventual renderer does not build them.
2. **No advance policy for the sha**, which is ADR-0048 open question 3 arriving in a third
   registry rather than a new problem. A sha nobody advances is how `1.90.0` happened.
3. **Whether `toolchain-registry.yaml`'s `tooling:` keys should migrate.** Deferred above.
4. **Whether an unregistered tool should fail a gate.** `knowledge.tool_is_registered` is
   attractive and probably undecidable — every `sed`, `awk` and `grep` in a script is a tool by
   the literal reading. Declared with a scope narrowed to declared install sources, and the
   narrowing is itself a guess.

## Related

- ADR-0040 — the model is `manifest.yaml`; this document applies it rather than amending it
- ADR-0022 · ADR-0014 · ADR-0015 · ADR-0048 · C-0005 — the five prior instances of finding 5
- ADR-0048 — a value on a cadence we do not control never lives inside an artifact
- RFC-0015 — a fact has one home; why `tool-registry.yaml` carries no version
- C-0004 — generated output is regenerated, never hand-edited
- STD-0002 — the Mermaid floor, unchanged
- STD-0015 — why it does not apply to a diagram, and the one command it forbids
- REQ-0012 — approval, least privilege, exit path; §4 egress and §5 exit
- REQ-0014 — supply-chain integrity and provenance; finding 4 is its evidence
- REQ-0007 — agent authority; a renderer that never fetches is the mitigation it asks for
