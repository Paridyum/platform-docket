---
id: RFC-0004
title: A repository class is registry data, not a row in a locked table
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec, template]
applies_to: ["*"]
tags: [extensibility, repo-classes, stacks, sdk, mobile, constitution]
breaking: true
resolved_by: C-0005
enforced_by: []
---

# RFC-0004 — A repository class is registry data, not a row in a locked table

## Status

**PROPOSED** as of 2026-08-07. **Feedback deadline: 2026-09-14.** Stage 2 of ADR-0013.

Supersedes nothing yet. If accepted it requires superseding **three** locked artifacts —
C-0002, ADR-0003 and ADR-0010 — once each, and never again if written correctly. The first
draft said one; see *The true supersession set*.

RFC-0003 (TypeScript and Next.js) becomes the first consumer of this mechanism rather
than a standalone change. Read this one first.

**C-0005 was itself superseded by C-0007 on 2026-08-09**, which carries the class-registry
delegation forward unchanged — the mechanism this RFC proposed is intact, and C-0007 is where to
read it now. The pointer below is left naming C-0005 because that is the artifact that closed
this question; following a supersession chain is the reader's job and rewriting history to skip
a link is how the record stops being one.

**Resolved by C-0005 on 2026-08-07.** This RFC was still `proposed` with `resolved_by: null`
while `class-registry.yaml` existed, C-0005 was `locked` and naming C-0005 in its title, and ten
classes were declared through the mechanism — the artifact had landed and its RFC had not been
closed. Recorded rather than quietly flipped, because an RFC trailing the reality it proposed is
the same class of drift as a stale count in a header: nothing was wrong, nothing was maintained,
and the lifecycle in README.md says `RFC → accepted → ADR` for a reason.

Found while auditing gaps after ADR-0022. RFC-0003 had the same shape and was narrowed rather
than closed, since its subject genuinely remains open.

## Problem

Adding one repository class costs **six coordinated edits, one of which is superseding the
naming constitution.**

| Where a class is enumerated | Locked? |
| --- | --- |
| C-0002's prefix table — 7 rows | **yes** |
| ADR-0003's `applies_to` — a web app is deployed | **yes** |
| ADR-0010's `applies_to` — a web app needs CI | **yes** |
| `knowledge.schema.json` `scope` enum — 8 values | no |
| `manifest.schema.json` `class` enum — 7 values | no |
| `check-knowledge.sh` `scope_value_for()` map | no |
| The creation guard's path patterns | no |

C-0002 is locked and supersession here is whole-artifact, so adding `web-` means restating
every naming rule in it — repositories, crates, binaries, HTTP paths, databases, events,
Kubernetes, environments — to change one table row.

**The foreseeable classes are not one.** Web applications, React Native, Flutter, generated
SDKs, shared packages, background workers. Six classes added one at a time is **six
supersessions of the naming constitution**, each one a full rewrite, each one an
opportunity to lose a detail. Nobody will do that. What will actually happen is that the
second stack arrives without a class, because the ceremony is too expensive — which is how
the previous estate ended up *"predominantly Python with TypeScript frontends, none
matching C-0002"* (ADR-0009).

A second, quieter version of the same problem, and the sharper one: **`applies_to` is a
prefix glob, so a locked artifact can never be extended to a new prefix.** ADR-0003 and
ADR-0010 are neither Rust-specific nor universal — they sit in a middle category that must
grow with every deployable class and cannot. The Rust standards are *not* affected and were
wrongly named in the first draft; see *The true supersession set*.

## This repository has solved this exact shape twice

The pattern is established, which is the strongest argument that it applies again.

- **ADR-0014** — check ids were going to be hardcoded in `tool-forge`. Moved to
  `check-registry.yaml`, validated by a schema, with resolution defined as set arithmetic.
  Adding a check is now a registry entry.
- **ADR-0015** — scope was an enumeration that silently excluded two real repository
  classes. Fixed by defining *resolution* as a rule (`org` matches everything) rather than
  by extending a list.

Both moved an enumeration out of prose into validated data, and both kept the *rule* in the
artifact. This is the third instance: **C-0002 should define the naming grammar; a registry
should hold the classes.**

## Proposal

**`class-registry.yaml`, alongside `check-registry.yaml`, is the single source of truth for
repository classes. C-0002 is superseded once to define the grammar and delegate the list.**

```yaml
version: 1
classes:
  - class: service
    prefix: svc-
    scope: service            # the value artifacts use in `scope`
    stack: rust               # which language standards bind
    description: A business capability service, one bounded context
    deployable: true
    declared: 2026-08-07
  - class: web
    prefix: web-
    scope: web
    stack: typescript
    description: A user-facing web application
    deployable: true
    declared: 2026-08-07
```

Each entry generates, rather than duplicates:

- the `scope` enum in `knowledge.schema.json`
- the `class` enum in `manifest.schema.json`
- `scope_value_for()` in the gate — **the prefix-to-scope map stops being hardcoded**, which
  ADR-0015 already flagged as a maintenance hazard
