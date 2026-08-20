---
id: RFC-0025
title: The fleet manager bootstraps itself, and Paridyum is customer zero
type: rfc
status: proposed
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [hosting, fleet-manager, bootstrap, proxmox, talos, fleet, vocabulary, c-0008, bet-0009]
breaking: true
enforced_by: []
verified_by: [TST-0062]
sources:
  - url: https://github.com/siderolabs/omni/blob/main/LICENSE
    title: Sidero Omni — LICENSE (Business Source License)
    retrieved: 2026-08-18
    supports: >
      Omni is BUSL-licensed: free for non-production use — testing, proof of concept, homelabs —
      with production use requiring a commercial licence from Sidero, and conversion to MPL four
      years after each release. Decision 5's finding that no free adopt-path existed.
  - url: https://docs.siderolabs.com/omni/self-hosted/run-omni-on-prem
    title: Sidero — run Omni on-prem
    retrieved: 2026-08-18
    supports: >
      Self-hosting is documented and supported; the constraint is licensing rather than
      capability. Stated so the rejection is not misread as a technical one.
  - url: https://ashimov.com/posts/proxmox-multitenancy/
    title: Proxmox security and multi-tenancy — roles, pools, API tokens, isolation
    retrieved: 2026-08-18
    supports: >
      Proxmox pools and RBAC enforce hard limits only on storage; CPU and RAM remain best-effort,
      so a pool admin creating oversized guests can still degrade neighbours. Decision 6.
  - url: https://pve.proxmox.com/wiki/User_Management
    title: Proxmox VE — user management, roles, pools and ACLs
    retrieved: 2026-08-18
    supports: >
      The permission model is users, roles, paths and ACLs, designed for administrators of a
      cluster rather than for end customers scoped to their own resources. Decision 6.
  - url: https://emirb.github.io/blog/microvm-2026/
    title: Your container is not a sandbox — the state of microVM isolation in 2026
    retrieved: 2026-08-18
    supports: >
      Across roughly eighteen months every major platform converged on the position that
      untrusted code requires stronger isolation than a container, with microVMs the current
      standard. Decision 6's rejection of plain runc on `fleet-shared`.
  - url: https://northflank.com/blog/kata-containers-vs-firecracker-vs-gvisor
    title: Kata Containers vs Firecracker vs gVisor — container isolation compared
    retrieved: 2026-08-18
    supports: >
      Firecracker boots in ~125 ms with under 5 MiB overhead; Kata boots in ~200 ms with minimal
      overhead and presents standard container APIs over several VMMs; gVisor intercepts syscalls
      at 10–30% overhead on I/O-heavy work. Decision 6's runtime selection.
  - url: https://crates.io/crates/talos-rust-client
    title: talos-rust-client — Rust gRPC client for Talos, crates.io
    retrieved: 2026-08-18
    supports: >
      A Rust gRPC client for the Talos machine API with mTLS, generated from the official proto
      files. Decision 7's claim that the Talos side needs no new protocol work.
  - url: https://crates.io/crates/proxmox-api
    title: proxmox-api — Rust bindings for the Proxmox VE HTTP API, crates.io
    retrieved: 2026-08-18
    supports: >
      Community bindings exist but the principal crate has not been updated in roughly two years.
      Decision 7's choice to generate a client from the published API schema rather than depend
      on a stale one.
---

# RFC-0025 — The fleet manager provisions Paridyum

## Status

**PROPOSED**, 2026-08-18. Opened because BET-0009 was accepted the same day and describes *what*
hosting sells without saying *how it is built*, and because the obvious build order — hosting and
platform as two independent estates — throws away the thing that makes the pairing worth doing.

## Renamed 2026-08-18 — this was "the hosting control plane"

**`control plane` was already taken and this RFC took it anyway.** GLO-0007 defines the control
plane as the platform's global-facts tier; RFC-0018 is titled *"the control plane is one
repository"* and means the four `platform-*` contexts. This RFC used the same words for a
machine-provisioning system.

