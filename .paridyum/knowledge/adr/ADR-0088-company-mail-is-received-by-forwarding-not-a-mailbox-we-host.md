---
id: ADR-0088
title: Company mail is received by forwarding to a primary inbox, not a mailbox we host or Gmail
type: adr
status: accepted
created: 2026-09-13
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["*"]
tags: [mail, email, addresses, mailbox, forwarding, cloudflare, adr-0067, adr-0083, adr-0085, req-0012, std-0017]
breaking: false
enforced_by: []
---

# ADR-0088 — Company mail is received by forwarding, not a mailbox we host

## Status

**Accepted**, 2026-09-13. **Refines ADR-0067 decisions 2 and 4** — the identity half of ADR-0067
is unchanged; only *where the mailbox physically lives* moves off Gmail. A human direction change
(C-0004): the operator declined Gmail as the company mailbox.

## Context

ADR-0067 decided two things about a person's mail: the address is `@paridyum.com` and it is the
**identity** (D4), and the **mailbox behind it is a free consumer Gmail account** (D2). The first
half is the load-bearing one — *"the paridyum.com address is what a person is; the Gmail address is
where mail lands, and it is swappable."*

The operator has now declined Gmail as the company mailbox and has receiving covered another way.
That does not touch ADR-0067's identity decision — it exercises exactly the swappability D4 built
in. So this ADR changes the mailbox, and only the mailbox.

## Decision

**1. Inbound `@paridyum.com` mail is received by forwarding to the operator's primary inbox** —
today via Cloudflare Email Routing (the estate already runs the domain's DNS on Cloudflare, and
Email Routing is free, catch-all-capable, and severs by deleting a rule). There is **no mailbox
this estate hosts, and no Gmail account named as the mailbox.** Where the primary inbox is is
opaque to the estate, which is exactly the property ADR-0067 D4 relied on.

**2. The address scheme is unchanged.** Person addresses stay `firstname.<person id>@paridyum.com`
(ADR-0067 D1); role/function addresses stay service-qualified (STD-0017 — `noreply@…`,
`<service>-<role>@…`, never `admin@`). A forwarding rule maps each to the primary inbox.

**3. Outbound is unchanged.** System/transactional mail still goes PMG → Resend, DKIM-aligned
(ADR-0083, ADR-0085). Forwarding is inbound only; it does not send.

**4. This is "for now", with a named reversal.** Forwarding stores nothing and proves nothing about
retention — it lands in a mailbox the estate does not control. When volume, retention, shared team
mailboxes, or a residency requirement (RFC-0015) demands a **stored** mailbox, the reversal is a
self-hosted mailstore on the estate behind PMG (Stalwart or mailcow — MX → PMG filter → mailstore),
which the PMG gateway (ADR-0083) was already the front half of. That is a deliberate build, entered
when one of those triggers fires, not before.

## Consequences

- **No inbound storage or retention we govern.** Forwarded mail lives in the operator's primary
  inbox; offboarding and any DPDP retention obligation on received mail are the primary inbox's
  problem, not the estate's — acceptable while the only recipient is the operator, a trigger to
  revisit the moment received mail must be retained or shared as a company record.
- **A forwarding provider is a dependency**, however thin. Cloudflare Email Routing going down means
  inbound mail bounces or is deferred; REQ-0012 governs it as it governs any vendor, at the low
  weight a forwarder earns.
- **The `@paridyum.com` identity is preserved and still severable** — deleting a person's forwarding
  rule ends inbound to their address, the same lever ADR-0067 D4 wanted, now pulled at Cloudflare
  rather than at Gmail.

## Related

- **ADR-0067** — the address-is-identity decision this refines; D2's Gmail mailbox is what moves.
- **ADR-0083, ADR-0085** — the outbound mail plane (PMG → Resend), unchanged.
- **STD-0017** — role/function addresses are service-qualified, never `admin@`.
- **RFC-0015, REQ-0012** — residency (the reversal trigger) and vendor governance (the forwarder).
