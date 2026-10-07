---
id: RFC-0037
title: An address encodes country, routing, and nothing a machine can outgrow
type: rfc
status: accepted
created: 2026-08-31
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [addressing, ipam, networking, kubernetes, talos, cilium, nat, wireguard, netbird, residency, rfc-0021, rfc-0014, rfc-0019, rfc-0023, rfc-0035]
breaking: false
enforced_by: []
---

# RFC-0037 — An address encodes country, routing, and nothing a machine can outgrow

## Status

**Accepted**, 2026-08-31. This discharges **RFC-0021 decision 9**, which deferred address
allocation to "a separate addressing RFC" and has been open since RFC-0021 was written.

## Context

### Nothing was allocated, and the archive says so on purpose

Before this RFC, `paridyum/knowledge` contained **no IP address of any kind**. Searched with a
literal regex for every RFC 1918 pattern, the estate's entire governance returned zero matches.
That was not an oversight. The allocation is recorded as owed and deliberately unwritten,
in each of these:

- **RFC-0021 decision 9** defers it, with a warning quoted in decision 8 below.
- **`device-registry.yaml`** — `NO ADDRESS RANGES`, and separately `NO SEQUENCE ALLOCATIONS`,
  because "a governance file declares intent, and observed state is measured elsewhere and never
  written back."
- **`region-registry.yaml`** catalogues the archive's IPAM reservation as "a different axis
  entirely."
- **RFC-0014** — "IP address allocation is a different axis."

### Which meant every address in use was chosen by software

`192.168.122.0/24` is libvirt's compiled-in default. `100.64.0.0/16` is NetBird's default
allocation out of the CGNAT space. `192.168.1.0/24` is a consumer router's factory setting.
Paridyum had claimed none of them, and the first substrate guest could not be reached because
nothing decided what its address should be.

### The pressures that actually conflict

**Geography in an address is a liability, and also an audit control.** RFC-0021 decision 9's own
argument is that "names and addresses must be decoupled" — encode a fact in an identifier and
moving the machine forces renumbering. Against that: **RFC-0014 makes a region a residency
boundary**, and residency is the one property where an address crossing a line is a compliance
event rather than an inconvenience. If country is in the address, a residency violation is
visible in a routing table. That is a control nothing else in the estate provides.

**Uniqueness costs space; reuse costs reachability.** RFC-0019 chose Talos with Cilium, and a
Kubernetes cluster may span hypervisors. Cluster members must reach each other at their real
addresses — etcd peers, kubelet, and the CNI do not traverse a proxy, and a translated address
makes a node unable to recognise itself in the cluster's own membership. Meanwhile RFC-0035 D1
denies guest→host by default and RFC-0023 D5 needs no inbound reachability at all, both of which
argue for NAT everywhere.

**`172.16.0.0/12` cannot be a global namespace.** It contains sixteen `/16` blocks. Any scheme
that requires those to be unique across the estate caps the estate at sixteen of whatever owns
one.

## Decisions

### Decision 1 — `10.0.0.0/8` is the routed estate; everything routed comes from it

An address is **routed** if any machine outside its own host can reach it as itself: cluster
members, hypervisors, routers, storage, anything advertised into the mesh.

**Every routed address is allocated from `10.0.0.0/8` and is globally unique across Paridyum.**
No exceptions, and the rule is deliberately phrased so that the test — "can something off-box
reach this by this address?" — is answerable without knowing the topology.

The corollary is the useful half: **an address outside `10.0.0.0/8` is, by construction, not
routed.**

**This plan governs addresses Paridyum ASSIGNS, not addresses assigned TO Paridyum.** A
machine's uplink — the address its ISP, its office LAN, or a rented provider's fabric hands it —
is **out of scope**. It is not allocated here, not recorded in the registry, and not required to
be in `10.0.0.0/8`, even though other machines on that foreign network can obviously reach it.