That is the collision GLO-0007 was written to prevent, repeated one week later on the word GLO-0007
itself introduced. Renamed to **fleet manager** before it spread past one artifact. GLO-0009 carries
the definition and the names rejected.

## The question

BET-0009 permits selling hosting. RFC-0019 requires the platform never share a cluster with
anything else. Read naively those two produce **two separate systems built twice**: two
provisioning stacks, two monitoring stacks, two on-call rotations, for one person.

**Is there an arrangement where hosting and platform share everything that is safe to share, and
share nothing that is not?**

## Decision 1 — the fleet manager is shared, the data plane is not

**One fleet manager. At least two physically separate fleets.**

| | What it is | Who is on it |
| --- | --- | --- |
| **Fleet manager** | Provisioning, lifecycle, quota, metering, customer records. Terraform against a Proxmox API, Talos configuration, the customer-facing API | Nobody's workload runs here |
| **`fleet-shared`** | Proxmox cluster carrying external hosting customers, multi-tenant | Hosting customers |
| **`fleet-platform`** | Proxmox cluster carrying Paridyum's own platform, single-tenant | Paridyum only |

**This satisfies RFC-0019 decision 3 rather than bending it.** D3 forbids the platform sharing a
*Proxmox cluster* with other workloads. Two fleets are two clusters, which D3 already anticipates:
*"Proxmox clusters are per-datacentre anyway, so this is one more cluster in a facility that
already has one, not one more facility."*

**What D3 does not forbid is shared automation.** The isolation guarantee C-0006 rests on is about
where a workload *executes*, not about which program wrote the Terraform. A fleet manager that
provisions both fleets and runs on neither crosses no boundary the guarantee names.

**BET-0009 decision 1 is refined by this, not contradicted.** It said "separate metal" and meant
the data plane; it is amended in place to say so, because "not a separate VM, not a separate
Proxmox node" reads as forbidding the shared fleet manager too, and that reading would force the
two-systems-built-twice outcome this RFC exists to avoid.

## Decision 2 — Paridyum is customer zero, and never a privileged one

**Paridyum's platform is provisioned through the same fleet-manager API as any hosting customer,
against the same quotas, through the same metering, with an internal invoice it actually receives.**

This is C-0008 one layer down. That constitution makes Paridyum tenant zero in the runtime;
ADR-0050 decision 4 registers it as product zero for the same reason, and states the argument
generally:

> *The alternative — an unregistered operator governing a registry of other people's products — is
> the asymmetry C-0008 forbids in the runtime, and it would have been harder to see here, because
> nothing looks more natural than the platform being the exception.*

**Nothing looks more natural than the platform being the exception**, and infrastructure is where
that temptation is strongest, because the operator has root on the hypervisor and a hand-built path
is always five minutes faster.

**Why it is worth the friction.** Every failure mode a hosting customer would hit is hit first by
the party who can fix it, with their own production affected. BET-0005 made dogfooding structural
at the application layer; this makes it structural at the infrastructure layer. A provisioning bug
that would silently degrade a paying customer takes down Paridyum first.

**The rule that makes it checkable:** *no path into a fleet that is not the fleet-manager API.* If
an operator can `ssh` into a node and hand-place a workload, decision 2 is decoration. RFC-0019
decision 2 already chose Talos precisely because *"a node nobody can SSH into cannot drift"* — the
same property does the work here.

## Decision 3 — the bootstrap sequence, and the circularity is named

**`sitar-as-in-01-ar` seeds the fleet manager and then stops being load-bearing.**

