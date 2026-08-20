---
id: RFC-0008
title: Next.js as the web framework, and the trust boundary it introduces
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0046
scope: [web]
applies_to: ["web-*"]
tags: [nextjs, rsc, web, framework, trust-boundary, kubernetes, self-hosting]
breaking: false
enforced_by: []
---

# RFC-0008 — Next.js as the web framework, and the trust boundary it introduces

## Status

**PROPOSED** as of 2026-08-07. Split out of RFC-0003, which had covered TypeScript and Next.js
in one artifact.

The split is not tidiness. `class-registry.yaml` models **class** and **stack** as independent
axes, and the two subjects sit on different ones:

| | Binds | Governed by |
| --- | --- | --- |
| TypeScript — a **stack** | `library`, `web`, `mobile` | RFC-0003 |
| Next.js — a **framework** | `web` only | this RFC |

A shared package, a Next.js application and a React Native app are all bound by the same
language rules; only one of them has Server Actions. Governing both from one artifact would bind
`lib-*` and `mobile-*` repositories to rules about a framework they never load — the
axis-conflation C-0005 and RFC-0004 exist to prevent. `scope: [web]` and
`applies_to: ["web-*"]` is the whole point of the split.

Everything below is RFC-0003's original research, unchanged in substance and narrowed in scope.

## Framework choice

| Concern | Choice | Rejected |
| --- | --- | --- |
| Framework | Next.js, App Router, `output: "standalone"` | Pages Router, Remix, plain React SPA |
| Framework lint | `@next/next` ESLint plugin | generic TS lint only |

The rest of the stack — compiler settings, package manager, test runners, validation — is
RFC-0003's and applies here unchanged.

## The two risk classes this framework adds

Carried from RFC-0003, where they were sections 2 and 3 of *What TypeScript needs that Rust does
not*. Neither is a TypeScript property; both are Next.js properties, which is why they moved.

### 1. The server/client boundary is a security boundary with no Rust equivalent

This is the finding that most changes the shape of the proposal.

- **Server Actions are publicly callable HTTP endpoints.** Without an explicit check, every
  Server Action is **unauthenticated by default**. It does not look like an endpoint, which
  is exactly why it is forgotten.
- **React's Taint API is opt-in.** Forgetting to taint a value means credentials, tokens or
  private user data flow silently into the client bundle. There is no compile error.
- **Static analysis that does not understand `"use server"` / `"use client"` misses entire
  classes of data exposure.** A generic TypeScript linter will not see this.
- **CVE-2025-55182 "React2Shell"**: unauthenticated remote code execution in the RSC Flight
  protocol, **CVSS 10.0**, exploitable by a single HTTP request in a default configuration.
  The root cause is unsafe deserialisation — Flight's decoder implicitly expands object
  properties, so an attacker injects keys, pollutes prototypes, and reaches a server-side
  execution path. Exploitation was observed from 2025-12-05, initially red-team traffic and
  then coin-miner delivery.

  **Corrected 2026-08-08 after verification.** This was written as CVE-2025-66478, which is
  the Next.js-specific downstream advisory and was **rejected by NVD as a duplicate** of
  CVE-2025-55182. Next.js still tracks its own id, so both appear in the wild — but a
  scanner keyed on the rejected id alone matches nothing in NVD, and citing the downstream id
  as the root cause also hides that the flaw is React's, not the framework's. **Any RSC
  renderer is affected, not only Next.js**, which is a materially different blast radius and
  the single reason this correction is worth making. Cite `CVE-2025-55182 (React) /
  CVE-2025-66478 (Next.js)` together.

In Rust, "does this value cross a trust boundary" is largely a type-system question. In
RSC it is a convention enforced by a directive string at the top of a file. An agent
generating code will get this wrong, and nothing in the current governance would notice.

**This deserves its own `ANTI-` artifact and its own checks**, and it is the strongest
argument that a TS stack needs *more* governance than the Rust one, not a translation of it.

### 2. Self-hosting Next.js on Kubernetes has specific, non-obvious costs

ADR-0003 locks Kubernetes from day one. Next.js's best-supported host is Vercel. They are
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

## Verified baseline, 2026-08-08

Re-checked against primary sources before this RFC is decided, because every version and
severity figure below was written from memory and two of them were wrong.

| Fact | Verified value |
| --- | --- |
| Next.js current stable | **16.3.0**, released 2026-08-03 |
| RSC Flight RCE | **CVE-2025-55182** (React, CVSS 10.0) · CVE-2025-66478 (Next.js, NVD-rejected duplicate) |
| Middleware auth bypass | **CVE-2025-29927**, CVSS 9.1 |
| Async Server Components under Vitest | **Cannot be rendered** — Playwright is not optional |

### Middleware is not an authorisation boundary, and there is a CVE proving it

**CVE-2025-29927** (CVSS 9.1) let an attacker skip Next.js middleware entirely by sending an
`x-middleware-subrequest` header. Fixed in 12.3.5, 13.5.9, 14.2.25 and 15.2.3 — all far
behind current — so this is not a live exposure at 16.3. It is retained because the *design
lesson* outlives the patch: middleware runs at the edge for routing, a single header once
removed it from the request path, and any application that treated it as the auth check was
open by default.

