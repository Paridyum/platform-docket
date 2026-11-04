---
id: ADR-0136
title: The app zone is a dedicated low-cost domain, set as configuration, so changing it later is configuration too
type: adr
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy, routing]
tags: [paridyum-xyz, app-zone, default-host, public-suffix, tls, cloudflare, adr-0121, rfc-0056, rfc-0072, p1]
breaking: false
enforced_by: []
---

# ADR-0136 — The app zone is a dedicated low-cost domain, set as configuration, so changing it later is configuration too

## Status

Accepted, 2026-10-09, by the operator. It answers PLAN-PARIDYUM-IN's P1. The operator first chose
`app.paridyum.in` because a `.app` domain was not affordable, and asked that the zone be easy to switch.
Checking the edge showed that `app.paridyum.in` cannot be served on the current tier (Context). Offered
the alternatives, the operator chose **a cheap dedicated domain**. **The domain's name is the operator's,
recorded here when bought:** **`paridyum.xyz`**, bought by the operator and added to Cloudflare on
2026-10-09 (nameservers `ignat`/`perla.ns.cloudflare.com`, verified the same day). Default hosts are
`<slug>.paridyum.xyz`.

## Context

ADR-0121 gives every tenant a default host, `<slug>.<app zone>`, under a zone that is a public suffix. It
prefers `.app`. Without any app zone, every tenant must publish DNS for its own domain before anything
serves (RFC-0072 §6).

**Why `app.paridyum.in` does not work today.** The edge accepts HTTPS only from the rented tier
(Cloudflare, ADR-0061). That tier's free certificate covers a zone's apex and **one** level below it.
`acme.app.paridyum.in` is two levels below paridyum.in, so TLS would fail at the rented tier. infra-edge's
inventory records the same limit for paridyum.com. Making it work would need either a paid certificate
manager (about $10 a month, more than a domain costs a year) or opening the edge to direct HTTPS, which
bypasses the rented tier and is a security-posture change. `app.paridyum.in` also shares a registrable
domain with the portal, so a tenant page could toss cookies at it.

**A dedicated domain has neither problem.** `acme.<zone>` is one level below the zone's apex, which the
free certificate covers. Nothing on it shares a registrable domain with the portal or with paridyum.com.

## Decision

1. **The app zone is a dedicated domain**, chosen and bought by the operator at low cost (a `.xyz` or
   similar), and added to the rented tier. A tenant's default host is `<slug>.<zone>`. The zone is ours,
   so a binding under it is `domain_active` at creation (ADR-0121 D4), and own domains are optional.
2. **The zone is configuration, never code.**
   - It is a value: `SAGA_APP_ZONE` in platform-tenancy, `ROUTING_APP_ZONE` in platform-routing, and the
     edge's wildcard site and DNS row.
   - No code compares a host to the zone's literal name.
   - ADR-0121 D1 is unchanged: nothing under paridyum.in is a tenant host.
3. **Changing the zone later keeps every link working:**
   1. set the new value;
   2. bind each tenant's `<slug>.<new zone>` (`domain_active`, ours);
   3. make it canonical where the old default host was;
   4. keep the old host bound as a redirect, as ADR-0121 keeps a renamed tenant's old host.

   A `.app` domain later is this procedure.
4. **The zone is submitted to the Public Suffix List** before the first tenant other than Paridyum is
   served on it (ADR-0121's condition, carried). Until it is listed, tenants on the zone can set cookies
   for each other's default hosts. They cannot set them for the portal or for paridyum.com.
5. **`*.<zone>` resolves through the edge,** with one proxied wildcard record. Products are paths, routed
   and cached by host, as on every tenant host (ADR-0130).

## Consequences

- **One more domain to renew and protect.** Losing it takes every default host down, so renewal goes
  where the other domains' renewals are tracked.
- **The first experience needs no DNS from the tenant:** `<slug>.<zone>` serves at activation.
- **Until the Public Suffix List lists the zone,** tenants are not cookie-isolated from each other. The
  first non-Paridyum tenant waits for the listing (decision 4).

## Alternatives considered

1. **`app.paridyum.in`.** The operator's first choice, rejected after checking the edge: a two-level
   name is outside the free certificate. It would cost a paid certificate (about $10 a month) or opening
   the edge, and it shares a domain with the portal.
2. **A paid certificate manager for `*.app.paridyum.in`.** Rejected: it costs more per year than a
   domain, and keeps the shared-domain risk.
3. **Opening the edge to direct HTTPS.** Rejected for now: it bypasses the rented tier's protection. It
   will be needed for tenants' own domains (PLAN-TENANT-HOSTS H4), and that is its own decision.
4. **`<slug>.paridyum.in`.** Rejected: ADR-0121 D7 retired it. It puts tenants beside the portal's own
   names.
5. **No app zone, own domains only.** Rejected by the operator: every first experience would wait on DNS.

## Related

ADR-0121 (unchanged; its app zone is now named by this decision), RFC-0056 D5, RFC-0072, ADR-0130,
ADR-0061, PLAN-PARIDYUM-IN P1, PLAN-TENANT-HOSTS H4.
