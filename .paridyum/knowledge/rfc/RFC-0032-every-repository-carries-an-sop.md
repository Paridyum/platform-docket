---
id: RFC-0032
title: Every repository carries an SOP, and every repository declares what it provides
type: rfc
status: proposed
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org]
applies_to: ["*"]
tags: [documentation, sop, operations, discoverability, manifests, knowledge-capture, std-0002, adr-0040, adr-0036]
breaking: true
enforced_by: []
---

# RFC-0032 — Every repository carries an SOP, and every repository declares what it provides

## Problem

A repository in this estate can be complete, reviewed, merged and green while carrying nothing
that tells a person or an agent **how to do a job with it**.

STD-0002's required set answers four questions: what is this (`README.md`, explicitly a
60-second document), how is it built (`ARCHITECTURE.md`), what changed (`CHANGELOG.md`), and
what is it machine-readably (`manifest.yaml`). The operating question — *I have this job in
front of me; which commands, in which order, what does correct output look like, and what do I
do when it does not* — is required only of `svc-*` and `platform-*`, and there `RUNBOOK.md`
answers a narrower question still: STD-0002 defines it as *"it is 3am and this is paging"*, with
mitigation deliberately ahead of diagnosis. That is incident response. It is not how the thing
is used on an ordinary Tuesday.

This is observable in the estate as it stands on 2026-08-19:

- **`tool-docket`** issues fourteen document types across a five-stage engagement machine,
  allocates serials, appends to a tamper-evident register, renders unregistered drafts, verifies
  an issued PDF years later, and performs DPDP erasure that leaves a tombstone. Its README shows
  one path — onboarding a new intern — in eight lines, because STD-0002 tells it to be a
  60-second document and it is obeying. Nothing in the estate records what a failed `verify`
  actually means, what `erase --confirm` destroys and what survives it, which stage owes which
  documents, or how to reissue a document that was issued wrongly.
- **`tool-forge`** is the same shape at smaller scale: two commands, one of which rewrites a
  vendored directory in another repository.

Both were built in-session. The knowledge of how to drive them exists in a transcript, and
STD-0011 already states what a transcript is worth: *"an agent starting tomorrow begins with
what is in `knowledge/` and nothing else."*

**The second half is the same defect one level up.** Nothing states what capability each
repository already owns in a form anyone retrieves *before* starting work. `manifest.yaml`
carries identity, ownership and dependencies; it does not carry what job the thing does for its
operator. So the question *"does something already do this?"* has no answer that is not reading
every README in the estate, and the practical consequence is that the capability gets written
again — inline, for one task, while the tool that already does it sits undeleted, unupdated and
unused.

**Who experiences it, and how often.** Every session after the one that built the thing, which
is every session but the first. **What happens if nothing is done:** the estate accumulates
tools only their author can drive, and C-0010 makes that worse rather than better — there is no
prototype tier, so every tool built is a tool someone is expected to operate correctly later.

**How we know it is fixed.** Someone who has never seen the repository can complete every job it
exists for from the repository alone; and one command over the estate answers *what already does
X* before any design starts.

## Constraints

Locked or accepted artifacts that bind this design:

| Artifact | What it binds here |
| --- | --- |
| **STD-0002** (locked, breaking) | The required-file set and the README's 60-second contract. Not superseded by this RFC — see alternatives. |
| **ADR-0036** | Check parameters come from `args`, not prose. The file list lives in `enforced_by[].args`, which is why a second standard can add a file without creating a second prose list. |
| **ADR-0040** + RFC-0015 | Cross-repository views are **generated from manifests**; a fact has one home. This forbids a hand-authored tool catalogue outright. |
| **C-0007** | A list that will be extended is registry data, never prose. |
| **ADR-0002** | `locked` requires a check that rejects the violation, and `owner` is a claim `scripts/test-checks.sh` must back by planting the defect. |
| **ADR-0015** point 6 | `knowledge` is class-less by design and carries no `manifest.yaml`. |
| **STD-0011** | Write what will be needed again where it will be retrieved. This RFC is that rule turned on a repository's own operation. |
| **C-0010** | No prototype tier. A tool whose operation is undocumented is a finished artifact with an unfinished delivery. |
| **ADR-0007** | The vendored snapshot is copied into every repository, which is what makes putting one tool's operating detail into `knowledge/` expensive for every other repository. |

## Proposal

### 1. `SOP.md` is required in every classed repository

A fifth required file, binding every class in `class-registry.yaml`. It answers the question no
existing file answers: **how is this operated, job by job.**

