# Logbook — decisions

The authored half of `LOGBOOK.md` per RFC-0033 decision 3: **what was tried and rejected here that
never earned an ADR.** Append-only, newest last.

**RFC-0033 is `proposed`.** The generated half and the `logbook/ledger/` hook do not exist yet, and
per its decision 4 the ledger is written by a hook and never by an agent. This file is the authored
half only, started by hand because the record it holds would otherwise be lost while the tooling is
pending.

---

## 2026-08-31 — intake of the Windows Sync Fleet plan → RFC-0038, REQ-0021

**What arrived.** A plan document on `sitar-as-in-01-ar`
(`/home/rohan/win-sync-fleet-plan.md`, sha256 `20fb28a2…b2934`) proposing six Windows guests on the
substrate host to upload a 3.6 TB drive to Sync.com. Requested: intake it and write governance.

**How it was worked.** `paridyum-platform:decompose-spec` for the intake, then
`paridyum-platform:new-decision` for the artifacts. The host was audited over the tunnel before
anything was written, because the plan describes the host and the host was the thing to check.

**Three things the audit changed, recorded because each would have been wrong if inferred:**

1. The plan's *"second physical drive"* is `sda2` — the NTFS `Storage` partition **RFC-0035 decision
   8 protects by name**. The collision was invisible from either document alone; both describe the
   same 3.6 TB in different words.
2. The staging tier is arithmetically impossible on this host — 3.5 TB used against 375 GB free.
   Measured, not argued.
3. No guest and no storage pool are defined. **The substrate does not exist yet**, which is what
   made RFC-0038 decision 7's bounded window available instead of a flat refusal.

**Rejected, and why — the sub-ADR record.**

- **Writing an ADR alongside the RFC.** ADR-0013 stage 3 puts a human between the design and the
  decision, and RFC-0038 decision 7 narrows a hardening boundary. An ADR now would have recorded a
  decision nobody made.
- **An erratum on RFC-0035 instead of a new RFC.** ADR-0031's split is clean: an erratum fixes a
  wrong fact, and this changes what decision 8 *decides* for a bounded case. Erratum rejected as the
  cheaper wrong instrument.
- **A `sources:` block for the intaken plan.** `knowledge.schema.json` requires `^https?://` and the
  input is a local file; the gate caught it. Provenance moved into the Status section with hash,
  size and mtime, which is where a local input belongs anyway.
- **Storing the intaken document in this repository.** No artifact directory admits a non-artifact
  document, `research/` is an append-only JSONL log, and `docs/` is the superpowers tree. The hash
  identifies the input instead; the file stays where its owner keeps it.
- **A `winsync` repository.** RFC-0038 decision 1 — the work is an operation on a governed host, not
  a Paridyum deliverable. A repository would have made a personal chore part of the estate.

## 2026-09-02 — the winsync guest, built to the point the gate allows

**What was done.** Media verified, answer file written, OpenTofu module written and validated
against a live provider schema, everything staged on `sitar:/home/winsync`. The `tofu apply` itself
was refused by the harness's permission classifier and is left for the operator to run.

**The correction that mattered.** The first audit measured `qemu:///session` — `rohan`'s default
libvirt URI, which holds nothing — and concluded the substrate did not exist. In `qemu:///system`
`pve-staging` is defined and running. RFC-0038's measured table and decision 7 were rewritten and
the mistake recorded as an errata section rather than quietly repaired. **A measurement without a
scope is not evidence**, and that is now written into RUN-0005's failure list so the next person
meets it as a documented trap rather than as a discovery.

**Rejected, and why — the sub-ADR record.**

- **Remastering the Windows ISO with the answer file inside it.** One cdrom instead of two, and it
  fails silently: Microsoft ISOs are UDF-primary with a stub ISO9660 layer holding only
  `README.TXT`, and `xorriso` reads ISO9660 only. Remastering would have produced a 380 KB "Windows
  ISO" containing a readme. A second small ISO instead, which is what Windows Setup searches anyway.
- **virtio disks plus the virtio-win driver ISO.** The conventional answer, and it adds a
  third-party dependency, a second cdrom and a driver-injection step. SATA and e1000e are what
  Windows carries natively, and the bottleneck in this run is a residential uplink, not a disk bus.
