---
id: RFC-0003
title: TypeScript as a second first-class stack
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, library, web, mobile]
applies_to: ["*"]
tags: [typescript, stack, supply-chain, npm, pnpm]
breaking: true
resolved_by: ADR-0046
enforced_by: []
---

# RFC-0003 — TypeScript as a second first-class stack

## Status

**PROPOSED** as of 2026-08-07. **Feedback deadline: 2026-09-14.** Stage 2 of ADR-0013.

Nothing here is adopted, and nothing has been built — deliberately, unlike RFC-0002. A
second language stack is a larger commitment than anything decided so far and the
decomposition should be reviewed before a line of it exists.

**Narrowed to the stack axis on 2026-08-07, and its Next.js content moved to RFC-0008.**
`class-registry.yaml` models *class* and *stack* as independent axes, and TypeScript is a
stack: it binds `library`, `web` and `mobile` alike, because a shared package, a Next.js
application and a React Native app are all governed by the same language rules. Next.js is a
**framework** on that stack and binds `web` alone. One RFC covering both conflated two scopes —
the exact axis-conflation C-0005 and RFC-0004 exist to prevent — so the npm supply chain, the
compiler settings and the test runners stay here, and the React Server Components trust boundary
and the Kubernetes self-hosting costs move to RFC-0008. `scope` was also corrected: it named
`platform`, `service` and `spec`, none of which TypeScript binds, and omitted `web` and `mobile`,
which it does. That was written before the `web` and `mobile` classes existed.

**Superseded in sequencing by RFC-0004.** This RFC proposed adding a `web-` prefix and
paying the C-0002 supersession last, which is correct for one class and wrong for six.
RFC-0004 establishes that repository classes belong in a registry rather than a locked
table, and this RFC becomes its **first consumer** — the first class added through the
mechanism, which also proves the mechanism before anything depends on it. Everything below
about the TypeScript stack and the npm supply chain stands unchanged; only the ordering
moves. Read RFC-0004 first.

## Problem

The platform is Rust-shaped end to end, and a user interface cannot be written in it.

**23 of 112 checks are Rust-only** (`cargo.*`, `rust.*`, `module.*`, `clippy.*`,
`rustdoc.*`, `rustfmt.*`, `crate.*`). Three of five standards — STD-0001 testing,
STD-0003 lints, STD-0004 code organisation — are Rust in substance and scoped
`applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]`. The only service template is
`templates/rust-service/`, and the only reusable workflow is `rust-ci.yml`.

**None of that is wrong.** ADR-0005 decides *"Rust is the implementation language for all
platform services, business services, shared libraries and tools"* — and says nothing
about a frontend, because none was contemplated. The gap is an absence, not a conflict.

But it is a load-bearing absence: **C-0002 has no repository class a web application could
use**, so a Next.js app has no name, no scope value, no template, no CI, and no standard
that binds it.

### This is a re-adoption, not a first adoption

ADR-0009 records that the archived `pd-*` estate was *"predominantly Python with TypeScript
frontends, none matching C-0002."* TypeScript is not new to this organisation — it was
present, ungoverned, and archived along with everything else when the platform restarted
greenfield.

That is the argument for doing this deliberately now rather than letting it arrive. The
previous estate acquired TypeScript by drift and it ended up in the archive. The distinction
this time is not the language; it is whether it arrives with a class, a standard and a
template, or in whatever shape the first person to need a page happens to produce.

## What already applies without change

Worth establishing first, because it is most of the governance and it is language-neutral.

| Concern | Applies as-is |
| --- | --- |
| Identity, attribution, Conventional Commits | C-0003 — 4 checks |
| Protected operations, secrets, generated code | C-0004 — 5 checks |
| Knowledge integrity, immutability, scope | ADR-0002/0014/0015 |
| Change lifecycle, design before code | ADR-0013 |
| Decomposition, boundaries, ownership | ADR-0019, ANTI-0001…0006 |
| Documentation, `manifest.yaml` | STD-0002 — `applies_to: ["*"]` |
| Changelog | STD-0005 — 7 checks |
| Kubernetes, Helm, probes, limits | ADR-0003 |
| CI architecture, pinned SHAs, permissions | ADR-0010 |
| Secret scanning | ADR-0017 |
| **Accessibility and i18n** | **REQ-0009 — and this RFC fires its trigger** |

**89 of 112 checks are language-neutral.** The work is a second stack, not a second
platform.

### REQ-0009 is triggered by this

