---
id: ADR-0111
title: A distributed agent is a class, and a station authenticates as a Native client by device authorization
type: adr
status: accepted
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, agent, service]
applies_to: ["*"]
tags: [agent, class, registry, identity, oidc, native, device-authorization, rfc-8628, zitadel, adr-0070, adr-0047, rfc-0054, c-0007, c-0005, agent-reel]
breaking: false
enforced_by: []
sources:
  - url: https://zitadel.com/docs/guides/integrate/login/oidc/device-authorization
    title: "ZITADEL — Device Authorization Flow (RFC 8628)"
    retrieved: 2026-09-27
    supports: >
      That the identity provider this estate runs supports device authorization as a Native
      application with the Device Code auth method — a code the user approves in a browser,
      then tokens on the device — which is decision 2's mechanism and why no secret is
      distributed to machines. Same source RFC-0054 cites; cited here too because this ADR's
      central decision rests on it (RFC-0020).
---

# ADR-0111 — A distributed agent is a class, and a station authenticates as a Native client by device authorization

## Status

**ACCEPTED**, 2026-09-27, on the operator's authorisation while RFC-0054 was being closed.
Not `locked`: the class half is checkable in principle once the class is published —
`repo.name_matches_scheme` already binds the prefix — but the identity half is a configuration
fact (an application's declared type) with no registry yet to check it against. `enforced_by`
is empty honestly; ADR-0070 decision 7's application registry is where a check would attach.

## Context

**RFC-0054 gives a tenant-facing product a station**: an agent installed on machines the estate
does not control, which ingests camera media locally and claims work from the product's service.
It needs an identity, and it falls through ADR-0070 decision 3's mapping in every direction:

- **`tool-` — none.** *"A tool binds loopback and resolves an operator locally… It is not on the
  internet and it has no OIDC client. Giving one a client is how a `tool-` becomes a service
  without an ADR."* A station is internet-facing, tenant-facing, and authenticates to a service —
  the exact opposite.
- **`console-` — Native**, but defined as *"an interface for Paridyum's own operators."* A
  tenant's station is not Paridyum's operator surface.
- **`mobile-` — Native**, but defined as distribution through an app store. A signed binary a
  product distributes is not that.

Two properties decide the answer, and both are forced by where a station lives: **no secret may
be distributed with it** (it sits on machines that are not ours, and ADR-0070 decision 5's
reason is stronger here, not weaker), and **no browser is guaranteed** (headless ingest hosts
exist). Zitadel answers both with **device authorization (RFC 8628)**: a human approves a code
once in a browser, tokens land on the device, and the session is revocable per station.

## Decision

**1. A new class, `agent`, and it is registry data.** The entry lands with this ADR (C-0007: the
rule lives in the artifact, the list lives in `class-registry.yaml`): prefix `agent-`, scope
`agent`, stack `rust`, `deployable: false`, `ci: true`. A repository in this class **distributes
a signed application to machines the estate does not control**; it is never scheduled onto the
cluster — which is why `deployable: false` is load-bearing rather than incidental — and
REQ-0014's supply-chain rules apply to what it ships.

**2. A station authenticates as a Native application by device authorization.** The Zitadel
application type is **Native**; the auth method is the **Device Code** flow; the human approves
the station once in a browser; the station holds a refresh session that is **revocable per
station**. **No client secret exists**, so nothing secret travels with the binary — the same
property ADR-0070 decision 5 buys for consoles, for a stronger reason.

**3. Per application, per environment.** ADR-0070 decision 1 is untouched: a station's staging
and production identities are different applications.

**4. The agent is not a service.** It binds no internet-facing listener and owns no request path
of its own. What it does — claiming account-scoped jobs, holding media, writing to the tenant's
destination — is RFC-0054 decision 3's worker model, and its tokens obey ADR-0047's claims
contract like any other resource access.

**5. ADR-0070's mapping table gains a row; its `tool-` row is untouched.** `agent` → **Native** →
**Device Code** → *a distributed client on machines the estate does not control, acting for one
tenant.* This extends that decision's table rather than rewriting any of it, and ADR-0070
carries a dated amendment note pointing here so the two do not disagree.

## Consequences

**Accepted costs**

1. **A new class touches three files in one change** — `class-registry.yaml`, and the two
   generated enums (`knowledge.schema.json`'s scope, `manifest.schema.json`'s class). Done
   deliberately in one change; the gate's agreement checks are the proof it stayed consistent.
2. **A new application type to operate.** Device sessions are another credential surface: an
   agent left on a stolen laptop is a session someone must notice and revoke. The product's
   console listing stations per account is the mechanism, and revocation is per station.
3. **The class list grows beyond what exists.** No `agent-` repository is created by this ADR —
   the class is declared because its first member is designed and the alternative is a `tool-`
   pretending. The registry's own rule applies: a class may exist before a repository does, and
   the honesty note above the class list says so.

**Gained**

- The station has an honest home, and `tool-` keeps its rule intact instead of being widened by
  the first distributed client that needed one.
- A future distributed client never has to be smuggled through `tool-` or `console-`.
- Headless hosts work: the Device Code flow needs no browser on the station, only on the human's.
- No secret distribution: the one credential class that survives binaries being copied is the one
  used.

**Rejected alternatives**

- **Reuse `tool-` for the station.** Rejected on ADR-0070 decision 3's own words; it would be the
  exact drift that rule names, and it would put an internet identity on a class defined by
  having none.
- **Reuse `console-`.** Rejected: it defines Paridyum's operator surfaces, and a tenant-facing
  client in it inverts the class while making every future tenant station an "operator" artifact.
- **Reuse `mobile-`.** Rejected: distribution channel and stacks are both wrong, and widening a
  class to fit one member is how classes lose meaning.
- **A machine user with a private key per station.** Rejected: it distributes a secret to every
  station (what decision 5 refuses), revocation is per key rather than per approved human, and
  the key would be minted remotely — the approval would happen where the human is not.
- **No identity; local-only mode as the product.** Rejected: a capability that only its authors
  can use is what C-0008 and ADR-0049 exist to refuse.

## Related

- **RFC-0054** — the product whose station this is, and the pairing flow built on decision 2
- **ADR-0070** — the mapping this extends; its decision 3 gains the row, its decision 1 and 5
  apply unchanged
- **ADR-0047** — Zitadel per region, organization per tenant, and the claims contract the
  agent's tokens obey
- **C-0007**, `class-registry.yaml` — the registry data this ADR adds, and the three-file
  agreement the gate verifies
- **ADR-0049** — the tenant path the station is part of; nothing it does resolves without one
- **REQ-0021**, **RUN-0005** — egress discipline and the Windows worker this station will
  eventually replace
- **REQ-0014** — supply chain for a signed, distributed binary