- **UEFI/OVMF with swtpm.** Available on the host and unnecessary: Windows 10 LTSC has no TPM or
  Secure Boot floor, and OVMF adds an nvram file whose ownership is one more thing to get right on
  a host where three accounts already have to agree about a directory.
- **The dmacvicar provider's XSLT escape hatch.** Written and then deleted: provider 0.9 is a typed
  mirror of the domain XML, so bus and device type are stated directly. The XSLT existed only to
  work around an abstraction that this version does not impose.
- **Putting the drive attachment in the module behind a variable.** Refused on the same reasoning as
  RFC-0038 decision 3: a variable is a thing someone flips while reading only the file it lives in.
  The gated act stays outside the module and is added by hand, once, by someone who has read the RFC.
- **`sudo` for the tooling.** OpenTofu is a single static binary and went into `~/.local/bin`
  checksum-verified. The only root action in the whole build was one `install -d`, and it was run by
  the operator rather than requested as standing access.

## 2026-09-02 — the libvirt NAT network was refused by the host, not by a decision

`libvirt_network` with `forward mode='nat'` defined cleanly and failed to start:
`nft -ae insert rule ip libvirt_network guest_output ... No such file or directory`. libvirt's
nftables backend has no `libvirt_network` table on this host — the concrete form of RFC-0035's
audit row *"Host firewall: none — nftables, iptables, firewalld and ufw all inactive"*, and the
reason the substrate's own network is `forward mode='open'`, which applies no firewall rules at all.

**Rejected: making the firewall work.** It is the correct long-term answer and it needs root, a
decision about the host's firewall posture, and RFC-0035 decision 4 discharged properly. None of
that belongs inside a build step for a disposable guest.

**Rejected: `forward mode='open'` to match the substrate.** It would have started, and it would
have produced a guest with no outbound path — the host does not masquerade — plus the appearance of
a network boundary where there is none.

**Taken: QEMU user-mode networking.** No bridge, no libvirt network object, no nft table, no root,
outbound only, and slirp throughput is far above a residential uplink. It is not isolation — slirp
still routes through the host's stack — and the runbook says so where someone might otherwise
assume it. The resource count for phase 1 dropped from four to three.

## 2026-09-02 — the guest had no ACPI, and three hypotheses died on the way to finding out

**The symptom.** Windows loaded 616 MiB of `boot.wim` and hung, screen full of coloured garbage.
Reproducible to the byte: `sdb rd_bytes` stopped at exactly 646,395,904, and after a reset at
exactly 1,292,791,808 — double, the same load repeated. vCPU 0 sat in 16-bit real mode at
EIP `0x839`, CS `0x2000`, **the identical address across two independent boots**, in bootmgr's
real-mode segment-switch stub.

**The cause.** The domain had no `<features>` element at all — no ACPI, no APIC. **This provider
emits only the features you declare and supplies none of libvirt's usual defaults.** Windows
requires ACPI, so bootmgr got its payload and died at the handoff. libvirt named it only when the
UEFI change was attempted: *"UEFI requires ACPI on this architecture."*

**Three wrong hypotheses, recorded because the sequence is the lesson.**

1. **The video device.** Wrong, and it cost a rebuild. std VGA rendered the same garbage as cirrus,
   and `virsh screenshot` agreed with the monitor's own `screendump` — which is precisely what
   proved the framebuffer content was real rather than mis-captured. **A corrupted console is a
   symptom of a dead guest, not a cause.** The check that settles it is a reset: a clean SeaBIOS
   screen from the same renderer means the capture path works.
2. **The answer file not being read.** `sdc` had one 2 KB read against a 5 KB file, which looked
   like proof Setup never parsed it. It was a consequence — Setup never got far enough to look.
3. **SeaBIOS and the SATA CD-ROM.** Argued from the real-mode hang and the 616 MiB read through
   INT 13h, and genuinely an unusual combination for a Windows install. UEFI is the better platform
   and the switch is kept, but it was not the bug, and the reasoning was a plausible story fitted to
   a symptom rather than a cause identified.

**What would have found it in one step:** reading the generated domain XML for what is *absent*.
The phase-1 instruction in RUN-0005 said to check three things in the XML and all three were
present and correct; none of them was `<features>`. **A generated-XML review that only checks what
you declared cannot find what the generator omitted** — and on a provider that is a thin mirror of
the XML, omission is the whole failure class. The runbook now says to grep for `<features>` first.