| File | The question it answers | Reader |
| --- | --- | --- |
| `README.md` | What is this, can I run it in 60 seconds | someone who just cloned it |
| `ARCHITECTURE.md` | How is it built, and why that way | someone about to change it |
| `SOP.md` | **I have this job. What exactly do I do?** | **someone who must use it, today, correctly** |
| `RUNBOOK.md` (svc/platform) | It is 3am and this is paging | someone woken up |

### 2. The content contract, so "in depth" is a shape rather than an adjective

`SOP.md` is **task-indexed, not feature-indexed**. One section per job the repository exists to
let someone do — named in the operator's words, not the code's. Every section carries:

1. **When you do this** — the trigger, in a sentence.
2. **Preconditions** — what must already be true, and how to check it rather than assume it.
3. **The commands** — exact, copy-paste, in order, with the arguments a real invocation has.
4. **What correct looks like** — actual output, not a description of output.
5. **How you know it worked** — the verification command, separate from the command that did it.
6. **When it goes wrong** — the failure messages this task actually produces, what each means,
   and the recovery. Not a generic troubleshooting appendix.

Plus two whole-document sections:

- **What this cannot do** — the boundary, so the next reader stops here instead of discovering
  it at the end of an afternoon. Same purpose as ARCHITECTURE's *what was deliberately not
  built*, which STD-0002 already calls its most valuable section.
- **Destructive and irreversible operations** — named explicitly, with what is destroyed, what
  survives, and whether it is recoverable. C-0004 governs protected operations; this is where a
  repository states which of its own commands are one.

### 3. `manifest.yaml` gains `provides`

```yaml
provides:
  capability: Issues, registers and verifies the documents of an engagement.
  entrypoints: [docket]
```

`capability` is one sentence: the job, in the words someone would search for. `entrypoints` are
the names a person actually types or calls — binaries for `tool-*`, the crate for `lib-*`, the
base path for a service. Required for every classed repository, enforced by the schema, so
`docs.manifest_schema_valid` covers it with no new check id.

This is the fact's one home (RFC-0015). It is deliberately **not** a second copy of the SOP's
task list; it is the sentence that makes the repository findable, and the SOP is what is found.

### 4. Discovery is a query over manifests, never a catalogue file

Before designing any new capability, retrieve what exists:

```sh
grep -A3 '^provides:' ~/Developer/paridyum/*/*/manifest.yaml
```

`forge catalog` should eventually render this properly — ADR-0040 already decided that
cross-repository views are generated — but **the retrieval step must not wait on tool-forge**,
whose implemented surface is the smaller part of what it is declared to own — the gate prints
that split on every run. A grep over the one home of the fact is available today, cannot drift, and is exactly what ADR-0040 forbids a hand-authored `TOOLS.md` for being
unable to claim.

### 5. Enforcement

| Check | Severity | Owner | State |
| --- | --- | --- | --- |
| `docs.required_files_present` cited again with `args: { files: ["SOP.md"] }` | error | `check-repo.sh` | implemented in this change; defect planted in `test-checks.sh` |
| `docs.manifest_schema_valid` (existing) | error | `check-knowledge.sh` | covers `provides` via the schema |
| `docs.sop_covers_every_entrypoint` — every name in `provides.entrypoints` appears in `SOP.md` under a task heading with a command block | warn | `tool-forge` | declared, owed |

The third is what stops a stub satisfying the rule, and it is a proxy: it proves the entrypoint
is mentioned, not that the mention is useful. Declared at `warn` and it should stay `warn` for
the reason STD-0011 gives — an error-severity heuristic over prose is routed around within a
week.

**The retrieval obligation in §4 is enforced by nothing.** *Did you check whether this already
exists* cannot be mechanically asked, which is C-0006's and STD-0011's position at a different
setting. Stating it and declaring no check is honest; naming a proxy would be the false
confidence ADR-0002 exists to prevent.

## Alternatives considered

1. **Extend `README.md` instead.** Rejected: it destroys the 60-second contract STD-0002 makes
   deliberately. A README that answers every operating question is a README nobody finishes, and
   the orientation function is lost to serve the operating one.
2. **Reuse `RUNBOOK.md` and require it of all classes.** Rejected: one filename, two audiences,
   two questions. A service would then hold the 3am path and the ordinary path in one document,
   and the 3am path is precisely the one that must stay short — STD-0002 puts mitigation before
   diagnosis for that reason. Conflating them makes the incident document longer, which is the
   one thing it must never be.
3. **A per-tool runbook in `knowledge/runbooks/`.** Rejected on three grounds: STD-0002 makes the
   repository the unit of documentation; every tool change would become a knowledge PR reviewed
   by CODEOWNERS who did not make it; and ADR-0007 vendors this repository into every other, so
   `tool-docket`'s operating detail would be copied into every unrelated checkout in the estate.
