---
id: RFC-0019
title: A datacentre is one Proxmox cluster, and Kubernetes runs on virtual machines above it
type: rfc
status: proposed
created: 2026-08-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [proxmox, kubernetes, datacentre, virtualisation, ceph, talos, infrastructure, adr-0020, adr-0037]
breaking: false
enforced_by: []
sources:
  - url: https://pve.proxmox.com/wiki/Cluster_Manager
    title: Proxmox VE — Cluster Manager
    retrieved: 2026-08-12
    supports: >
      Corosync requires under 5 ms latency between all nodes; above roughly 10 ms with more
      than three nodes, stable operation is unlikely. Decision 1 rests entirely on this.
  - url: https://pve.proxmox.com/wiki/Stretch_Cluster
    title: Proxmox VE — Stretch Cluster
    retrieved: 2026-08-12
    supports: >
      Supported stretched-cluster cases are same-datacentre fire sections, same campus, or
      dark fibre; a Q-Device tie-breaker tolerates ~100 ms where corosync does not.
  - url: https://forum.proxmox.com/threads/how-many-nodes-in-a-single-cluster-with-ceph-requirement.175884/
    title: Proxmox forum — node count for a Ceph cluster
    retrieved: 2026-08-12
    supports: >
      Ceph functions at three nodes and five is the production minimum — three tolerates one
      node failure, five tolerates two.
  - url: https://www.thomas-krenn.com/en/wiki/Ceph_Perfomance_Guide_-_Sizing_&_Testing
    title: Ceph performance guide — sizing and testing
    retrieved: 2026-08-12
    supports: At least four OSDs per node; more smaller OSDs outperform few large ones.
  - url: https://cyberpanel.net/blog/kubernetes-proxmox
    title: Kubernetes on Proxmox — setup and best practices, 2026
    retrieved: 2026-08-12
    supports: >
      etcd is write-intensive and requires SSD-backed storage; three control-plane nodes
      behind a virtual IP; Cilium over Flannel for production network policy.
  - url: https://foggyclouds.io/2026-02-08-k8s-talos-proxmox-deploy/
    title: Deploying Kubernetes on Proxmox with Terraform and Talos
    retrieved: 2026-08-12
    supports: >
      Talos is immutable and API-only — no SSH, no shell — with the whole lifecycle including
      upgrades driven declaratively. Decision 2's node-OS argument.
  - url: https://a-cup-of.coffee/blog/talos-capi-proxmox/
    title: Cluster API + Talos + Proxmox
    retrieved: 2026-08-12
    supports: A Cluster API provider for Proxmox exists, which decision 2 defers rather than adopts.
---

# RFC-0019 — A datacentre is one Proxmox cluster

## Status

**PROPOSED** as of 2026-08-12. Feedback deadline **2026-09-11**.

Fifth in the region series. RFC-0014 named the datacentre and deliberately did not say what one is
made of. This says. It does **not** touch ADR-0020, which is `locked` — Kubernetes remains the
deployment target and its charts, probes and limits are unchanged. This is about what the cluster
runs *on*, which ADR-0037 assigns to infrastructure repositories.

## The question

RFC-0014 established that a region contains one or more datacentres, that a datacentre is a failure
domain, and that adding one is an operational act. It left the substrate unspecified.

The proposal is Proxmox VE in each datacentre, with Kubernetes running in virtual machines on top,
and the same Proxmox estate also hosting virtual machines for other work.

Three things have to be settled before that is buildable: **what a Proxmox cluster's boundary is**
(it turns out physics decides this, not preference), **how Kubernetes is created on it**, and
**whether the platform may share a cluster with anything else**. The third is the one with a
security answer rather than an engineering one.

## Decision 1 — one Proxmox cluster per datacentre, and it never spans two

**A datacentre is exactly one Proxmox VE cluster. A Proxmox cluster never spans datacentres.**