**Kept deliberately unchanged: `cpu mode = host-passthrough`.** It was the next suspect and it stays
untouched so that if this boot still fails it is a single variable rather than a confounded one.

## 2026-09-02 — UEFI booted, and two tooling traps cost a cycle each

With ACPI present the guest reached long mode and OVMF, then reported `BdsDxe: No bootable option
or device was found`. The devices were all correctly attached, which made the message misleading:
it was **Microsoft's UEFI boot image prompting *"press any key"* and timing out**, exactly the risk
named when UEFI was chosen. `virsh send-key ... --holdtime 900 KEY_ENTER` in a loop across the boot
window clears it, and it stays reproducible because it is a scripted step rather than a click.

**Trap 1: the monitor's `system_reset` silently did nothing.** It returned success, the screen
looked identical, and the media read counters did not move — which was the only evidence that no
reset had occurred. `virsh reset` worked immediately: `sdb` went from 3.7 MB to 1.78 GB. **A
counter that does not move is the check for whether a command took effect**; a command that returns
success is not.

**Trap 2: a provider bug on list ordering.** `os.firmware_info.features` is sorted internally by the
provider, so declaring `secure-boot` before `enrolled-keys` fails the apply with "Provider produced
inconsistent result after apply". Declared in the provider's order and moved on. Recorded because
the error names a provider bug and the fix is in our file, which is a confusing combination to meet
at speed.

**The domain exists but is not in OpenTofu state** — it was created by the apply that then failed on
trap 2. Nothing was destroyed to fix this while an install was running; the state is reconciled by
`tofu import` once the guest is built, never by destroying a working guest to satisfy a state file.

---

## 2026-09-05 — adopting a diagram renderer → RFC-0040, ADR-0065, ADR-0066, STD-0020, RUN-0006

**What arrived.** A GitHub URL — `tt-a1i/archify` — and the instruction to integrate it across
governance "for slicker visualisation", plus *"maintain a tool list in governance that these are
the stuff we used."* Two requests wearing one sentence, and the second turned out to be the larger.

**How it was worked.** `superpowers:brainstorming` classified it architectural and gated on three
questions before anything was written. The tool was then cloned, run and read — `doctor`, the five
renderer schemas, `package.json`, the git tags — before a line of governance was drafted, because
every claim an artifact makes about a vendor is a claim someone will rely on.

**Four things the investigation changed, recorded because each would have been wrong if inferred:**

1. **Adopting archify naively violates ADR-0040, exactly.** Its `architecture` renderer consumes a
   document declaring components, boundaries and connections — `manifest.yaml`'s model in a second
   file. `docs.no_hand_authored_architecture_model` is keyed on file shape and names `.dsl`,
   `.puml` and `.d2`; a `.architecture.json` is none of them, so the sixth instance of this
   repository's oldest defect would have passed every gate, wearing a tool nobody would think to
   check.
2. **The package is not on npm.** `"private": true`, and its newest tag is `v2.9.0` against a
   shipped version of `2.17.0-dev.1` — **no release tag corresponds to the code anyone would run.**
   A version pin was not available; the pin had to be a commit sha, and the README's own
   `npx skills add` install had to be declined.
3. **The rendered output fetches a font from Google.** Found by grepping the committed HTML, after
   ADR-0065 decision 7 had already been drafted claiming it was self-contained. The diagram works
   offline — the load is async behind `media="print"` — so this is a disclosure rather than a
   rendering failure: opening a governance document would tell a third party someone read it. The
   decision was corrected against the file, and `render-diagram.sh` was written to strip it.
4. **The tool list had nowhere to go.** `toolchain-registry.yaml` states one entry per stack and is
   already carrying seven tools parked under whichever stack needed them first. Everything without
   a stack to hide under — `gh`, `jq`, `python3`, `mermaid` — was declared nowhere at all.

**Rejected, and why — the sub-ADR record.**

- **Superseding ADR-0040 to make room for the tool.** Nothing it decides is wrong, and superseding
  a correct artifact to fit a vendor is the drift ADR-0052 names. It would also have discarded
  decision 1 — *"adding a view is a renderer, never a new source of facts"* — which is the sentence
  that made the adoption available at all.
- **An erratum on ADR-0040.** ADR-0031 reserves errata for a wrong fact. This changes nothing it
  decides, so an erratum was the wrong instrument in the other direction: cheaper, and dishonest.