| Stage | What exists | What it costs |
| --- | --- | --- |
| **0 · Seed** | Fleet manager runs on the RFC-0023 `staging` cluster. No customer workloads anywhere | ₹0 — hardware is sunk |
| **1 · First fleet** | Fleet manager provisions one rented Indian node as `fleet-shared`. First external customers | ~₹3,000–4,000/mo |
| **2 · Self-provision** | Fleet manager provisions `fleet-platform` on separate rented metal. Paridyum's platform deployed onto it *by the fleet manager*, as customer zero | +one node |
| **3 · Evacuate the seed** | Fleet manager migrates onto `fleet-platform`. `sitar-as-in-01-ar` reverts to a development substrate and a cold rebuild target | ₹0 |
| **4 · Crossover** | At the threshold MKT-0001 computes, fleets move to owned colocation | Capex, on MKT-0001's trigger |

### Why a residential machine may host the fleet manager

RFC-0023 accepted cost 3 is blunt: *"No availability of any kind. One machine, one power supply,
one network connection, in a residence."* Stage 0 puts the fleet manager there anyway, and the
justification is a property rather than an optimism:

**A fleet-manager outage stops provisioning. It does not stop running workloads.** A customer whose
application is serving traffic on `fleet-shared` keeps serving it while the fleet manager is dark.
What breaks is signup, scaling and lifecycle operations — degraded, visible, and recoverable.

**This property is load-bearing and must be protected deliberately.** The moment the fleet manager
sits on a request path — a shared ingress, a licence check, an auth callback, a metering hook that
blocks — stage 0 becomes indefensible and the whole sequence collapses. **Open question 2.**

### The circularity, stated rather than hidden

After stage 3 the fleet manager runs on metal it provisioned itself. That is a genuine circular
dependency and it has one failure mode worth naming: **if the fleet manager is lost and its state
lives only on the fleet it created, there is no path back.**

Two requirements fall out, and they are not optional:

1. **Fleet-manager state is backed up off-fleet**, to storage neither fleet provides.
2. **`sitar-as-in-01-ar` is retained as a cold rebuild target**, and the restore is rehearsed
   before stage 3 completes — not documented, rehearsed. An untested restore is the same class of
   claim as an unenforced lock.

## Decision 4 — which fleet a workload lands on is a variable

RFC-0023 decision 4's invariant, extended one level up: **nothing may encode "this is the hosting
fleet" or "this is the platform fleet."** Fleet is an input to the same provisioning code, exactly
as node count and hypervisor endpoint already are.

A difference that cannot be expressed as a variable is a design difference, and it must be recorded
here rather than discovered when the second fleet is created.

## Decision 5 — build the multi-tenancy layer, adopt the machine mechanics

**C-0006 already supplies the build-versus-adopt rule and it does not need a new one.** That
document splits the world into commodity — *"we will not position on these, price on these, or
describe them as our value"* — and non-commodity, *"therefore ours to build."* Applied here:

| Component | Verdict | Why |
| --- | --- | --- |
| Creating a guest on a hypervisor | **Adopt** — Proxmox API | Commodity mechanism |
| Kubernetes cluster lifecycle | **Adopt** — Talos | Commodity, and RFC-0019 D2 chose it for this |
| Container sandbox | **Adopt** — see decision 6 | Never invent a sandbox |
| **Tenancy, quota and isolation policy** | **BUILD** | C-0006 non-commodity item 2: *provable isolation… the guarantee is the product* |
| **Metering and the customer model** | **BUILD** | BET-0007 and BET-0008 both rest on it |
| **The desired-state model and reconciler** | **BUILD** | It encodes our policy; nothing else can |

**This is not a cloud being built. It is the multi-tenancy and policy layer over Proxmox and
Talos** — which is exactly what C-0006 says is not commodity and exactly what no vendor sells.

**No free adopt-path existed for the built half.** Sidero Omni is the closest product and is
BUSL-licensed: production use requires a commercial licence. Beyond cost, it fails REQ-0012's
requirement that every dependency be *"approved, scoped, and exitable"* — a source-available
licence converting to MPL four years after each release is the least exitable position available
at the centre of a business. **The rejection is commercial and legal, not technical**, and Omni
remains the right thing to evaluate if this RFC is ever reversed.

## Decision 6 — isolation is structural, and quota does not live in Proxmox

Two findings from 2026-08-18 that change what must be built, both cited above.

