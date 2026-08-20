---
id: GLO-0007
title: Control region — where the platform's global facts live, not where a customer belongs
type: glossary
status: proposed
created: 2026-08-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [region, residency, control-plane, vocabulary, rfc-0014, adr-0047]
breaking: false
enforced_by: []
---

# GLO-0007 — Control region

## Definition

**The control region is the one region holding the control plane: the platform's global facts, and
the `paridyum` Zitadel instance holding staff. Exactly one exists.**

Field: `control: true` in `region-registry.yaml`. A property of a region, never of a tenant.

## What it is not

**It is not a tenant's region.** A tenant's region is where that tenant's data lives, one per
tenant, chosen by the operator and contractually immutable (ADR-0041). The control region is where
*our* global facts live. A tenant that happens to sit in the control region is not thereby special,
and one that does not is not thereby distant.

**It is not a tier.** Being the control region is never sold, never a paid upgrade, and never a
reason to prefer one region over another. RFC-0014 decision 1 states the rule about what is *sold*;
this is the same rule applied to a word.

**It is not a home region.** That term is retired, and this entry is why.

## Why this term exists at all

`home region` meant two different things and nobody noticed for a week.

ADR-0047 decision 1 introduced it for the first sense — *"a separate `paridyum` instance in the home
region holds staff and operators"* — meaning **the platform's** home. Read plainly, and in every
conversation about residency, it means **the customer's** home: the region a business belongs to.
Both readings are natural. That is exactly what makes a word unusable, and exactly the situation
GLO-0004 was written for, where `tenant` meant two different isolation boundaries depending on who
read it.

The cost of leaving it was concrete. A design discussion produced *"each continent would have its own
home region"* — a sentence that is obviously true under one reading and a redesign of the control
plane under the other.

So: **control region** for ours, **region** for a tenant's, and `home` for neither.

## The related trap

The second sense — a customer's home — is deliberately **not** given a new name. It is just the
tenant's `region`, and adding a synonym would recreate the problem this entry solves.

## Related

- GLO-0008 — datacentre, the other half of the vocabulary RFC-0014 needed
- GLO-0004 — the precedent: one word, two isolation boundaries, and what it cost
- ADR-0047 — decision 1, where the ambiguous term entered, and the instance this region holds
- ADR-0041 — `tenant.region` as contractual and immutable, the sense this term is *not*
- RFC-0014 — decision 1, which retires `home region` and defines the split
- RFC-0015 — the control plane's behaviour, and why this region carries the strictest availability target
- RFC-0017 — why an operator's personal data does *not* live here, despite the operator being global