This is not a preference. Proxmox's cluster membership runs on corosync, which **requires latency
under 5 ms between all nodes**, and above roughly 10 ms with more than three nodes stable operation
is unlikely. Proxmox's own stretched-cluster guidance scopes the supported cases to *different fire
sections in the same datacentre, different buildings on the same campus, or locations connected by
dark fibre*. Consistency matters as much as the average — a spike past 5 ms triggers timeouts even
when the mean is fine.

**So `in-mum-1` and `in-hyd-1` are two independent Proxmox clusters, not one stretched cluster.**
Roughly 700 km apart, they are not a campus and not dark fibre. Nothing at the hypervisor layer
joins them, which is exactly right — RFC-0014 calls a datacentre a failure domain, and a shared
corosync ring would make two failure domains into one.

**Within a datacentre, the fire-section case is available and worth using.** Two fire sections in
one facility satisfy the 5 ms rule and give the cluster a real internal failure boundary. That is
where a stretched Proxmox cluster belongs, if anywhere.

**A tie-breaker is required at even node counts.** Proxmox's Q-Device tolerates ~100 ms, unlike
corosync itself, so it may sit in a third location without violating decision 1.

## Decision 2 — Kubernetes runs in virtual machines, declaratively, on an immutable node OS

**Kubernetes control-plane and worker nodes are Proxmox virtual machines**, not containers on the
hypervisor and not bare metal. That is what makes the same hardware able to carry non-Kubernetes
workloads at all, and it keeps ADR-0020's contract — charts, probes, limits — untouched above.

**The node operating system should be immutable and API-only.** Talos Linux is the current answer:
no SSH, no shell, configuration applied declaratively, the whole lifecycle including upgrades driven
through an API rather than by logging in. The argument is the same one this repository already makes
about generated output under C-0004 and about `PreToolUse` hooks under ADR-0007 — **a violation that
cannot be expressed is worth more than a violation that is forbidden.** A node nobody can SSH into
cannot drift, which is the failure mode ADR-0009's archived estate demonstrated at scale.

**Provisioning is declarative and lives in an `infra-` repository**, per ADR-0037. Terraform or
OpenTofu against the Proxmox API, with the cluster defined as code and no hand-built node. Cluster
API providers for Proxmox exist and are the more ambitious path; they are worth revisiting once
there is more than one cluster to manage, and are premature at one.

**Sizing rules that are not negotiable, because getting them wrong is expensive later:**

| Concern | Rule |
| --- | --- |
| Control plane | Three control-plane VMs, `kube-apiserver` behind a virtual IP |
| etcd storage | **Local NVMe, never Ceph.** etcd is write-intensive and latency-sensitive; putting it on network storage is the most common way to build a cluster that works until it is busy |
| Ceph nodes | Three is the minimum that functions, **five is the minimum for production** — three tolerates one node failure, five tolerates two |
| OSDs | At least four per node; more smaller OSDs outperform few large ones |
| Network | 10 GbE floor, **25 GbE for a new build**, with Ceph public, Ceph private and corosync on separate interfaces |
| CNI | Cilium rather than Flannel, for network policy and observability |
| Backup | Proxmox Backup Server for virtual machines, Velero for cluster state — they are different objects and neither substitutes |

## Decision 3 — the platform never shares a cluster with anything else

**Paridyum's Kubernetes runs on Proxmox clusters that carry no other workload. Virtual machines for
other projects run on a separate cluster.**

This is the security decision in this RFC and the reason it is not merely an infrastructure note.
C-0006 names what is *not* commodity and therefore ours to build:

> **Provable isolation at two levels.** Not "we use separate schemas" but a guarantee with a check
> behind it.

Everything ADR-0024 does — two columns, RLS `FORCE`, operator scope as a positive grant — sits
*inside* the platform's database. **A virtual machine for another project is outside that model
entirely**: no `tenant_id`, no policy, no session context, nothing the migration gate can see. If it
shares a hypervisor, a Ceph pool and a corosync ring with the platform, then the isolation guarantee
rests on the hypervisor boundary rather than on anything this repository can check — and a
hypervisor escape or a noisy-neighbour event crosses the boundary the product is sold on.

