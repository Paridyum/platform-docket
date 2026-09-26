# The web surface — `lib-ui`, `web-paridyum`, and the `delivery` capability

**Written 2026-08-22.** Design for the first three repositories that put a Paridyum interface in
front of a person who is not an operator: a shared design system, the landing page at
`paridyum.com`, and the capability that will eventually deploy both.

This document is prose in `docs/`, not a knowledge artifact. It binds nothing. Where it and a
knowledge artifact disagree, **the artifact wins.**

**Closes:** T-0.6 (vertical one chosen) · the ROADMAP phase 0 exit gate · RFC-0028 decisions 1, 2
and 4, by construction rather than by argument.
**Feeds:** RFC-0028 (the template this makes writable) · REQ-0009 (whose trigger this fires) ·
RFC-0016 (whose open question 2 this answers).

---

## 1. Why this exists, and why now

Three facts, and the third is the one with a clock on it.

**Paridyum has no public surface.** `paridyum.com` serves nothing. BET-0005 and RFC-0022 both rest
on the sentence *"`paridyum.com` sells to prospective tenants"*, and that sentence is currently
false. Two artifacts depend on a thing that does not exist.

**The `web` class has no repository, no standard and no template.** RFC-0028 is `proposed` with
five open decisions, and it says plainly what happens if that persists: *"repository number one
establishes the conventions by accident and repository number two inherits them by copying, which
is how ADR-0009 records the previous estate happening."*

**That copying has already started, and it is measurable.** `console-testing/src/app/globals.css`
is a byte-level copy of `console-docket/src/app/globals.css`, header comment included — the file
in `console-testing` still opens *"Design tokens for console-docket."* It carries `.gate`, a class
whose entire meaning is docket's rule that an interface must not offer a way past a refusal, into a
console that has no refusals. `palette.test.ts` was copied too, and its own comment concedes the
problem: *"Copying a test between repositories is usually a smell; here the alternative is
inheriting a class of bug and not its detection."*

Two frontends, zero shared code, one already-wrong file. A third frontend is about to be created.
**The cheap moment is before it is.**

---

## 2. What was decided, and by whom

Recorded here because two of these are strategy rather than engineering, and a strategy decision
taken inside a design conversation is the kind DECISION-DEBT's "possible unrecorded decisions"
section exists to catch.

| Decision | Taken by | Note |
| --- | --- | --- |
| The delivery capability is a registered product, not an internal mechanism | operator | Selected over "no new product" and over accepting RFC-0025 as-is |
| The design system is a private package, not a vendored snapshot | operator | Registry corrected from npmjs.com to GitHub Packages on cost and auth grounds — see §3.5 |
| Origin is `sitar`, and renting a node is a variable | operator | *"development and deployment is done so when we have funds we immediately rent the node"* |
| The marketing site gets its own identity; primitives are shared | operator | Forces two-layer tokens — see §3.3 |
| **`hosting` is vertical one** | operator | Consumes the anti-goal 6 slot. See §4.5 and §8 |

**The last row is the significant one.** `product-registry.yaml` records that promoting `hosting`
to `building` is C-0006 anti-goal 6's gate and that the choice was *"deliberately not taken"* by
registering it. It is taken now. The consequence is stated rather than discovered: **`veer-entitlements`
is blocked until `hosting` has a renewed paying customer**, which is C-0009 obligation IV's own
stated trigger and which that entry already anticipated in terms this document will not improve on.

---

## 3. Part A — `lib-ui`, the design system

### 3.1 The problem, without a solution in it

Design decisions that must be identical across every Paridyum frontend are currently stored once
per frontend, by copy. Nothing owns them, nothing versions them, nothing detects divergence, and
the copies have already diverged in a way that is wrong rather than merely different.

### 3.2 Four tiers, and where ANTI-0004's line falls

ANTI-0004 is `accepted` rather than `locked` because *"whether a function is business logic or a
utility is a judgement no check can make."* It supplies the test, and the test is exactly right
here: **if the business changed its mind, would this file need editing?**

