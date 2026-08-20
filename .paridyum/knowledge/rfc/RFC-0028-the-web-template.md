---
id: RFC-0028
title: The web template — what template-web contains, and the five decisions it needs first
type: rfc
status: proposed
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, web, template]
applies_to: ["template-web", "web-*"]
tags: [web, nextjs, typescript, template, a11y, i18n, csp, cache-components, tenancy, req-0009, adr-0046, rfc-0016]
breaking: false
enforced_by: []
sources:
  - url: https://nextjs.org/docs/app/guides/multi-tenant
    title: How to build multi-tenant apps in Next.js — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      THE ABSENCE IS THE FACT. The framework's entire multi-tenancy guidance is a pointer to a
      hosted starter template, last updated 2025-04-15. There is no upstream pattern for nested
      tenancy, so ADR-0024's shape is ours to define and ours to maintain. Recorded because a
      reader would reasonably assume a guide of that title contains guidance.
  - url: https://nextjs.org/docs/app/guides/authentication-with-cache-components
    title: How to implement authentication with Cache Components — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Cache Components (cacheComponents, "use cache", "use cache: private",
      "use cache: remote", cacheLife, cacheTag, updateTag) is the caching model in Next.js 16,
      and cache keys and tags are stored in plain text and are not hashed. This is the fact
      ANTI-0009 rests on.
  - url: https://nextjs.org/docs/app/guides/authentication
    title: How to implement authentication in Next.js — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Names the Data Access Layer / verifySession() / DTO pattern as the recommended shape, which
      is ADR-0046 decision 5 arrived at independently. States that proxy-level checks are
      OPTIMISTIC ONLY and must not perform data access, and that a layout does not control
      whether the rest of the route renders — route segments and parallel slots are rendered by
      the router regardless.
  - url: https://nextjs.org/docs/app/guides/data-security
    title: How to think about data security in Next.js — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Server Actions are reachable by direct POST whether or not they are referenced in the UI;
      action ids are encrypted and rotate at most every 14 days; unused actions are stripped.
      Documents experimental.taint (taintObjectReference, taintUniqueValue) as a partial runtime
      mitigation for the RSC boundary, and states that only the Data Access Layer should read
      process.env.
  - url: https://nextjs.org/docs/app/guides/content-security-policy
    title: How to set a Content Security Policy for your Next.js application — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      A nonce-based CSP requires every page to render dynamically: static optimisation and ISR are
      disabled, CDN caching is lost, and Partial Prerendering is INCOMPATIBLE because static shell
      scripts cannot access the nonce. experimental.sri offers hash-based CSP that preserves
      static generation, and is experimental and App-Router-only. This is decision 1 below.
  - url: https://nextjs.org/docs/app/guides/self-hosting
    title: How to self-host your Next.js application — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Multi-replica correctness requires NEXT_SERVER_ACTIONS_ENCRYPTION_KEY stable across
      instances, deploymentId for version-skew protection, a consistent generateBuildId, a shared
      cacheHandler with cacheMaxMemorySize 0 and refreshTags() for cross-instance tag
      coordination, graceful SIGTERM drain of 10-30s for after() callbacks, and a proxy that does
      not buffer or streaming degrades silently. Also records that sharp on glibc needs
      memory-allocator configuration.
  - url: https://nextjs.org/docs/app/guides/internationalization
    title: Internationalization — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      App Router i18n is app/[lang]/ plus an application-owned locales array in proxy, not
      framework config. next/root-params exposes the locale to any Server Component without prop
      drilling, but its getters DO NOT RUN in Client Components, Server Actions or Route Handlers.
      The documented negotiation pattern redirects on Accept-Language, which ADR-0034 decision 3
      permits only as a seed for a caller with no persisted setting.
  - url: https://nextjs.org/docs/app/guides/multi-zones
    title: How to build micro-frontends using multi-zones and Next.js — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      Independent Next.js applications may serve one domain via per-zone assetPrefix and rewrites.
      Cross-zone links must use an anchor rather than Link, paths must be unique per zone, and
      serverActions.allowedOrigins becomes mandatory. This is the mechanism for separating a
      marketing surface from an application surface without a second framework.
  - url: https://nextjs.org/docs/app/guides/ai-agents
    title: How to set up your Next.js project for AI coding agents — Next.js documentation (v16.3.1)
    retrieved: 2026-08-18
    supports: >
      On 16.3 and later, next dev auto-generates and upserts AGENTS.md and CLAUDE.md at the
      project root inside a managed block when an agent is detected, unless agentRules is set to
      false. Version-matched docs ship at node_modules/next/dist/docs/ and an MCP server is
      exposed at /_next/mcp. This is decision 2 below.
  - url: https://nextjs.org/blog/july-2026-security-release
    title: July 2026 Security Release — Next.js blog
    retrieved: 2026-08-18
    supports: >
      Nine CVEs patched in July 2026 covering SSRF, middleware/proxy bypass, denial of service,
      cache confusion and disclosure of internal Server Function identifiers, following thirteen
      advisories in May 2026. The advisory directs production users to the 16.2.x Active LTS or
      15.5.x Maintenance LTS branch rather than the newest minor. CVE-2026-64642 affects App
      Router applications on Turbopack with exactly one entry in the i18n locales array, in which
      configuration proxy-level checks can be bypassed entirely.
  - url: https://socket.dev/blog/pnpm-11-adds-new-supply-chain-protection-defaults
    title: pnpm 11 Adds Supply Chain Protection Defaults for Minimum Release Age — Socket
    retrieved: 2026-08-18
    supports: >
      pnpm 11 ships minimumReleaseAge defaulting to 1440 minutes, so ADR-0046 decision 6's
      quarantine window is now a default rather than a configuration. It also replaced five
      settings — onlyBuiltDependencies, onlyBuiltDependenciesFile, neverBuiltDependencies,
      ignoredBuiltDependencies and ignoreDepScripts — with a single allowBuilds map, so the
      ADR's policy is intact while its vocabulary is stale.
  - url: https://nodejs.org/en/blog/announcements/evolving-the-nodejs-release-schedule
    title: Evolving the Node.js Release Schedule — Node.js
    retrieved: 2026-08-18
    supports: >
      From October 2026 Node.js moves from two major releases a year to one, version numbers
      align with the calendar year, and every release becomes LTS. Node 24 is Active LTS as of
      August 2026, 22 is Maintenance, and 26 is Current pending its October LTS transition. The
      cadence change is why the TypeScript pin's advance policy is worth deciding now rather than
      after (ADR-0048 open question 3, TASKS T-2.12).
  - url: https://zitadel.com/blog/how-to-self-host-zitadel-typescript-login-ui
    title: How to self-host the Zitadel TypeScript login UI — ZITADEL
    retrieved: 2026-08-18
    supports: >
      ADR-0046's load-bearing reason survives verification: the login is MIT-licensed, built on
      Next.js and the Session/User V2 APIs, and documented for self-hosting. The qualification
      the ADR does not carry is that the repository and packages are described as beta and
      subject to change.
