---
id: RFC-0035
title: A substrate is a guest on a host it does not trust, and least privilege is a user rather than a habit
type: rfc
status: accepted
created: 2026-08-23
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [hardening, least-privilege, substrate, kvm, nested-virtualisation, apparmor, luks, systemd, rfc-0023, req-0015]
breaking: false
enforced_by: []
sources:
  - url: https://thehackernews.com/2026/07/16-year-old-linux-kvm-flaw-lets-guest.html
    title: 16-Year-Old Linux KVM Flaw Lets Guest VMs Escape to Host on Intel and AMD x86 Systems
    retrieved: 2026-08-23
    supports: >
      CVE-2026-53359 ("Januscape") is a KVM shadow-MMU guest-to-host escape, present since
      August 2010, fixed by commit 81ccda30b4e8 merged to mainline on 2026-06-19. Exploitation
      requires TWO conditions together: root inside the guest, and nested virtualisation
      exposed by the host. Both halves matter to decision 6 — the second is what this estate
      controls, and the first is what stage B0 denies.
  - url: https://tuxcare.com/blog/zapscape-cve/
    title: CVE-2026-64561 Zapscape KVM Flaw Explained
    retrieved: 2026-08-23
    supports: >
      A second 2026 KVM guest-to-host escape carrying local root. Its stated worst case is
      "a KVM-based environment in which mutually untrusted users receive virtual machines with
      nested virtualization" — which is the configuration RFC-0025 stage B1 creates and stage
      B0 does not. Recorded because two independent instances in one quarter is a rate, not an
      incident, and decision 6 is written against the rate.
  - url: https://libvirt.org/drvqemu.html
    title: libvirt — QEMU/KVM/HVF hypervisor driver
    retrieved: 2026-08-23
    supports: >
      sVirt confinement of QEMU activates only when a MAC layer is available and a profile for
      the libvirt daemon is loaded. On a host where AppArmor is compiled in but disabled, the
      confinement silently does not happen — there is no error, only an absence. Decision 3.
  - url: https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html
    title: systemd.exec — sandboxing directives
    retrieved: 2026-08-23
    supports: >
      ProtectSystem=strict, ProtectHome, PrivateTmp, NoNewPrivileges, CapabilityBoundingSet and
      SystemCallFilter are unit-level controls applied by the service manager rather than by the
      service, so they hold whether or not the service cooperates. systemd-analyze security
      scores a unit's remaining exposure from 0 to 10, which is what makes decision 5 measurable
      rather than aspirational.
---

# RFC-0035 — A substrate is a guest on a host it does not trust

## Status

**ACCEPTED** by the operator on 2026-08-23, proposed the same day. Opened because RFC-0023 put
the platform substrate on an operator's personal working machine and left the consequence as an
intention rather than a control.

`enforced_by` is empty and this is deliberately **not** `locked`. Under ADR-0002 `locked` is
reserved for what a check can mechanically reject, and `tool-forge` cannot inspect a host it does
not run on. Decision 8's guest-side check is the one rule here a machine could enforce; until it
exists, every decision below is upheld by `infra-substrate`'s playbook and by review.

**The two decisions that accept risk were accepted knowingly**: decision 6 permits nested
virtualisation while two 2026 KVM escapes exist in that class, bounded by the absence of an
untrusted guest at stage B0; decision 7 makes disk encryption a precondition on a laptop that
has none today.

## Problem

RFC-0023 decided the development substrate runs on `sitar-as-in-01-ar`, and its own accepted
cost 3 states the position exactly:

> *"No availability of any kind. One machine, one power supply, one network connection, in a
> residence. **Everything on it is disposable and must be treated that way — which is a
> discipline, not a property, until something enforces it.**"*

Nothing enforces it. An audit of the machine on 2026-08-23 found:

| Control | State |
| --- | --- |
| Disk encryption | **none** — no LUKS anywhere, on a laptop, with `/home` and a 3.6 TB personal partition in the clear |
| MAC (AppArmor) | module present, **`enabled=N`** — so nothing would confine a QEMU process |
| Host firewall | **none** — nftables, iptables, firewalld and ufw all inactive |
| Secure Boot | disabled; a TPM is present at `/dev/tpm0` and unused |
| `kptr_restrict` | `0` — kernel pointers readable by unprivileged users |
| Accounts | one human account, in `wheel`; no service account exists |
| Also running | a display manager, Bluetooth, a local model server, and **a web browser** |

The last row is the problem in one line. **The machine that is to host the platform substrate is
simultaneously browsing the web**, and a browser is the largest attack surface a desktop has.

RFC-0023's own alternatives section says *"the machine is an operator's working environment
first"*, so this is a deliberate arrangement rather than an oversight. What is missing is the set
of controls that makes the arrangement survivable, and a written statement of which direction
trust flows.

