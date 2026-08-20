---
id: RFC-0023
title: The development substrate is production one level down, and the transition is a variable
type: rfc
status: proposed
created: 2026-08-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [substrate, proxmox, talos, kubernetes, workstation, hostname, edge, secrets, terraform]
breaking: false
enforced_by: []
---

# RFC-0023 — The development substrate is production one level down

## Status

**PROPOSED**, 2026-08-15. Opened because there is no substrate to deploy anything onto, and the
first one built will decide — by accident if not by decision — how much of it has to be rebuilt
when real hardware arrives.

## The question

**Where does code run before there is a datacentre, and how is that arranged so that acquiring a
datacentre changes a variable rather than a design?**

Two machines exist: a macOS authoring host, and an Arch Linux machine with 64 GB of memory. The
naive answer — run things on whichever is convenient — produces an environment that works and
transfers to nothing, which is the failure this RFC exists to prevent.

## What is already decided elsewhere

| Already decided | Where |
| --- | --- |
| Kubernetes is the deployment target for every deployable class | **ADR-0020 (locked)** |
| A datacentre is one Proxmox cluster, and it never spans two | RFC-0019 D1 |
| Kubernetes runs in VMs on an immutable, API-only node OS | RFC-0019 D2 |
| The platform never shares a cluster with anything else | RFC-0019 D3 |
| The edge is rented, terminates TLS, and resolves `Host` to a region | RFC-0016 D1 |
| A hostname is family, continent, country, sequence, optional platform | RFC-0021 D1 |
| Device families and host platforms are registry data | RFC-0021 D4 · `device-registry.yaml` |
| The hostname is not a database | RFC-0021 D8 |
| Signed commits and a conception log from the first commit | RFC-0013 D3 |
| Runtime secrets are managed outside the repository | REQ-0001 (`deferred`) |
| Test tiers, and which of them need a running deployment | STD-0008 · `test-registry.yaml` |

None of the above is restated below. Where this RFC appears to decide something already decided,
the other artifact governs.

## Decision 1 — the machine is `sitar-as-in-01-ar`

RFC-0021's fixed-host production, filled in: family `sitar`, continent `as`, country `in`,
sequence `01`, platform `ar`.

**Family is a class of physical hardware, not a class of workload.** `device-registry.yaml` says
so directly. A machine running a hypervisor and a Kubernetes cluster is a server whatever those
clusters serve, so `sitar` is correct even though everything on it is non-production. RFC-0021 D8
keeps environment out of the hostname for the same reason it keeps the datacentre out: the
hostname is an identity, not a record.

**`ar` is admitted to `platforms:` by this RFC.** Two characters, from the registry, no existing
code shadowed. The Argentina collision is real and does not bite: the registry's objection to `es`
for ESXi is that `sitar-eu-es-01-es` repeats one string inside a single name, which `ar` only does
for a device placed in Argentina. `tl` already collides with Timor-Leste and was allocated anyway.

**No workstation family is created and the macOS host is not registered.** RFC-0021 open question 1
asks whether personal devices belong in this registry and supplies the principle — *"an operator
device that reaches production is exactly what an inventory should know about."* Under decision 5
the authoring host reaches nothing. It is therefore not infrastructure, and inventing a family for
it would admit a family with no member that matters.

## Decision 2 — Arch on the metal, Proxmox as a guest, and the cost is stated

The conventional answer is Proxmox VE on the metal, since it is a hypervisor OS and nesting it
wastes a layer. That is correct for production and rejected here for a narrower reason: **this is a
working machine before it is a substrate**, and Arch is the environment its operator works in.

The costs, which are real and accepted rather than argued away:

1. **Two levels of nested virtualisation.** Arch KVM → QEMU → Proxmox KVM → Talos VMs. This works
   on modern hardware with nested KVM enabled and CPU passthrough, and it degrades in performance
   at L2. **It is also unverified** — see risk R1, which is the one that could invalidate this
   whole RFC.
2. **A rolling-release distribution underneath a hypervisor.** RFC-0019 D2 chose Talos *because*
   it is immutable and eliminates configuration drift; Arch is the most drift-prone mainstream
   distribution. That inconsistency is tolerable here only because nothing production depends on
   this host. **It would not be tolerable on the metal that replaces it**, and this RFC does not
   license it there.
