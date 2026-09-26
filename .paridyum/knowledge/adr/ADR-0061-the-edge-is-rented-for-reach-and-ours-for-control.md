---
id: ADR-0061
title: The edge is rented for reach and ours for control, and leaving is rehearsed
type: adr
status: accepted
created: 2026-09-04
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [edge, cdn, cloudflare, digitalocean, residency, dns, tls, vendor, exitability, rfc-0016, rfc-0015, req-0012, c-0006, adr-0044]
breaking: false
enforced_by: []
---

# ADR-0061 — The edge is rented for reach and ours for control

## Status

**Accepted**, 2026-09-04.

## Context

The estate has a cluster serving a landing page and no public path to it. RFC-0016 decided the
edge is rented and selected Cloudflare; RFC-0023 D5 already assumed the origin dials out and
holds no inbound port. What was missing was who terminates the public connection.

Two facts, found by checking rather than assuming, decide the shape:

**Cloudflare Tunnel would remove the fallback.** It needs no public address, which is its appeal
and its defect: with no origin address there is nothing to repoint DNS at, so a vendor incident
becomes a total outage with no manual recovery. The property that removes the attack surface
removes the escape hatch with it.

**Regional Services is Enterprise-only.** RFC-0016 D4 names it as the control for which
datacentres may decrypt HTTPS, which is what RFC-0015 D3 depends on. At a price this estate can
pay, a rented edge cannot deliver that guarantee — which is RFC-0016's own stated reversal
condition for decision 1, and it has now fired.

A third fact changed the weighing rather than a decision. The origin is five virtual machines on
one hypervisor in a house, so an edge buys latency and not availability — but that is temporary
by intent, and a design chosen for a single-machine origin would have to be replaced precisely
when it started to matter.

## Decision

### 1. A rented tier and one of ours, divided by what the traffic carries

```
visitor
  → rented tier    anycast, DDoS absorption, TLS at the nearest point of presence
  → our tier       one node per residency region, publicly-trusted certificate
  → WireGuard      the mesh that already exists
  → cluster
```

**Rented for reach. Ours for control.** Neither is load-bearing alone: the rented tier cannot
satisfy residency, and ours cannot absorb an attack.

### 2. Latency is engineered — at the rented tier and in the path behind our node, not by adding nodes of ours

**LATENCY IS A FIRST-CLASS CONCERN AND THIS DECISION IS ABOUT WHICH INSTRUMENT SERVES IT.** An
earlier phrasing of this section read "an edge node exists for residency and control, never for
latency", and it was misread on first contact as *we do not care about latency*. That reading is
wrong and the sentence invited it.

The estate has three latency instruments and they are not interchangeable:

| Instrument | What it improves | Whose |
| --- | --- | --- |
| **Anycast and points of presence** | the first hop — the visitor reaches a fast network wherever they are | rented |
| **Caching at the point of presence** | everything RFC-0016 D2 permits never travels the long leg at all | rented |
| **The path behind our node** | the leg from the point of presence to our node, and our node to the cluster | **ours** |

**The third is ours and it is not small.** The mesh hop between our node and the cluster is a
path we control, and a relayed WireGuard connection has been measured on this estate at roughly
190 ms where a direct one is a few. A node placed where the rented tier peers badly, or a mesh
path that silently falls back to a relay, costs more than any amount of node-multiplying would
recover. That is the latency work this repository actually owns.

**What adding a node of ours cannot do is help a visitor who is far from their tenant's region.**
RFC-0015 D3 fixes the tenant's data in one region and D5 makes that the operator's choice. A node
nearer the visitor cannot serve them, because what they came for is not in it — and the rented
tier is already terminating TLS and serving cache near them.

So:

**ONE NODE PER RESIDENCY REGION. NEVER ONE PER POPULATION CENTRE.**

C-0006 classes CDN as commodity and RFC-0016 D1 concludes that building a point-of-presence
network is not ours to build. That conclusion is correct and this decision does not contest it:
one origin in the region a tenant's data must stay in is not a point-of-presence network.

The rule exists because the boundary is crossed with good intentions. A node added *because
users are far away* is the first node of a CDN, and the archive already built that across eight
regions — its own retrospective judged it too large an operational surface for a platform at
this stage.

**THE RULE IS CONDITIONAL ON A RENTED TIER BEING IN FRONT, and that is worth stating because the
exit is a design goal.** If the rented tier is ever removed — the drill in decision 6 made
permanent — then anycast and PoP caching leave with it, and node count becomes a latency
question again immediately. Leaving is therefore not only a vendor decision; it is the decision
to take on the instrument this decision currently declines to build.

### 2a. The original phrasing, kept

*Left visible rather than deleted, because the correction is the useful part: the rule did not
change, only the sentence that was doing the explaining. Recorded so nobody reinstates the
shorter wording thinking it says the same thing.*

> An edge node of ours exists for residency and control, never for latency.

### 3. The rented tier is used for proxying, DDoS and DNS hosting, and nothing else

No Workers, no Page Rules, no Transform Rules, no vendor bot management, no vendor-issued origin
certificates, no vendor analytics anything depends on.

