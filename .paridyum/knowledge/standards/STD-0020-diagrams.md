---
id: STD-0020
title: A diagram names its kind, keeps its source beside it, and holds no fact of its own
type: standard
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, web, console, tool, library, service, spec]
applies_to: ["*"]
tags: [diagrams, documentation, archify, mermaid, adr-0065, adr-0040, std-0002, c-0004, rfc-0015]
breaking: false
enforced_by:
  - check: docs.diagram_ir_is_valid
    severity: error
    rationale: >
      §5's whole authoring loop is `validate` before `deliver`. A committed IR that does not
      validate renders nothing, and the failure surfaces when someone next regenerates rather
      than when it was introduced.
  - check: docs.diagram_source_is_present
    severity: error
    rationale: >
      §3's layout is the rule this enforces. An HTML with no IR beside it opens perfectly and
      cannot be changed by anyone, which is the failure C-0004 exists to prevent, wearing the
      appearance of working documentation.
  - check: docs.diagram_is_current
    severity: error
    rationale: >
      §6. Generated output committed to a repository fails by going stale rather than by being
      wrong, and a stale diagram is worse than an absent one because it looks authoritative.
---

# STD-0020 — Diagrams

## Status

**`accepted`, and lockable.** Every check in `enforced_by` runs today in `check-repo.sh` and is
defect-tested in `scripts/test-checks.sh`. Each skips, with a printed warning, on a
workstation where archify is not installed at its pinned sha — so §5 and §6 are enforced wherever
the tool is present and upheld by review where it is not. Created `accepted` and locked
separately, per the `new-decision` skill §5b.

This standard says **how**. ADR-0065 decided **whether**, and RFC-0040 holds the evidence.

## 1. Which renderer

Two, and the choice is not a preference.

**Mermaid, inline in markdown.** STD-0002 requires one block in every `ARCHITECTURE.md` and that
requirement is unchanged. Use Mermaid when the reader is **in a diff or a pull request** — GitHub
renders it there and renders nothing else. A diagram a reviewer must leave the review to see is a
diagram nobody checks.

**Archify, as a committed HTML file.** Use it when the thing being drawn has **ordering, state,
time or branching**, which Mermaid lays out for you and therefore badly. The author controls
hierarchy, lanes and spacing; that control is the reason to reach for it.

Neither replaces the other. If you are deleting a Mermaid block to "avoid two tools", stop —
ADR-0065 decision 4 keeps it deliberately and it is also the exit path in `tool-registry.yaml`.

## 2. Which kind

Five renderers. Four are available to you.

| Kind | Draw when the subject is | Example in this repository |
|---|---|---|
| `workflow` | ordered steps across lanes, with branches and gates | the change lifecycle; the knowledge gate's sections |
| `sequence` | messages between participants over time | a token exchange; a request through the tenancy chain |
| `dataflow` | where data moves and which boundaries it crosses | a row's path under row-level security |
| `lifecycle` | states a thing occupies and the transitions between them | an artifact from `proposed` to `superseded` |
| `architecture` | components, boundaries, connections | **generated only — see §7** |

If two kinds seem to fit, the subject is probably two diagrams. A workflow carrying state badges
and a lifecycle carrying step numbers are both the wrong one drawn defensively.

## 3. Where it lives, and what it is called

```
docs/diagrams/<subject>.<kind>.json     the IR — authored source, hand-edited
docs/diagrams/<subject>.<kind>.html     the render — generated output (C-0004)
```

`<subject>` is kebab-case and names the thing, not the picture: `change-lifecycle`, not
`change-lifecycle-diagram`. The kind is in the filename because the CLI takes it as an argument
and a reader should not have to open the file to know which command regenerates it.

**Both are committed.** The IR because it is what a human edits; the HTML because a reader must be
able to open it from a fresh checkout with no Node and no pinned sha installed. ADR-0065 accepted
cost 3 is the price of that and it is paid on purpose.

## 4. Content rules

1. **A diagram holds no fact of its own.** It renders what an artifact, a manifest, a registry or
   a script already decides. RFC-0015 gives a fact one home, and a picture is the most attractive
   place to break that rule — it cannot fail a build and it is read more often than the thing it
   describes. If a diagram is the only place something is written down, **the diagram is not the
   problem**: write the artifact.

2. **`meta.title` names the subject; `meta.subtitle` cites the artifact.** A reader who wants the
   authority must be able to get to it from the picture. `"ADR-0013 · the change lifecycle"` is
   the shape.

3. **Node ids are stable and semantic.** They are the anchors in `#focus=`, `#route=` and
   `#view=` links, so an id is a URL someone may have pasted into an artifact. Rename one and you
   break a citation, silently.

