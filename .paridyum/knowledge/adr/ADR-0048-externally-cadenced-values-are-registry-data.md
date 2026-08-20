---
id: ADR-0048
title: A value that changes on a cadence we do not control is registry data
type: adr
status: accepted
created: 2026-08-17
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [registry, versions, toolchain, immutability, drift, locked-artifacts]
breaking: false
enforced_by: []
---

# ADR-0048 — A value that changes on a cadence we do not control is registry data

## Status

**ACCEPTED**, 2026-08-17. Not `locked`: ADR-0002's bar is a check that rejects the violation, and
the check this wants is described below but not built (open question 1).

## Context

**This rule has been applied four times and written down zero times.**

| Instance | Where | What was moved out |
| --- | --- | --- |
| Repository classes | C-0005 → C-0007 | the class list, out of locked constitutional prose |
| Artifact types | ADR-0022 | the type list, hardcoded at fifteen sites, fourteen failing silently |
| Test types | ADR-0029 | the test-type list |
| Counts | ADR-0035 | every count, declared a snapshot with the registry authoritative |

C-0007 states it as a slogan — **"the grammar is constitutional. The list is data."** A slogan is
retrievable by someone who already knows to look for it. Each of the four above was discovered by
being bitten, and the fifth has just happened.

**The fifth: the Rust toolchain pin.** STD-0003 is `locked` and its `rust-toolchain.toml` example
reads `channel = "1.90.0"`. It was correct when written on 2026-08-06. By August 2026 a reader
copying it received a compiler eleven months old, which `cargo install sqlx-cli` refuses to build
against — which is how it was found, rather than by anyone reading the standard.

Correcting it was not possible. ADR-0031 errata may not change what an artifact decides, and the
body stays byte-identical, so the erratum appended on 2026-08-16 **labels the trap and cannot
remove it**: the example still reads 1.90.0 and only a reader who reaches the end of the file is
warned.

**That is the whole argument in one instance.** A value with its own release cadence, placed inside
an artifact whose only legal correction is heavier than the correction, can do nothing but rot.

## Decision

**1. A value that changes on a cadence Paridyum does not control does not live inside an artifact
that requires supersession to change.** It lives in a registry, and the artifact names the registry.

**2. The test is one question: _does this change on a schedule I do not set?_** If yes, it is
registry data. Compiler and runtime versions, framework and dependency versions, external standards
and their revisions, published rate tables, counts of things the gate computes. If no — a rule, a
boundary, a prohibition, a definition — it belongs in the artifact, and putting *it* in a registry
would be the opposite error.

**3. An artifact may show such a value only as an illustration that names its source.** Not
`channel = "1.90.0"` alone, but the shape plus *"the current pin is in `toolchain-registry.yaml`."*
An example with no pointer becomes authoritative by default, because a reader copying a block does
not know it was meant to be illustrative.

**4. `toolchain-registry.yaml` is created by this decision**, holding the pinned toolchain per
stack. Per stack rather than per template, because there will be more than one Rust template — a
service, a tool, a library — and a version copied into each of them drifts between them, which is
this same defect one level down.

**5. This is not licence to move rules into registries.** ADR-0022's registries carry *lists that
get extended*; this adds *values that get superseded by their upstream*. Both are data. A judgement,
a boundary or a prohibition is neither, and C-0006 is the standing example: its anti-goals are not
a list to be extended, they are six decisions.

## Consequences

**Gained**

- A toolchain upgrade becomes a one-line registry edit, with no locked artifact touched. That was
  the request this decision answers.
- The rule is retrievable. CLAUDE.md's *never infer, retrieve* only works if the fifth person to
  hit this can find the first four.
- The next stack — TypeScript and Node under RFC-0003 and RFC-0008 — inherits the mechanism instead
  of repeating the discovery.

**Accepted costs**

1. **A registry is a second place to look.** Every registry added is another file a reader must
   know exists, and `knowledge/` already carries several. Mitigated only by decision 3: the
   artifact must point at it.
2. **This does not retroactively fix locked artifacts.** STD-0003 keeps its stale example until it
   is superseded. The rule prevents the next one; it cannot repair the last one.
3. **The boundary in decision 2 is a judgement.** *"A schedule I do not set"* is clear for a
   compiler release and arguable for, say, a coverage threshold — which nobody upstream sets, but
   which is also revised more often than a standard should be. Wrong calls are expected at the
   edges and are cheap to correct in either direction.
4. **A registry with one entry looks like ceremony.** `toolchain-registry.yaml` will hold a single
   stack until TypeScript lands. RFC-0013 D5's argument — *"a registry with zero entries governs
   nothing"* — is about zero, not one, and one entry that is already load-bearing is not the same
   thing as an empty one.

## Alternatives considered

**Put the version in `templates/rust-service/rust-toolchain.toml` and point the standard there.**
Cheaper, and no new file. Rejected on decision 4's reasoning: templates multiply per class, and the
version would then exist in each of them with nothing keeping them equal. It also gives the value
no schema and no place for the metadata a pin wants — when it was set, and why.

**Erratum on STD-0003 and move on.** Done on 2026-08-16 and demonstrably insufficient: the body is
immutable, so the stale example survives and only a reader who reaches the end of the file learns
it is stale. It buys time; the trap remains.

**Never lock artifacts that contain versions.** Inverts the problem rather than solving it — the
rule *"pinned exactly, never a channel"* deserves to be locked, and it is only the number beside it
that does not.

**Do nothing; bump the pin when it breaks.** The honest status quo, and it worked: `sqlx-cli`
refusing to install is what surfaced it. Rejected because the detector was a dependency's minimum
version, which is luck rather than a mechanism, and because the same failure will recur every six
weeks in a place where nothing is watching.

## Open questions

1. **Can the rule be checked?** A plausible check reads *"fails when a `locked` artifact contains a
   version-shaped literal that no registry declares"* — a semver regex over locked bodies, with the
   registry as the allowed set. That is mechanically expressible, which would let this ADR lock.
   Not declared here, because `check-registry.yaml` cannot express a check owed by a component
   other than `tool-forge` without counting it as implemented (TASKS T-2.7).
2. **Does a coverage threshold fall inside decision 2 or outside?** Nobody upstream sets it, so by
   the letter it is outside; it is revised more often than a standard should be, so by the spirit
   it may be inside. Left open deliberately rather than answered by guess.
3. **What advances the pin, and when?** This decision says *where the number lives*, not *when it
   changes*. TASKS T-2.12 carries that, and the two together are what make the next upgrade
   mechanical.

## Related

- C-0007, C-0005 — *the grammar is constitutional, the list is data*; the slogan this generalises
- ADR-0022, ADR-0029 — the same move for artifact types and test types
- ADR-0035 — the same move for counts; this extends it from counts to versions
- ADR-0031 — the errata mechanism whose limits are this decision's worked example
- ADR-0002 — why this is `accepted` rather than `locked`
- STD-0003 — the locked standard that could not be corrected
- RFC-0003, RFC-0008 — the next stacks, which inherit `toolchain-registry.yaml`