Separation is cheap now and expensive later, and "separate cluster" is a smaller ask than it sounds:
Proxmox clusters are per-datacentre anyway under decision 1, so this is one more cluster in a
facility that already has one, not one more facility.

**A question this RFC cannot answer.** If those other virtual machines are *sold* rather than run
for ourselves, that is infrastructure resold, and C-0006 anti-goal 2 is explicit: *"We will not
resell infrastructure at a markup as a revenue line."* BET-0002 carries the argument. Running our
own projects on our own hardware raises no such question. **Which of the two is intended has to be
stated before this decision means anything**, and it is open question 1 rather than an assumption
made here.

## Decision 4 — replication between datacentres is a database concern, and it is not free

**This corrects RFC-0015 decision 3**, which claimed a synchronous standby across datacentres gives
*"a near-zero recovery point and a recovery time measured in minutes, without a single byte crossing
a border."* The residency half is right. The near-zero half was written without a substrate.

Decision 1 makes the two datacentres independent Proxmox clusters with nothing shared at the
hypervisor layer. So replication happens **at the database**, and synchronous commit across roughly
700 km adds that round trip to every write. That is a real and permanent cost on the hot path, not a
tuning detail.

The choice is therefore explicit rather than assumed:

| | Recovery point | Cost |
| --- | --- | --- |
| Synchronous **within** a datacentre, asynchronous across | Zero locally, seconds across | Write latency unchanged |
| Synchronous **across** datacentres | Zero | Every commit pays the inter-datacentre round trip |

**Recommended: synchronous within, asynchronous across**, with the replication lag stated as the
recovery-point objective REQ-0004 item 12 requires. Losing a datacentre then costs seconds of
writes, not zero — and RFC-0015 decision 3's promise is amended to say so rather than left to be
discovered during the first failover drill.

## Consequences

**Accepted costs**

1. **This is a hardware business now.** Capacity planning, procurement, spares, remote hands, and a
   failure mode — a dead node at 3 a.m. — that no cloud account has. ADR-0009's archived estate ran
   `cloud_provider: inhouse` across eight regions, and REGION-MODEL.md §7 judged that *"a very large
   operational surface for a platform at this stage."* One datacentre actually built is a different
   proposition from eight declared, but the direction is the one that warning was about.

2. **C-0006 lists web hosting as commodity**, and this owns it rather than depending on it. The
   honest counter-argument is BET-0002: a flat price per running business needs predictable unit
   costs, and owned metal makes that price defensible where a cloud bill that scales with usage does
   not. Both are true; the tension is recorded rather than resolved.

3. **A Terraform apply can now reboot a node with virtual machines on it**, and ADR-0037 decision 4
   states plainly that `infra-*` release governance is *"genuinely ungoverned"* and belongs to
   REQ-0004, which is deferred. This decision raises the blast radius of that gap from a cluster
   change to a power event.

4. **Ceph at five nodes and 25 GbE is a real floor.** A three-node cluster is a demonstration; it
   tolerates one failure, and a production region that tolerates one failure has no maintenance
   window.

5. **Two substrates to operate**, since the platform cluster and the general-purpose cluster are
   separate under decision 3. That is the price of the isolation argument and it is paid per
   datacentre.

**Gained**

- The datacentre stops being an abstraction. RFC-0014's *"adding a datacentre is an operational
  act"* becomes a concrete, costed operation.
- Unit economics come under control, which is the thing BET-0002 wagers on and the thing a cloud
  bill makes unpredictable.
- The isolation boundary the product is sold on stays inside machinery this repository can reason
  about, rather than resting on a shared hypervisor.
- Capacity and failure domains separate cleanly: **more nodes** answers capacity, **more clusters**
  answers failure domains, and RFC-0014 conflated the two slightly.

