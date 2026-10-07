---
id: ADR-0127
title: Docket is a Release in each tenant's console, and no standalone HR console is served
type: adr
status: accepted
created: 2026-09-29
locked_at: null
supersedes: [ADR-0098]
superseded_by: null
scope: [org, web, tool]
applies_to: ["*"]
product: [docket, paridyum]
tags: [docket, console, web-console, release, multi-tenancy, oidc, adr-0098, adr-0092, adr-0097, adr-0126]
breaking: true
enforced_by: []
---

# ADR-0127 — Docket is a Release in each tenant's console, and no standalone HR console is served

## Status

**Accepted** 2026-09-29. The operator directed that everything a tenant runs is brought up by the
paridyum.in API, and agreed that `console.paridyum.com` is off that model. Supersedes ADR-0098, and
carries its decisions 2, 4 and 5 forward unchanged.

## Context

ADR-0098 served docket's HR interface as `console-docket` at `console.paridyum.com`: one deployment,
one sign-in client, for Paridyum's operators only. It was written before the org model
(ADR-0113 to ADR-0122).

Under the org model, and under ADR-0126:

- every tenant's products are paths on its one canonical host;
- `/dashboard` is that tenant's console (ADR-0118 D4);
- every tenant signs in through its own clients, created by the saga.

A standalone console that exists for one tenant and could not exist for any other is exactly the
hand-spun exception the operator ruled out. Its image also did not start (console-docket D-22), so
nothing is lost by stopping.

## Decision

1. **Docket's interface is part of the tenant console,** at `<tenant host>/dashboard`, shown when the
   tenant has a docket Release. There is no `console.paridyum.com` and no standalone HR deployment.
   Paridyum's HR uses `paridyum.com/dashboard`, like any tenant.
2. **Carried from ADR-0098 D2: the console forwards the user's own token.** A BFF with an
   authorization-code + PKCE login, a signed HttpOnly session, and the user's access token relayed
   to docket's API as a `Bearer` — through that tenant's docket client (ADR-0126 D4), not a shared one.
3. **`console-docket`'s code is the source,** moved into the tenant console, not rewritten.
4. **Carried from ADR-0098 D4: the `console-` class stays** for interfaces that genuinely must run on
   a workstation.
5. **Carried from ADR-0098 D5: the CLI and break-glass paths are unchanged** (ADR-0097).

## Consequences

**Accepted costs.**

- **Paridyum's HR has no browser interface until the tenant console exists** (PLAN-PARIDYUM-IN W1
  to W3). The CLI remains (ADR-0097).
- **Moving console-docket's screens** into the tenant console is work that a standalone deploy would
  not have needed.
- **Left to retire:** the `console-docket-web` client created on 2026-09-29 and the unmerged work
  that served the standalone console.

**Gained.**

- One console per tenant, holding whatever the tenant enabled, provisioned by the saga.
- No sign-in client, host or deployment exists for one tenant alone.

**Rejected alternatives.**

- **Serve the standalone console now and fold it in later.** That builds and operates a surface the
  plan is about to remove, and its image does not start.

## Related

- ADR-0098 — superseded by this ADR.
- ADR-0126 — per-tenant clients and host resolution.
- ADR-0118 D4 — `/dashboard` is the tenant console.
- ADR-0092, ADR-0097 — docket's token model and the CLI.