| Tier | Contents | Shared? |
| --- | --- | --- |
| 1 · **Tokens** | colour, type, space, radius, motion — semantic names, per-theme values | yes |
| 2 · **Primitives** | Button, Input, Field, Dialog, Table, Badge. Focus, keyboard and ARIA behaviour | yes |
| 3 · **Blocks** | PageHeader, EmptyState, and for marketing Hero, FeatureGrid, PricingTable, CTA | yes |
| 4 · **Product devices** | `.gate`, `.known` / `.inferred` / `.unchecked` | **never** |

**Tier 4 is the rule that stops `lib-ui` becoming a shared kernel.** `.gate` draws a barrier with
nothing on it to click, because §8 of docket's design says an interface must not offer a way past a
refusal — a `Refusals` component containing a button fails a test for exactly that reason. That is a
rule about how Paridyum treats lifecycle gates. Move it into a shared library and the rule is owned
by nobody and changed by editing a third repository, which is ANTI-0004's shape precisely.

It is also not hypothetical. `.gate` is in `console-testing` today and means nothing there.
**Deleting it is part of this work, not a follow-up.**

The `web.*` check namespace already exists (`web.data_layer_modules_are_server_only`,
`web.server_actions_authorize`), so tier 4's boundary is upheld at review in the same way ANTI-0004
is — named, argued, and deliberately not given a false-confidence check.

### 3.3 Tokens are two layers, because the marketing identity differs

The operator chose a distinct marketing identity over inheriting the console palette. That decision
is what makes the token layer a contract rather than a colour list:

```
semantic contract   --color-surface, --color-surface-raised, --color-ink,
(shared by every    --color-ink-soft, --color-accent, --color-danger,
 surface, never     --color-warning, --color-success, --color-rule
 redefined locally)
        |
        +-- theme: console   IBM Plex, indigo `seam`, lac `seal`, verdigris, cool slate ground
        +-- theme: marketing  its own face, its own accent, its own ground
```

Consequences that fall straight out, and neither is optional:

1. **The contrast test runs per theme × per mode.** Today it runs light and dark for one palette.
   With two themes that is four passes, and **a new theme cannot ship until it clears them.** The
   bug that test was written for — the chrome band at 2.1:1, green lint, green types, green tests,
   found only in a screenshot — is exactly the bug a second theme reintroduces.
2. **`--color-chrome` keeps its exception.** The existing palette already carries a pair that
   deliberately does not follow the theme, with the reasoning written down. That behaviour is
   preserved as a token-level concept, not re-derived per theme.

The existing `@theme` block and `palette.test.ts` are good work in the wrong place. They move; they
are not rewritten.

### 3.4 Storybook

Storybook is a dev dependency of `lib-ui` and the acceptance surface for tiers 1–3. Every exported
component has a story; the story is where states that are hard to reach in an app — loading,
disabled, error, RTL, long content — are made reachable.

**Its static build is a CI artifact, not a deployment.** `library` is `deployable: false`, so no
class could host it, and inventing one is the vestigial-concept failure `class-registry.yaml`'s own
header warns about. It is built in CI and downloadable; it is not a running service.

**It cannot appear in the repository name.** C-0007: the `<name>` is *"a noun naming the capability,
never a technology and never an adjective."* `lib-storybook` is illegal on that clause.

**Name: `lib-ui`.** C-0007's abbreviation rule permits what is *"widely recognised across the
industry"* and states the test is recognition rather than membership of its illustrative list; `ui`
passes on the same footing as `api` and `sdk`. `lib-interface` was rejected — `spec-` repositories
are already *"interface contracts"* and the collision is the kind GLO-0007 exists to prevent.

### 3.5 Distribution — GitHub Packages, not npmjs.com

The operator chose a private package. The registry choice was corrected on evidence:

| | npm Teams | GitHub Packages |
| --- | --- | --- |
| Cost | **$7 per member per month**, paid org required, charged per member even for members with no access | Free plan: 500 MB storage, 1 GB/month transfer |
| CI authentication | granular access token with bypass-2FA, set as a secret in every consumer repo, rotated by hand | **none** — "Manage Actions access" grants the consumer repo and the default `GITHUB_TOKEN` works |
| CI transfer | counts against quota | free when pulled via `GITHUB_TOKEN` inside Actions |
| Vendors | a second one | already the vendor of record |

