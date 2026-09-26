# Diagrams

Rendered by `archify` from the typed JSON IR beside each file. **ADR-0065** decides, **STD-0020**
is the standard, **RUN-0006** is the procedure.

Open the `.html` from disk. Nothing here is hosted, and nothing reaches the network when opened.

| Diagram | Kind | Answers |
|---|---|---|
| [`change-lifecycle`](change-lifecycle.workflow.html) | workflow | What happens to a change, and which four stages ordinary work skips |
| [`knowledge-gate`](knowledge-gate.workflow.html) | workflow | The order `check-knowledge.sh` runs in, and why section 0 is load-bearing |
| [`tool-adoption`](tool-adoption.workflow.html) | workflow | How a tool enters the estate, and the three places it is refused |
| [`artifact-status`](artifact-status.lifecycle.html) | lifecycle | The statuses an artifact takes, and the only two ways out of `locked` |
| [`enforcement-layers`](enforcement-layers.sequence.html) | sequence | ADR-0007's four layers on one attempted edit, and each layer's escape |
| [`tenancy-chain`](tenancy-chain.dataflow.html) | dataflow | A row's path through nested tenancy, and where it fails closed |

## Editing one

```sh
./scripts/install-tools.sh archify              # once, at the pinned sha
./scripts/render-diagram.sh docs/diagrams/<name>.<kind>.json
```

The `.json` is authored source. The `.html` is generated output under C-0004 — **regenerated,
never hand-edited** — and both belong in the same commit.

## What is not here

**No architecture diagram.** ADR-0040 puts the architecture model in `manifest.yaml` and makes
every cross-repository view generated from it, so an `architecture`-kind IR would be a second copy
of facts the manifests already own. `docs.architecture_ir_is_hand_authored` rejects one. The
generator is owed by `forge` and does not exist yet (TASKS T-2.20); until it ships, components and
their edges belong in the hand-written Mermaid block STD-0002 requires in `ARCHITECTURE.md`.

**No brand.** `visual_preset` is a closed vendor enum and STD-0015's tokens cannot be applied. A
diagram is engineering output, not a brand surface.

**No fact that lives only here.** Every diagram renders what an artifact, a registry or a script
already decides. A picture cannot fail a build and is read more often than the thing it describes,
which makes it the most attractive place to break RFC-0015.
