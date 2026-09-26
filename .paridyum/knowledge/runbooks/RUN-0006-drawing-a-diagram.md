---
id: RUN-0006
title: Drawing a diagram — pick the kind, validate the IR, commit both files
type: runbook
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [diagrams, archify, mermaid, adr-0065, std-0020, tooling, onboarding]
breaking: false
enforced_by: []
---

# RUN-0006 — Drawing a diagram

The answer to *"I need a picture of this"*, asked by someone who has a thing to explain and does
not want to read the governing artifacts first. ADR-0065 is the decision, STD-0020 is the
standard, and
this is the procedure.

## The whole model in five lines

1. If the reader is in a **pull request**, use Mermaid. GitHub renders it there and renders
   nothing else.
2. Otherwise, if the subject has **ordering, state, time or branching**, use archify.
3. You write **JSON**. The tool writes the **HTML**. You never edit the HTML.
4. `validate`, then `./scripts/render-diagram.sh`. Both files go in **one commit**.
5. A diagram never holds a fact that is not already written down somewhere else.

## 0. Install the tool

Once per workstation. The pin is a commit sha in `tool-registry.yaml` and the installer reads it
from there, so there is nothing to type and nothing to remember.

```sh
./scripts/install-tools.sh archify
```

It clones at the pinned sha into `~/.paridyum/tools/archify/<sha>/`, runs `archify doctor`, and
points `~/.paridyum/tools/archify/current` at it. Re-running is safe and does nothing when the
sha already matches.

**Do not run `npx skills add tt-a1i/archify -g`**, which is the install the project's README
documents. It delegates the clone to a third-party CLI at no pinned revision — an unpinned
dependency fetching an unpinned dependency. ADR-0065 decision 5 declines it.

**Do not put `archify` on `PATH`.** Scripts resolve it with:

```sh
"$(./scripts/tool-path.sh archify)" doctor
```

A tool found on `PATH` is whatever the workstation happened to install, which is the drift the
registry exists to close.

## 1. Pick the kind

| The subject is… | Kind |
|---|---|
| ordered steps across lanes, with branches and gates | `workflow` |
| messages between participants over time | `sequence` |
| where data moves and which boundaries it crosses | `dataflow` |
| states a thing occupies, and how it leaves them | `lifecycle` |
| components, boundaries and connections | **you may not write one — see §6** |

Unsure? Ask the tool:

```sh
"$(./scripts/tool-path.sh archify)" guide "the path a request takes through the tenancy chain"
```

If two kinds seem to fit, the subject is probably two diagrams.

## 2. Write the IR

```
docs/diagrams/<subject>.<kind>.json
```

`<subject>` is kebab-case and names the thing, not the picture — `change-lifecycle`, not
`change-lifecycle-diagram`.

Start from a real one rather than a blank file. The bundled examples are the best reference and
the ones in this repository are the closest to house style:

```sh
ls knowledge/docs/diagrams/*.json
"$(./scripts/tool-path.sh archify)" examples
```

Four rules that are easy to get wrong and expensive to fix later:

- **`meta.subtitle` cites the artifact.** `"ADR-0013 · the change lifecycle"`. A reader who wants
  the authority must be able to get there from the picture.
- **Node ids are permanent.** They are the anchors in `#focus=` and `#route=` links, so an id is a
  URL someone may have pasted into an artifact. Renaming one breaks a citation silently.
- **Do not set `meta.locale`.** GLO-0006 makes locale four independent fields resolved through
  the tenancy chain; archify's is a presentational switch. A fifth locale concept in this
  repository is not wanted.
- **Do not set `animation: "trace"` unless sequence is the point.** A diagram that animates
  because it can is one that says everything is important.

## 3. Validate, preview, render

```sh
A="$(./scripts/tool-path.sh archify)"

"$A" validate lifecycle docs/diagrams/artifact-status.lifecycle.json --json
"$A" preview  lifecycle docs/diagrams/artifact-status.lifecycle.json    # watches; keeps last good
./scripts/render-diagram.sh docs/diagrams/artifact-status.lifecycle.json
```