## Constraints

- **C-0008 / ADR-0049** — nothing runs outside a tenant, and no operator bypass exists. A control
  that only the operator can satisfy is not a control this estate may rely on.
- **C-0004** (locked) — irreversible and governance-altering operations require a human.
- **REQ-0015** — `accepted` and already FIRED. A design accepting untrusted input states what
  could go wrong, and the org keeps one list of accepted risk. This RFC feeds that list.
- **RFC-0019 D2** — Talos was chosen partly because *"a node nobody can SSH into cannot drift."*
  The same property is unavailable on a general-purpose host, so it must be replaced by controls.
- **RFC-0023 D4 / RFC-0025 D4** — the substrate's location is a variable. Nothing may encode
  "this is sitar", which is also what makes every rule below portable to rented metal.
- **RFC-0025 stage B0** — fleet manager only, **no customer workloads anywhere**. Decision 6
  rests entirely on this.

## Decisions

### 1 — The substrate is a guest. The host is not trusted by it, and cannot be reached from it

The direction is the whole design and it is stated once, plainly:

> **Host → guest is permitted. Guest → host is denied.**

The host's role is hypervisor and nothing else. No Paridyum service runs on the host directly;
every one runs inside a guest. That is what makes RFC-0023's "everything on it is disposable"
true of the substrate rather than merely said about it.

**Consequently no shared-filesystem path exists between guest and host.** No `virtiofs`, no `9p`,
no shared clipboard, no USB redirection, no SPICE agent channel. Every one of those is a
guest→host path, and each is normally added for convenience during exactly the debugging session
where it will be forgotten.

**The mark of correctness: the operator's data is not reachable from a guest even when the guest
is fully compromised.** That is checkable by inspection of the domain XML and is decision 8's
verification.

### 2 — A `paridyum` service account owns the substrate, and it is not the operator's account

A dedicated unprivileged system user, no login shell, no `wheel`, owning the VM storage pool and
the libvirt domains. Its home is mode `0700`.

**The operator's account is not the substrate's account**, and the reason is narrower than
hygiene: the operator's account has a browser, an agent harness with a GitHub PAT, SSH keys and a
shell history. Running the hypervisor as that user makes every one of those reachable from a
QEMU process. The estate already knows the shape of this risk — ADR-0045 risk R3 records
credentials existing before REQ-0001 discharges secret management.

**The `paridyum` user gets no passwordless `sudo`.** Provisioning is a libvirt API call, not a
shell command, which is the same property RFC-0025 D2 requires one level up: *"no path into a
fleet that is not the fleet-manager API."*

### 3 — AppArmor is enabled and enforcing before the first guest starts

