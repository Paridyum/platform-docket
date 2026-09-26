---
id: RFC-0049
title: A tenant is a set of releases, and /dashboard is where they are deployed
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra, web, console, service]
applies_to: ["*"]
product: [paridyum, tenancy]
tags: [deployment, release, control-plane, app, bootstrap, self-hosting, platform-deployment, adr-0042, adr-0100, adr-0039, adr-0020, adr-0052, rfc-0009, rfc-0045, rfc-0047, rfc-0048, anti-0006]
breaking: false
---

# RFC-0049 — A tenant is a set of releases, and the dashboard is where they are deployed

## Problem

**There is no noun in this estate for "docket, running, for tenant X, at version Y."** Every other
fact about a tenant has an owner — its identity, its placement, its entitlement, its region, its
branding — and the one fact a tenant most obviously has, *what is actually running for it*, has
none.

This is not a new discovery. The ROADMAP flags it, in bold, as blocking:

> ⚠️ **Unresolved before 3a.** RFC-0009 §7's wave included `platform-deployment`; ADR-0042 fixes the
> wave at five and omits it. **Release, Certificate, and the act of running an artifact have no
> owner.**

RFC-0009's own decomposition named the context and its entity precisely — *`platform-deployment`:
Runs an artifact for one tenant. Owns: Release* — and its ownership matrix assigns `Release` to it
and to nothing else. ADR-0042 then fixed the first wave at five contexts and left `Release`
unassigned. That was a sequencing decision, not a repudiation, and the gap has been open since.

Observably, today:

1. **Nothing in the estate is deployed.** Every `manifest.yaml` reads `lifecycle: experimental`
   except `infra-ci`, `tool-forge` and `tool-docket`. The governance layer is unusually complete and
   governs no running software — the ROADMAP's own one-sentence state.

2. **`/dashboard` provisions tenants but deploys nothing.** ADR-0100 makes it a BFF over
   `platform-tenancy` that drives ADR-0039's saga — which creates a tenant row, a Zitadel org,
   branding, grants, a domain and a billing customer. **The saga never starts an application.** A
   provisioned tenant has an identity and a hostname and nothing behind them.

3. **Paridyum cannot see its own estate as a tenant's estate.** Landing, careers, docket, admin and
   the control plane exist as repositories, charts and CI pipelines. Nowhere is there a view that
   says: *this tenant runs these products, at these versions, in this placement, and here is each
   one's health.* That view is what an operator actually wants from a control plane, and it is the
   same view a tenant wants of its own products.

4. **The bootstrap asymmetry has no mechanism, only a sentence.** ADR-0100 decision 4 says *"the
   provisioner is not provisioned by a tenant"* and leaves it there. When `/dashboard` needs upgrading, the
   procedure is the estate pipeline — which is fine, and is also the only deployment path in the
   estate that a human must remember rather than a system perform.

**Who experiences it.** Paridyum, as tenant zero, immediately and continuously: it is the party that
has products to run and no mechanism that runs them. And the ROADMAP's phase 3a cannot close while a
flagged blocker inside it has no owner.

**What happens if nothing is done.** Deployment gets built anyway — informally, inside
`platform-tenancy`'s saga or inside `/dashboard` itself, because something must start an application. Both
are ANTI-0006: a context acquiring a second responsibility because no other context claimed the
first. ADR-0042's own consequences name the standing temptation, and this is the shape it takes.

**How we will know it is fixed.** `/dashboard` lists every tenant, and for each tenant every product it
runs, at what version, in what placement, in what state — and Paridyum appears in that list as one
tenant among N, running its landing, its careers surface, its docket, and `/dashboard` itself.

## Constraints

- **RFC-0009 §3/§4** — `platform-deployment` *"Runs an artifact for one tenant. Owns: Release."*
  The decomposition is already done; this RFC re-admits it rather than re-deriving it.
- **ADR-0042 (accepted)** — the first wave is five contexts; decision 2 makes `platform-tenancy`
  first because it owns the tenant lifecycle; decision 7 says creating any of these still escalates
  to a human. `accepted`, not locked, so a sixth context extends rather than supersedes.
- **ADR-0019 (locked) decision 5** — creating a service escalates to a human. A sixth repository is
  a human decision, `ask` never `deny`.
- **ADR-0100 (accepted)** — `/dashboard` is a **surface**, not an owner: a BFF that writes no tenant
  entity and touches no other context's tables. Decision 4 is the bootstrap asymmetry this RFC
  narrows. Decision 3 fixes the URL model.
- **ADR-0039 (accepted)** — provisioning is a compensating saga with a public lifecycle. Deployment
  should be the same shape, not a second pattern.
