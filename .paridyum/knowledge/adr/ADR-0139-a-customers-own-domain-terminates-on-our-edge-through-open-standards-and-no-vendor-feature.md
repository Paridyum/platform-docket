---
id: ADR-0139
title: A customer's own domain terminates on our edge, through open standards and no vendor feature
type: adr
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform]
applies_to: ["*"]
product: [hosting, routing, tenancy]
tags: [custom-domains, edge, acme, tls, caddy, vendor-lock-in, exitability, adr-0061, adr-0121, adr-0136]
breaking: false
enforced_by: []
sources:
  - url: https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/start/getting-started/
    title: "Cloudflare for SaaS — Getting started"
    retrieved: 2026-10-09
    supports: >
      The rejected alternative: customer hostnames are a Cloudflare-specific object, the customer's CNAME
      targets a Cloudflare-proxied name, and an apex needs "apex proxying" (an Enterprise add-on).
  - url: https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/plans/
    title: "Cloudflare for SaaS — Plans"
    retrieved: 2026-10-09
    supports: >
      100 custom hostnames free, then USD 0.10 each per month on Free, Pro and Business; apex proxying is
      Enterprise only.
  - url: https://developers.cloudflare.com/support/troubleshooting/http-status-codes/cloudflare-1xxx-errors/error-1014
    title: "Cloudflare — Error 1014: CNAME Cross-User Banned"
    retrieved: 2026-10-09
    supports: >
      Why a customer whose DNS is on Cloudflare cannot CNAME to a proxied name in our account without
      Cloudflare for SaaS, and why our customer target is therefore DNS-only.
  - url: https://caddyserver.com/docs/automatic-https
    title: "Caddy — Automatic HTTPS (On-Demand TLS)"
    retrieved: 2026-10-09
    supports: >
      On-demand TLS obtains a certificate at the first handshake for a name not configured in advance,
      requires an `ask` endpoint to permit each name, and falls back between Let's Encrypt and ZeroSSL.
  - url: https://github.com/caddyserver/caddy
    title: "caddyserver/caddy (GitHub)"
    retrieved: 2026-10-09
    supports: >
      Licence Apache-2.0; latest release v2.11.7, published 2026-10-03.
  - url: https://letsencrypt.org/docs/rate-limits/
    title: "Let's Encrypt — Rate Limits"
    retrieved: 2026-10-09
    supports: >
      300 new orders per account per 3 hours; 50 certificates per registered domain per 7 days; 5 per exact
      identifier set per 7 days; 5 authorization failures per identifier per account per hour.
  - url: https://carrd.com/docs/sites/using-a-custom-domain
    title: "Carrd — Using a custom domain"
    retrieved: 2026-10-09
    supports: >
      The market's shape for an apex: A records to the provider's addresses, `www` as a CNAME.
  - url: https://www.framer.com/help/articles/how-to-connect-a-custom-domain/
    title: "Framer — How to connect a custom domain"
    retrieved: 2026-10-09
    supports: >
      The same shape: two A records for the apex, `www` CNAME to the provider's name.
---

# ADR-0139 — A customer's own domain terminates on our edge, through open standards and no vendor feature

## Status

Accepted, 2026-10-09, by the operator: "we don't want vendor lock-in", then "yes go ahead" to a fixed
address, direct traffic for customer domains, and Caddy for on-demand certificates. It amends ADR-0061
for customer domains only, and leaves it standing for Paridyum's own zones.

## Context

Hosting is vertical one (product-registry `hosting`, BET-0009), and a hosted site is worth little
without its owner's domain. ADR-0121 D2 gives a tenant an own domain, and routing proves ownership with
a TXT challenge. That proof works, but **nothing serves the domain once it is proven.** X2's synthetic
own domains worked only because they sat under `paridyum.com`, a zone in our own Cloudflare account. A
real customer's domain cannot work today:
- if their DNS is on Cloudflare, a CNAME to our proxied names is refused (error 1014, cross-account);
- if it is not, the request reaches Cloudflare with a name our account does not hold;
- our edge admits only Cloudflare's ranges, and holds certificates only for our names (ADR-0061 D4).

The vendor's answer is Cloudflare for SaaS. **The operator refused it as lock-in.** Every customer's
DNS would point at a Cloudflare-specific object, and leaving would mean asking every customer to change
their records. It would also break ADR-0061 D3's rule that nothing distinctive is used at the rented
tier. Its apex support is an Enterprise add-on anyway.

## Decision

### D1. Customers point at us with plain DNS, at names and addresses we own

| A customer's domain | Their record | Value |
| --- | --- | --- |
| a subdomain (`shop.example.com`) | `CNAME` | `domains.paridyum.in` |
| an apex (`example.com`) | `A` | the edge's fixed IPv4 address |
| ownership, before either | `TXT` at `_paridyum-verify.<host>` | routing's challenge (ADR-0121) |

- `domains.paridyum.in` is a **DNS-only** record (never proxied) to the edge's fixed address, declared
  in the routing table. It sits outside the app zone, so no tenant slug can take it.
- The edge holds **a fixed IPv4 address** (an Elastic IP), declared in infra-edge's OpenTofu. Moving it
  to another provider or node means moving the address or changing one record. Customers keep
  `domains.paridyum.in`, and only apex customers hold the address itself, which the portal shows.
- IPv6 (`AAAA`) is owed: the node has no IPv6 address today.

These are open standards (DNS, TLS, ACME). No customer record names a vendor.

### D2. The edge terminates customer domains with certificates it obtains itself, on demand

- **Caddy** (Apache-2.0) runs on the edge node beside nginx, for customer domains only. Its On-Demand
  TLS obtains a certificate at the first handshake for a name it has not seen.
