---
id: RUN-0008
title: Standing up the mail plane — PMG, a smarthost, and Zitadel's first send
type: runbook
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [mail, smtp, proxmox-mail-gateway, smarthost, dkim, spf, dmarc, zitadel, adr-0083, adr-0067, run-0004]
breaking: false
enforced_by: []
sources:
  - url: http://download.proxmox.com/debian/pmg
    title: "Proxmox Mail Gateway APT repository (pmg-no-subscription)"
    retrieved: 2026-09-12
    supports: >
      The repository line this runbook installs — suite `trixie`, component
      `pmg-no-subscription`. Declared because the body cites it directly and RFC-0020 requires an
      external fact to carry where it came from: this is a vendor-controlled endpoint whose
      suite and component names change with each Debian release, and a runbook naming them is a
      snapshot rather than a derivable truth. HTTP rather than HTTPS deliberately — the gotcha
      section explains why, and it is a property of this endpoint, not a preference.
  - url: https://pve.proxmox.com/wiki/Package_Repositories
    title: "Proxmox — Package Repositories"
    retrieved: 2026-09-12
    supports: >
      The published sha256 of `proxmox-archive-keyring` that this runbook verifies the extracted
      keyring against (`136673be…`), and the documented repository layout the line above follows.
      Retrieved 2026-09-12 and recorded in `research/2026-09-12-a8266409.jsonl` as a `fetched`
      entry — this citation is the reconciliation RFC-0027 asks for, not a date typed from
      memory.
---

# RUN-0008 — Standing up the mail plane

The procedure behind ADR-0083. It brings up the estate's one mail gateway, points it at a smarthost
for real delivery, authenticates the sending domain, and makes Zitadel send through it. Do the stages
in order; each is verifiable before the next.

## What only the operator can supply

Three inputs never come from the agent, and the runbook stops for each:

1. **A smarthost credential.** A Gmail **app password** for the sending account (ADR-0067 D2's Gmail
   as a relay). Generate it in the Google account (2-Step Verification → App passwords). It is a
   RUN-0004 Automation secret → SOPS, not the agent's to hold.
2. **DNS records.** SPF, DKIM and DMARC on the sending subdomain, added in the Cloudflare dashboard
   (no DNS IaC exists). Stage 3 prints the exact records.
3. **The `tofu apply` that creates the VM.** Creating a machine is watched, not silent (ADR-0056).

## The interim, named honestly

If an operator login is needed **before** the gateway is up, Zitadel may point at the smarthost
directly (Stage 4 with `SMTP_HOST=smtp.gmail.com`). This is a stopgap that breaks ADR-0083 decision 1
(a service sending outside the gateway); it unblocks one login and is removed once PMG is live.

## Stage 1 — the PMG guest

PMG runs as a Debian VM on the Proxmox VE (`pve-staging`, 10.1.1.2), address `10.1.1.31`
(`address-registry.yaml`, `pmg-staging`, vmid 1031), sized ~2 GiB / 2 vCPU / ~16 GiB within
RFC-0023's guest budget.

As built, 2026-09-12 — PMG **9.1 on Debian 13** (PMG 9 is trixie-based):

- The **Debian 13 genericcloud** image is placed on `local:import/` (checksum-verified) — mirrored how
  the Talos image got there; owed as a substrate `debian_image` role, done by hand once for now.
- The VM is declared in `infra-terraform/tofu` (`proxmox_virtual_environment_vm.pmg`, gated on
  `pmg_enabled`), disk `import_from` the Debian image, native cloud-init (`user_account` + the
  operator key + static `10.1.1.31/24`) — **no snippet**, because `local` has no `snippets` content
  type. `make pmg-up` (operator-watched — creating a VM is refused by the auto-approve classifier).
- **PMG is installed over SSH, not by cloud-init** (ADR-0056: the substrate ends at the host; this is
  a guest). Add the repo `deb [signed-by=…] http://download.proxmox.com/debian/pmg trixie
  pmg-no-subscription` — **HTTP, not HTTPS** (see the gotcha below) — with the keyring extracted from
  the `proxmox-archive-keyring` .deb and **verified against the wiki's published sha256**
  (`136673be…`), then `apt-get install -y proxmox-mailgateway`.
