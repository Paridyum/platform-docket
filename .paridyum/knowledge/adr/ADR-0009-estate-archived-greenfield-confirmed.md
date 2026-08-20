---
id: ADR-0009
title: The pd-* estate is archived; Paridyum restarts greenfield in Rust
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [estate, migration, archive, greenfield, rust]
breaking: true
enforced_by:
  - check: org.no_legacy_prefixed_repos
    severity: error
    args: { forbidden_prefix: "pd-", except: ["pd-coming-soon"] }
    rationale: A pd-* repo reappearing in Paridyum means the archive boundary leaked.
  - check: repo.name_matches_scheme
    severity: error
    rationale: C-0002 now applies to the whole org with no legacy exemption.
---

# ADR-0009 — The `pd-*` estate is archived; Paridyum restarts greenfield in Rust

## Status

**LOCKED** as of 2026-08-06. Resolves RFC-0001.

## Context

RFC-0001 recorded that `Paridyum` was not greenfield: 41 private repositories, all
`pd-*` prefixed, predominantly Python with TypeScript frontends, none matching C-0002.
It asked a question this ADR now answers — *is the Rust work replacing the Python
platform, sitting beside it, or separate?*

The answer was given by action rather than argument. On 2026-08-06, 40 of the 41
repositories were transferred to a new **`paridyum-archive`** organisation.

## Decision

**`Paridyum` is greenfield. The Python estate is archived, not migrated.**

Verified state after the transfer:

| | |
| - | - |
| `Paridyum` | `pd-coming-soon` only (public) |
| `paridyum-archive` | 40 repositories, all private |

`pd-coming-soon` remains deliberately — it is the public landing page and has no
bearing on the platform architecture.

### What this settles

RFC-0001 offered three options for reconciling the naming schemes. **Option C was
taken in effect — not by renaming, but by removing.** The consequences follow:

- **C-0002 applies to the whole organisation with no legacy exemption.** The proposed
  `applies_to` narrowing is unnecessary. A compliance sweep should now find zero
  violations, and any `pd-*` repository appearing in `Paridyum` means the archive
  boundary leaked.
- **ADR-0005 stands as written.** Rust was recorded as a greenfield choice; that is
  now accurate rather than incomplete. No supersession is needed.
- **`tool-forge` does not duplicate `pd-cli`.** `pd-cli` is archived. The concern
  RFC-0001 raised about building a second CLI in parallel with an existing one is
  resolved by the first one being retired.

### Operational facts recorded because they are easy to lose

These came out of the transfer and are not recoverable from git history:

- **GitHub Actions secrets did not transfer.** `pd-docs`, `pd-platform-infra`,
  `pd-regional`, `Paridyum`, and `pd-internal` lost their secrets — GitHub drops them
  on org transfer and they are unreadable via the API. If any archived repository's CI
  is ever needed, the secrets must be re-added by hand in `paridyum-archive`.
- **Archived repositories were not locked read-only.** They remain live and writable.
  This is a hazard: a commit to an archived repository looks like active work and will
  be missed. Locking them is recommended and not yet done.
- **No local clones pointed at `Paridyum/*`.** Seven of the eight directories under
  `~/Developer/paridyum` were empty scaffolding, and `knowledge/` had no remote at all.
  So no remote needed re-pointing, and there is no stale-redirect hazard when
  repository names are recreated in `Paridyum`.

## Consequences

**Accepted costs**

- Whatever institutional knowledge lives only in those 41 repositories is now one
  organisation further away. Nothing was deleted, but "archived" reliably becomes
  "never opened again".
- The Python platform's working behaviour is no longer a reference implementation
  under active maintenance. Rebuilding a capability in Rust means re-deriving
  requirements the Python version had already discovered.
- Any dependency an archived repository had on another is now an archive-internal
  concern nobody is maintaining.

**Gained**

- Every locked decision in this knowledge base applies cleanly to every repository in
  the organisation. No exemptions, no dual conventions, no "except the legacy ones".
- `tool-forge` can define the standard for every repository that will exist, rather
  than accommodating forty that already do.
- The naming scheme, the testing standard, and the CI architecture bind from the first
  repository rather than from some future migration.

**Follow-up not done here**

Locking the archived repositories read-only. Left as an operational task rather than
folded into this ADR, because it is reversible and independent of the decision.
