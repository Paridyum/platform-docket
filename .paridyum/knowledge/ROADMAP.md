# Paridyum — Roadmap

**Rewritten 2026-08-18.** Covers the path from "governance exists, nothing runs" to a platform and
a hosting business, both live, on metal we control.

This file lives in `knowledge/` root beside `DECISION-DEBT.md`, **not** in a typed artifact
directory. It is cross-repo and hand-maintained, which is exactly the class `knowledge/` refuses to
carry *as an artifact* — so it is carried as prose instead. Nothing here binds anything. Where this
document and a knowledge artifact disagree, **the artifact wins** — every claim below is sourced to
one, and the sources are the authority.

> **Durability, resolved 2026-08-18 — T-0.1 closed.** This file and `TASKS.md` moved here from
> `~/Developer/paridyum/`, which is not a git repository. They now have history, backup and the
> ADR-0025 checkpoint hook behind them.
>
> **T-0.1 framed this as a trade-off and it was not one.** The task recorded that placing them in
> `knowledge/` "puts them under the gate, which is an argument both for and against." It does not.
> `scripts/check-knowledge.sh` scans only the directories named in `artifact-registry.yaml`; the
> other root-level prose here has always been ungated. Verified by re-running the gate across the
> move and observing the artifact count unchanged.

---

## 1. Where we actually are

**Do not trust a number written here.** Run `scripts/check-knowledge.sh` — it prints the artifact
count, the declared-versus-implemented check split and the unenforced locks on every run. This
repository has been caught by a hand-maintained count three times, and ADR-0035 now rejects new
ones at the gate.

**The one-sentence state:** an unusually complete governance system exists, it audits itself
honestly, and it governs no running software.

**What changed on 2026-08-18** — the largest single day of commercial decisions so far:

| | |
| --- | --- |
| **RFC-0013 accepted** | The IP posture is decided. D1–D4 have owners, D5 stays deferred, and signing was already in place |
| **C-0006 anti-goal 2 amended** | Narrowed from the asset to the **unit of sale**, and given an argument that survives BET-0007 being killed |
| **BET-0008 accepted** | The estate is cost of goods. Closes RFC-0019 open question 1, open since 2026-08-12 |
| **BET-0009 accepted** | Hosting may be sold — by the workload, on a separate data plane, under four triggers |
| **RFC-0025 proposed** | The fleet manager: how hosting gets built, and Paridyum as customer zero |
| **MKT-0001…0003** | Cost, operator evidence and DPDP status now live as dated observations rather than inside the wagers |

**The one-sentence plan:** stop widening the governance layer; run two tracks that converge on one
substrate; let everything else wait for a trigger that is already written down.

---

## 2. The ordering argument

Four constraints fix the order. None is a preference.

**Legal ordering was irreversible, and it is now closed.** RFC-0013 D3 required signed commits and a
conception log from the first commit; git history cannot be retroactively signed into
contemporaneous evidence, and India has no trade secret statute to fall back on. **Accepted
2026-08-18.** Still owed and still expiring: D1's technical-effect thesis and D2's baseline
benchmark, neither of which can be taken once the naive implementation is gone.

**Decomposition precedes implementation.** ADR-0019, and RFC-0009 stops explicitly: *"Q1 and Q3 must
be answered before `platform-tenancy` or `platform-identity` is created."* Both were answered in the
2026-08-08 revisions.

**The harness must fail before it is trusted.** `templates/infra-ci/rust-ci.yml` has **never
executed**. The first `platform-` repo is its first caller, and discovering its checks are broken
while also debugging a new service is the expensive ordering.

**Isolation cannot be retrofitted.** *New 2026-08-18.* RFC-0025 decision 6 establishes that a
customer workload landing on a non-sandboxed runtime must be *impossible*, and that quota cannot
live in Proxmox — which enforces hard limits on storage only, leaving CPU and memory best-effort.
Both are cheap at the first commit and structural afterwards, which is the same shape of argument
RFC-0013 won on.

---

## 3. Two tracks, and where they meet

The platform and the hosting business are **not sequential**. They share a substrate, and one of
them provisions the other.

```
Track A — the platform        Phase 0 → 1 → 2 → 2b → 3a → 3b → 3c → 3d
                                                ↘            ↗
Track B — the fleet manager                B0 → B1 → B2 → B3 → B4
                                                      ↑
                     B2 provisions Track A's runtime (RFC-0025 decision 2)
```

**The convergence point is B2.** The fleet manager provisions `fleet-platform`, and Paridyum's
platform is deployed onto it **as customer zero, through the same API as any hosting customer.**
That is C-0008 one layer down, and it means Track A stops needing a provisioning story of its own.