- Give it **4 GiB** (`qm set <vmid> --memory 4096` then a **host-side** `qm reboot` — a guest reboot
  does not re-read QEMU memory), and install `qemu-guest-agent` (the cloud image lacks it).
- Verify: `https://10.1.1.31:8006` answers 200, `ss -lnt` shows 25/26/8006, and
  `systemctl --failed` is empty.

## Stage 2 — the smarthost (real delivery)

PMG does not deliver to the internet directly from this estate — no controlled PTR, an unproven IP
behind Cloudflare and NetBird (ADR-0083 D2). It relays through the smarthost.

**The relay is Gmail on port 587 (submission), deliberately independent of port 25.** Outbound 25 is
the direct-to-MX path, and it is commonly blocked on a VPS — so relying on it is fragile even where it
is open. Submission (587, STARTTLS + auth) and SMTPS (465) are what the smarthost uses, and both are
confirmed reachable from the gateway (`smtp.gmail.com:587` returns a `220 … ESMTP` banner). Whether
port 25 is open or blocked on any egress path never affects sending, because nothing here uses it.

As built, 2026-09-12 — a Gmail app password in `infra-terraform/secrets/mail-smarthost.sops.yaml`
(keys `smarthost_host/port/user/password`), and PMG configured over SSH:

1. **`sasl_passwd`**: `[smtp.gmail.com]:587 <gmail>:<app-password>` in `/etc/postfix/sasl_passwd`
   (mode 600), then `postmap`. **Strip the spaces** from the app password (`abcd efgh …` → 16 chars)
   so the map parses cleanly.
2. **Template override, not the web UI** (the UI has no authenticated relay) and never `main.cf`
   directly (PMG regenerates it). Copy `/var/lib/pmg/templates/main.cf.in` to
   `/etc/pmg/templates/main.cf.in` and append:
   ```
   relayhost = [smtp.gmail.com]:587
   smtp_address_preference = ipv4
   smtp_sasl_auth_enable = yes
   smtp_sasl_password_maps = hash:/etc/postfix/sasl_passwd
   smtp_sasl_security_options = noanonymous
   smtp_sasl_mechanism_filter = plain, login
   smtp_tls_security_level = encrypt
   ```
   Then `pmgconfig sync` and `systemctl reload postfix`.
3. `apt-get install -y libsasl2-modules` — without it the smtp client has no PLAIN/LOGIN and fails
   with **"No worthy mechs found"**.