> **⚠️ ERRATUM, 2026-08-24 — THIS DECISION IS UNMET ON `sitar-as-in-01-ar`, AND CANNOT BE MET
> THERE.** Appended and dated per ADR-0031; the decision below is unchanged and still binds
> every host that can satisfy it.
>
> **Arch's libvirt has no AppArmor support compiled in.** `strings /usr/bin/libvirtd | grep -c
> apparmor` returns 0, `/usr/lib/libvirt/virt-aa-helper` does not exist, and the package ships
> no profiles. This is not a missing helper or a configuration error — there is no
> configuration that enables it. AppArmor itself is healthy on the host: 225 profiles loaded,
> 147 enforcing.
>
> **This is a collision between RFC-0023 and this decision, and neither anticipated it.**
> RFC-0023 licensed Arch for this host because *"the machine is an operator's working
> environment first"*; this decision assumed a libvirt that could confine. Both were reasonable
> and together they are unsatisfiable.
>
> **What is lost, stated precisely rather than dramatically.** sVirt gives each domain a
> generated profile so one QEMU cannot reach another's disks, and confines QEMU beyond Unix
> permissions. Its primary beneficiary is VM-to-VM isolation, and at stage B0 there is one
> guest and no untrusted tenant — there is nobody to isolate from.
>
> **What still holds, measured on 2026-08-24 rather than assumed:** QEMU runs as the
> unprivileged `paridyum` account; `/home/rohan` is mode 700 and `paridyum` is denied it; the
> operator's NTFS partition is not mounted, so decision 8 holds; `paridyum` is in `kvm` and
> `libvirt` only, with no sudo. The boundary decisions 1 and 8 exist to protect is enforced by
> file permissions and verified.
>
> **TRIGGER — sVirt becomes mandatory the moment a host carries a workload Paridyum does not
> control.** That is the same condition decision 6 uses for nested virtualisation, and it
> fires at RFC-0025 stage B1. It resolves without migrating this host: B1 buys a rented node,
> which is new hardware provisioned fresh, and it is provisioned on a distribution whose
> libvirt confines. `sitar` stays the Arch workstation RFC-0023 licensed, and the operating
> system question resolves exactly where RFC-0023 said it would — *"the moment the machine
> stops being a workstation."*
>
> The acceptance is recorded in `infra-substrate` as `libvirt_confinement_gap_accepted`, which
> defaults to false so that a run stops and a human decides rather than a control silently
> becoming `none`.

libvirt's sVirt confinement of QEMU **activates only if a MAC layer is present**. On this host
AppArmor is compiled in and disabled, so the confinement that is supposed to stop a compromised
QEMU touching the host filesystem is simply absent — and absent without an error, which is the
worst way for a control to be missing.

This is a boot-parameter and a service, and it must be verified by observation
(`aa-status` reporting the libvirt and QEMU profiles loaded) rather than by having set it.

### 4 — The host firewall is default-deny inbound, and the substrate is reachable only over the tunnel

nftables, default-deny inbound, with the loopback and established flows permitted. Management
interfaces bind the NetBird interface, never the LAN and never `0.0.0.0`.

The current SSH arrangement already satisfies this and should be preserved deliberately rather
than by accident: sshd answers on the NetBird address and is closed on the LAN. **The property is
"reachable only over the tunnel", not "we happen to use NetBird"** — RFC-0023 D4's variable rule
applies to the tunnel vendor as much as to the host.

### 5 — Every Paridyum unit is sandboxed by the service manager, and the score is recorded

`NoNewPrivileges=true`, `ProtectSystem=strict`, `ProtectHome=true`, `PrivateTmp=true`, a
`CapabilityBoundingSet` reduced to what the unit actually uses, and `SystemCallFilter=@system-service`.

These are applied by systemd rather than by the service, so they hold whether or not the service
cooperates — which is the property that makes them worth more than an equivalent instruction in a
runbook.

**`systemd-analyze security <unit>` is run and its score recorded in the repository's logbook.**
Not a threshold, deliberately: a number with no argument behind it is REQ-0016's "derived from
taste" defect. A recorded score makes a regression visible at the next change.

### 6 — Nested virtualisation is permitted at stage B0 only, and the trigger is written now

Two guest-to-host escapes were disclosed in one quarter — **Januscape** (CVE-2026-53359, shadow
MMU, present since 2010, fixed 2026-06-19) and **Zapscape** (CVE-2026-64561, escape plus local
root). Both require the same two conditions **together**: root inside a guest, and nested
virtualisation exposed. Zapscape's stated worst case is *"mutually untrusted users receive
virtual machines with nested virtualization."*

RFC-0025 stage **B0 carries no customer workloads**, so the first condition has no holder: the
only occupant of the guest is Paridyum. Nesting Proxmox is therefore acceptable at B0 and its
purpose is precise — to give `svc-fleet` a real Proxmox API to reconcile against, where
performance is irrelevant because nothing is being served.

> **TRIGGER — nested virtualisation is disabled (`kvm_intel.nested=0`) on any host that carries a
> workload Paridyum does not control.** This fires at RFC-0025 stage B1, by that stage's own
> definition, and does not wait for a further decision.

This is recorded as a *rate* rather than an incident. Two independent instances in one quarter is
a reason to write the rule against the class, not to patch and move on.

### 7 — Disk encryption precedes the first secret

The host holds no Paridyum secret until the volume holding it is encrypted at rest. A laptop
without full-disk encryption has no meaningful boundary against physical loss, and this one
currently carries `/home` and a 3.6 TB personal partition in the clear.

**Stated as a precondition rather than a task**, because the ordering is what matters: once a
credential has been written to an unencrypted volume, encrypting the volume afterwards does not
un-write it. That is the same irreversibility argument RFC-0013 D3 was accepted on.

A TPM is present and unused; sealing to it is the obvious implementation and is not mandated
here, because the requirement is encryption at rest and the mechanism is an operational choice.

### 8 — The personal boundary is explicit, and it is verified from the guest side

The operator's data — `/home`, and the NTFS `Storage` partition — is **never mounted into a
guest and never mounted while a guest is running**.

Verification is from the guest, not from the host: a check that runs *inside* a guest and asserts
it can see no host filesystem, reach no host-only address, and resolve no host path. STD-0009's
principle applied to a different boundary — **the denial is the assertion.** A host-side
inspection of the domain XML confirms the intended configuration; only a guest-side check
confirms the achieved one.

## Alternatives considered

**Harden the host and keep running services on it directly.** Cheapest, and it is what happens by
default. Rejected because it inverts the trust direction: a browser-bearing desktop would be
inside the substrate's trust boundary, so every desktop compromise is a substrate compromise. The
controls above would still all be worth having; they would just be protecting the wrong thing.

**Proxmox VE on the metal, no Arch, no nesting.** Correct for production, one layer cheaper, and
it removes decision 6 entirely along with both CVEs' precondition. Rejected here for the reason
RFC-0023 already gave — the machine is the operator's working environment first — and recorded
because **it becomes the decision the moment this stops being a workstation**, exactly as
RFC-0023's own alternatives section says.

**A second physical machine for the substrate.** Removes the dual-use problem completely and is
the honest answer. Deferred on cost, not on merit, and it is what RFC-0025 stage B1 buys anyway.
Naming it here means the eventual purchase is a planned step rather than a discovery.

**Disable nested virtualisation now and run Talos directly under QEMU.** Removes both CVEs'
precondition immediately. Rejected because it skips the Proxmox API — which is precisely the
provisioning chain nothing has tested and the part most likely to be wrong (RFC-0023's own
argument against the same shortcut).

## Consequences

**Accepted costs**

1. **Every control here is unenforced.** `enforced_by` is empty and that is honest: `tool-forge`
   cannot inspect a host it does not run on. These are upheld by the SOP and by decision 8's
   guest-side check, which is the only one of them a machine can perform.
2. **A service account and a denied guest→host path make debugging slower.** The convenience
   being removed is real, and it is removed at the moment it is cheap rather than after the first
   incident.
3. **Full-disk encryption on a running machine is a reinstall or a careful in-place conversion.**
   Decision 7 makes it a precondition, so this cost lands before the substrate is useful rather
   than after.
4. **Nested virtualisation stays on until B1**, with two disclosed escapes in the class. The
   mitigation is the absence of an untrusted guest, which is a property of the stage rather than
   of the software, and it therefore expires exactly when the stage does.
5. **The host remains an operator's daily-use machine**, which no control above changes. This is
   the residual risk and it is the reason decision 1 exists.

**Gained**

- The trust direction is written down, so a future convenience that reverses it is visibly a
  reversal rather than a small change.
- The nested-virtualisation risk has a trigger instead of a memory.
- RFC-0023's "discipline, not a property" sentence acquires the controls it was waiting for.

## Rollout

Ordered, because two of the steps are preconditions rather than tasks.

| | Step | Gate |
| --- | --- | --- |
| 1 | AppArmor enabled and enforcing; libvirt/QEMU profiles loaded | `aa-status` shows the profiles — **see the erratum on decision 3: the libvirt half is unattainable on Arch** |
| 2 | nftables default-deny inbound; management binds the tunnel interface | scan from off-tunnel shows nothing |
| 3 | `paridyum` service account; storage pool `0700` under it | no login shell, not in `wheel` |
| 4 | Full-disk encryption | **precondition for any secret on the host** |
| 5 | Guests created with no shared path, minimal devices | domain XML inspection |
| 6 | Guest-side isolation check | the check runs inside a guest and fails closed |
| 7 | systemd sandboxing on every Paridyum unit | score recorded in the logbook |

Undoing this is removing the units and the account; no data migration is involved, which is a
consequence of decision 1 rather than a coincidence.

## Open questions

1. **Does the guest-side isolation check belong to `tool-forge` or to `infra-*`?** It is the only
   mechanically enforceable rule here, so its home decides whether any of this is ever checked.
2. **Does a workstation enter `device-registry.yaml`?** RFC-0021 open question 1 is still open,
   and this RFC gives a host controls without settling whether the host is registered.
3. **Is `sitar-as-in-01-ar` renamed when Proxmox arrives?** RFC-0021 D6 says the platform suffix
   is part of the hostname and a rebuild is a rename. Nested Proxmox on Arch is not a rebuild, so
   the name holds today — but the boundary is undefined.
4. **Does REQ-0001 gate decision 7, or the reverse?** Encryption at rest is a precondition for a
   secret; secret management is what puts secrets there. The two have to be sequenced explicitly.

## Related

- **RFC-0023** — the substrate decision this supplies controls for, and the sentence that asked for them
- **RFC-0025** — stage B0/B1, and D2's "no path in that is not the API", which decision 2 mirrors
- **RFC-0019** — D2's "a node nobody can SSH into cannot drift", unavailable here and replaced by controls
- **REQ-0015** — the risk register this feeds; already fired
- **REQ-0001**, **ADR-0045** — secret management and risk R3, which decision 7 is sequenced against
- **STD-0009** — "the denial is the assertion", the shape decision 8 borrows
- **C-0004**, **C-0008**, **ADR-0049** — protected operations, and no privileged path
- **RFC-0021** — hostnames, and open questions 2 and 3
