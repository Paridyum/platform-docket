---
id: ADR-0066
title: A tool the estate uses is registry data, and the registry holds no version
type: adr
status: accepted
created: 2026-09-05
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, web, console, tool, library, service, spec]
applies_to: ["*"]
tags: [tooling, registry, supply-chain, adr-0048, adr-0022, rfc-0015, req-0012, req-0014, reproducibility]
breaking: false
enforced_by:
  - check: knowledge.tool_registry_valid
    severity: error
    rationale: >
      Every other registry in this repository has a schema and a check. `toolchain-registry.yaml`
      has neither, and its own header records that as TASKS T-2.7 — a registry nothing validates
      is a registry that drifts into free text. Declaring this one without the check would
      reproduce the 2026-08-06 audit finding in the file created to answer it.
  - check: knowledge.tool_registry_holds_no_version
    severity: error
    rationale: >
      The single rule that keeps this registry from becoming a second copy of
      `toolchain-registry.yaml`. An entry carrying both `pin_home` and a version is two homes for
      one fact, and RFC-0015's whole argument is that the second one drifts silently. This is the
      invariant, not a style preference.
  - check: knowledge.declared_tool_install_is_pinned
    severity: error
    rationale: >
      An install source with a mutable ref — a branch, `latest`, or a bare clone — is an
      unpinned dependency wearing a declaration, and it is exactly what the entry was written to
      prevent. A registry recording an unpinned install is worse than no registry, because it
      reports the dependency as governed.
---

# ADR-0066 — A tool the estate uses is registry data

## Status

**ACCEPTED** as of 2026-09-05. Designed in RFC-0040 finding 5.

## Context

The request was *"maintain a tool list in governance that these are the stuff we used."* There is
no such list, and finding out where one should go turned out to be the decision.

`toolchain-registry.yaml` was the obvious home and is not it. Its own header states the constraint:

> **THE PINNED COMPILER OR RUNTIME FOR EACH STACK. One entry per stack, and the entry is the
> authority.**

One entry per stack. A tool is not a stack, and that file is already carrying the strain of the
gap. `talosctl`, `helm`, `kubectl`, `cilium`, `sops`, `age` and `ansible-lint` sit under a
`tooling:` key inside whichever stack happened to need them first — `sops` and `age` under
`ansible` although ADR-0057 makes them estate-wide, `cilium` under `kubernetes` although it is a
Helm chart version rather than a tool at all. The file argues the point against itself:

> "`talosctl` is listed as TOOLING rather than given its own stack. A stack is a thing you write
> code in, and nobody writes Talos."

That is correct and it solves the wrong half. It explains why `talosctl` is not a stack without
providing anywhere else for it to be, so it went into the only file that existed.

**Everything with no stack to hide under has no entry at all.** `gh`, `ripgrep`, `jq`, `python3`
and the schema validator the knowledge gate depends on, `mermaid`, `shellcheck`, the agent harness
plugins — the estate depends on every one of them and declares none. `RUN-0001` installs several
by hand, which is a reproducibility debt by the operator's own standing rule.

This is the **sixth** instance of one pattern. ADR-0014 moved check ids out of code. ADR-0015
replaced a scope enumeration with a rule. C-0005 moved repository classes out of locked prose.
ADR-0022 moved artifact types out of fifteen hardcoded sites. ADR-0048 moved version pins out of a
locked standard. Each time the defect was the same: **a list that must be extended, hardcoded in a
place extending it is expensive.** Here it is hardcoded in no place at all, which is the version
of the defect that produces nothing to audit.

Its consequence is already recorded, three times, in `toolchain-registry.yaml`'s own notes: with
no entry to derive from, the first repository pins whatever the workstation happened to have and
the next copies it. That mechanism put an **end-of-life Node runtime into three repositories**.
The tool half of the estate has been running that same mechanism the whole time with nothing to
detect it.

Adopting archify (ADR-0065) forced the question rather than raising it. Its pin must be a commit
sha, and there was nowhere to write one down.

## Decision

**A tool the estate uses is declared in `tool-registry.yaml`. The registry owns adoption facts and
owns no version number.**

1. **`tool-registry.yaml` is created, with `tool-registry.schema.json` and
   `knowledge.tool_registry_valid`.** Every other registry here has both.

2. **An entry declares adoption, not version.** `id`, `name`, `purpose`, `licence`, `source`,
   `install` (with a pinned ref), `adopted_by` naming the artifact that decided it, `tier`,
   `exit`, `declared`, and `notes`. `purpose` is one sentence and is the field a future reader
   actually needs — the same role `provides` plays in a manifest under STD-0013.

3. **The registry holds no version where another file already holds one.** An entry for a tool
   pinned in `toolchain-registry.yaml` carries `pin_home: toolchain-registry` and **no version
   field**. RFC-0015 gives a fact one home; a version copied into a second registry is this
   repository's oldest defect moved sideways, and `knowledge.tool_registry_holds_no_version`
   rejects an entry carrying both.

4. **A tool with no other home pins here, and pins immutably.** `install.ref` must be a commit
   sha, a signed tag, or a package version — never a branch, never `latest`, never a bare clone.
   `knowledge.declared_tool_install_is_pinned` fails on a mutable ref.