4. Trusted networks (already done in Stage 1's follow-up): the estate net `10.1.1.0/24` and the pod
   CIDR `10.2.0.0/17` in `/config/mynetworks`, so the cluster may relay.

Three gotchas, each cost a failed send and are recorded in **When it breaks**: `relayhost` is the
OUTBOUND smarthost — PMG's `relay` setting is the INBOUND relay to a downstream mailserver and does
nothing for outbound; the SASL client modules must be installed; and IPv6 to Gmail is unreachable
here, so prefer IPv4.

- Verify (done): a `sendmail` test from the gateway delivered with
  `relay=smtp.gmail.com[…]:587 … status=sent (250 2.0.0 OK … gsmtp)` and an empty queue.

**Now: two providers, sender-routed (live 2026-09-12).** PMG holds BOTH smarthosts and routes by the
envelope sender (ADR-0085):

- **Resend** for the domain — `@mail.paridyum.com` / `@paridyum.com` → `[smtp.resend.com]:587`. Resend
  DKIM-signs `paridyum.com`, so system mail sends as `noreply@mail.paridyum.com`, DMARC-aligned.
- **Gmail** for the Gmail identity — `@gmail.com` → `[smtp.gmail.com]:587`. Gmail cannot send the
  domain (it would rewrite/reject), so it is not a failover for domain mail — it carries gmail-sender
  mail, and is the hot standby you flip the default `relayhost` to if Resend ever fails.

The config on PMG:
- `/etc/postfix/sasl_passwd` — one line per provider, keyed by relay host (creds from the two SOPS
  secrets `secrets/mail-smarthost-{resend,gmail}.sops.yaml`), then `postmap`.
- `/etc/postfix/sender_relay` — `@domain [host]:port` per route, then `postmap`; wired as
  `sender_dependent_relayhost_maps` in the template. Default `relayhost = [smtp.resend.com]:587`.
- `pmgconfig sync` + `systemctl reload postfix`. Verify a route with
  `postmap -q @mail.paridyum.com hash:/etc/postfix/sender_relay`.

**Never point PMG at an unverified provider** (Resend test-mode 550s until its domain shows verified);
keep the previous smarthost until the new domain is confirmed.

## Stage 3 — authenticate the sending domain (DNS, operator)

Automated mail sends from the **sending subdomain** `mail.paridyum.com`, never the root (ADR-0067),
so a flagged send never spends the human domain's reputation. Add in Cloudflare:

- **SPF** (`mail.paridyum.com` TXT): `v=spf1 include:_spf.google.com -all` (Gmail relay is the sender).
- **DKIM**: generate a key on PMG (or use Gmail's per-account DKIM if sending as the Gmail identity);
  publish the public key as the `<selector>._domainkey.mail.paridyum.com` TXT record. Private key
  stays on the PMG host (RUN-0004).
- **DMARC** (`_dmarc.mail.paridyum.com` TXT): start `v=DMARC1; p=none; rua=mailto:security@paridyum.com`,
  tighten to `quarantine`/`reject` once the aggregate reports are clean.
- Verify: `dig +short TXT mail.paridyum.com` and a test send scored at [mail-tester.com] ≥ 9/10.

## Stage 4 — Zitadel sends through PMG  (DONE 2026-09-12)

Zitadel's **instance** SMTP points at PMG on port 26, and every system email flows through it.

- `make zitadel-smtp` (`configure-smtp.py`) adds and activates the config through the Admin API,
  idempotent by description. Host `10.1.1.31:26`, `tls: false` (the hop is internal on the trusted
  estate network and carries no credential — PMG needs none from a trusted network, and the TLS that
  matters is PMG → Gmail on 587). **No user/password**: the cluster is in PMG's `/config/mynetworks`,
  so Zitadel relays unauthenticated to PMG; the only credential in the plane is the Gmail app password
  on PMG. Interim sender `rohanbatraind@gmail.com` (Gmail rewrites a non-alias sender anyway; becomes
  `noreply@mail.paridyum.com` after Stage 3's alias + DNS).
- The operator user is declared in `organizations.yaml` and created by `make org-provision`; creating
  a human with an unverified email and no password puts them in `USER_STATE_INITIAL` and Zitadel sends
  the **init email** — verified live: `client=…[10.1.1.21] → PMG → relay=smtp.gmail.com[…]:587
  status=sent`. The operator clicks the link, sets their password into Bitwarden, logs in with login
  name `rohan`. **ADR-0082's last mile is done.**
- Gotcha (recorded below): the user-grant search is `POST /users/grants/_search` with a `userIdQuery`,
  not `/users/{id}/grants/_search` (405).

## Stage 5 — environment routing (localhost / staging / prod)

ADR-0083 D4. The environment is the sender's network and identity, not a guess.

- **localhost / dev:** never delivered externally. Point dev at a local sink (MailHog/Mailpit) or a
  PMG transport that discards; a laptop must not be able to email a candidate.
- **staging:** sink, or a PMG recipient allowlist of internal addresses, on a staging from-identity.
- **prod:** the smarthost path above.
- Verify each: a dev send is captured and never leaves; a staging send to an external address is
  refused or sunk; a prod send arrives.

## When it breaks

- **A newly-created Zitadel virtual instance sends no init email, though the tenant instance sends
  fine** → **SMTP in Zitadel is PER INSTANCE.** `configure-smtp.py` sets the *tenant* instance's
  sender via the admin API; a second virtual instance (e.g. `id.paridyum.com`, ADR-0086) created
  afterwards has **no sender of its own**, so its owner's init email is never sendable — not lost,
  never sent, and nothing about the working tenant mail changed. It looks like sending broke; it did
  not. Two independent things must both hold, and the deadlock between them is the trap: (1) the
  instance needs SMTP to send the init email, and (2) configuring an instance's SMTP needs a login
  *inside* it, which needs that same init email. **Fix:** give Zitadel a cluster-wide
  `DefaultInstance.SMTPConfiguration` (values.yaml — the PMG sender, no credential, safe in the
  ConfigMap), `make zitadel-install`, then `make zitadel-instance-recreate` so the instance is
  reborn *inheriting* a sender and its owner's init email finally goes out. (Found 2026-09-13
  creating the control-plane instance; the init to `rohan@paridyum.com` never arrived because the
  new instance had no SMTP.)
- **The instance's owner address does not receive** → `@paridyum.com` mail is received by
  *forwarding* now (ADR-0088, Cloudflare Email Routing → a primary inbox), not a hosted mailbox. If
  the forwarding rule for that address is not live, a perfectly-sent init email still lands nowhere.
- **Mail lands in spam** → SPF/DKIM/DMARC alignment (Stage 3) or the from-address is not on the
  sending subdomain. Score it at mail-tester before blaming PMG.
- **PMG rejects the relay** → the sender's network is not in Mail Proxy → Networks (Stage 2).
- **Gmail relay refuses auth** → the app password, not the account password; 2-Step Verification must
  be on; the from-address must be one Gmail is allowed to send as.
- **A 403 with code 1010 from any Zitadel API call** → Cloudflare blocking a default User-Agent, not
  a credential (see the Zitadel SOP).
- **`apt` to download.proxmox.com fails TLS ("no alternative certificate subject name")** → the host
  resolves to an IPv6 CDN mirror that serves a cert for `enterprise.proxmox.com`. Use the **HTTP**
  repo — packages are GPG-signed, so integrity is the key's job, not TLS. (`Acquire::ForceIPv4=true`
  also helps for the signature DB.)
- **The trixie signing key 404s at `proxmox-release-trixie.gpg`** → it is not published there; it
  ships in the `proxmox-archive-keyring` .deb in the repo pool. Extract
  `/usr/share/keyrings/proxmox-archive-keyring.gpg` from it and verify its sha256 against the wiki
  before trusting it.
- **An interrupted `apt install proxmox-mailgateway` leaves it half-configured** → `grub-pc` waits on
  an install-device prompt (preseed `grub-pc/install_devices` to the boot disk) and the PMG rule DB
  may be missing. Finish with `dpkg --configure -a`; confirm `Proxmox_ruledb` has tables and
  `pmg-smtp-filter` is active.
- **`pmg-smtp-filter` exits 255 / freshclam services fail** → too little RAM (2 GiB); clamd needs
  ~1.5 GiB. Give the VM 4 GiB. Remember a memory change needs a **host-side** `qm reboot` — a guest
  reboot keeps the old allocation.
- **`hostname -f` reverts to the short name after a reboot** → cloud-init's `update_etc_hosts`
  rewrote `/etc/hosts`. Disable that module and pin the FQDN line (Stage 1).
- **Outbound mail goes direct-to-MX (port 25) and Gmail bounces it "not authorized to send
  directly"** → you set PMG's `relay` (the INBOUND relay to a downstream mailserver), which drives
  `relay_transport` and is not used for outbound. The outbound smarthost is `relayhost` in the
  template. Delete the `relay`/`relayport`/`relaynomx` config and set `relayhost = [smtp.gmail.com]:587`.
- **`SASL authentication failed … No worthy mechs found` / `no mechanism available`** → the Cyrus
  SASL client plugins are missing. `apt-get install -y libsasl2-modules`.
- **`connect to smtp.gmail.com[<ipv6>]:587: Network is unreachable`** → the VM has no IPv6 route and
  postfix tried the AAAA first. `smtp_address_preference = ipv4` in the template.
- **`405 Method Not Allowed` on `POST /users/{id}/grants/_search`** → wrong path. User-grant search is
  the collection endpoint with a query: `POST /management/v1/users/grants/_search` body
  `{"queries":[{"userIdQuery":{"userId":"…"}}]}`. The *create* is per-user (`/users/{id}/grants`).
- **Resend `550 … domain is not verified` / `You can only send testing emails to your own address`**
  → the account is in test mode until a domain is verified; auth and relay are fine. Verify the
  sending domain in Resend (add its DNS records to Cloudflare) before pointing PMG at it — until then
  keep the smarthost on Gmail. Switching PMG to an unverified provider silently breaks real sends.
- **Zitadel logo upload `405`** → logos are the ASSETS API (`/assets/v1/org/policy/label/logo/dark`),
  not the management API; SVG is accepted (ADR-0084).