- **ADR-0020 (locked)** — Kubernetes is the deployment target; charts, probes, digest-pinned images.
- **ADR-0052, ADR-0037** — OpenTofu provisions; infra repositories *define* the cluster rather than
  run in it. The genesis path in §Proposal D is theirs and stays theirs.
- **ANTI-0001 / ANTI-0003 / ANTI-0006** — one owner per entity; no foreign table access; no god
  service. The three that decide §Alternatives 1 and 2.
- **RFC-0045 / ADR-0103 (proposed)** — a Release deploys into its tenant's **placement**; the
  resolver is the seam that makes that possible without the deployer knowing about containment.
- **RFC-0048 (proposed)** — the public surfaces become tenant-served, which is what makes them
  Releases rather than one-off sites.
- **RFC-0047 (proposed)** — a partner runs its own control plane. Everything here must work when the
  thing being deployed is `/dashboard` itself, because that is what a partner stamp is.
- **ADR-0049 (locked)** — nothing runs outside a tenant, and Paridyum holds no scope a customer could
  not hold. A Release belongs to a tenant; there are no tenantless deployments.
- **ROADMAP phase 3a** — the ordering `platform-tenancy → platform-identity → platform-routing →
  spec-product`, and the ⚠️ this RFC answers.

## Proposal

### A. `Release` is the missing noun, and it has one owner

**A Release is a product running for a tenant**: `(tenant, product, version, placement, state)`, plus
its health and its history. Owned by **`platform-deployment`** — RFC-0009's sixth context, re-admitted
to the wave as the thing ADR-0042's own ⚠️ says is missing.

What it owns and what it does not:

| Owns | Does not own |
|---|---|
| `Release` — a product instance for a tenant | `Tenant`, `Account` (`platform-tenancy`) |
| the deploy/upgrade/rollback lifecycle | `Domain`, `Certificate` (`platform-routing`) |
| version and channel per tenant | `Entitlement` (`platform-entitlement`) |
| health and last-known state | the *placement* (`platform-tenancy`, via RFC-0045) |

**`Domain` and `Certificate` stay with `platform-routing`.** RFC-0009's matrix already assigns them
and the ROADMAP's ⚠️ groups them with Release only because both were homeless, not because they
belong together. Routing binds a hostname to a Release; deployment never writes a routing table
(ANTI-0003).

### B. A tenant is a set of Releases — and that is the view `/dashboard` shows

This is the reframing the rest depends on. A tenant is not *a row*; a tenant is a row **plus what it
runs**. Paridyum's own estate, expressed in the model:

```
tenant: paridyum  (tenant zero, ADR-0049 D2)
  ├── Release  landing   ← web-paridyum      single-tenant, product zero's own (RFC-0048 §B)
  ├── Release  careers   ← web-hiring        tenant-served (RFC-0048 §A)
  ├── Release  docket    ← tool/console-docket  multi-tenant (ADR-0099)
  ├── Release  admin     ← web-admin
  └── Release  app       ← the control plane  ★ the special case (§D)

tenant: <any other>
  ├── Release  careers
  └── Release  docket
       (no `app` Release — unless franchised, RFC-0047)
```

An ordinary tenant's estate differs from Paridyum's **only by which Releases it holds**, which is
ADR-0049 decision 2 made structural instead of aspirational: tenant zero is not a different kind of
tenant, it is a tenant with more Releases.

### C. `/dashboard` is a surface over Release, owning nothing

ADR-0100 decision 1's pattern, unchanged and extended: `/dashboard` is a BFF that calls
`platform-tenancy` for tenants and now `platform-deployment` for Releases. It writes neither. What it
gains is the second half of the view it was always missing — today it can create a tenant and show
its provisioning status; it should also show what that tenant runs and let an operator deploy,
upgrade, roll back and inspect.

Per ADR-0100 decision 3 it stays at `<platform-zone>/dashboard`, never reachable from a tenant's hostname.

### D. Paridyum's `app` Release — genesis versus steady state

The operator's phrasing was *"app running and deploying itself, special case for paridyum."* The
honest form of that, and the reason it is not circular:

> **`/dashboard` cannot bring itself into existence. It can manage its own existence thereafter.**

Two phases, named separately because conflating them is what makes self-hosting sound impossible:

1. **Genesis** — `/dashboard` v0 is deployed by the estate's normal pipeline (ADR-0052, ADR-0020,
   `infra-kubernetes`). No Release exists; nothing is managing anything. This is unchanged from
   today and **must remain permanently available**, because it is the only path back when §D.2
   fails.
2. **Steady state** — once running, `/dashboard` registers a Release for *itself*, owned by tenant zero,
   and thereafter upgrades itself through the same lifecycle as every other Release.

