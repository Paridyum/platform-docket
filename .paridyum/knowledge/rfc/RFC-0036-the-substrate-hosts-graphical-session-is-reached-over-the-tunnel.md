---
id: RFC-0036
title: The substrate host's graphical session is reached over the tunnel, by a service that owns no port on the LAN
type: rfc
status: proposed
created: 2026-08-31
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra]
applies_to: ["infra-*"]
tags: [remote-desktop, sunshine, moonlight, wayland, hyprland, netbird, nftables, apparmor, rfc-0035, adr-0058]
breaking: false
enforced_by: []
sources:
  - url: https://docs.lizardbyte.dev/projects/sunshine/latest/md_docs_2configuration.html
    title: Sunshine — configuration reference
    retrieved: 2026-08-31
    supports: >
      `origin_web_ui_allowed` accepts `pc` (loopback only) or `lan`, and defaults to `lan`;
      `upnp` defaults to off and opens the router when on. Both are decisions 3 and 4.
  - url: https://docs.lizardbyte.dev/projects/sunshine/latest/md_docs_2linux.html
    title: Sunshine — Linux setup, capture methods and capabilities
    retrieved: 2026-08-31
    supports: >
      Wayland capture is chosen per compositor: KMS capture needs CAP_SYS_ADMIN on the
      binary, wlroots capture needs none. The documented `setcap` step is specific to the KMS
      path and is not a general install step — decision 5.
---

# RFC-0036 — Reaching the graphical session

## Status

**`proposed`.** It records a change that has been applied to one host; it becomes `accepted`
when a human has read it, and it does not graduate to an ADR unless a second host needs the
same capability.

## Problem

The substrate host is a laptop that is also the operator's daily working environment
(RFC-0023). Every existing route to it is a terminal: ADR-0058 settled OpenSSH over the
NetBird tunnel, and that is sufficient for everything Ansible does and for everything a
person does at a shell.

It is not sufficient for the work that is not a shell. Reading the Hyprland session — a
browser holding a console the estate has no CLI for, a graphical installer, the display
output of a guest — currently requires being in the same room as the machine. The observable
cost is that a class of work is blocked whenever the operator is not physically present, and
the machine is a laptop precisely because that is often.

Fixed when the operator can see and drive the host's graphical session from another machine,
over the same tunnel that already carries SSH, with no port reachable from the LAN or the
public Internet and no new credential stored in this estate.

## Constraints

- **RFC-0035 D4** (`accepted`) — default-deny inbound; the substrate is *"reachable only over
  the tunnel"*. This is the binding constraint and it is not weakened here: the property is
  the transport, not the protocol that rides it.
- **RFC-0035 D1** — guest → host is denied. A remote-desktop service is **host → operator**
  and touches no guest boundary; nothing here creates a shared path into a guest, and no
  clipboard or USB path is opened between a guest and the host.
- **RFC-0035 D3** — AppArmor enforcing. The new service is host software, so its confinement
  state is a fact that must be recorded rather than assumed. See decision 6.
- **ADR-0058** — NetBird is the network. This adds a service to that network; it does not add
  a second one.
- **C-0004 / STD-0016 rule 5** — no credential is stored. Sunshine's own password therefore
  cannot be set by the playbook, and decision 4 says who sets it and where.
- **ADR-0053** — a pin is exact and current. The package is a pinned release artifact with a
  recorded digest, not a floating AUR build.

## Decisions

### 1 — The service is Sunshine, installed from the project's own signed release for Arch

Sunshine publishes a `pkg.tar.zst` per release. It is installed by URL at a pinned version
with a recorded `sha256`, and verified before `pacman -U` sees it.

**Not the AUR.** `yay -S sunshine` builds whatever the PKGBUILD points at today, invokes
`sudo` from inside a helper the playbook does not control, and has no digest to record. That
is a floating dependency wearing a version number, and ADR-0053 exists because of exactly
that shape.

### 2 — It runs as a user service in the operator's session, never as root and never system-wide

`systemctl --user`, in the operator's own session, owning only what that session owns.

A system service would have to run as root to reach a seat it does not own, and the thing it
would then hold is the operator's screen, keyboard and microphone — the one process on the
machine where a privilege escalation is worth the most. The capture APIs are session APIs;
running in the session is both the least privilege and the only arrangement that works
without fighting `logind`.

**Consequence, stated rather than discovered later: no graphical session, no remote desktop.**
The unit is `WantedBy=graphical-session.target`, so a host sitting at its display manager has
nothing to capture. This is correct — there is no screen yet — and it means the first login
after a reboot is still a physical act unless autologin is separately decided. It is not
decided here.

