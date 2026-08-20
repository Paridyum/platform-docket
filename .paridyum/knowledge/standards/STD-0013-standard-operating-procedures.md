---
id: STD-0013
title: Every repository documents how it is operated and declares what it provides
type: standard
status: accepted
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
scope: [infra, library, mobile, platform, sdk, service, spec, template, tool, web]
applies_to: ["*"]
tags: [documentation, sop, operations, discoverability, manifests, knowledge-capture]
breaking: true
enforced_by:
  - check: docs.sop_present
    severity: error
    rationale: >
      A capability whose operation lives only in the transcript of the session that built it
      is undelivered (C-0010), and STD-0011 states what a transcript is worth to the next
      agent. Presence is the floor and it is the half that can be rejected mechanically.
  - check: docs.sop_covers_every_entrypoint
    severity: warn
    rationale: >
      Presence alone is satisfiable by a stub. Requiring every declared entrypoint to appear
      beside a command raises the floor; it stays `warn` because it is a heuristic over prose
      and an error-severity heuristic gets routed around, which is worse than none.
  - check: docs.manifest_schema_valid
    severity: error
    rationale: >
      `provides` is enforced by the schema rather than by a new id. The manifest is already
      validated on every run, and a second check reading the same file would be a second
      place to keep in step.
---

# STD-0013 — Standard operating procedures

## Status

**`accepted`, and lockable.** `docs.sop_present` runs today in `check-repo.sh` with its defect
planted in `scripts/test-checks.sh`, which is ADR-0002's bar. It is created `accepted` and locked
in a separate commit per the `new-decision` skill §5b — ADR-0029 carries two errata because that
ordering was not followed there.

Implements RFC-0032.

## The rules

> **1. Every repository that declares a class carries `SOP.md`, and it is written in the change
> that completes the capability — not in a follow-up.**

> **2. Every repository declares in `manifest.yaml` what capability it provides, and retrieving
> that declaration comes before designing a new one.**

The second rule is the reason the first is worth its cost. An SOP nobody can find is an SOP
nobody reads, and the failure this standard exists to prevent is not *the tool was hard to use* —
it is *the tool was rebuilt by someone who did not know it existed*.

## Why a document of its own

STD-0002's required set answers *what is this*, *how is it built*, *what changed*, and *what is
it machine-readably*. None of them answers *I have this job in front of me, what exactly do I
do*, and the two files that come closest are each committed to a different question:

| File | The question it answers | Reader |
| --- | --- | --- |
| `README.md` | What is this, can I run it in 60 seconds | someone who just cloned it |
| `ARCHITECTURE.md` | How is it built, and why that way | someone about to change it |
| **`SOP.md`** | **I have this job. What exactly do I do?** | **someone who must use it, correctly, today** |
| `RUNBOOK.md` (service, platform) | It is 3am and this is paging | someone woken up |

A README that answers the operating question stops being a 60-second document, and a RUNBOOK
that answers it stops being short — which is the one property an incident document cannot lose.
RFC-0032 records the alternatives in full, including the one that should eventually win:
merging this standard into STD-0002 the next time that artifact is opened for any other reason.

## What `SOP.md` contains

**Task-indexed, not feature-indexed.** One section per job the repository exists to let someone
do, named in the operator's words rather than the code's — *reissue a document that was issued
wrongly*, not *the `issue` subcommand*.

Every task section carries all six:

1. **When you do this** — the trigger, in a sentence.
2. **Preconditions** — what must already be true, and the command that checks it rather than an
   instruction to assume it.
3. **The commands** — exact, copy-paste, in order, with the arguments a real invocation has.
4. **What correct looks like** — the actual output, pasted. Not a description of output.
5. **How you know it worked** — the verification command, separate from the command that did
   the work. A command that reports its own success is not evidence.
6. **When it goes wrong** — the failure messages *this task* actually produces, what each one
   means, and the recovery. Not a troubleshooting appendix at the end of the file, where it is
   read by nobody who is not already lost.

And two whole-document sections:

- **What this cannot do.** The boundary, so the next reader stops here rather than discovering
  it at the end of an afternoon. Same purpose STD-0002 gives ARCHITECTURE's *what was
  deliberately not built*, and the same reason it is the most valuable section.
- **Destructive and irreversible operations.** Named explicitly: what is destroyed, what
  survives, whether it is recoverable, and what must be true before it is run. C-0004 governs
  protected operations at org level; this is where a repository states which of its own commands
  are one.

**Skeleton:**