### Proxmox cannot enforce the guardrail BET-0008 depends on

Proxmox pools and RBAC enforce **hard limits only on storage. CPU and memory remain best-effort.**
The permission model is built for administrators of a cluster, not for customers scoped to their
own resources.

**BET-0008 trigger clause 2 says the unmetered promise is unfunded without a working guardrail.**
That guardrail therefore **cannot** be a Proxmox pool. It lives in Kubernetes — `ResourceQuota`,
`LimitRange`, and CNI-level bandwidth policy — on the Talos clusters above the hypervisor.
Proxmox remains the machine layer; it is not the policy layer, and treating it as one would leave
the guardrail believed-in and absent, which is this repository's most frequent defect.

### A container is not a security boundary

`fleet-shared` runs arbitrary customer images. The industry position as of 2026 is that untrusted
code requires stronger isolation than a container, with microVMs the current standard.

**`fleet-shared` must not run plain `runc`.** Kata Containers is the selection: microVM isolation
presented through standard container APIs, so the fleet manager provisions workloads without
knowing the sandbox is there. Firecracker boots faster with lower overhead and is worth revisiting
if boot latency ever becomes the binding constraint; gVisor's 10–30% I/O penalty is the wrong
trade for hosted applications.

**Structural, not configured.** A customer workload landing on a non-sandboxed runtime must be
*impossible*, not discouraged — a default `RuntimeClass` that an unset field silently bypasses is
the same failure as an unenforced lock. **This is the single most important thing to get right in
the whole design**, and it is worth stating plainly that decision 1 is not weakened by it: microVMs
make sharing a node between *customers* defensible, and never make sharing a node with *tenant
data* defensible. Defence in depth, not a substitute.

## Decision 7 — reconcile, do not command

**Desired state persists in Postgres; controllers observe, diff and converge; every operation is
idempotent and retry-safe.**

The reason is not fashion. **An imperative provisioner must be correct once. A reconciler must only
be eventually correct** — a materially weaker requirement, and the right one for a system with one
maintainer where a crash mid-provision is a certainty rather than a risk. State is external truth
rather than process memory, so a worker dying loses no work.

**What this does not require writing.** Talos exposes a gRPC machine API with mTLS and a Rust
client generated from the official protos already exists. Proxmox publishes an API schema, so its
client is generated rather than hand-written — the principal community crate is roughly two years
stale, which is a reason to generate rather than to depend.

## Decision 8 — tier is a variable, and only the first is built

*Answers open question 3. The operator asked on 2026-08-18 whether both could be planned; they can,
and the distinction that makes it safe is between planning and building.*

**Tier 1 — apps.** The customer deploys containerised software — WordPress and similar — and never
runs `kubectl`. One Talos cluster per fleet; one Kubernetes namespace per customer; Kata as the
sandbox; `ResourceQuota`, `LimitRange` and NetworkPolicy as the guardrail. **Highest density, and
it is what BET-0009 already sells.**

**Tier 2 — Kubernetes.** The customer gets a real API server, their own CRDs and their own RBAC, via
vcluster; dedicated worker nodes for those who need the workload boundary too. Lower density, higher
price, a different competitor set.

| | Tier 1 — apps | Tier 2 — Kubernetes |
| --- | --- | --- |
| Customer sees | An application | A cluster |
| Isolation | Namespace + Kata microVM | Virtual or dedicated control plane |
| Density | Highest | Lower, and the price must carry it |
| Ships | **B1** | On trigger, not on schedule |

**The architecture admits both from the first commit; only tier 1 is built.** This is decision 4
generalised: *tier is a variable*, exactly as fleet is. A workload's placement — which fleet, which
tier, which runtime class — is an input to one provisioning path, never a branch in it. Designing
for both tiers costs a field; building both costs a second system.

**vcluster is Apache 2.0**, so tier 2 stays available rather than becoming a purchase. That is worth
recording because the Omni finding in decision 5 shows how easily the reverse happens, and a tier
planned against a source-available dependency is a tier that may not be buildable when its trigger
fires.