REQ-0009's trigger is *"the first user-facing interface is written."* Adopting this RFC is
the last moment before that becomes true. Its items — a stated WCAG level, `axe` in CI, no
hardcoded strings, RTL-safe layout — must be built into the template rather than mandated
afterwards, because retrofitting them is a rewrite of the view layer and building them in
is nearly free. That is the single strongest argument for deciding this before writing any
frontend code.

## What TypeScript needs that Rust does not

This is where "match the Rust infra" is the wrong target. Three risk classes have no Rust
analogue, and a stack that only mirrors `rust-ci.yml` would be **worse** than the Rust
infra, not equal to it. One is stack-level and stays here; two are framework-level and are
RFC-0008's.

### The npm supply chain is categorically more hostile than crates.io

`deny.toml` is sufficient for Rust. It is not a model for npm.

Research findings, 2026: **PackageGate** — six zero-days disclosed in January 2026
affecting npm, pnpm, vlt and Bun. Over **1.2 million malicious packages** catalogued.
Self-propagating worms are now a named category. The controls that actually distinguished
teams that survived 2026:

- **Lifecycle install scripts disabled.** `npm ci --ignore-scripts`. A post-install script
  is arbitrary code execution at install time, on developer machines and CI runners both.
  R### Moved to RFC-0008

Two further risk classes were investigated here and belong to the framework rather than the
stack, so they now live in RFC-0008: **the React Server Components server/client trust
boundary**, which is a security boundary enforced by a directive string and has no Rust
analogue, and **the specific costs of self-hosting Next.js on Kubernetes** under ADR-0003. Both
findings stand unchanged; only their scope narrowed from every TypeScript repository to `web-*`.

 from day one. Next.js's best-supported host is Vercel. They are
reconcilable, and the reconciliation is not free:

- **`output: "standalone"`** produces a self-contained bundle and a single `node server.js`.
  This is the supported self-hosting path and must be set from the first commit.
- **`sharp` is required** for server-side image optimisation. Without it Next falls back to
  a much slower pure-JS path or fails outright — and it is a native dependency, so the
  Dockerfile must be multi-stage and architecture-aware.
- **ISR needs a cache shared across replicas.** The default is per-instance, so with more
  than one pod, users see different content depending on which pod answers. This is the
  Kubernetes-specific trap and it is silent.
- Health checks, resource limits and HPA are already required by ADR-0003 and apply
  unchanged.

## Proposed stack

Established from the 2026 baseline rather than preference.

| Concern | Choice | Rejected |
| --- | --- | --- |
| Language | TypeScript, `strict: true` + `noUncheckedIndexedAccess` | JavaScript, loose TS |
| Package manager | **pnpm** | npm, yarn, bun — see §1 |
| Module resolution | `"bundler"`, `isolatedModules: true` | node10/node16 |
| Unit / integration | **Vitest** + React Testing Library | Jest |
| End-to-end | **Playwright** | Cypress |
| Lint | ESLint flat config + `@typescript-eslint`, `vitest` plugins | tslint, no lint |
| Accessibility | `axe` in CI, per REQ-0009 | manual only |
| Validation | a schema validator at every trust boundary | hand-rolled checks |

Vitest over Jest because it shares Vite's configuration, parallelises by default, and is what
the Next.js team's own testing documentation now uses. Playwright over Cypress because async
Server Components need a real browser and a real network. **Framework choice is RFC-0008's**,
along with the `@next/next` ESLint plugin that only a `web-*` repository needs.

## Verified baseline, 2026-08-08

The stack table above names tools without versions, which is the part that decays. Re-checked
against primary sources; two findings below **change the proposal** rather than confirming it.

| Concern | Verified pin | Note |
| --- | --- | --- |
| Node | **24 (Active LTS**, EOL 2028-04-30) | 22 is Maintenance (EOL 2027-04-30); 26 becomes Active LTS 2026-10-28 |
| TypeScript | **6.0** (2026-03-23) — *not 7* | see below; 7.0 shipped 2026-07-08 |
| Package manager | **pnpm 11** (2026-04-28) | security defaults changed; see below |
| Lint | **ESLint 9+, flat config** | oxlint/Biome are faster and thinner — not primary |
| Unit | **Vitest 4** | browser mode stable since 4.0 |
| E2E | **Playwright** | mandatory, not preferred — see RFC-0008 |

### TypeScript 7 is adoptable and must not be adopted yet

TypeScript 7.0 shipped 2026-07-08 as the Go-native rewrite, 8–12× faster by Microsoft's
measurement. **It ships without a public compiler API.** `typescript-eslint`'s type-aware
rules are built directly against that API, and ESLint's own repositories are blocked on
`typescript-eslint` gaining TS 7 support. A stable API is expected in **7.1, around October
2026**; `@typescript/typescript6` exists as a shim that re-exports the 6.0 API for tooling.

