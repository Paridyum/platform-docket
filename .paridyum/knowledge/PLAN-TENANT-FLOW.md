# Plan — the self-serve tenant flow

> **Decisions taken 2026-09-20 by the operator.** Eight of them, recorded in §1 before any code is
> written, because the artifact precedes the implementation. Three amend accepted ADRs and two
> authorise contexts that ADR-0042 decision 7 reserves to a human.

*Written 2026-09-20. Scope: a stranger signs up at `paridyum.com`, provisions a tenant, and that
tenant runs docket, a landing and careers at its own hostname.*

**`TASKS.md` remains the authority** and Sprint Genesis still owns the arc. This document answers
*"how does the self-serve tenant flow ship"*, records the decisions that made it answerable, and
names what must be written before it is built. `PLAN-THREE-SURFACES.md` §0 holds the verified
estate state this plan starts from.

---

## 1. The flow, as decided

```
paridyum.com/                     web-paridyum     public landing, already serving
        │
        │  self-serve signup — Zitadel account, email verified
        ▼
paridyum.com/dashboard            web-dashboard    THE CONTROL PLANE
        │                                          · provision a tenant (max 3)
        │                                          · account-level security settings
        │                                          · no billing
        │
        │  the ADR-0039 saga runs, mints <tenant>.paridyum.com
        ▼
<tenant>.paridyum.com/            web-paridyum     that tenant's landing
<tenant>.paridyum.com/dashboard   web-console      THE TENANT CONSOLE  ← new repository
        │                                          lists that tenant's Releases:
        │                                            · docket
        │                                            · landing
        │                                            · careers
        ▼
    a tenant operator signs in here independently, at id.paridyum.com,
    against their own tenant's Zitadel organization (RFC-0012)
```

**Paridyum is this flow's first traveller, not its exception.** Tenant zero holds more Releases —
it holds the control plane — and nothing else about it differs. That is ADR-0049 decision 2 made
structural, and it is the test every decision below is measured against.

### The eight decisions

| # | Decision | What it changes |
| --- | --- | --- |
| **D1** | **`platform-deployment` is authorised** as the sixth bounded context, owning `Release`. | ADR-0042 decision 7 gate passed. Unblocks T-G.0, and with it Genesis waves 2, 3 and 4. → **successor ADR to ADR-0042** |
| **D2** | **`platform-entitlement` is authorised**, on the same gate. Owns `PlatformPlan`, `Subscription`, `Entitlement`. | Already decomposed by ADR-0042 as one of the five; decision 7 still reserved creating it to a human. Same ADR as D1. |
| **D3** | **The path stays `/dashboard`.** `/app` was a misremembering of the 2026-09-15 rename. | Nothing. ADR-0100 D3's revision stands and the estate already agrees. |
| **D4** | **Two deployables, not one surface with two modes.** `web-dashboard` is the control plane on the platform zone; a new **`web-console`** is the tenant console on a tenant's host. | **Amends ADR-0100 D3.** As revised it says `/dashboard` shows whichever tenant owns the host — one surface, every host. It is now one *path*, two *deployables*, two OIDC clients (ADR-0070 D2). |
| **D5** | **Signup is fully self-serve.** A stranger creates an account and provisions a tenant unattended. | **Amends ADR-0100 D5**, which defers self-service signup and ships operator-driven creation first. |
| **D6** | **Entitlement, not billing.** An operator's plan carries `max_tenants`, default **3**, enforced by the saga — it refuses, it does not warn. More tenants is a support conversation. | `platform-billing` **stays deferred**: ADR-0042 D3's trigger is *the first operator who must be charged*, and nobody is charged. T-3.8's trigger fires instead, which is correct — BET-0007 says quota is a correctness feature that *"must exist before the price does."* |
| **D7** | **`docket` registers as `kind: capability`.** | Unblocks T-B1, T-B3, T-G.6 and the docket half of T-A14. Owes a tenant-facing interface, which is `web-console` + web-docket — not an MKT-. |
| **D8** | **Subdomain now, custom domain as an alias later.** `<tenant>.paridyum.com` is minted at provisioning and stays canonical. | The routing layer must treat the subdomain as canonical **from the start** rather than retrofitting that when custom domains land (T-3.3). |

### Two things D6 gets right that are worth stating separately