The auth row is the one that matters, because it removes a cost this design would otherwise have had
to accept. `nextjs-ci.yml`'s own header documents why a PAT was avoided once already: the default
`GITHUB_TOKEN` carries `Contents: read` for the **calling** repository only, *"and the usual fix is a
PAT that REQ-0001 has no home for and that GitHub Free cannot provide as an org secret for a private
repository (ADR-0004)."* Package-level Actions access grants sidestep that entirely.

Mechanics:

- `lib-ui` publishes `@paridyum/ui` to `npm.pkg.github.com`, private.
- Each consumer repository is added under the package's **Manage Actions access**.
- Consumer `.npmrc`: `@paridyum:registry=https://npm.pkg.github.com` plus
  `//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}`.
- `nextjs-ci.yml` gains `permissions: packages: read`.
- Local development needs a `packages:read` PAT **once per machine**, not once per repository.

⚠️ **Adding that permission is a breaking change for callers under `nextjs-ci.yml`'s own rule.** Its
header states the rule for vendored scripts — *"refresh EVERY caller's snapshot in the same change
that adds the step"* — and the same discipline applies: every caller is updated in one change, or a
caller's next push goes red.

### 3.6 Governance — how "every frontend shares it" is enforced rather than hoped

ADR-0030 requires every binding rule to name its verification. ADR-0014 makes check ids registry
data. So the rule lands as an artifact plus entries in `check-registry.yaml`, not as prose.

| Check | Severity | What it catches |
| --- | --- | --- |
| `ui.tokens_are_not_locally_defined` | error | A `web-` or `console-` repository declaring `--color-*` or `--font-*` outside the package. **This is the check that would have caught the copy.** |
| `ui.design_system_is_pinned` | error | An inexact version range. ADR-0053 — a pin is exact and current |
| `ui.design_system_pin_is_current` | warn | Staleness, the same shape as the vendored-knowledge snapshot audit |
| `ui.exported_component_has_a_story` | error | An export with no story — the documentation drifting from the component |

Artifacts owed:

- **An ADR — the interface is one library, and a frontend may not define its own tokens.**
  `locked`, with `enforced_by` naming the checks above. It is checkable, so under ADR-0002 it
  earns `locked` rather than stopping at `accepted`.
> **Note added 2026-08-22, after this document was written.** The number **STD-0015 was
> allocated to a different artifact** — *the visual identity* — when the operator's logo boards
> arrived later the same day. Ids are issued next-unused with no gaps, and this document binds
> nothing. **The design-system standard described below takes the next free number when it is
> written.** Every reference to "STD-0015" in this file means the design system and does NOT
> mean the artifact now sitting at that id.

- **A standard — the design system.** The tiers, the ANTI-0004 line, the two-layer token
  contract, the contribution model, and SemVer policy: patch for a visual fix, minor for a new
  component, major for a changed component API.

`class-registry.yaml` needs no change. `library` already permits `typescript`, and its own note says
so: *"`library` is the same concept either way."*

---

## 4. Part B — `web-paridyum`, the landing page

### 4.1 What being repository #1 of the `web` class means

RFC-0028's five open decisions stop being hypothetical. Three are answered by building:

**Decision 1 — CSP: `experimental.sri`, not nonces.** RFC-0028 records the collision and says
neither artifact priced it: a nonce-based CSP *"requires every page to render dynamically: static
optimisation and ISR are disabled, CDN caching is lost, and Partial Prerendering is INCOMPATIBLE."*
This page must rank and must be served from Cloudflare's cache. Hash-based SRI preserves static
generation. It is `experimental` and App-Router-only, and that is the accepted cost.

**Decision 2 — `agentRules: false`.** On 16.3+ `next dev` auto-upserts `AGENTS.md` and `CLAUDE.md`.
In this estate `CLAUDE.md` is a governance router under ADR-0026 and ADR-0007's hooks police what may
be written to it. A dev server editing a governance file is not a thing to discover during a
debugging session.

**Decision 4 — `template-web`**, matching the seven references rather than `class-registry.yaml`'s
single `template-web-app`.

Decisions 3 (a `typescript` entry in `toolchain-registry.yaml`) and 5 (the TypeScript action-choice
table) are **not** answered here. They are ADR-0048 and ADR-0021 territory respectively, and RFC-0028
already says item 5 needs its own ADR supplementing ADR-0046.

