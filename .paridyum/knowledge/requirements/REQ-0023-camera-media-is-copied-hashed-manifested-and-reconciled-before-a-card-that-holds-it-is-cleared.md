---
id: REQ-0023
title: Camera media is copied, hashed, manifested and reconciled before a card that holds it is cleared
type: requirement
status: proposed
created: 2026-09-27
locked_at: null
supersedes: null
superseded_by: null
scope: [org, service, web, tool, infra]
applies_to: ["*"]
tags: [media, custody, checksum, manifest, archive, ingest, sync, vault, rfc-0054, adr-0071, req-0021, adr-0018]
breaking: false
verified_by: []
enforced_by: []
---

# REQ-0023 — Camera media is copied, hashed, manifested and reconciled before a card is cleared

## Status

**ACCEPTED, 2026-09-27** — proposed and revised the same day, then accepted by the operator
alongside RFC-0054. The split is ADR-0018's: **these obligations are about camera media, not about
that design.** If RFC-0054 is rejected or replaced, and the operator still wants shoot media
governed, these bind whatever replaces it — they were accepted on their own terms, not as a rider
to the design's fate.

**Revised 2026-09-27, the same day**, when RFC-0054 changed shape: the design moved from an
operator's tool to a tenant-facing product, and these obligations did not move with it — they are
about media, and they now bind the product per account. Nothing below changed.

`enforced_by` is empty and honest. Nothing that runs in CI can see a camera card, a card reader or
a consumer drive, and `tool-forge` cannot inspect a machine it does not run on — the same limit
REQ-0021 records. **Item 8 is the one item a machine could check** (pre-flight validation of
paths and counts before an upload), and it is the one to build first if any of them ever is. The
rest is enforced by the tool's refusals and by the runbook, which is to say: by compliance, until
something better exists.

## Requirement

> **When camera media is ingested, the source is never written, every byte is hashed and
> manifested before it is trusted, a card is cleared only when the media exists in a verified
> archive and one further governed copy, and the pipeline can always answer what still depends on
> each card.**

The context is one bounded thing: **custody of camera-originated media from card to off-host
store.** It is not itself a capability, it owns no runtime entity, and it gets no repository of
its own — the product that discharges it is named by the design, not by this requirement. What it
owns is the obligation, which is why it is a requirement and not a service.

## Numbered items

> **1 · The source is read-only to the pipeline, in every step, on every device.**

Cards, phones and drives are read; nothing in the pipeline writes, moves, renames or "repairs"
them. Reformatting a card is a deliberate human act performed *outside* the pipeline, and only
after item 6 permits it. The accident this prevents is specific and famous: a host or tool
offering to repair a dirty camera card's filesystem, and returning a card that is clean, empty,
and was never copied.

> **2 · What is copied is enumerated against the card's real listing, never against a pattern.**

*"The DCIM folder"* is an assumption; a card also carries the camera's trash, sidecars, and
whatever the last device left on it. Enumeration is taken from the card as it is, recorded, and
copied from — the same discipline REQ-0021 item 3 applies to egress, and for the same reason: a
plan that assumed five folders met a drive with 41 entries once already.

> **3 · Every byte is hashed as it is read, and the copy is re-hashed and compared before it is
> trusted.**

The write side is not believed; the read-back is the evidence. A mismatch fails the batch loudly
and nothing downstream — no manifest, no clearing, no upload — proceeds from an unverified copy.
The hash algorithm is xxHash64, one the ASC MHL specification names, so the manifest speaks a
standard the industry already reads.

> **4 · Every ingest produces a manifest that binds bytes to card, device and shoot.**

An ASC MHL-compatible copy generation plus the shoot's ledger entry: file count, byte total,
hashes, timestamps, card identity, device identity, operator. Later questions — *is this copy
complete, is this card the one that held this shoot, can this bundle be restored* — are answered
against the manifest, never against memory. A copy without a manifest is an assertion; with one it
is a checkable claim.

> **5 · The archive has one grammar, and it is decided once: date-first paths, a slug that never
> changes, originals preserved in their camera structure, and no person's name in any path
> segment.**

Paths are the one thing about media that cannot be changed later without breaking every manifest
that references them. The no-names rule is ADR-0071 §4's reason turned on media: file and folder
names are metadata a zero-knowledge drive does not necessarily encrypt, and a shoot is where a
name would otherwise land.