**And the target is not free on a bare Hyprland login.** `graphical-session.target` is reached
when the session is started under a manager that raises it — `uwsm` in Hyprland's case. Started
from a plain session entry, the compositor runs, the target never activates, the enabled unit
never starts, and the failure presents as *"Sunshine is enabled and does nothing"* with nothing
in its log, because there is no log. Verified by observation rather than assumed: started with
no session at all, the service runs, finds no display and no encoder, and says so — which is the
same symptom the wrong session entry produces and is the reason this paragraph exists.

### 3 — Sunshine's web UI answers on loopback only, and UPnP is off

`origin_web_ui_allowed = pc` and `upnp = off`, both written by the playbook rather than left
at their defaults, because one of the two defaults is wrong for this estate.

The web UI is the service's whole control plane: pairing, applications, and the settings that
would undo this RFC. `lan` — the default — would serve it to anything that can route to the
machine. It is reached instead through an SSH port-forward over the tunnel, which reuses
ADR-0058's transport and adds no second authentication surface.

**Said precisely, because the imprecise version invites a wrong conclusion: the socket still
binds `0.0.0.0`.** `ss -ltn` shows the ports on every interface, and `origin_web_ui_allowed`
is an origin check inside the application rather than a bind address. So the property this
decision delivers is *the control plane refuses a request that did not come from loopback* —
and what keeps the port itself off the LAN is decision 7's firewall, not this setting. Two
controls, neither sufficient alone, which is why the role asserts the other one exists.

**UPnP is the sharper edge.** Its entire function is to ask the router to expose these ports
to the public Internet. A remote-desktop port on a residential IP is found by a scanner in
hours, and *"do not port-forward"* is not a preference — the estate has one route in and this
RFC does not add a second.

### 4 — The password is set by a person, through loopback, and is never in this estate

The playbook installs and configures. It does not create the Sunshine credential, and there
is no encrypted `group_vars` entry for it.

`make secret` exists (ADR-0057) and would technically hold it. It should not: RUN-0004's table
sorts a secret by who reads it, and nothing here reads this one unattended. It is an
**operator** credential — a person types it into a web UI at setup and at a re-pair — so it
goes to the password manager, which is where the SSH key already is.

**And an encrypted commit would be worse than no commit**, for the reason RUN-0004 gives:
every historical version of a committed file stays readable to anyone who later obtains the
age key, so a credential that could have lived in a password manager would instead be given an
immortal copy for no gain.

The account is **`sunshine-admin`**, per STD-0017 rule 3. Every shorter answer was taken:
`rohan` is a Unix account on the same host, `paridyum` is RFC-0035 D2's account and must keep
meaning *holds no privilege*, `operator` names a paying customer in GLO-0001, and `admin` says
nothing. The credential is generated alphanumeric — not for entropy, but because the reset path
is a shell command and a quote character in it turns recovery into a quoting bug.

**Sunshine has exactly one web-UI credential and cannot represent a second account.** That is a
STD-0017 rule 4 gap, recorded here as that rule requires: a second person needing access today
would have to be handed this password, and the estate should know that rather than discover it.
It is acceptable at one operator and stops being acceptable at two.

Named per RUN-0004's convention: **`sunshine — <RFC-0021 hostname>`**, folder
`Paridyum/substrate`, `governed_by: RFC-0036 D4`. The URI is deliberately **not** set to
`https://localhost:47990` — autofill matches host and port, and this control plane is reached
through a forward that any other project's local server could be sitting behind on another
day.

### 5 — The capture method is chosen from what the compositor offers, and the capability is granted only if that choice needs it

Hyprland is wlroots-derived, and the two capture paths are not equivalent in what they cost:

| Path | Needs | When it is right |
| --- | --- | --- |
| wlroots screencopy | nothing | the compositor exports it |
| KMS | `cap_sys_admin+p` on `/usr/bin/sunshine` | it does not |

**Every third-party guide opens with the `setcap` line.** Applying it blindly grants a
capability that is one of the broadest in Linux — mount, `perf`, arbitrary device access — to
a binary that may not have needed it. So the role defaults to granting nothing, detects what
the running compositor actually offers, and grants the capability only when the KMS path is
the one in use, as a variable an operator sets deliberately.

`ptrace` and `sys_ptrace` appear in some of the same guides. Neither is granted, ever.

### 6 — Sunshine is unconfined by AppArmor, and that is recorded rather than assumed