So the fast compiler costs every type-aware lint rule, and type-aware rules are the entire
reason this RFC prefers ESLint over a syntactic linter. **Pin TypeScript 6.0** — the last
JavaScript-based release, 2026-03-23, and the last with a public API — and treat TS 7 as a
dated migration rather than a default.

This is the kind of finding that only appears on inspection. "Use the latest TypeScript" is
the obvious instruction, it is wrong here, and an agent generating a `package.json` would get
it wrong by doing the reasonable thing. Set `--stableTypeOrdering` in TS 6 to match 7.0's
behaviour and reduce the eventual diff.

### The npm supply chain got worse while this RFC sat proposed

§1 above cites PackageGate and 1.2M malicious packages. Both stand. What changed:

**Shai-Hulud resurfaced on 2026-08-04 — four days ago.** The CHAINDROP worm compromised
**1,300+ package versions** across ~2 billion monthly downloads, poisoning `keyv`,
`cacheable`, `flat-cache` (~565M weekly) and `file-entry-cache` (~557M weekly). It steals npm
credentials and republishes every package the stolen account can publish, which is why the
blast radius is a maintainer's whole portfolio rather than one package.

**`flat-cache` and `file-entry-cache` are ESLint's own caching dependencies.** The compromise
was not adjacent to this proposal — it was inside the toolchain it proposes, in the same week
it was being decided. That is the argument for the quarantine window in open question 5, and
it answers it: the window belongs in resolution, not CI.

pnpm 11 now ships the relevant defence **on by default**, which materially reduces the work:

| Setting | Default in pnpm 11 | Effect |
| --- | --- | --- |
| `minimumReleaseAge` | **`1440`** (24h) | refuses versions published in the last day |
| `blockExoticSubdeps` | **`true`** | no transitive git/tarball sources |
| build scripts | **off** unless in `allowBuilds` | lifecycle scripts are opt-in per package |
| `trustPolicy: no-downgrade` | opt-in | blocks installs where trust level regressed |

A 24-hour window is not arbitrary: Shai-Hulud was detected in ~12 hours and the 2025
`debug`/`chalk` compromise in ~2.5, so one day catches both. Adopt `no-downgrade` explicitly —
it is the one that is *not* a default — and record `allowBuilds` per package rather than
reaching for `dangerouslyAllowAllBuilds`, whose name is the documentation.

*(An earlier draft of this section named `minimumReleaseAgeStrict`. It appears in secondary
write-ups and not in pnpm's own configuration docs, so it is dropped rather than repeated —
an unverified setting name in a standard produces a config that silently does nothing.)*

### Publishing is already decided by the registry

Relevant to the `library` and `sdk` classes, and not previously recorded: **npm revoked all
classic tokens permanently on 2026-02-03.** The supported path is OIDC **trusted publishing**,
which issues short-lived credentials from CI and **generates provenance attestations
automatically**, with no `--provenance` flag and no long-lived token in a secret store.

This removes a decision from REQ-0014 rather than adding one: for anything published to npm,
provenance is not a thing to design, it is a thing to switch on — and the alternative was
withdrawn by the registry.

## Options

### A. One repo class, `web-`, and TypeScript standards alongside the Rust ones

A new C-0002 prefix `web-`, a `web` scope value, `STD-000n` for TypeScript testing/lints/
organisation mirroring STD-0001/0003/0004, a `templates/web-app/` and a `nextjs-ci.yml`.

Honest and correctly scoped: the Rust standards already carry
`applies_to: ["platform-*", "svc-*", "lib-*", "tool-*"]`, so a `web-*` repository is not
bound by them and no conflict arises.

**Cost: C-0002 must be superseded.** It is locked, its prefix table is the thing changing,
and supersession here is whole-artifact — the replacement restates all of C-0002's naming
rules for repositories, crates, APIs, databases, events, Kubernetes and environments.
That is the single largest cost in this RFC and it is unavoidable under any option that
adds a prefix.

### B. Reuse `svc-` for web applications

No supersession, no new prefix.

**Rejected — it is broken, not merely inelegant.** STD-0001, STD-0003 and STD-0004 all list
`svc-*` in `applies_to`. A TypeScript repository named `svc-web` would be bound by
`cargo.toolchain_file_present`, `clippy.clean_with_all_targets_all_features` and
`module.no_orphan_source_files`, none of which it can satisfy. That is precisely the
permanently-red-with-no-legal-fix condition ADR-0015 rejected fail-safe-to-everything for.

