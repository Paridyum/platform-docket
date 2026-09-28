# Plan — landing, dashboard, and docket as a service

> **Superseded in part, 2026-09-28 — ADR-0113 to ADR-0122.** A 2026-09-16 sprint record, kept as
> written. Its dashboard is now **the portal at paridyum.in** — the platform's own surface, serving
> orgs — and tenant zero is Paridyum's tenant only (ADR-0118). ADR-0049 is superseded by ADR-0116,
> so T-E3's list reads ADR-0116. T-G.5's `operator` ambiguity is closed by ADR-0114 and ADR-0113:
> the party above a tenant is the **org** (GLO-0012).

> ## ⚠️ SPRINT MARKED RISKY — 2026-09-16
>
> **This sprint is authorised to change a running estate.** Nine Helm releases are serving on a
> cluster that has been up for fourteen days, four hostnames answer the public, and the work below
> includes schema migrations against a live database and two repository renames under STD-0022.
>
> **The rollback point is the commit that introduced this banner.** Its message carries the exact
> instruction; the tag `pre-parallel-implementation` marks the same tree. Rolling back is
> legitimate and costs only the work below — nothing in this sprint is irreversible *until* a
> rename lands or a migration takes data (ADR-0081).
>
> **Two things in the estate are already irreversible and are not fixed by rolling back:**
> the fourteen days of traffic written with no correlation id (T-G.2) and the deployments recorded
> nowhere (T-A14). Those get *worse* on rollback, not better.
>
> **Stop conditions.** Halt and report if: a migration runs against rows rather than an empty
> table; a rename has begun and cannot be completed in the same change; the gate goes red on an
> artifact rather than on a count; or `kubectl` is answering something other than the `staging`
> cluster (§1, RUN-0001's addendum).

*Written 2026-09-16. Revised 2026-09-16 after verification. Re-audited 2026-09-20 — see §0.
Scope: getting three surfaces deployed, for tenant zero and then for a second tenant.*

**This does not replace `TASKS.md`.** Sprint Genesis already owns this arc and remains the
authority. This document answers *"how do the three surfaces actually ship"*, names what Genesis
leaves implicit, and records what changed.

---

## 0. Re-audit — 2026-09-20

**Every line below was produced by running something, four days after the revision above.** The
cluster was probed with `KUBECONFIG=infra/infra-terraform/kubeconfig`, the hostnames with `curl`,
the gate with `scripts/check-knowledge.sh`, and the two undeployed surfaces' image pipelines with
`gh run list`. Where a claim could not be produced from here it says so.

### The estate has not moved. The repositories have.

`helm list -A` returns the **same nine releases**, with the same revision numbers, as on
2026-09-16. `kubectl get nodes` shows five Ready at 18d. Four hostnames answer
(`paridyum.com`, `careers.paridyum.com`, `admin.paridyum.com` → `307 → 200`;
`id.paridyum.com` → `302 → 200`). **Nothing has been deployed in four days.**

What did move is substantial and sits entirely in git:

| Landed since 2026-09-16 | Where | Effect on this plan |
| --- | --- | --- |
| **ADR-0099 stages 1–4** — docket derives its tenant from the token, reads config from the tenant's rows, Paridyum seeded and cut over, and a new tenant can be onboarded | `tool-docket`, `platform-tenancy` 0008/0009 | **The largest single advance in the arc, and this plan never tasked it.** §4c listed only governance and renames; the engine became genuinely multi-tenant underneath it. |
| OIDC login + BFF, Helm chart, Dockerfile, `publish.yml` | `console-docket` | D-18 item 1 resolved. Does **not** make it deployable — see F10. |
| `product: [paridyum]` declared | `web-paridyum` | **T-A6 done.** |
| `TESTING.md` + `DECISION-DEBT.md` in the three web repos | `web-*` | **T-A2 done.** |

### What is still not true, verified today

| Claim | Verified by | Result |
| --- | --- | --- |
| `web-dashboard` deployed | `helm list -A` | **no release** |
| `console-docket` served | `curl https://console.paridyum.com` | **host does not resolve** — image now exists (T-A19 closed); routing + OIDC callback still open, F11 |
| `console.paridyum.com` is routed at all | `routing-table.yaml` | **no entry exists** — see F11 |
| `docket` registered as a product | `grep docket product-registry.yaml` | **no match** (F1 stands) |
| `region` on `tenants` | `grep region platform-tenancy/migrations/*.sql` | **no column** — migration written the same day, platform-tenancy#7, **not applied** |
| `audit_events` exists | `grep -rl audit_events migrations/` | **no table**, 18 days lossy — migration written the same day, platform-tenancy#6, **not applied** |
| The cache-key defect is fixed | `web-hiring/src/app/[lang]/careers/page.tsx:14` | `revalidate = 60` still there, tenant still the constant `paridyumTenantId()` (T-A12 stands) |
| `.claude/settings.json` in the three web repos | `ls` | **absent in all three** — **fixed later the same day**, T-A17 |
| The gate is clean | `scripts/check-knowledge.sh` | **zero errors, 51 warnings** — was 41 on 2026-09-16 |

**The warning count rose by ten while no task closed against it.** T-E4 is owed a count that is
measured rather than carried forward.

### F10 — `console-docket` has no image, and the reason is a failed build *(new)*

The plan recorded the console as "not deployed" and attributed it to the rename and to RFC-0044.
Both are true and **neither is the proximate blocker**. `gh run list --workflow publish.yml`
returns exactly one run, on 2026-09-14, and it **failed**:

```
✗ Lockfile failed supply-chain policy check (559 entries in 18.2s)
ERROR: process "…pnpm install --frozen-lockfile --ignore-scripts…" exit code: 1
```

`pnpm-workspace.yaml` sets `minimumReleaseAge: 1440`. So `ghcr.io/paridyum/console-docket`
has **no digest to deploy**, and the chart refuses to render without one. `web-dashboard`'s
equivalent pipeline **succeeds** (last green 2026-09-15).

**What it changes:** §4c's table is missing its first row. Before any rename, registration or RFC
acceptance, the console needs an image that builds.

> ### ⚠ F10's diagnosis was wrong, and the re-run disproved it — corrected 2026-09-20
>
> This finding said *"a re-run may be enough — the policy is an age floor and the packages have
> since aged six days."* A `workflow_dispatch` run on `main` was triggered on 2026-09-20
> ([run 35502636215]) and **failed identically.** The age hypothesis is dead.
>
> **The `✗ Lockfile failed supply-chain policy check` line is not the failure.** Build output
> continues past it — `Progress: resolved 454 … added 63` prints *after* it — and the process
> exits on something 0.9 seconds later:
>
> ```
> [ERR_PNPM_FETCH_401] GET https://npm.pkg.github.com/@paridyum%2Fauth: Unauthorized - 401
> No authorization header was set for the request.
> ```
>
> **The real cause: `console-docket` has no repository secrets at all.** Every other web
> repository carries `PARIDYUM_PACKAGES_TOKEN`; this one carries nothing, so `secrets: inherit`
> inherits an empty value, the Dockerfile's `if [ -s /run/secrets/node_auth_token ]` guard
> silently skips writing `.npmrc`, and `pnpm install` 401s on the private `@paridyum/auth`
> dependency.
>
> | Repository | `PARIDYUM_PACKAGES_TOKEN` |
> | --- | --- |
> | `web-dashboard`, `web-paridyum`, `web-admin`, `web-hiring` | present |
> | **`console-docket`** | **absent — no secrets at all** |
>
> **This is ADR-0045 risk R3 arriving in a new place.** GitHub Free has no organisation-level
> secrets for private repositories, so the token must be copied into every repository by hand and
> nothing reports one that was missed. `console-docket` is the newest web repository and the copy
> never happened.
>
> **Two things this exposes and does not fix.** The Dockerfile's `-s` guard converts a missing
> credential into a confusing failure 15 seconds later instead of an immediate one naming the
> secret — a guard written for "the secret is legitimately absent" applied where it never is. And
> the supply-chain line prints `✗` on a build that then continues, which is what made a
> five-second read of the log produce the wrong root cause twice.
>
> ### ✔ RESOLVED 2026-09-20 — and the 401 was only the outer layer
>
> `PARIDYUM_PACKAGES_TOKEN` was set on `console-docket` and `pnpm install` succeeded. The build
> then failed **again**, further along, on something no one had ever seen:
>
> ```
> playwright.config.ts(27,8): error TS2307: Cannot find module './e2e/support/environment'
> ```
>
> `.dockerignore` drops `e2e/` from the build context — correctly; a test harness does not belong
> in a runtime image — and kept `playwright.config.ts`, which imports from it. `tsconfig.json`
> includes `**/*.ts`, so `next build` type-checked a file whose import target the same
> `.dockerignore` had just removed. **One harness, half excluded.**
>
> **A blocked pipeline hides every failure behind it.** This finding was revised twice — the age
> policy, then the 401 — and each revision was a correct reading of the outermost error. That is
> the structural reason an estimate against a red pipeline is unreliable, and the argument for
> not leaving one red.
>
> **Result:** `ghcr.io/paridyum/console-docket` @ `sha256:ac84f7f9…ccf504d`, the console's first
> digest. T-A19 closed; [console-docket#2] carries the fix. The credential itself is debt, not a
> fix — **D-21**, a personal `gho_` token standing in for a role account (STD-0017 rule 1).
>
> [run 35502636215]: https://github.com/Paridyum/console-docket/actions/runs/35502636215
> [console-docket#2]: https://github.com/Paridyum/console-docket/pull/2

### F11 — Neither undeployed surface has a route, and they fail differently *(new)*

`routing-table.yaml` holds six live hostnames and **no entry for the console or the dashboard**.
The two absences are not the same shape and the plan treated deployment as the only gap:

- **The console** is specified at `console.paridyum.com` by ADR-0098 D3. That hostname has no
  record, no routing-table row, and — per `values-staging.yaml` — a deliberately unregistered
  redirect URI: the client is still the loopback `http://127.0.0.1:8471/callback` under the
  `ssh -L` model (ADR-0072), and the chart's own comment defers the web callback to **D-20**.
  So serving it is three changes, not one: an image (F10), a routing row, and a re-registered
  OIDC redirect.
- **The dashboard** is specified at `<host>/dashboard` by ADR-0100 D3 — a *path*, not a host, so
  it needs no routing row. But `web-dashboard` declares **no `basePath`** and its routes are
  `src/app/[lang]/{page,tenants,sign-in}`. Nothing in the repository serves `/dashboard`.
  Its `PARIDYUM_DASHBOARD_SITE_URL` is `https://paridyum.com`, which is `web-paridyum`'s host and
  already holds a release. **How two deployables share one hostname is undecided**, and the plan
  assumed a deploy where an ADR asks for a path.

**What it changes:** T-B7 ("assign the dashboard its OIDC client") is necessary and not
sufficient. The dashboard is blocked on a routing decision nobody has taken, not only on
`make zitadel-apps`.

### The ordering this re-audit does not change

**T-A14 (T-G.8, the `Release` read model) is still the right start**, and four more days of
untracked deploys is the argument for it rather than against. **T-A10 and T-A12 got worse by four
days each** and remain the only items in the plan that cannot be recovered by doing them later.

---

## Revision note — 2026-09-16

The first version of this plan said **"Nothing in the estate is deployed."** That was false, and
the way it was false matters more than the fact.

The paragraph after it said the `kind-pd-dev` context "exists and its API server refuses
connections" — which was true. **`kind-pd-dev` is not the estate's cluster.** `go-live.sh` at the
root of this repository, written 2026-09-14, says so in a comment:

> Point at the Talos cluster (reached over NetBird), not whatever dead local context kubectl last used.

`kubectl` was left pointing at a dead local context, the plan read it as the estate, and drew a
conclusion about the whole estate from it. Everything downstream — the §3 spine, T1 as the
"single highest-value unblock", §9's "nothing here has ever run" — follows from that one
unexamined `kubectl`.

**The cluster has been up for 14 days and nine releases are deployed.** §1 below is verified
against the live cluster, with the command that produced each line. Where a claim could not be
verified from here, it says so rather than guessing.

The second thing the revision fixes: **the plan omitted nine Genesis tasks**, including every
task in Wave 1 — the wave Genesis orders first on the principle the plan itself quotes. A plan
that restates *"irreversibility, not architecture"* and then drops the wave containing both ⏱
items is not a read of Genesis; it is a different plan wearing its name.

---

## 1. Where we actually are

**Verified 2026-09-16 against the live cluster**, not against manifests. The manifests are stale
— see F0b. Reproduction:

```sh
export KUBECONFIG=infra/infra-terraform/kubeconfig   # Talos `staging`, over NetBird
kubectl get nodes          # 5 nodes, Ready, 14d, v1.36.2
helm list -A               # 9 releases, all `deployed`
curl -o /dev/null -w '%{http_code}\n' https://<host>
```

| Surface | Repository | Helm release | Running | Serves |
| --- | --- | --- | --- | --- |
| **Landing** | `web-paridyum` | `paridyum-web/paridyum` rev **14** | 2/2, 14d | `paridyum.com` **307 → 200** |
| **Careers** | `web-hiring` | `paridyum-web/hiring` rev **4** | 2/2 | `careers.paridyum.com` **307 → 200** |
| **Tenant admin** | `web-admin` | `paridyum-web/admin` rev **2** | 2/2 | `admin.paridyum.com` **307 → 200** |
| **Identity** | `platform-identity` | `platform/identity` rev **4** | 2/2 | `id.paridyum.com` **302 → 200** |
| **Docket (engine)** | `tool-docket` | `platform/docket` rev **3** | 2/2, 41h | API, staff-only |
| **Dashboard** | `web-dashboard` | *(none)* | — | — |
| **Docket (interface)** | `console-docket` | *(none)* | — | `console.paridyum.com` → no response |

**Read the two codes, not the last one.** Every serving hostname answers its *first* request with a
redirect — a locale hop for the web surfaces, a login hop for the identity pair — so `200` alone
is the final page after following, not the host's own answer. This document first recorded these
as plain `200`, which was the same conflation of *reachable* with *serving what you expected* that
produced §1's original error, one order of magnitude down. Probe both:

```sh
curl -sS -o /dev/null -w '%{http_code}\n'    https://<host>   # the host's own answer
curl -sS -o /dev/null -w '%{http_code}\n' -L https://<host>   # where it lands
```

**Infrastructure also running and not mentioned by the original plan:** `zitadel` (the shared
issuer, v4.15.3), `zitadel-login`, `zitadel-login-control`, `cnpg` (the Postgres operator),
`longhorn` (the CSI driver T-2b.3 chose), `cilium` (CNI).

**So: five of seven surfaces are deployed, four serve the public, and the infrastructure that
T-2b.3, T-2b.5 and T-2b.6 describe as "chosen but not yet run" has been running for ten days.**

### Phase 2b has evidence, and nothing says so

`TASKS.md` reads as though the substrate were still ahead. It is behind — the tasks below each
name a `done when` that a running system now satisfies. They are listed here as **candidates to
close on their own evidence**, not as claims: each should be closed by running its `done when`
line, which is the rule this revision exists to enforce.

| Task | Its `done when` | Evidence visible from here |
| --- | --- | --- |
| T-2b.3 | *one driver is chosen and runs on the staging cluster* | `longhorn-system` ns, 10d, `longhorn-1.12.1` deployed |
| T-2b.5 | *`kubectl get nodes` shows five* | five `staging-*` nodes, Ready, 14d — drain/rejoin unverified |
| T-2b.6 | *no credential in plaintext; the mechanism metal will use* | `.sops.yaml` in four infra repos, with encrypted secrets in use |
| T-2b.7 | *a hostname resolves through the edge to a service on the cluster* | four hostnames answer `200` via Cloudflare; `edge-in-01` per `go-live.sh` |
| T-2b.8 | *a request from the public internet reaches a pod and returns* | `paridyum.com` — the whole chain, since ≥2026-09-05 |

**T-2b.5's second clause is the one to check before closing it** — a control-plane node drained
and rejoined is a different claim from five nodes being Ready, and only the first proves the
topology. T-2b.4 is not in this table: `infra-terraform` holds no `*.tf` at its top level, so
whether the guest is created by OpenTofu or by hand was not established from here.

### What is genuinely not done

| Thing | Status |
| --- | --- |
| `web-dashboard` deployed | **no** — image pipeline exists, `helm/values.yaml` `digest: ""`, no release |
| `console-docket` served | **no** — no release; ADR-0098 D3 specifies `console.paridyum.com`, which does not answer |
| `Release` as a schema + read model | **no** — nothing in the estate records what is deployed (T-G.8) |
| `region` on `tenants`, immutable | **no** — T-G.3; ADR-0039 step 1 and ADR-0041 D3 both require it |
| Correlation id in the audit stream | **no** — T-G.2; permanently lossy, and traffic is live |
| Off-host backup with a stated RPO | **no** — T-G.7; `0001_tenancy_foundation.sql` still gates real applicants on it |
| The `product:` declaration | **no** — T-G.6, F3 |
| The two renames | **no** — F2, and it is now two renames, not one |
| `docket` in the product registry | **no** — F1 |

**The single highest-value unblock is no longer the cluster. It is T-G.8** — because the estate
has nine running releases that nothing records, and every remaining task is written against a
picture of the estate that does not exist. Genesis says this in its own words: *"Record what is
already deployed before deploying anything."* That sentence was written for exactly this state.

---

## 2. Findings

F1–F4 are the original plan's, verified. F0, F0b, F5–F9 are new. Each says what it changes.

### F0 — §1's premise was false, and `kubectl` read the wrong cluster *(new)*

Covered in the revision note. **What it changes:** the §3 spine's root ("a cluster exists and
accepts a deploy") has been true since 2026-09-02. T1 is deleted. T10 ("deploy
`platform-identity` — first real deploy") is already done, at revision 4. §9's central risk
("nothing here has ever run") is inverted: things have run for two weeks, and the risk is that
**nothing that governs them knows it**.

### F0b — The estate's own records agree with the plan and disagree with the cluster *(new)*

This is the more serious finding. It is not one document.

| Record | Says | Reality |
| --- | --- | --- |
| `web-paridyum/manifest.yaml` | *"nothing is deployed and no visitor has seen it"* | serving since ≥2026-09-05 |
| `web-dashboard/LOGBOOK.md` | *"No cluster is running."* | 5 nodes, 14d |
| `infra-edge/dns/routing-table.yaml` | zone `paridyum.com` `status: live`; most hosts `planned` | four hostnames answer |
| `paridyum.com` itself | 200, Next.js, `x-edge-cache: HIT` | — |

**The record drifted from the estate, and the plan — whose header says "Verified, not
remembered" — inherited the drift by reading the record.** The estate's founding rule is that a
green check is unproven until you have watched it fail; this is the same failure one level up,
and it is the strongest argument in this document for T-G.8 existing.

**What it changes:** every remaining task's "done when" must be checked against the cluster, not
against a manifest. And reconciling the record with the estate is itself work — see T-A7.

### F1 — `docket` is not a registered product, so it cannot be sold *(verified)*

`product-registry.yaml` holds seven products — `paridyum`, `entitlement`, `identity`, `routing`,
`tenancy`, `hosting`, `veer-entitlements`. `docket` is **not among them** (`grep -n docket
product-registry.yaml` returns nothing).

**Two corrections to the original wording.** The field is `stage`, not `lifecycle`; and
`catalogue` / `capability` are values of `kind`, alongside `platform`. The registry's own header
is the authority on what that means:

> **catalogue** — the only kind that owes an MKT- before it may be built.

**What it changes:** nothing about the conclusion. "Docket as a service" is a registration that
has never happened, and the first task is a registry entry plus a `kind` decision — a commercial
decision wearing a registry field, and the operator's.

### F2 — Two renames, not one, and one of them is tracked nowhere *(verified, then extended)*

**ADR-0098 is `accepted`** (2026-09-15) and its decision 1 reads:

> **The HR operator interface is a served `web-` product.** `console-docket` is reclassified and
> renamed to **`web-docket`**.

The directory is still `consoles/console-docket`; no `web-docket` exists anywhere.

**The extension the original plan missed.** `tool-docket` carries the identical defect. Its
manifest declares `class: platform` ("RECLASSIFIED by ADR-0093 §3") behind a `tool-` prefix that
C-0005/C-0007 map to `tool`. So the estate has **two** accepted-decision-vs-disk-name divergences,
and a rename of one leaves the pair inconsistent — RFC-0044's `applies_to` already names
`web-docket` while the other half would still be `tool-docket`.

**And one of them cites its own debt entry by the wrong number.** `tool-docket/manifest.yaml` says:

> NAMING TENSION, tracked as decision debt (`DECISION-DEBT.md` D-58)

**The entry is D-64, not D-58** — *"docket is class `platform` but still named `tool-docket`"* —
and it is exactly the record the manifest describes: ADR-0093 §3, the prefix↔class mismatch, the
clean resolution, and the rename as an unscheduled breaking operation. `58` and `64` are a digit
transposition, so this is a wrong pointer rather than a missing record.

**Correction to this finding's first draft, which said the rename was "tracked nowhere."** That
was wrong, and it was wrong the same way §1 was: a `grep` was run, its output was truncated by
`head`, and a conclusion about the whole file was drawn from the visible part. The entry sat
twenty lines below the last line read. Recorded rather than quietly repaired, because the failure
is the one this document exists to describe.

**`console-docket`'s equivalent (D-18) is correct**, and the pair together is the point: the two
renames are one change or neither (STD-0022), and only one of the two pointers to them was right.

**The defect class is invisible to every gate.** `repo.no_dangling_decision_references` matches
only knowledge artifact-id patterns (`ADR-`, `C-`, `STD-`, …) and explicitly excludes
`DECISION-DEBT.md`; `D-NN` is neither. A wrong `D-` citation cannot fail any current check, so
the green run on this repository is not evidence that the citation is correct.

**What it changes:** T6 becomes two atomic renames, and a new task fixes the dangling citation —
which is an ADR-0011 failure independent of whether the rename happens now.

### F3 — Four manifests declare no `product:` *(verified)*

`web-paridyum`, `web-hiring`, `tool-docket` and `console-docket` have no `product:` field.
`web-admin` declares `product: [identity]` and `web-dashboard` declares `product: [tenancy]`.

Under ADR-0050 an absent value means *every* product. Under RFC-0048 a public surface is a
tenant's surface, and under RFC-0049 a tenant is a set of Releases — so a surface with no product
cannot be a Release, cannot be entitled, and cannot be deployed *as* anything.

**What it changes:** T-G.6 is a prerequisite for T-G.8, not a tidy-up. The original plan had this
right.

**But three of the four cannot be fixed by declaring anything, and the plan assumed they could.**
A declared slug is checked against the registry by `knowledge.product_slugs_resolve`, and RFC-0048
§D assigns `web-hiring` a *hiring/careers* product while `product-registry.yaml` holds no such
entry — with RFC-0048's own open question 2 leaving the choice open. `tool-docket` and
`console-docket` are blocked on `docket` being registered at all (F1). So **one of the four is a
task and three are a decision.** T-G.6 read as a tidy-up because the four were grouped; they are
three different blockers, and this was found by a lane that refused to guess a slug rather than by
reading the task.

### F4 — Eight `proposed` artifacts — but the gate's warning is about twelve different ones *(verified, corrected)*

All eight are `proposed` and none is accepted: RFC-0045 through RFC-0051 and ADR-0103, all
created **2026-09-15**.

**The correction is in the reasoning, not the count.** The original attached the gate's judgement
to this list, quoting:

> … *"an RFC nobody closed is a decision being made by default."*

**That warning is about a different set of artifacts.** Those the gate names are `GLO-0007`,
`GLO-0008`, and `RFC-0014` through `RFC-0023` — all `proposed` for over 30 days. None of §5's
artifacts is among them; §5's were created 2026-09-15, the day before this revision. The gate's
set includes
**RFC-0023** (the substrate) and **RFC-0016** (the rented edge) — the two that describe the
infrastructure that has been running for fourteen days, and which T-2b.2 marks ⛔ with the note
that *"accepting an RFC is not an agent's act."*

**What it changes:** §5's ordering survives, but the plan was quoting a warning about a different
set of documents — and the documents it was actually warned about are the ones governing the
cluster it did not know existed. Both sets are now in §5.

### F5 — RFC-0044 is missing from §5 *(new)*

**RFC-0044, "docket as a service completes the platform against ADR-0042, tenant zero first"**,
is `proposed`, created 2026-09-15. Its `applies_to` lists `tool-docket` **and `web-docket`**. It
is *the* docket-as-a-service RFC — the one whose title is the original plan's §4c — and §5 never
mentions it.

**What it changes:** §5's Set A is one artifact longer than the original claimed. RFC-0044 is the
design that F1's registration and F2's renames are downstream of, so it belongs before them.

### F6 — Nine Genesis tasks are absent, including all of Wave 1 *(new)*

The plan cites T-G.0d, T-G.0e, T-G.6, T-G.8, T-G.11–T-G.18. It omits:

| Omitted | Why it matters |
| --- | --- |
| **T-G.0** ⛔ | authorise the sixth bounded context — *"blocks waves 2, 3 and 4 entirely"* |
| **T-G.0b** ⛔ | is there a second failure domain — *"C2, C3, BYOK and every stamp in RFC-0047"* |
| **T-G.0c** ⛔ | flat or nested — blocks RFC-0047 |
| **T-G.1** ⏱ | the cache key — *"**the only latent defect in Genesis**"* |
| **T-G.2** ⏱ | one correlation id, one append-only audit stream — *"**permanently lossy**"* |
| **T-G.3** ⏱ | `region` on `tenants`, immutable — lossy once there are two regions |
| **T-G.4** | clear the red gate — see F7 |
| **T-G.5** | the `operator` ambiguity — *"every design above inherits it"* |
| **T-G.7** ⛔ | off-host backup with a stated RPO — *"**the largest task in Genesis and the one everything waits on**"* |

The plan quotes Genesis's ordering principle verbatim — *"irreversibility, not architecture"* —
and then drops the wave that principle puts first. **T-G.2 and T-G.3 are now retrofits against a
live system rather than pre-deploy work**, which is the cost the omission bought.

### F7 — T-G.4 is already done *(new)*

T-G.4 says *"three errors stand on every run."* It does not:

```
$ scripts/check-knowledge.sh
Knowledge check passed (41 warning(s))
```

Zero errors. The task's own text anticipates this — *"run the gate for the current list rather
than trusting this line (ADR-0035)"* — and the line is stale. **What it changes:** T-G.4 closes
on evidence rather than on work. The plan's §1 claim of *"the knowledge gate at zero errors"* was
**correct**; the plan was right about the gate and wrong about the cluster.

### F8 — Lane L4's verification rule cannot apply to `platform-tenancy` *(new)*

§6 of the original says *"a lane is done when its repository's own CI is green."* L4 owns
`platform-tenancy`, which is **not a repository.** Its own README:

> ## This is not a repository yet, and saying so is the point

No `manifest.yaml`, no crate, no `API.md`, no `RUNBOOK.md`, no CI. It exists as a directory
because `0001_tenancy_foundation.sql` had to live somewhere `check-migrations.sh` scans.

**What it changes:** either L4's verification is restated (tests + a tenant row reaching
`active`), or promoting `platform-tenancy` to a real repository becomes an explicit task before
the lane can be assigned. The plan currently assigns an agent to a lane with no CI to pass.

### F9 — The `knowledge-audit` skill hardcodes the prefix list *(new)*

`paridyum-platform:knowledge-audit` prescribes this sweep:

```sh
--jq '.[] | select(.name | test("^(platform|svc|lib|tool|infra|spec|template)-") | not)'
```

Seven prefixes, restated by hand. `class-registry.yaml` declares more than that — the pattern
omits `web-`, `console-`, `mobile-` and `sdk-`. Run as written it reports `web-dashboard`,
`web-hiring`, `web-paridyum` and `web-admin` as C-0002 violations. All four are compliant.

This is the defect `class-registry.yaml`, `check-registry.yaml`, `artifact-registry.yaml` and
`product-registry.yaml` each exist to kill — *"a list that must be extended, hardcoded in a place
extending it is expensive"* — and it has gone stale by five prefixes, inside the plugin that
audits for exactly that.

**What it changes:** the sweep is corrected in §6 below. Fixing the skill itself is a change to
`paridyum-platform`, which is a knowledge artifact — **T-A8**.

---

## 3. The dependency spine

Corrected. The original's root is struck because it is already true and has been for two weeks.

```
 ~~A cluster exists and accepts a deploy~~        ← TRUE SINCE 2026-09-02. Not a blocker.
        │
        ├── ~~platform-identity runs~~            ← RUNNING, rev 4, id.paridyum.com 200
        │
        ├── ~~landing / careers / admin deploy~~  ← RUNNING, serving the public
        │        │
        │        └── and nothing records any of it  ◄── THE ACTUAL BLOCKER (T-G.8)
        │
        ├── platform-tenancy runs as a repository ── it is a directory, not a repo (F8)
        │        │
        │        └── the provisioning saga executes (ADR-0039)
        │
        ├── Release is a schema + read model (T-G.8)  ◄── start here
        │        │
        │        └── a surface can BE a Release  ← needs F3 (product: declared)
        │
        └── the deploy saga runs one product end to end (T-G.13)
                 │
                 ├── dashboard      as a Release  ← needs T-G.0, and T11 (OIDC client)
                 └── web-docket     as a Release  ← needs F1 (registered) + F2 (renamed)
                                                  + RFC-0044 accepted
```

**The spine's only irreversibility is at the top.** Two weeks of live traffic are being written
with no correlation id (T-G.2) and no recorded deployment (T-G.8). Both are cheap now and
neither is recoverable later.

---

## 4. The three surfaces

### 4a. Landing — `web-paridyum`

**Deployed and serving.** Revision 14, `paridyum.com` → 200, `x-powered-by: Next.js`,
`x-nextjs-prerender: 1`. This is the surface the original plan called "not deployed" and
"nearest to done" — both halves were right about the work and wrong about the state.

| Needs | Why | Genesis |
| --- | --- | --- |
| `product:` on the manifest | F3 — cannot be a Release otherwise | T-G.6 |
| Its manifest corrected | it says *"nothing is deployed"*; it is serving | T-A7, F0b |
| Content schema landed | RFC-0048: content is reviewed, not a CMS | T-G.11 |
| Resolves its tenant from the request | it is *Paridyum's* landing, not *the* landing | T-G.12 |
| A Release record | so a second tenant can hold the same product | T-G.8, T-G.13 |

**Still the thing that makes it a service rather than a website:** the same deployable, resolving
a different tenant per host. If `paridyum.com` is special in the code, it is not a product. The
routing table already reasons about this correctly — the `paridyum.com` zone is `role: marketing`,
*"Not tenant-serving. Carries no tenant data, so residency does not bind it."*

### 4b. Dashboard — `web-dashboard`

**Not deployed, and the only surface where the original plan's state reading holds.** Four
screens, both locales, a publish workflow and a Helm chart; `helm/values.yaml` `digest: ""`.

> ### ⚠ 4b's blocker was wrong — measured 2026-09-21
>
> This section said the dashboard needs *"OIDC client assigned — blank and the pod will not
> start"*. **All three claims are false, and the third is a safety guarantee that is not there.**
>
> **1. It is not declared, so `make zitadel-apps` would assign it nothing.** `zitadel-apps-plan`
> reports three applications — `web-hiring`, `console-docket`, `web-admin` — and `web-dashboard`
> is not among them. Nothing in `applications.yaml` declares it.
>
> **2. It has no authentication code at all.** No `src/lib/auth/`, and no `api/auth/{login,
> callback,logout}` — where `web-admin` has all three. Its own sign-in form says so:
> *"The OIDC redirect belongs to platform-identity and is not wired yet."* Assigning a client
> would hand a credential to code that does not read it.
>
> **3. "The pod will not start" is false.** `src/` contains **no `process.env` reads whatsoever**,
> so nothing consumes `PARIDYUM_OIDC_CLIENT_ID` and nothing fails on its absence. The pod would
> start happily and serve an unauthenticated surface — which is the opposite of the protection the
> chart comment claims, and the same defect as every other one found this week: a guarantee
> asserted in a comment and enforced by nothing.
>
> **So the dashboard is not blocked on provisioning. It is blocked on the BFF being unwritten** —
> `web-admin`'s `src/lib/auth/*` and `bff.ts` are the model, and ADR-0098 already names them as
> the pattern. That is a build, not a `make` target, and it is why T-B7 as written cannot close.

| Needs | Why |
| --- | --- |
| ~~OIDC client assigned~~ **the auth BFF written** | see the box above — there is no auth code, and the client is not declared |
| `platform-tenancy` reachable | `listTenants()` returns `[]` today, honestly |
| `Release` read model | T-G.8 — the tenant detail view has nothing to show without it |
| **T-G.0 answered** | ⛔ authorises the sixth context; §6's lane for it is otherwise unassignable |
| RFC-0050 + RFC-0051 accepted | F4 |

**Already correct, and worth not re-litigating:** provisioning is a screen rather than a spinner;
a failed tenant sits beside healthy ones; nothing is preselected in the placement control;
thirteen jurisdictions read *On request*.

### 4c. Docket as a service — `tool-docket` + `console-docket`

**The engine is deployed and serving** (`platform/docket`, rev 3) — which the original plan did
not know. The interface is not. What remains is genuinely mostly not code, and the original was
right about that.

| Needs | Why | Status |
| --- | --- | --- |
| **RFC-0044 accepted** | it is the design this whole section is downstream of (F5) | `proposed` |
| Registered in `product-registry.yaml` | F1 — ADR-0050; unregistered means unsellable | not started |
| `kind` decided — `catalogue`, `capability` or `platform` | decides whether C-0006's evidence obligation binds | **operator** |
| MKT- evidence, if `catalogue` | C-0006: evidence not derived from a tenant we own | not started |
| `console-docket` → `web-docket` | F2 — ADR-0098 D1 | **not started** |
| `tool-docket` → `platform-docket` | F2 — ADR-0093 §3; tracked as D-64 | **not started** |
| The account silo retrofit | docket is multi-tenant *within* a tenant (GLO-0004) | T-G.0d — open |
| `product:` on both manifests | F3 | T-G.6 |

**If `docket` registers as `catalogue` it cannot reach `building` on the strength of Paridyum
using it.** C-0006 requires evidence from outside, BET-0004 is the wager that the catalogue is
the product, and Paridyum is the tenant under test in this very plan — so Paridyum's own use is
disqualified by construction.

---

## 5. Governance — what must be accepted

**Eleven artifacts, in two sets.** The second set is the one the gate is actually complaining
about (F4).

### Set A — Genesis's artifacts

| # | Artifact | Decides | Blocks |
| --- | --- | --- | --- |
| G1 | **ADR-0103** | containment as an entitlement attribute | RFC-0045, every placement |
| G2 | **RFC-0045** | the isolation envelope — placement, class, attestation | the deploy saga's shape |
| G3 | **RFC-0049** | a tenant is a set of Releases | T-G.8, every surface-as-a-Release |
| G4 | **RFC-0048** | a public surface is a tenant's surface | landing, careers |
| G5 | **RFC-0044** | docket as a service (F5 — missing from the original) | F1, F2, T-G.15–T-G.17 |
| G6 | **RFC-0047** | renting the platform is a stamp | whether a tenant can hold `app` |
| G7 | **RFC-0050** | the dashboard's first-run flow | the dashboard's screens |
| G8 | **RFC-0051** | what the placement control may ask | the create-tenant screen |
| G9 | **RFC-0046** | what BYOK does and does not give | the attestation's honesty |

**Order matters for G1 → G2 → G3 only.** The rest are independent.

### Set B — what the gate actually flags

Twelve artifacts `proposed` for **over 30 days** — the ones the original plan quoted the warning
about while listing different documents.

| Artifact | Why it now matters here |
| --- | --- |
| **RFC-0023** ⛔ | the substrate — **already built and running 14 days** (T-2b.2) |
| **RFC-0016** ⛔ | the rented edge — **already live** (`edge-in-01` + Cloudflare) |
| **RFC-0019** | one Proxmox cluster / Kubernetes above it — T-2b.4 |
| **RFC-0014, RFC-0017** ⛔ | the region model — T-1.6; `region` is T-G.3 |
| **RFC-0018** ⛔ | is the control plane one repository — T-1.5 |
| **RFC-0020** | external facts cited and dated — 20+ unresolved research citations |
| **RFC-0021** | device hostnames |
| **RFC-0022** ⛔ | the operator competes with its tenants — T-0.11 |
| **RFC-0015** | a fact has one home |
| **GLO-0007, GLO-0008** | control region; datacentre |

**RFC-0023 and RFC-0016 are the sharpest.** The cluster they describe has been serving production
traffic for fourteen days while both remain `proposed`. T-2b.2's note — *"accepting an RFC is not
an agent's act"* — is correct and is why this sits in the operator's column. But the estate is
now in the inverse of the state the governance exists to prevent: not code written against no
design, but **a design never accepted for code that is already in production.**

**A rejection is cheaper than a silent default.** If RFC-0051 is wrong about refusing a sizing
control, saying so now costs one screen. Saying so after tenants have used it costs a migration.

---

## 6. Lanes, and what verifies each

| Lane | Owns | **Verified by** | Depends on |
| --- | --- | --- | --- |
| **L1 · Runtime record** | T-G.8 — `Release` schema + read model, seeded from the live cluster | a `Release` row per running release, read from the cluster, not typed | nothing — **start here** |
| **L2 · Governance** | §5 Set A + Set B | `check-knowledge.sh` at zero errors | nothing — parallel with L1 |
| **L3 · Renames** | `console-docket`→`web-docket`, `tool-docket`→`platform-docket` | STD-0022 completeness: `grep -r` for each old name returns nothing live | nothing — **one atomic change each** |
| **L4 · Estate hygiene** | the three repos' missing files; the D-58→D-64 citation | `check-repo.sh --root .` green | nothing |
| **L5 · Lossy retrofits** | T-G.1, T-G.2, T-G.3 | the assertion fails before the fix and passes after | nothing — **time-critical** |
| **L6 · Docket product** | F1 registration, RFC-0044, the account silo | a `docket` row in the registry; its CI | `kind` decision, RFC-0044 |
| **L7 · Tenancy** | `platform-tenancy` promoted to a repository, then the saga | its own CI **once it has one** (F8) | T-G.0 |

### What must NOT be parallelised

- **§5 Set A is not one agent each.** G1→G2→G3 is a chain; accepting RFC-0045 without ADR-0103
  settled decides ADR-0103 by implication. One reviewer, in order.
- **Each rename is one atomic change** (STD-0022). Splitting one across agents is how a rename
  leaves the quiet references behind — which is precisely the state `tool-docket` is already in.
- **Nothing writes `product-registry.yaml` concurrently.** It is the source of truth (ADR-0035).
- **Nothing is accepted by an agent.** Every row in §5 is the operator's act. A lane that
  "accepts" an RFC has decided a commercial question by assumption.

### A lane is done when its evidence exists — not when an agent says so

The original said *"done when its repository's own CI is green."* That rule does not hold for L1
(the evidence is a cluster), L5 (the evidence is a test failing then passing), or L7 (there is no
CI — F8). Restated: **a lane is done when its "done when" line has been executed and its output
attached.** Given F0b, a claim of done that was not run is the exact failure this revision exists
to correct.

### Corrected naming sweep

The `knowledge-audit` skill's command omits four live prefixes (F9). Run this instead — it reads
the prefix list from the registry rather than restating it:

```sh
python3 - <<'PY'
import yaml, subprocess, json, re
prefixes = [c['prefix'] for c in yaml.safe_load(open('class-registry.yaml'))['classes']]
pat = re.compile('^(' + '|'.join(re.escape(p) for p in prefixes) + ')')
repos = json.loads(subprocess.check_output(
    ['gh','repo','list','Paridyum','--limit','200','--json','name,visibility']))
for r in repos:
    if not pat.match(r['name']) and r['name'] != 'knowledge':
        print(f"NAMING: {r['name']} ({r['visibility']}) does not match C-0002")
PY
```

---

## 7. The to-do, ordered

Every task states **what**, **why it blocks**, and **done when**. A task without a falsifiable
completion condition is the failure ADR-0018 names.

### Now — nothing blocks these

- [ ] **T-A1. Open PRs for the two pushed branches** — `knowledge/feat/supply-chain-posture-check`
      and `infra-ci/fix/nextjs-hygiene-root`. The second turns three repositories red on merge,
      which is the point. *Lane L4.*
      **Done when:** both PRs are open and CI has reported on each.
- [x] **T-A2. Fix the three repositories the hygiene gate was hiding.** `web-admin`, `web-hiring`
      and `web-paridyum` each lack `TESTING.md` (STD-0008) and `DECISION-DEBT.md` (ADR-0011).
      *Done 2026-09-16 (commits `05bfbf8`, `8fd241b`); verified by `ls` on 2026-09-20 — all six
      files present. Lane L4.*
      **Done when:** `check-repo.sh --root .` reports green for all three.
      **The task's third clause was false when written and is the reason T-A17 exists.** It said
      *"all three have `.claude/settings.json`"*; none of them does, then or now. T-A17 was raised
      separately against the same three repositories, so the plan asserted a file present in one
      task and absent in another — and only the second was checked.
- [ ] **T-A3. Rename `console-docket` → `web-docket`**, atomic per STD-0022: directory,
      repository, `package.json`, `manifest.yaml` (including `class`), the Zitadel application and
      redirect URIs, every registry, every doc header, every cross-reference. ADR-0098 D1 orders
      it; D-18 tracks it. *Lane L3.*
      **Done when:** `grep -r 'console-docket'` across the estate returns only superseded
      artifacts and this plan.
- [ ] **T-A4. Rename `tool-docket` → `platform-docket`**, same rule. ADR-0093 §3 reclassified it;
      F2 shows the debt is tracked nowhere. *Lane L3.*
      **Done when:** as T-A3, and `platform-docket` carries a real debt entry for anything left.
- [x] **T-A5. Correct the debt citation in `tool-docket/manifest.yaml`** — it cited
      `DECISION-DEBT.md` D-58, a digit transposition of **D-64**, which is the naming entry and
      always was. One token, one line. *Done 2026-09-16. Lane L4.*
      **Done when:** the manifest cites an entry that exists — **met**, verified by reading D-64.
      **Two things this exposed and did not fix, recorded rather than closed:**
      `repo.no_dangling_decision_references` cannot see a wrong `D-` citation at all (it matches
      knowledge artifact ids and excludes `DECISION-DEBT.md`), so no gate would have caught this;
      and `tool-docket`'s vendored `.paridyum/knowledge/standards/` stops at **STD-0014**, so
      STD-0022 — the standard that governs the rename D-64 describes — is not readable from that
      checkout. Both belong to T-A7.
- [x] **T-A6. Declare `product: [paridyum]` on `web-paridyum`.** *Done — commit `31f6a91`,
      "feat(manifest): declare product: [paridyum], and record the deployment". Verified
      2026-09-20: the manifest declares it, and `check-knowledge.sh` passes 18 `product:`
      references with zero errors. T-G.6, F3. Lane L4.*
      **Done when:** the manifest declares the product, justified by RFC-0048 §D (*"`web-paridyum`
      → `[paridyum]`"*), and `knowledge.product_slugs_resolve` passes — **met.**
      **T-G.6 does not close with it.** The task covers `web-hiring` too, and that half is the
      decision this entry already described: no hiring/careers product is registered, and
      registering one is an operator act under C-0006.
      **This task named three repositories it cannot cover, and a lane proved it by refusing to
      guess.** `knowledge.product_slugs_resolve` checks the declared slug against
      `product-registry.yaml`, so a slug that is not registered fails the gate:
      - **`web-hiring` is blocked exactly as the docket pair is, and the original grouping missed
        it.** RFC-0048 §D assigns it *"the hiring/careers product"* — deliberately **not**
        `paridyum` — and no such product is registered. RFC-0048's **open question 2** asks
        precisely this (*"Is `careers` its own catalogue product, or the public half of the hiring
        product?"*) and leaves it open, and RFC-0048 is itself `proposed`. Registering one is an
        operator act owing an `MKT-` under C-0006, gated by anti-goal 6.
      - **`tool-docket` and `console-docket` are blocked on T-B3** — `docket` is not registered
        (F1). They join the work at T-B3's done-when, not here.
      So `web-paridyum` is the only one of the four that can be done today, and the other three
      are the *same* blocker wearing three names: **a surface cannot declare a product that has
      not been registered.** That is a decision, not a task.
- [ ] **T-A7. Reconcile the record with the estate.** Correct the stale "nothing is deployed"
      claims in the deployed repos' manifests and logbooks, and the routing table's `planned`
      entries for hosts that answer. *F0b. Lane L4.*
      **Done when:** no live repository claims the estate is undeployed, and the routing table
      matches `curl`. **Progress 2026-09-16:** routing table corrected (six hostnames + the region
      block, both response codes recorded); `web-paridyum`, `web-hiring` and `web-admin` corrected.
      **`web-dashboard` is deliberately excluded.** Its manifest carries the same sentence —
      *"nothing is deployed and no visitor has seen it"* — and for that repository **it is true**:
      it has no Helm release. The sentence is identical; the claim is not. This task is about
      checking each repository against the estate, not grepping for a phrase.
- [x] **T-A16. `web-admin/SOP.md` is titled `# SOP — web-paridyum` and releases the wrong
      product.** *Done 2026-09-20 — [web-admin#1] and [web-hiring#3].*
      **It was two repositories, not one, and the task's own last line is what found the second.**
      `web-hiring` carried the identical inheritance — same title, same `helm upgrade web-paridyum`,
      same `paridyum.com/en` verification. `web-dashboard` and `web-paridyum` are correct.
      **The hazard was worse than a wrong name.** Namespace `paridyum-web` holds three releases, so
      the inherited command would not have errored — it would have replaced the landing page's
      workload with the following repository's chart, successfully. Both SOPs now say to check
      `helm list` first.
      `web-admin`'s was rewritten rather than patched: most of it described work that surface does
      not do — posting jobs from a `jobs/` directory it does not have, and checking copy against
      C-0006's forbidden framings, which govern public market claims an authenticated operator
      surface does not make. `web-hiring`'s "Posting a job" is current (ADR-0095) and was left
      alone, because rewriting a correct section is how a good document acquires a regression.

[web-admin#1]: https://github.com/Paridyum/web-admin/pull/1
[web-hiring#3]: https://github.com/Paridyum/web-hiring/pull/3
[web-paridyum#1]: https://github.com/Paridyum/web-paridyum/pull/1

- [ ] **T-A16 original text.** `web-admin/SOP.md` is titled `# SOP — web-paridyum` and releases the
      wrong product. Its procedure is `helm upgrade web-paridyum ./helm -n paridyum-web` verifying
      `https://paridyum.com/en` — a copy-paste from the repository it was forked from, with no
      disclaimer banner, unlike the README. STD-0013 governs; `check-repo.sh` checks that an SOP
      *exists*, never that it is about this repository. **The most consequential remaining
      falsehood in that repo** — an operator following it deploys the landing page over admin.
      *Lane L4.*
      **Done when:** the SOP names `web-admin` throughout and its verification step targets
      `admin.paridyum.com`. Check the other forks for the same inheritance.
- [x] **T-A17. `.claude/settings.json` is absent from `web-admin`, `web-hiring` and
      `web-paridyum`.** *Done 2026-09-20 — [web-paridyum#1], [web-admin#1], [web-hiring#3].*
      All three now declare the marketplace and `check-repo.sh --root <repo>` reports zero
      marketplace warnings for each. **This was the third instance of a mechanism reading as
      installed while never having fired** (T-2.5, T-2.13, T-2.15) and the first where the file
      holding the declaration did not exist at all.
      **T-A2 asserted these files were present while this task was raised for their absence**, and
      only the second was ever checked — recorded in that task rather than repaired silently.

- [ ] **T-A17 original text.** `.claude/settings.json` is absent from `web-admin`, `web-hiring` and
      `web-paridyum`. ADR-0007 layer 1 — the hook that denies a write to a locked artifact — is
      declared in that file, and without it *none* of the three protected paths is enforced in
      those repositories; `CLAUDE.md` states this plainly. The file existing is also what offers
      the plugin install to a contributor who trusts the folder. *Lane L4.*
      **Done when:** all three declare the marketplace and `check-repo.sh` reports zero errors.
- [ ] **T-A18. The vendored snapshots are stale, and each is stale differently.** `web-paridyum`'s
      stops at **ADR-0056 / RFC-0035** — so RFC-0048, which assigns it its `product:` value, is
      not readable from the checkout that now cites it. `web-admin`'s stops at **ADR-0087**;
      `tool-docket`'s stops at **STD-0014**, so STD-0022 — governing the rename its own D-64
      describes — is absent. `forge knowledge sync` is the mechanism (ADR-0001). *Lane L4.*
      **Done when:** each snapshot is current and `forge knowledge sync --check` is clean.
      **The pattern, not the instances, is the finding:** ADR-0001 duplicates the knowledge base
      into every repository so an agent can read every rule that binds it offline. Each snapshot
      is the age of the last sync, and nothing surfaces that age — `check-repo.sh` reports a
      snapshot missing, never a snapshot behind. Three repositories were citing artifacts they
      could not read, and each was found by an agent noticing mid-task rather than by a gate.
- [ ] **T-A19. Get `console-docket` to produce an image.** Its only `publish.yml` run failed on
      the pnpm supply-chain age policy, so no digest exists and the chart cannot render. *F10.
      Lane L3/L4.* **The cheapest open item in the plan.** Re-run first — the policy is a 1440-minute
      age floor and the lockfile's 559 entries have since aged six days — and only if it fails
      again is this a lockfile change rather than a re-run.
      **Done when:** a `publish.yml` run is green and `helm/values-staging.yaml` can be given a
      real `--set image.digest=sha256:…`.
- [ ] **T-A20. Decide how the dashboard is reached.** ADR-0100 D3 specifies `<host>/dashboard`;
      `web-dashboard` declares no `basePath`, serves `/[lang]/…`, and points
      `PARIDYUM_DASHBOARD_SITE_URL` at `https://paridyum.com` — a hostname `web-paridyum` already
      holds. Two deployables, one origin, no decision. *F11.*
      **Done when:** either the repository serves under `/dashboard` and the edge routes that path
      to it, or an artifact records that it gets a hostname instead and `routing-table.yaml` gains
      the row. **Not an agent's call where it changes what a hostname means** — but the absence of
      any answer is what makes T-B7 insufficient.
- [x] **T-A8. Correct the `knowledge-audit` skill's prefix sweep** — *done 2026-09-21.* The
      prefix list already came from `class-registry.yaml`; **running it was the part nobody had
      done**, which is what the done-when asked for. 20 repositories, 18 live, 11 prefixes: the
      four `web-*` repos this task was raised for are clean, and the single flag was
      `pd-coming-soon` — **archived** since 2026-05-01 and superseded by `web-paridyum`. An
      archive is what the estate *used* to name and cannot be renamed without unarchiving it, so
      it would have been flagged forever. The sweep now excludes archived repositories and
      reports **zero findings**.

- [ ] **T-A8 original text.** Correct the `knowledge-audit` skill's prefix sweep to read `class-registry.yaml`
      rather than restating seven of twelve prefixes. *F9. Lane L2.*
      **Done when:** the sweep reports zero false positives across the org, demonstrated by
      running it.
- [ ] **T-A9. Close T-G.4.** The gate is at zero errors; the task's line is stale. *F7. Lane L2.*
      **Done when:** the task is marked done in `TASKS.md` with the gate output attached.
- [ ] **T-A9b. Close the Phase 2b tasks whose evidence now exists** — T-2b.3, T-2b.5, T-2b.6,
      T-2b.7, T-2b.8 (§1). Run each `done when` line against the live cluster; close the ones that
      pass and record the output on the ones that do not. *F0b. Lane L4.*
      **Done when:** each of the five is either closed with its evidence attached or annotated
      with the specific clause that failed — **T-2b.5's drain/rejoin in particular**, which five
      Ready nodes do not prove.

### Now — lossy, and getting more expensive every day

- [ ] **T-A10. T-G.2 — one correlation id, one append-only audit stream.** ⏱ ADR-0041 D3 requires
      it *"from the first request."* Requests have been served for 14 days with none. *F6. Lane L5.*
      **Done when:** a fourth `SET LOCAL` carries the correlation id, `audit_events` exists
      append-only, and grants record who granted, to whom, when and how they end.
- [ ] **T-A11. T-G.3 — `region` on `tenants`, NOT NULL, no DEFAULT, immutable.** ⏱ *F6. Lane L5.*
      **Done when:** `db.region_is_chosen_at_provisioning_and_immutable` passes, and an `UPDATE`
      trigger refuses a move.
- [ ] **T-A12. T-G.1 — the cache key carries the tenant.** ⏱ *"The only latent defect in
      Genesis"* — and `careers/page.tsx` is now **serving live**, so it is latent no longer.
      *F6. Lane L5.*
      **Done when:** the resolved tenant is in the route, not a `cacheTag`, and
      `web.cache_key_carries_tenancy` fails a build that omits it. **Note:** the gate reports this
      check is *"declared in check-registry.yaml but no artifact cites it"* — it needs an artifact
      before it can be enforced.
      **Confirmed on disk 2026-09-16**, which upgrades F6 from an omission to a live defect:
      `export const revalidate = 60;` is at `src/app/[lang]/careers/page.tsx:14` and
      `src/app/[lang]/careers/[id]/page.tsx:18`, while the tenant is a **constant** —
      `paridyumTenantId = () => required("PARIDYUM_TENANT_ID")` in `src/lib/auth/config.ts`. So
      ISR keys on pathname against a tenant that never varies. Correct today; a cross-tenant leak
      the moment the tenant comes from `Host`. The path is exactly T-C4's.
- [ ] **T-A13. T-G.7 — off-host backup with a stated RPO, one rehearsed restore.** ⛔ *"The largest
      task in Genesis and the one everything waits on."* Gates real applicants into `web-hiring`
      (`0001_tenancy_foundation.sql`) and `paridyum` → `stage: operated`. *F6. Lane L5.*
      **Done when:** the off-host copy is scheduled with a stated RPO, one restore is performed
      and recorded, and the RPO is a number the attestation can cite.

### Now — the estate's own picture

- [ ] **T-A14. T-G.8 — `Release` as a schema and a read model, seeded from the cluster.**
      **Start here.** Validates the entity shape against reality before a saga is written against
      it, and reality is the full set `helm list -A` prints. *Lane L1.*
      **Done when:** every running release in the estate appears as a `Release` row with tenant,
      product, version, placement and state — and the count matches `helm list -A`.
- [ ] **T-A15. T-G.5 — clear the `operator` ambiguity.** *"Every design above inherits it."*
      *F6. Lane L2.*
      **Done when:** one reading is chosen, the losing artifacts are corrected or superseded, and
      `operator_id` either appears in a migration or stops being claimed by the glossary.

### Next — unblocked by the above

- [ ] **T-B1. Decide `docket`'s `kind`** — `catalogue`, `capability` or `platform`. Operator
      decision; `catalogue` binds C-0006's MKT- obligation. *F1.*
- [ ] **T-B2. Accept or reject ADR-0103**, then RFC-0045, then RFC-0049, in that order. §5 G1→G3.
- [ ] **T-B3. Accept or reject RFC-0044**, then register `docket` in `product-registry.yaml` at
      the `kind` T-B1 decided. *F1, F5.*
- [ ] **T-B4. Accept or reject RFC-0048, RFC-0050, RFC-0051, RFC-0046, RFC-0047** — parallel.
- [ ] **T-B5. Accept or reject RFC-0023 and RFC-0016.** The infrastructure they describe is
      **running in production and serving the public.** *F4 Set B.*
- [ ] **T-B6. Answer T-G.0, T-G.0b, T-G.0c.** ⛔ Three of Genesis's five Wave 0 decisions, absent
      from the original plan. T-G.0 blocks waves 2–4 entirely. *F6.*
- [ ] **T-B7. Assign the dashboard its OIDC client** (`make zitadel-apps`). *L7/L1.*
- [ ] **T-B8. Promote `platform-tenancy` to a repository** — crate, `manifest.yaml`, the documents
      `class-registry.yaml` requires, CI, and REQ-0022 item 9's `spec-` contract. *F8.*
      **Done when:** it has a manifest and its own CI, and lane L7 can be verified by it.

### Then — the arc Genesis names

- [ ] **T-C1. T-G.9 — `/dashboard` renders the estate.** Paridyum appears as one tenant among N,
      listing its landing, careers, docket, admin and `app`.
- [ ] **T-C2. T-G.10 — the placement resolver, inert.** `poolFor(tenantId)`; no request path
      constructs a pool directly.
- [ ] **T-C3. T-G.11 — the landing content schema.** → produces a migration.
- [ ] **T-C4. T-G.12 — careers resolves its tenant from the request.** `paridyumTenantId()` is
      deleted; an unresolvable host 404s rather than falling back.
- [ ] **T-C5. T-G.13 — the deploy saga, one product end to end.**
- [ ] **T-C6. T-G.14 — the tenant-scoped extractor.** Export (Art. 20), restore, delete (Art. 17).
- [ ] **T-C7. T-G.15 — provisioning requests the entitled Releases.**
- [ ] **T-C8. T-G.16 — upgrade, rollback, channel.**
- [ ] **T-C9. T-G.17 — `app` registers its own Release.**
- [ ] **T-C10. T-G.18 — the isolation attestation.**
- [ ] **T-C11. T-G.0d — docket's account silo, retrofit or accept.** The window closes when
      T-G.7 opens the gate on real HR records.

### The finish line

- [ ] **T-D1. A second tenant provisions through the dashboard and gets its own landing and docket
      at its own hostname** — with Paridyum holding no scope that tenant could not hold.

---

### Beyond the arc — estate defects the audit surfaced

*Not part of the three surfaces. Recorded because an audit that finds a defect and does not task
it has produced a finding rather than a change, and because three of these actively block tasks
above. Where `TASKS.md` already owns the item, its id is named rather than a new one minted.*

**These block tasks above**

- [x] **T-E1. Five checks cite `blocks:` lists nothing accounts for.** — *done 2026-09-21.*
      Six one-sided citations closed by adding the check to each named artifact's `enforced_by`.
      **Each was verified as TRUE rather than as gate-silencing**: ADR-0039 step 1 really does
      insert the tenant row *"(with `region`)"*; GLO-0005's whole content really is *granted,
      never inferred*; C-0006 anti-goal 5 really is the no-self-serve rule. ADR-0024 is `locked`
      and needed nothing — it already cites the `db.*` checks naming it. *Unblocks T-B2.*

- [ ] **T-E1 original text.** Five checks cite `blocks:` lists nothing accounts for. `db.region_is_chosen_at_provisioning_and_immutable`,
      `domain.a_named_rung_is_an_operable_rung`, `domain.containment_is_granted_never_inferred` and
      `domain.no_self_serve_path_to_a_dedicated_rung` all declare they block `ADR-0039`, `ADR-0041`,
      `GLO-0005` or `C-0006`, and only `ADR-0103` cites them back. *Blocks T-B2.*
      **Done when:** each check's `blocks` list is either cited by every artifact in it, or narrowed
      to what cites it.
- [x] **T-E2. `web.cache_key_carries_tenancy` and `repo.supply_chain_posture_is_declared` are
      declared but no artifact cites them.** — *done 2026-09-21.* Both were cited in **prose**
      and not in `enforced_by:`, which is what the gate reads. RFC-0048 slice 1 and ANTI-0009 now
      carry the first; RFC-0003 carries the second. **ANTI-0009 is the notable one** — the
      anti-pattern for tenancy-blind cache keys had existed as prose with no owner since it was
      written, and now names the check that refuses it. *Unblocks T-A12 and T-A1.*
      **One defect this produced and the gate caught:** RFC-0048 had no `enforced_by` key at all,
      not an empty one, so the first attempt appended list items under `breaking: true` and made
      the frontmatter unparseable. Found on the next gate run rather than shipped.

- [ ] **T-E2 original text.** `web.cache_key_carries_tenancy` and `repo.supply_chain_posture_is_declared` are
      declared but no artifact cites them. A check with no artifact behind it is a check with no
      owner. *Blocks T-A12 and T-A1 respectively.*
      **Done when:** each is cited by the artifact that requires it, or removed from the registry.

**These degrade the estate and are already owned by `TASKS.md`**

- [ ] **T-E3. T-2.3 — resolve the five locked-but-unenforced artifacts.** ADR-0020, **ADR-0049**,
      ANTI-0002, ANTI-0003, STD-0009. *ADR-0049 joined this list and is the one that matters here:
      "nothing runs outside a tenant" is locked, and nine releases are running.*
      **Done when:** `check-knowledge.sh` reports zero locked artifacts enforced by nothing.
- [x] **T-E4. T-2.6 — fix the three standing gate warnings.** — *2026-09-21: one fixed, two have
      a written reason to persist, which is the task's own second branch.*
      **Fixed:** RUN-0008 cited `download.proxmox.com/debian/pmg` with no declared source. Both
      that URL and the Proxmox wiki page documenting it are now declared, the wiki with its real
      `fetched` date from the research log.
      **Persists — STD-0005 is `locked`.** It cites keepachangelog 2.0.0 with no `sources` block.
      A missing citation is not a *factual error*, so ADR-0031's errata path does not fit; and
      superseding a locked standard to add a citation is heavier than the defect. Recorded rather
      than forced.
      **Persists — the 48 hardcoded registry counts.** The gate's own text already calls these
      *"not defects; a review list for counts used as evidence"*, and the baseline holds at 48.
      A written reason to persist is exactly what it already carries.

- [ ] **T-E4 original text.** T-2.6 — fix the three standing gate warnings. STD-0005's uncited keepachangelog
      source; hardcoded registry counts sitting at their baseline; `patterns/` declared with no
      directory. **Done when:** the gate runs clean, or each warning has a written reason to persist.
- [x] **T-E5. Clear the unresolved research citations (RFC-0020, RFC-0027).** — *done 2026-09-21
      in substance, and the task's own done-when is unachievable as written.*
      **Seven of the eight URLs now carry provenance**, each with its REAL fetch date read out of
      the research log rather than typed from memory: the two Sync.com pages → ADR-0071, the
      Zitadel AddOIDCApp and version-pinned `management.proto` → ADR-0070, AddSMTPConfig →
      ADR-0083, the VHS repository metadata → ADR-0075, and the Proxmox wiki → RUN-0008. The
      eighth, `github.com/mkdocs/catalog`, is a genuine dead end and stays one.
      **⚠ CITING THEM DOES NOT CLEAR THE WARNINGS, AND CANNOT.** `knowledge.fetched_sources_are_reconciled`
      reads the frozen `kind: session` records the Stop hook wrote, and **deliberately does not
      recompute** — its own comment says why: *"recomputing here would produce a second, worse
      answer — every fetch in the repository's history compared against today's frontmatter, which
      reports a page cited in a superseded artifact as uncited forever."*
      So the warning is a **historical fact** — *this session ended with these uncited* — which is
      true, stays true, and should. RFC-0027 D6's `kind: correction` voids a **wrong entry** (a
      fixture, a mistaken capture), not a real fetch since cited. There is no mechanism to clear
      these and there should not be.
      **The count is therefore not a backlog to drive to zero**, and a task that reads as one
      invites someone to falsify the log to close it. What the citations actually bought is
      RFC-0020's real purpose: those eight external facts now carry where they came from and when
      they were true.

- [ ] **T-E5 original text.** Clear the unresolved research citations (RFC-0020, RFC-0027). The gate reports a
      run of URLs read by past sessions that no artifact declares in `sources` — Zitadel admin and
      management APIs, the Proxmox package-repository wiki, Sync.com's requirements pages,
      `charmbracelet/vhs`, `mkdocs/catalog`. RFC-0027 decision 6 allows a dead end to be voided
      rather than cited, but voiding must be recorded. **Done when:** each is cited under a `sources`
      block or voided with a reason.
- [ ] **T-E6. T-2.13 and T-2.14 — the `PreToolUse` guard's two holes.** `sed -i` bypasses the guard
      entirely, and a hand-written file carrying the word "edit" in a comment becomes classified as
      generated. Both are enforcement-layer defects in the one layer that does not depend on the
      GitHub plan. **Done when:** the guard covers file-mutating Bash, or ADR-0007 and `CLAUDE.md`
      state the limit plainly; and the generated-marker match is anchored or the denial quotes the
      line that matched.

---

## 8. What this plan deliberately excludes

- **Self-service signup.** ADR-0100 D5 defers it; operator-driven creation ships first.
- **A sizing control.** C-0006 anti-goal 2 — the unit of sale may not name a resource.
- **A datacentre selector.** RFC-0015 D3 — the pair is what makes the standby work.
- **A second region.** One is built. Thirteen jurisdictions read *On request*.
- **A typeface.** STD-0015 records it as debt; `DECISION-DEBT.md` holds it.
- **An icon set.** RFC-0050 open question 3.
- **Quotas** (RFC-0045 slice 8) — they degrade rather than leak.

---

## 9. The honest risk

**The original risk section was inverted and is worth reading for that reason.** It said *"nothing
here has ever run"* and named the plan's largest untested assumption as *"that any of it deploys
at all."* Nine releases have been running for fourteen days. The risk was never that the estate
cannot deploy; it is that **the estate does not know what it has deployed** — and the plan
demonstrated the failure by writing "nothing is deployed" one paragraph after invoking the
estate's rule that a green check is unproven until you have watched it fail.

The live risks now:

**Most of §5's proposed artifacts already have code written against them — and some of that code
is in production.** RFC-0023 and RFC-0016 are `proposed` while the cluster and edge they describe
serve the public. That means the acceptance pass is not a rubber stamp, and a reviewer who feels
unable to reject is not reviewing.

**The lossy window is open and closing.** T-G.2's correlation id and T-G.3's region column were
cheap against an empty database. They are being deferred against one with two weeks of live
traffic on it, and T-G.2 is explicitly unrecoverable.

**And the record is not to be trusted.** Every "done when" in §7 that can be checked against a
machine should be. The plan's first draft is the evidence for why: it read four manifests, a
logbook and a routing table, all of which agreed, and all of which were wrong.