This is the compiler-bootstrap and GitOps self-management pattern, and it narrows ADR-0100
decision 4 minimally and precisely:

| | |
|---|---|
| **D4 today** | *the provisioner is not provisioned by a tenant* |
| **D4 after this** | *the provisioner's **first** deployment is not provisioned; every subsequent one is* |

The asymmetry stops being a property of control planes in general and becomes a property of
**genesis** — which is also exactly what RFC-0047 needs, because a partner's control plane is a
Release that Paridyum's `/dashboard` deploys, and only the root's genesis is special.

**The failure mode, named because it is real:** a bad `app` Release can break the thing that deploys
Releases. Mitigations are not optional — the previous version is retained and rollback does not
depend on the new one being functional; the genesis path stays runnable by a human at any time; and
an `app` Release upgrade is gated differently from every other kind.

### E. Deploying is a saga, the same shape as provisioning

ADR-0039's pattern reused rather than a second one invented: asynchronous, a public lifecycle, every
forward step declaring a compensation.

```
requested → deploying → active
                │
                └──▶ compensating ──▶ failed
```

Lowercase identifiers per ADR-0032. Compensation for a deploy is a rollback to the previous Release,
never a deletion — the same *"compensate means return to the safest valid state"* rule ADR-0039
states for provisioning.

**And provisioning gains a step.** ADR-0039's saga currently ends at `active` with nothing running;
it should end by requesting the Releases the tenant's entitlement grants. A tenant becomes `active`
when its products are running, not when its row is written.

### F. A Release deploys into its tenant's placement

RFC-0045 §Proposal A's resolver is what makes this work: `platform-deployment` asks for a tenant's
placement and deploys there. At C0 that is the shared cluster; at C1 it is the same cluster against
a different database; at C2/C3 it is a different cluster entirely. **The deployer does not branch on
containment class** — it resolves a placement and acts on it, which is the property that keeps one
deployment path serving every rung.

### G. Version is per Release, and channel is per tenant

RFC-0047 §P5 needs it for partners; every tenant needs it sooner. A Release carries a version; a
tenant carries a channel; an upgrade is a Release transition with a compensation. This is where
RFC-0047's version-skew problem gets solved once for everyone rather than invented for partners.

## Alternatives considered

1. **Fold deployment into `platform-tenancy`'s saga.** The path of least resistance — the saga
   already exists and already provisions. **Rejected**: ADR-0042 decision 2 makes tenancy the owner
   of the *tenant lifecycle*, and the artifact lifecycle is a different lifecycle with a different
   write rate, a different failure model and a different cadence. Merging them is ANTI-0006 and
   ADR-0042's consequences name it as *"the one mistake here that is effectively permanent."*

2. **Put Release in `/dashboard`.** Also tempting, because `/dashboard` is where a human would look for it.
   **Rejected**: ADR-0100 decision 1 is explicit that `/dashboard` is a surface and owns no entity. A BFF
   that acquires a table is no longer a BFF, and the next surface that needs Releases would have to
   read `/dashboard`'s tables (ANTI-0003).

3. **Track B's fleet manager absorbs it.** The ROADMAP explicitly leaves this open — *"Track B may
   absorb part of this — the fleet manager runs artifacts — and if it does, that is a decision to
   write down rather than let happen."* **A genuine option and the strongest alternative.**
   Rejected here on scope: the fleet manager (RFC-0025) runs *workloads for hosting customers* under
   BET-0009, which is a different product with a different buyer. A Release is a catalogue product
   running for a tenant. Sharing the mechanism later is plausible; starting there would put the
   catalogue's deployment path inside a hosting product whose triggers have not fired.

4. **GitOps only — no `Release` entity.** Charts in a repository, a controller reconciling them, no
   database row. Operationally excellent and genuinely how the deploy should *execute*. **Rejected as
   the whole answer**: it gives no tenant-facing noun. A tenant cannot be shown what it runs, RFC-0045's
   attestation cannot cite a version, and entitlement cannot gate a product it cannot name. The
   proposal is compatible with GitOps underneath — `Release` is the declared intent; a reconciler may
   be how it is realised.

5. **Do nothing; keep deploying by pipeline.** The status quo, and viable while exactly one tenant
   exists. **Rejected because that condition is ending**: RFC-0048 makes careers tenant-served,
   ADR-0099 makes docket multi-tenant, and both produce per-tenant deployments that a human pipeline
   cannot fan out to.

## Consequences

**What this makes worse.**

1. **A sixth context, a sixth repository, and a human escalation to create it** (ADR-0019 D5). It
   also amends ADR-0042's "the first wave is five", which was a deliberate scoping decision — and
   re-opening a scoping decision is how waves become unbounded. The defence is that ADR-0042's own
   ⚠️ asked for this, and that the alternative is the responsibility landing somewhere worse.