### 4.2 Version — 16.2.x LTS, and the consoles are wrong today

RFC-0028 finding 1: the July 2026 advisory, which patched nine CVEs following thirteen in May,
*"directs production users to the 16.2.x Active LTS or 15.5.x Maintenance LTS branch rather than the
newest minor."*

`console-docket` and `console-testing` both pin `next: 16.3.1` — the newest minor. That is a live
divergence from a recommendation already written down in this repository. `web-paridyum` pins
**16.2.x**, and correcting the two consoles is recorded as a task rather than folded in silently.

### 4.3 Origin is an input, never a constant

The origin is `sitar-as-in-01-ar` today — RFC-0023's substrate, behind a Cloudflare Tunnel. RFC-0023
accepted cost 3 is blunt about what that means: *"No availability of any kind. One machine, one power
supply, one network connection, in a residence."*

The operator's decision is that this is temporary and that renting a node must be immediate when
funded. **That is only true if nothing encodes the origin**, which is an invariant this estate has
already written twice:

- RFC-0025 decision 4: *"nothing may encode 'this is the hosting fleet' or 'this is the platform
  fleet.' Fleet is an input to the same provisioning code."*
- RFC-0023 decision 4 carries the same invariant one level down.

So ADR-0020's obligations are satisfied **from the first commit, on the residential machine**:
`output: "standalone"`, a multi-stage non-root Dockerfile, a Helm chart with `livenessProbe` and
`readinessProbe` bound to `/healthz` and `/readyz`, resource limits,
`terminationGracePeriodSeconds: 30` for `after()` callbacks, and ingress buffering disabled so
streaming does not degrade silently.

Renting the node then becomes a Helm values change and a DNS repoint. **Satisfying ADR-0020 now is
what makes it cheap later**; deferring it is what would turn "when we have funds" into a rebuild.

### 4.4 Production posture

| Concern | What ships |
| --- | --- |
| Rendering | Static shell plus PPR where available; `cacheComponents` on, cache keys constructed only through `lib/cache/keys.ts` — **ANTI-0009**, whose failure is one tenant's rendered output served to another |
| SEO | Metadata API, `sitemap.ts`, `robots.ts`, JSON-LD `Organization`, static OG images. Dynamic OG generation is rejected: it complicates CSP and costs per request for no ranking benefit |
| i18n | `app/[lang]/` with `next/root-params`, **two locales, one right-to-left**, layout on CSS logical properties |
| a11y | WCAG 2.2 AA, `eslint-plugin-jsx-a11y`, `@axe-core/playwright` in Playwright |
| Performance | `useReportWebVitals` with **no thresholds** — REQ-0016 requires numbers be chosen, not inherited, and RFC-0016 notes no single number covers a local and a remote tenant |
| Correlation | `traceparent` propagation per ADR-0044, whose scope includes `web` |
| Errors | `app/global-error.tsx`, `app/global-not-found.tsx` |

**Two locales, one RTL, is not a nicety.** CVE-2026-64642 fires on App Router applications with
*exactly one* entry in the locales array, in which configuration proxy-level checks can be bypassed
entirely. REQ-0009 item 9 wants RTL considered from layout rather than retrofitted. A single-locale
launch is both the vulnerable configuration and the expensive one.

### 4.5 The offer — prospective tenants and hosting

The page addresses two audiences, and the second one carries the §2 decision.

**Positioning is fixed by artifacts, not invented.** C-0006 fixes the category — *"Paridyum is a
business operating platform"* — and explicitly forbids the internal North Star as external copy:
*"Do not use it as external copy... it is unfalsifiable and abstract to the person deciding whether
to sign up."* Four framings are named as actively wrong and none may appear: no-code platform,
website builder, white-label SaaS, app store. BET-0006 fixes the segment: the wedge is the operator,
not the builder.

**Hosting is now a real offer with a real price**, because `hosting` moves to `stage: building`.
BET-0009 fixes the unit of sale and the page must not undercut it: **priced per running workload,
never per resource.** BET-0009 trigger 4 is the direct test — *"more than one in four evaluating
customers asks to be quoted per vCPU or per GB and declines when refused"* — so a pricing table
quoting vCPU or GB would not merely be off-message, it would manufacture the evidence that kills the
bet.

