---
id: RFC-0038
title: The personal drive is read-only, its egress is exclusive, and it finishes before the substrate exists
type: rfc
status: proposed
created: 2026-08-31
locked_at: null
supersedes: null
superseded_by: null
resolved_by: null
scope: [org, infra]
applies_to: ["*"]
tags: [substrate, personal-data, egress, passthrough, windows, kvm, libvirt, vendor, rfc-0035, rfc-0023, req-0012, glo-0009]
breaking: false
enforced_by: []
sources:
  - url: https://help.sync.com/hc/en-us/articles/42790759628819-Updating-from-an-unsupported-version-of-the-Sync-desktop-app
    title: Updating from an unsupported version of the Sync desktop app
    retrieved: 2026-08-31
    supports: >
      The vendor's supported-platform floor, which decision 12 selects an edition against:
      Windows 8, 10 and 11; 32-bit operating systems are no longer supported. Read via
      search extract on 2026-08-31 — help.sync.com returns 403 to a direct fetch, and the
      limitation is recorded rather than hidden.
  - url: https://help.sync.com/hc/en-us/articles/38275586765075-The-Sync-desktop-application
    title: The Sync desktop application
    retrieved: 2026-08-31
    supports: >
      The desktop application exists for Windows and Mac only, and **Sync does not support
      Windows Server operating systems or running as a service.** This answers open question 3
      in the negative and independently confirms the intaken plan's rejection of Server Core —
      the constraint is the vendor's, not a preference. A native Linux client does not exist;
      a Linux WebDAV client is described as in progress. Read via search extract, as above.
  - url: https://www.sync.com/help/the-sync-desktop-application/
    title: The Sync desktop application (canonical URL)
    retrieved: 2026-08-31
    supports: >
      The vendor's own permanent link for the entry above; it 301s to the help-centre article,
      which returns 403 to a direct fetch. Recorded so the claim has a citable address that does
      not depend on the help centre's current article id.
  - url: https://www.microsoft.com/en-GB/evalcenter/evaluate-windows-10-enterprise
    title: Windows 10 Enterprise — Microsoft Evaluation Center
    retrieved: 2026-08-31
    supports: >
      Microsoft supplies a 90-day evaluation of Windows 10 Enterprise LTSC as a ~4.9 GB x64
      ISO, no product key required, behind a registration form. Decision 12's licensed route
      to first-party media. Mainstream support for LTSC 2021 runs to 2027-01-12, which exceeds
      the run's expected duration by a wide margin.
---

# RFC-0038 — The personal drive is read-only, its egress is exclusive, and it finishes before the substrate exists

## Status

**PROPOSED**, 2026-08-31. Opened on intake of the *Windows Sync Fleet — Deployment Plan* from
`sitar-as-in-01-ar`, which proposes creating Windows guests on the host RFC-0023 designated as the
platform substrate and passing that host's 3.6 TB personal NTFS partition into one of them.

**This RFC exists because the plan's premise is the exact configuration RFC-0035 decision 8
forbids**, and that collision cannot be resolved by an implementation detail. Decision 3 states it
plainly and decision 7 proposes the only narrowing that keeps both artifacts true.

`enforced_by` is empty for the reason RFC-0035 gives for the same host: `tool-forge` cannot inspect
a machine it does not run on. Nothing here is `locked` and nothing pretends to be checkable.

**Nothing in this RFC authorises the run.** It proposes a narrowing of a boundary decision, which
under C-0004 and ADR-0013 is a human's to accept. Until a human accepts it, the correct state of
the work is *not started* — not *started carefully*.

**The intaken document, identified so this RFC's input is unambiguous.** *Windows Sync Fleet —
Deployment Plan*, retrieved from `sitar-as-in-01-ar:/home/rohan/win-sync-fleet-plan.md` on
2026-08-31 — 7695 bytes, mtime 2026-08-31 21:21:33 +0530, sha256
`20fb28a267b5af6edc9caa15c5b667c02803c36cc15832ef28212f6d7a8b2934`. It proposes one main Windows
guest owning a raw passthrough of a second physical drive plus up to five ephemeral Windows worker
guests, each uploading one folder to Sync.com from a per-worker qcow2 copy staged by the main guest.
Every decision below is written against that document; the hash is recorded because the file is
mutable and this RFC answers one specific version of it. No `sources` block: the input is a local
document rather than an external claim, and RFC-0020's citation machinery is for the latter.

