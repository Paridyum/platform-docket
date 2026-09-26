---
id: ADR-0058
title: A substrate host is reached by OpenSSH over the NetBird tunnel, and NetBird's own SSH server is disabled
type: adr
status: accepted
created: 2026-08-23
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["infra-*"]
tags: [ssh, netbird, transport, automation, bootstrap, rfc-0035, adr-0056, c-0003]
breaking: false
enforced_by: []
---

# ADR-0058 — Two SSH transports

## Status

**ACCEPTED** as of 2026-08-23, the day RFC-0035 was accepted and the day its first apply was
attempted and failed.

`enforced_by` is empty. The property worth checking — *a playbook can reach every host in the
inventory without human interaction* — is not a repository check; it is what a run either does
or does not do, and `make check` failing is the signal.

## Context

RFC-0035 decision 4 says a substrate host is **reachable only over the tunnel**, and states
that the property is *"reachable only over the tunnel, not we happen to use NetBird."* It does
not say what speaks SSH at the other end, and on 2026-08-23 that omission stopped the first
apply.

`sitar-as-in-01-ar` had exactly one transport: NetBird's built-in SSH server. It is
**JWT-gated** — every session authenticates through SSO, so there is no key material on any
disk, revocation is central and immediate, and access is MFA-backed. For a person that is
strictly better than an authorized key.

For a playbook it is unusable. Every fresh session requires a browser round-trip;
`ControlPersist` hides that for a few hours and then it returns. **A substrate whose whole
purpose is to reconcile cannot depend on somebody clicking a login**, and RFC-0025 decision 7
— *reconcile, do not command* — is not satisfiable by a transport that requires a human at
the moment of reconciliation.

Three failures on the same day made the case concrete rather than theoretical:

1. The NetBird SSH server's per-peer toggle flipped, and the port accepted TCP while serving
   nothing. There was no second door.
2. NetBird's client writes `/etc/ssh/ssh_config.d/99-netbird.conf` — marked *"Generated
   automatically - do not edit manually"* — matching every NetBird address and hostname and
   routing them through `netbird ssh proxy`. That proxy hung, and the failure surfaced as
   `Connection timed out during banner exchange`, which names the remote sshd. **The remote
   sshd was healthy throughout.**
3. That same generated file sets `PreferredAuthentications password,publickey` — password
   FIRST — so a host with a loaded agent still prompts for a password.

## Decision

**REVISED 2026-08-23, hours after this artifact was first written and before any of it was
applied.** The original decision was that both transports run and neither is disabled. That is
struck out below rather than deleted, because the reasoning was sound and the *facts* were
wrong — and a reader who only sees the conclusion learns nothing about how it was reached.

> ~~A substrate host runs both transports, and neither is disabled: NetBird SSH for people,
> OpenSSH for automation.~~

**They cannot coexist, and NetBird's SSH server cannot do what was asked of it.**

Two facts, both established by observation on the day:

1. **They compete for one socket.** NetBird's SSH server binds the peer's tunnel address on
   port 22 — the exact address and port `sshd` must bind to satisfy decision D4 of RFC-0035.
   A more specific bind wins, so while NetBird holds it, `sshd` is unreachable over the tunnel
   no matter how it is configured. There is no arrangement in which both serve port 22.
2. **NetBird SSH is JWT-mandatory and offers no key path.** `netbird ssh` fails with
   `request JWT token: rpc error: code = DeadlineExceeded`, and a direct connection negotiating
   `PreferredAuthentications=none` is offered **only** `password` — never `publickey`. Its
   password prompt is the front of an SSO flow and does not check a Unix password at all, which
   is why the operator's correct password was rejected repeatedly.

**So: OpenSSH over the NetBird tunnel, with keys. NetBird's SSH server is disabled.**

**NetBird remains the network, and that is the whole point.** What is disabled is an optional
feature bolted onto the tunnel, not the tunnel. There is still one mesh, one vendor for
connectivity, and no second VPN — the estate did not deviate from NetBird, it stopped using a
sub-feature that could not carry automation.