```markdown
# SOP — <repository>

<one sentence: what jobs this document covers>

## Task: <the job, in the operator's words>

**When:** <trigger>
**Preconditions:** <what must be true> — check with `<command>`

    <the commands, in order>

**Correct output:**

    <pasted, real>

**Verify:** `<a different command that proves it>`

**When it goes wrong:**
| What you see | What it means | What to do |

## What this cannot do
## Destructive operations
```

## What `SOP.md` is not

- **Not a tutorial.** A narrative is read once; a task index is read at the moment of need,
  which is the only moment it will ever be read.
- **Not the design.** *Why it works this way* is ARCHITECTURE's, and a reason inside a procedure
  is a reason someone must read past while under time pressure.
- **Not exhaustive command reference.** Every entrypoint appears (`docs.sop_covers_every_entrypoint`),
  under the task that uses it. A flag that no task needs is a flag with no reason to exist.

## `provides` in `manifest.yaml`

```yaml
provides:
  capability: Issues, registers and verifies the documents of an engagement.
  entrypoints: [docket]
```

- **`capability`** — one sentence, in the words someone would search for. It is what makes the
  repository findable; the SOP is what they find.
- **`entrypoints`** — the names a person or a caller actually types: binaries for `tool-*`, the
  crate for `library`, the base path for a service.

**This is the fact's one home** (RFC-0015). It is not a summary of the SOP's task list, and it is
never copied into a catalogue file — ADR-0040 already decided that cross-repository views are
generated from manifests, and `product-registry.yaml` refuses a `repos:` key on the same grounds.
A capability list assembled by hand is correct on the day it is written and kept correct
afterwards by nothing but attention.

## Retrieval before building

Before designing any new capability, ask what already provides it:

```sh
grep -A3 '^provides:' ~/Developer/paridyum/*/*/manifest.yaml
```

`forge catalog` should render this properly in time. **The obligation does not wait on it** —
tool-forge owes most of the declared id-space, and a rule that arrives when a binary does is a
rule that was not in force for every session in between.

**Nothing enforces this rule and nothing will.** *Did you check whether this already exists* is
not mechanically askable, which is STD-0011's position and C-0006's at a different setting.
Naming a proxy check here would be the false confidence ADR-0002 exists to prevent.

## How each class reads this

- **Class-less repositories are exempt.** `knowledge` declares no class by design (ADR-0015
  point 6) and its operating procedures already live in `runbooks/` and `scripts/`. Requiring
  `SOP.md` of it would be requiring it to become something it deliberately is not.
- **`library`** — the operator is a programmer integrating it, and rustdoc owns the API surface
  (STD-0002). Its SOP documents *integration tasks*: how the dependency is added, the call shapes
  that matter, and the failure modes that are not obvious from the type signature. It will be the
  shortest SOP in the estate, and if after the first library it reads as ceremony, the correction
  is to narrow this standard's `scope` — not to tolerate a file written dishonestly.
- **`service` and `platform`** — both `SOP.md` and `RUNBOOK.md`, and they must not converge. If
  a section belongs in both, it belongs in the RUNBOOK and the SOP links to it.
- **`spec` and `template`** — the tasks are regeneration and release: how a new version is cut,
  what consumers must do, how a breaking change is signalled.

## When it is written

**In the change that completes the capability.** Not in a follow-up issue, and not when someone
next needs it — by then the person who knew is gone and the cost is the original discovery again
(STD-0011). C-0010 is the shorter argument: there is no prototype tier, so a tool whose operation
is undocumented is not an unfinished document beside a finished tool. It is an unfinished
delivery.

## Enforcement, and what it does not reach

`docs.sop_present` rejects an absent file. `docs.sop_covers_every_entrypoint` rejects an
entrypoint that appears nowhere near a command. Neither can tell whether a procedure is correct,
current, or ever run — and `docs.no_stale_readme_commands`, the check whose shape would catch a
rotted command, is declared and unimplemented. An SOP carries more commands than a README, so
that gap is wider here than where it was first declared. Stated so it is not mistaken for
covered.

## Related

- RFC-0032 — the design, its alternatives, and its accepted costs
- STD-0002 — the required-file set this extends without superseding
- STD-0011 — write what will be needed again where it will be retrieved; this is that rule
  turned on a repository's own operation
- ADR-0040 — cross-repository views are generated from manifests, never hand-authored
- ADR-0036 — check parameters come from `args`, which is why a second standard adds a file
  without creating a second prose list
- C-0010 — there is no prototype tier; an undocumented capability is an undelivered one
- ADR-0015 — scope resolution, and why a class-less repository is exempt