## The question

**The operator has 3.5 TB of personal data on the machine that is also the platform substrate host,
and it needs to reach a consumer cloud vendor whose only client is a Windows desktop application.
How is that done without reversing the trust direction the estate just finished writing down?**

## What was measured, not assumed

Observed on `sitar-as-in-01-ar` over the tunnel — the drive and capacity figures on 2026-08-31, the
virtualisation rows **re-measured on 2026-09-02 in `qemu:///system` scope after the first pass read
`qemu:///session` and drew the wrong conclusion** (errata, 2026-09-02). Every decision below rests
on this table rather than on the plan's description of the host:

| Fact | Measured value | Why it matters here |
| --- | --- | --- |
| The drive | `sda2`, 3.6 T, **NTFS, label `Storage`** | **This is the partition RFC-0035 decision 8 names by label.** It is not "a second drive"; it is the protected one |
| Its state | mounted **on the host** at `/run/media/rohan/Storage` | **RFC-0035 decision 8's second clause — *never mounted while a guest is running* — is violated on the host today**, independently of this plan, because a guest is running |
| Its occupancy | 3.7 T total, **3.5 T used, 243 G free, 94 %** | The volume of data the vendor and the uplink must carry |
| Its top level | **41 entries**, including `SteamLibrary`, `Games`, two `C-DRIVE-ARCHIVE-*` trees, `Hyper-v-vms`, `$RECYCLE.BIN`, `Recovery`, and an existing `Sync` | The plan assumes five target folders. The drive does not have five of anything |
| Host free space | `/home` 422 G total, **375 G available**; `/` 46 G total, 21 G available | The staging tier must fit 3.5 T of copies into 375 G |
| Guests defined | **`pve-staging`, running** — 12 vCPU, 40 GiB, autostart enabled, Proxmox 9.2 media attached, no host block device and no shared filesystem | **The substrate exists.** This is the fact decision 7 is written against, and the first version of this RFC had it wrong — see the errata |
| Storage | pool **`paridyum`** at `/home/paridyum/vm`, 421 GiB capacity, **395 GiB available**; network `paridyum` active | Still three orders of magnitude short of 3.5 TB, so decision 6 is unaffected |
| Memory budget | 62 GiB total, **40 GiB committed to the substrate guest** | ~22 GiB for everything else, which is what decision 5's fan-out would have spent |
| Disk encryption | **none** — zero LUKS devices | RFC-0035 decision 7 is unmet, so no credential may be written here |
| Nested virtualisation | `kvm_*.nested = Y` | RFC-0035 decision 6's stage-B0 permission still holds |
| Memory / CPU | 62 GiB, 16 threads | Six concurrent Windows guests fit. Decision 5 argues they should not exist anyway |

**The uploader would not be the first guest on this host; it would be the second, alongside a running
substrate.** That is the fact decision 7 turns on, and it removes the clean sequencing the first
version of this RFC believed was available. It also means RFC-0035 decision 8 is already being
breached on this host in its second clause — the operator's 3.6 TB partition is mounted while
`pve-staging` runs — which is a live hardening gap that this plan did not create and does not
depend on. It belongs in REQ-0015's register whatever happens to this RFC.

## Constraints that already bind, retrieved rather than invented

| Already decided | Where |
| --- | --- |
| The operator's data — `/home` and the NTFS `Storage` partition — is **never mounted into a guest and never mounted while a guest is running** | **RFC-0035 D8** |
| Host → guest permitted; **guest → host denied**. No `virtiofs`, no 9p, no shared clipboard, no USB redirection, no SPICE agent | **RFC-0035 D1** |
| The host holds **no Paridyum secret until the volume holding it is encrypted** — stated as a precondition because encrypting afterwards does not un-write the credential | **RFC-0035 D7** |
| A `paridyum` service account owns the substrate and is not the operator's account; no passwordless `sudo` | RFC-0035 D2 |
| Management interfaces bind the tunnel interface, never the LAN, never `0.0.0.0` | RFC-0035 D4, ADR-0058 |
| **A *fleet* is one Proxmox cluster carrying workloads.** Two exist by name and neither is this. *"Do not invent a second name for a fleet"* | **GLO-0009**, RFC-0025 |
| OpenTofu owns *what machines exist*; Ansible owns *what is true on this machine*; **nothing in a shell script may create a machine** | **ADR-0052, ADR-0056** |
| A third-party service holding organisation data is approved, scoped and **exitable**, and produces evidence. Status `accepted`, **trigger FIRED 2026-08-08** | **REQ-0012** |
| Accounts are named for the role, unambiguous in every namespace they appear in | STD-0017 |
| Virtual-machine names are **not** governed by the hostname grammar; that gap is assigned elsewhere and never filled for a non-cluster guest | RFC-0021 D9 |
| `wn` = Windows, `kind: os`, **`status: active`** | `device-registry.yaml` |
| The vendor's client is **Windows and Mac only**; **Windows Server and running as a service are not supported**; minimum Windows 8/10/11, 64-bit only | help.sync.com, retrieved 2026-08-31 |
| An external dependency's provenance is established rather than assumed | REQ-0014 |
| Everything built is the production system; `dev` names where something runs, never how well it is built | C-0010 |
| Irreversible and governance-altering operations require a human | C-0004 (locked) |