- **Permitting hand-authored architecture IR "until `forge` ships".** Rejected. A temporary
  permission on a path already walked five times is not temporary, and the cost of refusing is
  visible and bounded — four usable kinds instead of five, recorded as an open question rather than
  softened in the decision.
- **Extending `toolchain-registry.yaml` instead of a new registry.** It would mean inventing a fake
  stack for `gh` and `jq`, or nesting them under an unrelated one, which is what the `tooling:` key
  already is — adopted deliberately rather than by accident.
- **Migrating the seven `tooling:` pins into the new registry.** Full coverage was achieved instead
  with `pin_home` and no version, so no number moved. Changing behaviour while moving it is the
  failure `artifact-registry.yaml` records about ADR-0010's `ci: false`, and it would have made a
  registry's first change also its first regression.
- **Implementing `knowledge.declared_tool_install_is_pinned` in the same change.** It fails on four
  pre-existing unpinned `cargo install` entries. Turning the gate red on four decisions this change
  had no mandate to make was rejected in favour of declaring the check and recording the defect —
  ADR-0014's bargain, TASKS T-2.16.

**One check was declared owed and then earned, in that order.** `docs.diagram_is_current` —
regenerate the HTML and compare — rests on the render being byte-reproducible, which was archify's
claim and not a measured property of it. It was therefore declared against `tool-forge` and left
unimplemented, honestly. The claim was then measured: every diagram re-rendered byte-identically,
and the same IR delivered to two different filenames produced identical bytes, so the output is
path-independent and a temporary-file comparison is valid rather than approximately valid. The
check was written against that evidence. Recorded because the declaration and the measurement are
two different acts, and only the second one earns the `owner` column — which is the rule
`check-registry.yaml`'s header states and the discipline that stops it drifting back into a wish.

**What the gate caught in this work, which is the point of having one.** An RFC with no
alternatives section; a dangling `RUN-0006` reference written before the runbook existed; four new
hardcoded registry counts above the ADR-0035 baseline; three `blocks:` lists naming artifacts that
did not cite the check back; and a literal four-digit artifact id inside a *test fixture* that the
cross-reference scanner correctly read as a dangling reference. Five defects in governance about
enforcement, found by the enforcement.

## 2026-09-16 — `PLAN-THREE-SURFACES.md` verified, found inverted, and rewritten

**What arrived.** A planning document for landing, dashboard and docket-as-a-service, headed
*"Verified 2026-09-16, not remembered"*, resting on: **"Nothing in the estate is deployed. The
`kind-pd-dev` context exists and its API server refuses connections. Every plan below is
downstream of that."** Requested: govern it as far as the skills allow, and make it implementable.

**The correction that mattered.** The premise was false, and the way it was false is the whole
entry. `kind-pd-dev` is a **dead local `kind` context**, not the estate — `go-live.sh`, in this
same checkout since 2026-09-14, warns in a comment: *"Point at the Talos cluster (reached over
NetBird), not whatever dead local context kubectl last used."* `kubectl` was left on the decoy, and
a conclusion about the entire estate was drawn from its refusal to connect.

The estate: five `staging-*` Talos nodes `Ready` for fourteen days, nine Helm releases `deployed`,
and `paridyum.com`, `careers.paridyum.com`, `admin.paridyum.com` and `id.paridyum.com` all
answering `200`. Five of the plan's seven surfaces were already running. Its §3 root ("a cluster
exists and accepts a deploy"), its T1 ("the single highest-value unblock"), its T10 ("deploy
platform-identity — first real deploy") and its §9 ("nothing here has ever run") were all
downstream of one unexamined command.

**The finding underneath the finding, and the reason this is worth a logbook line.** The plan did
not invent "nothing is deployed" — it read it. `web-paridyum/manifest.yaml` says *"nothing is
deployed and no visitor has seen it"*; `web-dashboard/LOGBOOK.md` says *"No cluster is running"*;
`infra-edge/dns/routing-table.yaml` marks most hosts `planned`. **Four independent records agreed
with each other and disagreed with the estate.** The plan inherited the drift by trusting the
record, which is the failure the estate's own rule — *a green check is unproven until you have
watched it fail* — names one level down from where it was written.

**Rejected, and why — the sub-ADR record.**

