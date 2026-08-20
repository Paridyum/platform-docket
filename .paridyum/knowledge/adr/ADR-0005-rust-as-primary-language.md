---
id: ADR-0005
title: Rust is the primary implementation language
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [platform, service, library, tool]
applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]
tags: [language, rust, stack]
breaking: true
enforced_by:
  - check: cargo.toolchain_file_present
    severity: error
    rationale: An unpinned toolchain means the local build and CI can disagree.
  - check: cargo.edition_is_current
    severity: error
  - check: cargo.workspace_lints_inherited
    severity: error
  - check: rust.no_unwrap_or_expect_outside_tests
    severity: error
  - check: rust.no_panic_in_libraries
    severity: error
    args: { applies_to: ["lib-*"] }
  - check: rust.anyhow_only_in_binaries
    severity: error
  - check: cargo.no_diesel_dependency
    severity: error
    rationale: SQLx is the locked data access layer; two ORMs is two sets of habits.
---

# ADR-0005 — Rust is the primary implementation language

## Status

**LOCKED** as of 2026-08-06.

## Context

Paridyum needs one default language so that shared libraries are actually shareable,
one CI shape applies everywhere, and an agent moving between repositories does not have
to re-learn idioms.

Rust was chosen before this knowledge base existed; this ADR records the reasoning so
it survives, and pins the stack decisions that ride along with it.

## Decision

**Rust** is the implementation language for all platform services, business services,
shared libraries and tools.

The following are locked alongside it, because "use Rust" without them still leaves
every repository free to diverge:

| Concern        | Locked choice        | Rejected                            |
| -------------- | -------------------- | ----------------------------------- |
| Async runtime  | `tokio`              | `async-std`, `smol`                 |
| HTTP server    | `axum` + `tower`     | `actix-web`, `rocket`, `warp`       |
| Database       | `sqlx` (Postgres)    | `diesel`, `sea-orm`                 |
| Serialisation  | `serde`              | —                                   |
| Observability  | `tracing` + OpenTelemetry | `log`, `env_logger`            |
| Error handling | `thiserror` in libraries, `anyhow` in binaries | `Box<dyn Error>` |
| Toolchain      | pinned `rust-toolchain.toml`, stable channel | floating stable |

**Error handling is the rule most often violated by generated code**, so it is stated
precisely: a `lib-*` crate returns typed errors via `thiserror` and never panics.
`anyhow` is permitted only in binary crates, at the top level, where there is nobody
left to handle the error. `unwrap()` and `expect()` are permitted only in tests.

SQLx over Diesel because compile-time-checked raw SQL keeps the query surface visible
and reviewable, which matters more than ORM ergonomics when agents write the queries.

## Consequences

**Accepted costs**

- Slower compile times than Go, and a steeper ramp for any future hire.
- Some ecosystems (data science, some cloud SDKs) are thinner than in Python or Go.
  Where Rust is genuinely wrong for a job, that gets its own ADR rather than a silent
  exception — this ADR says "primary", not "only".
- Pinning `axum` and `sqlx` means major-version upgrades are org-wide coordinated events.

**Gained**

- One idiom set, so `lib-core` is genuinely reusable and CI is one template.
- Errors and panics are structurally constrained rather than left to reviewer taste.
- No per-repository stack debate, and no agent inventing an answer to a settled question.