This converts §1's Server Action finding from a convention into a rule with evidence:
**every Server Action authorises itself, regardless of what middleware did.** An
authorisation check that exists once, upstream, in a layer that can be bypassed by a header
is the shape of the failure — and it is exactly what a generated action will omit, because
the middleware "already handles auth" reads as true in every file except the one that
matters.

The pattern the ecosystem converged on is a **server-only Data Access Layer**: authentication,
authorisation and data access behind one module that returns minimal DTOs, so a component
cannot reach the database without passing the check. That is mechanically checkable — a
database import outside the DAL — in a way "is this value sensitive" is not, and it is a
better candidate for the `error`-severity check open question 2 asks for.

### Playwright is load-bearing, not a second opinion

Vitest 4 stabilised browser mode, but **cannot render async Server Components** — React's
async component support is not stable in the runner. So the component model this RFC adopts
is partly untestable by the unit runner, and E2E stops being the top of the pyramid and
becomes the *only* layer that exercises RSC at all. STD-0001's pyramid is written for Rust
and assumes the inverse ratio; a `web-*` repository cannot satisfy it as written. Decide the
ratio here rather than discovering it as a failing coverage gate.

## What this RFC still has to decide

Split out with its findings intact and its decisions open. Four questions:

1. **Does the RSC trust boundary get its own `ANTI-` artifact?** RFC-0003 concluded it deserves
   one and that conclusion is inherited, not re-argued. The open part is what it can enforce: a
   Server Action with no authorisation check is mechanically detectable, and "is this value
   sensitive" is not — the same shape as ANTI-0004, where the rule is upheld by review and the
   check is an honest proxy.
2. **Which checks, and can any of them be `error`?** `"use server"` / `"use client"` are
   directive strings, so the checks need a parser that understands them; a generic TypeScript
   linter is blind here, which is the finding. An unparsed check that reports success would be
   the defect class the ADR-0014 audit found four times.
3. **Does ISR get used at all?** The shared-cache requirement is the one Kubernetes trap that is
   silent — with more than one pod, users see different content depending on which pod answers.
   Declining ISR entirely is a legitimate answer and cheaper than operating a shared cache
   correctly. ADR-0020 binds `web` because a standalone Next.js build is a normal workload.
4. **Does `sharp` justify a multi-stage architecture-aware Dockerfile in the template**, or is
   image optimisation disabled and delegated to the CDN? The second is less capable and much
   less to get wrong.

## Alternatives rejected

Carried from RFC-0003's framework row, which listed them in a table cell without the reasoning.
The check that surfaced this omission is `rfc.has_alternatives_section`, and it was right: a
table cell reading "Rejected: Remix, Pages Router, plain React SPA" records the outcome and
none of the argument.

**Pages Router.** The stable, better-documented half of Next.js, and it does not have Server
Components — so it would avoid every hazard in section 1 above. Rejected because it is the
framework's deprecated path: new capability lands in the App Router, and adopting a
maintenance-mode router to dodge a trust boundary means paying for the migration later with
interest. The boundary is a real cost and this is the wrong way to avoid it.

**Remix, or React Router in framework mode.** A genuinely defensible choice with a simpler and
more explicit data-loading model, and no directive-string trust boundary. Rejected on ecosystem
depth rather than design: under BET-0003 the application layer is **generated**, and generation
quality tracks how much idiomatic public code exists for a framework. Next.js has more of it by
a wide margin. This is the closest call here, and it would flip if generation quality turned out
not to depend on corpus size.

**A plain React single-page application on a static host.** No server framework, no RSC, no
trust boundary, and it discards server rendering — which the tenant-facing pages need for both
first-paint and search visibility, since a tenant's product is on their own domain and their
customers arrive cold.

**No framework: server-rendered HTML from the Rust services.** Would keep one language end to
end and satisfy ADR-0005 without a second stack at all. Rejected in RFC-0003 rather than here —
that is the stack question, not the framework question, and RFC-0003 records it as option D.

## Costs of adopting Next.js specifically

1. **The trust boundary is a convention, not a type.** In Rust, whether a value crosses a trust
   boundary is largely a type-system question. In RSC it is a directive string at the top of a
   file. Under BET-0003 the application layer is **generated**, and a generated Server Action
   with no authorisation check is exactly the subtly-wrong output that bet warns about — it looks
   right and it is publicly callable. This is the strongest argument that the web stack needs
   *more* governance than the Rust one rather than a translation of it.
2. **The framework's best-supported host is not ours.** ADR-0003 locks Kubernetes; Next.js is
   optimised for Vercel. They reconcile, and `output: "standalone"`, `sharp` and the ISR cache
   are the price. Each must be set from the first commit, because each is a silent degradation
   rather than a build failure.
3. **A framework CVE is our CVE.** CVE-2025-55182 / CVE-2025-66478 was remote code execution in the RSC protocol
   at CVSS 10.0. Adopting a framework at this layer means its protocol vulnerabilities are ours
   to patch on their schedule, across every `web-*` repository at once.

## Related

- RFC-0003 — TypeScript as a stack; the axis this RFC is not on
- C-0005, RFC-0004 — class and stack as independent axes, which is why this split exists
- ADR-0003, ADR-0020 — Kubernetes as the deployment target; `web` is `deployable: true`
- REQ-0009 — accessibility and internationalisation, triggered by adopting a UI stack
- ANTI-0004 — the model for an anti-pattern whose check is an honest proxy
- BET-0003 — why a convention-enforced trust boundary is a generation risk