- **An ADR or RFC for the record-drift pattern.** Well-founded — it generalises ADR-0040's
  generated-view rule to operational state, and the estate has four registries built on the same
  principle. Rejected on the plan's own argument: §5 already carries eleven `proposed` artifacts,
  twelve more sit over thirty days old, and §5's own text says *"a rejection is cheaper than a
  silent default."* Adding a ninth proposed RFC to the pile it criticises is the wrong instrument
  today. Recorded in the plan's F0b for the operator to choose.
- **A hand-authored snapshot of what is deployed.** It is exactly what T-G.8's input needs, and
  writing it as prose would reproduce the defect F0b names — a document asserting runtime state
  that drifts from the runtime. The plan's T-A14 makes it a *read model* instead.
- **Hand-editing `DECISION-DEBT.md`.** Its header says *"do not hand-edit below"*; the entries are
  hook-generated. The findings went into the plan, which is where a reader of this arc will be.
- **Taking F4's reasoning at face value.** It quoted the gate's warning — *"an RFC nobody closed is
  a decision being made by default"* — and attached it to RFC-0045…0051 and ADR-0103. All eight are
  a day old. **The gate's warning is about a different set**: `GLO-0007`, `GLO-0008` and
  `RFC-0014`–`RFC-0023` — which includes RFC-0023 and RFC-0016, the substrate and the edge that
  have been serving production for a fortnight. The plan quoted a warning about the documents it
  had not read, in support of a claim about documents the warning was not about.
- **A new runbook for cluster access.** STD-0011's rule — *correcting an existing artifact beats
  writing a new one* — and RUN-0001 already carries the NetBird material the procedure depends on.
  Written as a dated addendum there rather than as a runbook of its own. *(This bullet named the
  runbook id it declined to create, and the gate's cross-reference scanner read it as a dangling
  reference — correctly. Recorded rather than quietly repaired: a reference to a thing that does
  not exist is indistinguishable from a reference to a thing that was never written.)*

**Where the plan was right, recorded so the revision is not read as a demolition.** F1 (`docket` is
unregistered — verified, and its `kind` is the operator's commercial call), F3 (`product:` absent
on four manifests), the dashboard's not-deployed state, §5's ordering, and the claim that *"the
knowledge gate is at zero errors"* — which the gate confirms, and which is why T-G.4's *"three
errors stand on every run"* line is stale rather than the plan being wrong.

**Two further defects found by auditing rather than by reading.** `tool-docket` carries the same
accepted-decision-vs-disk-name divergence as `console-docket` — its manifest declares
`class: platform` behind a `tool-` prefix — and its manifest cites `DECISION-DEBT.md` **D-58** as
tracking it. And the `knowledge-audit` skill's own naming sweep hardcoded seven prefixes against
a registry that declares more, so it reported every compliant `web-` repository as a C-0002
violation — the hardcoded-list defect, inside the tool that audits for it.

**The first of those two was authored wrong, and caught by the agent sent to fix it.** This entry
said the rename was *"tracked nowhere"*. It was not: **D-64 is the naming entry** and states the
divergence, the resolution and the unscheduled rename almost verbatim as the manifest describes
it. `58` and `64` are a digit transposition. The error's shape is the one this whole session is
about — a `grep` was run, its output was cut off by `head`, and a conclusion about the whole file
was drawn from the visible part. The entry sat twenty lines below the last line read. The agent
refused the instruction to append a new entry, on the correct grounds that duplicating an
existing record of the same debt is worse than the defect it was sent to fix.

**What that exposed, and which no gate can see.** `repo.no_dangling_decision_references` matches
only knowledge artifact-id patterns (`ADR-`, `C-`, `STD-`, …) and explicitly excludes
`DECISION-DEBT.md` — `D-NN` is neither. **A wrong `D-` citation cannot fail any current check**,
so a green run on that repository was never evidence the citation was right. Separately,
`tool-docket`'s vendored snapshot stops at STD-0014, so STD-0022 — the standard governing the
rename D-64 describes — is not readable from that checkout at all.

**What the gate caught in this work.** Three hardcoded registry counts I introduced into the
rewrite (`knowledge.hardcoded_registry_counts`, ADR-0035 D5), taking the estate's total above its
baseline of 48. Removed by naming the sets instead of counting them — which reads better, and is
the rule the check exists to teach. The gate is green: zero errors, 41 warnings.

## 2026-09-27 — the gate that reported `n/a` on itself, and the class it cost four times