**What would make tier 2 arrive early:** a customer who will pay for it before tier 1 has proven
its density. That is a repricing of BET-0009 rather than a scheduling change, because the per-app
unit of sale is what the whole bet rests on — a cluster is not a workload.

## Decision 9 — the fleet manager is `svc-fleet`, one repository, class `service`

*Answers open question 4.*

**`svc-fleet`**, and under C-0007's derivations that fixes the rest: crate `paridyum-fleet`, binary
`fleet`.

**Why `service` and not `infra`.** `class-registry.yaml` gives `infra` `deployable: false` and
`ci: false`, and ADR-0037's correction states the reason plainly — *"an infra repository DEFINES the
cluster; it does not run inside it."* The fleet manager runs, continuously, with an API and a
reconciler. It is not a description of a cluster. Note the consequence of getting this wrong: an
`infra-` repository has **no CI standard at all**, which is a live gap that file already records, and
putting a customer-facing Rust service there would inherit it.

**Why `service` and not `platform`.** `platform` is *"platform services every product depends on."*
The fleet manager is the hosting product's engine. Under decision 2 Paridyum is provisioned by it,
which looks like a dependency — but that is an infrastructure dependency at build time, not a
runtime one every product carries. `service` is *"a business capability service. One repository is
one bounded context,"* which is exactly what fleet lifecycle is.

**One repository, not several.** Fleet lifecycle is one bounded context. The reconciler is a worker,
and `class-registry.yaml` is explicit that a worker is not a separate class — *"a service with no
`exposes.http` block… adding a `worker-` prefix would classify by a property the manifest already
records."*

**ADR-0042 is not violated, and this is worth stating so it is not read as sneaking in a sixth.**
That ADR fixes the first wave of **platform contexts** at five. `svc-fleet` is not a platform
context, is a different class, and sits on a different track. It adds nothing to the wave ADR-0042
bounds. What it does add is a repository the operator maintains, and that cost belongs to BET-0009
trigger clause 1 rather than to ADR-0042.

## Decision 10 — the request path is enumerated, and the fleet manager is absent from it

*Answers open question 2, which was the last blocker on this RFC.*

**The rule: no request that serves a customer's end user may traverse the fleet manager.** Decision
3 permits seeding on a machine with no availability of any kind, and this is the entire
justification. It is therefore an architectural constraint rather than a preference, and it is
enumerated rather than asserted:

| Path | Fleet manager on it? | What keeps it off |
| --- | --- | --- |
| **A** · End user loads a hosted workload | **No** | RFC-0016 — the rented edge terminates TLS and resolves `Host` to a region. The fleet manager *writes* routing config; the edge *serves* it |
| **B** · Customer uses the management interface | **Yes, and correctly** | Management is not serving. The fleet manager being down means nothing can be deployed; everything deployed keeps running |
| **C** · Certificate issuance and renewal | **No** | RFC-0016 already gives per-hostname certificates to the edge. **Brokering ACME through the fleet manager is forbidden** |
| **D** · Metering | **No — by this decision** | Pull-based only. See below |
| **E** · Entitlement and quota | **No, structurally** | Decision 6 put enforcement in Kubernetes `ResourceQuota` and `LimitRange` — static configuration, not a live lookup. Suspension is a lifecycle operation that scales a workload to zero |
| **F** · DNS for customer domains | **No** | Customers delegate to the edge; a DNS provider serves the zone |

**Metering is pull-based and never synchronous.** The fleet manager scrapes cluster metrics on an
interval, buffers, and reconciles usage afterwards. A per-request counter write is the easiest way
in the world to put the fleet manager on every request by accident, and it would do so while looking
like a small feature.

**Row E is worth noticing, because nobody designed it.** Quota enforcement moved into Kubernetes for
an unrelated reason — Proxmox enforces hard limits on storage only (decision 6) — and the
consequence is that entitlement became static configuration rather than a live call. **A decision
taken for a cost reason turned out to protect an availability property.** That is a coincidence
rather than a plan, and it is recorded so that a later change which reintroduces a live entitlement
lookup is understood as reopening this question rather than as a refactor.