---

# RFC-0028 — The web template

## Status

**PROPOSED** as of 2026-08-18. Resolves nothing on its own; it exists to be argued with and then
to become an ADR, or to be split into several.

Two artifacts are blocked on this one. REQ-0009 states that *"there is no interface, no framework
chosen, and no `template-web` to put the tooling in"* and that *"the right moment is the creation
of the first frontend template."* RFC-0016's open question 2 is *"does `template-web` exist to
receive decision 3?"* — and the answer is still no.

## The question

ADR-0046 chose the stack. It did not say what the scaffold contains, and the only precedent —
`templates/rust-service/` — contains nine configuration files and one SQL grant, and **no source
at all**.

That precedent cannot be followed here, because four obligations already written down require
source rather than configuration:

| Obligation | Why configuration cannot carry it |
| --- | --- |
| `web.server_actions_authorize` | *"Heuristic by necessity — 'an authorization call' is a project convention, not a language construct — so it needs a named helper to look for, which `template-web` must establish before this can be implemented."* A helper is code. |
| ADR-0020 probes | `livenessProbe` and `readinessProbe` bind to health endpoints. Endpoints are route handlers. |
| ADR-0034 locale resolution | Six levels, four independent fields, and three documented library defaults that must be overridden. |
| RFC-0016 decision 3 | Static shell, batched request, no chaining, optimistic write. These are shapes, and a shape is copied from code or from nothing. |

So the question this RFC answers is not *"config or source"* but **how little source discharges
the obligations**, given that the `template` class is `ci: false` and every line in it is
therefore unbuilt and untested where it sits.

## What changed upstream since ADR-0046 was accepted

ADR-0046 was accepted on 2026-08-08. Seven facts dated after or unavailable at that time bear on
this scaffold, and two of them change what the ADR would have decided.

**1. Next.js runs LTS branches now.** The July advisory directs production users to 16.2.x Active
LTS, not the newest minor. ADR-0046 accepted cost 3 is *"Next.js moves fast and self-hosting is
not its default path; upgrades will occasionally break it."* An LTS branch is the direct
mitigation for the cost the ADR accepted without one. **Recommendation: pin the LTS branch.**