**The governing risk is that Track B eats Track A.** BET-0009 trigger clause 1 is the control, and
it fires on **schedule slippage, not margin**: if Phase 3a slips more than six weeks with the fleet
manager a contributing cause, BET-0009 and RFC-0025 both die regardless of how good the fleet
manager has become by then. That is the deal Track B was authorised under, recorded in RFC-0025
accepted cost 1.

---

## 4. Track A — the platform

### Phase 0 — Commercial and legal ground · **nearly closed**

**Blocks:** the first runtime commit.

| Item | State |
| --- | --- |
| RFC-0013 — IP posture | ✅ **Accepted 2026-08-18** |
| Roadmap durability — T-0.1 | ✅ **Closed 2026-08-18** |
| Price shape | ✅ BET-0007 — floor plus royalty |
| Operator and franchise model | ✅ BET-0005, BET-0006 |
| **Payment path research** | ⛔ T-0.10 — split settlement through a licensed aggregator is assumed and unresearched. Gates BET-0007's royalty and nothing else, so start it early and let it run |
| **Vertical one** | ⛔ T-0.6 — criterion fixed by BET-0005; the choice is not made |
| **Floor and royalty numbers** | T-0.7 — needs fully-loaded idle-tenant cost, which is unmeasured. See §6 |
| **RFC-0022** | ⛔ T-0.11 — questions 2–5 open; question 2 decides whether the access log is a guarantee or a partial one |
| D1 thesis · D2 harness | T-0.3, T-0.4 — owed, and both expire |
| Incorporation sequence | T-0.8 — build and measure → incorporate → DPIIT → file |

**Exit gate:** payment path cited · vertical one chosen · floor and royalty set · RFC-0022 accepted ·
D1 thesis exists · incorporation scheduled.

### Phase 1 — Close the decisions that block repository #1

RFC-0009 (Q5 blocks the first shared library) · RFC-0018 (does `infra-` sprawl) · RFC-0014 and
RFC-0017 (the region model, a first-class input to identity) · REQ-0001 promoted with a named
secrets mechanism.

**Raised in priority by Track B:** RFC-0016 and RFC-0019 are both still `proposed` and both are
load-bearing for hosting. RFC-0016 especially — the rented edge is the only inbound path *either*
track has.

**Exit gate:** RFC-0009 `accepted` with Q5 answered · RFC-0018 decided · region model decided ·
REQ-0001 accepted with a mechanism.

### Phase 2 — Make the harness enforce before it guards real code

Watch `rust-ci.yml` fail by planting each defect it claims to reject. Distribute `forge` so its
checks run outside `tool-forge`. Close or downgrade every locked-but-unenforced artifact — STD-0009
is the sharpest, because it governs tenant isolation and Phase 3 builds exactly that boundary.

**Do not build the unimplemented checks no first-wave repo needs.** Declared is a legitimate state
(ADR-0014).

### Phase 2b — The substrate

`sitar-as-in-01-ar` per RFC-0023. Nested KVM is **verified** (T-2b.1, 2026-08-15). Remaining: accept
RFC-0023 and RFC-0016, provision with OpenTofu, stand up Talos `staging`, build the real secrets
mechanism, tunnel to the rented edge, deploy something trivial end to end.

**Every decision this phase was blocked on closed on 2026-08-19; none of it has run.** The CSI driver
is Longhorn (RFC-0023 D7), the provisioning tool is OpenTofu rather than Terraform (ADR-0052, on the
same BUSL exitability test that rejected Omni), OpenTofu's native state encryption answers RFC-0023
open question 2, and the edge vendor is Cloudflare (RFC-0016 D4). GLO-0011 defines `environment` and
dissolves a collision between locked ADR-0021 and RFC-0023 D3 that neither artifact had noticed.

**What is left in this phase is now execution and two judgements.** The judgements are the operator's:
accept the two RFCs, and get a price for Cloudflare's Data Localization Suite — the add-on carrying
the three controls that answered the residency criterion, and the one input that could reverse the
vendor choice.

**This phase now also seeds Track B** — RFC-0025 stage B0 puts the fleet manager on this cluster.

**Exit gate:** a request from the public internet reaches a pod on `staging` and returns, having
travelled git → build → Terraform → cluster → rented edge.

### Phase 3 — The first wave

**3a — enough platform for exactly one tenant.** Order fixed by ADR-0042 decision 2:

```
platform-tenancy  →  platform-identity  →  platform-routing  →  spec-product
```

**3b — product one, operated by Paridyum.** Where the company first has end-customer income, and it
does not depend on BET-0006 being right.

**3c — make it franchisable.** `platform-entitlement`, the `spec-` contract, and RFC-0022's D1 and
D5 — which **must exist before the first franchisee, not after.**

**3d — first franchised tenant.** REQ-0013's trigger fires; RFC-0022 D4 promotes export from a
deferred requirement to a launch commitment.