**Row C is the dangerous one and no test will catch it.** A renewal brokered through the fleet
manager fails silently and surfaces at certificate expiry, weeks later. It is forbidden structurally
rather than tested, because a fault with a latency measured in weeks is outside any reasonable test
window.

**This is verified, not argued.** TST-0062 stops the fleet manager and asserts a hosted workload
still serves — STD-0009's *"the denial is the assertion"* applied to availability rather than
isolation. The enumeration above is true on the day it is written; the test is what keeps it true,
because this property decays on any deployment that adds a feature and prose does not notice.

## What "right" means here, as a checklist

*Added 2026-08-18, because "do it properly" needs a definition that can be checked.* Doing this
right is a small set of invariants, not a large set of features. Each is nearly free at the first
commit and expensive or impossible afterwards — the argument RFC-0013 was accepted on:

1. Reconciliation, not imperative (decision 7)
2. **No path into a fleet except the fleet-manager API** (decision 2) — Talos's absent SSH and
   shell is what makes this enforceable rather than aspirational
3. Isolation structural, not configured (decision 6)
4. Quota enforced where it can be enforced (decision 6)
5. Paridyum on the same path as any customer, no bypass (decision 2)
6. Fleet is a variable, never a branch (decision 4)
7. **Every state transition auditable** — RFC-0022's tenant-visible access log and BET-0008's
   metering both read this, and neither can be reconstructed from logs never written
8. Fleet-manager state restorable off-fleet (decision 3)

**None of these is a feature and none blocks a narrow first release.** Scope and correctness are
independent axes here, and this list is the correctness one.

## Consequences

**Gained**

- One provisioning system, one monitoring stack, one on-call rotation, for two businesses.
- The platform's infrastructure is exercised by paying strangers before it carries a paying tenant.
- BET-0008 open question 1 — the fully-loaded cost of one tenant — becomes measurable, because the
  fleet manager meters Paridyum the same way it meters anyone.

**Accepted costs**

1. ⚠️ **This is materially more work than BET-0009 assumed, it was chosen knowingly, and it is the
   largest risk in this RFC.** BET-0009 stage 1 reads as *"rent a node, sell managed apps"* — a
   weekend of Terraform and a payment link. A fleet manager with an API, quotas, metering and a
   customer model is a **product, built before the product.**

   A cheaper path was proposed on 2026-08-18 and declined: provision the first customers by hand and
   automate only when manual hurts, on the same reasoning RFC-0013 D5 uses to defer the docket —
   *"a registry with zero entries governs nothing."* **The operator chose to build it in Rust
   anyway**, and this entry exists so the choice is findable rather than forgotten.

   **BET-0009 trigger clause 1 is the control, and it points at this cost.** If Phase 3a slips more
   than six weeks with the fleet manager a contributing cause, the bet dies and this RFC dies with
   it — regardless of how good the fleet manager is by then. That is the deal this cost was accepted
   under.

2. **A build-versus-buy question this RFC does not answer.** ADR-0005 locks Rust for runtime
   services. A Rust fleet manager before `platform-tenancy` exists inverts ADR-0042's build order.
   The alternative — assemble the first fleet manager from existing tools and replace it later —
   collides with REQ-0012's demand that every external dependency be *"approved, scoped, and
   exitable."* **Open question 1**, and it is the one that decides whether stage 0 is weeks or
   months.

3. **Paridyum's platform can now be taken down by a hosting-side defect.** That is the dogfooding
   benefit and the operational risk in one sentence, and it is a real reduction in platform
   availability accepted in exchange for finding those defects with our own service degraded rather
   than a customer's.

4. **Two fleets are two estates.** Patching, monitoring, backup, on-call — doubled. BET-0009 cost 5
   already carries this; sharing the fleet manager reduces it but does not remove it.

