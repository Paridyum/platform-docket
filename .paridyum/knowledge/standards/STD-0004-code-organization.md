---
id: STD-0004
title: Code organization by responsibility, never by size
type: standard
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [platform, service, library, tool]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [organization, modules, architecture, maintainability, refactoring]
breaking: true
enforced_by:
  - check: module.has_responsibility_doc
    severity: error
    rationale: A module nobody can state the responsibility of does not have one. This is the check that replaces a line limit.
  - check: module.responsibility_is_singular
    severity: warn
    rationale: A responsibility sentence needing "and" describes two modules. Advisory, because sometimes the conjunction is incidental.
  - check: module.no_grabbag_names
    severity: error
    args: { forbidden: ["utils", "util", "helpers", "helper", "common", "misc", "shared", "core_utils", "stuff"] }
  - check: module.no_dependency_cycles
    severity: error
    rationale: Cargo prevents crate-level cycles but permits module-level ones; those are the ones that actually accumulate.
  - check: module.no_orphan_source_files
    severity: error
    rationale: An unlinked .rs file compiles nowhere and is invisible to review.
  - check: clippy.cognitive_complexity_within_threshold
    severity: error
  - check: crate.public_api_is_documented
    severity: error
    args: { applies_to: ["lib-*"] }
---

# STD-0004 — Code organization

## The principle

**Structure follows responsibility, never size.**

A file is not too long because it has many lines. It is too long when it does more
than one thing, and the line count is a symptom that sometimes appears and sometimes
does not. A 900-line module implementing one state machine is well organized. A
120-line module holding four unrelated helpers is not.

This standard therefore contains **no line limit**, and adding one later would be a
regression. Line thresholds produce two failures: they split cohesive code at an
arbitrary boundary, creating modules that must be read together but now live apart;
and they let genuinely tangled small files pass because they happen to be short.

## The rule that replaces a line limit

**Every module states its responsibility in a `//!` doc comment, in one sentence.**

```rust
//! Settlement of issued invoices against payment intents.
```

This is mechanically checkable — the comment either exists or it does not — and it
does the work a line limit was reaching for, because the difficulty is in the writing.
A module doing one thing takes ten seconds to describe. A module doing several
produces a sentence like:

```rust
//! Invoice settlement, refund processing, and PDF rendering.
```

**If the sentence needs "and", you have found the split.** That is not a formatting
nit; the conjunction is the module boundary announcing itself. The check flags it as a
warning rather than an error, because occasionally the conjunction is incidental
("parsing and validation" of a single format may genuinely be one concern) — and a
human should decide which case this is.

## Banned module names

`utils`, `helpers`, `common`, `misc`, `shared`, `stuff` — rejected outright.

These are not names. They are the absence of a name, and they are how a monolith
begins: the first thing goes in because it fits nowhere else, and within a year it is
the largest module in the crate and depends on everything. A function that seems to
belong in `utils` belongs in the module that owns its domain, or in a new module
whose responsibility you can state.

The same reasoning applies to crates: no `lib-common`, no `lib-utils`.

## When to refactor

Refactor when the code becomes hard to **understand**, **navigate**, or **change** —
not when it crosses a threshold. Concrete signals:

- The responsibility sentence needs "and".
- A change to one concern forces edits scattered across the file.
- Two engineers (or two agents) working on unrelated features keep colliding here.
- The module imports from every layer of the crate.
- You scroll to find things you know are there.
- Tests for the module have to construct unrelated state to test one behaviour.

That last one is the most reliable and the most ignored: **awkward tests are a design
signal, not a testing problem.** If a test needs elaborate setup for a simple
assertion, the unit under test has too many collaborators.

## Structure

```
crate/
├── src/
│   ├── lib.rs          //! crate responsibility; re-exports; nothing else
│   ├── domain/         pure types and rules — no I/O, no framework
│   ├── application/    use cases orchestrating the domain
│   ├── adapters/       HTTP, database, queue — the outside world
│   └── config.rs
```

Dependencies point **inward only**: adapters → application → domain. The domain knows
nothing about axum, sqlx, or HTTP. This is what makes the domain testable without a
container and what makes replacing an adapter a local change.

`lib.rs` contains the crate-level `//!` doc and re-exports. Logic in `lib.rs` is a
smell — it is the one file every reader opens, and it should orient them, not detain
them.

## Module cycles

Cargo prevents circular dependencies **between crates**. It does not prevent them
between **modules inside a crate**, and those accumulate silently. A cycle means the
two modules are one module with a boundary drawn in the wrong place — either merge
them or extract the shared concept into a third.

Detected with `cargo modules dependencies --no-externs`, which also reports orphan
`.rs` files that are not linked into the module tree. An orphan compiles nowhere, is
invisible to review, and is usually a half-finished idea nobody deleted.

## When a module becomes a crate

Split a crate when a module has **no inward dependencies from the rest of the crate**
and is used by more than one consumer, or when its compile time is material and it
changes rarely. Many modules in one crate degrade build time, because the crate is
the unit of compilation and any change recompiles all of it.

Do not split for tidiness. A new crate costs a `Cargo.toml`, a README, a version, a
changelog, and a release (STD-0002, STD-0005). That price is worth paying for reuse or
build time, and not for aesthetics.

## What is deliberately not enforced

**Line counts. Function counts. File counts.** None of them are checked, and none
should be added. They measure the wrong thing, and the moment one exists people
optimise for it — splitting a coherent module into three at 1,999 lines is worse code
that passes the check.

The enforced signals above are proxies for cohesion; the responsibility sentence is
the primary one. Everything else is a prompt for a human to look.