`check-repo.sh` and `check-migrations.sh` both derive their target from their own location:
`dirname $0/..`. In this repository that is right. In every other repository they live at
`.paridyum/knowledge/scripts/` — a read-only vendored snapshot (ADR-0001) — so run without `--root`
they inspected **the snapshot** and reported `n/a` for almost everything.

**`n/a` is indistinguishable from a pass to everyone who has ever read one.** In two days it cost:

| Repository | Real errors standing behind a run that said it passed |
| --- | --- |
| `platform-tenancy` | **11** — no CLAUDE.md, no CHANGELOG, no ARCHITECTURE, no API, no RUNBOOK, no TESTING, **no CI at all** |
| `lib-auth-typescript` | **8** — and an empty `.github/workflows/` directory |
| `infra-kubernetes` | **3** — including no CI, in the repository that defines the cluster |
| `infra-terraform` | unmeasured |

`check-migrations.sh`'s own header had already named this class: *"A gate that prints 'passed' over an
empty input set is the defect class the ADR-0014 audit found four times. `n/a` is not a pass."* It was
true of the script's **inputs** and nobody applied it to the script's own **invocation**.

**The fix is a refusal, not a warning.** Both scripts now exit 2 when their own location is a
`.paridyum/knowledge` path and `--root` was not given, and the message names the command that works —
because a reader who now knows something is wrong and not what to type is only slightly better off.

**And the guard immediately found a live bug**, which is the part worth remembering. Grepping for call
sites that would break under it turned up `rust-ci.yml` calling `check-migrations.sh` with no `--root`:
for any repository with a `migrations/` directory that step scanned this snapshot's
`tests/fixtures/migrations/fail-*` — fixtures that exist *in order to fail* — so **it could never have
passed.** Nobody saw it because no repository with migrations had CI until this week.

Both templates had drifted the same way while the live `infra-ci` workflow already passed `--root .`,
so a repository scaffolded from a template inherited the blind spot.

**The ordering is the lesson.** A guard that lands without its call sites fixed breaks CI the moment the
Actions allowance resets. Guard, then grep for what it breaks, then fix that — in the same change.

## 2026-10-07 — a gaming line → RFC-0059, ADR-0133

**What arrived.** The operator: pilot or prototype a few game ideas that may or may not ship, with
naming that does not depend on the engine — Unreal for one title, Bevy for another — under "our
gaming vertical". First asked for a reading of the workspace and where games fit; then, on seeing
the recommendation, to complete the sprint.

**How it was worked.** The router first, then the artifacts it routes to for naming a repository
and proposing a vertical: C-0006, C-0007, C-0010, `class-registry.yaml`, `product-registry.yaml`,
then ADR-0004, ADR-0005, ADR-0006, ADR-0107, ADR-0111, ADR-0112, RFC-0052 and RUN-0001. Three
decisions went to the operator in one structured question — the gaming gate, the class, merging —
and `paridyum-platform:new-decision` wrote both artifacts. The work happened in a worktree off
`origin/main`, because the local `main` checkout was behind `origin/main` by a month of merges and
ahead of it by unpushed checkpoint commits holding only research logs.

**Tried and rejected here, none of it earning an ADR of its own:**

1. **The company route as an exemption.** The first recommendation, in chat, was that a separate
   company "fits better". That is right about the entity and wrong if read as lifting anti-goal 6;
   writing D1 out exposed it. A company changes who signs, not what attention is spent on. RFC-0059
   keeps the correction as a rejected alternative rather than letting the chat version stand.
2. **One pull request per artifact.** RFC-0059 and ADR-0133 cite each other, and `check-knowledge.sh`
   fails a dangling reference as an error, so either split leaves a branch red. They shipped
   together, with the review-budget exception saying why.
3. **Reading "allow for prototyping" as an amendment to C-0006.** Recorded instead as the narrowest
   reading the answer supports: spikes now, the gate untouched. The wider reading would have
   written a constitutional amendment on an inference.

**Found on the way, each queued as its own change:** the checkpoint guard measured the working-tree
size, so it left out exactly the Git LFS assets its own message recommends (ADR-0133 decision 9);
and the session-context hook still matched C-0002's seven prefixes, so every `web-` and `console-`
repository was told at session start that its name broke the scheme, and shown every locked ADR
unfiltered.