**Quote it.** `"$A"` and not `$A` — `tool-path.sh` prints one executable path precisely so this
works under zsh, which does not word-split unquoted expansions, and RUN-0001 names macOS a
supported workstation. The unquoted form was written first here and failed on the first machine
it was run on.

`validate --json` prints path-prefixed diagnostics with rule codes — `meta.views[2].focus[0]`
naming the node id that does not exist, not "invalid document".

`deliver` is atomic: it validates, renders, checks the output, and only then writes. A failed
render leaves the previous HTML intact.

**Do not call `archify deliver` yourself.** `render-diagram.sh` wraps it because the vendor's
template links JetBrains Mono from `fonts.googleapis.com` — the diagram works offline, but opening
one online tells a third party that someone read it. The script deletes those three references and
refuses to write a file that still reaches an external origin. The visible cost is that diagrams
render in system monospace. It also infers the kind from the filename, so there is one argument
instead of four, and it refuses an architecture IR outright (§6).

## 4. Commit both files, together

```sh
git add docs/diagrams/artifact-status.lifecycle.json \
        docs/diagrams/artifact-status.lifecycle.html
```

`docs.diagram_source_is_present` fails on an HTML with no IR beside it.
`docs.diagram_is_current` regenerates and compares, so committing the JSON without the HTML is a
red build for whoever pushes next.

## 5. Changing one later

Edit the JSON. Run `./scripts/render-diagram.sh` on it. Commit both.

**Never open the HTML to fix a label.** It is generated output under C-0004 — regenerated, never
hand-edited — and the next `deliver` destroys the edit without telling anyone.

## 6. Why you cannot draw an architecture diagram

Because `manifest.yaml` already is one. It declares ownership, exposure and dependency for every
repository, and ADR-0040 decided that the architecture model has exactly one home and every
cross-repository view is *generated* from it. An `architecture`-kind IR would be a second copy,
and a second copy of that enumeration is the defect this repository has been caught by five times.

The generator is owed by `forge` and does not exist yet. Until it ships:

- for components and their edges, use the **Mermaid block in `ARCHITECTURE.md`** that STD-0002
  already requires — that one is deliberately hand-written, because it answers *why this
  boundary*, and no generator infers intent;
- for anything with ordering, state, time or branching, the four kinds above are open.

## 7. When something goes wrong

| Symptom | Cause | Fix |
|---|---|---|
| `archify: command not found` | resolved from `PATH` | use `./scripts/tool-path.sh archify` |
| `doctor` reports a missing renderer | partial clone | delete the sha directory and re-run the installer |
| validate names a node id you can see in the file | the id is referenced from `meta.views` or an edge and defined nowhere | ids are matched exactly; check case and hyphens |
| the HTML opens blank | you edited the HTML | regenerate from the IR; §5 |
| the diagram fetches a font when opened | rendered with a bare `archify deliver` | re-run `render-diagram.sh` |
| the diagram is right and the build is red | the HTML was not regenerated with the IR | run `render-diagram.sh`, amend the commit |
| a label is unreadable at the default zoom | `viewBox` is too small for the content | widen `meta.viewBox`; the renderer does no auto-layout on purpose |

## What this runbook does not cover

**Publishing.** Nothing here is published anywhere. The HTML is a file in the repository, opened
from disk. There is no hosted viewer, no gist and no share link, and STD-0020 §8 says so as a rule
rather than an omission.

**Brand marks.** `archify brands capture` fetches and digest-pins a vendor logo. It must never be
pointed at Paridyum's own mark — STD-0015's `ui.mark_is_not_redrawn` requires byte-identity
against the source board and a capture is a re-export by definition.

## Related

- ADR-0065 — the decision
- STD-0020 — the standard this procedure implements
- ADR-0040 — why §6 says no
- STD-0002 — the Mermaid block that stays
- C-0004 — generated output; §5
- RUN-0001 — workstation setup, which this extends
- `tool-registry.yaml` — the pin, the licence, and the exit plan