The dossier obligation for `catalogue` at `building` is **MKT + REQ + the `product-contract` slot**.
MKT-0001, MKT-0002 and MKT-0003 already satisfy the MKT half; a REQ and a `spec-` contract are owed.

### 4.6 The waitlist is the first personal data this estate collects from the public

DPDP obligations attach at the first submission: stated purpose, consent that is not bundled,
retention with an actual expiry, and a deletion path. `tool-docket` already models this discipline
and is the precedent to follow rather than reinvent.

The form is a Server Action, authorising on every path through the named DAL helper — the target
`web.server_actions_authorize` matches. Next.js's own data-security guidance is the reason this is
not optional: **Server Actions are reachable by direct POST whether or not they are referenced in
the UI.** A public form with no server-side rate limit and no origin check is an open write endpoint.
`serverActions.allowedOrigins` and `bodySizeLimit` are set.

---

## 5. Part C — the `delivery` capability

### 5.1 The problem, without a solution in it

Under BET-0005 every tenant runs a catalogue product under their own brand, on their own domain.
Getting a built artifact live at a hostname — and back off it — is therefore a runtime capability the
platform owes every tenant. Today nothing owns it, and the landing page in Part B will be deployed by
whatever anyone happens to wire up first.

### 5.2 The boundary, which is the whole design

RFC-0025 decision 10 already established the shape one layer down: *"The fleet manager **writes**
routing config; the edge **serves** it."* Delivery is the same split, one layer up.

| Owner | The question it answers |
| --- | --- |
| `infra-ci` | Did this commit build and pass? — **ends at a signed artifact** |
| `svc-delivery` | Which artifact is live, at which hostname, for which tenant? |
| `svc-fleet` | Which machine runs it? |
| `platform-routing` | Given this inbound request, which tenant and region? |
| the rented edge (Cloudflare) | TLS, certificates, static cache |

**The rule: delivery writes the binding and the release; routing and the edge serve them.**

Delivery is never on a path that serves an end user. That property is enumerated the way RFC-0025
decision 10 enumerates it and **verified the way TST-0062 verifies it** — stop the service, assert a
deployed site still serves — rather than asserted. RFC-0025's own warning applies unchanged: this
property *"decays on any deployment that adds a feature and prose does not notice."*

Two boundaries deserve naming because crossing them is the likely failure:

- **Certificate issuance stays with the edge.** RFC-0025 decision 10 row C forbids brokering ACME
  through the manager, and explains why no test will catch it: *"a renewal brokered through the
  fleet manager fails silently and surfaces at certificate expiry, weeks later."*
- **`platform-routing` owns `DomainBinding` as a type** (ADR-0042). Delivery writes bindings; it does
  not own the resolution model. A second implementation is ANTI-0004 at the service layer.

### 5.3 Why this is not the thing C-0006 forbids

C-0006 names web hosting and CDN as commodity and rejects `website builder` outright — *"names the
least valuable layer we own."* The argument that this capability is not that, written rather than
assumed:

**A website builder is an authoring tool sold to someone who wants a website. `delivery` is the
mechanism by which a tenant's product reaches the tenant's own domain** — the thing BET-0005's
white-label model structurally requires and C-0008 requires be built as a product a tenant can buy
rather than as Paridyum's internal deployment script. The commodity parts stay rented: Cloudflare
terminates TLS, issues certificates and caches. What is built is the release model, the domain
binding and the tenancy of both.

**It registers as `capability`, not `catalogue`**, which is also why it does not compete with
`hosting` for the anti-goal 6 slot — capabilities are not verticals.

### 5.4 Registry and repositories

```
product-registry.yaml
  delivery      kind: capability   stage: proposed  ->  building when svc-delivery starts
  hosting       kind: catalogue    stage: building  (was proposed — see §2)

repositories
  lib-ui         class: library   stack: typescript   deployable: false
  web-paridyum   class: web       stack: typescript   deployable: true
  svc-delivery   class: service    stack: rust        deployable: true
  web-delivery   class: web        stack: typescript  deployable: true
  spec-delivery  class: spec       stack: none        — the tenant-facing-interface slot
```