**2. `middleware` is `proxy`.** One `proxy.ts` per project, scoped by `config.matcher`. The
rename matters beyond spelling: CVE-2026-64642 is a proxy bypass, and the authentication guide is
now explicit that proxy-level checks are **optimistic only** and must not touch data.

**3. Cache Components is the caching model, and its keys are plain text.** This is the finding
with the largest blast radius and it has produced its own artifact — ANTI-0009. A cache key that
omits ADR-0024's second tenancy column serves one tenant's rendered output to another, below the
level at which ADR-0038's fail-closed session context operates.

**4. A nonce-based CSP is incompatible with RFC-0016's static shell.** Nonces force every page
dynamic, disable ISR, forfeit CDN caching, and are explicitly incompatible with Partial
Prerendering. RFC-0016 decision 3's first pattern is a cacheable static shell. One of the two has
to give, and neither artifact priced the collision. This is decision 1.

**5. `next dev` writes to `CLAUDE.md`.** On 16.3+ it auto-generates and upserts `AGENTS.md` and
`CLAUDE.md` inside a managed block whenever it detects an agent. In this estate `CLAUDE.md` is a
governance router and ADR-0007's hooks police what may be written to it. A dev server editing a
governance file is not a thing to discover during a debugging session. This is decision 2.

**6. pnpm 11 renamed ADR-0046 decision 6's mechanism.** `minimumReleaseAge` now defaults to one
day, so the quarantine window is shipped rather than configured, and `allowBuilds` replaced the
five settings the ADR's era knew. The *policy* is intact; the *vocabulary* is stale — which is
exactly the ADR-0048 rot pattern, and an argument for those names living in the template.

**7. Node's release model changes in October.** One major per year, calendar-aligned numbering,
every release LTS. `toolchain-registry.yaml` has no TypeScript entry at all, and ADR-0048's open
question 3 — when a pin advances — is unanswered. The upstream cadence is about to move under it.

Two facts held up under checking and are recorded because they were load-bearing: the DAL / DTO /
`server-only` shape is Next.js's own recommendation, arrived at independently of ADR-0046
decision 5; and Zitadel's Next.js login is real, MIT and self-hostable, with the qualification
that it is **beta and subject to change**, which ADR-0046 does not say.

One absence is also a finding. The framework's multi-tenancy guide is a pointer to a hosted
starter template, last updated 2025-04-15. **There is no upstream pattern for nested tenancy.**
BET-0004 makes many web front-ends inevitable, so this shape is ours to define once and maintain
forever, and the template is where it is defined or not at all.

## Options considered

Three shapes for the scaffold, plus the one that keeps being proposed. The obligations table
above is what decides between them, not preference.

### Option A — configuration only, following `template-rust-service`

Nine configuration files and no source, exactly mirroring the one precedent this estate has.

Rejected because the four obligations in the table cannot be discharged by configuration, and
because ADR-0046 decision 7's whole argument is that accessibility and internationalisation
*"go into `template-web` as scaffolding, which is the difference between a rule and a default."*
A configuration-only template returns every one of them to prose — the state REQ-0009 already
describes and calls a rewrite of the view layer to escape. It also leaves both
`web.server_actions_authorize` and ANTI-0009's candidate check permanently unimplementable,
because each needs a named helper to match and neither would get one.

This option is the most consistent with existing practice, which is why it deserves the first
hearing. It loses on the obligations, not on taste.

### Option B — configuration plus a thin reference application (recommended)

The configuration set, plus the smallest source that makes each convention checkable: a named
authorization helper, a tenancy-scoped cache-key constructor, one `server-only` data module, one
authorised Server Action, the locale chain, two locales, health endpoints, and one page carrying
RFC-0016's four patterns.

The argument against it is not weak. It is the first source in the estate that no CI builds
(open question 1), and every file is an upgrade liability on a framework that shipped
twenty-two advisories in three months. What contains that is derivation: twelve files, each
traceable to a written clause, so the surface grows only when an obligation does. A file that
cannot name its obligation is a file that does not belong in the template — which is a rule this
RFC can state and a later reviewer can apply.

### Option C — a full starter kit

Option B plus a design system, layout primitives, a component library and a complete
authentication flow. The fastest route to a real screen, and what a team without governance
would build.

