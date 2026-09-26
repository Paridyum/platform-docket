---
id: STD-0018
title: A mesh object takes the name of the thing it refers to, and never invents one
type: standard
status: accepted
created: 2026-09-01
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra]
applies_to: ["*"]
tags: [naming, netbird, mesh, vpn, networking, rfc-0037, rfc-0021, c-0007]
breaking: false
enforced_by: []
---

# STD-0018 — A mesh object takes the name of the thing it refers to

## Status

**Accepted**, 2026-09-01.

## Why this exists

The overlay network holds objects that nothing names: networks, resources, routing peers,
access policies, groups and setup keys. C-0007 binds repositories, crates, HTTP APIs, databases,
events, environment variables, container images, Kubernetes objects and environments — and stops
there. RFC-0021 names devices. Neither reaches the mesh.

The gap is not cosmetic. A mesh object grants **reachability**: a network resource decides which
addresses a peer can send packets to, and a policy decides which peers. Names that are invented
at a console drift from the estate they describe within a week, and the first symptom is a
policy nobody dares delete because nobody can say what it grants.

## The rule

> **A mesh object is named after the thing it refers to, using the name that thing already has
> in the registry that governs it. A mesh object never introduces a new name.**

Everything below is that rule applied. If a case is not listed, apply the rule rather than
inventing a convention for it.

### Form

Lowercase, hyphen-separated, ASCII letters and digits. No spaces, no capitals, no underscores.

Capitals are excluded because the console and the API treat names as case-sensitive strings
while humans do not: `IN-BIDHOLI-01` and `in-bidholi-01` are two objects and one intention, and
the estate finds out when a policy silently matches neither.

### Where each name comes from

| Object | Name | Source |
| --- | --- | --- |
| Network | the region it carries, lowercased | `address-registry.yaml` → `regions[].name` |
| Resource | the region's name plus what the resource is | the same entry, plus `-subnet` or `-host` |
| Routing peer | the device's hostname | RFC-0021 · `device-registry.yaml` |
| Group | the device family, plural | `device-registry.yaml` → `families[].code` |
| Policy | `<group>-to-<resource>` | the two objects it joins |
| Setup key | the device family it enrols | `device-registry.yaml` → `families[].code` |

Worked through for the first region, so the mapping is not left as a description:

```
network        in-bidholi-01              ← region IN-BIDHOLI-01
resource       in-bidholi-01-subnet       ← its 10.1.1.0/24
routing peer   sitar-as-in-01-ar          ← the device, named by RFC-0021
group          tablas                     ← the operator-workstation family
policy         tablas-to-in-bidholi-01-subnet
setup key      tabla                      ← enrols machines of that family
```

### Why derived rather than descriptive

A descriptive name — `office-lan`, `k8s-net`, `admin-access` — reads better on the day it is
written and describes an intention rather than a thing. When the region is renumbered, the
cluster moves, or the device is rebuilt, the descriptive name still reads plausibly and is now
wrong, and nothing detects that.

A derived name is wrong *visibly*: `in-bidholi-01-subnet` pointing at a subnet that
`address-registry.yaml` no longer allocates is a contradiction anybody can check, and a check
could be written for it.

This is the same reasoning ADR-0035 applies to counts and RFC-0037 D3 applies to continents:
**where a fact already lives somewhere, refer to it rather than restating it.**

### Retirement

**A mesh object is deleted when the thing it names is retired, and its name is not reused.**
RFC-0021's rule for hostnames and RFC-0037 D7's for addresses apply unchanged and for the same
reason: logs, policies and audit records from that period refer to the object by that name, and
reusing it makes the record lie about who could reach what.

## What this does not decide

**Who may reach what.** This standard governs names, not access. A policy named
`tablas-to-in-bidholi-01-subnet` says what it joins and says nothing about whether that join
should exist — that is RFC-0037 open question 2, which is open precisely because advertising a
subnet widens what every peer can reach.

**The mesh's own address space.** NetBird allocates from `100.64.0.0/16` and RFC-0037 D3
reserves `X = 208–209` for an overlay. Whether the mesh renumbers is RFC-0037 open question 3.

## Alternatives considered

- **Free-form names with a review step.** Cheapest to adopt and it fails in the direction that
  matters: review catches a bad name once, and nothing catches the name that was right when
  written and became wrong afterwards.
- **Encoding the CIDR in the name** — `net-10-1-1-0-24`. Unambiguous, and it duplicates the
  allocation into a second place that must be renumbered in step with the first. RFC-0037 D7
  exists to prevent exactly that duplication.
- **Extending C-0007 instead.** The right long-term home, and it is `locked` — reopening a
  locked constitutional artifact to add a section is a supersession, which is a heavier
  instrument than a first attempt at a convention deserves. A standard can be superseded
  cheaply if this proves wrong, and folded into C-0007 once it has survived contact.

## Consequences

**Accepted costs.**

- **Names are ugly.** `tablas-to-in-bidholi-01-subnet` is not a name anyone would choose for
  readability, and every one of them must be looked up rather than recalled.
- **A rename cascades.** Renaming a region in `address-registry.yaml` means renaming a network,
  a resource and every policy referring to it. That is the cost of derived names, and it is
  paid at the moment the registry changes rather than discovered later.
- **Nothing enforces it.** `enforced_by` is empty. The mesh is configured through a console
  with no artifact in any repository, so there is nothing to run a check against — which is
  itself worth recording, because an unenforced naming rule in a system nobody can diff is the
  weakest form this could take.

**Gained.**

- A mesh object can be traced to the registry entry that justifies it, so "what does this
  policy grant" is answerable without asking the person who created it.
- A stale object is visibly stale rather than merely old.
