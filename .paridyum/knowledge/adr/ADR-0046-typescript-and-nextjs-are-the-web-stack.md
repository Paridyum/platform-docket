---
id: ADR-0046
title: TypeScript is the second stack and Next.js is the web framework, with the RSC boundary named
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, web, mobile, library]
applies_to: ["web-*", "mobile-*", "lib-*"]
tags: [typescript, nextjs, web, stack, rsc, supply-chain, rfc-0003, rfc-0008]
breaking: false
enforced_by:
  - check: repo.name_matches_scheme
    severity: error
    rationale: >
      The `web` class and the `typescript` stack already exist in class-registry.yaml; this ADR
      is what makes them legitimate to use rather than merely declared. The prefix check is what
      keeps a TypeScript repository from arriving without a class, which ADR-0009 records as how
      the previous estate happened.
  - check: web.data_layer_modules_are_server_only
    severity: error
    rationale: >
      Decision 4's trust boundary has no runtime signal — a value crossing it is simply
      serialised to the browser. `server-only` converts the leak into a build error, and this
      is what notices a new data module that forgot to declare it.
  - check: web.server_actions_authorize
    severity: error
    rationale: >
      A Server Action is a public HTTP endpoint that looks like a function call, which is why
      the omission is invisible at the call site rather than merely easy to make.
  - check: ci.calls_shared_workflow
    severity: error
    rationale: >
      A second stack is the point at which "CI is a reusable workflow" stops being a
      convenience and becomes the only thing keeping two toolchains from diverging into two
      independently-maintained pipelines.
---

# ADR-0046 — TypeScript and Next.js

## Status

**ACCEPTED** as of 2026-08-08. **Resolves RFC-0003 and RFC-0008.**

## Context

Two RFCs have been open since 2026-08-07, and both are approaching ADR-0011's 30-day staleness
warning on **2026-09-06** — before their own stated feedback deadlines of 2026-09-14 and 09-15, a
mismatch RFC-0012 already records.

Neither is genuinely contested. RFC-0004 resolved the question RFC-0003 named as its blocker, and
`class-registry.yaml` already ships `web-` with `stacks: [typescript]` and `mobile-` with
`[typescript, dart]`. **The stack is declared and unratified**, which is the worst of both: a
repository could be created against it today with no ADR saying it was allowed.

BET-0004 raises the stakes. A catalogue of vertical applications means many web front-ends, so
the framework decision is made once and paid for many times.

## Decision

**1. TypeScript is a second first-class stack**, alongside Rust (ADR-0005). Not a scripting
convenience — a stack with its own standards, template and CI workflow, binding `web-`, `mobile-`
and TypeScript `lib-` repositories.

**2. Rust remains primary for services.** ADR-0005 is untouched: `platform-*` and `svc-*` are
`stacks: [rust]`. TypeScript exists for interfaces, and a service written in it would be drift
rather than a choice.

**3. Next.js is the web framework**, App Router, self-hosted via `output: "standalone"` — which
is why `web` is `deployable: true` and ADR-0020's charts and probes apply unchanged.

Two reasons beyond preference. It is what `platform-generation` would target if BET-0004's
trigger clause 1 ever fires and the factory gets built, so choosing it keeps that door open.
And **Zitadel's own reference login UI is a self-hostable Next.js application built on the
Session API** — the exact component RFC-0012 puts in `platform-identity`'s login plane. Adopting
a different framework would mean reimplementing the thing the vendor ships as an example.

**4. The RSC server/client boundary is a trust boundary and gets its own anti-pattern.** This is
the substantive risk and it has no Rust analogue. A React Server Component runs with server
credentials; a `"use client"` component does not. A value crossing that line is serialised to the
browser, and the mistake — passing a server-side object containing a secret, a token, or another
tenant's row into a client component — looks like an ordinary prop.

Server Actions compound it: they are **publicly callable HTTP endpoints, unauthenticated by
default**, that look like function calls. RFC-0008 cites a CVSS 10.0 advisory on exactly this
surface. Every Server Action authenticates and authorises as if it were a route handler, because
it is one.

**5. A server-only data access layer.** Database and Zitadel access lives in modules marked
server-only, never imported from a client component. Authorization decisions happen there, not in
a component.

**6. Dependency intake is guarded.** `pnpm` with install scripts disabled by default and a
quarantine window on new versions. The npm supply chain is the largest attack surface either
stack brings, and it is the one place TypeScript is materially riskier than Rust.

**7. Accessibility and internationalisation land in the template, not in prose.** Adopting this
fires REQ-0009's trigger. Its items 2, 6 and 7 — automated axe checks, no hardcoded strings,
translation key standards — go into `template-web` as scaffolding, which is the difference
between a rule and a default. STD-0010 already governs the locale-neutral half.

## Consequences

**Accepted costs**

1. **A second toolchain, permanently.** Two lint configurations, two test runners, two CI paths,
   two dependency ecosystems, two sets of upgrade work. ADR-0005 chose one language partly to
   avoid this; the cost is accepted because the alternative is a browser without a framework.

2. **The RSC boundary is guarded in the decidable half only.** Two checks are declared —
   `web.data_layer_modules_are_server_only` catches the leak vector, and
   `web.server_actions_authorize` catches the unauthenticated endpoint. Both are `tool-forge`'s
   and neither runs today, and **the general rule is not checkable at all**: "a value crossed the
   boundary that should not have" needs type-level flow analysis, and the attempt was not made
   rather than made badly. `web.server_actions_authorize` additionally needs `template-web` to
   establish a named authorization helper before it has anything to look for.
   Until then the highest-consequence rule here rests on review, and the mistake it prevents
   looks like an ordinary prop.

3. **Next.js moves fast and self-hosting is not its default path.** `output: "standalone"` is
   supported and less travelled than Vercel; upgrades will occasionally break it.

4. **npm supply chain risk is real and only partly mitigated.** Disabled install scripts and a
   quarantine window reduce it. They do not remove it, and REQ-0014 owns what is missing.

5. **REQ-0009 fires now.** Accessibility and i18n become live obligations on the first component
   rather than deferred, which is correct and is work.

**Gained**

- Two RFCs close before they go stale, and the declared-but-unratified stack becomes legitimate.
- `template-web` becomes writable, which is where a11y and i18n stop being prose.
- The identity login UI has a reference implementation in the chosen framework rather than a
  rewrite.

**Rejected alternatives**

- **Remix.** RFC-0008 calls this the closest call, and the argument that decided it was corpus
  size for generation — which BET-0003's withdrawal weakened. It remains defensible, and Zitadel's
  reference login being Next.js is now the stronger reason.
- **A plain SPA with a Rust API.** Removes the RSC boundary entirely and with it server-side
  rendering, SEO, and the streaming model the login UI depends on. It also means writing the login
  UI from scratch.
- **Defer until the first screen.** What REQ-0009 already warns against, and the reason it exists:
  the framework decision has to precede the template, and the template is what makes the rules
  defaults.

## Related

- RFC-0003, RFC-0008 — resolved by this
- ADR-0005 — Rust as primary; unchanged for services
- ADR-0020 — Kubernetes, which binds `web` because it is `deployable: true`
- ADR-0021 — the reusable CI workflow a second stack makes load-bearing
- REQ-0009 — accessibility and i18n, whose trigger this fires
- REQ-0014 — supply-chain integrity, which owns what decision 6 does not cover
- RFC-0012 — the login plane whose reference implementation is Next.js
- STD-0010 — the locale-neutral half of i18n, already enforced
- BET-0004 — the catalogue that makes this decision recur per product