3. **A layer that does not exist in production.** Anything that works only because QEMU is beneath
   Proxmox is a bug that will surface at the transition. Decision 4's invariant is the mitigation.

## Decision 3 — one hypervisor, one cluster, and its name is `staging`

RFC-0019 D1 makes a Proxmox cluster a *datacentre*; D3 forbids the platform sharing a *Kubernetes
cluster* with anything else. Multiple clusters on one hypervisor satisfy both, so the hypervisor is
provisioned once and clusters are the unit that multiplies.

**One cluster, and it is called `staging`.** Not `dev`, and the reason is decision 4's invariant
applied to a name. If this cluster is called `dev`, then acquiring metal produces `dev` and `prod`
and nothing staging — so something gets renamed, and a rename is a migration. Called `staging`, the
transition *adds* an environment: metal becomes `prod`, this stays `staging`, nothing changes
identity.

**A second environment was considered and dropped because no test tier serves it.** Of the tiers
in `test-registry.yaml`, only `post-deploy` requires a running deployment; `pr` carries a
ten-minute budget and STD-0008 forbids I/O in the unit tier outright. The fast inner loop is unit
and integration tests against a container on the authoring host, and it never needed a cluster.

**Topology mirrors production: three control-plane nodes and two workers.**

⚠️ **Three control-plane VMs on one physical machine is HA-shaped, not HA.** The host is a single
point of failure and losing it loses all three. This buys **no availability whatsoever** and must
never be described as though it does. What it buys is correct etcd quorum behaviour, realistic
drain and upgrade semantics, and manifests identical to what metal will run — the class of bug that
works at one node and breaks at three is exactly the class that would otherwise debut in
production.

Allocation, with 64 GB installed and 16 GB reserved for the host. **Revised 2026-08-19** when
decision 7 chose a CSI driver with a per-node memory requirement the first version could not meet:

| | |
| --- | --- |
| Arch host — editor, registry, build cache | 16 GB |
| Proxmox VE guest | 40 GB |
| ├ Proxmox overhead | ~4 GB |
| ├ 3 × Talos control plane @ 4 GB | 12 GB |
| ├ 2 × Talos worker @ 8 GB | 16 GB |
| └ headroom | ~8 GB |
| unallocated | 8 GB |

⚠️ **The first version of this table accounted for 48 GB of 64 and nobody noticed.** Host 16 plus
guest 32 leaves 16 GB unstated — not reserved for anything, not described as slack. It was found
while checking whether Longhorn's 8 GB-per-storage-node requirement fit, and it is the only reason
that requirement is affordable without squeezing the control plane down to 3 GB and putting etcd
under memory pressure on all three nodes at once.

The revision spends 8 GB of the 16 on the workers and leaves 8 GB genuinely spare, so the guest may
grow once without re-planning. **A capacity table that does not sum to the hardware is not a
capacity plan**, and this one was one arithmetic check away from making a driver look impossible.

## Decision 4 — environment is a variable, never a branch

**The invariant this RFC exists to protect: nothing may encode "this is the development box."**

Acquiring hardware must change inputs, not designs. Concretely, the places this is usually
violated, and the rule for each:

| Divergence | The usual failure | The rule here |
| --- | --- | --- |
| **Storage** | local-path locally, networked storage in production — PVCs behave differently and manifests fork | One CSI driver, both places, even where it is overkill locally. **Decision 7** — Longhorn, replica count as the variable |
| **Secrets** | dotfiles locally, "a real solution later" | Decision 6 |
| **Node count** | one node locally, three in production | A Terraform variable. Decision 3 sets it to production topology now |
| **Ingress and TLS** | self-signed locally, real certificates in production | Decision 5 — the rented edge terminates TLS in both |
| **Hypervisor endpoint** | hardcoded | A provider variable; metal and guest Proxmox expose the same API |

**A configuration difference that cannot be expressed as a variable is a design difference**, and
it must be recorded in this RFC rather than discovered at the transition.

## Decision 5 — git is the only path from the authoring host, and the edge is the only path in

**Outbound.** The macOS host commits and pushes; the substrate pulls, builds and deploys. No file
synchronisation, no remote-mount build, no `scp` of an artifact.

The slower loop is the point. **Every deployed artifact then traces to a signed commit**, which is
what RFC-0013 D3 wants from git history and cannot get if unsigned working-tree code can reach a
cluster. A sync-on-save arrangement would break the conception-evidence chain the IP strategy rests
on, silently, and there is no way to reconstruct it afterwards.