The test in the first paragraph is therefore narrower than it first reads: routed means
reachable **from elsewhere in the Paridyum estate**, not merely reachable from something
off-box. Without that narrowing the rule would demand renumbering every network a Paridyum
machine is plugged into, including ones we do not own and cannot change — which is both
impossible and pointless, since nothing in the estate addresses a machine by its uplink.

Two things follow, and both are load-bearing:

- **An uplink MAY use DHCP** and usually should. Decision 9's prohibition binds routed
  addresses; an uplink is not one.
- **An uplink MUST NOT be depended upon.** It is not stable, not recorded, and not reachable
  from the estate. A machine that must be reachable holds a routed address of its own, on an
  interface it controls, in addition to whatever its uplink happens to be. A reader seeing `192.168.4.7` in a log knows it is host-local without checking.

### Decision 2 — the octets mean country, region, host, in that order

```
10 . X . Y . Z
│    │   │   └── host within the region
│    │   └────── region (a numbered subdivision within a country)
│    └────────── country
└─────────────── the routed estate
```

`X` is allocated from the fixed table in decision 3. `10.X.Y.0/24` is a region. `Z` identifies a
host within it, with `.1` reserved for the region's gateway and `.0`/`.255` unusable as ever.

**A cluster may span hypervisors. It may not span countries.** RFC-0014 makes a region a
residency boundary; a cluster straddling `X` values would place one tenant's data on both sides
of a legal line. This is the constraint that lets geography stay in the address without the
usual cost — a machine moving between regions is rare, and a machine moving between countries is
already a residency event with paperwork attached.

### Decision 3 — the country table is fixed, and is namespace rather than inventory

| Continent | Country | X range | Blocks |
| --- | --- | ---: | ---: |
| Asia | India | 1–29 | 29 |
| | UAE | 30–37 | 8 |
| | Singapore | 38–45 | 8 |
| | Japan | 46–53 | 8 |
| | South Korea | 54–61 | 8 |
| | Indonesia | 62–69 | 8 |
| | Taiwan | 70–77 | 8 |
| Africa | South Africa | 78–97 | 20 |
| Europe | Finland | 98–107 | 10 |
| | Sweden | 108–117 | 10 |
| | Poland | 118–127 | 10 |
| | Spain | 128–137 | 10 |
| North America | Canada | 138–152 | 15 |
| | United States | 153–167 | 15 |
| South America | Brazil | 168–177 | 10 |
| | Chile | 178–187 | 10 |
| Australia | Australia | 188–207 | 20 |
| — | **Unusable** | 0 | 1 |
| — | Overlay (mesh) | 208–209 | 2 |
| — | Edge routing | 210–214 | 5 |
| — | Reserved | 215–255 | 41 |

**Continent is derived from country, and `region-registry.yaml` is the authority.** The continent
column above is reproduced as a reading aid so the table can be scanned; it is **not** a second
source. Where the two disagree, the registry wins (ADR-0035), and the column is the thing that is
wrong. Continent labels here therefore match the registry's spelling exactly rather than a
tidier one.

**This table is a namespace declaration and contains no allocations.** Which `X` a country's
first block takes, which `Y` values exist, and which `Z` is which machine are **inventory**, and
inventory is recorded in the registry named in decision 7 — not here. This is
`device-registry.yaml`'s rule applied to addresses, and it is why no region is named anywhere in
this artifact.

**Zero is never allocated, at any level.** `X = 0` and `Y = 0` are unusable and no allocation
may take them; numbering starts at 1.

For `X` the reason is concrete rather than aesthetic: `X = 0` is `10.0.0.0/16`, the single most
common default in cloud VPCs, hypervisors and consumer appliances. It is the block most likely
to collide the first time Paridyum rents metal, and a collision there is not a clash between two
Paridyum sites — it is a clash with a provider's own network, which we cannot renumber. Removing
it from the namespace costs one block out of India's range and removes the failure entirely.