Arch ships no AppArmor profile for Sunshine and this RFC does not write one. The service runs
unconfined, inside an unprivileged user session, on a host whose other profiles are loaded and enforcing.

**The failure this decision guards against is the one RFC-0035 D3's erratum already recorded
once**: a control that is absent while everything looks configured. `infra-substrate`'s
`docs/MAC-CONFINEMENT.md` is the register — every component that could be confined, whether it
is, and why not. A gap that is written down can be closed; a gap nobody enumerated is found by
an incident.

### 7 — No firewall rule is added, and that is the point

RFC-0035 D4's ruleset accepts the tunnel interface and drops everything else. Sunshine's eight
ports are therefore already reachable from the tunnel and already unreachable from the LAN,
with no rule naming them.

**The role asserts that the default-deny ruleset is in place, and refuses to install if it is
not.** Installing a capture service onto a host with no firewall would put a video stream and
a pairing endpoint on the LAN for as long as it took somebody to notice — and the ordering
that prevents it is a line in `site.yml`, which is exactly the kind of load-bearing detail
that survives being written down and does not survive being remembered.

### 8 — The session owns a virtual output, so the machine has a screen with the lid shut

`roles/laptop` already stops a closed lid from suspending the host. It does not stop the lid
from **disconnecting the panel**: eDP goes away, and with nothing plugged in the compositor has
no enabled output. A compositor with no output produces no frames, and Sunshine's log then says
*"Unable to find display"* — the same sentence it prints when nobody is logged in at all, which
is the reason to remove this as a possibility rather than diagnose it twice.

So one headless output is created at session start and configured at a fixed mode. Which screen
is streamed stays `auto` by default — the physical panel while one is awake, the virtual one when
it is not. Pinning `output_name` to the headless output is the other reasonable choice and gives
a *dedicated* remote desktop that the person at the laptop is not looking at; it is a variable
rather than a decision because the two answers suit different days.

**This is not headless operation.** It removes the dependency on a *monitor*, not the dependency
on a *login* — decision 2 still holds, and unattended reachability across a reboot still needs
the autologin that decision 2 rejects while RFC-0035 D7 is unmet.

### 9 — There is one desktop session, it belongs to the operator, and `paridyum` never gets one

The obvious-looking arrangement — the operator's desktop on one seat, a second desktop for
remote use owned by the `paridyum` account — is **forbidden**, and by an existing decision rather
than a new one.

RFC-0035 D2 created that account so that a compromised hypervisor process reaches nothing of the
operator's: no browser, no agent harness holding a PAT, no SSH keys, no shell history. It holds
`kvm` and `libvirt` membership and `/usr/sbin/nologin`. **Giving it a graphical session with a
capture service attached would hand the account that owns every VM a screen, a keyboard, a
clipboard and virtual input** — which is not a weakening of D2, it is its inverse. The account
whose purpose is to be unable to see the operator's work would become the account that watches
it.

A second *desktop* is a reasonable want and decision 8 satisfies it with a second **screen** in
one session. If a genuinely separate second session is ever needed, it takes a new account named
under STD-0017 — never this one — and a second Sunshine on different ports, and that is a new
decision rather than a variable.

### 10 — On a hybrid machine, the GPU that encodes is the GPU that composites

A laptop with an integrated and a discrete GPU offers a choice, and the wrong one fails in a way
that points at the network.

Frames reach Sunshine as **dmabuf handles owned by the compositor's device**. Importing one into
an EGL context on the *other* device fails with `EGL_BAD_PARAMETER` — logged as
`Couldn't import RGB Image: 0000300C`, then `Could not convert image`, once per attempted frame.
Everything else works: the client pairs, RTSP negotiates, the control stream connects, audio
flows. Only video is absent, and the client says:

> *No video received from host. Check your firewall and port forwarding rules for port(s):
> UDP 47998, UDP 48000.*

**That message is wrong, and it cost most of an evening.** The firewall was verified twice —
once by listening on the exact video port over the tunnel and receiving probes, once by watching
the control stream on the adjacent port work throughout. The host was not blocked from sending;
it had nothing to send.

So: `adapter_name` names the compositor's render node, and the encoder is the one that node
supports. On a hybrid laptop that means the **integrated** GPU and its VAAPI driver, not the
discrete GPU and NVENC — the faster encoder that cannot see the frames is worth less than the
slower one that can.