**`svc-delivery`, class `service`, follows T-0.17's precedent** — the analogous fleet manager was
placed at `svc-fleet, class service` rather than as a new `platform-` context. ADR-0042 fixes the
first wave at five contexts and RFC-0018 holds the control plane to a closed list, so adding a sixth
is a decision in its own right and is not taken here.

**The surface is `web-delivery`, not `console-delivery`.** `class-registry.yaml` gives the test and
it is reachability, not intent: *"If tenant traffic can reach it, it is not a console: C-0008 makes it
a product and it belongs in `web-`."* Tenants will reach this.

Names derive cleanly under C-0007: `svc-delivery` → crate `paridyum-delivery`, image
`ghcr.io/paridyum/delivery`, Helm release `delivery`, namespace `services`,
`PARIDYUM_DELIVERY_*`. "Delivery" over "deployment": a noun naming the capability, and it does not
collide with a Kubernetes `Deployment`.

### 5.5 The URL, and the DNS gap it exposes

**`deploy.paridyum.com`.**

Existing practice is already subdomains — `id.paridyum.com` and `login.paridyum.com` (RFC-0012),
`inf.paridyum.com` (RFC-0021 decision 7). A separate apex domain would buy a second brand to
establish, a second certificate story, a second SEO surface and a public-disclosure decision under
RFC-0013, for a surface only Paridyum and its tenants use.

**There is no rule, only practice.** C-0007 rejected a DNS registry on the grounds that the proposed
scheme's *"registries for products, projects, teams and DNS describe an organisation that does not
exist."* It exists now, and three subdomains have been chosen in three separate artifacts with no
governing clause. That is the C-0002 defect repeating in a different namespace.

- **An ADR — a product surface is a subdomain of `paridyum.com`; a tenant's product is on the
  tenant's domain.** Number allocated when written. Small, and it closes the gap before there are ten subdomains rather than after.

---

## 6. Sequencing

Order matters because `web-paridyum` will copy whatever exists on the day it is created.

| | Step | Why here |
| --- | --- | --- |
| 1 | **The subdomain ADR** — product surfaces are subdomains | Unblocks every URL below |
| 2 | **`lib-ui` v0.1.0** — tokens only, extracted from the consoles, contrast test moved in and widened to two themes. Plus the token ADR, the design-system standard (see the note in §3.6 — *not* STD-0015, which is now the visual identity), checks | Must exist before step 3 or step 3 copies again |
| 3 | **`web-paridyum`** — consuming `lib-ui` from its first commit. Ships through `infra-ci` + Cloudflare to the `sitar` origin | The landing page does not wait for step 5 |
| 4 | **Backfill** — both consoles migrate to `lib-ui`; `.gate` deleted from `console-testing`; both corrected to Next 16.2.x LTS | Proves the package works on a real consumer before more are created |
| 5 | **The delivery RFC** + register `delivery` at `proposed`; then `svc-delivery`, `web-delivery`, `spec-delivery` and a REQ on promotion to `building` | Takes over step 3's path afterwards |

**Step 3 precedes step 5 deliberately, and step 5 is not optional.** Paridyum migrating its own
landing page onto `svc-delivery` after the fact *is* RFC-0025 decision 2's shape — customer zero,
same path as anyone else, every failure mode hit first by the party who can fix it. The risk is named
in §8 rather than hidden: a landing page that already works is the most common reason the delivery
product never gets built.

---

## 7. Artifacts owed

Numbers are the next unused as of 2026-08-22 and are **provisional** — confirm at creation, since a
concurrent session can take one.

| Artifact | Type | Status on landing |
| --- | --- | --- |
| An ADR — a product surface is a subdomain of `paridyum.com` | adr | `accepted`; `locked` if a check is written |
| An ADR — the interface is one library, and a frontend may not define its own tokens | adr | `locked`, `enforced_by` the four `ui.*` checks |
| STD-0015 — the design system | standard | `accepted` |
| An RFC — the delivery capability and its boundary | rfc | `proposed` |
| A REQ — delivery requirements | requirement | on promotion to `building` |
| REQ — hosting requirements | requirement | owed now: `catalogue` at `building` |
| `check-registry.yaml` | registry | the `ui.*` entries |
| `product-registry.yaml` | registry | `hosting` → `building`; `delivery` added; `veer-entitlements` note amended per §2 |