4. **Supersede STD-0002 with a merged documentation standard.** Correct in the abstract — one
   artifact should own the required-file set. Rejected *now* because it costs the full C-0004
   supersession of a locked, `breaking: true` standard carrying seven `enforced_by` entries, in
   order to add one row, and every existing citation of STD-0002 would need re-checking against
   a new id. STD-0011's rule — *correcting an existing artifact beats writing a new one* — is
   real and is being deliberately traded against supersession cost. Named here as the eventual
   tidy-up rather than pretended away.
5. **A hand-authored catalogue — `TOOLS.md`, or `tool-registry.yaml` in this repository.**
   Rejected: it is the exact shape ADR-0040 forbids, and `product-registry.yaml` already refuses
   a `repos:` key on the same reasoning. A capability list assembled by hand is correct on the
   day it is written and kept correct afterwards by nothing but attention.
6. **Convention rather than a required file.** Rejected: the estate has two tools, both
   undocumented for operation, and both were built by people who agreed with this convention in
   principle. That is the evidence.

## Consequences

What this makes worse, stated plainly:

- **A fifth required file.** Every new repository costs more before its first merge, and
  `templates/` must carry an SOP skeleton or the template itself fails the gate it ships with.
- **A required file is satisfiable by a stub.** Presence is checkable; depth is not.
  `docs.sop_covers_every_entrypoint` raises the floor and does not reach usefulness. This is the
  same honest limit STD-0011 records for itself.
- **The required-file obligation is no longer stated in one place.** ADR-0036's `args` mean
  there is no second *list*, but there is now more than one artifact to look in for *whether a
  file is required*. That is a
  real cost, accepted for alternative 4's reason, and it is the thing to fix when STD-0002 is
  next opened for any other reason.
- **`lib-*` is the weakest fit.** A library's operator is a programmer integrating it, and
  rustdoc already documents the API. The SOP that survives that overlap is the integration-task
  document — add the dependency, the three call shapes that matter, the failure modes — and it
  will be the shortest SOP in the estate. If, after the first library, it reads as ceremony, the
  right correction is to narrow the standard's `scope`, not to tolerate a file nobody writes
  honestly. Recorded as an open question rather than settled.
- **An SOP drifts like every document.** It carries more commands than a README, so
  `docs.no_stale_readme_commands` — declared, unimplemented, `warn` — matters more here and
  covers it less.

## Rollout

1. **This RFC lands alone**, per ADR-0013 §4 and `pr.design_artifact_predates_implementation`.
2. **A new standard** is written `accepted`, citing the checks above. Its id is allocated when
   it is written — naming it here would be a forward reference, which the gate rejects, and this
   RFC's `resolved_by` is where the link belongs once the artifact exists.
3. **`check-repo.sh`** requires `SOP.md` for classed repositories; **`scripts/test-checks.sh`**
   plants the missing-file defect and watches it fail, which is what makes `owner` a claim rather
   than a wish. **`manifest.schema.json`** gains `provides` as required for classed repositories.
4. **Backfill, in the same change**: `tool-docket` and `tool-forge` get real SOPs and `provides`
   blocks. The estate must not be in violation of its own rule at the moment the rule lands —
   that is how a gate gets disabled in its first week.
5. **Lock the standard in a separate commit** once the gate is green, per the `new-decision` skill
   §5b. ADR-0029 carries two errata because that ordering was not followed.

**Undo** is cheap: delete the standard, revert the check and the schema field. The SOPs remain as
ordinary documentation and cost nothing to keep.

## Open questions

1. **Does `knowledge` owe an `SOP.md`?** It is class-less by design (ADR-0015 point 6) and its
   operating procedures already live in `runbooks/` and `scripts/`. Proposed answer: no, and the
   standard should say so explicitly rather than leave it to be inferred from the absence of a
   class.
2. **Does the `paridyum-platform` plugin owe one?** Its skills are operated constantly and its
   source is not a repository in this estate. Deferred until it is one.
3. **`provides.entrypoints` is a declaration, not a derivation.** A tool can add a subcommand
   and not declare it, and `docs.sop_covers_every_entrypoint` will pass. Deriving entrypoints
   from the binary's own command table would close it and requires building the binary, which is
   tool-forge work.
4. **`scripts/check-significance.sh` does not exist**, though ADR-0013's `design-change` skill
   opens by instructing the reader to run it. Found while following that skill to produce this
   RFC. Unrelated to this design and recorded so it is not found again the same way.
