---
id: ADR-0133
title: A game is a repository class, and no name in it carries the engine
type: adr
status: accepted
created: 2026-10-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, game]
applies_to: ["*"]
tags: [game, class, registry, naming, engine, stack, c-0007, adr-0107, adr-0111, rfc-0059, git-lfs, checkpoint, run-0001]
breaking: false
enforced_by: []
---

# ADR-0133 — A game is a repository class, and no name in it carries the engine

## Status

**ACCEPTED** 2026-10-07, by the operator under ADR-0087's personal authority, with RFC-0059.

Accepted rather than locked. What is mechanical here is already checked under other artifacts — the
prefix by `repo.name_matches_scheme` (C-0007), the registry entry by `knowledge.class_registry_valid`
— and the rule this ADR adds, that no name carries an engine, is judgement no check can make.

## Context

**The operator, 2026-10-07:** the naming must be engine-agnostic, *"as we might use ue5 for one game
bevy for other"*. RFC-0059 settles where a gaming line sits and when a game may be built. This decides
what a game's repositories are called and how the estate classifies them, before the first one exists.

**No class fits.**

- `mobile` ships through a store, but its stacks are `typescript` and `dart`, and most games are not
  phone apps.
- `agent` runs on machines the estate does not control, but it acts for one tenant and authenticates
  by device authorization (ADR-0111). A game does neither.
- `tool` is developer-facing, and `web` is served from the cluster.

A game is a program distributed to players' own devices, through a store and never onto the cluster,
written in whichever language its engine uses.

**Two rules already decided bound the name.** C-0007: a repository is `<class-prefix><name>`, and the
name is *"a noun naming the capability, never a technology and never an adjective"*. ADR-0107: where a
class declares more than one stack, a repository *"may carry the stack as a trailing segment of its
name, and does carry it as soon as a second stack implements the same capability"*.

**Three locked ADRs bound what a game can be.** ADR-0005 makes `platform-`, `svc-`, `lib-` and
`tool-` Rust. ADR-0006 supports Windows workstations only through WSL2. ADR-0004 keeps every
repository on GitHub's Free plan.

## Decision

### 1. A new class, `game`, and it is registry data

The entry lands with this ADR — C-0007: the rule lives in the artifact, the list in
`class-registry.yaml`. Prefix `game-`, scope `game`, stacks `[rust, cpp]`, `deployable: false`,
`ci: true`. **A repository in this class is one title's client**, distributed to players' devices and
never scheduled onto the cluster. `deployable: false` is load-bearing: ADR-0020's probes, limits and
chart do not reach it, and neither do C-0007's environment-variable, image and label sections.

### 2. A stack is a language, never an engine

`rust` for an engine written in Rust, such as Bevy; `cpp` for one written in C++, such as Unreal.
`mobile` set the precedent: React Native is `typescript` and Flutter is `dart` — the framework is not
the stack. **A stack is what binds language rules; an engine binds nothing estate-wide.** A third
engine adds its language to `stacks` when its first title chooses it, which is a registry edit.

### 3. A repository is `game-<codename>`

The codename is a noun, stable for the repository's life: never the engine, never an adjective
(C-0007), never the release title. **A title is a brand** (RFC-0059 D6), chosen when it is announced
and free to change. A codename is an identifier, and STD-0022 makes renaming one a complete rename.
`game-lantern` — not `game-bevy-lantern`, not `game-crimson`, not the title.

### 4. The stack suffix appears only for a port

ADR-0107 applies unchanged. Two titles in two engines are two capabilities, so neither carries a
suffix: `game-lantern` on `rust`, `game-orchard` on `cpp`. One title built in two engines is one
capability in two stacks, so both carry it: `game-lantern-rust`, `game-lantern-cpp`. The suffix is the
registry's stack name, so even then no engine is named.

### 5. The engine is named in exactly one place

**The title's own ADR**, written when the title enters `building`: `scope: [game]`,
`applies_to: ["game-<codename>"]`, and `product: [games]` once the line is registered (RFC-0059 D1).
It applies REQ-0012's exit test to the engine's licence, records the engine in `tool-registry.yaml`
(ADR-0066), and pins it exactly (ADR-0053) inside the title's repository. `toolchain-registry.yaml`
stays one compiler per stack: two titles may sit on two engine versions, and a per-stack engine pin
would force them onto one.

So a rule has three homes, by reach: every game — `scope: [game]`; one language — its stack; one
title — `applies_to`.

### 6. Clones live in `~/Developer/paridyum/games/`

RUN-0001's layout gains the line. Its subdirectories mirror the class prefixes, so a repository's name
decides where it is cloned.

