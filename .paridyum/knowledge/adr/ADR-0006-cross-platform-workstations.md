---
id: ADR-0006
title: Workstations run macOS or Linux natively; Windows is supported through WSL2
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [workstation, macos, linux, windows, wsl2, onboarding]
breaking: false
enforced_by:
  - check: repo.scripts_are_lf_only
    severity: error
    rationale: A CRLF shell script fails inside WSL and in containers with an error that never mentions line endings.
  - check: repo.gitattributes_normalizes_eol
    severity: error
  - check: workstation.doctor_passes
    severity: warn
    rationale: Enforced on the machine by bootstrap --check / forge doctor, not by repository CI.
---

# ADR-0006 — Workstations run macOS or Linux natively; Windows is supported through WSL2

## Status

**LOCKED** as of 2026-08-06.

## Context

RUN-0001 originally provisioned macOS only, matching the single workstation that
existed. Supporting contributors — or future-you on different hardware — requires
deciding what "a Paridyum workstation" means on other operating systems, before
divergent setups accumulate.

Three environments matter: macOS, Linux, and Windows. The first two are close enough
that one script can serve both. Windows is genuinely different, and the choice is
between supporting native Windows or routing through WSL2.

Native Windows can build Rust. What it does poorly is everything around it: the
`sqlx` and `cargo-chef` toolchain assumes a Unix shell, Docker on native Windows
containers is a different runtime from the Linux containers we deploy (ADR-0003),
and every script in the platform would need a PowerShell twin maintained in
lockstep. That last cost compounds forever.

## Decision

**macOS and Linux are supported natively. Windows is supported through WSL2 only.**

One bash script, `scripts/bootstrap.sh`, provisions all three, because WSL2 presents
as Linux. Platform differences are isolated in `scripts/lib/platform.sh`, which
detects the OS and abstracts the package manager — `brew`, `apt`, `dnf`, `pacman`.

`scripts/bootstrap.ps1` exists but is deliberately thin. It provisions the Windows
side only — enabling WSL2, installing a distribution, checking Docker Desktop's WSL
integration, fixing `core.autocrlf` — and then hands off. It installs no Rust, no
`gh`, no `kubectl`. Those live inside WSL2 where the code is actually built.

Native Windows is **not** a supported target. `bootstrap.sh` detects MinGW/MSYS/Cygwin
and refuses with a pointer to `bootstrap.ps1` rather than half-working.

### Consequences that are easy to get wrong

Two rules that produce confusing failures when broken, so both are checked:

- **Repositories live on the Linux filesystem** (`~/Developer`), never on `/mnt/c`.
  Cross-filesystem I/O through the Windows mount is roughly an order of magnitude
  slower, and Linux file permissions are not preserved, so executable bits on
  scripts silently vanish.
- **Line endings are LF everywhere.** `core.autocrlf false` on Windows, plus a
  committed `.gitattributes`. A shell script checked out with CRLF fails inside WSL
  and in Docker with `bad interpreter: no such file or directory` — an error that
  never mentions line endings and costs an hour the first time.

## Consequences

**Accepted costs**

- Windows contributors carry the WSL2 setup step and a small ongoing overhead
  (Docker Desktop integration, occasional clock skew after sleep).
- `bootstrap.ps1` is a second script to maintain, though a small and slow-changing one.
- Linux package coverage is uneven: `kubectl` and `terraform` are absent from default
  Debian and Fedora repositories and need vendor repositories. The script detects
  this and prints the specific instructions rather than failing opaquely.

**Gained**

- One provisioning path, one shell, one set of scripts across every environment.
- The development environment matches the deployment target — Linux containers on
  Kubernetes (ADR-0003) — so "works on my machine" failures are structurally rarer.
- Onboarding a Windows contributor does not fork the platform.

**Rejected alternatives**

- *Native Windows support* — doubles every script in the platform permanently, and
  still leaves a Docker runtime that differs from production.
- *Dev containers / Codespaces as the only path* — attractive and may come later, but
  it presumes a working Docker and network on day one and makes offline work
  impossible. Worth revisiting as an addition, not a replacement.
