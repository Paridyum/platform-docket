---
id: ADR-0107
title: A shared library's repository name carries its stack, and the artifact it publishes does not
type: adr
status: accepted
created: 2026-09-24
locked_at: null
supersedes: null
superseded_by: null
scope: [org, library, platform, tool]
applies_to: ["*"]
tags: [naming, library, stack, auth, lib-auth, c-0007, adr-0092, adr-0047, rfc-0004, rfc-0009, std-0022, anti-0004]
breaking: true
enforced_by:
  - check: repo.name_matches_scheme
    severity: error
    rationale: >
      Decision 1. The segment is legal only where the class declares more than one stack, which
      is what keeps this a check rather than a free-text suffix: `lib-auth-rust` passes because
      `library` declares two, `svc-billing-rust` fails because `service` declares one.
  - check: cargo.package_name_matches_repo
    severity: error
    rationale: >
      Decision 2. The derivation gains a step — minus the class prefix, minus the stack segment —
      and an implementation reading the pre-2026-09-24 spec would have demanded
      `paridyum-auth-rust`. Both specs were corrected before either was implemented, which is the
      one moment that costs nothing.
---

# ADR-0107 — A shared library's repository name carries its stack, and the artifact it publishes does not

## Status

**ACCEPTED** as of 2026-09-24 by the operator.

**This narrows one sentence of C-0007** — *"never a technology"* — for the case C-0007 did not
have in front of it. C-0007 is `accepted` rather than `locked`, so the narrowing is written into
it in place, with this ADR as the reason. Nothing else in C-0007 moves, and nothing legal under
it becomes illegal.

## Context

**ADR-0092 decision 3 named the trigger and the trigger has fired.** It recorded a second in-crate
Rust JWKS verifier as debt, and said extracting *"a `lib-auth` Rust sibling waits for the third
consumer, and is recorded as debt rather than built speculatively."* There are now three:

| Consumer | Where | How it got there |
| --- | --- | --- |
| `platform-identity` | `src/auth.rs` | the first, written for the login plane |
| `tool-docket` | `src/adapters/auth.rs` | the second (ADR-0092 D3), narrowed to an operator token |
| `platform-tenancy` | `src/auth.rs` | the third, **copied verbatim** from `platform-identity` |

The third copy names its own cost in its module header rather than hiding it, and names the reason
the copy was the honest move: the shared Rust library did not exist, and inventing one there would
have decided an open question — RFC-0009 Q5 — silently and in the wrong place.

**The extraction was blocked on a name, not on code.** `Paridyum/lib-auth` already exists and is
TypeScript: it publishes `@paridyum/auth`, which `web-paridyum`, `web-admin`, `web-dashboard`,
`web-hiring` and `console-docket` all depend on, and which ADR-0038, ADR-0045, ADR-0082 and
ADR-0092 D5 all refer to by repository name.

**One capability in two stacks is two repositories, by rule.** `class-registry.yaml` permits
`library` on `rust` and `typescript`, and its own header is explicit that the two are independent
axes: *"a repository picks one and declares it as `stack` in its manifest."* `manifest.schema.json`
makes `stack` a single string. So the estate cannot answer this with one repository, and two
repositories need two names inside one namespace — the GitHub organisation.

**C-0007 forbids the only discriminator available.** *"The `<name>` is a noun naming the
capability, never a technology and never an adjective. `svc-billing`, not `svc-rust-billing`."*

**RFC-0004 saw this coming and left it open**, in the section that produced the class registry:
*"`lib-` already exists and is Rust-specific by C-0002… Either `lib-` becomes stack-parameterised,
or a second prefix appears — an open question below rather than a decision here."*
`class-registry.yaml` answered half of it — *"It does not: `library` is the same concept either
way"*, so no second prefix — and this is the half it did not answer. It has been open since
2026-08-07 and it is now load-bearing.

## Decision

### 1. Where a class permits more than one stack, the repository name carries the stack

A repository whose class declares two or more `stacks` in `class-registry.yaml` may carry the stack
as a trailing segment of its name, and **does** carry it as soon as a second stack implements the
same capability:

```
lib-auth-rust          stack: rust        crate paridyum-auth
lib-auth-typescript    stack: typescript  package @paridyum/auth
```

The segment is the stack name **exactly as the registry declares it** — `rust`, `typescript`,
`dart` — so it is derived rather than chosen, per STD-0018's reasoning about derived names being
visibly wrong rather than plausibly wrong.

**Written against the registry's `stacks`, never against a prefix.** This is the trap C-0007
records in ADR-0003 and ADR-0010: *"a prefix glob in a locked artifact can never be extended to a
new prefix."* `library` is the class that needed this today. `mobile` is the other multi-stack
class — React Native on `typescript`, Flutter on `dart` — and it is governed by the same sentence
without that sentence being rewritten.