> **6 · A card is cleared only when the shoot it held is archived (item 3 passed) and present in
> one further governed place — the off-host store reconciled per item 9, or a second governed
> local store.**

And at every moment the pipeline can answer *what still depends on this card*, per card, before
anything is reformatted. This is the core of this requirement: the ordering is what makes a
wedding's only copy impossible to accidentally reformat. Escrow — a card held longer than its
owner would like — is the accepted cost, stated rather than discovered.

> **7 · Nothing in the pipeline deletes, moves or renames an archived original.**

Deletion is a human act, deliberate and separate; the pipeline's verbs are copy, verify, upload,
reconcile, report. A tool that can delete is a tool that can lose media while reporting success,
and the report is what item 4 exists to make trustworthy.

> **8 · The destination vendor's limits are checked before anything is staged, because their
> failure mode is a silent skip.**

Paths are validated against the vendor's current incompatible-character set, files-per-folder
counts are checked against the hard cap, and a violation fails the batch **before** upload.
A rename is never silent and a skip is never a warning: both are losses that present as successes,
which is the class of failure this requirement exists to abolish. This is the mechanically
checkable item on this list.

> **9 · Completion is reconciliation against the manifest, by a read-back that is not the
> uploader's own status.**

A count and a byte total from the ledger — **and object hashes wherever the destination publishes
them** — compared with a fresh listing of what the destination holds, obtained through a channel
that is not the uploading client's claim. Where only the same vendor's client exists, its listing
export is the floor and an independent read-back is the target. **A transport is not trusted until
one shoot has round-tripped through it end to end and been re-hashed from the far side** — a
backup that has never been restored is a belief, not a control — and it is re-proven after a major
adapter or client update, because silent large-upload failures are a documented property of one
vendor here, not a hypothetical.

> **10 · The pipeline holds no truth it cannot show, and no metadata it does not need.**

Logs carry counts, identifiers and outcomes — never filenames of classified content, never EXIF,
never locations. The archive and its manifests are the answer to *where is every byte*; the logs
are not a second, leakier copy of them. ADR-0080's shape applies to a tool on a laptop exactly as
it applies to a service.

## What this deliberately does not require

**A particular design.** The obligations bind whatever moves camera media; RFC-0054 names a
product — a metadata-only service and the tenants' own stations — and if that design is rejected
or replaced, these bind the replacement instead. What this requirement does not do is specify an
implementation: no repository, no service, no fleet membership is owed to *this* document
(ADR-0018's split).

**A knowledge registry for equipment.** The inventory is state; a governance file declares the
namespace and observed state is measured elsewhere (`device-registry.yaml`'s split). Item 4's
manifests and the agent's inventory carry the list, not a YAML file reviewed like a decision.

**Hashes on the far side of a zero-knowledge store.** Where the destination publishes object
hashes (the S3 class), item 9's reconciliation uses them; where it cannot (a zero-knowledge Sync
store), nothing there can be re-hashed without downloading it. That is why item 9 asks for routine
listing reconciliation and a full read-back rehearsal once per transport — honest about what is
cheap and what is not.

**A standing permission to write to any card or drive.** Item 1 is absolute in the pipeline, and
nothing here authorises a later exception "just for this run".

## Related

- **RFC-0054** — the proposed design that discharges this, and the vendor facts it is built on
- **ADR-0071** — the off-host store, its ciphertext rule, and *"a backup that has never been
  restored is a belief, not a control"*, which item 9 applies to media
- **REQ-0021** — what bulk egress from a governed host owes; items 1, 3, 7, 8 and 10 reuse its
  discipline deliberately rather than restating it
- **RFC-0038**, **RUN-0005** — the read-only-source and enumerated-set precedents, and the
  Windows uploader machinery
- **ADR-0018** — why these obligations are separable from the design
- **RFC-0021**, `device-registry.yaml` — the governance-declares/inventory-records split item 4
  and *What this deliberately does not require* rely on
- **ADR-0080** — classified data has no exit into a log; item 10 is that rule for this pipeline
- **C-0010** — there is no prototype tier; the first governed ingest is the production process at
  one shoot's scale
