---
id: ADR-0098
title: The HR operator interface is served, not run on a workstation (web-docket)
type: adr
status: accepted
created: 2026-09-15
locked_at: null
supersedes: [ADR-0055]
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [classes, naming, web, typescript, deployment, auth, adr-0020, adr-0055, adr-0070, adr-0092, adr-0093, c-0002, c-0008]
breaking: true
enforced_by: []
---

# ADR-0098 — The HR operator interface is served, not run on a workstation

## Status

**ACCEPTED** as of 2026-09-15. Supersedes ADR-0055's decision that the operator
console runs on the operator's own machine. ADR-0055 was accepted, not locked,
so this supersession needs no unlock.

## Context

ADR-0055 created the `console-` class for the graphical interface to
`tool-docket`, deciding it *"runs where the operator is rather than served from
the cluster"* — `deployable: false`, no Helm chart, no probes. The reasoning was
that HR staff would run it on their own machines, and that an interface owing
nothing to a cluster should not pretend to be a `web-` deployable.

**The assumption did not survive contact with the operators.** HR staff are not
developers. "Run it on your machine" means, per person: install a Node
toolchain, clone or download a build, keep it updated as the schema and the API
change, and trust a binary that reaches production personnel data — for people
whose job is hiring, not operations. A hosted URL behind sign-in is what every
comparable admin surface is, and what HR can actually use. The workstation
console optimised for a property (no cluster dependency) that costs more than it
saves for this audience.

ADR-0055 **already named the exit**: its decision 2 says *"If tenant traffic can
reach it, it is not a console: C-0008 makes it a product and it belongs in
`web-`."* The HR interface is not tenant-reachable — it is operators-only — so
C-0008 does not force the move. But C-0002's rule is simpler and decides it
anyway: **a served, user-facing web application is a `web-`.** The thing that
makes this a `web-` is that it is served, not who reaches it.

The served backend already exists and already authenticates: ADR-0093 made
`docket` a served, multi-user staff platform, and ADR-0092 made every write
carry a Zitadel operator token bound to a docket operator by subject. What has
been missing is a served *interface* to it — `console-docket` has the pages but
no auth, and calls the API with a plain attribution header the token-authed
backend now rejects.

## Decision

**1. The HR operator interface is a served `web-` product.** `console-docket` is
reclassified and renamed to **`web-docket`**: prefix `web-`, class `web`,
`deployable: true`. It therefore owes what ADR-0020 requires of every deployable
— a Helm chart, liveness/readiness probes, resource limits, a digest-pinned
image — the same obligations `web-hiring` and `web-admin` carry.

**2. It authenticates operators with a BFF that forwards their own token.** It
mirrors `web-admin`'s pattern (`src/lib/auth/*`, `src/lib/bff.ts`): an OIDC
authorization-code + PKCE login against the control-plane issuer
(`id.paridyum.com`), a signed HttpOnly session cookie, and the operator's Zitadel
access token relayed as a `Bearer` to `docket`'s API. No service account asserts
identity on the operator's behalf — the user's own token carries the subject
`docket` binds to (ADR-0092), and the shorter-lived access token is refreshed
from a kept refresh token (`offline_access`).

**3. It is served behind the edge at `console.paridyum.com`, operator-only.** A
Cilium L2 VIP on the estate network, an `edge_proxy_site` on `edge-in-01`, and a
Cloudflare record — the path `careers` and `admin` already take. Its OIDC client
is a **WEB** client in the `paridyum-operations` project (ADR-0070), redirect
`https://console.paridyum.com/api/auth/callback`, and that project's
`role_check` refuses a token with no operator role. Tenant traffic never reaches
it; if that ever changes it becomes a product under C-0008, but it is not one
now.

**4. The `console-` class is not deleted.** ADR-0055's insight — that the estate
needs a word for an operator interface, distinct from a tenant `web-` product —
stands. What is superseded is only its assumption that such an interface can
never be served. A genuinely workstation-only operator tool (one that must reach
a device, a local file, a smartcard) is still a `console-`. The HR interface is
not that; it is served, so it is `web-`.

**5. The CLI and break-glass paths are unchanged.** ADR-0097's `docket login`
(device-flow CLI) and the break-glass database link remain the terminal and
automation surfaces. This ADR adds the graphical served surface HR uses; it
removes nothing.

## Consequences

- **A repository rename**, `console-docket` → `web-docket`, with GitHub's
  redirects covering existing references while they are updated (the same move
  ADR already anticipates for `tool-docket` → `platform-docket`). Its manifest
  `class` moves `console` → `web`; `class-registry.yaml`'s `console` entry keeps
  its one remaining justification (decision 4).
- **It now owes ADR-0020.** A chart, probes, limits, and a digest-pinned image
  published by CI — none of which a `console-` owed. This is the cost of the
  reclassification and it is the point of it.
- **A served operator surface is an attack surface.** It is mitigated by being
  operator-only (project `role_check`), by holding no long-lived secret (PKCE
  public client, ADR-0070 D5), by keeping tokens in HttpOnly+Secure cookies
  never readable by client JS, and by the backend enforcing the operator token
  independently (ADR-0092) — the BFF is not the only thing standing between a
  request and the data.
- **The workstation console entry in `applications.yaml`** (the Native/loopback
  `console-docket` client) is retained for local/break-glass operator use; the
  served surface gets its own WEB client, one client per surface per environment
  (ADR-0070 D1).