## Decisions

### 1 — This is not a Paridyum workload, and it is governed anyway

The data is the operator's, the vendor is the operator's, and the purpose has nothing to do with the
platform. **Governance attaches here because of *where it runs*, not because of what it is.** The
host is the substrate host; a decision made on it is a decision about the substrate whether or not
that was the intent.

So: no product entry, no tenant, no `svc-` or `platform-` repository, and it is not customer zero's
work. What it is instead is an **operation** — and under RFC-0032 and STD-0013 an operation's home
is a runbook, with a logbook under RFC-0033 recording how the run actually went. That is the whole
artifact footprint this deserves.

**The reason to say this out loud** is that the opposite reading is available and wrong: that
because the work is personal, the host's rules are somebody else's problem. C-0010 already refuses
the cheaper version of that argument — there is no tier of this estate where a thing is built to a
lower standard because of what it is for.

### 2 — The word *fleet* is refused

GLO-0009 defines a fleet as one Proxmox cluster carrying workloads and says directly: *"Do not
invent a second name for a fleet."* The intaken plan is titled *Windows Sync Fleet*, which takes the
estate's most load-bearing infrastructure noun for six Windows VMs on a laptop.

**The work is `winsync`; its guests are the *uploader* and, if decision 5 ever justifies them,
*workers*; the whole activity is a *run*.** Renaming costs nothing today and costs an incident's
worth of confusion the first time somebody greps for "fleet" during one.

### 3 — The drive in the plan is the drive RFC-0035 protects, and this is the governing fact

The plan says *"a second physical drive"*. Measured, that drive is `sda2` — NTFS, label `Storage`,
3.6 T — which is the partition RFC-0035 decision 8 names in its own text:

> *"The operator's data — `/home`, and the NTFS `Storage` partition — is **never mounted into a
> guest and never mounted while a guest is running**."*

**The plan's central mechanism is therefore the one configuration the most recent hardening decision
forbids, by name.** Not adjacent to it, not a grey area: passing `sda2` into a guest is the
prohibited act, and it is prohibited twice over — as a mount into a guest, and as a mount coexisting
with a running guest.

This is recorded as a decision of its own because the failure mode being prevented is a real one:
the collision is easy to not notice, since the plan and the RFC use different words for the same
3.6 TB of NTFS, and an implementer reading only the plan would build the forbidden thing in good
faith.

### 4 — The passthrough is read-only, and that is the decision the rest hangs from

Every operation the run needs — enumerate folders, read bytes, upload them — is a read. **Nothing
in the purpose requires write access to the source**, and so nothing gets it: the block device is
attached `readonly`, and the guest's inability to write is a property of the domain definition
rather than a promise about the guest's behaviour.

What this removes, precisely:

- A Windows guest offering to format, `chkdsk`-repair, or replay the journal of an NTFS volume it
  finds with a dirty bit — which is *the* classic way to lose an NTFS volume to a VM.
- Anything running in a Windows guest, invited or not, writing to 3.5 TB of the operator's data.
- The plan's checksum-verification step for the staged copies, because there are no copies.

**And it dissolves the constraint the plan's whole architecture was built around.** The plan opens
with *"a raw block device can only be passed through to one VM at a time"*, and the staging tier —
five qcow2 images, copy, verify, detach, reattach — exists solely to work around that. The
exclusivity is a property of *writable* attachment. A read-only device may be attached to several
domains at once, so with decision 4 in force the workaround has nothing left to work around.