For `Y` the reason is legibility: `10.X.0.0/16` is a country block and `10.X.0.0/24` would be
its first region, two different things whose notation differs only by a mask a reader may not be
looking at. Reserving `Y = 0` means a region's address can never be mistaken for its country's.

The rule is stated for both together rather than as two exceptions, because a namespace with one
arbitrary hole is a thing people forget and a namespace where zero is simply not a value is a
thing they do not.

**Resolving a conflict between the two source drafts.** One listed `208–209` as a global
WireGuard network and `210–214` as edge routing, with `215–255` reserved; the other listed
`208–255` as reserved in a single row. The table above takes the first, because a reserved range
that quietly absorbs two named purposes loses the record that they were named.

### Decision 4 — a country's first block is the backbone; its remaining blocks are allocated whole

Within a country's `X` range:

- **The first `/16`** is the **backbone**, structured exactly as decision 2 describes: `Y` is a
  region `/24`, `Z` is a host. Hypervisors, routers, storage, physical kit.
- **Every remaining `/16`** is allocated **whole** to a single large consumer — a Kubernetes
  cluster, or a VM fabric that will not fit in a `/24`.

India therefore holds one backbone block and twenty-eight spare; UAE one and seven.

The reason for allocating whole blocks rather than subdividing the backbone is that the two have
different lifetimes. A backbone address outlives the machine holding it; a cluster's internal
space dies with the cluster. Mixing them means retiring a cluster leaves holes in the region
that someone must remember are reusable.

### Decision 5 — a cluster is one `/16`, and Talos is pinned to it

**A Kubernetes cluster is allocated one `/16` from its country's spare blocks**, subdivided:

| Purpose | Slice of the cluster's `10.X.0.0/16` | Size |
| --- | --- | --- |
| Pod CIDR | `10.X.0.0/17` | 32 768 addresses — 128 nodes at the `/24`-per-node default |
| Service CIDR | `10.X.128.0/18` | 16 384 services |
| Load-balancer VIPs | `10.X.192.0/18` | 16 384 — Cilium LB-IPAM, **usable only with BGP**; see below |

**A LOAD-BALANCER VIP MUST BE ANNOUNCED BEFORE IT IS AN ADDRESS**, and how it is announced
decides where it may come from. This was written before a cluster existed and the reservation
above is right for only one of the two mechanisms.

- **L2 announcement** — the node answers ARP for the VIP. This works only if the VIP is inside
  the **node's own L2 subnet**, because ARP does not cross a router. A VIP from the cluster's
  `/18` would be announced to a segment where nothing is looking for it, and the failure is a
  Service that has an address and no traffic.
- **BGP** — the node advertises the VIP to a router that then routes it. Any address works,
  including the `/18` above, and it needs BGP-capable infrastructure that this estate does not
  have.

**So a cluster announcing over L2 allocates its load-balancer VIPs from its REGION's `/24`, and
records them in `address-registry.yaml` like any other address in that region.** The `/18`
remains reserved for the same cluster under BGP, and is not released — a cluster that gains BGP
should not have to renumber to use it.

This is the difference between an address plan and a routed address: the `/18` was allocated by
arithmetic that was internally consistent and had nowhere to be announced.

**Talos MUST be configured with these values explicitly. Its defaults MUST NOT be used**, and
this is the decision with the sharpest failure mode in the RFC.

Talos defaults to a pod CIDR of `10.244.0.0/16` and a service CIDR of `10.96.0.0/12`. Both fall
inside `10.0.0.0/8` — the routed estate. `10.96.0.0/12` spans `10.96.0.0`–`10.111.255.255`,
which is Finland's range (98–107) and part of Sweden's (108–117). `10.244.0.0/16` sits in
Reserved.