It also makes the authoring host non-infrastructure, which is what lets decision 1 leave it out of
the device registry.

**The signing key is usable on the authoring host and nowhere else.** *Added 2026-08-15 — decision
5 did not say this and it is the gap that would erode it.* A substrate that can reach the signing
key can author signed commits, and the property this decision exists to protect — every deployed
artifact traces to a signed commit from the machine a human works on — dissolves without anyone
choosing to drop it.

The rule is about **use, not storage.** A password manager holding the key and serving it through
an SSH agent is fine, and is what RUN-0001 now recommends; what this forbids is that agent being
reachable from `sitar-as-in-01-ar`. The substrate pulls with a **per-repository read-only deploy
key generated on the box**, which never leaves it and is in no vault. It signs nothing.

**Inbound.** The machine has no static address and needs none. RFC-0016 D1 puts TLS termination and
`Host`-to-region resolution at a rented edge, so the origin dials *out* and holds a tunnel. No
inbound ports, no dynamic-DNS, and per-hostname certificates for tenant custom domains are already
the edge's responsibility.

This is a reason to resolve RFC-0016, which is `proposed`. It is not a claim that it is resolved.

## Decision 6 — the real secrets mechanism is built now, though REQ-0001 has not fired

REQ-0001 is `deferred`, and ADR-0045 risk R3 expires *"at the first deployment."* **This substrate
is not that deployment** — nothing here serves a customer, and R3 stands.

Build the real mechanism anyway.

A development environment with a different secrets story than production fails at precisely the
transition this RFC exists to make seamless. The Zitadel masterkey, database credentials and
service-account keys all exist the moment anything runs; Terraform state contains them; and if
those live in untracked files while a real answer is deferred, then the day metal arrives is the
day the mechanism gets tested, with production as the test case.

**This is design work pulled forward, not a requirement promoted.** REQ-0001 stays `deferred` on
its own trigger.

## Decision 7 — the CSI driver is Longhorn, and the replica count is the variable

*Added 2026-08-19, answering open question 1.* Decision 4 requires **one** driver for both this
substrate and the metal that replaces it, and the question was which one survives both.

**Longhorn, both places.** The scale-down that makes it fit here is `numberOfReplicas` on the
StorageClass — 2 on this substrate, 3 on metal — which is a **variable**, and decision 4 permits a
variable while forbidding a design difference. That is the whole test, and it is the reason this
driver wins rather than the reason it is convenient.

**What Talos needs for it**, and both are system extensions rather than configuration: `iscsi-tools`
for `iscsid`, `iscsiadm` and the `iscsi_tcp` module, and `util-linux-tools` for `nsenter` and
`fstrim`, required from Longhorn v1.5.¹

**Every replicated driver requires a disk separate from the Talos install disk.**¹ This is the one
finding here that changes another task: **T-2b.4's Terraform must give each node a second virtual
disk**, sized as a variable, or no driver in the candidate set works at all. It is cheap to declare
in the module now and a rebuild of every node later.

**Why the per-node figure moved the allocation table.** Longhorn's own guidance is 4 vCPU and 8 GB
per storage node.² The original table gave workers 6 GB. Rather than run below the recommendation
and attribute the resulting behaviour to nested virtualisation, the table was revised — which the
unallocated 16 GB paid for.

**Two costs recorded rather than argued away.**

1. **The substrate runs a replication factor metal will not.** Longhorn's default policy is three
   replicas and wants at least three nodes;² two workers give two. Expressible as a variable, and
   still a difference — a two-replica cluster cannot demonstrate the recovery behaviour a
   three-replica one has. Anything that depends on quorum among replicas is not validated here.
2. **Network-replicated block storage is the worst possible workload for the half of R1 that is not
   retired.** Longhorn replicates synchronously over the cluster network, and that network is inside
   two levels of nested virtualisation. If L2 nesting is too slow for anything, this is what shows
   it first. That is an argument for measuring at T-2b.5 before trusting any latency number this
   substrate produces — see R5.

**Rejected, each with the reason it lost.**

