---
id: REQ-0021
title: Operator data leaves a governed host read-only, enumerated, and reconciled against the source
type: requirement
status: accepted
created: 2026-08-31
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["*"]
tags: [personal-data, egress, substrate, passthrough, vendor, read-only, rfc-0038, rfc-0035, req-0012]
breaking: false
enforced_by: []
---

# REQ-0021 — Operator data leaves a governed host read-only, enumerated, and reconciled

## Status

**`accepted`, 2026-08-31.** No trigger, because nothing here is waiting: the obligations below apply
the moment bulk operator data moves off a host this estate governs, and such a move is proposed now
in RFC-0038.

`accepted` is about the obligations, not about a schedule and not about RFC-0038's design. **RFC-0038
is `proposed` and its decision 7 needs a human**; if it is rejected or replaced, these requirements
bind whatever replaces it. That is the split ADR-0018 draws — a requirement outlives the design that
happens to discharge it.

`enforced_by` is empty and honest: `tool-forge` cannot inspect a host it does not run on. Item 9 is
the one item here a machine could check, and it is the one to build first if any of them ever is.

## Requirement

> **When bulk operator data leaves a host this estate governs, the source is read-only, what leaves
> is enumerated before it moves, the destination is a governed vendor, and completion is decided by
> reconciliation against the source rather than by the client that did the uploading.**

The context is one bounded thing: **egress of operator-owned bulk data from a governed host.** It is
not a platform capability, it owns no runtime entity, and it gets no repository — the entities it
touches (the volume, the guest, the vendor account) are each already owned elsewhere. What it owns
is the *obligation*, which is why this is a requirement and not a service.

## Numbered items

> **1 · The source volume is attached read-only, and read-only is a property of the attachment.**

Not a convention the reader of a runbook is trusted to hold, and not a promise about how the
uploading software behaves. An attachment that *cannot* write makes a whole class of loss —
filesystem repair on a dirty volume, an offer to format, anything hostile that reaches the guest —
structurally impossible rather than unlikely.

> **2 · Nothing on the source is deleted, moved, or modified by an egress. Ever.**

An egress is not a migration. Reclaim and teardown touch guests and copies only. Stated absolutely
because the one case where it would be tempting — a full source volume needing room — is exactly the
case where losing the wrong bytes is unrecoverable.

> **3 · What leaves is an explicit include/exclude list, decided against the real listing.**

Never *"the folders on the drive"*. A real volume carries recycle bins, recovery partitions, cached
virtual machines, and other people's data; a plan that assumed five folders met a drive with 41
entries (RFC-0038). The enumeration is a human decision recorded before the first byte moves, and
its absence is the defect — not a detail to settle during the run.

> **4 · No credential for the destination is written to an unencrypted volume, and none is baked
> into a machine image.**

RFC-0035 decision 7's ordering argument, and it does not depend on the credential being Paridyum's:
once written to an unencrypted volume, encrypting afterwards does not un-write it. A credential
baked into a template is a *copy* of a live secret sitting in a file for as long as the template
exists.

> **5 · The destination vendor is discharged under REQ-0012 before the first byte, and the exit plan
> is part of that.**

REQ-0012 is `accepted` and fired. *How does the data come back, over how long, at what cost, if the
vendor or the account goes away?* — answered before the upload, because after it the question is
being asked about data that is already gone.

> **6 · A guest performing an egress reaches the destination and nothing else.**

No route to the management tunnel, no route to the LAN, no path to the host. RFC-0035 decision 1's
direction is unchanged by the guest's purpose, and a guest holding bulk personal data has no
business on a management plane in either direction.

> **7 · Concurrency is justified by a measured bottleneck.**

The default is one uploader. Parallel uploaders on one uplink divide it; each additional one costs a
guest, a vendor seat, a credential copy, and another reader of the data. A fan-out with no
measurement behind it is REQ-0016's *"derived from taste"* defect pointed at throughput.

> **8 · Completion is reconciliation against a source inventory, never the uploader's own report.**

A count and a byte total taken from the source, compared with what the destination holds. A client's
"up to date", a log line, a stable folder size and a timeout may all decide *when to look*; none of
them decides *what is true*. Teardown follows reconciliation, never the reverse.

> **9 · An egress guest asserts its own isolation from inside itself.**

STD-0009's shape, as RFC-0035 decision 8 already borrows it: **the denial is the assertion.** A
host-side reading of the domain definition confirms what was intended; only a check running inside
the guest confirms what was achieved. This is the mechanically checkable item on this list.

> **10 · An egress is an operation: it carries a runbook before it runs and a logbook after.**

RFC-0032 and STD-0013 for the runbook, RFC-0033 for the logbook. A bulk egress is rare, slow,
irreversible in its effects on a third party, and touches a boundary — which is precisely the profile
of the work that gets done from memory and then cannot be explained six months later.

## What this deliberately does not require

**A repository, a service, or a place in the fleet manager.** Operator-owned data movement is not a
platform capability, and routing it through `svc-fleet` would make the fleet manager the surface
through which personal chores are done — the beginning of ANTI-0006's shape at the infrastructure
layer. RFC-0038 decision 1 states the same boundary from the other side.

**A permanent permission to mount protected data into a guest.** RFC-0035 decision 8 stands. Any
narrowing of it is a decision a human accepts, bounded and expiring, per RFC-0038 decision 7 — and
this requirement is not a standing authorisation for the next one.

## Related

- **RFC-0038** — the design that discharges this for the first instance, and the measured host facts
  behind items 1, 3, 6 and 7
- **RFC-0035** — decision 8's protected boundary, decision 1's trust direction, decision 7's
  credential ordering
- **REQ-0012** — vendor governance, invoked by item 5
- **REQ-0016** — the "derived from taste" argument item 7 reuses
- **STD-0009** — *"the denial is the assertion"*, the shape of item 9
- **STD-0017** — how the vendor account is named
- **RFC-0032**, **RFC-0033**, **STD-0013** — the runbook and logbook item 10 requires
- **ANTI-0006** — the shape *What this deliberately does not require* refuses
- **ADR-0018** — why the requirement is separable from the design that discharges it