**ADR-0042 decision 6 already split this and the split was retrieved, not invented:** *"`invoices`
is two entities — Paridyum's invoice to an operator (`platform-billing`) and a tenant's invoice to
its accounts (a catalogue product). One word, two owners, two lifecycles."* Paridyum→operator money
lives on the control plane when it exists; tenant→its-own-customers money is a product feature of
whatever the tenant sells, gated on T-0.10's payment path and bounded by ANTI-0007.

**`max_tenants` is the abuse control that makes D5 safe.** Self-serve signup without a quota is an
open provisioning endpoint. Verified email caps abuse at three idle tenants per address, which is a
support problem rather than an incident.

---

## 2. What must be written before anything is built

Governance precedes implementation. Each lands in its own commit before the work it justifies
(ADR-0013 §4).

| | Artifact | Carries | Blocks |
| --- | --- | --- | --- |
| **G1** | **Successor ADR to ADR-0042** | D1 + D2 — the sixth context authorised, entitlement authorised | everything in §4 |
| **G2** | **ADR amending ADR-0100** | D4 (two deployables) + D5 (self-serve signup) | `web-console` existing at all |
| **G3** | **Correct RFC-0049 §C** | it cites the **pre-revision** ADR-0100 D3 — *"never reachable from a tenant's hostname"* — which the 2026-09-15 revision already overturned, and D4 changes again | RFC-0049 acceptance |
| **G4** | **Accept ADR-0103 → RFC-0045 → RFC-0049**, in that order | §5 G1→G3 of `PLAN-THREE-SURFACES.md`; accepting RFC-0045 without ADR-0103 settled decides ADR-0103 by implication | the deploy saga's shape |
| **G5** | **Accept RFC-0044**, then register `docket` at `kind: capability` | D7 | T-G.6, docket as a Release |
| **G6** | **Accept RFC-0048, RFC-0050, RFC-0051** | the surfaces' own designs — parallel | `web-console`'s screens |

**Nothing here is an agent's act.** Every acceptance is the operator's. An agent that "accepts" an
RFC has decided a commercial question by assumption.

**One artifact is owed that does not exist yet:** the signup flow D5 authorises is *"a public route
that starts `requested → provisioning`"* in ADR-0100's own words, and no artifact designs it. G2
authorises it; something still has to specify the abuse controls, the verification step and what a
half-finished signup leaves behind.

---

## 3. Step 0 — git hygiene — **done 2026-09-20**

**Final state: 19 of 19 repositories clean, pushed, and signing.** Five PRs open, none merged —
merging is the operator's act.

What this actually found is larger than the instruction that prompted it. Three of the five items
below were not on any task list.

- [x] **S0.1. Five stale local branches cleaned.** Verified by content (`git diff origin/main`)
      rather than by `--contains`, because a squash merge does not preserve the commit and
      `--contains` would have reported every one of them as unmerged. Three were byte-identical to
      `main` and were deleted. **Two were not, and both carried real work** — see S0.4.
- [x] **S0.2. `platform-identity`'s `target/` untracked** — [PR #1]. 9,582 Cargo build artifacts,
      swept in by `30a580c` while it was doing something else, in the only repository in the
      estate with no `.gitignore` at all. Every path was checked before removal: `CACHEDIR.TAG` at
      the root, and the only `.rs` files anywhere under it are build-script output. Removed from
      the index, not from history — a rewrite would invalidate every clone, and the cost of
      leaving them is size.