**Why this is not `svc-rust-billing`.** For a service the stack is invisible to every consumer: a
caller sends HTTP and cannot tell what answered. For a library the stack **is what may consume it
at all** — a Rust service cannot import an npm package, and no amount of capability-naming changes
that. The suffix names the set of repositories that can depend on this one, which is a fact about
the capability, not an implementation detail leaking. That distinction is the thing C-0007's rule
was missing, because at the time it was written no capability existed in two stacks.

### 2. The artifact the repository publishes drops the suffix

```
lib-auth-rust        →  crate   paridyum-auth
lib-auth-typescript  →  package @paridyum/auth
```

**Because the artifact's namespace is already per-stack.** Cargo resolves crates, npm resolves
packages, and neither can see the other, so the segment that disambiguates two repositories in one
GitHub organisation carries no information in either registry. `paridyum-auth-rust` names the
language twice, to a tool that cannot install anything else.

The derivation stays mechanical, which is C-0007's whole ethos — *"every rule here is mechanical so
that no agent has to exercise taste"*. It gains one step:

> **artifact name = repository name, minus the class prefix, minus the stack segment.**

**`manifest.yaml`'s `name` keeps the segment** — `lib-auth-rust`, `lib-auth-typescript` — because it
identifies the *repository*, which is the thing that collided, and `forge` builds one dependency
graph across both.

> **Found while applying this, and deliberately not fixed here:** `manifest.schema.json` describes
> `name` as *"the repository name with its class prefix removed: `svc-billing` is `billing`"*, and
> **every manifest in the estate carries the full repository name instead** — `platform-identity`,
> `tool-forge`, `web-admin`, `lib-ui`, all eleven. The convention in the code is unanimous and the
> schema description disagrees with all of it; the pattern permits both, so nothing ever failed. The
> two new manifests follow the convention rather than the description, and reconciling them is a
> separate change — noted rather than silently inherited.

**Two check specs in `check-registry.yaml` are corrected to say this**:
`cargo.package_name_matches_repo` and `repo.name_matches_scheme`. Neither is implemented, so this
is a specification fix rather than a behaviour change — and it is made now rather than left,
because the registry's own history is of specs that drifted and were then implemented literally
(`repo.name_matches_scheme` pointed at C-0002's prefix table long after C-0007 replaced it, and an
implementation obeying it would have rejected every `web-`, `sdk-` and `mobile-` repository).

### 3. `lib-auth` is renamed `lib-auth-typescript`, in one change

Per STD-0022: the directory, the GitHub repository, `manifest.yaml`, `package.json`'s `repository`,
the README and SOP headers, `CODEOWNERS`, and the live references in ADR-0038, ADR-0045, ADR-0082
and ADR-0092 D5. A rename is not done while the old name survives anywhere.

**`@paridyum/auth` does not change** (decision 2), so no consumer's imports or dependency
specifiers change. That is what holds the blast radius of this rename to the repository's own files
and the knowledge references, and it is a consequence of decision 2 rather than a happy accident.

### 4. `lib-auth-rust` holds the mechanism, and the products keep the policy

What the crate carries — the part all three copies hold identically today:

- bearer-token extraction from an `Authorization` header value;
- RS256 verification against a `kid`-keyed key set, and JWKS **parsing**;
- the issuer and audience pins ADR-0047 D5 and ADR-0070 D4 require, including the explicit
  project-id membership test that `aud` needs because Zitadel also puts the client's own id there;
- the organization claim, the project-roles claim, and the plane the issuer selects (RFC-0012 D2).

What stays in the consumer, each because it is policy and not mechanism (ANTI-0004):

- **the HTTP error body** — `spec-identity`'s closed `code` enum is a product contract, and a
  library that owned it would be deciding the wire form of every product's failures;
- **which role an operation requires** — `operator` is `tool-docket`'s rule (ADR-0092 D1);
- **the tenant uuid derivation** from an org id (ADR-0082) — `tool-docket`'s domain owns it;
- **fetching** the JWKS — a startup concern of a deployable. The library parses; the service
  fetches. This is also what keeps the crate free of `tokio`, `reqwest` and `axum`.

This is RFC-0009 §6's split — *the mechanism has no domain opinion; the policy lives in the context
that owns it* — applied to authentication rather than to tenancy. **If `lib-auth-rust` ever grows a
role name, an error code or a table name, it has crossed the line.**

### 5. It is consumed as a git dependency, pinned to a tag

No private cargo registry exists and every crate in the estate is `publish = false`, so a tag is
the pin available today: `git = "ssh://git@github.com/Paridyum/lib-auth-rust.git", tag = "v0.1.0"`.
Exact and current, per ADR-0053. STD-0019 §7's argument for `lib-ui` applies here unchanged and for
the same reason: a security mechanism that changes underneath a service without that service's
commit changing is divergence in the other direction.

