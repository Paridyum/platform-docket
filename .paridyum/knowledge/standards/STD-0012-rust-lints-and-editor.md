---
id: STD-0012
title: Lint policy and editor configuration for Rust repositories
type: standard
status: accepted
created: 2026-08-17
locked_at: null
supersedes: [STD-0003]
superseded_by: null
scope: [platform, service, library, tool]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [clippy, rustfmt, lints, vscode, rust-analyzer, editor]
breaking: true
verified_by:
  - TST-0047
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
channel = "<the rust pin from toolchain-registry.yaml>"   # exact, never "stable"
components = ["rustfmt", "clippy", "rust-src", "llvm-tools-preview"]
profile = "minimal"
```

Pinned to an exact version. `channel = "stable"` means the compiler changes under you
and CI disagrees with your machine the week a release lands. `rust-src` is required by
rust-analyzer; `llvm-tools-preview` by `cargo-llvm-cov`.

**The version is deliberately absent from this file, and that is the only substantive change
from STD-0003.** [`toolchain-registry.yaml`](../toolchain-registry.yaml) holds the pin per
stack and is the authority; `templates/rust-service/rust-toolchain.toml` carries the same
value for scaffolding to copy. Read the registry, never this example.

STD-0003 wrote the number here. It was correct on 2026-08-06 and false within a year — a
reader copying the block got a compiler `cargo install sqlx-cli` refused to build against, and
because STD-0003 was `locked` the number could not be corrected: ADR-0031 errata may not change
what an artifact decides, so the erratum appended to it labels the trap and cannot remove it.
**A value that changes on a cadence we do not control cannot live in an artifact whose only
legal correction is heavier than the correction.** ADR-0048 generalises that.

What has *not* changed is the rule: exact, never a channel, with those four components. That
changes on nobody's schedule but ours, so it belongs here rather than in the registry — putting
it there would be ADR-0048 decision 5's opposite error.

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

