---
id: ADR-0085
title: System mail sends as the domain, through a provider that signs it
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra]
applies_to: ["*"]
tags: [mail, smtp, dkim, spf, dmarc, deliverability, transactional-email, adr-0083, adr-0067, run-0008]
breaking: false
enforced_by: []
---

# ADR-0085 — System mail sends as the domain, through a provider that signs it

## Status

**Accepted**, 2026-09-12, and **implemented the same day**: the provider is **Resend**, PMG relays to
`smtp.resend.com:587`, and Zitadel sends as `noreply@mail.paridyum.com` — verified end to end
(Zitadel → PMG → Resend, `status=sent`). Refines ADR-0083 decision 2 (whose "first smarthost is
Gmail" was the bootstrap) for the system-mail lane. Not `locked`: it graduates when a `dig`/mail-tester
check asserts DKIM and DMARC aligned to `paridyum.com`. The Gmail relay (ADR-0083) remains the
documented fallback.

## Context

The mail plane works: Zitadel → PMG → Gmail, delivered. But it sends **as `rohanbatraind@gmail.com`**,
not `noreply@paridyum.com`, and the reason is not fixable at PMG. Free consumer Gmail, used as an
authenticated relay, signs outbound with **DKIM `d=gmail.com`** and the envelope is Gmail's, so for a
`From: noreply@paridyum.com` neither SPF nor DKIM aligns with `paridyum.com` — **DMARC cannot pass**,
and Gmail rewrites a non-alias sender anyway. ADR-0067 D5 recorded exactly this cost when it chose free
Gmail for human mailboxes.

Human mail and system mail are different problems. ADR-0067 keeps humans on Gmail — that stays.
System/transactional mail (a user's init link, a verification, a candidate notification) must send as
the domain and land in the inbox, which requires a sender that can **DKIM-sign `paridyum.com`.** Free
Gmail cannot; the options that can are Google Workspace (which ADR-0067 D5 deliberately declined) or a
**transactional email provider**.

## Decision

1. **System mail sends as `noreply@mail.paridyum.com`, on the sending subdomain (ADR-0067).** The
   subdomain isolates automated-sending reputation from the human root domain, so a flagged system
   send never spends `paridyum.com`'s deliverability.

2. **The smarthost for the system-mail lane is a transactional email provider, not Gmail.** PMG relays
   to a provider (SES, Postmark, Resend, Brevo, Mailgun — operator's choice; a free/low tier covers
   this volume) that verifies `paridyum.com` and **signs it with DKIM**, so `From: noreply@…`,
   `DKIM d=paridyum.com`, and DMARC all align. This is the same PMG mechanism ADR-0083/RUN-0008
   already built — a `relayhost` and an authenticated `sasl_passwd` — pointed at the provider instead
   of Gmail. Gmail relay remains the documented fallback until the provider is live.

3. **Humans stay on Gmail; only system mail moves.** ADR-0067 is unchanged: a person's mailbox is
   still consumer Gmail. This ADR governs what Zitadel and the app *send*, not where people *receive*.

   **PMG keeps both providers, sender-routed** (implemented 2026-09-12): `@mail.paridyum.com` and
   `@paridyum.com` route to Resend (DKIM-aligned), `@gmail.com` routes to Gmail, via
   `sender_dependent_relayhost_maps`. Gmail cannot send the domain, so it is not a failover for
   domain mail — it carries gmail-identity mail and is the hot standby the default `relayhost` flips
   to if Resend fails. Both credentials live in SOPS (`mail-smarthost-{resend,gmail}.sops.yaml`).

4. **The credential is a SOPS Automation secret; the DNS is the operator's.** The provider's SMTP
   credential lives in SOPS on PMG (RUN-0004), exactly as the Gmail one does. The SPF, the provider's
   DKIM record, and DMARC on the sending subdomain are published in Cloudflare — no DNS IaC exists, so
   this is an operator step from an exact record list the runbook provides.

5. **DMARC starts at `p=none` and tightens.** Publish `p=none` with aggregate reporting first; move to
   `quarantine` then `reject` once reports show the provider's mail aligning cleanly.

## Consequences

- **Operator inputs (unavoidable):** choose a provider, create the account, verify `paridyum.com`
  there. Everything after is ours: the DNS record list to paste into Cloudflare, PMG's `relayhost` +
  `sasl_passwd` repointed at the provider (RUN-0008 Stage 2, provider instead of Gmail), and Zitadel's
  sender set to `noreply@mail.paridyum.com` (`configure-smtp.py`).
- **Until then, nothing breaks.** ADR-0083's Gmail relay keeps delivering; the only difference is the
  visible sender. This ADR is the plan and the DNS/records prep, executed the moment the provider
  account exists.
- **The estate gains a real transactional-mail identity** — inbox-placement for candidate and operator
  mail — without moving humans off Gmail or self-hosting outbound (which port 25 and IP reputation
  forbid, ADR-0083 D2).

## Related

- **ADR-0083** — the mail plane and its bootstrap Gmail smarthost this refines for system mail.
- **ADR-0067** — human mailboxes stay Gmail; the sending-subdomain reputation isolation this reuses.
- **RUN-0008** — the standup runbook; the provider becomes Stage 2's smarthost and Stage 3's DNS.