## Alternatives considered

**Two independent estates, built separately.** The naive reading of BET-0009 plus RFC-0019.
Rejected because it builds the same provisioning system twice for one operator, and because it
forfeits decision 2's dogfooding entirely — the platform would be provisioned by a path no customer
ever exercises, which is how the customer path rots.

**One fleet, isolation by VM or namespace.** Considered and refused on 2026-08-17, recorded in
BET-0009 decision 1. Saves one rented node and puts arbitrary customer containers on the same metal
as every tenant's data. Unchanged by this RFC.

**Platform first, hosting later.** The conservative order, and it remains available — nothing here
is irreversible before stage 1. Rejected as the default because it forfeits the revenue that funds
the metal and the density measurement the platform's own pricing needs, while the platform's build
order is fixed by ADR-0042 regardless.

**Fleet manager on rented metal from the start, never on `sitar`.** Marginally safer and costs one
more node from day one. Worth revisiting if open question 2 resolves badly — if the fleet manager
turns out to sit on a request path, this becomes the decision rather than an alternative.

## Open questions

1. ~~**Build or assemble the first fleet manager?**~~ **ANSWERED 2026-08-18: build it, in Rust.**
   Decided by the operator after the scope cost was put to them explicitly and reaffirmed, and
   after Omni was found BUSL-licensed for production. Decision 5 carries the build-versus-adopt
   line; accepted cost 1 carries the cost; BET-0009 trigger clause 1 measures it.

2. ~~**Can the fleet manager be kept off every request path?**~~ **ANSWERED 2026-08-18 by decision
   10.** Paths enumerated; five are off-path under RFC-0016 and decision 6, one (management) is
   deliberately on-path and harmless, and metering is constrained to pull-based by this RFC.
   Verified by TST-0062 rather than asserted.

3. ~~**One Talos cluster per hosting customer, or namespaces with Kata inside a shared cluster?**~~
   **ANSWERED 2026-08-18 by decision 8: namespaces with Kata for tier 1, and tier 2 planned but not
   built.** A cluster per customer was rejected on density — three control-plane nodes and etcd
   wanting roughly four times its database size in RAM is a customer's control plane eating the
   margin before an application runs.

4. ~~**Which repository, and which class?**~~ **ANSWERED 2026-08-18 by decision 9: `svc-fleet`,
   class `service`, one repository.** ADR-0042 turns out not to be engaged — it bounds platform
   contexts, and this is neither.

5. **Does a hosting customer get a `tenant` row?** Carried unchanged from BET-0009 open question 2.
   If yes, RFC-0022's no-privilege guarantee and access log extend to hosting customers — and
   invariant 7 above makes that cheap rather than expensive, which is an argument for yes.

6. **Which CSI driver?** RFC-0023 open question 1, now with a third consumer and no more answered
   than before.

7. **What meters Paridyum's own usage before `platform-metering` exists?** Decision 2 requires an
   internal invoice; RFC-0009 §7 fires that context on *"the first resource whose overuse costs us
   money,"* which under this RFC is every resource from stage 2.

## Related

- **BET-0009** — what hosting sells; decision 1 there is refined by decision 1 here
- **BET-0008** — the estate as cost of goods, and the metering this RFC makes possible
- **MKT-0001** — the crossover that fires stage 4
- **C-0008** · **ADR-0049** — tenant zero, never privileged; decision 2 is the same rule for metal
- **ADR-0050** — product zero, and the asymmetry argument decision 2 borrows
- **RFC-0019** — one Proxmox cluster per datacentre; decision 3 there is satisfied, not amended
- **RFC-0023** — the substrate that seeds stage 0, and the invariant decision 4 extends
- **RFC-0016** — the rented edge, which both fleets need and which is still `proposed`
- **ADR-0042** — the platform build order this RFC runs alongside rather than through
- **GLO-0009** — fleet manager, and why this RFC stopped saying "control plane"
- **GLO-0007** — control region, which is what "control plane" still means