## Consequences

**Gained.**

- One token contract for three Rust consumers, in one place, with one test suite. A change to the
  audience rule is now one edit rather than three, and the third copy's header stops being true.
- ADR-0092 D3's debt closes, on the trigger it named rather than on someone noticing.
- The next capability that exists in two stacks has a name available. That is `lib-tenancy`, and
  the naming half of RFC-0009 Q5 no longer blocks it.

**Accepted costs.**

- **Two implementations of one security mechanism survive**, and they can now skew as *libraries*
  rather than as copies. ANTI-0004's version-skew argument lands on the estate's authentication
  boundary and nothing detects a divergence — no shared token fixtures, no cross-stack test. This
  is a smaller version of the same risk rather than its removal: three copies became two libraries.
- **The suffix reads as the thing C-0007 forbids**, and every reader will have to be told why it is
  not. That is the cost of an exception to a mechanical rule, and it is why the exception is written
  into C-0007 itself rather than left in an ADR nobody reaches from the naming constitution.
- **GitHub keeps redirecting the old repository path after the rename.** So a stale reference to
  `Paridyum/lib-auth` keeps resolving, and STD-0022's *"not done while the old name survives"*
  cannot be verified by watching something break. The grep is the only evidence available.
- **Neither decision is enforced yet.** `repo.name_matches_scheme` and
  `cargo.package_name_matches_repo` are named in `enforced_by` and both are still awaiting
  `tool-forge` — `scripts/check-knowledge.sh` prints how many are on every run — so until they
  exist this is upheld at review. Stated rather than omitted,
  per ADR-0002: a declared check is a specification, not enforcement, and this ADR stays `accepted`
  rather than `locked` for exactly that reason.

**Rejected alternatives.**

- **Suffix the artifact too** — `paridyum-auth-rust`, `@paridyum/auth-typescript`. Perfectly
  mechanical, no exception anywhere, and rejected because it renames an npm package five
  repositories depend on in order to encode a fact npm already knows.
- **One repository holding both stacks.** The most truthful answer to *"is this one library or
  two?"*, and rejected on cost: `manifest.schema.json` makes `stack` a single string and
  `class-registry.yaml` makes one stack per repository a rule, so fixing a name would mean changing
  both — and the result is one repository with two CI workflows, two release cadences and one
  version number that means different things to two consumers.
- **A different capability noun for each** — `lib-auth` and `lib-principal`. Rejected: the
  capability is the same one, so the second name would be a lie told to a namespace, and STD-0018's
  argument applies — a plausible-sounding name that is wrong is worse than an ugly one that is
  checkable.
- **Rename the Rust side instead and leave `lib-auth` as TypeScript.** The cheaper rename by far,
  and rejected because it leaves the collision unsolved for the next capability and makes the pair
  asymmetric: one name would say nothing about its stack while its sibling says everything.
- **Do not extract at all**, on ANTI-0004's own advice that *"duplication is often the cheaper
  mistake."* Rejected because of what is duplicated: a change landed in two of the three copies is
  a hole at the authentication boundary that no test in any of the three repositories would show.

## What this does not decide

- **RFC-0009 Q5 / T-1.1 / T-3.5 — whether the tenancy mechanism is one library or one per stack.**
  This supplies the name that answer will need and nothing else. The session-context half of
  `@paridyum/auth` — `withSessionContext` and `assertAccount`, which RFC-0009 §6 assigns to
  `lib-tenancy` and which nothing imports yet — stays where it is until that question is answered
  on its merits.
- **Whether `lib-auth-rust` refreshes its keys on an unknown `kid`.** Both platform copies fetch
  the JWKS once at startup; the TypeScript verifier refetches through `jose`. The Rust behaviour is
  carried over unchanged rather than corrected while being moved, for the reason
  `class-registry.yaml` records about ADR-0010's `ci: false`: changing behaviour while moving it is
  how detail gets lost. It is recorded as debt in the new repository.
- **Which stack a new library starts in.** A library exists in a second stack when a second stack
  needs it, not in anticipation — STD-0019's rule for components, unchanged.

## Related

- **ADR-0092 D3** — the trigger this fires, and the second verifier that recorded it.
- **ADR-0047 D5**, **ADR-0070 D4**, **RFC-0012 D1/D2** — what the verifier must check, and why
  every product validates locally.
- **C-0007** · **class-registry.yaml** — the naming grammar this narrows, and the registry the
  narrowing is written against.
- **RFC-0004** — where the open question was raised and half-answered.
- **RFC-0009 §6 / Q5** · **ANTI-0004** — mechanism versus policy, and the version-skew cost.
- **STD-0022** — the rename is complete or it is not done.
- **ADR-0053** · **STD-0019 §7** — a pin is exact and current.