- **Rook-Ceph.** Unambiguously the right answer on metal, and rejected here on the vendor's own
  documentation: Sidero's storage guide states Ceph *"can be rather slow for small clusters"* and
  *"relies heavily on CPUs and massive parallelization for performance"*¹ — the two properties
  nested L2 virtualisation degrades most. With 2 GB per OSD as the floor and 4–8 GB recommended,³ it
  does not fit the guest, and tuning `osd_memory_target` down to make it fit would mean the
  substrate misrepresents metal performance in the optimistic direction, which is worse than not
  having the driver.
- **Piraeus/LINSTOR.** Genuinely lighter, and Sidero calls it *"a solid choice for clusters where
  performance and reliability matter."*¹ Rejected on operational familiarity rather than
  engineering: DRBD is far less commonly run under Talos, so a larger share of the debugging would be
  original work, and this substrate exists to de-risk the chain rather than to add a novel component
  to it. **The closest runner-up** — revisit if Longhorn's memory requirement proves optimistic.
- **local-path-provisioner.** Trivial here and wrong on metal, which is exactly the shape decision 4
  forbids.
- **OpenEBS Mayastor.** Requires huge-page support,¹ and huge pages inside an L2 guest is a new
  class of problem to own for a performance benefit this substrate cannot measure anyway.
- **NFS.** Sidero's guide *"strongly recommend[s] you avoid this method."*¹
- **Deferring the choice.** Legitimate — T-2b.3 records that it blocks nothing until a workload needs
  persistence. Rejected because the second-disk finding above changes T-2b.4, which is the *next*
  task, and discovering it after five nodes exist means rebuilding them.

## Consequences

**Gained**

- RFC-0019 stops being an untested assertion. Proxmox → Talos → Kubernetes is currently a claim in
  a `proposed` artifact that nothing has exercised; this exercises the provisioning chain on a
  machine where being wrong costs a weekend rather than a hardware budget.
- ADR-0020's Kubernetes target becomes reachable, which unblocks every `deployable` class.
- The `post-deploy` tier acquires somewhere to run.

**Accepted costs**

1. **Nested virtualisation performance**, and the risk it does not work at all (R1).
2. **A rolling-release host under a hypervisor** (decision 2 cost 2), licensed here and nowhere
   else.
3. **No availability of any kind.** One machine, one power supply, one network connection, in a
   residence. Everything on it is disposable and must be treated that way — which is a discipline,
   not a property, until something enforces it.
4. **A slower authoring loop than file synchronisation would give**, accepted for RFC-0013 D3.
5. **Time not spent shipping.** This produces nothing customer-facing. It is bounded by decisions
   1–6 rather than open-ended, and that bound is the mitigation.

## Alternatives considered

**Proxmox VE on the metal.** Correct for production and one layer cheaper. Rejected because the
machine is an operator's working environment first, and reformatting it to a hypervisor OS costs
that. Note this alternative becomes the *decision* the moment the machine stops being a
workstation — decision 2 licenses Arch for this host, not for the class.

**No hypervisor — Talos or k3s directly on the metal.** Simplest and fastest, and it would work.
Rejected on two counts: it contradicts RFC-0019 D1, which would then need amending rather than
validating; and the provisioning chain it skips — Terraform against a hypervisor API — is
precisely the part nothing has tested and the part most likely to be wrong.

**A managed Kubernetes cluster from a cloud provider.** Faster than all of this and genuinely
tempting. Rejected because it validates nothing about RFC-0019, and because it introduces a
provider dependency that REQ-0012 would require be *"approved, scoped, and exitable"* before a
substrate that is meant to be thrown away.

**Two environments, `dev` and `staging`.** Rejected in decision 3: no tier in `test-registry.yaml`
serves the second one, and the memory is better spent on production topology in the first.

**File synchronisation between the hosts instead of git.** Faster inner loop. Rejected in
decision 5: it lets unsigned code reach a cluster, and RFC-0013 D3's evidence cannot be
reconstructed after the fact.

## Open questions

1. ~~**Which CSI driver, and does it run on both a laptop-scale cluster and real hardware?**~~
   **Answered 2026-08-19 by decision 7 — Longhorn**, with `numberOfReplicas` as the variable that
   satisfies decision 4. The answer moved the decision 3 allocation table and added a second virtual
   disk to T-2b.4. It is answered rather than *verified*: nothing has run it, and cost 2 of decision
   7 is the reason to expect the verification to be the interesting part.