Rejected on two counts. The `template` class is `ci: false`, so a large body of source would sit
unbuilt and untested where it lives, and the liability grows per file with nothing checking any
of it. And a design system inside the scaffold cannot be versioned independently of the
scaffold — it belongs in a `lib-` repository, which ADR-0046 already permits TypeScript to
occupy. The fastest demo and the cheapest estate are different objectives, and BET-0004 is a bet
on the second one.

### Option D — defer until the first `web-` repository needs it

Rejected once already, in ADR-0046's own alternatives: *"the framework decision has to precede
the template, and the template is what makes the rules defaults."* Deferring means repository
number one establishes the conventions by accident and repository number two inherits them by
copying, which is how ADR-0009 records the previous estate happening.

Recorded here rather than treated as settled, because it will be proposed again the first time
the template stands between someone and a deadline, and it will sound reasonable then.

## Proposal — what `template-web` contains

Every entry traces to a written obligation. Anything that does not trace is not in the template.

### Configuration

| File | Obligation |
| --- | --- |
| `package.json`, `.npmrc` — pnpm 11, explicit `minimumReleaseAge`, empty `allowBuilds` | ADR-0046 decision 6, REQ-0014 |
| `tsconfig.json` strict, eslint incl. `eslint-plugin-jsx-a11y`, prettier | ADR-0046, ADR-0021 `lint`, REQ-0009 item 2 |
| `vitest.config.ts` | TST-0001, TST-0002 at STD-0008's `pr` tier |
| `playwright.config.ts` with `@axe-core/playwright` | TST-0003, REQ-0018, REQ-0009 item 2 |
| `next.config.ts` — `output: "standalone"`, `cacheComponents`, `experimental.taint`, `serverActions.allowedOrigins` and `bodySizeLimit`, `deploymentId`, `generateBuildId`, `cacheHandler`, `cacheMaxMemorySize: 0` | ADR-0046 decisions 3 and 4, ADR-0020, RFC-0016 |
| `Dockerfile` — multi-stage, non-root, writable `.next`, sharp allocator note | ADR-0020 |
| `helm/` — probes, `terminationGracePeriodSeconds: 30`, ingress buffering disabled | `repo.has_helm_chart`, `helm.declares_probes`, ADR-0020, RFC-0016 |
| `.github/workflows/ci.yml` — ten-line caller of `nextjs-ci.yml@v1` | ADR-0021, `ci.calls_shared_workflow` |
| `manifest.yaml` **with no `slo` block**, README, ARCHITECTURE, CHANGELOG | STD-0002, REQ-0016 |

The absent `slo` block is deliberate and has precedent. REQ-0016's trigger already fired once on
`templates/rust-service/manifest.yaml`, which declared numbers *"derived from taste"*; the block
was removed from the template so numbers are chosen rather than inherited. RFC-0016 sharpens it
further — a budget depends on the distance between a user and their tenant's region, so no single
number covers a local and a remote tenant.

### Source — twelve files, each a check's target or a clause's shape

| File | Obligation |
| --- | --- |
| `lib/auth/dal.ts` — `verifySession()` in React `cache()`, `server-only`, DTO helpers | `web.server_actions_authorize` gets its named target; ADR-0046 decision 5 |
| `lib/cache/keys.ts` — the only constructor of cache keys and tags, tenancy-scoped | **ANTI-0009** |
| `lib/data/example.ts` — `import "server-only"`, one tainted value | `web.data_layer_modules_are_server_only`, ADR-0046 decision 4 |
| `app/actions/example.ts` — one Server Action authorising on every path, delegating to the DAL | the pattern the check matches |
| `proxy.ts` — optimistic checks only, no data access, `matcher` set | CVE-2026-64642; the auth guide's stated limit |
| `lib/locale/resolve.ts` — six levels, four fields independent, platform default declared | ADR-0034 decisions 1-4, GLO-0006 |
| `app/[lang]/` + `next/root-params`, with a second path for Server Actions | ADR-0034; root-params does not run in actions |
| `messages/` — **two locales, one right-to-left** | CVE-2026-64642 and REQ-0009 items 6-9 |
| layout on CSS logical properties | REQ-0009 item 9 |
| `app/healthz`, `app/readyz` route handlers | ADR-0020 probes |
| one page carrying RFC-0016's four patterns | RFC-0016 decision 3 |
| `app/global-error.tsx`, `app/global-not-found.tsx`, `useReportWebVitals` with no thresholds | REQ-0009, REQ-0016 |
| correlation-id propagation | ADR-0044, whose scope includes `web` |

Two locales, one of them right-to-left, is where a security fact and an accessibility requirement
happen to agree. CVE-2026-64642 fires on *exactly one* entry in the locales array; REQ-0009
item 9 wants right-to-left considered from layout rather than retrofitted. A template shipping
English only is both the vulnerable configuration and the expensive one.

