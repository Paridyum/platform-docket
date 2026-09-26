---
id: RUN-0005
title: Building and running the winsync uploader — the procedure RFC-0038 assigns
type: runbook
status: accepted
created: 2026-09-02
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["*"]
tags: [winsync, egress, windows, libvirt, opentofu, substrate, rfc-0038, req-0021, req-0014]
breaking: false
enforced_by: []
---

# RUN-0005 — Building and running the winsync uploader

RFC-0038 decision 1 assigns this work a runbook rather than a repository, because it is an
operation on a governed host and not a Paridyum deliverable. **RFC-0038 is the decision; this is
the procedure.** REQ-0021 is what the procedure owes.

## The shape of it in five lines

1. The guest is built with **no access to anything protected**, and that part is ungated.
2. It is used to answer one question: **does the vendor's client tolerate a read-only source?**
3. Only then does a human choose option A or B of RFC-0038 decision 7.
4. Under option A the substrate stops, the drive is attached **read-only**, and the upload runs.
5. Completion is reconciliation against a source inventory. Nothing on the source is ever written.

**Steps 1 and 2 do not require the decision in step 3.** That ordering is the point: the pivotal
fact is cheap to learn and the boundary narrowing is expensive to accept, so the cheap thing goes
first.

## Where everything lives

| Thing | Path | Notes |
| --- | --- | --- |
| Module | `sitar:/home/winsync/tofu/` | `main.tf`; state alongside it |
| Media | `/home/winsync/iso/` | Windows ISO, and the answer-file ISO |
| Answer file | `/home/winsync/autounattend.xml` | source for the ISO below |
| Guest storage | `/home/winsync/vm/` | libvirt pool `winsync` |
| `tofu` | `~/.local/bin/tofu` | unprivileged install, checksum-verified |

`/home/winsync` is `2770 rohan:paridyum`. Owner `rohan` so the operator can write; group
`paridyum` because that is the account QEMU runs as on this host; setgid so new files inherit the
group. **This exists because `/home/paridyum` is `0700` and belongs to the substrate, and
`/home/rohan` is `0700` and QEMU cannot read it** — the media had nowhere else to sit.

## Phase 1 — build the guest (ungated)

```sh
ssh sitar
cd /home/winsync/tofu
~/.local/bin/tofu plan          # expect 3 creates: pool, volume, domain
~/.local/bin/tofu apply
```

The domain is created **defined but not running** (`running = false`), so the XML libvirt
generated can be read before anything boots:

```sh
virsh -c qemu:///system dumpxml winsync-01 | less
```

Check three things, and they are the three that are wrong most often:

- every `<disk>` is `bus='sata'` — a virtio disk means an installer that reports no disks at all
- the two `<disk device='cdrom'>` entries are `sdb` (Windows) and `sdc` (answer file), in that order
- `<graphics>` listens on `127.0.0.1` and nothing else

**The UEFI boot prompt needs a keypress, and it is scripted rather than clicked.** Microsoft's
UEFI boot image prints *"press any key to boot from CD or DVD"*, and unpressed it hands control
back to the firmware, which reports `BdsDxe: No bootable option or device was found` — a message
that looks like a missing device and is really an expired prompt. Press keys through the window:

```sh
virsh -c qemu:///system reset winsync-01
for i in $(seq 1 8); do
  virsh -c qemu:///system send-key winsync-01 --holdtime 900 KEY_ENTER
done
```

`--holdtime` paces the loop across the boot window. **Use `virsh reset`, not the monitor's
`system_reset`** — the latter returned success here and did nothing at all, which cost a diagnostic
cycle. The way to tell a real reset from a fake one is the media read counter: it climbs.

Then start it and watch the install:

```sh
~/.local/bin/tofu apply -var running=true
virsh -c qemu:///system vncdisplay winsync-01     # e.g. 127.0.0.1:0  → port 5900
```

From a workstation, tunnel the console rather than exposing it:

```sh
ssh -L 5901:127.0.0.1:5900 sitar
# then point a VNC client at localhost:5901
```

The install is unattended. When it is done, `C:\winsync-ready.txt` exists and the guest is logged
in as `winsync` with autologon.

**Patch it before it ever sees data.** The media is build 19044.1288 from October 2021; let Windows
Update run to completion first. That is a long unattended job and it happens while the guest holds
nothing, which is the cheapest possible time for it.

## Phase 2 — answer the pivotal question

RFC-0038 open question 1, and it decides the whole design:

1. Create a **throwaway** qcow2 with a few files, attach it **read-only**, and boot.
2. Install the vendor client in the guest, entering the credential **interactively** — never into
   the answer file, never into an image (RFC-0038 D9, REQ-0021 item 4).
3. Point it at the read-only volume and see whether it uploads or refuses.

**If it refuses**, decision 4 is refuted, and the honest next step is RFC-0038's option B — the
drive in an enclosure on the macOS host — not a writable passthrough on this one.

## Phase 3 — the gated part

Do not begin without a human decision on RFC-0038 decision 7, recorded. Then, in order:

| | Step | Gate |
| --- | --- | --- |
| 1 | REQ-0012 discharge written: scope, exit plan for 3.5 TB, seat facts, account named per STD-0017 | before the first byte |
| 2 | Enumerate the drive's 41 top-level entries into an explicit include/exclude list | operator's decision, recorded here |
| 3 | Under option A: `virsh -c qemu:///system shutdown pve-staging`, autostart disabled | RFC-0038 D7 condition 2 |
| 4 | A root nftables rule denying the guest the LAN and the NetBird interface | RFC-0038 D8 — **not satisfied by NAT alone** |
| 5 | Host unmounts `sda2` (`udisksctl unmount -b /dev/sda2`) | RFC-0035 D8 second clause |
| 6 | Attach `/dev/sda2` **read-only** — `source = { block = { dev = "/dev/sda2" } }`, `read_only = true` | the module does not carry this; it is added deliberately and removed after |
| 7 | Upload in sequence, one folder at a time | RFC-0038 D5 — one uploader unless a measurement says otherwise |
| 8 | Reconcile each folder against a source inventory: file count and byte total | REQ-0021 item 8. Not the client's own "up to date" |
| 9 | Detach, destroy the guest, restart `pve-staging` | restarting the substrate ends the narrowing |

**Step 6 is the one to be slow about.** `read_only = true` plus `device = "disk"` and a block
source is the entire difference between an egress and an incident.

## Known exposures, stated rather than discovered later

- **The local Windows account password is in `autounattend.xml` in plaintext**, and on the answer
  ISO built from it, on a host with no disk encryption. Autologon requires it and the vendor's
  client requires an interactive session, so it cannot be avoided while the design stands. It is a
  disposable guest's local account, it is not the vendor credential, and it is not a Paridyum
  secret — but it is a password on an unencrypted volume and it is named here for that reason.
  Rotate or destroy the guest when the run ends.
- **User-mode networking is not isolation either.** slirp routes through the host's stack, so the
  guest can still reach the LAN. Phase 3 step 4 is the fix and it needs root; until it is in place
  the guest must hold no data. What user-mode networking does buy is the absence of a bridge, a
  libvirt network object and an nft table — three things that would otherwise need to be correct.
- **The evaluation activation is reported broken on LTSC 2021** owing to a key Microsoft shipped
  incorrectly. Unactivated Windows runs and syncs. The workarounds circulating for it are refused
  under REQ-0014 (RFC-0038 D12).

## If it goes wrong

- **The guest loads `boot.wim` and then hangs, screen full of coloured garbage** — it has no ACPI.
  **This provider emits only the features you declare and adds none of libvirt's usual defaults**,
  so a domain with no `features` block gets no `<features>` element, no ACPI and no APIC, and
  Windows requires ACPI. The signature is precise and worth recognising: vCPU 0 in 16-bit real
  mode at a fixed EIP in bootmgr's real-mode stub, identical across reboots, with the media's read
  counter stopping at exactly the same byte total each time. Check
  `virsh -c qemu:///system dumpxml winsync-01 | grep -A3 '<features>'` **first** on any new guest
  on this provider — an empty result is the bug.
- **Do not chase the video device.** The corrupted framebuffer is a *symptom* of the dead guest,
  not a cause. Both `cirrus` and std `vga` render it identically, and `virsh screenshot` and the
  monitor's own `screendump` agree — which is what proves the content is real. A guest reset gives
  a clean readable SeaBIOS screen from the same renderer, and that is the check that separates
  "the capture is broken" from "the guest is broken".
- **Installer reports no disks** — a `virtio` disk survived into the XML. Fix the module, not the guest.
- **Setup stops on the product-key page** — the empty `<Key></Key>` did not take. Re-check the
  `UserData` block; do not paste a key from anywhere but Microsoft.
- **Guest boots the installer again after installing** — the Windows media is still `sdb` and no key
  was pressed at *"press any key to boot from CD"*. Expected on the first reboot only; if it loops,
  detach `sdb` once the disk is bootable.
- **`Network Start Failed ... nft -ae insert rule ip libvirt_network`** — libvirt's nftables
  backend has no `libvirt_network` table on this host, which is the same *"host firewall: none"*
  finding RFC-0035's audit recorded, and it is why the substrate's own network runs
  `forward mode='open'` (open mode applies no firewall rules at all). **The module does not use a
  libvirt network for this reason** — the guest gets QEMU user-mode networking, which needs no
  bridge, no nft table and no root. Do not "fix" this by switching to `mode='open'` on a routed
  subnet: that yields a guest with no outbound path and a false sense of isolation.
- **`tofu` cannot reach libvirt** — `rohan` is in the `libvirt` group and manages `qemu:///system`
  through polkit. An unqualified `virsh` answers `qemu:///session`, which holds nothing. Always
  pass `-c qemu:///system`; the errata on RFC-0038 exist because that was missed once.

## Related

- **RFC-0038** — the decisions, the measured host facts, and the errata
- **REQ-0021** — what any bulk operator egress owes, independent of this design
- **RFC-0035** — decision 8's boundary, decision 1's trust direction, decision 7's credential ordering
- **RFC-0023** — the substrate this run displaces under option A
- **ADR-0052**, **ADR-0056** — OpenTofu creates the machine; nothing here configures inside one
- **REQ-0012**, **REQ-0014**, **STD-0017** — the vendor, the media's provenance, the account's name
- **RFC-0033**, **STD-0013** — the logbook this run writes, and why this document exists