### 7. Everything else a game needs uses an existing class

Online services are `svc-` in Rust (ADR-0005), asset-pipeline programs are `tool-`, a title's website
is `web-`, a contract is `spec-`. A `game-` repository holds its title and nothing estate-wide.

### 8. Deferred, each with the event that reopens it

The sanctioned way not to decide (C-0010, ADR-0018):

| Deferred | Why not now | Reopened by |
| --- | --- | --- |
| A dedicated server built from a title's code | a cluster workload in the title's stack; ADR-0005 admits only Rust to `svc-`, and `game-` is not deployable | the first title that needs an authoritative server |
| Code shared between titles | ADR-0005 makes `lib-` Rust, so a shared C++ engine plugin has no class | the second title that would copy code from the first |
| Release governance | `deployable: false` puts it outside ADR-0020, as for `mobile`; REQ-0004 owns it, and RFC-0058's store-account model is the starting point | the first title entering `building` |
| Asset storage | binary assets need Git LFS or a store beside the repository; the Free plan's quotas (ADR-0004) are measured and cited (RFC-0020), never assumed | the first title entering `building` |
| An engine that needs native Windows | ADR-0006 supports Windows workstations only through WSL2. If a chosen engine needs native Windows to edit or package, ADR-0006 is superseded — never excepted — before that title's engine ADR is accepted. A Windows build machine is a separate question from a workstation; ADR-0131 (proposed) already brings a Windows 11 host into the estate | that title's engine ADR |

### 9. The checkpoint guard measures what is committed, not what is on disk

`checkpoint-session.sh` leaves out any staged file over 5 MB, and its message advises Git LFS for a
large file that belongs in the repository. It measured the **working-tree** file. An LFS-tracked asset
is full-size on disk and a small pointer in the index, so the guard left out exactly the files it tells
you to track that way. It now measures the staged blob (`git cat-file -s ":<path>"`), with a
regression test that drives the hook through a clean filter. Without the fix, no game asset would ever
be checkpointed.

## Consequences

**Accepted costs**

1. **A class with no repository.** Like `mobile` and `sdk`, it has no standard, template or CI
   workflow; the first `game-` repository is scaffolded by hand and becomes the de facto template
   without being reviewed as one.
2. **`cpp` is a stack with nothing behind it** — no toolchain pin, no lints, no standard. The first
   C++ title writes them.
3. **Two names per title.** The codename in every identifier and the title in public is a familiar
   source of confusion, and the price of renaming a title without renaming a repository.
4. **No release governance at all** until the first title enters `building` — the state `mobile` has
   been in since it was declared.
5. **The cost of large assets is unknown.** The Free plan's LFS quotas are not yet measured, and
   storage that outgrows them is an unbudgeted cost.

**Gained**

- An engine can be chosen per title, and changed, without renaming anything.
- A port shows in the name exactly when there is one.
- A Rust title inherits the estate's Rust toolchain pin as it stands.
- Naming is fixed before the first repository, while it costs nothing.

**Rejected alternatives**

**Stacks named by engine (`bevy`, `unreal`).** Standards bind a stack, so a Bevy title would need the
Rust rules declared a second time under another name — Bevy is a Rust crate — and a port's suffix
would carry the engine.

**A class per engine (`bevy-`, `unreal-`).** The technology in the prefix: C-0007's *never a
technology*, moved one segment left.

**`mobile-` for mobile games.** Its stacks are TypeScript and Dart, and a title that ships to phones
and PCs would need a class for each.

**`agent-`.** It acts for one tenant and authenticates as a Native client by device code (ADR-0111).
A game does neither.

**The release title as the repository name.** Titles change, and under STD-0022 every rename is
complete — directory, GitHub repository, manifest, CODEOWNERS and every live reference.

**An `engine:` field in `manifest.yaml`.** A second home for a fact the title's ADR already holds
(RFC-0015). Revisit when a generated view (ADR-0040) needs it.

## Related

- RFC-0059 — where the gaming line sits, and when a game may be built
- C-0007 · ADR-0107 — the naming grammar and the stack suffix
- ADR-0111 — the last class added, and the shape this ADR follows
- ADR-0005 · ADR-0006 · ADR-0004 — the locked ADRs that bound a game
- ADR-0020 · REQ-0004 — the release governance a non-deployable class lacks
- ADR-0053 · ADR-0066 · REQ-0012 — an engine is pinned, registered and exitable
- STD-0022 — why a codename, not a title
- RUN-0001 — the workspace layout
- ADR-0043 — the checkpoint hook decision 9 corrects (it superseded ADR-0025)