The consequence is not a startup error. A pod attempting to reach a hypervisor in Finland at
`10.98.1.5` matches the service CIDR first and is routed to a **Service VIP that does not
exist**. Nothing fails loudly; traffic goes somewhere else. Renumbering a live cluster's service
CIDR requires rebuilding the cluster, so the cost of finding this later is the cluster.

`100.64.0.0/10` was considered as a home for pod and service space, on the reasoning that
keeping cluster internals out of `10.0.0.0/8` entirely is cleaner. **Rejected**: it is CGNAT
space, Indian ISPs allocate from it routinely, and a node whose WAN address collides with the
pod CIDR fails in a way indistinguishable from a CNI bug.

### Decision 6 — everything not routed is local, reusable, and never advertised

| Range | Role | Uniqueness |
| --- | --- | --- |
| `10.0.0.0/8` | backbone, cluster nodes, pod/service/LB — anything routed | **globally unique** |
| `172.16.0.0/12` | LANs behind a router | reusable on every router |
| `192.168.0.0`–`192.168.127.0/24` | container and VM bridges | reusable on every host |
| `192.168.128.0`–`192.168.255.0/24` | NAT bridges | reusable on every host |

**A host MUST NOT use a local range that collides with its own uplink.** The reusable ranges
above are reusable *across* hosts, not *within* one. A machine whose upstream network is
`192.168.1.0/24` — the factory default of most consumer routers, and the current uplink of the
first substrate host — cannot also number a bridge in `192.168.1.0/24`, because the collision is
on that host's own routing table and no registry can see it. This is the one uniqueness rule
that must be checked locally rather than centrally.

**A subnet advertised into the mesh MUST be globally unique. A subnet never advertised MAY
overlap freely.** Because decision 1 puts every routed address in `10.0.0.0/8`, this rule needs
no separate discipline to uphold: the ranges that may overlap are exactly the ranges that cannot
be advertised.

**NAT is the default and it is a security control, not a convenience.** RFC-0035 D1 denies
guest→host; RFC-0023 D5 and RFC-0016 D1 mean the origin dials out and nothing needs to reach in.
Under NAT a machine is unreachable unless something explicitly forwards to it, so isolation is
the resting state rather than a rule an operator must remember.

**A bastion is an access pattern and is not a substitute for routing.** A jump host serves
humans and tooling — SSH, `kubectl`, a console — and composes correctly with NAT. It cannot
carry a cluster data plane, because etcd and the CNI do not open sessions through a proxy. Where
machines must reach each other, the answer is decision 1, not a bastion.

**The router that fronts a site is a routing peer, not a translator, for what it advertises.** It
joins the mesh and advertises its site's routed subnets; hosts behind it need no mesh client.
This is why `172.16.0.0/12`'s sixteen blocks are not a constraint on the estate: nothing
requiring uniqueness lives there.

### Decision 7 — allocations live in a registry, and this artifact holds none

`address-registry.yaml` records every allocation: which `X` a country's block occupies, which
`Y` values exist, which cluster owns which `/16`, and what has been retired. Each entry carries
the country, the CIDR, a human-readable name, an owner, a purpose, a status
(`active`/`reserved`/`retired`), and audit dates.

**A retired allocation is never deleted and its numbers are never reused.** RFC-0021's rule for
hostnames applies unchanged and for the same reason: the address is the handle by which logs,
firewall rules, and backups from that period refer to the machine. Reusing it makes the archive
lie.

This RFC declares the namespace. The registry records what is in it. The separation is
`device-registry.yaml`'s, quoted in the context above, and it is the reason no region is named
in this document.

### Decision 8 — IPv4 is allocated now; IPv6 gets a rule and no addresses

RFC-0021 decision 9 left a warning for whoever wrote this RFC:

> RFC 4193 requires ULA global IDs be **pseudo-random**, so a sequential country allocation does
> not survive contact with IPv6.