4. **`meta.views` earn their place or are omitted.** Up to five guided views. One view per real
   question a reader has — "where does this pause", "which paths are terminal". Five views that
   restate the same topology are worse than none, because they imply the diagram is more
   complicated than it is.

5. **Prefer `animation: "trace"` only where sequence is the point.** The renderer respects
   `prefers-reduced-motion`, so motion is not an accessibility failure here — it is an attention
   claim, and a diagram that animates because it can is one that says everything is important.

6. **Relabel the legend, always.** The renderer's default legend names its own internal kinds —
   *"Agent logic"*, *"Policy"*, *"Cloud service"* — which mean nothing in this estate and quietly
   assert a vocabulary nobody chose. Set `meta.legend.entries.<kind>.label` to what the shape means
   *here*: a gate that refuses, a registry read, a boundary. The kind is the renderer's; the word
   is ours.

7. **Do not set a locale other than `en`.** GLO-0006 makes locale four independent fields
   resolved through the tenancy chain, and archify's `meta.locale` is a single presentational
   switch over viewer chrome. Setting it would be a fifth, unrelated locale concept in a
   repository that has already been caught calling two different things `tenant`.

## 5. Authoring loop

```sh
A="$(./scripts/tool-path.sh archify)"

"$A" guide    "what I am trying to draw"                 # picks a kind and a starting recipe
"$A" validate <kind> docs/diagrams/x.<kind>.json --json
"$A" preview  <kind> docs/diagrams/x.<kind>.json         # watches, keeps last good output
./scripts/render-diagram.sh docs/diagrams/x.<kind>.json
```

`validate` before rendering, always. `deliver` underneath is atomic — it validates, renders,
checks the output and only then writes — so a failed render leaves the previous HTML intact.

**Resolve the tool, never a bare `archify` on `PATH`**, and **quote the result**. The pin is a sha
in `tool-registry.yaml`, and a tool found on `PATH` is whatever the workstation happened to
install. `tool-path.sh` prints one executable path so `"$A"` works under zsh, which does not
word-split unquoted expansions.

## 6. Regeneration

The HTML is generated output under C-0004: **regenerated, never hand-edited.** Do not open it to
fix a label. Change the IR and re-run `render-diagram.sh`.

A change to the IR and its regenerated HTML belong in the **same commit**. `docs.diagram_is_current`
regenerates and compares, so a split commit is a red build for the person who made the second half.
Check it yourself before committing:

```sh
./scripts/render-diagram.sh --check docs/diagrams/*.json
```

## 7. Architecture diagrams are generated, and you may not write one

ADR-0065 decision 2. An `architecture`-kind IR declares components, boundaries and connections —
which is `manifest.yaml`'s model in a second file, and ADR-0040 decision 2 forbids exactly that.
A hand-authored `.architecture.json` outside a generated directory fails
`docs.architecture_ir_is_hand_authored`.

**Nothing generates them yet.** `forge` is owed that renderer. Until it ships you have four kinds,
and if you need to show components and their edges, that is what STD-0002's Mermaid block in
`ARCHITECTURE.md` is for.

## 8. What a diagram is not

- **Not a brand surface.** `visual_preset` is a closed vendor enum and STD-0015's tokens cannot be
  applied to it. `ui.brand_accent_is_not_a_text_colour` and `ui.tokens_are_not_locally_defined`
  govern product surfaces built from `lib-ui`; they do not govern a diagram, and stating this is
  cheaper than a standard that silently fails on every file.
- **Never a home for the mark.** `archify brands capture` fetches and re-exports a logo. Pointed at
  Paridyum's own mark it is the precise defect `ui.mark_is_not_redrawn` exists to catch. External
  vendor marks are fine.
- **Not published.** The HTML is a file in the repository, opened from disk. It does not go to a
  hosted viewer, a gist, or any service.
- **Not a substitute for prose.** A picture shows shape. It does not show why the shape was chosen,
  and C-0001 principle 8's argument about ADRs applies unchanged: the value is in the rejected
  alternative, and no diagram renders one.

## Related

- ADR-0065 — the decision this implements
- RFC-0040 — the evidence; findings 1 and 2 are §7's argument
- ADR-0040 — the model is `manifest.yaml`
- STD-0002 — the Mermaid requirement, unchanged; §1's floor
- STD-0015 — does not apply; §8
- C-0004 — generated output; §6
- RFC-0015 — a fact has one home; §4 item 1
- GLO-0006 — locale is four fields, not a switch; §4 item 6
- `tool-registry.yaml` — the pin, and the reason §5 refuses a bare `PATH` lookup
