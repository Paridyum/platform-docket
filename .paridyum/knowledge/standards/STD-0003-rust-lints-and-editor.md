---
id: STD-0003
title: Lint policy and editor configuration for Rust repositories
type: standard
status: superseded
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: STD-0012
scope: [platform, service, library, tool]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [clippy, rustfmt, lints, vscode, rust-analyzer, editor]
breaking: true
enforced_by:
  - check: cargo.workspace_lints_defined
    severity: error
  - check: cargo.members_inherit_workspace_lints
    severity: error
    rationale: A member without `lints.workspace = true` silently opts out of every lint.
  - check: clippy.clean_with_all_targets_all_features
    severity: error
    args: { flags: ["--all-targets", "--all-features", "--", "-D", "warnings"] }
  - check: rustfmt.check_passes
    severity: error
  - check: cargo.toolchain_file_pinned
    severity: error
  - check: editor.vscode_settings_committed
    severity: warn
    rationale: Committed editor settings make the local loop agree with CI; a mismatch is discovered at PR time.
---

# STD-0003 — Lints and editor configuration

Every lint here is enforced identically in the editor, in pre-commit, and in CI. When
those three disagree, engineers learn to ignore the two that are wrong, and the
enforcement decays. Agreement is the point.

## Lints are declared once, at the workspace root

`[workspace.lints]` in the root `Cargo.toml`; every member inherits. A member that
does not inherit has silently opted out of the entire policy, which is why it is
checked.

```toml
# Cargo.toml — workspace root
[workspace.lints.rust]
unsafe_code           = "forbid"   # forbid, not deny: cannot be re-allowed locally
missing_docs          = "warn"     # "deny" in lib-* (STD-0002)
unreachable_pub       = "warn"
rust_2018_idioms      = { level = "warn", priority = -1 }
unused_qualifications = "warn"

[workspace.lints.clippy]
all      = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# ADR-0005: libraries return typed errors and never panic.
unwrap_used            = "deny"
expect_used            = "deny"
panic                  = "deny"
todo                   = "deny"
unimplemented          = "deny"
indexing_slicing       = "deny"    # a[i] panics; use .get()
integer_division       = "deny"
float_cmp              = "deny"

# Correctness traps that compile fine and fail in production.
await_holding_lock     = "deny"    # deadlocks under load, never in tests
mem_forget             = "deny"
lossy_float_literal    = "deny"

# pedantic lints that are noise rather than signal
module_name_repetitions = "allow"
must_use_candidate      = "allow"
missing_errors_doc      = "warn"   # STD-0002 wants these, but as warnings
```

```toml
# every member Cargo.toml
[lints]
workspace = true
```

### `unsafe_code = "forbid"`

`forbid`, not `deny`, so it cannot be re-enabled with a local `#[allow]`. A crate that
genuinely needs `unsafe` gets an ADR and an explicit exception in its own manifest —
which makes the exception visible in review rather than buried in an attribute.

### `unwrap_used` in tests

Denied in `src/`, allowed in tests — `.unwrap()` in a test is a legitimate assertion.
Achieved with `#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]` at
the crate root, not by weakening the workspace lint.

## `clippy.toml`

```toml
avoid-breaking-exported-api = false   # pre-1.0; correctness beats stability
disallowed-methods = [
  { path = "std::env::set_var",       reason = "unsound with threads; load config at startup" },
  { path = "std::time::SystemTime::now", reason = "inject a clock — STD-0001 requires deterministic tests" },
]
disallowed-types = [
  { path = "std::collections::HashMap", reason = "use ahash::AHashMap, or BTreeMap where ordering matters" },
]
too-many-arguments-threshold = 8
cognitive-complexity-threshold = 20
```

`disallowed-methods` is the most valuable and least-used clippy feature: it turns a
convention that would otherwise live in a code-review checklist into a compile error.
`SystemTime::now` is listed because STD-0001 requires deterministic tests, and an
un-injected clock is the most common reason they are not.