⚠️ **Unresolved before 3a.** RFC-0009 §7's wave included `platform-deployment`; ADR-0042 fixes the
wave at five and omits it. **Release, Certificate, and the act of running an artifact have no
owner.** Track B may absorb part of this — the fleet manager runs artifacts — and if it does, that
is a decision to write down rather than let happen.

**The standing temptation, named so it can be refused:** RFC-0009 §8 already rejected one
`platform-core` split later. *"Merging contexts to reduce repository count is how ANTI-0006 gets
built, and it is the one mistake here that is effectively permanent."*

---

## 5. Track B — the fleet manager and hosting

**Designed in RFC-0025** (`proposed`). Sells what BET-0009 permits: managed workloads, priced per
workload, on a data plane physically separate from the platform's.

### Blocking decisions, before any code

| | Decision | State |
| --- | --- | --- |
| T-0.13 | Build or adopt the fleet manager | ✅ **Build, in Rust.** No free adopt-path existed — Omni is BUSL |
| T-0.16 | Apps or Kubernetes? | ✅ **Apps.** Namespace + Kata, tier 2 designed and not built |
| T-0.17 | Repository and class | ✅ **`svc-fleet`, class `service`**, one repository |
| T-0.18 | Register the hosting product | ✅ **`hosting`, `catalogue`, `stage: proposed`** |
| T-0.14 | Can the fleet manager stay off every request path? | ✅ **Yes.** Paths enumerated in RFC-0025 decision 10, and **verified by TST-0062** rather than asserted |
| **T-0.15** ⛔ | Accept or reject RFC-0025 | **The only one left.** Everything it was blocked on is answered |

**One decision was deliberately not taken.** The `hosting` product sits at `stage: proposed`.
Promoting it to `building` is C-0006 anti-goal 6's gate, and it probably answers T-0.6 — hosting
becomes vertical one, or it competes with whatever vertical one turns out to be, and the second is
forbidden until the first has a renewed paying customer. See §7.

### The stages

| | | Cost |
| --- | --- | --- |
| **B0 · Seed** | Fleet manager on the RFC-0023 `staging` cluster. No customer workloads anywhere | ₹0 — hardware sunk |
| **B1 · First fleet** | One rented Indian node as `fleet-shared`. First external customers | ~₹3,000–4,000/mo |
| **B2 · Self-provision** | `fleet-platform` provisioned; Paridyum deployed onto it as customer zero. **Convergence with Track A** | +one node |
| **B3 · Evacuate the seed** | Fleet manager migrates to `fleet-platform`; `sitar` becomes a development substrate and a **rehearsed** cold rebuild target | ₹0 |
| **B4 · Crossover** | Owned colocation at the threshold MKT-0001 computes | Capex, on MKT-0001's trigger |

### Build versus adopt

C-0006's own commodity split decides it (RFC-0025 decision 5). **Adopt** the machine mechanics —
Proxmox API, Talos, the Kata sandbox. **Build** tenancy, quota, isolation policy, metering and the
desired-state reconciler, because C-0006 non-commodity item 2 says the isolation guarantee *is* the
product.

**This is not a cloud being built.** It is the multi-tenancy and policy layer over Proxmox and
Talos. No free adopt-path existed for that half: Sidero Omni is BUSL and requires a commercial
licence for production, failing REQ-0012's exitability test outright.

### The invariants — what "right" means

RFC-0025 carries the checklist. Each is nearly free at the first commit and expensive afterwards:
reconciliation not imperative · **no path into a fleet except the API** · isolation structural not
configured · quota enforced where it can be · Paridyum on the same path as any customer · fleet is a
variable · every state transition auditable · state restorable off-fleet.

**None is a feature and none blocks a narrow first release.** Scope and correctness are independent
axes; this list is the correctness one.

---

## 6. The measurement that unblocks pricing

**BET-0008 open question 1 and T-0.7 are the same missing number:** the fully-loaded cost of one
tenant. It cannot be computed today, because the denominator is density and density is unmeasured.

**Track B measures it.** Ten hosting customers on one node produce density, abuse rate and support
load directly — feeding BET-0007 trigger clause 1, BET-0009 trigger clauses 2 and 3, and the floor
Phase 0 currently cannot set.

That is the strongest non-revenue argument for Track B existing, and it is worth stating because it
is easy to forget while the schedule trigger is doing the arguing.

---

## 7. Waiting on a trigger

**Nothing below is scheduled.** Each fires on a condition already written down. This is the whole
product surface, named so it is visible and deliberately *not* planned in detail — a plan for
something years out is a fiction that decays, which is the failure ADR-0035 catches with counts and
ADR-0018 catches with triggerless requirements.

