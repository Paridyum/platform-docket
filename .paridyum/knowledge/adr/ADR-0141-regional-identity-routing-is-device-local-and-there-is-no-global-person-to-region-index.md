---
id: ADR-0141
title: Regional identity routing is device-local, and there is no global person-to-region index
type: adr
status: proposed
created: 2026-10-10
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy, identity]
tags: [regions, residency, identity, login, portal, routing, device-cookie, federation, adr-0117, adr-0119, adr-0121, rfc-0014, rfc-0015, rfc-0051]
breaking: false
enforced_by: []
---

# ADR-0141 — Regional identity routing is device-local, and there is no global person-to-region index

## Status

**PROPOSED** 2026-10-10 by the operator. It records how a returning person reaches the region their
credential lives in once there is more than one portal instance, and it affirms **ADR-0117 D6** — *"there is no
global index from a person to a region"* — by giving the routing a mechanism that holds no global
fact. Nothing here is built for a second region yet (ADR-0117 D10); the decision is made now because
the portal's entry screen is being built now, and the question cannot be deferred past it without
the screen answering it by accident.

It takes RFC-0015 and RFC-0051 as direction; both are `proposed`.

## Context

**N regional portal instances, and one host.** ADR-0117 D5 gives each region a **portal instance** —
a Zitadel instance holding that region's org members, with its own sign-in host under `paridyum.in`
(ADR-0121 D6). paridyum.in itself is one host served from the edge, carrying no data (ADR-0117 D4).

So a returning person who types `paridyum.in` must reach **the region their credential lives in**,
and the platform must not learn, centrally, which region that is. RFC-0015 D5 forbids inferring a
region from the address a request arrives from; ADR-0117 D6 forbids a global person-to-region index,
including a hash of an address, because any such index is personal data in the control plane
(RFC-0014 D2, ADR-0115).

**Three models were on the table.**

| | Model | What it stores globally | What it costs |
| --- | --- | --- | --- |
| **A** | **Device-local memory, no global fact** | nothing | a cold device must be routed by a link or by asking |
| **B** | **A global routing record**, `handle → region` | a mapping from a person or handle to a region | a person-to-region index — location data — in the control plane |
| **C** | **A global identity**, one credential for all regions | the credential itself | the whole identity population central, contradicting RFC-0017 and ADR-0117 D5 |

**Model B looked cheap and is not.** The temptation is to call it "minimal data" — a handle and a
region, no name, no address. But **a region is location data**: it says where a person lives or
works, which is the fact residency exists to protect. *Minimal* is a measure of size, not of kind,
and this is the kind the control plane is forbidden to hold (ADR-0117 D6, RFC-0014 D2). Hashing the
handle does not change it: a hash of a unique identifier is still a join key back to the person, and
ADR-0117 D6 already refused *"even a hash of an address"* for exactly this reason.

**Model C is the model the estate already rejected.** One credential for all regions is the
"everything to `.in`" direction ADR-0119 replaced, and it puts every org member's identity in one
place regardless of where they belong.

**The hazard that makes A's cost real, and is not hidden here.** A cold device — a new browser, a
cleared cookie jar, a link opened on a phone — has no memory of the region, and if it lands on the
wrong region's sign-in, a person can **create a second credential there**, in a region they do not
belong to. That is a duplicate account, and it is the price of holding no global index. The decision
below mitigates it (R4, and the ordered confirmation screen) rather than eliminating it: a platform
that refuses a global index cannot make the cold-landing case impossible, only hard to fall into
silently.

## Decision

**Model A only, for now: routing is device-local, and there is no global person-to-region index.**
This affirms ADR-0117 D6.

**The global entry at `paridyum.in` routes from three sources, in order:**

1. **The signed device cookie `__Host-pd_region`.** The region a person last signed in to is
   remembered on their device. `__Host-` scopes it to the one origin and forbids a `Domain`
   attribute, so it cannot leak across hosts; it is signed, so it cannot be forged into a
   wrong-region redirect.
2. **The region named in every mail link.** Every link a region's mail carries names the region it
   belongs to (ADR-0117 D6), so a link is a routing fact that needs no lookup.
3. **A one-tap confirmation screen** for a device with neither: the likely region shown **first**,
   ordered by the visitor's country and language through RFC-0051's `suggest_for`, and **nothing
   preselected** (ADR-0103 D4 — refused, never defaulted). The person chooses.