> **This is the pivotal verification, and it is honest to call it that:** the run is only buildable
> this way if the vendor's Windows client tolerates a read-only source, and if concurrent read-only
> NTFS mounts across guests behave. Open questions 1 and 2. If either fails, the design changes —
> but it changes *from* a measured refutation rather than *to* a guess.

### 5 — Concurrency is justified by a measured bottleneck, and the default is one uploader

The plan spins up five workers to upload five folders in parallel. **On a residential connection the
uplink is one resource and five uploaders divide it rather than multiply it** — RFC-0023 already
records what this machine is: *"one machine, one power supply, one network connection, in a
residence."*

So the fan-out is not built until a measurement says the bottleneck is somewhere other than the
uplink. Until then: **one uploader guest, folders in sequence.** If the measurement does justify
workers, they are worth exactly what the measurement showed and no more.

The cost of guessing wrong in the other direction is not symmetric. Five workers means five more
Windows guests, five more vendor device seats, five more copies of a credential, and five more
guests with a read path to the operator's data — all bought with no throughput gained.

### 6 — The staging tier is rejected on arithmetic, not on taste

The plan stages each folder into a qcow2 image on host storage. Measured: **3.5 TB of source
against 375 GB of free host storage — 10.7 %.** The design as written cannot run. Batching it
folder-by-folder with reclaim in between makes it *possible*, at the price of copying 3.5 TB through
the host's SSD to upload it, and it still fails outright for any single folder above ~375 GB, which
`SteamLibrary` and the two `C-DRIVE-ARCHIVE-*` trees may well exceed.

Decision 4 removes the need for it entirely. It is rejected here explicitly, with the number, so
that it is not reintroduced later as an optimisation.

**And the folder set is an input to be enumerated, never assumed.** The drive has 41 top-level
entries, several of which — `$RECYCLE.BIN`, `Recovery`, `Hyper-v-vms`, `DumpStack.log.tmp`, an
existing `Sync` directory — are things an upload should probably exclude rather than include. Which
folders leave the machine is a decision the operator makes against the real listing, in the runbook,
before the first byte moves.

### 7 — The run requires exclusivity, and the substrate is already holding the host

*Rewritten 2026-09-02. The first version of this decision rested on a measurement taken in the wrong
libvirt scope and concluded that no guest existed. One does, it is the platform substrate, and it is
running. See the errata.*

Measured in `qemu:///system`: **`pve-staging` — the nested Proxmox substrate of RFC-0023 — is
defined, running, autostart enabled, holding 12 vCPU and 40 of the host's 62 GiB**, with an active
`paridyum` pool and network. So RFC-0035's rollout is not at step 0; guests exist, created with no
shared path, which is its step 5. **Under the conditions this decision originally proposed, the
window's closing edge has already passed and the run is simply forbidden.**

That is the honest starting point, and it leaves three options rather than one.

**Option A — bounded exclusivity: the substrate stops for the duration of the run.** `pve-staging`
is shut down, its autostart disabled, and the host carries the uploader alone. This is available
rather than heretical: RFC-0023 accepted cost 3 says *"everything on it is disposable and must be
treated that way"*, and RFC-0025 stage B0 carries no customer workload, so nothing is being served
that stopping it interrupts. **The cost is not free and must not be described as free** — the
platform's own substrate is unavailable for however long 3.5 TB takes over a residential uplink,
which is weeks, and that is time taken from the estate rather than from nothing.

**Option B — the drive goes on hardware that is not the substrate host.** A USB enclosure and the
macOS authoring host. The vendor publishes a Mac client (open question 3, answered), so this needs
no Windows guest, no passthrough, no narrowing of RFC-0035 decision 8, and it stops the substrate
for zero days. **This is now the recommendation.** It costs an enclosure.

**Option C — coexistence: the uploader runs alongside the substrate.** Refused. Nested
virtualisation is enabled on this host, RFC-0035 decision 6 accepts that only because *"the only
occupant of the guest is Paridyum"*, and a Windows guest holding a live vendor credential and a read
path to 3.5 TB of the operator's data is precisely the second occupant that reasoning excluded. Two
guest-to-host escapes in one quarter is the rate that decision was written against, and this is the
configuration their worst case names. The memory budget says the same thing more cheaply: 22 GiB
remain uncommitted.