Each of `lib-ui`, `web-paridyum`, `svc-delivery` also carries an SOP (RFC-0032, STD-0013) and a
logbook (RFC-0033), and `manifest.yaml` **with no `slo` block** — REQ-0016's trigger fired once
already on `templates/rust-service/manifest.yaml` for numbers *"derived from taste."*

---

## 8. Accepted costs

Stated rather than smoothed over, on the principle the rest of this repository applies.

1. **A private package breaks fork PRs.** ADR-0012 makes contribution happen from forks, and fork
   PRs get no access to secrets or to package grants. `pnpm install` will fail for any outside
   contributor. **Latent, not live** — DECISION-DEBT records that no PRs are opened today. The exit
   is making `lib-ui` public, which is cheap and one-way.
2. **500 MB of GitHub Packages storage is shared with Actions artifacts.** A design-system tarball is
   negligible; Playwright traces and Storybook builds are what would consume it. Worth a budget note
   before the first retention policy is needed rather than after.
3. **The landing page's availability is one residential machine.** RFC-0023 accepted cost 3, adopted
   knowingly. Cloudflare's cache blunts it for static content and does nothing for the waitlist form.
4. **`experimental.sri` is experimental.** Chosen over a nonce because the alternative forfeits the
   caching this page exists to use. It can be withdrawn upstream.
5. **`veer-entitlements` is blocked** until `hosting` has a renewed paying customer. A dated
   consequence of a decision taken in §2, not a discovery.
6. **`web-paridyum` becomes the de facto `template-web`** whether or not it is reviewed as one —
   RFC-0028's own warning. Mitigated only by writing the template from it in step 4's window, while
   the reasons for each file are still fresh.
7. **`scripts/check-significance.sh` does not exist**, though the change lifecycle's first step calls
   for it. This design was classified significant on the obvious grounds that it creates four
   repositories. Recorded because an unenforced step is the class of gap this repository prints
   rather than hides.

---

## 9. Open questions

1. **Does `lib-ui` ship Tailwind's `@theme` block, or framework-neutral CSS custom properties?**
   Both consoles are Tailwind v4 today, so the block is the cheaper answer and the one this design
   assumes. A framework-neutral token layer costs more now and is the only version that survives
   Tailwind being replaced.
2. **Does the hosting offer appear at launch or at a second pass?** `hosting` reaching `building`
   makes it permissible; the REQ and the `product-contract` slot are owed and do not exist yet.
   Launching the platform half first is legitimate and is not the §4.5 dodge, provided the sequencing
   is a schedule rather than a permanent state.
3. **Where does `web-delivery` authenticate?** RFC-0012 puts the login plane in `platform-identity`
   on Zitadel, whose Next.js packages RFC-0028 records as **beta and subject to change**. Until that
   exists, the surface has no login and is reachable only from `sitar`.
4. **Does `template-web` get written from `web-paridyum`, or independently?** RFC-0028 open question
   1 — a `ci: false` repository holds unbuilt source — is unresolved either way, and its cheapest
   proposed answer is nightly scaffolding plus CI on the scaffolded output.

---

## Related

- **RFC-0028** — the web template; decisions 1, 2 and 4 answered here by construction
- **RFC-0016** — the edge is rented, Cloudflare chosen in decision 4; open question 2 answered
- **RFC-0025** — the write/serve split this borrows, and decision 2's customer-zero shape
- **ADR-0020** — probes, limits and chart, satisfied on a residential origin so the swap stays free
- **ADR-0042** — `platform-routing` owns `Route` and `DomainBinding`
- **ADR-0046** — the stack; **ADR-0053** — a pin is exact and current; **ADR-0055** — the console class test
- **ANTI-0004** — the line between tier 3 and tier 4; **ANTI-0009** — the tenancy-blind cache key
- **C-0006** — the commodity split, the rejected framings, anti-goal 6
- **C-0007** — naming, and the DNS gap the subdomain ADR closes; **C-0008** — every runtime capability is a product
- **C-0009** — the covenant, and what §2's decision defers
- **BET-0005**, **BET-0006**, **BET-0009** — brand, segment, and the unit of sale the page must honour
- **REQ-0009** — accessibility and i18n, whose trigger this fires; **REQ-0016** — why no `slo` block
