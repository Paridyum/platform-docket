---
id: GLO-0008
title: Datacentre — a failure domain inside a region, never a residency boundary
type: glossary
status: proposed
created: 2026-08-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [region, datacentre, availability, residency, vocabulary, rfc-0014, rfc-0019]
breaking: false
enforced_by: []
---

# GLO-0008 — Datacentre

## Definition

**A datacentre is a failure domain inside a region. A region may contain one or more; the count is a
property of that region, not a platform-wide policy.**

Identifier: **the site's name in `address-registry.yaml`**, `<COUNTRY>-<LOCALITY>-<NN>`, e.g.
`IN-BIDHOLI-01`, recorded per region in `region-registry.yaml`. *Changed 2026-09-29 by the operator:
it was `<region>-<IATA city>-<ordinal>` (`in-mum-1`), which gave the one real site a second name that
no inventory used.* Under RFC-0019 one datacentre is one Proxmox cluster.

## The rule that separates it from a region

> **Adding a datacentre is an operational act. Adding a region is a legal one.**

A datacentre may be added, drained or retired without telling anyone, because it is a term of no
contract. A region is `tenant.region` — contractual, immutable, changeable only by migration and
contract amendment (ADR-0041).

**Nothing tenant-facing may ever name a datacentre.** Putting one in an API response, a support
conversation or a contract would make an operational fact contractual by accident, and the fact is
one we need to change without asking.

## What it is not

**It is not a region**, and the two were the same word until RFC-0014. Conflating them is what made
residency appear to cost availability, which it does not: redundancy is achieved *between*
datacentres inside a region, so a region can survive losing a building without a byte crossing a
border (RFC-0015 decision 3).

**It is not a zone or a site.** Both words are already taken here — `zone` for time zones (GLO-0006,
REQ-0009, the `timestamptz` checks) and for DNS zones; `site` for a location in the source, which
appears throughout the check registry. A third meaning of `zone` is the mistake GLO-0004 exists to
prevent.

## The naming consequence, which is self-checking

Because two locations in one jurisdiction are two datacentres and never two regions, a number can
only ever distinguish buildings. So:

> **A region code never carries a number. If you are reaching for one, you want a datacentre.**

The archive is the counter-example. It used AWS-shaped region codes, and in AWS `ap-south-1` is
Mumbai while `ap-south-2` is Hyderabad — which AWS calls two *regions*, though both are India.
Adopting that shape imports a meaning that contradicts this vocabulary, and it did mislead once
before being caught. `region-registry.schema.json` now rejects `ap-south-1` as a region code.

## Related

- GLO-0007 — control region, the other half of the vocabulary RFC-0014 needed
- GLO-0004 — the precedent for why a word meaning two things is worth an entry
- GLO-0006 — `zone` in its time-zone sense, one of the two words this term avoids
- ADR-0041 — why a region is contractual and a datacentre is not
- RFC-0014 — decision 1, which introduces the distinction and the naming rule
- RFC-0015 — decision 3, in-region redundancy, which is what the distinction buys
- RFC-0019 — one datacentre is one Proxmox cluster, and why corosync forbids spanning two