> **PROPOSED NARROWING of RFC-0035 decision 8, under option A only.** Its prohibition is unchanged
> as to substrate guests, permanently. It is narrowed for a guest whose entire purpose is moving the
> operator's own data off the machine, and only under all six of these conditions together:
>
> 1. the attachment is **read-only** (decision 4);
> 2. **`pve-staging` is shut down and its autostart is disabled** for the whole run, so no Paridyum
>    guest is running while the operator's partition is attached to anything;
> 3. **no Paridyum secret** is present on the host, and none is written while the run is open;
> 4. the guest has **no path to the host** — RFC-0035 decision 1 is untouched and applies in full;
> 5. the guest has **no path to the tunnel** (decision 8 below);
> 6. the narrowing **expires when the substrate is restarted**, which is now the closing edge that
>    replaces the rollout-step reference the first version used. Restarting `pve-staging` ends the
>    permission with no further decision required.
>
> **Under option B none of this is needed, which is the strongest argument for option B.**

Why a narrowing rather than a supersession: RFC-0035 decision 8's purpose is that *"the operator's
data is not reachable from a guest even when the guest is fully compromised"*, and that purpose is
about protecting the operator from the **substrate**. A guest that exists only to move the
operator's own data is the single case where the mount is the point rather than a leak. Option A
keeps the protected property true of every guest the decision was written about, by ensuring none of
them is running.

Why it is not an erratum: under ADR-0031 an erratum fixes a wrong fact and leaves the decision
untouched. This changes what RFC-0035 decides for one bounded case, so it is a decision, and
decisions of this kind are accepted by a human under C-0004 and ADR-0013 stage 3.

**Separately and not gated by any of the above: building the Windows guest is not the narrowed
act.** A guest with its own system disk and no passthrough attaches nothing protected, and it is how
open question 1 gets answered against a throwaway volume before anyone is asked to accept a
narrowing at all. It may be built under option A or B, and under coexistence with `pve-staging`,
because until the drive is attached it holds nothing that decision 8 protects.

### 8 — The guest reaches the vendor and nothing else

RFC-0035 decision 1's direction holds unchanged: **host → guest permitted, guest → host denied.** No
`virtiofs`, no 9p, no shared clipboard, no USB redirection, no SPICE agent channel — the plan
proposes none of these and this decision is what keeps it that way when a debugging session gets
inconvenient at 2 a.m.

Added for this run specifically: **the guest has no route to the NetBird interface and no route to
the LAN.** It egresses to the vendor and to whatever the Windows guest needs to activate and update
itself, and it can reach no management plane. ADR-0058 makes the tunnel the estate's only inbound
path; a Windows guest holding 3.5 TB of personal data has no business being on it in either
direction.

Verification is from the guest, borrowing STD-0009's shape as RFC-0035 decision 8 does: **the denial
is the assertion.** A host-side reading of the domain XML confirms the intended configuration; only
a check run inside the guest confirms the achieved one.

### 9 — The vendor is governed, and the credential is not baked into an image

REQ-0012 is `accepted` and its trigger **FIRED on 2026-08-08**. A third-party service about to hold
3.5 TB of the operator's data is squarely inside it, so before the run:

- **Approved and scoped** — what data goes, what does not (decision 6's enumeration is this list).
- **Exitable** — *how do 3.5 TB come back, over how long, at what cost, if the vendor or the account
  goes away?* This is the question a consumer cloud vendor is least likely to answer well, and
  discovering the answer after the upload is discovering it too late.
- **Seat facts established before the design depends on them** — the plan flags the device-limit
  risk itself; decision 5 makes it mostly moot by defaulting to one uploader.

On the credential, RFC-0035 decision 7's ordering argument applies with no modification even though
this is not a Paridyum secret: **the host has no disk encryption, and once a credential is written
to an unencrypted volume, encrypting the volume afterwards does not un-write it.** So the vendor
credential is **entered at run time into the running guest, never baked into a golden template
image and never stored on the host.** The plan's *"account linked in the template"* is the exact
shape this refuses — an image on an unencrypted laptop that is, itself, a copy of a live credential.

The account is named per STD-0017 — for its role, unambiguous in every namespace — and it is one
account, not one per guest.

### 10 — The machines are created by the tools that create machines

The plan proposes *"a simple host-side shell script"* looping `virsh` to clone, attach, start, poll
and destroy. **ADR-0056 draws the line this crosses: nothing in a shell script may create a
machine.** OpenTofu owns what machines exist (ADR-0052); Ansible owns what is true inside one; and
Windows guest configuration under Ansible follows STD-0016 like any other host.

The cost is real and is accepted rather than argued away: for a one-off chore, a provider and a
playbook are heavier than twenty lines of shell. It is accepted because this host is the substrate
host, and *"a hand-built path is always five minutes faster"* is the sentence RFC-0025 decision 2
wrote about exactly this temptation. The one-off script is also, per C-0010, the prototype tier that
does not exist here.

### 11 — Completion is verified against the source, and the source is never written

Two rules, and the second is absolute:

1. **A folder is done when what the vendor holds is reconciled against an inventory taken from the
   source** — a file count and a byte total at minimum. Not when the client's UI says "up to date",
   and not when a timer expires. The plan's own open question — poll the log, fixed timeout, or
   folder-size-stable heuristic — is answered by refusing all three as the *authority*: they may
   decide when to *look*, never what is *true*.
2. **Nothing on the source drive is deleted, moved, or modified by this work, ever.** Decision 4
   makes that structural rather than procedural. Reclaim and teardown touch guests and copies only.
   The drive is 94 % full and this run is not a migration.

### 12 — The guest is first-party Microsoft media, and the leanest supported edition is chosen rather than manufactured

**Windows 10 Enterprise LTSC 2021, x64**, from Microsoft's Evaluation Center — a 90-day evaluation,
no product key, ~4.9 GB, mainstream support to 2027-01-12.

Why this edition rather than a lighter-sounding one, in the order the constraints actually bind:

- **The vendor's floor is Windows 8/10/11, x64, and Windows Server is explicitly unsupported —
  as is running as a service.** That is the vendor's statement, not a preference, and it closes
  the intaken plan's *"Server Core or LTSC"* question and its own open question about whether an
  interactive desktop session is really needed. It is: there is no service mode to fall back to.
- **LTSC is the lean build Microsoft itself ships.** No Store, no bundled UWP applications, no
  Cortana, no Copilot — which is the "nothing bloated" requirement satisfied by an edition choice
  instead of by a debloating pass over a consumer image.
- **No TPM or Secure Boot floor**, unlike Windows 11, so the guest needs neither `swtpm` nor an
  OVMF variables dance. Both are available on the host if a later decision wants them.
- **The uploader is not a workstation.** Nothing about this guest needs the newest Windows; it
  needs a supported one that runs one application for a few weeks and is then destroyed.

> **A third-party "debloated" image — Tiny11, a stripped consumer ISO, anything rehosted — is
> refused.** REQ-0014 governs provenance, and an operating system image assembled by an unnamed
> party is the least auditable dependency in the whole run, sitting underneath a live credential
> and a read path to 3.5 TB of the operator's data. The same rule refuses ISO mirrors, archive
> rehosts and activation workarounds: the media comes from Microsoft or the run does not start.

Two facts recorded because they will otherwise be discovered at an inconvenient moment. The
evaluation's activation is reported to fail on LTSC 2021 owing to a key Microsoft shipped
incorrectly — an unactivated Windows still runs and still syncs, and the workarounds circulating
for it are exactly what the paragraph above refuses. And `/var/lib/libvirt` sits on a 46 G root
volume with **21 G free**, so the guest's system disk and the installation media live on the
`/home` volume's 375 G instead; the default path is a dead end here.

## Alternatives considered

**Attach the drive to a machine that is not the substrate host.** The honest answer, and it deletes
this RFC: a USB enclosure and the macOS authoring host, or any machine that is not carrying the
platform substrate. RFC-0035 decision 8 then never comes under pressure, decision 7's window is
unnecessary, and the vendor's macOS client — if it exists, which is open question 3 — makes the
Windows guest unnecessary too. **Deferred on hardware, not on merit**, exactly as RFC-0035 defers
"a second physical machine" for the substrate itself. Naming it here means buying an enclosure is a
planned step rather than a discovery, and it is the recommendation if the answer to open question 1
or 2 is no.

**Do it on the Arch host directly, no guest at all.** Cheapest by far. Rejected on the plan's own
finding: the vendor has no CLI, no Linux client, and no protocol worth scripting against — so there
is nothing on the host to run. If that finding is wrong (open question 3), this becomes the obvious
answer and the whole design collapses into a single process on the host, which is a good outcome
and should be checked for before anything is built.

**Build the plan as written.** One main guest with a writable passthrough, five workers, staged
qcow2 copies. Rejected three times over: it is the configuration RFC-0035 decision 8 forbids
(decision 3), the staging tier does not fit in 375 GB (decision 6), and the fan-out buys no
throughput on one residential uplink (decision 5). Recorded because it is the plausible
implementation an agent would produce from the intaken document alone, at speed, in good faith.

**Wait until the substrate exists and do it properly under `svc-fleet`.** Rejected as a category
error: the fleet manager provisions Paridyum's platform, and this is the operator's personal data.
Decision 1's point is that this work must be *governed by* the estate without becoming *part of*
it — and putting a personal backup chore through the fleet manager is how a god-shaped operations
surface starts.

**Encrypt the host first, then run.** Correct, and it is RFC-0035 decision 7's own ordering. Not
made a precondition here only because decision 9 removes the credential from the host entirely, so
the run writes no secret to the unencrypted volume. If any part of the design later needs a stored
credential, decision 7 fires and encryption becomes a precondition again.

## Consequences

**Accepted costs**

1. **Nothing here is enforced.** `enforced_by` is empty; `tool-forge` cannot inspect this host. The
   run is held to this by its runbook, its logbook, and the guest-side check in decision 8.
2. **The run is slower than the plan.** One uploader, in sequence, over a residential uplink, for
   3.5 TB. That is the honest shape of the problem; the plan's five workers were speed that was
   never available.
3. **A narrowing of a two-week-old hardening decision is on the table at all.** That is a real cost
   and it is why decision 7 is written as a proposal to a human rather than as a resolution.
4. **OpenTofu and Ansible for a personal chore is heavy**, and it will feel absurd on the day. The
   alternative is a hand-built path on the substrate host, which is the thing C-0010 and RFC-0025
   decision 2 both refuse.
5. **The window is a sequencing constraint on the substrate.** If the operator wants the substrate
   built now, this run does not happen on this host — the two are mutually exclusive by decision 7,
   and that is the point rather than a side effect.

**Gained**

- The collision between the plan and RFC-0035 decision 8 is on the record as a collision, in the
  words of both artifacts, rather than being resolved silently by whoever implements first.
- Read-only passthrough turns "the guest will not write to the operator's data" from a hope into a
  property of the domain definition — and takes the staging tier, its 375 GB ceiling, and its
  verification step out with it.
- The permission has an expiry written into it (RFC-0035 rollout step 5) instead of depending on
  somebody remembering it later.
- The estate's vocabulary survives contact with a document that used its most load-bearing noun for
  something else.

## Rollout

Ordered, because the first two items are gates rather than tasks and the last one closes the window.

| | Step | Gate |
| --- | --- | --- |
| 0 | A human chooses option A or option B, and under A accepts decision 7's narrowing | C-0004, ADR-0013 stage 3 — **no drive is attached without this.** Steps 1–3 and the guest build are not gated by it |
| 1 | Answer open questions 1–3 | If 3 is "a Linux or macOS client exists", stop and reconsider the whole design |
| 2 | REQ-0012 discharge: scope, exit plan, seat facts, account named | Written down before the first byte |
| 3 | Enumerate the 41 entries into an explicit include/exclude list | Operator's decision, recorded in the runbook |
| 4 | Under option A: `pve-staging` shut down, autostart disabled | Decision 7 condition 2 |
| 5 | Host unmounts `sda2`; it is not mounted on the host while the guest holds it | Decision 3's second clause |
| 6 | Uploader guest created by OpenTofu, `readonly` block attachment, no tunnel route, no shared path | Domain XML inspection **and** the guest-side check |
| 7 | Credential entered into the running guest at run time | Never on the host, never in an image |
| 8 | Upload in sequence; reconcile each folder against a source inventory | Decision 11 |
| 9 | Teardown; the substrate is restarted, which closes the narrowing; the logbook records what the run actually did | After this, RFC-0035 decision 8 is unnarrowed |

Undoing this is destroying one guest. There is no data migration, and that is a consequence of
decision 4 rather than a coincidence.

## Open questions

1. **Does the vendor's Windows client work against a read-only source?** A sync client that writes
   state into the folder it watches would refute decision 4, and decision 4 is what the rest hangs
   from. This is the first thing to test and the cheapest to test.
2. **Do concurrent read-only NTFS mounts across guests behave?** Only matters if decision 5's
   measurement ever justifies workers. NTFS is not a cluster filesystem; with no writer this is
   expected to be fine and *expected* is not *verified*.
3. ~~**Is the client genuinely Windows-only?**~~ **ANSWERED 2026-08-31, and half of it changes
   nothing while the other half sharpens an alternative.** The vendor publishes clients for Windows
   and Mac only; there is no Linux client, and Windows Server and service operation are both
   unsupported. So the Windows guest is forced, and *"do it on the Arch host directly"* is dead.
   **The macOS client, however, exists** — which means the recommended alternative below is not
   merely cheaper but fully supported: the drive in an enclosure on the macOS authoring host needs
   no guest, no passthrough, and no narrowing of RFC-0035 decision 8. It costs an enclosure.
4. **Does the uploader guest get a name, and from where?** RFC-0021 decision 9 explicitly does not
   govern VM names, and no artifact has filled that gap for a guest that is not a cluster node. The
   `wn` platform code is active in `device-registry.yaml`, which makes `sitar-as-in-NN-wn` *look*
   available — and taking it would assert this ephemeral guest is a registered device, which it is
   not. Proposed for now: named by the run tooling, prefixed `winsync-`, and **not** entered in
   `device-registry.yaml`. This overlaps RFC-0035 open question 2 and RFC-0021 open question 1.
5. **What closes the window if the run stalls?** Sharpened by the corrected measurement: under
   option A a 3.5 TB upload over a residential uplink stops the platform substrate for its whole
   duration, which may be weeks. A stated maximum — after which the run is suspended, the drive
   detached and the substrate restarted — belongs in the runbook and is the operator's call. It is
   also the number that decides whether option B's enclosure is cheaper than the delay.

## Related

- **RFC-0035** — decision 8 is the boundary this narrows, decision 1 the direction it leaves intact,
  decision 7 the credential ordering decision 9 borrows, rollout step 5 the window's closing edge
- **RFC-0023** — the substrate host, and *"one machine, one power supply, one network connection, in
  a residence"*, which is decision 5's whole argument
- **RFC-0025** — decision 2's *"a hand-built path is always five minutes faster"*, which decision 10
  is written against
- **GLO-0009** — what a fleet is, and the instruction not to invent a second name for one
- **ADR-0052**, **ADR-0056**, **STD-0016** — who creates machines, who configures them
- **REQ-0012** — vendor governance, `accepted` and fired; discharged for this vendor by decision 9
- **REQ-0021** — the requirement this RFC's decisions are the design for
- **STD-0017** — the vendor account's name
- **STD-0009** — *"the denial is the assertion"*, the shape decision 8's verification borrows
- **RFC-0021** — decision 9's unfilled VM-naming gap, and decision 7's compact-form rule for `wn`
- **RFC-0032**, **RFC-0033**, **STD-0013** — the runbook and logbook decision 1 assigns
- **C-0004**, **ADR-0013** — why decision 7 is a proposal rather than a resolution
- **C-0010** — why there is no cheaper version of this because the data is personal

## Errata

### 2026-09-02 — the virtualisation facts were measured in the wrong libvirt scope

**Said:** *"Guests defined | **none** — `virsh list --all` empty, **no storage pool defined**"*, and
in the paragraph beneath the table, *"The first guest ever created on this host would be a Windows
guest holding the operator's personal data."* Decision 7 was built on that: it offered a window that
was open *"right now, closing the moment the first Paridyum guest is defined."*

**Wrong.** Those commands ran against `qemu:///session`, which is `rohan`'s default URI on this host
and which holds nothing. In `qemu:///system` the substrate guest **`pve-staging` is defined and
running** — 12 vCPU, 40 GiB, autostart enabled — alongside an active `paridyum` storage pool at
`/home/paridyum/vm` and an active `paridyum` network. The substrate was built before this RFC was
written.

**Consequences, since this was load-bearing rather than decorative:** the window decision 7 offered
was already closed by its own condition 6; RFC-0035 decision 8's second clause is being breached on
the host today rather than merely at risk; and the choice is now between stopping the substrate for
the duration (option A), moving the drive to other hardware (option B), or coexistence (option C,
refused). The measured table and decision 7 are corrected in place rather than annotated, because a
wrong measurement left standing in a table that other decisions cite is worse than a rewrite — and
this section is the record that it was wrong.

**The general lesson, recorded because it will recur:** on a host where the operator's default
libvirt URI is `qemu:///session`, an unqualified `virsh` command answers a question nobody asked. A
scope is part of a measurement, and a measurement without one is not evidence.