- the creation guard's path patterns
- `repo.name_matches_scheme`'s accepted prefixes

Adding a class becomes: **one registry entry, one standard, one template, one CI workflow.**
No supersession.

### `stack` is the second axis, and it is the one that actually matters

A class says *what a repository is*. A stack says *which language rules bind it*. They are
different, and conflating them is what makes STD-0001's `applies_to` list unmaintainable.

```
class     web-        stack typescript
class     mobile-     stack typescript      (React Native — same stack, different class)
class     mobile-     stack dart            (Flutter — same class, different stack)
class     svc-        stack rust
class     sdk-        stack generated       (multi-language, not hand-written)
```

Standards then bind to a **stack**, not to a list of prefixes. STD-0001 becomes "the Rust
testing standard, binding stack `rust`" and does not need superseding when `web-` appears —
it simply does not match. A future shared standard binds a stack set, and adding a stack to
it is a registry change rather than a supersession.

**This is the part that makes React Native and Flutter cheap later.** React Native is the
`typescript` stack in a `mobile-` class; Flutter is a `dart` stack in the same class. Most
of the TypeScript work RFC-0003 proposes is reused by React Native for free.

## What each foreseeable stack needs beyond a class

Researched now so the mechanism is designed against real requirements rather than a guess.

### SDKs — `sdk-`, stack `generated`

An SDK is **generated code**, and that is already governed: C-0004 makes generated code
immutable (`agent.generated_code_immutable`), and STD-0002 already requires `API.md` to be
generated from `specs/openapi.yaml` with hand-editing a CI failure. The discipline exists.

What is missing is publishing. Industry practice is *evergreen SDKs* — client libraries
regenerated from the spec on every API change and published automatically to npm, PyPI,
crates.io, Maven Central and so on. OpenAPI Generator covers 50+ languages and is
explicitly *"a solid option when you have the bandwidth to build the surrounding
governance, versioning and publishing infrastructure yourself."* That infrastructure is the
actual work, not the generation.

Consequence for the class model: an `sdk-` repository has **no hand-written source**, so
most code standards do not apply to it, and the checks that do are about provenance — is
this current with the spec, was it regenerated rather than edited.

### Mobile — `mobile-`, stacks `typescript` (React Native) and `dart` (Flutter)

Three constraints with no web or Rust analogue:

- **Code signing certificates and provisioning profiles are long-lived secrets in CI.**
  This is REQ-0001 territory and it is the strongest single argument for resolving runtime
  secret delivery before mobile, not after.
- **App store review breaks continuous delivery.** REQ-0004's progressive delivery assumes
  you control the rollout. You do not; review latency sits between merge and users.
- **OTA updates are a second deployment path that bypasses store review** — and would
  bypass CI gates too if ungoverned. CodePush was deprecated in March 2025 and App Center
  retired with it; EAS Updates is the 2026 successor, with Hermes bytecode diffing and
  phased rollouts. OTA can change the JS bundle and assets but **not native code,
  permissions or store metadata**, so the boundary between "OTA-able" and "needs a release"
  is a governance rule, not a technical detail.

And one cross-cutting consequence worth stating loudly:

> **A mobile client you cannot force-update makes API deprecation materially harder.**
> REQ-0010 §4 assumes a `Sunset` header and an announced removal date. A year-old app on
> someone's phone reads neither. Mobile turns API versioning from a policy into a
> compatibility obligation with a long tail.

### Shared packages — `pkg-` or `lib-` per stack

