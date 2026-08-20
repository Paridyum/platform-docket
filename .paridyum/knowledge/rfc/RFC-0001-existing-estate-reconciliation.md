---
id: RFC-0001
title: Reconciling the existing pd-* estate with the locked platform decisions
type: rfc
status: accepted
created: 2026-08-06
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0009
scope: [org]
applies_to: ["*"]
tags: [migration, naming, rust, legacy, estate]
breaking: true
enforced_by: []
---

# RFC-0001 — Reconciling the existing `pd-*` estate with the locked platform decisions

**Status: RESOLVED 2026-08-06 by ADR-0009.** The estate was transferred to the
`paridyum-archive` organisation, making `Paridyum` genuinely greenfield. Option C was
taken in effect — not by renaming, but by removing. Retained below as the record of
what was found and why it mattered.

**Original status: proposed.** It is written as an
RFC because resolving it requires choices only the owner can make.

## The discovery

C-0002 and ADR-0005 were written on the assumption that Paridyum was greenfield. It is
not. Queried against the GitHub API on 2026-08-06, the `Paridyum` organisation
contains **41 repositories**:

- All prefixed `pd-`. **Zero** match the C-0002 scheme (`svc-`, `lib-`, `tool-`, …).
- Predominantly **Python**, with TypeScript frontends. C-0002's crate and binary rules
  and ADR-0005's Rust stack apply to none of them.
- Largely dormant on `main` — no commits to the default branch of `pd-core`, `pd-cli`
  or `pd-auth` since 2026-05-06, though some branch activity through late July.
- A coherent platform already exists: `pd-auth`, `pd-billing`, `pd-token`,
  `pd-logging`, `pd-integrations`, `pd-ipam`, an AI stack (`pd-ai-core`,
  `pd-ai-runtime`, `pd-ai-knowledge`, `pd-ai-tool-gateway`), frontends, and
  **`pd-cli` — a CLI that already exists in Python.**

The existing convention is not arbitrary. `pd-<domain>[-<subdomain>]` with `-fe` /
`-frontend` suffixes is applied consistently across all 41.

## Why this matters

Two locked artifacts are incomplete rather than wrong:

- **ADR-0005** presents Rust as a greenfield choice. It is actually a **rewrite**
  decision affecting an existing Python platform. The consequences section says
  nothing about migration cost, dual-running, or what happens to 41 repositories —
  which is the most important consequence of the decision.
- **C-0002** mandates a naming scheme that 41 repositories violate. As written, a
  compliance sweep reports the entire organisation as non-conformant.

Neither can be edited in place (C-0004). Both need superseding artifacts once this
RFC resolves.

## Options

### A. Supersede C-0002 to adopt `pd-` as the prefix

Keep the existing convention, express the class in the second segment:
`pd-svc-billing`, `pd-lib-core`, `pd-tool-forge`. Existing repos stay valid;
new ones gain the class marker. No renames, no broken clone URLs.

*Cost:* longer names; the 41 existing repos still lack a class marker.

### B. Scope C-0002 to new repositories only

C-0002 applies to Rust repositories created from the template. `pd-*` is explicitly
recognised as the legacy estate and exempted, with an `applies_to` narrowing.

*Cost:* two conventions coexisting indefinitely.

### C. Rename the estate to match C-0002

Full consistency, immediately.

*Cost:* 41 renames, every clone URL, CI reference, and cross-repo import broken at
once, for an estate that is currently dormant. Hard to justify unless the estate is
being retired anyway.

**Recommendation: B**, narrowing C-0002 to Rust repositories and naming the legacy
estate explicitly. It is the only option with no migration cost, and if the Rust
rewrite does replace these services, the legacy names retire on their own.

## The larger question this RFC cannot answer

**Is the Rust work a rewrite that replaces the Python platform, a new platform layer
beside it, or a greenfield product that happens to share an organisation?**

Each answer implies a different ADR-0005 supersession, a different roadmap, and a
different answer to whether `tool-forge` duplicates `pd-cli`. Until it is answered:

- ADR-0005 stands but is knowingly incomplete on migration.
- C-0002 should not be used to audit the existing estate; it would report 41 failures
  that are not failures.
- `tool-forge` should not be built without first establishing what `pd-cli` does, so
  the new tool either supersedes it deliberately or stays out of its way.

## Next step

Answer the question above, then supersede C-0002 and ADR-0005 with artifacts that
record the estate honestly.
