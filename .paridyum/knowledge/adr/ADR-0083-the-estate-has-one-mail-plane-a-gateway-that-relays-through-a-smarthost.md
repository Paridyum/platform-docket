---
id: ADR-0083
title: The estate has one mail plane — a gateway that relays through a smarthost, environment-aware
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [mail, smtp, email, proxmox-mail-gateway, smarthost, deliverability, dkim, spf, dmarc, adr-0067, adr-0056, adr-0049, rfc-0023, req-0003]
breaking: false
enforced_by: []
sources:
  - url: https://zitadel.com/docs/apis/resources/admin/admin-service-add-smtp-config
    title: "ZITADEL — Admin API: AddSMTPConfig"
    retrieved: 2026-09-12
    supports: >
      How Zitadel is pointed at an SMTP relay, which is what makes this ADR's first consumer
      workable: Zitadel has no SMTP of its own and cannot send an operator's init email without
      being configured against one.
---

# ADR-0083 — The estate has one mail plane: a gateway that relays through a smarthost

## Status

**Accepted**, 2026-09-12. Its first consumer is Zitadel, which cannot send an operator's init email
until it exists, and its first user is Paridyum — tenant zero sending its own mail (ADR-0049). Not
`locked`: it stands up in stages (gateway, then smarthost, then DNS authentication, then environment
routing), each verifiable, and nothing checks the topology yet.

## Context

Two needs converged. The immediate one: Zitadel has no SMTP, so no human can receive the init email
that lets them set a password — the last mile of ADR-0082's operator login. The standing one,
recorded in **ADR-0067**'s accepted costs: "sending as `@paridyum.com`... needs an SMTP relay with
SPF and DKIM configured," and automated/bulk sending should be "isolated on its own subdomain" so a
flagged send never damages the root domain's human mail. The estate always knew it needed a mail
relay; this builds it rather than bolting an SMTP host onto Zitadel alone.

Three facts decide the shape:

1. **PMG is a gateway, not a deliverability service.** Proxmox Mail Gateway is a Postfix-based relay
   that filters and routes; internal hosts send through it (port 26) and it forwards outbound. It
   does not, by existing, make mail arrive. That is a separate problem.

2. **This estate cannot deliver mail directly.** Outbound is a nested Proxmox guest on a home/estate
   network behind Cloudflare and a NetBird mesh — no controlled reverse DNS, an IP with no sending
   reputation, in ranges mailbox providers routinely refuse. Direct-to-MX delivery to Gmail from
   here lands in spam at best and damages the domain at worst. **A smarthost is not optional.**

3. **The estate already runs on Gmail (ADR-0067 D2).** Free consumer Gmail is the long-term mailbox,
   which means an authenticated Gmail SMTP relay (`smtp.gmail.com`) is a smarthost we already have —
   Gmail's deliverability, no new vendor, and consistent with the mail posture already chosen.

So the plane is: **everything sends through PMG (control), PMG relays through a smarthost
(deliverability), and the environment decides whether "relay" means the internet or a sink.**

## Decision

1. **One mail plane, one gateway.** Every system that sends email — Zitadel, the web app, cron,
   alerts — sends through the Proxmox Mail Gateway, never direct-to-MX. One place for routing, policy,
   rate control and observability, and one place a send can be traced. A service that sends email by
   any other path is a defect.

2. **External delivery is through an authenticated smarthost, never direct.** PMG relays outbound to
   a smarthost that owns the sending reputation. The first smarthost is `smtp.gmail.com` on 587 with
   an app password (ADR-0067 D2's Gmail, as a relay). Authenticated relay is configured through PMG's
   Postfix **templating**, not the web UI and not by editing `main.cf` — PMG regenerates the latter.
   The credential is a RUN-0004 Automation secret in SOPS.

3. **Automated mail is on a sending subdomain, not the root (ADR-0067).** System mail sends from a
   sending subdomain (e.g. `mail.paridyum.com`) with its own SPF, DKIM and DMARC, so a flagged
   automated send never degrades the deliverability of a human's `@paridyum.com` mail. The root
   domain's reputation is a human asset and automated sending does not get to spend it.

4. **The environment decides where "outbound" goes.** Which trusted network a message arrives from
   determines its fate — it is not guessed from content:
   - **localhost / dev:** captured in a sink, **never delivered externally.** A developer must not be
     able to email a real candidate from a laptop. Fail closed.
   - **staging:** sunk, or delivered only to an allowlist of internal addresses, on a staging
     sending identity — never to an arbitrary external recipient.
   - **prod:** relayed through the smarthost to the real recipient.
   This is the "localhost / staging / prod email" separation, made a property of the sender's network
   and credential rather than a hope.

5. **PMG runs as a VM on the Proxmox VE, declared in OpenTofu (ADR-0056).** A Debian guest with
   cloud-init installing `proxmox-mailgateway`, an address from the region (`address-registry.yaml`),
   sized within RFC-0023's guest budget. One node now. **PMG clustering is the path to the
   "load-balancing"/HA the operator asked for** — declared as the direction, built when a second node
   is warranted, not before (no capability before need, C-0006 applied to our own infra).

6. **Secrets and keys.** The smarthost credential and the DKIM private key are Automation secrets
   (RUN-0004): SOPS for the relay credential, and the DKIM private key stays on the PMG host,
   encrypted at rest, never in git. The DKIM public key is a DNS TXT record.

7. **Paridyum is the first consumer, unprivileged (ADR-0049 D2).** Zitadel points its SMTP at PMG;
   the operator's init email is the first send. Nothing about Paridyum's use is a path a customer
   tenant could not also take.

## Consequences

- **New OpenTofu resource:** a PMG VM in `infra-terraform/tofu`, with an allocated region address and
  a cloud-init that installs PMG. Applying it creates a machine — a deliberate, operator-watched
  `tofu apply`, not a silent one.
- **DNS is operator work.** No Cloudflare IaC exists, so the sending subdomain's SPF, DKIM and DMARC
  records are added in the Cloudflare dashboard from an exact list this decision's runbook provides.
- **A smarthost credential is owed:** a Gmail app password for the sending account, placed in SOPS.
  It is the one credential the operator supplies; it never transits the agent as plaintext in git.
- **Zitadel SMTP config** points at PMG (host, port, from-address on the sending subdomain), wired the
  way its other config is — SOPS plus a Make target.
- **An interim exists and is named, not hidden:** until PMG is up, Zitadel can point at the smarthost
  directly to unblock a single operator login. It is a stopgap that violates decision 1 (a service
  sending outside the gateway), so it is used knowingly and removed once PMG is live.
- **Deliverability is bounded honestly.** Even through a smarthost, `@paridyum.com`-from-Gmail mail is
  subject to Gmail's alias rules and this domain's DNS authentication; the sending subdomain and its
  DMARC are what keep a bad automated send from spending the human domain's reputation.

## Related

- **ADR-0067** — the mail posture this implements: Gmail mailboxes, the relay it said was owed, the
  sending-subdomain reputation isolation.
- **ADR-0056 / RFC-0023** — OpenTofu declares the machine; the substrate's guest budget it fits in.
- **ADR-0082** — the operator login whose init email is the first thing this plane carries.
- **ADR-0049 / C-0008** — Paridyum sends as the first tenant, unprivileged.
- **REQ-0003** — a classified field never leaves in a log; the same discipline applies to what an
  email body may carry.