- [x] **S0.3. T-A1's two PRs opened.** `knowledge` — [PR #1], and `infra-ci` — [PR #3].
- [x] **S0.4. The two branches that were not stale, rebuilt and proposed.** `tool-docket`'s D-64
      citation fix (T-A5, committed 2026-09-16 and never pushed) cherry-picked onto a clean branch
      off `main` rather than reopening a merged one — [PR #15]. `infra-ci`'s hygiene fix was the
      second half of T-A1.
- [x] **S0.5. `infra-kubernetes` had `__pycache__` committed** — [PR #4]. The same defect class as
      S0.2, found in the same sweep. ADR-0025's checkpoint runs `git add -A` and cannot tell a
      cache from a source file, so one provisioning run plus one idle was enough.

### S0.6 — the finding that was not a hygiene problem

**`knowledge/main` had not moved since 2026-08-19.** All 159 commits of governance since — ADR-0053
through ADR-0103, RFC-0034 through RFC-0051, Sprint Genesis, both plans — sat on a branch named
after one check in it, pushed, with no PR. `main` was 0 commits ahead, so nothing had diverged;
it had simply never been merged. [PR #1] proposes it.

**The mechanism explains itself.** ADR-0025's checkpoint commits to whatever branch is checked
out. For a month it checkpointed onto a feature branch, so *"the work is committed"* was true
continuously while *"the work is on `main`"* was false — and the hook reports the first.

**The bill arrived at merge time.** `check-pr.sh` refused the PR: *121,558 reviewable lines
exceeds the 400-line budget (STD-0007)*. That gate is right about proposals and this is not one —
it authors nothing, and the 159 commits were each written and gated incrementally. Splitting them
into 400-line slices now would mean inventing a review history that never happened and
re-proposing decisions already accepted. A `Review-budget-exception:` is recorded in the PR body,
where the check requires it to be visible to a reviewer rather than buried in a config file.
**The exception is the symptom; S0.6 is the cause.** A month of checkpointing onto a branch is
what turned an ordinary merge into one that needs a waiver, and the fix is upstream of
`check-pr.sh`: the hook should not be able to make `main` a month stale without anything saying
so.

**One hypothesis this produced was wrong and is recorded because checking it was cheap.** A stale
`main` looked like the single root cause of T-A18's three stale vendored snapshots. It is not: the
snapshots hold ADR-0056 and ADR-0087, both *ahead* of `main`'s ADR-0052, so they synced from a
working tree at different dates. T-A18 remains three independent instances.

### S0.7 — signing was off in two repositories, and the gate said so

`commit.gpgsign` was globally `true` and **locally overridden to `false`** in `infra-kubernetes`
and `web-hiring`. RFC-0013 D3 requires signed commits from the first commit and states that
history *"cannot be retroactively signed into contemporaneous evidence"* — so this is evidence
lost, not a lint.

**The damage is one commit** (`84dfbf5`, a checkpoint, 2026-09-15); `web-hiring` had the override
but had not committed under it. Both overrides are removed and all nineteen repositories now
inherit the global `true`, verified by sweep.

**The pre-commit gate did print `commit.gpgsign is not true (C-0003)` every time.** It is a
warning, it does not block, and it scrolled past above the output that was being read — which is
this estate's recurring failure shape (T-2.5, T-2.13, T-2.15, F0b) appearing a fifth time: *a
mechanism that reports correctly to nobody who is reading.*

[PR #1]: https://github.com/Paridyum/knowledge/pull/1
[PR #3]: https://github.com/Paridyum/infra-ci/pull/3
[PR #4]: https://github.com/Paridyum/infra-kubernetes/pull/4
[PR #15]: https://github.com/Paridyum/tool-docket/pull/15

---

## 4. The build, ordered

*Ordering principle inherited from Genesis: **irreversibility, not architecture.***

### Wave A — the lossy things, and the cheap unblock

Four more days have passed on both ⏱ items since `PLAN-THREE-SURFACES.md` was revised.

- [x] **A1. T-G.2 — one correlation id, one append-only audit stream.** ⏱ **Written 2026-09-20,
      [platform-tenancy#6], not applied.** `paridyum.current_correlation()`, `audit_events`
      (append-only), `operator_grants` (provenance immutable, ending write-once). Fully additive.
      The 18 days of requests already served carry no correlation id and never will; this stops
      the loss rather than repairing it.
- [x] **A2. T-G.3 — `region` on `tenants`, `NOT NULL`, no `DEFAULT`, immutable.** ⏱ **Written
      2026-09-20, [platform-tenancy#7], not applied — it takes data.** Three-step
      add/backfill/constrain, because `tenants` holds one row. The backfill is determinate: one
      live region.
      **And the check it had to satisfy did not exist.** `db.region_is_chosen_at_provisioning_and_immutable`
      was declared 2026-09-15 with `owner: check-migrations.sh` and **implemented by nobody** — the
      T-2.7 defect, where every owner value other than `tool-forge` counts as built. It read as
      enforcing for five days because nothing printed. Implemented and watched failing against six
      planted defects ([knowledge#1]) before the migration was trusted to satisfy it.
- [x] **A3. T-G.0d — docket's account silo. ANSWERED 2026-09-20: accept, with a trigger.**
      The operator's answer: *"docket gets created inside a tenant. so they need to be like that."*
      **ADR-0102** records it — docket is instantiated inside a tenant, its records are that
      tenant's, and there is no party beneath the tenant with a claim on one. The five tables stay
      `scope tenant`, which is now a decision rather than the leftover of docket having been
      single-tenant when they were written.
      **No migration.** T-G.0d's second permitted outcome.
      **This task named the wrong tables and the correction matters.** It said "add `account_id` to
      `applications`, `principals` and `profiles`." Those three are **already** `scope account` and
      already carry both columns. The tables at issue are the **docket person-bearing** ones, all
      `scope tenant`: `docket_persons`, `docket_identities`, `docket_register`, `docket_events`,
      `docket_signatures`. Every one is empty, so the window ADR-0024 D9 warns about is still fully
      open — the retrofit is free today and a policy rewrite against live rows later.

[platform-tenancy#6]: https://github.com/Paridyum/platform-tenancy/pull/6
[platform-tenancy#7]: https://github.com/Paridyum/platform-tenancy/pull/7
[knowledge#1]: https://github.com/Paridyum/knowledge/pull/1

### §7 — what A3 was blocked on, and how it was answered

*Resolved 2026-09-20 by ADR-0102. Kept because the three schema facts below are what made the
question answerable, and because the alternative this came closest to taking was wrong.*

**Nothing had decided what an `account` means inside docket, and the retrofit could not be written
without that.**

GLO-0004 defines the inner silo precisely: an account is **the tenant's customer**. docket's model
has no customer level. `docket_companies` (from `org.yaml`, with an `owned_by` self-FK) is the
tenant's own group of legal entities — its subsidiaries, not its customers. So `company` is not
`account`, and mapping one to the other would invent the product's shape in a migration.

**There is no `accounts` table anywhere in the schema.** `applications`, `principals` and
`profiles` carry `account_id` as a bare `uuid` with no foreign key. Whatever an account is, nothing
in this database yet says.

**The answer, and the alternative it rejected.** docket is instantiated *inside* a tenant, so its
records belong to that tenant and nothing below the tenant has a claim on one. The near-miss was
mapping `account` onto `docket_companies` — rejected on GLO-0004, because a company in `org.yaml`
is a *subsidiary* of the tenant and an account is a *customer* of it. Encoding a group's internal
structure as the platform's isolation boundary would have made a tenant's own subsidiaries
invisible to each other by default, which is the opposite of what a corporate HR group needs.

**The trigger, and what it costs if it fires.** The question underneath was commercial: **who buys
docket?** If it is a business
running HR for its own staff, docket is tenant-scoped and correct as it stands. If it is a staffing
agency or a PEO administering people on behalf of client companies, those clients are accounts and
every person-bearing table needs the inner discriminator. T-G.0d offers "accept the omission with a
written trigger" as a legitimate outcome for exactly this reason.

**What is already built to support either answer**, so the decision is not urgent for mechanical
reasons — only for the closing-window one: `check-migrations.sh` gained
`-- paridyum:rescope <table> <category>` in this sprint, because the `scope` directive lives in an
already-applied migration whose sha `migrate.py` refuses to see edited. Without it a table's
category could not be corrected at all, and the retrofit ADR-0024 D9 anticipates was unexpressible.
- [x] **A4. T-A19 — `console-docket` produces an image. DONE 2026-09-20.**
      `ghcr.io/paridyum/console-docket` @ `sha256:ac84f7f9…ccf504d` — the console's first digest, so
      the chart can finally render. [console-docket#2] carries the fix and the debt.
      **This task's diagnosis was wrong twice, and both are recorded rather than repaired quietly.**
      First the pnpm age policy (disproved by a re-run that failed identically). Then the 401 — real,
      but only the *outermost* of two failures: `console-docket` held no repository secrets at all
      (ADR-0045 R3's third location), and behind that sat a type error nobody had ever seen, because
      the build had never reached type-checking. `.dockerignore` drops `e2e/` and kept
      `playwright.config.ts`, which imports from it; `tsconfig.json` includes `**/*.ts`, so
      `next build` type-checked a file whose import target the same file had just removed.
      **A blocked pipeline hides every failure behind it**, which is the argument for not leaving one
      red — and the reason an estimate made against the first error was wrong twice.
      **The credential is debt, not a fix** — D-21: the value is the operator's personal `gho_`
      token, so CI authenticates as a person against STD-0017 rule 1, carrying `admin:org`/`repo`/
      `workflow` where `read:packages` alone would do.

[console-docket#2]: https://github.com/Paridyum/console-docket/pull/2

- [ ] **A4 original text.** ⛔ Blocked on the operator, and the blocker is a credential. The re-run on 2026-09-20 failed identically and disproved this
      task's original diagnosis; the real cause is that `console-docket` holds **no repository
      secrets at all**, while every other web repository carries `PARIDYUM_PACKAGES_TOKEN`. The
      build 401s fetching the private `@paridyum/auth`. `PLAN-THREE-SURFACES.md` F10 carries the
      correction and the evidence.
      **Done when:** `PARIDYUM_PACKAGES_TOKEN` exists on `Paridyum/console-docket` with
      `read:packages`, and a `publish.yml` run is green with a digest in its summary.
      **An agent must not mint or copy this token** — RUN-0004 governs handling a secret, and a
      credential pasted by an agent is a credential in a transcript.

### Wave B — the estate's picture

> #### ⚠ T-G.8's "done when" is wrong, and seeding proved it — 2026-09-20
>
> Both T-G.8 and T-A14 end *"…and the count matches `helm list -A`."* It cannot, and should not.
>
> `helm list -A` returns **nine** releases. **Four are substrate, not products:** `cilium`,
> `cnpg`, `longhorn`, `zitadel`. A Release is *"a product running for a tenant"* (RFC-0049 §A) —
> the CNI does not run for a tenant, it is what tenants run *on*. Giving them Release rows under
> tenant zero would make Paridyum the tenant that "runs Cilium", which is exactly the tenant-zero
> privilege ADR-0049 D2 exists to deny.
>
> | | count |
> | --- | --- |
> | tenant-held Releases — `admin`, `docket`, `hiring`, `web-paridyum` | **4** |
> | not a Release — `cilium`, `cnpg`, `longhorn`, `zitadel`, **`identity`** | **5** |
>
> **`identity` moved after this box was written, and the correction is in B1b.** RFC-0049 §B's own
> example never listed it as a Release; it is shared platform machinery, not a product a tenant
> holds a copy of.
>
> **Corrected completion test:** every running release **that is a tenant's product** appears as a
> `Release` row, and the four substrate releases are accounted for as substrate rather than
> silently missing. A count that matches nine would be a worse answer than one that matches five.
>
> **And only three of the five can be seeded today**, which is F1 and F3 arriving as a concrete
> blocker rather than a tidy-up:
>
> | Release | product slug | can seed? |
> | --- | --- | --- |
> | `web-paridyum` | `paridyum` | ✅ registered |
> | `admin` | `identity` | ✅ registered |
> | `identity` | `identity` | ✅ registered |
> | `docket` | — | ❌ **not registered** (F1). D7 decided `kind: capability`; the registry write has not happened |
> | `hiring` | — | ❌ **no product exists** (F3). RFC-0048 open question 2 leaves it open; registering one owes an MKT- under C-0006 |
>
> So B1 lands the schema and seeds what it honestly can. The remaining two are a registry decision,
> not a task — and `product-registry.yaml` is the one file this plan says nothing may write
> concurrently (ADR-0035).


- [x] **B1a. T-G.8 — the `Release` schema.** Written 2026-09-20, [platform-tenancy#9], stacked on
      #7. `releases` + `release_events` (append-only), owned by `platform-deployment` under
      ADR-0101 D2. `placement` is deliberately unconstrained until ADR-0103 is accepted.
- [x] **B1b. T-G.8 — seed it from the live cluster.** `scripts/seed-releases.py`, verified against
      the cluster 2026-09-20: **9 releases → 4 Releases, 5 not-a-Release, 0 unmapped.** Not applied
      — `releases` does not exist in the database until #9 merges, so the dry run is the deliverable.
      **And it found a second thing the box above missed.** `platform-identity` is not a Release
      either. The first draft mapped both `admin` and `identity` to product `identity`, which
      `UNIQUE (tenant_id, product, instance)` would have collapsed — silently keeping whichever was
      written last. **RFC-0049 §B had already answered it and nobody had read it that way:** its
      worked example of tenant zero lists `landing`, `careers`, `docket`, `admin` and `app`, and
      `identity` is not among them. It is shared machinery every tenant depends on, like Zitadel.
      `web-admin` is the Release; the service behind it is not.
      So the corrected count is **4, not 5** — and the category is not "substrate" but "not a
      tenant-held Release", which covers cluster infrastructure *and* shared platform services.
      This is what RFC-0049 says slice 1 is for: *"validates the entity shape against reality before
      a saga is written against it."* Two corrections in one afternoon, both from running it.
- [ ] **B2. `platform-entitlement`, minimally — `max_tenants`.** D6. One plan distinction enforced
      rather than described, which is T-3.8's own `done when`. Needs G1.
- [ ] **B3. T-G.6 — `product:` on the remaining manifests.** Unblocked for the docket pair by D7.
      `web-hiring` stays blocked on a decision, not a task (RFC-0048 open question 2).
- [ ] **B4. T-G.10 — the placement resolver, inert.** `poolFor(tenantId)` replaces the module-level
      `pool()`. Behaviourally inert at C0 and the seam every containment rung needs.

### Wave C — the flow itself

- [ ] **C1. Self-serve signup on `paridyum.com`.** D5. Verified email → account → the control
      plane. Needs G2 and the missing design artifact named in §2.
- [ ] **C2. The control plane provisions a tenant, and the saga refuses at `max_tenants`.** D6.
      **Done when:** a fourth provision attempt is refused by `platform-tenancy`, not by the BFF.
- [ ] **C3. Provisioning mints `<tenant>.paridyum.com`.** D8. One wildcard record, one wildcard
      cert at the edge, and a `routing-table.yaml` row generated rather than hand-written.
      **Done when:** a tenant is reachable at its subdomain the moment its saga completes.
- [ ] **C4. `web-console` — the tenant console.** D4, a new `web-` repository. A BFF over
      `platform-deployment`, listing that tenant's Releases. Owns nothing (ANTI-0003).
- [ ] **C5. T-G.12 — the public surfaces resolve their tenant from the request.**
      `paridyumTenantId()` is deleted; an unresolvable host 404s rather than falling back.
      **Carries T-G.1 with it** — the careers cache key must hold the resolved tenant *before* the
      host stops being a constant, or ISR serves one tenant's postings to another.
- [ ] **C6. T-G.13 — the deploy saga, one product end to end.** Careers first, it is simplest.
- [ ] **C7. Deploy `web-dashboard` and `web-console`.** Needs T-A20 (how the dashboard is reached)
      and T-B7 (its OIDC client). Both are still open.

### The finish line

- [ ] **C8.** A stranger signs up, provisions a tenant, and reaches their own docket at their own
      hostname — **with Paridyum holding no scope that tenant could not hold.**

---

## 4b. Second audit — 2026-09-20, CI state across every open PR

**Nobody had looked at whether the pipelines were green.** Fifteen open PRs were audited with
`gh pr list --json statusCheckRollup`, and the result was not "some are red":

| Finding | Severity |
| --- | --- |
| **A live security advisory on a deployed service** — RUSTSEC-2026-0285, `rustls 0.23.44` | ⛔ |
| **`console-docket`'s CI had never run.** Not once, since 2026-09-13 | ⛔ |
| `platform-tenancy` and `infra-kubernetes` have **no CI workflows at all** | open |
| The test harness hid the cause of a failure it had correctly detected | fixed |
| A broken intra-doc link, and a local gate that could not catch it | fixed |

### RUSTSEC-2026-0285 — a TLS flaw in the identity service

*"TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries."*
`platform-identity` is **deployed** — `platform/identity` rev 4 — and serves `id.paridyum.com`,
the estate's single sign-in surface. A TLS handshake flaw in the crate terminating its outbound
TLS is the most exposed place in the estate to carry one. Red since 08:53, seen by nobody.

Fixed by a lockfile bump to `0.23.45` ([platform-identity#2]); ADR-0053 already permits it —
*"an advisory bypasses the cadence."*

### `console-docket` had CI, a trigger, and no runs — the fifth instance of the signature failure

`nextjs-ci.yml` declares `packages: read`; the caller granted only `contents: read`. **A reusable
workflow cannot be granted more permission than its caller holds**, so GitHub refused every run
before any job began — `startup_failure`, with *"This run likely failed because of a workflow file
issue"* naming no permission, no line and no file.

**The silence is the defect.** `statusCheckRollup` returns `none`, which reads as *"no checks
configured"* and not *"every run died at startup"*. Nothing distinguishes the two. So this
repository had, simultaneously and invisibly: **no CI** (startup failure) and **no packages
token** (401 on every image build) — two independent silent failures, neither reported.

Fixed in [console-docket#2]; that repository's **first green CI run** completed 2026-09-20 12:15.

**Count of this failure mode so far: T-2.5, T-2.13, T-2.15, three absent `.claude/settings.json`,
signing silently off in two repos, and now this.** It is the estate's most reliable defect, and
every instance was found by running something rather than reading it.

### The `Review-budget-exception` mechanism had never worked — a sixth instance

Found by trying to use it. `knowledge#1` was refused by `check-pr.sh` — *121,569 reviewable lines
exceeds the 400-line budget* — so a `Review-budget-exception:` was added to the PR body, where the
check's error message says to put it. **The check refused it again.**

The body was fine: fetched exactly as CI fetches it, the check's own regex matched it. **CI could
not see the body at all.** `knowledge-check.yml` declared `permissions: contents: read`, and
reading a PR body needs `pull-requests: read` — so `gh pr view --json body` was refused, the
step's `|| : > /tmp/pr-body.md` fallback wrote an **empty file**, and the check correctly reported
that no exception was given.

**Every PR over 400 lines in this repository was unwaivable, and the error told the author to do
the one thing that could not take effect.**

**The fallback is why it stayed hidden.** A refused API call and a PR with genuinely no exception
produced byte-identical input — the failure was silent by construction. Both are fixed: the
permission is granted, and a failed fetch now fails loudly and names the permission.

**This is the same defect as `console-docket`'s missing `packages: read`, found hours earlier.**
Two missing workflow permissions, two silent failures, both in mechanisms that read as configured.
That is now six instances of this estate's signature pattern, and **every one was found by trying
to use the thing rather than by reading it.**

### ⛔ Still open: two repositories have no CI at all

`platform-tenancy` holds **every migration in the estate** — and has no workflow. So
`check-migrations.sh`, the gate enforcing ADR-0024's nested tenancy, **has never run in CI against
the repository it exists for.** `infra-kubernetes` is the same. Both are open, and the first is
the more serious.

[platform-identity#2]: https://github.com/Paridyum/platform-identity/pull/2
[console-docket#2]: https://github.com/Paridyum/console-docket/pull/2

---

## 4c. The merge and deploy runbook — 2026-09-20

**Order matters in three places, and two of them are not obvious.** Everything below is green
unless noted. `gh pr merge` is not available to the agent (the harness refuses it), so this is
written to be executed rather than described.

### Stage 1 — the web repos, BEFORE the check that will judge them

```sh
gh pr merge 1 --squash --delete-branch -R Paridyum/web-paridyum
gh pr merge 1 --squash --delete-branch -R Paridyum/web-admin
gh pr merge 3 --squash --delete-branch -R Paridyum/web-hiring
```

**Then, and only then:**

```sh
gh pr merge 3 --squash --delete-branch -R Paridyum/infra-ci
```

**This ordering is the point.** `infra-ci#3` makes the repository-hygiene check actually run in
every TypeScript repository — it has been pointed at the vendored snapshot instead of the
repository since it was written, so it ran, passed, and examined nothing. Merged first, it turns
`web-paridyum`, `web-admin` and `web-hiring` red for the `.claude/settings.json` those three PRs
add. Merged after, it finds them already compliant.

### Stage 2 — security and hygiene, independent of everything

```sh
gh pr merge 2 --squash --delete-branch -R Paridyum/platform-identity   # ⛔ RUSTSEC-2026-0285
gh pr merge 1 --squash --delete-branch -R Paridyum/platform-identity   # target/ untracked
gh pr merge 4 --squash --delete-branch -R Paridyum/infra-kubernetes
gh pr merge 15 --squash --delete-branch -R Paridyum/tool-docket
gh pr merge 16 --squash --delete-branch -R Paridyum/tool-docket
gh pr merge 2 --squash --delete-branch -R Paridyum/console-docket
```

**`platform-identity#2` before `#1`.** #1 is red *only* because it predates the advisory fix; it
goes green once #2 is in. Merging #1 first leaves a red main until #2 lands.

**`platform-identity#2` is the one to do first of everything here** — it is a live TLS advisory on
a service that is deployed and serving `id.paridyum.com`.

### Stage 3 — the migrations, in sequence

```sh
gh pr merge 7 --squash --delete-branch -R Paridyum/platform-tenancy   # 0011, region
gh pr merge 6 --squash --delete-branch -R Paridyum/platform-tenancy   # 0010, correlation id
gh pr merge 8 --squash --delete-branch -R Paridyum/platform-tenancy   # .gitignore
gh pr merge 9 --squash --delete-branch -R Paridyum/platform-tenancy   # 0012/0013 + seed
```

**#7 first, and this is the second non-obvious ordering.** Implementing
`db.region_is_chosen_at_provisioning_and_immutable` means **any tree without `0011` fails it** —
correctly, since such a tree has a `tenants` table with no `region`. #9 is stacked on #7 and
retargets to `main` automatically once #7 lands.

### Stage 4 — the governance

```sh
gh pr merge 1 --squash -R Paridyum/knowledge      # 159 commits; do NOT --delete-branch
```

Last, so that nothing else is waiting on a 159-commit merge, and because every other repository
vendors from it.

### Stage 5 — apply, then seed

```sh
export KUBECONFIG=infra/infra-terraform/kubeconfig
cd platform/platform-tenancy
python3 scripts/migrate.py status          # expect 0010-0013 pending
python3 scripts/migrate.py apply
python3 scripts/seed-releases.py           # dry run: 9 -> 4 Releases, 5 not-a-Release
python3 scripts/seed-releases.py --apply
```

⚠ **`0011` takes data**, which this sprint's own banner names as a stop condition. It is the
first migration in the estate that does. The backfill is determinate — one tenant row, one live
region — and `migrate.py` will refuse its `down` afterwards, so the recovery path becomes a
restore. That is expected and is ADR-0081 working, not a failure.

### What still cannot be deployed after all of this

| Surface | Missing |
| --- | --- |
| `web-dashboard` | **the auth BFF, which is unwritten.** Measured 2026-09-21: no `src/lib/auth/`, no `api/auth/*` routes, and **no `process.env` reads anywhere in `src/`** — so nothing consumes `PARIDYUM_OIDC_CLIENT_ID`, the chart's "the pod will not start" is false, and the pod would serve an unauthenticated surface. It is also not declared in `applications.yaml`, so `make zitadel-apps` would assign it nothing. Plus the routing decision (T-A20). |
| `web-docket` (console) | a `routing-table.yaml` row — `console.paridyum.com` has **no entry** — and an OIDC callback still set to the `ssh -L` loopback (D-20) |

Both are decisions rather than tasks, which is why they are not in the stages above.

---

## 5. What this plan deliberately excludes

- **`platform-billing`.** D6. Its trigger has not fired.
- **Custom domains.** D8 — subdomain is canonical first; the alias is T-3.3.
- **Tenant→customer payments.** Gated on T-0.10 (⛔, no dated citations) and bounded by ANTI-0007.
- **RFC-0047 (renting the platform)** and **RFC-0046 (BYOK)** — both gated, both outside Genesis.
- **The second failure domain** (T-G.0b) — unanswered, and nothing in this flow needs it.
- **Quotas beyond `max_tenants`** (RFC-0045 slice 8) — they degrade rather than leak.

---

## 6. The honest risk

**D4 and D5 together remove two defences and replace them with one.** ADR-0100's original reserved
`/app` to the platform zone, so the control plane was unreachable from a tenant's hostname *by
routing*. The 2026-09-15 revision gave that up for a uniform URL model, and its own text says so
plainly: *"the control plane is now gated by entitlement and by the Release a tenant holds, not by
the URL."* D4 partially restores it — two deployables means a tenant's host runs code that has no
provisioning screens in it — but D5 then opens account creation to anyone.

**So the thing standing between a stranger and the provisioning surface is entitlement and operator
scope, and both are currently enforced by compliance rather than by machinery.** STD-0009 is
`locked` and unenforced (T-2.3). ADR-0049 — *"nothing runs outside a tenant"* — is `locked` and
enforced by nothing while nine releases run.

**That is the argument for B2 preceding C1**, and for T-2.3 being real work rather than hygiene:
the first isolation test this estate writes should be the one a prospective tenant would ask for,
and it should be watched failing before it is trusted (RFC-0022 D1).