2. ~~**Where does Terraform state live, and how is it encrypted?**~~ **Answered 2026-08-19 by
   ADR-0052 decision 3.** OpenTofu 1.7's native client-side state encryption, passphrase key
   provider, the passphrase held by the same mechanism decision 6 builds. State is encrypted before
   it reaches any backend, so local state on a single disk stops being the gap ADR-0045 R3 names —
   and no vendor is added to achieve it. **The passphrase itself is now the single point of failure**
   and decision 6 owns it; losing it loses the ability to manage what the state describes.
3. **Does the Proxmox guest need a device identity of its own?** It is a hypervisor running as a
   guest, which `device-registry.yaml` does not contemplate — its `platform` field describes what
   runs on *hardware*. This RFC treats Proxmox as a workload on `sitar-as-in-01-ar` and assigns it
   no name. RFC-0021 D9 assigns Talos node naming to the cluster tooling, which is consistent.
4. **What retires this machine?** Decision 3 says metal becomes `prod` and this stays `staging`.
   That is an assertion about a future purchase and nothing schedules it.
5. **Does anything enforce decision 5's boundary?** A kubeconfig copied to the authoring host would
   break it silently. Candidate checks are named in the risks below; none is declared yet, and
   declaring them is an ADR-0014 change this RFC does not make.

## Risks

- ~~**R1 — nested KVM does not work on this CPU.**~~ **Retired 2026-08-15: nested KVM is enabled
  and confirmed on `sitar-as-in-01-ar`.** Decisions 2 and 3 stand. **The performance half of this
  risk is not retired** — L2 nesting is confirmed *available*, not confirmed *fast enough*, and
  that is only measurable once Talos nodes actually run. If it proves unusable the fallback is
  unchanged: Talos directly under QEMU, losing the RFC-0019 validation but keeping everything
  above the hypervisor line.
- **R2 — decision 4's invariant erodes one exception at a time.** Nobody decides to fork the
  configuration; someone adds one conditional under deadline. This is the failure this RFC is most
  likely to suffer and open question 5 is the only proposal against it.
- **R3 — the substrate becomes load-bearing.** Something real ends up depending on a machine with
  no redundancy in a residence, because it was there and it worked. Consequence 3 states the
  property; nothing enforces it.
- **R4 — the transition never happens.** Metal is a purchase, and a working development
  environment reduces the urgency of the thing it was built to prepare for. Open question 4.
- **R5 — a latency number measured here is believed.** *Added 2026-08-19 with decision 7.* Longhorn
  replicates synchronously across a cluster network that sits under two levels of nested
  virtualisation, so storage-dependent measurements on this substrate are pessimistic by an unknown
  factor. The danger is not the slowness, it is quoting it: T-0.4's D2 baseline benchmark is owed and
  expiring, and a baseline captured here would be attributed to the naive implementation rather than
  to L2 nesting. **Storage-path baselines belong on metal, or with the substrate named in the
  measurement.**

## Sources

1. Sidero Labs, Talos Linux storage guide — the CSI option set, the per-driver system extensions
   (`iscsi-tools`, `util-linux-tools`, DRBD, huge pages), the requirement that replicated drivers use
   a disk separate from the Talos install disk, and the quoted assessments of Ceph, Piraeus/LINSTOR
   and NFS. Retrieved 2026-08-19.
2. Longhorn node sizing — 4 vCPU and 8 GB RAM recommended per storage node; default replica count 3,
   wanting at least three nodes. Retrieved 2026-08-19.
3. Ceph hardware recommendations — 2 GB per OSD minimum, 4–8 GB recommended, `osd_memory_target`
   defaulting to 4 GB for BlueStore. Retrieved 2026-08-19.

## Related

- ADR-0052 — OpenTofu, which answers open question 2 and corrects T-2b.4's tool
- GLO-0011 — `environment`, which reconciles decision 3's cluster name with locked ADR-0021
- ADR-0020 — the locked Kubernetes target this substrate exists to reach
- RFC-0019 — Proxmox, Talos and the cluster shape this validates rather than restates
- RFC-0016 — the rented edge, which is why decision 5 needs no static address
- RFC-0021 · `device-registry.yaml` — the hostname grammar and the `ar` admission
- RFC-0013 — D3's signed-commit evidence, which decision 5 protects
- REQ-0001 · ADR-0045 — the secrets gap decision 6 designs for without promoting
- REQ-0012 — vendor governance, which the rejected managed-cluster alternative would engage
- STD-0008 · `test-registry.yaml` — the tiers that decided one environment is enough