It is correct, and it forecloses the obvious move of mirroring decision 3's table into `fd00::/8`.
Rather than invent a scheme that RFC 4193 forbids, this RFC decides the **rule** and defers the
**addresses**:

**When IPv6 is allocated, the ULA global ID MUST be generated pseudo-randomly per RFC 4193
§3.2.2 and recorded in the registry. It MUST NOT be derived from country, region, or any other
attribute of the thing addressed.** Structure may live in the subnet ID; it may not live in the
global ID.

This is left as an open question rather than claimed as resolved, because a rule with no
allocation is not a plan.

### Decision 9 — routed addresses are static; only local ranges may use DHCP

**A routed address is configured statically on the machine that holds it, from the allocation
recorded in the registry. A machine MUST NOT obtain a routed address by DHCP.**

**A local range MAY use DHCP**, because a lease that never leaves the host it was issued on
crosses no boundary and reproduces no state anyone else depends on.

This is not a preference for static addressing in the abstract. DHCP requires the client to
reach a server, and on this estate that server sits on the host while the client sits in a guest
— exactly the direction **RFC-0035 D1 denies**. The first substrate guest built under this RFC
demonstrated it: the guest broadcast `DHCPDISCOVER`, the host's `input` chain dropped it under
`policy drop`, and the installer fell back to a vendor default address in a subnet nothing routes
to. The guest was on the correct bridge and unreachable.

There were two ways out and only one of them is consistent. Opening `udp/67` and `udp/53` from
the guest bridge would make DHCP work and would put the **first hole in RFC-0035 D1**, exposing
the host's resolver to every guest, permanently, to save writing an address down once. Assigning
the address statically needs no exception at all: the guest never asks the host for anything, the
boundary stays absolute, and the address is deterministic — which is what an infrastructure host
should be regardless.

The general form is worth stating because it will recur: **where a boundary and a convenience
conflict, move the requirement rather than the boundary.**

### Decision 10 — this RFC allocates private addresses only, and does not disturb RFC-0023 D5

RFC-0023 decision 5 states that the substrate "has no static address and needs none", and a
reader arriving from that sentence to decision 9 above will think the two disagree. They do not,
and the distinction is worth writing down before someone resolves it the wrong way.

RFC-0023 D5 is about **public reachability**: RFC-0016 D1 puts TLS termination at a rented edge,
the origin dials *out* and holds a tunnel, and so the machine needs no address anyone on the
internet can find — no static WAN address, no dynamic DNS, no inbound ports.

This RFC allocates **private addresses inside the estate**. Nothing here gives any machine a
publicly routable address, and nothing here opens an inbound path. A substrate host has a
deterministic address that Paridyum can reach over the mesh, and remains invisible from outside
exactly as RFC-0023 D5 requires.

**No public address space is allocated by this RFC, and none may be, without a decision that
supersedes RFC-0023 D5.**

## Alternatives considered

- **Carve Kubernetes globally from Reserved (215–255).** Simplest to hold in mind, and it
  discards the residency property — a Finnish cluster and an Indian cluster become
  indistinguishable by address — while leaving country ranges idle and a single global block
  filling up.
- **Pod and service CIDRs in `100.64.0.0/10`.** Keeps `10.0.0.0/8` pure. Rejected on CGNAT
  collision, decision 5.
- **Globally unique `172.16.0.0/12` LANs.** Sixteen `/16` blocks is a hard cap on the estate.
- **No geography at all — flat allocation from a registry.** Honest about RFC-0021 D9's
  "decouple names and addresses", and it gives up the one audit control that motivated keeping
  geography. Reconsider if renumbering cost ever exceeds the audit value.
- **A multi-tenant model in this artifact.** The third source draft describes tenants, routers
  owning `172.16.0.0/12`, per-tenant Docker networks, and *permits overlap between tenants*.
  That is correct for a product modelling other people's networks and wrong for an estate whose
  entire purpose here is uniqueness. It belongs in a product specification, not in this RFC.