- **Every issuance is asked first.** Caddy's `ask` calls routing, which answers yes only for a host
  bound to a tenant **and `domain_active`**, so TXT-verified. Anything else is refused before an ACME
  order exists. That protects the CA's limits (300 orders per account per 3 h; 5 failures per
  identifier per hour) and stops the node issuing for names it does not serve.
- **Challenges are TLS-ALPN-01, with HTTP-01 as the fallback.** DNS-01 is impossible for a domain whose
  DNS is the customer's, so ADR-0061 D4's DNS-01 rule stays for our own names only.
- **The CA is replaceable**, which is part of the point. Let's Encrypt is primary and ZeroSSL the
  fallback, through ACME, and Caddy's issuer list is configuration.
- Certificates are stored on the node (`/var/lib/caddy`). A rebuilt node re-issues on demand. Above
  roughly 300 customer domains, a rebuild would meet the per-account order limit, so shared storage
  becomes owed then.

### D3. One node, two terminators, split by the name the client asks for

On :443, an nginx `stream` listener reads the TLS SNI without terminating it (`ssl_preread`):
- a name in the inventory's own sites (`paridyum.com`, `paridyum.in`, `*.paridyum.xyz`, …) goes to
  nginx's HTTPS server, which works exactly as today;
- any other name goes to Caddy.

Both receive the client's address through the PROXY protocol. On :80, nginx keeps serving its own
names. Any other name has its `/.well-known/acme-challenge/` proxied to Caddy and everything else
redirected to HTTPS.

Caddy then proxies a customer's host the way the edge proxies any tenant host (ADR-0130): `/` and the
product paths go to the same shared deployments, with `Host` preserved. **The apps resolve the tenant
from `Host` as they already do** (`resolve_surface`), and an unbound or taken-down host answers 404.

### D4. The edge admits the world on 80 and 443, and our own zones still admit only the rented tier

The security group opens 80 and 443 to all addresses, because customer domains arrive directly. **Paridyum's
own names keep ADR-0061's posture:** their nginx servers accept a connection only when the
PROXY-protocol source is one of the rented tier's ranges (`allow`/`deny` on the real address). A
client that reaches the node directly and names `paridyum.com` gets 403. ADR-0061's exit drill is
unchanged.

### D5. The order a customer follows, and what each step proves

1. Add the TXT record. Routing verifies it, and the binding becomes `domain_active` (ADR-0121). The
   portal shows the record and a verify action (RFC-0072 S7.6 and S7.7).
2. Add the CNAME (or the apex A record).
3. The first visit over HTTPS asks routing, gets the certificate, and is served.

A binding removed (compensation, or RUN-0009) makes the apps answer 404 at once. The next `ask` for
that host is refused, so no renewal follows.

## Alternatives considered

- **Cloudflare for SaaS.** It absorbs attacks on customer domains and manages their certificates.
  **Rejected by the operator as lock-in:** customer records would target a vendor object, apex needs
  Enterprise, and it breaks ADR-0061 D3.
- **Caddy for every name, replacing nginx.** One terminator. **Rejected for now:** it rewrites a working,
  measured edge (cache rules, rate limits, the identified-request bypass of 2026-10-09) to gain nothing
  for our own names. It remains possible later.
- **Our own controller around lego, issuing a certificate when a binding becomes active, and writing an
  nginx server per customer host.** No new tool. **Rejected:** it is a certificate manager we would
  write and keep correct (renewal, retries, storage), where Caddy's is maintained and built for
  exactly this. Exit stays cheap either way: any ACME client replaces Caddy.
- **A second node for customer domains.** Clean separation. **Rejected for now:** cost, and a second
  thing to keep identical. The SNI split keeps that move open.

## Consequences

**Accepted costs.**
- **Customer-domain traffic is not absorbed by a rented tier.** The node is reachable directly.
  Mitigations: AWS Shield Standard (automatic), ADR-0138 D3's limits on Caddy's proxied paths, and the
  `ask` gate. A generic CDN can sit in front later without changing any customer record.
- **A new tool on the most exposed machine.** Caddy is pinned by version and checksum, like lego
  (ADR-0061, infra-edge).
- **The fixed address is a cost line** (a public IPv4 address), and moving it between providers is not
  free.
- **Certificate state lives on one node** until shared storage is needed (D2).

**Owed (recorded where they land):** IPv6; shared certificate storage above about 300 customer domains;
an alert on issuance failures; the portal's domain panel (RFC-0072 S7.6/S7.7, extended with D1's
records); an own domain held **outside** our account to prove D5 end to end.

## Rollout

Each slice is one PR, applied with a plan first:

1. **routing** — the `ask` endpoint: a read on its own listener, reachable only by the edge, that answers
   200 for a `domain_active` host and 404 otherwise. Tested.
2. **infra-edge (OpenTofu)** — the Elastic IP, the security-group change, and the `domains.paridyum.in`
   record (DNS-only).
3. **infra-edge (role)** — the SNI split, the PROXY protocol, the rented-tier allow-list on our own
   sites, and Caddy (pinned, checksummed) with on-demand TLS and the `ask` URL.
4. **Proof** — an outside domain through D5, with its certificate issued, served and taken down
   (RUN-0009).

Undoing it: close the security group to the rented tier again and remove Caddy's upstream from the
stream map. Our own names never depended on either.

## Related

- ADR-0061 — the edge; amended by this for customer domains (see its addendum).
- ADR-0121 — own domains and their TXT proof. ADR-0130 — products as paths on a host.
- ADR-0136 — the app zone. ADR-0138 — the abuse controls that now face direct traffic too.
- RFC-0016 — named Cloudflare for SaaS for this. This decision replaces that part.
- The Sites product (an RFC proposed separately): the hosted sites whose domains these are.