It is also wrong semantically: C-0002 defines a service as one bounded context, and a
frontend is a presentation layer over several.

### C. A separate frontend organisation or repository conventions outside the knowledge base

Rejected on ADR-0001 and ADR-0008: a second source of truth for policy fragments
cross-referencing, and the whole enforcement ladder assumes one knowledge base vendored
everywhere.

### D. Defer until the first UI is actually needed

Defensible, and it is what REQ-0009 already does for accessibility specifically.

Rejected for the stack as a whole because the decisions that are expensive to reverse —
package manager, App Router, standalone output, the server/client discipline — are all made
in the first hour of the first frontend repository, by whoever happens to be creating it.
That is exactly the "plausible implementation of the wrong decomposition" ADR-0019 exists
to prevent.

## Recommendation

**Option A, sequenced so the expensive supersession happens once and last.**

1. **An `ANTI-` artifact for the RSC trust boundary** — Server Actions are unauthenticated
   endpoints; unstainted server data reaches the client. Costs nothing structural, is
   readable by an agent immediately, and is the highest-value item here.
2. **An ADR for the TypeScript stack** — pnpm with `--ignore-scripts` and a quarantine
   window, strict TS, Vitest, Playwright, ESLint flat config. Mirrors ADR-0005's shape.
3. **An ADR for self-hosting Next.js** — standalone output, `sharp`, shared ISR cache,
   multi-stage Dockerfile. Extends ADR-0003 rather than superseding it.
4. **TypeScript standards** for testing, lint and organisation, scoped `web-*` only.
5. **`templates/web-app/` and `nextjs-ci.yml`**, with `axe` and bundle budgets built in so
   REQ-0009 is satisfied by the scaffold rather than mandated as prose.
6. ~~**Supersede C-0002 last**~~ — **revised by RFC-0004.** Paying the supersession last is
   right for a single class and wrong once React Native, Flutter, SDKs and shared packages
   are foreseeable: every stack added before the mechanism exists must be migrated into it
   afterwards. RFC-0004 moves the class list into a registry first, and `web-` then becomes
   a registry entry rather than a constitutional amendment.

## Open questions

1. **Is `web-` the right prefix**, or should it distinguish `web-` from a `bff-` that owns
   server-side data access? One prefix is simpler; two match the ownership model better.
2. **Does a Next.js app get an `owns.tables` block?** If it talks to a database directly it
   is a service by ANTI-0001's rules, and probably should not.
3. **Node version policy.** Rust pins an exact toolchain; Node needs an equivalent
   (`.nvmrc` / `.tool-versions`) and an EOL policy the Rust side does not have.
   *Narrowed by the verified baseline: Node 24 is Active LTS to 2028-04-30, so the pin is
   settled. What is still open is the **policy** — Node 26 becomes Active LTS on 2026-10-28,
   about ten weeks out, so the first `web-*` repository will face an LTS transition almost
   immediately. Decide now whether the rule is "track Active LTS" or "stay one line behind";
   the Rust side has no equivalent because a Rust toolchain does not reach EOL underneath you.*
4. **Bundle budgets** — what number, and is exceeding it an error or a warning? A budget
   nobody set is a budget nobody enforces.
5. ~~**Does the quarantine window belong in CI or in the lockfile update process?**~~
   *Answered by the verified baseline. Neither: pnpm 11 enforces it at **resolution**, which
   is the third option this question did not have. `minimumReleaseAge: 1440` is on by default,
   so it applies on every install on every machine, blocks nothing unrelated, and is not
   bypassable by re-running CI. The remaining decision is only whether to widen it beyond a
   day, and the detection times argue no.*
6. **Vercel is not being used.** Should that be recorded as a decision with its costs — no
   preview deployments, no edge middleware, self-managed ISR — rather than an omission?

## Related

- ADR-0005 — Rust for services, libraries and tools; silent on frontend, hence this RFC
- C-0002 — the prefix table this must change, and the largest cost here
- ADR-0003 — Kubernetes from day one, which standalone output exists to satisfy
- ADR-0019, ANTI-0001…0006 — boundaries and ownership, which apply unchanged
- REQ-0009 — accessibility and i18n, whose trigger this RFC fires
- REQ-0010 — API evolution, which Server Actions are a form of
- ADR-0017 — secret scanning, which does not understand the client bundle
- RFC-0008 — Next.js as the web framework; the two risk classes moved out of here