## `rustfmt.toml`

```toml
edition = "2024"
max_width = 100
use_field_init_shorthand = true
use_try_shorthand = true
newline_style = "Unix"            # ADR-0006
```

No nightly-only options. A format config that requires nightly means `cargo fmt`
silently does nothing different on stable, and the repository drifts.

## `rust-toolchain.toml`

```toml
[toolchain]
channel = "1.90.0"                # pinned, never "stable"
components = ["rustfmt", "clippy", "rust-src", "llvm-tools-preview"]
profile = "minimal"
```

Pinned to an exact version. `channel = "stable"` means the compiler changes under you
and CI disagrees with your machine the week a release lands. `rust-src` is required by
rust-analyzer; `llvm-tools-preview` by `cargo-llvm-cov`.

## VS Code

Committed at `.vscode/settings.json` so every clone gets a loop that agrees with CI.

```jsonc
{
  // Clippy, not check — otherwise the editor is green and CI is red.
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-targets", "--all-features"],

  "editor.formatOnSave": true,
  "[rust]": { "editor.defaultFormatter": "rust-lang.rust-analyzer" },

  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.imports.granularity.group": "module",
  "rust-analyzer.inlayHints.chainingHints.enable": true,

  // Never index build output — on a large workspace this alone costs seconds per search.
  "files.watcherExclude":  { "**/target/**": true },
  "search.exclude":        { "**/target/**": true, "**/*.lock": true },

  "files.insertFinalNewline": true,
  "files.trimTrailingWhitespace": true,
  "files.eol": "\n"
}
```

`.vscode/extensions.json` recommends `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`,
`vadimcn.vscode-lldb`, `fill-labs.dependi`, and `usernamehw.errorlens`. Recommendations,
not requirements — the editor is a personal choice; agreement with CI is not.

## Why editor config is committed

The counter-argument is that editor settings are personal. They are, except where they
determine whether your local result matches CI. `rust-analyzer.check.command` is the
clearest case: the default (`cargo check`) makes the editor green while CI fails on
clippy, and the engineer discovers it at PR time. That is not a preference; it is a
broken feedback loop.

## Errata

### 2026-08-16 — the `rust-toolchain.toml` example pins a compiler nobody uses

**What the text says.** The `rust-toolchain.toml` section shows
`channel = "1.90.0"  # pinned, never "stable"`.

**What is wrong.** No repository in the organisation pins 1.90.0. Rust 1.90.0 was released
2025-09-14; by August 2026 a reader copying that block verbatim received a compiler eleven
months old, which `cargo install sqlx-cli` refuses to build against — it requires 1.94 or
newer. The pin in `tool-forge` and in `templates/rust-service/rust-toolchain.toml` is
**1.97.1** as of this date, and it will move again roughly every six weeks.

**What was intended.** The *shape*, not the number: an exact version, with the four components
listed, and never a channel. That rule is unchanged and is what this standard decides. The
authoritative current pin is **`templates/rust-service/rust-toolchain.toml`**, which every
repository `forge` scaffolds inherits — read that file, not the example above.

**This erratum does not fix the defect, and cannot.** Per ADR-0031 decision 5 an erratum may
never change what the artifact decides, and everything above the `## Errata` heading stays
byte-identical — so the example still reads 1.90.0 and a reader who skims will still copy it.
Only a reader who reaches the end of the file is warned. The structural fix is C-0007's rule,
*the grammar is constitutional and the list is data*: the version belongs in the template and
the standard should name it rather than embed it. That requires supersession, not an erratum,
and this entry exists so the trap is labelled until then.

**Why the number was here at all.** It was correct when written on 2026-08-06. That is the
failure mode, not an oversight — a hand-maintained version inside a `locked` artifact can only
go stale, because the one legal way to correct it is heavier than the correction. `README.md`
records the same lesson three times over for counts; this is it one layer down, in a file that
cannot be edited to say so.