2. **Self-deployment can break the deployer.** §D names the mitigations; they are real work and they
   are the kind that is skipped because the failure is rare. The genesis path must stay runnable
   forever, which means it must stay *tested*, which means a path nobody uses must be exercised on a
   schedule.
3. **`/dashboard` becomes load-bearing.** Today it is a provisioning dashboard nobody depends on. After
   this, an outage means nothing can be deployed or rolled back — including `/dashboard` itself.
4. **Two lifecycles to reason about per tenant**, provisioning and deployment, with an ordering
   between them. Incidents now have a "which saga" question.
5. **Version skew becomes a real matrix**, earlier than RFC-0047 needed it. That is better than
   discovering it with a partner, and it is still a matrix.
6. **ADR-0100 decision 4 is amended**, and any reader who took its sentence at face value has to be
   re-pointed. §D's table is written so the change is a narrowing rather than a reversal.

**What it does not change.** Tenancy's ownership of `Tenant`/`Account`. Routing's ownership of
`Domain`/`Certificate`. RLS, the session contract, placement. `/dashboard` still owns no entity.

## Rollout

Sequenced so that **nothing waits on the new context that does not have to**.

| Slice | What lands | Depends on |
|---|---|---|
| **0** | This RFC accepted; the sixth-context decision escalated to a human (ADR-0019 D5); ADR-0042 amended by a successor ADR | — |
| **1** | `Release` as a schema and a read model — Paridyum's existing deployments *recorded* rather than performed | 0 |
| **2** | `/dashboard` renders the estate view: tenants → Releases → version, placement, state | 1 |
| **3** | The deploy saga: request, deploy, health, compensate — one product end to end (**careers**, the simplest) | 2, RFC-0048 slices 1–3 |
| **4** | ADR-0039's saga requests the entitled Releases at provisioning | 3 |
| **5** | Upgrade, rollback, channel | 3 |
| **6** | `app` registers its own Release; genesis path documented, tested and scheduled | 5 |
| **7** | Placement-aware deploys (C1+) | RFC-0045 slice 3 |

**Slice 1 is deliberately a read model.** Recording what is already deployed, before deploying
anything, makes the estate view real in days rather than quarters and validates the entity shape
against reality before any saga is written against it.

**Slice 6 is last for a reason.** Self-deployment is the highest-leverage and highest-blast-radius
piece, and it should run on a lifecycle that has already upgraded and rolled back other products
many times.

**How it is undone.** Slices 1–2 are additive and droppable. Slice 3 onward: the genesis path
remains, so the fallback is always "deploy by pipeline," which is today's behaviour.

## Open questions

1. **Does `platform-deployment` execute, or declare?** §Alternatives 4 argues `Release` is intent and
   a reconciler realises it. If so, how much of ADR-0020's chart machinery moves, and does
   `infra-kubernetes` keep the genesis path? The answer decides the repository's size.
2. **Is the sixth context authorised?** ADR-0019 D5 and ADR-0042 decision 7 both require a human.
   Blocking on slice 0.
3. **What is a Release's health, precisely?** Probes answer for a pod; a tenant wants to know whether
   its careers page is serving. These are not the same question, and the attestation (RFC-0045 §E)
   will cite whichever one is chosen.
4. **Does a tenant see its own Releases, or only an operator?** ADR-0049 decision 3 wants a
   tenant-facing consumer; showing a tenant what it runs is the obvious one, and it is also a support
   surface nobody has scoped.
5. **How does an `app` Release upgrade differ in gating?** §D says "differently"; it does not say how.
   A human approval, a canary, a mandatory rollback rehearsal — undecided.
6. **Does this absorb Certificate after all?** §Proposal A says no and RFC-0009's matrix agrees, but
   RFC-0044 stage 3 reserves an unwritten routing ADR for that work and it is currently nobody's.

## Related

- **RFC-0009** — the decomposition that already named this context and its entity
- **ADR-0042** — the five-context wave this extends, and whose ⚠️ this answers
- **ADR-0100** — `/dashboard` as a surface; decision 4's asymmetry, narrowed in §D
- **ADR-0039** — the saga shape reused, and the step it gains
- **ADR-0019, ADR-0020, ADR-0052, ADR-0037** — the human gate and the genesis path
- **RFC-0045, ADR-0103** — placement, which a Release deploys into
- **RFC-0047** — a partner's control plane is a Release; only the root's genesis is special
- **RFC-0048** — tenant-served public surfaces, which is what makes them Releases
- **ANTI-0006** — the god service both rejected alternatives would build
- **ROADMAP phase 3a** — the blocker this closes