**OpenSSH binds to the tunnel address only**, resolved from facts at run time rather than
hardcoded — the address has already changed once. Binding to `0.0.0.0` would place it on the
LAN, which RFC-0035 D4 forbids, so the role asserts and fails rather than falling back to the
permissive default. Password authentication is off.

**The authorised key is the one C-0003 already requires for commit signing.** An operator who
can reach a host is therefore an operator whose commits are attributable, which is a property
worth having deliberately rather than by coincidence.

**A consuming workstation pins `ProxyCommand none` for substrate hosts.** The vendor's
generated config is regenerated on upgrade, so the correction belongs in our configuration,
not in theirs.

## The bootstrap, which is not automatable

**Ansible cannot reach a machine with no door.** Enabling the first transport is one command
typed on the host, once per host, and no amount of tooling removes it — it is arithmetic, not
a gap. RUN-0004 and `infra-substrate`'s SOP carry the procedure.

Recorded because the alternative is worse: a bootstrap that *looks* automated but silently
depends on a pre-existing key is a bootstrap that fails on the first genuinely new machine,
at the moment nobody is expecting it.

## Alternatives considered

**NetBird SSH alone.** One transport, no keys anywhere, best-in-class human access. Rejected
on the day it failed three ways: automation cannot use it, it offers no key path at all, and
the machine had no other way in when its per-peer toggle flipped silently.

**OpenSSH alone, disabling NetBird SSH.** ✅ **This is now the decision.** It was rejected in
the first draft of this artifact on the grounds that it *"discards a genuinely better mechanism
for humans — SSO with MFA and central revocation beats an authorized key, and losing it to
tidiness would be trading a security property for a smaller diagram."*

That reasoning was correct and the conclusion was still wrong, because it assumed a choice
existed. It did not: the two cannot share port 22, and the mechanism being defended could not
authenticate a key or reliably issue its own token. **The security property was not traded for
tidiness — it was unavailable.** What replaces it is a key held in a password manager's agent,
behind a WireGuard tunnel that must itself be authenticated to enter.

**A dedicated automation account with its own key.** Correct at scale and the obvious next
step when more than one person operates the estate. Deferred: with one operator it adds an
account to manage and no isolation, since that operator already holds the key that would
create it. Revisit when RFC-0035 D2's service-account reasoning extends to human access.

**Password authentication for automation, with the password from SOPS.** Would have worked
immediately and needed no key distribution. Rejected because it requires `sshpass`, which is
absent from Homebrew core for the reasons that make it a bad idea, and because a password
prompt reachable from anything that can route to the tunnel is a brute-force surface that a
key does not have.

## Consequences

**Accepted costs**

1. **Two transports to keep working**, and the failure of either is invisible until it is
   needed. Mitigated only by both being exercised — automation uses one every run, a person
   uses the other.
2. **A key now exists on the host.** NetBird SSH's best property was that none did. The key is
   held in a password manager's agent rather than on disk, which narrows the exposure without
   closing it.
3. **The vendor's SSH config is fought rather than accommodated.** `ProxyCommand none` will
   need re-checking when NetBird changes what it generates, and nothing warns when that
   happens.
4. **One manual step per host, forever.** Named above rather than hidden.

**Gained**

- A playbook can run unattended, which is the precondition for RFC-0025 D7 meaning anything.
- The host stops having a single point of entry.
- The failure mode that cost a day — an error naming the wrong component — is written down.

## Related

- **RFC-0035** — D4, the tunnel-only rule this implements and does not widen
- **ADR-0056** — Ansible; **RFC-0025 D7** — reconcile, do not command, which this makes possible
- **RFC-0023 D4** — the host is a variable, applied here to its address
- **C-0003** — the signing key this authorises, and why that is deliberate
- **RUN-0004** — secrets, and the per-host bootstrap it shares
- **REQ-0001**, **ADR-0045** — no credential is stored for this; the key is in an agent
