---
id: ADR-0019
title: A specification is decomposed into owned bounded contexts before code exists
type: adr
status: locked
created: 2026-08-07
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org, platform, service, library, spec]
applies_to: ["*"]
tags: [architecture, bounded-context, ownership, decomposition, ai-agents]
breaking: false
enforced_by:
  - check: agent.service_creation_escalates
    severity: error
    rationale: Creating a service is knowable at the first write and is the decision that is effectively permanent once shipped; every other guard fires after the boundary is already paid for.
---

# ADR-0019 — A specification is decomposed into owned bounded contexts before code exists

## Status

**LOCKED** as of 2026-08-07. Resolves RFC-0002, which carries the full analysis, the
options considered and the research behind them. This artifact records what was decided;
it does not restate the reasoning.

## Context

ADR-0013 establishes design-before-code and computes significance from the diff, but its
lifecycle begins at *"issue or RFC §Problem"* — it assumes the problem arrives already
atomic. A specification containing a hundred requirements across nine actors is not
atomic, and nothing made it so.

Worse, ADR-0013 enforces at the pull request. By then the code exists, the service is
created, the table is owned, and the coupling is paid for. For a wrong boundary that is
too late.

RFC-0002 established that most of a decomposition pipeline already existed — the
lifecycle, `REQ-` artifacts, C-0002's naming, and `manifest.yaml` already declaring the
dependency graph and event contracts as data. Four gaps remained: no intake step, no
ownership declaration, no recorded forbidden shapes, and nothing stopping specification →
code at the moment it happens.

## Decision

**A specification is decomposed into bounded contexts with declared ownership before any
service is created, and creating a service requires a human.**

1. **The forbidden shapes are artifacts.** ANTI-0001 through ANTI-0006 record what must
   not be built and why. Three are `locked` with a check; three are `accepted` with none,
   because ADR-0002 is right that boundary judgement cannot be mechanically verified and
   a check there would manufacture false confidence.
2. **Ownership is declared in `manifest.yaml`.** `owns.tables` and `owns.events` name what
   a service is sole writer of. `manifest.schema.json` makes STD-0002's prose contract
   executable — which needed no supersession, because STD-0002 makes no exhaustiveness
   claim about its fields.
3. **Ownership is enforced by database grants, not by the declaration.** A distinct
   database role per service, write access only on declared tables, DDL held by a separate
   migrator role. The manifest declares; the grant stops the write; the gate verifies the
   two agree in both directions. This is the substantive half — a static check on
   cross-service access is weak, and per-service credentials are not.
4. **Decomposition has a procedure.** The `decompose-spec` skill: check what already
   exists, extract atomic requirements, discover entities, group into contexts, assign
   single ownership, prefer events over calls, check the graph against the ANTI- set, write
   one `REQ-` per context — then **stop and ask**.
5. **Creating a service escalates to a human.** A new `manifest.yaml`, or a new crate under
   a classed repository, prompts before it is written. `ask`, never `deny`.

### Why a creation guard is legitimate when ADR-0013 rejected one

ADR-0013 rejected a PreToolUse guard on implementation, and its reasoning stands: *"an
agent cannot know at first file-write whether the eventual diff will be significant, so the
hook would fire constantly on trivial work and be disabled."*

That reasoning does not reach this case, and the distinction is the whole justification.
Creating a service **is** knowable at the first write — a new `manifest.yaml` or a crate
under `svc-*`/`platform-*`/`lib-*`/`tool-*` is unambiguous — and it is precisely the
irreversible act. Ordinary editing is untouched, so the guard fires once per service rather
than constantly, and does not train the habit of approving without reading (C-0004).

`ask` rather than `deny` follows ADR-0008: *"a guard that denies every dangerous thing is
disabled within a week."* An exploratory spike is legitimate — ADR-0013 protects it
explicitly — so a human may simply say yes. What must not happen is a service appearing
without anyone noticing.

## Consequences

**Accepted costs**

- **The backlog grows by one** — `agent.service_creation_escalates`, 111 → 112.
- **This ADR ratifies work that was already shipped**, which inverts ADR-0013 twice over:
  implemented before the RFC was accepted, and accepted the day it was written. That
  happened at the author's direction and is recorded in RFC-0002's Status rather than
  hidden. Refusing to ratify shipped code would not un-ship it, but **the habit is worth
  not repeating**, and this note exists to make the next instance harder to justify.
- **Ownership declared is not ownership enforced** until the grants exist in a real
  database. Until then `owns.tables` is an intention, and the gate can only check that the
  intention and the template agree.
- **`domain.entity_has_single_owner` is only half implemented.** The declaration-versus-
  grants half runs; the two-repositories-claiming-one-table half needs `forge audit`, which
  does not exist. Recorded in its notes; do not read a pass as proof of single ownership
  across the org.
- **A guard that prompts on service creation will occasionally annoy.** That is the design
  operating, not failing — but if it ever fires on ordinary work it is a bug and must be
  fixed rather than worked around.

**Gained**

- The forbidden shapes are readable by an agent before it proposes a boundary, which is
  most of the protection and required no machinery.
- `docs.manifest_schema_valid` became implementable after being declared-and-unimplementable
  since it was written.
- The irreversible act now has a human in front of it, at the only moment where the cost of
  stopping is minutes rather than years.

**Rejected alternatives**

Recorded in full in RFC-0002 §Options. In brief: a decomposition skill alone (advisory, so
an agent that skips it is unaffected); the same plus artifacts but no guard (leaves the
irreversible act unguarded); and a fifteen-stage pipeline with a directory per stage,
rejected on ADR-0008's precedent that *"empty scaffolding is worse than none because it
advertises coverage that does not exist."*

## Known limitation

**Bulk repository creation is not governed.** A decomposition producing nine bounded
contexts means nine repositories under ADR-0001, and this ADR governs creating one service
at a time. Nine prompts in a row is not review — it is a formality that trains approval
without reading, which is the exact failure C-0004 warns about.

This is RFC-0002's third open question and it is **not answered here**. Answering it needs
a scaffolding path that does not exist yet, and inventing one now would be a decision made
without a single service to test it against.

## Related

- RFC-0002 — the analysis, options and research this resolves
- ADR-0013 — the lifecycle this extends at the front, and whose guard rejection it narrows
- ADR-0001 — polyrepo; one bounded context is one repository
- ANTI-0001 … ANTI-0006 — the shapes this exists to prevent
- STD-0002 — `manifest.yaml`, extended rather than superseded
- ADR-0008 — `ask` over `deny`, and the empty-scaffolding precedent