**After sign-in, the session's token issuer is authoritative.** The region is taken from the
verified token's issuer — the region's portal instance — never from a request parameter or the
cookie (ADR-0121 D6). The cookie and the link choose *where to send a sign-in*; they never decide
where a session is served.

### Rules

- **R1 — A credential is created in exactly one region.** A person's portal credential lives in the
  region they signed up in, and only there (ADR-0117 D6).
- **R2 — There is no global person id, and no id encodes a region.** No identifier in the control
  plane names a person, and no id carries its region in its bytes.
- **R3 — The region is discovered by device → link → choice.** Never by a server-side lookup, and
  never inferred from the address a request arrives from (RFC-0015 D5).
- **R4 — A wrong-region landing never silently mints a second credential.** The sign-up path in a
  region a person may not belong to must not create a credential without the person having been
  shown which region they are joining; a cold landing surfaces the choice before it creates
  anything.
- **R5 — Cross-region membership is linked-user federation (ADR-0117 D7), and the credential never
  moves.** A person admitted to an org or tenant in another region is a linked user there
  (ADR-0119 D4); their credential stays in its region.
- **R6 — A credential's region and an org's region are immutable** except by contract and migration
  (ADR-0117 D1, ADR-0041 D3). Routing does not change placement.
- **R7 — The control plane holds no personal data and no person-to-region mapping** (ADR-0115,
  ADR-0117 D3). Model B is forbidden by this rule, not merely unbuilt.

## Consequences

**Accepted costs**

1. **A cold device must be asked, or must arrive by a link.** Model A trades a global lookup for a
   step, and the step falls on the device with the least information.
2. **The duplicate-account hazard is mitigated, not removed.** A person who ignores the confirmation
   screen and signs up in a region they do not belong to can still hold a second credential; the
   screen makes it a choice they were shown, not one made for them.
3. **The device cookie is a fact that can be wrong** — a shared device, a person who belongs to two
   regions, a cookie that outlives a migration. The issuer is authoritative after sign-in precisely
   because the cookie is not.
4. **Mail links must carry the region**, so every region's mail template is region-aware. A link
   that forgets it falls back to the confirmation screen, which is slower and looks less certain.

**Gained**

- **Residency is true for the routing fact too** — nothing about which region a person belongs to
  leaves the region or the device.
- **The entry works with one region and needs no redesign for N**: a second region adds a cookie
  value, a link target and a row on the confirmation screen, and nothing central.
- **No global index exists to become a residency breach** — the failure RFC-0014 cost 2 names is
  designed out rather than monitored for.

**Rejected alternatives**

- **Model B — a global routing record `handle → region`.** Rejected for now: a region is location
  data, so the record is personal data in the control plane even when it looks minimal, and hashing
  the handle does not change what it is (ADR-0117 D6).
- **Model C — a global identity.** Rejected: it centralises the identity population ADR-0117 D5 and
  RFC-0017 deliberately split by region.
- **Inferring the region from the request's address or the browser's language.** Rejected by
  RFC-0015 D5 and GLO-0006; the wrong-region outcome is a contract amendment plus a migration
  (ADR-0041 D3), and a person on a trip is the worked example.
- **Preselecting the likely region on the confirmation screen.** Rejected by ADR-0103 D4: a default
  accepted without being read is not a choice, and this one is contractual.

**B and C may be revisited if the residency promise changes.** They are rejected *for now* on the
grounds that they hold location data centrally. If the estate ever makes a promise that requires a
central person-to-region fact — or if the duplicate-account cost of A is measured and found worse
than the residency cost of B — this is the decision to reopen. It is `proposed`, not locked, so that
reopening is a new ADR rather than an exception.

## Related

- **ADR-0117** D1, D3, D5, D6, D7, D10 — the org region, the personless registry, a portal instance
  per region, no global index, linked-user federation, one region live
- **ADR-0121** D6 — the portal is one host, its data served by the session's region from the token
  issuer
- **ADR-0119** D4 — cross-plane access is federation, never a shared user store
- **RFC-0015** D5 — how a region is chosen; never inferred from the address
- **RFC-0051** — the jurisdiction control, `suggest_for` ordering, nothing preselected
- **ADR-0103** D4 — refused, never defaulted · **ADR-0041** D3 — region contractual and immutable
- **ADR-0115** — no person in the control plane · **RFC-0014** D2 — the three planes
- **GLO-0006** — locale is not inferred from the address either