5. **`exit` is mandatory and is prose.** What is lost, what replaces it, and how long. REQ-0012 §5
   asks for an exit plan for anything the platform cannot run without, and the cheapest moment to
   write one is while the integration is small. It is required for **every** tier, not only
   `critical`, because the tier is a judgement made on the day of adoption and the judgement is
   what ages.

6. **`adopted_by` must resolve to an artifact id.** A tool in the estate with no artifact behind
   it is a decision nobody made — which is the standing rule that governance precedes
   implementation, given a field. Where a tool predates its artifact, `adopted_by` names the
   artifact that *records* it and the `notes` say the adoption was retrospective. **Recorded
   rather than backdated**, exactly as `toolchain-registry.yaml` did for the OpenTofu entry it
   added eight days late.

7. **Scope: a tool is a program the estate installs, pins, or depends on the behaviour of.** Not
   every binary a script invokes. `sed`, `awk` and `grep` are POSIX surface, not adoptions; `jq`,
   `gh` and `ripgrep` are installed choices that a machine may lack. The boundary is *"would a
   clean workstation fail without it, and did someone choose it?"* — both, not either.

8. **Migrating the existing `tooling:` pins out of `toolchain-registry.yaml` is not decided
   here.** Those tools get entries under item 3 with `pin_home: toolchain-registry`, so coverage
   is complete on the day this lands; the version numbers do not move. Changing behaviour while
   moving it is the failure `artifact-registry.yaml` records about ADR-0010's `ci: false`, and it
   would make a registry's first change also its first regression. Recorded in
   `DECISION-DEBT.md`.

## Consequences

**Accepted costs**

1. **An eleventh registry, and the estate now has two files about tools.** A reader asking "what
   version of `sops` do we run" must know it is in `toolchain-registry.yaml` and not the one
   named after tools. That is a genuinely bad affordance and it is the price of item 3 — the
   alternative is two answers, and a wrong second answer is worse than a confusing first one. The
   `pin_home` field exists to make the redirect explicit rather than tribal.

2. **`exit` will be written badly at first.** A mandatory prose field that nobody's work depends
   on is a field people fill from the neighbouring entry — the same shape as ADR-0040's cost 3
   about `actors` and `external`. Required anyway, because an exit plan written after the
   dependency is load-bearing is written under pressure and is usually fiction.

3. **Item 7's boundary is a judgement and no check can make it.** `knowledge.tool_is_registered`
   was considered and not declared: every reading that is mechanically decidable is either too
   wide (every binary in every script) or too narrow (only what a declared installer installs).
   Declaring an undecidable check would be ADR-0002's failure — a rule marked enforced that
   enforces nothing. The gap is that a tool can be adopted and never registered, and nothing will
   say so.

4. **Retrospective entries make the registry look better-governed than the history was.** Item 6's
   `notes` requirement is the mitigation and it depends on honesty rather than on a check.

**Gained**

- The question *"what do we use, why, under what licence, and how would we leave"* has an answer
  that is one file rather than a survey of nine.
- REQ-0012 §1 and §5 acquire a place to be satisfied, and REQ-0014 acquires the provenance record
  it has been asking for since 2026-08-07.
- The mechanism that spread an end-of-life Node runtime across three repositories is closed on the
  tool half of the estate: there is now an entry to derive from.
- An installer can be generated from the registry, which turns `RUN-0001`'s hand-installed tools
  into declared ones.

**Rejected alternatives**

**Extend `toolchain-registry.yaml`.** Rejected: its header states one entry per stack and its
authority is the pin. Widening it to hold adoption facts for `gh` and `jq` would mean either
inventing a fake stack for them or nesting them under an unrelated one — which is the defect the
`tooling:` key already is, adopted deliberately instead of by accident.

**A markdown table in `README.md` or a `TOOLS.md`.** Rejected: it is the "list in prose" shape
C-0005 moved repository classes *out of*, and `artifact-registry.yaml`'s header names README and
CLAUDE.md prose as the only two places its list can still rot unobserved. A tenth such place is
not an improvement.

**A field on each repository's `manifest.yaml`.** Rejected: it distributes one fact across every
repository, so the estate-wide question needs a survey and the licence of a tool is answered
differently in each checkout. `depends.infra` already shows the shape working for
*per-repository* dependency; a tool's licence and exit plan are not per-repository facts.

**Record only what is installed, generated by scanning a workstation.** Rejected: it records the
machine rather than the decision, so it cannot hold `purpose`, `adopted_by` or `exit` — the three
fields the request was actually about. It also makes whatever someone happened to install
authoritative, which is the mechanism this artifact exists to close.

**Do nothing and note tools in the adopting ADR.** Rejected: it is the status quo. ADR-0057 adopts
`sops` and `age`, and their versions still ended up under `stack: ansible` because an ADR is not a
place a version may live (ADR-0048). Prose scattered across adopting artifacts is not a list.

## Related

- RFC-0040 — finding 5 is this artifact's argument
- ADR-0048 — the fifth instance, and the reason no version lives in an artifact
- ADR-0022 · ADR-0014 · ADR-0015 · C-0005 — the other four instances
- RFC-0015 — a fact has one home; decision 3
- ADR-0065 — the adoption that forced this, and the first entry
- ADR-0057 — `sops` and `age`; adopted by an ADR, pinned under an unrelated stack
- REQ-0012 — §1 approval before adoption, §5 exit plan; decisions 2 and 5
- REQ-0014 — supply-chain integrity; decision 4
- RUN-0001 — the hand-installed tools this makes declarable