### Platform contexts

| Context | Fires when |
| --- | --- |
| `platform-metering` | the first resource whose overuse costs us money — **RFC-0025 makes this every resource from B2**, so expect it early |
| `platform-billing` | the first operator who must be charged |
| `platform-audit` | the first state change a customer could dispute — table first, context at the second writer. **Probably fires immediately** (RFC-0009 OQ2) |
| `platform-export` | REQ-0013's trigger: the first tenant-authored row |
| `platform-generation` | the third catalogue product is scoped, or two products share copied structure |

### Products

Registered in `product-registry.yaml` under ADR-0050.

| Product | State | Fires when |
| --- | --- | --- |
| `paridyum` | product zero, `building` | — |
| `tenancy` · `identity` · `routing` · `entitlement` | registered | Phase 3a |
| **`hosting`** — managed application hosting | ✅ registered `catalogue`, **`stage: proposed`** | **Promotion to `building` is the anti-goal 6 gate.** MKT-0001…0003 satisfy the dossier; the strategy decision is untaken |
| **catalogue vertical one** | undecided | T-0.6 — **and `hosting` may already be the answer.** If it is promoted, it is vertical one; if something else is chosen first, anti-goal 6 blocks hosting until that one has a renewed customer |
| catalogue vertical two | forbidden | C-0006 anti-goal 6 — **not before vertical one has a renewed paying customer** |

### Hosting tiers beyond the first

| Tier | Fires when |
| --- | --- |
| **Tier 2 — real Kubernetes** (vcluster, dedicated nodes) | a customer will pay for an API server before tier 1 has proven its density. **Designed for from the first commit, built on trigger** — RFC-0025 decision 8. vcluster is Apache 2.0, so it stays available rather than becoming a purchase |
| Managed Kubernetes for agencies | BET-0009 stage 3, and only while the abuse trigger stays comfortable |
| Backup and DR as a service | the dark horse — low abuse surface, high retention. Revisit if BET-0009 trigger 2 fires on managed workloads |
| GPU / AI inference | past the B4 crossover only. MKT-0002 records the margin and the megawatt-scale entry cost in the same breath |

---

## 8. Deliberately not on this roadmap

- **Unimplemented checks no first-wave repo needs.** Declared is a legitimate state (ADR-0014).
- **The `patterns/` directory.** Populate it *from* the first wave; a pattern with no implementation
  to generalise from is invented, not extracted.
- **The remaining deferred requirements.** Each has a trigger. Let them fire.
- **A second vertical.** C-0006 anti-goal 6.
- **Enterprise anything.** C-0006 anti-goal 5.
- **Regulated data.** C-0006 anti-goal 3 / ANTI-0007 — the only anti-goal whose violation can end
  the company in a single incident. **Track B adds a new way for someone who is not us to violate
  it**, so terms and an acceptable-use policy precede customer one.
- **Selling capacity by the gigabyte.** C-0006 anti-goal 2 as amended, and MKT-0001 shows why:
  retail is priced below our own production cost.

---

## 9. What would change this plan

- **T-0.16 answers "Kubernetes" rather than "apps."** Density drops, BET-0009 reprices, and Track B
  moves upmarket into a different competitor set. Not fatal, but a repricing rather than a tweak.
- **BET-0009 trigger clause 1 fires.** Track B dies and Track A absorbs the substrate work. Budget
  for this rather than treating it as failure — it is the control that authorised the track.
- **BET-0006 or BET-0007 reaches `review_date` 2026-12-01.** BET-0005 means Phase 3b's revenue
  survives the wedge being wrong.
- **BET-0007 trigger clause 3 fires** — split settlement proves unavailable. The royalty becomes
  unbuildable and BET-0002's flat shape returns. **Track B is unaffected**, being billed directly —
  a real diversification, and the only revenue line with no regulatory dependency.
- **RFC-0025 open question 2 resolves badly** — the fleet manager cannot stay off the request path.
  B0 moves to rented metal and the sequence costs one more node from day one.
- **Phase 2 finds `rust-ci.yml` substantially broken.** Likely, given it has never run.
- **`lib-tenancy` needs two implementations** (RFC-0009 Q5). Doubles the security mechanism and
  re-opens RFC-0009 §8's rejected alternative.

---

## Related

`CLAUDE.md` · `README.md` · `DECISION-DEBT.md` · `TASKS.md` ·
RFC-0009 (decomposition) · RFC-0013 (IP, accepted) · RFC-0019 (Proxmox) · RFC-0023 (substrate) ·
RFC-0025 (fleet manager) · C-0006 (market posture, amended) · C-0008 (tenant zero) ·
BET-0005…BET-0009 · MKT-0001…MKT-0003 · ADR-0042 (build order) · ADR-0050 (product axis)