## Consequences

**Accepted costs.**

- **Geography is in the address, so a machine changing country is renumbered.** This is
  deliberate — decision 2's residency argument — but it is a real cost and it will be paid.
- **A region is a `/24`: 253 usable hosts.** Sufficient for hypervisors and physical kit, and
  it will not be sufficient forever at a large site. The escape is decision 4's whole-block
  allocation, which is a different shape from simply widening the region, and someone will
  find that annoying.
- **The country table is fixed and 41 blocks remain reserved.** Adding a country later means
  allocating from Reserved, and countries added that way will not be contiguous with their
  continent's neighbours. The alternative — leaving the table open — was rejected because a
  namespace that can be extended ad hoc cannot be reasoned about.
- **Talos must be configured against its defaults on every cluster.** A cluster built by
  someone following upstream documentation will be wrong, and wrong in the silent way
  decision 5 describes.
- **Zero is spent to buy safety.** Reserving `X = 0` and `Y = 0` costs India a block and every
  country a region, for a hazard that would otherwise appear only once, at the worst moment. It
  is a deliberate trade and not free.

**Gained.**

- Residency is visible in a routing table.
- The routed/local distinction is legible from the address alone, with no topology knowledge.
- The overlap rule needs no discipline: what may overlap is exactly what cannot be advertised.
- The Kubernetes collision is found now, when it costs a configuration line, rather than after
  a cluster carries data, when it costs the cluster.

## Open questions

1. **IPv6 allocation.** Decision 8 sets the rule and allocates nothing. Who generates the ULA
   global ID, and when?
   **Answered 2026-10-04** by ADR-0132 (CI runners reach the internet dual-stack): the global ID was
   generated then, by the rule above, and is recorded in `address-registry.yaml` (`ipv6`). The first
   addresses are the region gateway's and `ci-runner-01`'s.
2. **The routing peer does not exist yet, and everything is reaching the estate through a
   bastion instead.** Decision 6 describes a router that joins the mesh and advertises its
   site's routed subnets, so hosts behind it need no mesh client. Nothing advertises
   `10.1.1.0/24` today: a workstation cannot reach the hypervisor, the Proxmox API, or any
   cluster node, and every tool that needs to has been pointed through an SSH forward on the
   substrate host.

   That is *sanctioned* by decision 6 — "a bastion is an access pattern... for humans and
   tooling" — and it is not equivalent. A bastion forwards a port per tool per invocation;
   advertising the subnet makes the estate addressable, which is what decision 1 is for. The
   present arrangement works and quietly teaches everyone that reaching the estate requires a
   tunnel, which is a habit that survives the fix.

   It is an open question rather than debt because the answer is not obviously "add the route":
   advertising a subnet from a peer widens what every other peer can reach, and who may reach
   the cluster network has not been decided.

3. **Does the mesh renumber?** NetBird currently runs on `100.64.0.0/16`; decision 3 reserves
   `X = 208–209` for the overlay. Moving is disruptive and staying leaves a routed range
   outside `10.0.0.0/8`, which decision 1 says should not happen.
4. **Resolution.** An address plan without a naming-to-address story is half a plan. RFC-0021
   settles what a machine is *called*; nothing settles how that name resolves to an address
   Paridyum controls. Internal DNS is undecided and is a separate artifact.
5. **MTU under the overlay.** WireGuard encapsulation reduces the usable MTU, and Cilium
   computes its own from the interface it finds. A cluster whose nodes reach each other across
   the overlay will suffer path-MTU failures that present as intermittent large-response
   timeouts — the failure mode hardest to attribute. Sizing is undecided.
6. **Enforcement.** `enforced_by` is empty. A check that an allocation exists in
   `address-registry.yaml` before an address appears in `infra-*` is writable and does not
   exist; so is a check that Talos configuration never carries the upstream default CIDRs. The
   second is the one that matters, per decision 5.