## Alternatives considered

- **Rent compute from a cloud provider in each jurisdiction.** Far less operational surface, and it
  is what C-0006's commodity classification points at. Rejected on BET-0002's economics and on
  residency control — but it is the alternative to revisit first if decision 1's operational cost
  proves the dominant one, and it is a per-region choice rather than a platform-wide one.

- **Kubernetes on bare metal.** Better performance, no virtualisation overhead, and it removes an
  entire layer. Rejected because the same hardware could then serve nothing else, which is the whole
  point of the proposal, and because node lifecycle becomes a hardware operation rather than an API
  call.

- **One stretched Proxmox cluster per region**, spanning both datacentres. Rejected by corosync's
  latency requirement rather than by preference. It would also merge two failure domains into one,
  which defeats RFC-0014's reason for having datacentres at all.

- **Share one Proxmox cluster between the platform and other workloads.** Cheaper by exactly one
  cluster per datacentre, and rejected in decision 3 on the isolation argument. Recorded because it
  is the version that will be proposed again when the second cluster's cost is visible and the risk
  is not.

- **A mutable node OS with configuration management.** Familiar, and rejected on ADR-0009's evidence:
  the archived estate's drift is what an SSH-able node produces over eighteen months.

## Open questions

1. **Are the other virtual machines sold, or ours?** Decision 3 flags this. Sold is C-0006
   anti-goal 2 and needs that argument answered before anything is built for it; internal raises no
   commercial question at all. Everything else in decision 3 holds either way.

2. **Does `infra-proxmox` become a repository, and does ADR-0037 still fit it?** ADR-0037 made
   `infra` `deployable: false` for repositories holding cluster definitions. A repository that
   provisions hypervisors is the same shape one layer down, and probably needs the release governance
   ADR-0037 says nothing currently provides.

3. **What is the Proxmox subscription posture?** The enterprise repository requires a subscription,
   and REQ-0012 requires every external dependency be *"approved, scoped, and exitable"*. Running
   the no-subscription repository in production is a decision, not a default.

4. **Does the region registry gain a substrate field?** `region-registry.yaml` records datacentres by
   id and status. Whether it should also record what each is made of — cluster size, Ceph presence —
   or whether that belongs entirely to an `infra-` repository, is unresolved. The registry's own
   FIX 2 argues against putting operational detail in a governance file.

## Reversal conditions

- **Decision 1 reverses if a datacentre pair ever sits on dark fibre within corosync's envelope.**
  Then a stretched cluster becomes available — and would still be a bad idea, because it recombines
  two failure domains.
- **Decision 3 reverses only with a written argument about what replaces the hypervisor boundary.**
  Not on cost.
- **The whole RFC reverses to rented compute per region** if operational load rather than capital
  proves to be the binding constraint. That is a real possibility and it is a per-region decision.

## Related

- RFC-0014 — names the datacentre; this says what one is made of, and sharpens capacity versus failure domain
- RFC-0015 — decision 3, whose recovery-point claim decision 4 corrects
- ADR-0020 — `locked`; Kubernetes as the deployment target, untouched by this
- ADR-0037 — infrastructure repositories define the runtime; also the ungoverned-apply gap this widens
- ADR-0003 — Kubernetes from day one, which this is the substrate for
- ADR-0009 — the archived estate, its `inhouse` regions, and the drift an SSH-able node produces
- C-0006 — web hosting as commodity, "provable isolation" as the product, and anti-goal 2
- BET-0002 — the flat price that owned metal makes defensible
- ADR-0024 — the isolation model that a foreign virtual machine sits entirely outside
- REQ-0004 — item 12's RPO/RTO, and the release governance ADR-0037 leaves unowned
- REQ-0012 — the vendor posture open question 3 needs
- `archived/paridyum-docs/REGION-MODEL.md` — §7, and eight in-house regions as a warning