### Not in the template

No `slo` block. No design system or component library — that is `lib-` class work and its own
change. No Zitadel integration; the login plane belongs to `platform-identity` and the upstream
packages are beta. No tenancy data access, which is blocked on T-1.1.

## The five decisions this RFC needs

1. **CSP: nonce, or `experimental.sri`?** Strict security posture against RFC-0016's static
   shell. They are mutually exclusive and both are defensible.
2. **`agentRules: false`, or manage the block?** Who owns `CLAUDE.md` in a `web-` repo.
3. **A `typescript` entry in `toolchain-registry.yaml`** — Node 24, pnpm 11, `floor_reason`, and
   an advance policy that survives October.
4. **`template-web` or `template-web-app`?** Seven references say the former;
   `class-registry.yaml` line 175 says the latter. C-0007 governs.
5. **The TypeScript action-choice table.** ADR-0021 states that *"equivalent choices for a new
   stack are fixed by that stack's ADR, not here"*, and ADR-0046 did not fix them. `nextjs-ci.yml`
   has no toolchain, cache or tool-install choices named.

Item 5 needs a home: an ADR supplementing ADR-0046, since an erratum may not change what an
artifact decides and ADR-0046 is not locked but is accepted and cited.

## What this RFC does not decide

**The marketing/application split.** Multi-Zones is the framework mechanism — per-zone
`assetPrefix`, `rewrites`, anchors rather than `Link` across zones, and mandatory
`allowedOrigins`. Separate subdomains avoid zones entirely and owe only `allowedOrigins`. Either
way it is two `web-` repositories and one framework, not two frameworks, and it is a separate
change from the template.

**REQ-0009 items 1 and 11.** WCAG 2.2 AA and a browser/device matrix. Adopting this template
fires REQ-0009's trigger and moves it from `deferred`, at which point both are live and neither
is decidable by a template.

## Open questions

**1. How is source in a `ci: false` repository verified?** `template-` is `stacks: [none]`,
`deployable: false`, `ci: false`. `template-rust-service` never met this because it holds no
source. The cheapest answer is indirect: `forge` scaffolds the template nightly and CI runs on
the *scaffolded* repository, which also tests the scaffolding rather than only the scaffold. The
alternative is giving `template-` a `ci` flag, which is a `class-registry.yaml` change and
therefore an ADR.

**2. Is T-1.1 blocking?** *"Does `lib-tenancy` become one library or one per stack?"* is marked
⛔ and observes that *"a Next.js interface needs the same guarantee in TypeScript."* The template
can ship without tenancy data access. The first real `web-` repository cannot, and ANTI-0009 has
just raised the cost of getting it wrong.

**3. Does the Next.js MCP server and the bundled version-matched documentation belong in the
template deliberately?** For an estate whose agents are governed by ADR-0007 and pinned to an LTS
branch, documentation that cannot drift from the installed version is worth more here than in
most codebases. It is also a second surface, and decision 2 has to settle first.

**4. Does `sharp` stay?** ADR-0020 flagged it as an unsettled stack concern. It is answerable now
— it needs glibc allocator configuration and interacts with the pod memory limit — but "answer"
and "keep" are different questions, and disabling image optimisation is supported.

## Related

- ADR-0046 — the stack; this proposes the scaffold it made writable
- ADR-0020 — probes, chart and limits, which bind `web` because it is `deployable: true`
- ADR-0021 — the reusable workflow, and the action-choice gap in decision 5
- ADR-0024, ADR-0038 — the tenancy boundary ANTI-0009 protects
- ADR-0034, GLO-0006 — locale as a chain of four independent fields
- ADR-0044 — correlation ids, scoped to `web`
- ADR-0048 — why the toolchain pin is registry data, and its open question 3
- ADR-0007 — the hooks decision 2 collides with
- ANTI-0009 — the cache-key rule this template must make checkable
- REQ-0009 — accessibility and i18n; this fires its trigger
- REQ-0014, REQ-0001 — supply chain, and the encryption key as a runtime secret
- REQ-0016 — performance budgets, and why no `slo` block ships
- REQ-0018 — running software tested from outside
- RFC-0016 — the edge, whose open question 2 this answers
- RFC-0012 — the login plane, deliberately out of scope
- STD-0002, STD-0008, STD-0009 — documentation, tiers, and isolation testing
- BET-0004 — the catalogue that makes this decision recur per product
- C-0007, `class-registry.yaml` — the naming conflict in decision 4