`lib-` already exists and is Rust-specific by C-0002 (*"`lib-*` repositories publish library
targets only"* and the crate naming rule). A TypeScript shared package is a different thing
with the same intent. Either `lib-` becomes stack-parameterised, or a second prefix appears
— an open question below rather than a decision here.

## Options

### A. Registry-backed classes, C-0002 superseded once

As proposed. One supersession, paid now, and every future class is a registry entry.

**Cost.** The supersession is real and it is of the naming constitution, the most-cited
artifact in the repository. It must restate every naming rule it currently carries. Doing
it while the rules are few and fresh is the cheapest this will ever be.

### B. Add classes one at a time as needed

No new mechanism. Each class supersedes C-0002.

**Rejected.** Six supersessions of a constitution is not a plan, it is a prediction that
the second one will not happen. The observable outcome is the previous estate: TypeScript
arrived without a class and the whole estate was archived.

### C. One generic `app-` class covering web, mobile and everything else

Cheapest, and wrong for the same reason ANTI-0006 rejects a god service: a class that means
everything constrains nothing. A web application and a Flutter app share almost no CI, no
test runner, no deployment model and no release constraints.

### D. Registry-backed classes, but do not supersede C-0002 — let the registry contradict it

**Rejected outright.** A registry that disagrees with a locked artifact is two sources of
truth for one question, and ADR-0002's whole argument is that this is the failure being
prevented. If the registry is authoritative, C-0002 must say so.

## Recommendation

**Option A, and do the C-0002 supersession as the first act rather than the last.**

This reverses RFC-0003's sequencing, deliberately. That RFC put the supersession last to
avoid paying it until necessary. With one class that was right. With six foreseeable
classes it is wrong: every stack added before the mechanism exists is a stack that has to
be migrated into it afterwards.

Order:

1. **Supersede C-0002** — naming grammar stays, the class list moves to the registry.
2. **`class-registry.yaml` + `class-registry.schema.json`**, with the seven existing
   classes, so it is descriptive on day one and provably equivalent to what C-0002 said.
3. **Generate the enums and the prefix map from it**, removing the four hardcoded copies.
4. **Re-express standards as binding a `stack`** rather than an `applies_to` prefix list.
   This is the part that needs care — STD-0001/0003/0004 are locked, so it means
   superseding them, and that cost belongs in this decision rather than a later surprise.
5. **Then** RFC-0003's TypeScript work, as the first class added through the mechanism —
   which also proves the mechanism works before anything depends on it.

## The true supersession set — corrected 2026-08-07

The first draft guessed four supersessions and named the wrong three. Checked properly:

| Artifact | `applies_to` | Needs superseding for a `web-` class? |
| --- | --- | --- |
| C-0002 | — | **Yes** — the prefix table is the class itself |
| ADR-0003 Kubernetes | `platform-*, svc-*, infra-*` | **Yes** — a web app is deployed |
| ADR-0010 CI/CD | `platform-*, svc-*, lib-*, tool-*` | **Yes** — a web app needs the workflow |
| STD-0001 / 0003 / 0004, ADR-0005 | Rust prefixes | **No** — these must *not* bind `web-` |
| STD-0002, STD-0005 | `["*"]` | **No** — already cover every class |

So it is **three, not four, and not one**. The Rust standards need no change precisely
because they are Rust standards; they correctly fail to match a new prefix. The
cross-cutting ones already use `["*"]`. What breaks is the middle category: artifacts that
are neither Rust-specific nor universal.

### Why there is no route around it

Three were tried and all fail:

1. **Give the `web` class the scope value `service`** so it matches ADR-0003's scope.
   Fails: `applies_to` is a *prefix glob*, and `web-app` matches neither `svc-*` nor
   `platform-*`. Scope gets you in the door; `applies_to` shuts it.
2. **Have the class registry declare which extra artifacts bind a class.** Fails: that is a
   second resolution mechanism competing with `scope`/`applies_to`, which is the
   two-sources-of-truth failure ADR-0002 exists to prevent.
3. **Scope the class `org`.** Fails: `org` binds *every* artifact (ADR-0015), not the two
   wanted.

**`applies_to` as a prefix glob makes a locked artifact permanently un-extendable to a new
prefix.** That is the defect underneath this whole RFC, and it is worth naming separately
from the class list, because it will recur for every artifact in that middle category.

### The supersessions must be written to be the last ones

This is the point of paying the cost. C-0002's successor delegates the class list to the
registry. ADR-0003's and ADR-0010's successors must replace their prefix globs with
something registry-driven — *"every class declared `deployable: true`"*, *"every class
declared `ci: reusable`"* — so that adding `mobile-` later needs **no further
supersession**. A supersession that hardcodes `web-` alongside `svc-` has bought one class
and left the defect in place.

## Open questions
2. **`lib-` for TypeScript, or a second prefix?** One prefix with a `stack` field is
   cleaner; two prefixes are more obvious at a glance.
3. **Is `mobile-` one class with two stacks, or `rn-` and `flutter-`?** One class matches
   the deployment target and the store constraints; two match the toolchains.
4. **Where do SDK publishing credentials live?** Registry tokens for npm, PyPI and crates.io
   are exactly REQ-0001's problem and are a further argument for resolving it early.
5. **Does `spec-` become the SDK source of truth**, with `sdk-*` repositories generated from
   it and never hand-edited? That is the evergreen model and it fits C-0004 unchanged.
6. **Is Flutter actually wanted**, or is React Native sufficient? Two mobile stacks is two
   sets of everything, and the class model should not be used to justify a choice nobody has
   made.

## Related

- C-0002 — the prefix table this moves to a registry; the artifact to be superseded
- ADR-0014 — the precedent: enumeration to registry, resolution as a rule
- ADR-0015 — the second precedent, and its note that the prefix-to-scope map is hardcoded
- ADR-0009 — the previous estate, which acquired TypeScript without a class
- RFC-0003 — TypeScript and Next.js, the first consumer of this mechanism
- REQ-0001 — secret delivery, which mobile signing and SDK publishing both need
- REQ-0004 — progressive delivery, which app store review breaks
- REQ-0010 §4 — API deprecation, which unforced mobile clients make materially harder
- ANTI-0006 — why one generic `app-` class is rejected