RFC-0016 D1's reasoning already said it — *"decision 1 keeps the tier dumb enough to be
portable"* — and this makes it a rule rather than an aspiration. **Every feature used is a
feature that must be rebuilt to leave**, and this tier's entire value as a commodity is that its
provider can be changed.

**Design toward a generic anycast CDN**, not toward a named replacement. The interface used is
proxy, cache, TLS and DDoS absorption, which every provider in the category offers. Nothing
distinctive is used, so nothing distinctive is gained and nothing distinctive is lost.

### 4. Our node holds a publicly-trusted certificate, obtained by DNS-01

**Never a vendor origin certificate.** A Cloudflare Origin CA certificate is trusted only by
Cloudflare, so a node holding one serves errors the moment the proxy is bypassed — which
destroys the fallback that is the strongest single reason for the node to exist.

**DNS-01 rather than HTTP-01**, because DNS-01 issues a certificate for a hostname the node is
not yet receiving traffic for. That is the property that makes adding a second node, replacing a
node, or moving to anycast cheap; HTTP-01 makes each of them a deadlock.

### 5. DNS records are code

Records live in this estate's repositories and are applied to a provider by API. **No record is
created or edited in a vendor console.**

RFC-0016 D3 already calls the routing table *generated*; this holds it to that. The mechanism is
the whole of the anti-lock-in position: when records are code, changing provider is changing a
target, and when they are console state, changing provider is a person transcribing a zone under
time pressure.

**The zone is already hosted at Cloudflare, so the first act is an export**, not a creation.

### 6. Leaving is rehearsed, not documented

**A scheduled drill bypasses the rented tier so traffic reaches our node directly, confirms the
site serves with a valid publicly-trusted certificate, and restores.**

On production, briefly, because a drill against something else proves something else.

RFC-0016 already names the exit — *"repointing DNS and terminating TLS at our own ingress"* —
and before this decision the estate had no ingress, so the exit was a sentence. This estate has
already learned what an unrehearsed procedure is worth: `make etcd-restore-plan` prints its own
steps and says outright that none of them has been executed.

### 7. Both vendors are selected and neither is approved

RFC-0016 D4 says it: *"Selection is not approval."* REQ-0012 owns approval, is `deferred`, and
D4 names four things still owed for Cloudflare. **DigitalOcean is a new vendor with no
assessment at all.**

Depending on both in production makes that debt live rather than theoretical, and it is recorded
here so that the first person to ask "did anyone check this vendor" finds the answer is no.

## Consequences

**Accepted costs.**

- **A machine to run.** One node, patched, monitored, and holding a certificate that must renew.
  The rented-only design had none, and that was its strongest argument.
- **A tier more to reason about.** A failure can now be in either, and the first question in any
  incident becomes *which one*.
- **The trace still does not start at the true edge.** ADR-0044 wants `traceparent` originated
  where the request arrives. Ours originates it one hop in, so the rented tier's leg is
  invisible. An improvement on nothing; not compliance.
- **One node is one node.** It has a four-dollar failure domain. The rented tier absorbs attacks
  but does not replace a node that has stopped.
- **The rule in decision 2 will feel wrong at some point** — there will be a real latency
  complaint from a real user in a region we have no node in, and the correct answer will be to
  fix it at the rented tier or not at all.

**Gained.**

- A fallback that can be exercised rather than described.
- The ingress that residency will require anyway, built before it is urgent.
- Three of RFC-0016 D1's five jobs that the free rented tier cannot perform.
- An exit that is a capability.

## Alternatives considered

- **Cloudflare Tunnel only.** Free, no machine, anycast, and the fastest thing to ship.
  Rejected: no origin address means no fallback, no ingress for residency-bound traffic later,
  and an exit clause with nothing to exit to. It optimises for the estate we have this month.
- **Our own nodes only, no rented tier.** Maximum control and maximum exposure: a four-dollar
  droplet is the entire public surface with no DDoS absorption, and region selection falls back
  to DNS, which RFC-0016 explicitly rejects as the router.
- **Our own point-of-presence network.** What the archive built. C-0006 calls the category
  commodity and RFC-0016 D1 refuses it; the retrospective agreed. Reconsider only if residency
  requirements multiply regions faster than a rented tier can serve them.
- **Cloudflare Enterprise with the Data Localization Suite.** Solves residency without a node of
  ours and is the correct answer at a size this estate is not. Unpriced, and RFC-0016's own
  cost 1 names that as the input that could reverse its vendor selection.

## Related

- **RFC-0016** — the edge, its five jobs, the cacheability rule, and the reversal condition this
  decision fires; amended the same day to record the partial reversal
- **RFC-0015 D3** — nothing crosses a region boundary, which is what the rented tier cannot
  guarantee at our price
- **C-0006** — CDN as commodity, which decision 2 exists to keep true
- **RFC-0023 D5** — the origin dials out and holds no inbound port; this moves the port to a
  rented machine rather than removing it
- **RFC-0037** — the edge routing block at `X = 210–214`, reserved before anything needed it
- **REQ-0012** — vendor approval, `deferred`, owed for both
- **ADR-0044** — `traceparent`, still partly unmet