**Two consequences worth stating.** The libva driver for that GPU must be installed, and it must
be *selected*: a desktop session that exports `LIBVA_DRIVER_NAME` for the discrete card (as
Hyprland's NVIDIA guidance does) hands Sunshine the wrong driver for the device it was given.
That override belongs to the unit, not the session — the session's value is right for the
session.

**The value is per host, in the inventory, never in `group_vars`.** It names a PCI render node
on one machine's hardware. A second substrate host inherits `auto` and is diagnosed on its own
terms rather than misconfigured by inheritance — the same discipline
`libvirt_confinement_gap_accepted` already follows.

## Alternatives considered

**Waypipe or `ssh -X`.** No new port, no new package, no pairing, and the transport is the one
ADR-0058 already chose. Rejected on capability rather than security: neither carries the
existing session — they run a remote application against a local compositor. What is needed
here is the operator's actual Hyprland desktop, mid-task, including whatever is already open.

**VNC (`wayvnc`).** Genuinely simpler, and a serious contender: one port, one password, no
pairing dance, no capability question. Rejected on what it is for. It is a remote *console* —
software encoded, unaccelerated, and visibly so at 4K. The machine has an RTX 3050 whose
encoder exists for this, and the work this RFC unblocks includes watching output change in
real time.

**RustDesk or a hosted equivalent.** Would work immediately and needs no tunnel at all,
because a third party relays the connection. Rejected on that sentence: it moves the trust
boundary from a WireGuard mesh the estate controls to a vendor's relay, to solve a problem —
reachability — that NetBird already solved.

**Sunshine as a system service with autologin.** The arrangement that makes the host reachable
after an unattended reboot, which decision 2 explicitly does not deliver. Rejected for now
because it requires a session that logs itself in, on a machine whose disk is not yet
encrypted (RFC-0035 D7, unmet). An autologin session on an unencrypted laptop turns theft into
access. Revisit when D7 is met.

## Consequences

**Accepted costs**

1. **A capture service now has the operator's screen, keyboard and clipboard.** That is what
   was asked for, and it means a compromise of this unit is a compromise of the session — not
   merely of a machine. The mitigations are that it holds no privilege the session lacks, and
   that nothing can reach it without first authenticating into the tunnel.
2. **A second thing to keep working across upgrades.** Capture on Wayland is the part of this
   stack that breaks on compositor updates, and nothing warns when it does — the failure
   appears as a black screen at the moment it is needed.
3. **The host is not reachable until somebody logs in.** Named in decision 2. It is a real
   limitation of the safe arrangement, not an oversight.
4. **One unconfined service on a host that confines nearly everything else.** Decision 6.
5. **A pinned package must be repinned by hand** to take a security fix. The digest is the
   price of not floating.

**Gained**

- Work that needed physical presence stops needing it, over a transport already trusted.
- No new inbound surface on the LAN, no port forward, and no third-party relay.
- The MAC register exists, which converts one specific unknown into an open item.

## Rollout

1. `--tags firewall` first, and the default-deny ruleset live. **Gate:** the role asserts it.
2. `--tags remote-desktop`. Installs, configures, enables the user unit.
3. The operator sets the Sunshine password through an SSH port-forward to loopback, and pairs
   the client with a PIN through the same forward.
4. Verify: `ss -ltn` shows the ports; `journalctl --user -u` shows the capture method actually
   chosen; a reboot, a Hyprland logout/login and a NetBird reconnect each leave it working.

Undone by removing the package and the user unit. No state migrates; the credential and the
paired-client list live on the host and die with it.

## Open questions

1. **Does `auto` pick the right screen with the lid shut?** Sunshine enumerates outputs at
   stream time and takes the first when `output_name` is unset. That it lands on the headless
   output when the panel is gone is expected, not observed — the lid has not been closed during
   a stream. Verify before relying on it.
2. **Does the client need governing?** The Moonlight end is a macOS machine that is not in
   `device-registry.yaml` and not covered by any `infra-*` repository. RFC-0035 open question
   2 is the same question wearing different clothes.
3. **What repins the package?** Nothing watches upstream releases for this estate. It is the
   generic version of a problem `proxmox_iso_version` already has.
4. **Does decision 5's detection belong in a check?** *"The capture path in use is the one
   intended"* is observable from the log and is not observed by anything.

## Related

- **RFC-0035** — D1, D3, D4, D7; the controls this rides on and does not widen
- **ADR-0058** — the tunnel transport this reuses for the web UI, rather than adding a second
- **ADR-0053** — the pin
- **ADR-0057** — the secret store this deliberately does not use, and why
- **STD-0016** — how the role is written; rules 1, 3, 5 and 10 all apply to it
