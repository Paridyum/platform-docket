---
id: GLO-0009
title: Fleet manager — what provisions metal, and deliberately not called a control plane
type: glossary
status: proposed
created: 2026-08-18
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [vocabulary, fleet, provisioning, hosting, control-plane, rfc-0025, glo-0007]
breaking: false
enforced_by: []
---

# GLO-0009 — Fleet manager

> **Superseded in part, 2026-09-28 — ADR-0113.** This entry quotes the name GLO-0007 retired as the lesson it
> teaches. The term stays retired (ADR-0113 D5); nothing here revives it.

## Definition

**The fleet manager is the system that provisions and manages fleets of physical and virtual
machines. It creates Proxmox virtual machines, applies Talos configuration, enforces quota, meters
usage, and holds hosting-customer records. No workload runs on it.**

A **fleet** is one Proxmox cluster carrying workloads. RFC-0025 names two: `fleet-shared` for
hosting customers and `fleet-platform` for Paridyum's own platform.

## What it is not

**It is not the control plane.** That term is taken, and the collision is the entire reason this
entry exists — see below.

**It is not a fleet.** The fleet manager provisions fleets and runs no customer workload itself.
RFC-0025 decision 3's whole justification for seeding it on a residential machine depends on this
separation holding: a fleet-manager outage stops provisioning without stopping anything that is
already running.

**It is not privileged.** RFC-0025 decision 2 makes Paridyum a customer of it on the same terms as
anyone else — same API, same quotas, same metering. C-0008 in the infrastructure layer.

## Why this term exists at all

**`control plane` already means something else here, and RFC-0025 was drafted using it for this
before anyone checked.**

The existing sense is GLO-0007's: the tier holding the platform's global facts and the `paridyum`
Zitadel instance, resident in exactly one control region. RFC-0018 is titled *"the control plane is
one repository"* and means the four `platform-*` contexts sharing that plane.

The new sense would have been a machine-provisioning system with a fleet of hypervisors under it.

**Both readings are natural, and that is exactly what makes a word unusable.** GLO-0007 wrote that
sentence about `home region`, which meant two things for a week before anyone noticed, and produced
a design statement that was *"obviously true under one reading and a redesign of the control plane
under the other."* The same trap, one week later, on the word GLO-0007 itself introduced.

So: **control plane** for the platform's global-facts tier, **fleet manager** for what provisions
metal, and neither term for the other.

## Names rejected, so they are not re-proposed

| Name | Why not |
| --- | --- |
| `hosting control plane` | A qualifier on an ambiguous word. `home region` would have survived as *"platform home region"* too, and GLO-0007 rejected that fix for the same reason: qualifiers get dropped in speech, and the ambiguity returns |
| `provisioner` | Understates it. Provisioning is one of its jobs; quota, metering and customer records are not provisioning |
| `substrate control plane` | Still contains the collision |
| `infrastructure control plane` | Same |

## A third sense, which is fine and must not be renamed

**`control plane` also means the Kubernetes control plane** — the API server, scheduler and etcd on
a cluster's control-plane nodes. RFC-0023 allocates three of them; RFC-0019 decision 2 sizes them.

**Leave that alone.** It is universal industry vocabulary, it appears in Talos's own documentation
and in `kubectl` output, and renaming it locally would make this repository's prose disagree with
every tool an operator runs. The disambiguation that matters is between *Paridyum's* control plane
(GLO-0007) and the fleet manager; a Kubernetes control-plane node is a third thing that nobody
confuses with either, because it is always qualified by the cluster it belongs to.

The test when writing: if the sentence would still parse with "the API server and etcd" substituted
in, it is the Kubernetes sense and needs no change.

## The related trap

**Do not invent a second name for a fleet.** A fleet is a Proxmox cluster carrying workloads, which
RFC-0019 already calls a *datacentre* when it is the unit of failure domain (GLO-0008). Those are
the same object seen from two angles — `fleet` names what it carries, `datacentre` names what it
fails as — and a third synonym would recreate exactly the problem this entry solves.

## Related

- **GLO-0007** — control region, and the `home region` collision this entry repeats the lesson of
- **GLO-0008** — datacentre, the same object a fleet is, named for its failure semantics
- **RFC-0025** — where the fleet manager is designed, and what was renamed on 2026-08-18
- **RFC-0018** — "the control plane is one repository", which means the *other* control plane
- **C-0008** · **ADR-0050** — tenant zero and product zero; the fleet manager makes it customer zero
